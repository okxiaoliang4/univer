use axum::{
    extract::DefaultBodyLimit,
    http::Method,
    routing::{delete, get, post},
    Router,
};
use migration::{Migrator, MigratorTrait};
use sea_orm::DatabaseConnection;
use sea_orm::{ConnectOptions, Database};
use socketioxide_redis::{RedisAdapter, RedisAdapterConfig, RedisAdapterCtr};
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info, log::LevelFilter, warn};
use uuid::Uuid;

// No feature flags needed - mimalloc is always used in server
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

// Server modules (now local to this crate - no feature flags!)
mod config;
mod database;
mod grpc;
mod handlers;
mod metrics;
mod services;
mod state;
mod types;

use config::Config;
use handlers::{api, socketio};
use socketioxide::SocketIo;
use state::ServerState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env file if it exists (ignore errors if file doesn't exist)
    if let Err(e) = dotenvy::dotenv() {
        warn!("Failed to load .env file: {}", e);
    } else {
        info!("Loaded .env file");
    }

    // Initialize tracing
    tracing_subscriber::fmt::init();
    info!("Tracing initialized");

    // Initialize OpenTelemetry with Prometheus exporter
    metrics::init_opentelemetry();

    // Initialize process metrics
    metrics::init_process_metrics();

    // Initialize Socket.IO metrics
    metrics::init_socketio_metrics();

    // Initialize all OT collaboration metrics
    metrics::init_all_ot_metrics();

    // Start metrics collection background task
    metrics::start_metrics_collection();

    // Load configuration
    let config = match Config::from_env() {
        config => {
            info!("Configuration loaded successfully");
            config
        }
    };
    config.log_summary();
    info!("Starting OT server on port {}", config.server_port);
    info!(
        "gRPC server will listen on port {}",
        config.grpc_server_port
    );

    // Connect to database
    info!("Connecting to database at {}", config.database_url);
    let mut connect_options = ConnectOptions::new(&config.database_url);
    connect_options
        // Increased from 100 to 200 to handle high concurrency
        // Each OT operation may need a connection for idempotency check + write
        .max_connections(200)
        // Increased from 10 to 20 for faster warmup under load
        .min_connections(20)
        .connect_timeout(Duration::from_secs(10))
        // Reduced from 30s to 5s - if we can't get a connection in 5s, better to fail fast
        // Long waits accumulate and make the situation worse
        .acquire_timeout(Duration::from_secs(5))
        // Increased threshold from 10ms to 50ms to reduce log noise
        // Focus on truly problematic queries (>50ms usually means contention)
        .sqlx_slow_statements_logging_settings(LevelFilter::Warn, Duration::from_millis(50))
        .sqlx_logging_level(LevelFilter::Debug)
        .idle_timeout(Duration::from_secs(600))
        .max_lifetime(Duration::from_secs(1800));

    let db: DatabaseConnection = match Database::connect(connect_options).await {
        Ok(db) => {
            info!("Connected to database successfully");
            db
        }
        Err(e) => {
            error!("Failed to connect to database: {}", e);
            return Err(e.into());
        }
    };

    // Run migrations
    info!("Running database migrations");
    if let Err(e) = Migrator::up(&db, None).await {
        error!("Database migration failed: {}", e);
        return Err(e.into());
    }
    info!("Database migrations completed successfully");


    // Connect to etcd first (needed for auth service discovery)
    info!("Connecting to etcd at {:?}", config.etcd_endpoints);
    let etcd_service = match services::EtcdService::connect(&config.etcd_endpoints).await {
        Ok(service) => {
            info!("Connected to etcd successfully");
            Arc::new(service)
        }
        Err(e) => {
            error!("Failed to connect to etcd: {}", e);
            return Err(e);
        }
    };

    // Initialize gRPC client service for external service calls
    info!("Initializing gRPC client service");
    let grpc_client = Arc::new(services::GrpcClientService::new(
        etcd_service.clone(),
        config.user_rpc_prefix.clone(),
        config.document_rpc_prefix.clone(),
    ));
    info!("gRPC client service initialized");

    // Initialize auth service with gRPC client
    info!("Initializing auth service");
    let auth_service = Arc::new(services::AuthService::new(
        grpc_client.clone(),
        config.permission_cache_ttl_seconds,
        config.skip_permission_check,
    ));
    info!("Auth service initialized");

    // Create application state
    // Note: WriteBehind worker has been moved to a separate crate (writebehind-worker)
    // that runs as an independent process. It will receive notifications via Redis Pub/Sub.
    info!("Creating server state");
    info!(
        "Cache config: ttl_seconds={}, batch_size={}",
        config.writebehind_ttl_seconds,
        config.writebehind_batch_size
    );

    let state = Arc::new(
        ServerState::new(
            db,
            config.s3_endpoint.clone(),
            config.s3_region.clone(),
            config.s3_bucket.clone(),
            config.s3_access_key.clone(),
            config.s3_secret_key.clone(),
            config.server_env.clone(),
            config.redis_url.clone(),
            config.awareness_redis_enabled,
            config.awareness_ttl_seconds,
            etcd_service,
            grpc_client,
            auth_service,
            config.writebehind_ttl_seconds,
            config.writebehind_batch_size,
        )
        .await,
    );
    info!("Server state created successfully");
    // Create Socket.IO layer with state
    let client = redis::Client::open(config.redis_url.clone())?;

    // Configure Redis adapter with increased timeouts and buffers for high load
    // - request_timeout: 15s (from 5s) for cross-instance broadcast under load
    // - stream_buffer: 4096 (from 1024) for handling message bursts
    // - ack_response_buffer: 1024 (from 255) for high-throughput ack responses
    let redis_adapter_config = RedisAdapterConfig::default()
        .with_request_timeout(Duration::from_secs(15))
        .with_stream_buffer(4096)
        .with_ack_response_buffer(1024);
    let adapter = RedisAdapterCtr::new_with_redis_config(&client, redis_adapter_config).await?;

    info!("Initializing Socket.IO layer");
    // Configure Socket.IO with optimized timeouts for production load:
    // - ping_interval: 15s (from 25s) for faster disconnect detection
    // - ping_timeout: 30s (from 20s) more time for client response under load
    // - max_buffer_size: 256 (from 128) for high-throughput broadcasting
    // - ack_timeout: 10s (from 5s) more time for client ack under network latency
    let (layer, io) = SocketIo::builder()
        .ping_interval(Duration::from_secs(15))
        .ping_timeout(Duration::from_secs(30))
        .max_buffer_size(256)
        .ack_timeout(Duration::from_secs(10))
        .with_state(state.clone())
        .with_adapter::<RedisAdapter<_>>(adapter)
        .build_layer();
    info!("Socket.IO layer initialized with optimized config: ping_interval=15s, ping_timeout=30s, max_buffer_size=256, ack_timeout=10s");

    let instance_id = Uuid::new_v4();
    info!("Registering instance {} with etcd", instance_id);
    let registration_ip = config.etcd_registration_ip.clone().unwrap_or_else(|| {
        local_ip_address::local_ip()
            .map(|ip| ip.to_string())
            .unwrap_or_else(|_| {
                warn!("Failed to detect local IP, using 127.0.0.1");
                "127.0.0.1".to_string()
            })
    });
    let endpoint = format!("{}:{}", registration_ip, config.grpc_server_port);
    info!("Registering endpoint: {}", endpoint);
    // etcd_service was moved earlier, so we need to clone or Arc it if necessary.
    let registration = match state
        .etcd_service
        .register_with_lease(
            "ot-collaboration",
            instance_id,
            endpoint.clone(),
            config.etcd_lease_ttl_seconds,
        )
        .await
    {
        Ok(reg) => {
            info!("Successfully registered with etcd: {}", endpoint);
            reg
        }
        Err(e) => {
            error!("Failed to register with etcd: {}", e);
            return Err(e);
        }
    };

    // Setup Socket.IO event handlers
    info!("Setting up Socket.IO event handlers");
    io.ns("/ws", socketio::on_connect).await?;
    info!("Socket.IO event handlers configured");

    // Build application with routes
    info!("Building application routes");
    let app = Router::new()
        .route("/health", get(api::health_check))
        .route("/metrics", get(metrics::metrics_handler))
        // Document CRUD
        .route("/api/documents", post(api::create_document))
        .route(
            "/api/documents/latest-snapshots",
            post(api::get_latest_snapshots),
        )
        .route(
            "/api/documents/{doc_id}",
            get(api::get_document).delete(api::delete_document),
        )
        .route("/api/documents/{doc_id}/clone", post(api::clone_document))
        .route(
            "/api/documents/{doc_id}/restore",
            post(api::restore_document),
        )
        .route("/api/documents/{doc_id}/changeset", post(api::broadcast_op))
        // Snapshots
        .route(
            "/api/documents/{doc_id}/snapshots",
            get(api::get_snapshot_list),
        )
        .route(
            "/api/documents/{doc_id}/snapshots/{snapshot_id}",
            get(api::get_snapshot).post(api::update_snapshot_name),
        )
        // Operations
        .route(
            "/api/documents/{doc_id}/operations",
            get(api::get_operations),
        )
        // Storage
        .route("/api/sign-urls", post(api::sign_object_urls))
        .route(
            "/api/storage/{storage_id}/doc-id",
            get(api::get_doc_id_from_storage_id),
        )
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods([
                    Method::GET,
                    Method::POST,
                    Method::PUT,
                    Method::DELETE,
                    Method::OPTIONS,
                ])
                .allow_headers(Any)
                .expose_headers([axum::http::header::CONTENT_TYPE]),
        )
        .layer(layer)
        .layer(DefaultBodyLimit::max(50 * 1024 * 1024))
        .with_state(state.clone());

    // Start HTTP server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server_port));
    info!("Binding HTTP server to {}", addr);
    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => {
            info!("HTTP server listening on {}", addr);
            listener
        }
        Err(e) => {
            error!("Failed to bind HTTP server to {}: {}", addr, e);
            return Err(e.into());
        }
    };

    // Start gRPC server
    let grpc_addr = SocketAddr::from(([0, 0, 0, 0], config.grpc_server_port));
    info!("Starting gRPC server on {}", grpc_addr);
    let grpc_state = state.clone();
    let grpc_server = tokio::spawn(async move {
        info!("gRPC server listening on {}", grpc_addr);
        match tonic::transport::Server::builder()
            .add_service(grpc::grpc_server(grpc_state.clone()))
            .add_service(grpc::editable_server(grpc_state))
            .add_service(grpc::reflection_server())
            .serve(grpc_addr)
            .await
        {
            Ok(_) => {
                info!("gRPC server stopped gracefully");
                Ok(())
            }
            Err(err) => {
                error!("gRPC server error: {}", err);
                Err(anyhow::anyhow!(err))
            }
        }
    });

    info!("Starting HTTP server");
    if let Err(e) = axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(registration, grpc_server))
        .await
    {
        error!("HTTP server error: {}", e);
        return Err(e.into());
    }

    info!("Server shutdown complete");

    Ok(())
}

async fn shutdown_signal(
    registration: services::etcd::EtcdRegistration,
    grpc_server: tokio::task::JoinHandle<anyhow::Result<()>>,
) {
    info!("Waiting for shutdown signal");
    if let Err(e) = tokio::signal::ctrl_c().await {
        warn!("Failed to wait for shutdown signal: {}", e);
    } else {
        info!("Shutdown signal received");
    }

    info!("Revoking etcd registration");
    registration.revoke().await;
    info!("Etcd registration revoked successfully");

    info!("Aborting gRPC server");
    grpc_server.abort();
    info!("Shutdown complete");
}
