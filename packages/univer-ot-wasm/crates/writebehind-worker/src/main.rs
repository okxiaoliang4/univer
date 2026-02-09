//! WriteBehind Worker - Independent background flush service
//!
//! This service handles asynchronous flushing of cached operations from Redis
//! to PostgreSQL. It runs as a separate process from ot-server.
//!
//! ## Features
//!
//! - Redis Streams with Consumer Groups for reliable, distributed processing
//! - Batch processing for efficiency
//! - XAUTOCLAIM for dead consumer recovery
//! - Health check and Prometheus metrics endpoint
//! - Graceful shutdown with pending flush completion

mod config;
mod metrics;
mod worker;

use crate::config::{Config, WriteBehindConfig};
use crate::worker::WriteBehindWorker;
use anyhow::{Context, Result};
use axum::{routing::get, Router};
use mimalloc::MiMalloc;
use ot_common::{CacheConfig, CacheService, StorageService, StreamQueueService};
use prometheus::{Encoder, TextEncoder};
use sea_orm;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::signal;
use tracing::{error, info, warn};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "writebehind_worker=info,ot_common=info".into()),
        )
        .init();

    info!("WriteBehind Worker starting...");

    // Load configuration
    let config = Config::from_env();
    info!("Configuration loaded");
    info!("  REDIS_URL: {}", config.redis_url);
    info!("  DATABASE_URL: {}", config.database_url);
    info!("  WORKER_COUNT: {}", config.worker_count);
    info!("  FLUSH_INTERVAL_MS: {}", config.flush_interval_ms);
    info!("  INSTANCE_ID: {}", config.instance_id);
    info!("  STREAM_BATCH_SIZE: {}", config.stream_batch_size);

    // Connect to database (no migration - ot-server handles that)
    info!("Connecting to database at {}", config.database_url);
    let mut connect_options = sea_orm::ConnectOptions::new(&config.database_url);
    connect_options
        // Increased from default 100 to 150 for high-throughput batch writes
        // WriteBehind worker handles concurrent stream consumers + batch COPY operations
        .max_connections(150)
        // Increased from default 5 to 10 for faster warmup
        .min_connections(10)
        .connect_timeout(std::time::Duration::from_secs(10))
        // 10s acquire timeout - batch writes can tolerate slightly longer waits
        .acquire_timeout(std::time::Duration::from_secs(10))
        .idle_timeout(std::time::Duration::from_secs(600))
        .max_lifetime(std::time::Duration::from_secs(1800))
        .sqlx_logging(false);

    let db = match sea_orm::Database::connect(connect_options).await {
        Ok(db) => {
            info!("Connected to database successfully");
            Arc::new(db)
        }
        Err(e) => {
            error!("Failed to connect to database: {}", e);
            return Err(e.into());
        }
    };
    info!("Database connection pool: max=150, min=10");

    // Create Redis client
    let redis_client =
        redis::Client::open(config.redis_url.as_str()).context("Failed to create Redis client")?;

    // Create cache service
    let cache_config = CacheConfig {
        ttl_seconds: config.cache_ttl_seconds,
        batch_size: config.batch_size,
    };
    let cache_service = CacheService::new(redis_client.clone(), cache_config)
        .await
        .context("Failed to create CacheService")?;
    let cache_service = Arc::new(cache_service);
    info!("CacheService created");

    // Create stream queue service (reuses CacheService connections)
    let queue_service = Arc::new(StreamQueueService::new(
        cache_service.connection_managers(),
    ));
    info!("StreamQueueService created");

    // Create storage service
    let storage_service = StorageService::new_with_db(
        db.clone(),
        config.s3_endpoint.clone(),
        config.s3_region.clone(),
        config.s3_bucket.clone(),
        config.s3_access_key.clone(),
        config.s3_secret_key.clone(),
        config.server_env.clone(),
        redis_client,
    )
    .await
    .context("Failed to create StorageService")?;
    let storage_service = Arc::new(storage_service);
    info!("StorageService created");

    // Create worker
    let worker_config = WriteBehindConfig::from(&config);
    let worker = Arc::new(WriteBehindWorker::new(
        db.clone(),
        cache_service.clone(),
        queue_service.clone(),
        storage_service.clone(),
        worker_config,
    ));

    // Log stream diagnostics before starting workers
    match queue_service.get_info().await {
        Ok(info) => {
            info!(
                "Stream diagnostics: length={}, pending={}, consumers={}",
                info.stream_length, info.pending_count, info.consumer_count
            );
            if info.stream_length > 0 && info.pending_count == 0 {
                info!(
                    "All {} stream entries have been delivered and ACKed. Waiting for new messages from ot-server.",
                    info.stream_length
                );
            }
        }
        Err(e) => warn!("Failed to get stream diagnostics: {}", e),
    }

    // Start workers (async - creates consumer group first)
    let handles = worker.start().await
        .context("Failed to start write-behind workers")?;
    info!("WriteBehind workers started");

    // Start background task for stream metrics
    let queue_service_metrics = queue_service.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(15)).await;
            if let Ok(info) = queue_service_metrics.get_info().await {
                metrics::set_stream_info(info.stream_length, info.pending_count, info.consumer_count);
                metrics::set_queue_depth(info.pending_count as i64);
            }
        }
    });

    // Start health/metrics server
    let health_port = config.health_port;
    let health_server = tokio::spawn(run_health_server(health_port));
    info!("Health server started on port {}", health_port);

    // Wait for shutdown signal
    info!("WriteBehind Worker ready, waiting for shutdown signal...");
    shutdown_signal(worker.clone(), handles).await;

    // Wait for health server to finish
    let _ = health_server.await;

    info!("WriteBehind Worker shutdown complete");
    Ok(())
}

/// Run health check and metrics HTTP server
async fn run_health_server(port: u16) {
    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/metrics", get(metrics_handler));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8810));
    info!("Health server listening on {}", addr);

    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            error!("Failed to bind health server: {}", e);
            return;
        }
    };

    if let Err(e) = axum::serve(listener, app).await {
        error!("Health server error: {}", e);
    }
}

async fn health_handler() -> &'static str {
    "OK"
}

async fn metrics_handler() -> String {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
}

/// Handle graceful shutdown
async fn shutdown_signal(worker: Arc<WriteBehindWorker>, handles: Vec<tokio::task::JoinHandle<()>>) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl+C, starting graceful shutdown");
        }
        _ = terminate => {
            info!("Received terminate signal, starting graceful shutdown");
        }
    }

    // Signal workers to stop
    worker.shutdown();

    // Wait for workers with timeout
    let timeout = tokio::time::timeout(std::time::Duration::from_secs(30), async {
        for handle in handles {
            let _ = handle.await;
        }
    });

    if timeout.await.is_err() {
        warn!("Worker shutdown timed out after 30 seconds");
    }

    // Final flush
    info!("Performing final flush of pending operations...");
    match worker.flush_all().await {
        Ok(count) => info!("Final flush completed: {} operations", count),
        Err(e) => error!("Final flush failed: {}", e),
    }
}

