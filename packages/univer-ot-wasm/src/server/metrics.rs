//! Metrics collection and Prometheus exporter
//!
//! This module provides OpenTelemetry integration with Prometheus exporter
//! for monitoring server metrics including memory usage and system information.

use axum::response::IntoResponse;
use opentelemetry::{global, KeyValue};
use opentelemetry_sdk::metrics::SdkMeterProvider;
use prometheus::{opts, register_gauge, register_int_gauge, Encoder, Gauge, IntGauge, TextEncoder};
use std::sync::OnceLock;
use tracing::{error, info};

// Static metrics collectors
static PROCESS_MEMORY_RSS: OnceLock<IntGauge> = OnceLock::new();
static PROCESS_MEMORY_VIRTUAL: OnceLock<IntGauge> = OnceLock::new();
static PROCESS_CPU_USAGE: OnceLock<Gauge> = OnceLock::new();

// Socket.IO metrics
static ONLINE_DOCUMENTS: OnceLock<IntGauge> = OnceLock::new();
static ONLINE_USERS: OnceLock<IntGauge> = OnceLock::new();

/// Initialize OpenTelemetry with Prometheus exporter
pub fn init_opentelemetry() {
    info!("Initializing OpenTelemetry with Prometheus exporter");
    let exporter = opentelemetry_prometheus::exporter()
        .with_registry(prometheus::default_registry().clone())
        .build()
        .expect("Failed to build Prometheus exporter");

    let provider = SdkMeterProvider::builder()
        .with_reader(exporter)
        .build();

    global::set_meter_provider(provider);
    info!("OpenTelemetry initialized successfully");
}

/// Initialize process metrics collectors
pub fn init_process_metrics() {
    PROCESS_MEMORY_RSS.get_or_init(|| {
        register_int_gauge!(opts!(
            "process_memory_rss_bytes",
            "Resident memory size in bytes"
        ))
        .expect("Failed to register process_memory_rss_bytes")
    });

    PROCESS_MEMORY_VIRTUAL.get_or_init(|| {
        register_int_gauge!(opts!(
            "process_memory_virtual_bytes",
            "Virtual memory size in bytes"
        ))
        .expect("Failed to register process_memory_virtual_bytes")
    });

    PROCESS_CPU_USAGE.get_or_init(|| {
        register_gauge!(opts!(
            "process_cpu_usage_ratio",
            "CPU usage ratio (0.0 to 1.0 per core)"
        ))
        .expect("Failed to register process_cpu_usage_ratio")
    });

    info!("Process metrics initialized");
}

/// Initialize Socket.IO metrics collectors
pub fn init_socketio_metrics() {
    ONLINE_DOCUMENTS.get_or_init(|| {
        register_int_gauge!(opts!(
            "socketio_online_documents_total",
            "Number of online documents with active connections"
        ))
        .expect("Failed to register socketio_online_documents_total")
    });

    ONLINE_USERS.get_or_init(|| {
        register_int_gauge!(opts!(
            "socketio_online_users_total",
            "Number of online users (connected sockets)"
        ))
        .expect("Failed to register socketio_online_users_total")
    });

    info!("Socket.IO metrics initialized");
}

/// Increment the online documents counter
pub fn increment_online_documents() {
    if let Some(gauge) = ONLINE_DOCUMENTS.get() {
        gauge.inc();
    }
}

/// Decrement the online documents counter
pub fn decrement_online_documents() {
    if let Some(gauge) = ONLINE_DOCUMENTS.get() {
        gauge.dec();
    }
}

/// Set the online documents counter to a specific value
pub fn set_online_documents(count: i64) {
    if let Some(gauge) = ONLINE_DOCUMENTS.get() {
        gauge.set(count);
    }
}

/// Increment the online users counter
pub fn increment_online_users() {
    if let Some(gauge) = ONLINE_USERS.get() {
        gauge.inc();
    }
}

/// Decrement the online users counter
pub fn decrement_online_users() {
    if let Some(gauge) = ONLINE_USERS.get() {
        gauge.dec();
    }
}

/// Set the online users counter to a specific value
pub fn set_online_users(count: i64) {
    if let Some(gauge) = ONLINE_USERS.get() {
        gauge.set(count);
    }
}

/// Update process metrics by reading from system APIs
fn update_process_metrics() {
    // Get memory info using procfs or system APIs
    #[cfg(target_os = "linux")]
    {
        if let Ok(statm) = std::fs::read_to_string("/proc/self/statm") {
            let parts: Vec<&str> = statm.split_whitespace().collect();
            if parts.len() >= 2 {
                let page_size = 4096; // Standard page size on Linux
                if let Ok(vsize) = parts[0].parse::<i64>() {
                    PROCESS_MEMORY_VIRTUAL.get().unwrap().set(vsize * page_size);
                }
                if let Ok(rss) = parts[1].parse::<i64>() {
                    PROCESS_MEMORY_RSS.get().unwrap().set(rss * page_size);
                }
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        // On macOS, we can use task_info or read from system APIs
        // For simplicity, we'll use a basic implementation
        use std::process::Command;
        if let Ok(output) = Command::new("ps")
            .args(["-p", &std::process::id().to_string(), "-o", "rss=,vsz="])
            .output()
        {
            if let Ok(output_str) = String::from_utf8(output.stdout) {
                let parts: Vec<&str> = output_str.trim().split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(rss) = parts[0].parse::<i64>() {
                        PROCESS_MEMORY_RSS.get().unwrap().set(rss * 1024); // ps returns KB
                    }
                    if let Ok(vsz) = parts[1].parse::<i64>() {
                        PROCESS_MEMORY_VIRTUAL.get().unwrap().set(vsz * 1024); // ps returns KB
                    }
                }
            }
        }
    }
}

/// Start metrics collection background task
///
/// This spawns a tokio task that updates process metrics every 15 seconds
pub fn start_metrics_collection() {
    tokio::spawn(async {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(15));
        loop {
            interval.tick().await;
            update_process_metrics();
        }
    });
    info!("Metrics collection task started");
}

/// Handler for Prometheus metrics endpoint
///
/// Returns all collected metrics in Prometheus text format
pub async fn metrics_handler() -> impl IntoResponse {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = vec![];

    if let Err(e) = encoder.encode(&metric_families, &mut buffer) {
        error!("Failed to encode metrics: {}", e);
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to encode metrics".to_string(),
        )
            .into_response();
    }

    match String::from_utf8(buffer) {
        Ok(metrics) => (
            axum::http::StatusCode::OK,
            [(
                axum::http::header::CONTENT_TYPE,
                "text/plain; version=0.0.4",
            )],
            metrics,
        )
            .into_response(),
        Err(e) => {
            error!("Failed to convert metrics to string: {}", e);
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to convert metrics to string".to_string(),
            )
                .into_response()
        }
    }
}
