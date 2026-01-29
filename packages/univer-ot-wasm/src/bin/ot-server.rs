use axum::{
    http::Method,
    routing::{get, post},
    Router,
};
use migration::{Migrator, MigratorTrait};
use sea_orm::Database;
use sea_orm::DatabaseConnection;
use std::{net::SocketAddr, sync::Arc};
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info, warn, Level};

#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

// Include modules that server code depends on
#[path = "../mutations/mod.rs"]
mod mutations;
#[path = "../transform/mod.rs"]
mod transform;
#[path = "../types.rs"]
mod types;
#[path = "../utils.rs"]
mod utils;

// Re-export server modules for binary
#[path = "../server/mod.rs"]
mod server;

use server::{
    config::Config,
    handlers::{api, socketio},
    metrics,
    state::ServerState,
};
use socketioxide::SocketIo;
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env file if it exists (ignore errors if file doesn't exist)
    if let Err(e) = dotenvy::dotenv() {
        warn!("Failed to load .env file: {}", e);
    } else {
        info!("Loaded .env file");
    }

    // Initialize tracing
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();
    info!("Tracing initialized");

    // Initialize OpenTelemetry with Prometheus exporter
    metrics::init_opentelemetry();

    // Initialize process metrics
    metrics::init_process_metrics();

    // Initialize Socket.IO metrics
    metrics::init_socketio_metrics();

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
    info!("gRPC server will listen on port {}", config.grpc_server_port);

    // Connect to database
    info!("Connecting to database at {}", config.database_url);
    let db: DatabaseConnection = match Database::connect(&config.database_url).await {
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

    // Create Socket.IO layer
    info!("Initializing Socket.IO layer");
    let (layer, io) = SocketIo::new_layer();
    info!("Socket.IO layer initialized");

    // Create application state
    info!("Creating server state");
    let state = match ServerState::new(
        db,
        config.snapshot_interval,
        config.s3_endpoint.clone(),
        config.s3_region.clone(),
        config.s3_bucket.clone(),
        config.s3_access_key.clone(),
        config.s3_secret_key.clone(),
            config.server_env.clone(),
        config.redis_url.clone(),
        config.awareness_redis_enabled,
        config.awareness_ttl_seconds,
        config.etcd_endpoints.clone(),
        io.clone(),
    )
    .await
    {
        state => {
            info!("Server state created successfully");
            Arc::new(state)
        }
    };

    let etcd_service = state.etcd_service.clone();
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
    let registration = match etcd_service
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
    socketio::setup_socketio(&io, state.clone());
    info!("Socket.IO event handlers configured");

    // Build application with routes
    info!("Building application routes");
    let app = Router::new()
        .route("/health", get(api::health_check))
        .route("/metrics", get(metrics::metrics_handler))
        .route("/api/documents", post(api::create_document))
        .route("/api/documents/{doc_id}", get(api::get_document))
        .route("/api/documents/{doc_id}/restore", post(api::restore_document))
        .route("/api/documents/{doc_id}/snapshots", get(api::get_snapshot_list))
        .route(
            "/api/documents/{doc_id}/snapshot",
            post(api::update_snapshot),
        )
        .route(
            "/api/documents/{doc_id}/snapshots/{snapshot_id}",
            post(api::update_snapshot_name),
        )
        .route(
            "/api/documents/{doc_id}/operations",
            get(api::get_operations),
        )
        .layer(
            ServiceBuilder::new()
                // CORS layer must be before Socket.IO layer to handle /socket.io/ requests
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
                .layer(layer),
        )
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
            .add_service(server::grpc::grpc_server(grpc_state.clone()))
            .add_service(server::grpc::editable_server(grpc_state))
            .add_service(server::grpc::reflection_server())
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
    registration: server::services::etcd::EtcdRegistration,
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
