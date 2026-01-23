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
use tracing::{info, Level};

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
    state::ServerState,
};
use socketioxide::SocketIo;
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env file if it exists (ignore errors if file doesn't exist)
    let _ = dotenvy::dotenv();

    // Initialize tracing
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    // Load configuration
    let config = Config::from_env();
    info!("Starting OT server on port {}", config.server_port);

    // Connect to database
    let db: DatabaseConnection = Database::connect(&config.database_url).await?;
    info!("Connected to database");

    // Run migrations
    Migrator::up(&db, None).await?;
    info!("Database migrations completed");

    // Create Socket.IO layer
    let (layer, io) = SocketIo::new_layer();

    // Create application state
    let state = Arc::new(
        ServerState::new(
            db,
            config.snapshot_interval,
            config.s3_endpoint.clone(),
            config.s3_region.clone(),
            config.s3_bucket.clone(),
            config.s3_access_key.clone(),
            config.s3_secret_key.clone(),
            config.redis_url.clone(),
            config.awareness_redis_enabled,
            config.awareness_ttl_seconds,
            config.etcd_endpoints.clone(),
            io.clone(),
        )
        .await,
    );

    let etcd_service = state.etcd_service.clone();
    let instance_id = Uuid::new_v4();
    let local_ip = local_ip_address::local_ip()
        .map(|ip| ip.to_string())
        .unwrap_or_else(|_| "127.0.0.1".to_string());
    let endpoint = format!("{}:{}", local_ip, config.grpc_server_port);
    let registration = etcd_service
        .register_with_lease(
            "ot-collaboration",
            instance_id,
            endpoint,
            config.etcd_lease_ttl_seconds,
        )
        .await?;

    // Setup Socket.IO event handlers
    socketio::setup_socketio(&io, state.clone());

    // Build application with routes
    let app = Router::new()
        .route("/health", get(api::health_check))
        .route("/api/documents", post(api::create_document))
        .route("/api/documents/{doc_id}", get(api::get_document))
        .route(
            "/api/documents/{doc_id}/snapshot",
            post(api::update_snapshot),
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
    info!("HTTP server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;

    // Start gRPC server
    let grpc_addr = SocketAddr::from(([0, 0, 0, 0], config.grpc_server_port));
    let grpc_state = state.clone();
    let grpc_server = tokio::spawn(async move {
        info!("gRPC server listening on {}", grpc_addr);
        tonic::transport::Server::builder()
            .add_service(server::grpc::grpc_server(grpc_state))
            .serve(grpc_addr)
            .await
            .map_err(|err| anyhow::anyhow!(err))
    });

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(registration, grpc_server))
        .await?;

    Ok(())
}

async fn shutdown_signal(
    registration: server::services::etcd::EtcdRegistration,
    grpc_server: tokio::task::JoinHandle<anyhow::Result<()>>,
) {
    let _ = tokio::signal::ctrl_c().await;
    registration.revoke().await;
    grpc_server.abort();
}
