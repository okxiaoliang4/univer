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

    // Create application state
    let state = Arc::new(ServerState::new(
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
    ));

    // Create Socket.IO layer
    let (layer, io) = SocketIo::new_layer();

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

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server_port));
    info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
