//! Metrics collection and Prometheus exporter
//!
//! This module provides OpenTelemetry integration with Prometheus exporter
//! for monitoring server metrics including memory usage, system information,
//! and OT (Operational Transform) collaboration metrics.
//!
//! ## Metric Categories
//!
//! ### Core Performance Metrics
//! - Operation latency (end-to-end, transform, broadcast, RTT)
//! - Throughput (ops/sec, operations per document)
//! - System resources (CPU, memory, WebSocket connections)
//!
//! ### Collaboration Metrics
//! - Conflict resolution (conflicts, transform complexity, retries)
//! - Consistency (sync events, version drift)
//! - Session metrics (active sessions, users per session)

use axum::response::IntoResponse;
use opentelemetry::{global, KeyValue};
use opentelemetry_sdk::metrics::SdkMeterProvider;
use prometheus::{
    exponential_buckets, histogram_opts, opts, register_gauge, register_histogram,
    register_int_counter, register_int_gauge, Encoder, Gauge, Histogram, IntCounter, IntGauge,
    TextEncoder,
};
use std::sync::{Mutex, OnceLock};
use tracing::{error, info};

// Static metrics collectors
static PROCESS_MEMORY_RSS: OnceLock<IntGauge> = OnceLock::new();
static PROCESS_MEMORY_VIRTUAL: OnceLock<IntGauge> = OnceLock::new();
static PROCESS_MEMORY_HEAP: OnceLock<IntGauge> = OnceLock::new();
static PROCESS_CPU_USAGE: OnceLock<Gauge> = OnceLock::new();

// CPU tracking state (for calculating CPU percentage between measurements)
struct CpuState {
    last_utime: u64,
    last_stime: u64,
    last_timestamp: std::time::Instant,
}

static CPU_STATE: OnceLock<Mutex<CpuState>> = OnceLock::new();

// Socket.IO metrics
static ONLINE_DOCUMENTS: OnceLock<IntGauge> = OnceLock::new();
static ONLINE_USERS: OnceLock<IntGauge> = OnceLock::new();

// ============================================================================
// OT Core Performance Metrics
// ============================================================================

// Latency metrics (Histograms for distribution analysis)
/// End-to-end latency from operation receive to application complete
static OT_OPERATION_E2E_LATENCY: OnceLock<Histogram> = OnceLock::new();
/// Time spent on operation transformation
static OT_TRANSFORM_LATENCY: OnceLock<Histogram> = OnceLock::new();
/// Time to broadcast operation to all clients
static OT_BROADCAST_LATENCY: OnceLock<Histogram> = OnceLock::new();
/// Client-server round trip time
static OT_CLIENT_RTT: OnceLock<Histogram> = OnceLock::new();

// Throughput metrics (Counters for cumulative values)
/// Total number of operations processed
static OT_OPERATIONS_TOTAL: OnceLock<IntCounter> = OnceLock::new();
/// Total number of operations processed successfully
static OT_OPERATIONS_SUCCESS: OnceLock<IntCounter> = OnceLock::new();
/// Total number of operations that failed
static OT_OPERATIONS_FAILED: OnceLock<IntCounter> = OnceLock::new();

// Connection metrics (Gauges for current state)
/// Current number of active WebSocket connections
static OT_WEBSOCKET_CONNECTIONS: OnceLock<IntGauge> = OnceLock::new();
/// Peak WebSocket connections (high water mark)
static OT_WEBSOCKET_CONNECTIONS_PEAK: OnceLock<IntGauge> = OnceLock::new();

// ============================================================================
// OT Lock Metrics
// ============================================================================

/// Total number of lock acquisition timeouts
static OT_LOCK_TIMEOUTS: OnceLock<IntCounter> = OnceLock::new();
/// Histogram of lock acquisition retry counts
static OT_LOCK_RETRIES: OnceLock<Histogram> = OnceLock::new();
/// Histogram of lock wait time in seconds
static OT_LOCK_WAIT_TIME: OnceLock<Histogram> = OnceLock::new();
/// Histogram of lock hold time in seconds
static OT_LOCK_HOLD_TIME: OnceLock<Histogram> = OnceLock::new();

// ============================================================================
// OT Conflict Resolution Metrics
// ============================================================================

/// Total number of conflicts detected
static OT_CONFLICTS_TOTAL: OnceLock<IntCounter> = OnceLock::new();
/// Total number of conflicts resolved successfully
static OT_CONFLICTS_RESOLVED: OnceLock<IntCounter> = OnceLock::new();
/// Number of transform retries due to conflicts
static OT_TRANSFORM_RETRIES: OnceLock<IntCounter> = OnceLock::new();
/// Histogram of transform complexity (nesting depth)
static OT_TRANSFORM_COMPLEXITY: OnceLock<Histogram> = OnceLock::new();

// ============================================================================
// OT Consistency Metrics
// ============================================================================

/// Total number of state inconsistencies detected
static OT_INCONSISTENCIES_DETECTED: OnceLock<IntCounter> = OnceLock::new();
/// Total number of state synchronizations performed
static OT_STATE_SYNCS: OnceLock<IntCounter> = OnceLock::new();
/// Total number of state repairs performed
static OT_STATE_REPAIRS: OnceLock<IntCounter> = OnceLock::new();
/// Histogram of version vector drift (difference between client and server versions)
static OT_VERSION_DRIFT: OnceLock<Histogram> = OnceLock::new();
/// Total number of changesets rejected due to excessive version drift
static OT_VERSION_DRIFT_REJECTIONS: OnceLock<IntCounter> = OnceLock::new();
/// Total number of version reconciliations (cache miss with pending ops)
static OT_VERSION_RECONCILIATIONS: OnceLock<IntCounter> = OnceLock::new();

// ============================================================================
// OT Session Metrics
// ============================================================================

/// Current number of active collaboration sessions
static OT_ACTIVE_SESSIONS: OnceLock<IntGauge> = OnceLock::new();
/// Total number of sessions created
static OT_SESSIONS_CREATED: OnceLock<IntCounter> = OnceLock::new();
/// Total number of sessions closed
static OT_SESSIONS_CLOSED: OnceLock<IntCounter> = OnceLock::new();
/// Histogram of session duration in seconds
static OT_SESSION_DURATION: OnceLock<Histogram> = OnceLock::new();
/// Total number of user joins
static OT_USER_JOINS: OnceLock<IntCounter> = OnceLock::new();
/// Total number of user leaves
static OT_USER_LEAVES: OnceLock<IntCounter> = OnceLock::new();
/// Histogram of users per session
static OT_USERS_PER_SESSION: OnceLock<Histogram> = OnceLock::new();

// ============================================================================
// OT Document Metrics
// ============================================================================

/// Histogram of operations per document (operation frequency)
static OT_OPS_PER_DOCUMENT: OnceLock<Histogram> = OnceLock::new();
/// Histogram of document size in bytes
static OT_DOCUMENT_SIZE: OnceLock<Histogram> = OnceLock::new();
/// Total number of document snapshots created
static OT_SNAPSHOTS_CREATED: OnceLock<IntCounter> = OnceLock::new();

// ============================================================================
// Write-Behind Caching Metrics
// ============================================================================

/// Total number of operations buffered in write-behind cache
static WRITEBEHIND_OPS_BUFFERED: OnceLock<IntCounter> = OnceLock::new();
/// Total number of operations flushed from cache to database
static WRITEBEHIND_OPS_FLUSHED: OnceLock<IntCounter> = OnceLock::new();
/// Histogram of flush latency in seconds
static WRITEBEHIND_FLUSH_LATENCY: OnceLock<Histogram> = OnceLock::new();
/// Current queue depth (pending documents to flush)
static WRITEBEHIND_QUEUE_DEPTH: OnceLock<IntGauge> = OnceLock::new();
/// Total number of cache hits
static WRITEBEHIND_CACHE_HITS: OnceLock<IntCounter> = OnceLock::new();
/// Total number of cache misses
static WRITEBEHIND_CACHE_MISSES: OnceLock<IntCounter> = OnceLock::new();
/// Total number of fallbacks to synchronous DB write
static WRITEBEHIND_FALLBACKS: OnceLock<IntCounter> = OnceLock::new();
/// Total number of skipped flushes (due to lock contention)
static WRITEBEHIND_SKIPPED: OnceLock<IntCounter> = OnceLock::new();

// ============================================================================
// Redis Resilience Metrics
// ============================================================================

/// Total number of Redis operation retries
static REDIS_RETRIES_TOTAL: OnceLock<IntCounter> = OnceLock::new();
/// Total number of Redis operation timeouts
static REDIS_TIMEOUTS_TOTAL: OnceLock<IntCounter> = OnceLock::new();

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

    PROCESS_MEMORY_HEAP.get_or_init(|| {
        register_int_gauge!(opts!(
            "process_memory_heap_bytes",
            "Heap memory size in bytes"
        ))
        .expect("Failed to register process_memory_heap_bytes")
    });

    PROCESS_CPU_USAGE.get_or_init(|| {
        register_gauge!(opts!(
            "process_cpu_usage_ratio",
            "CPU usage ratio (0.0 to 1.0 per core)"
        ))
        .expect("Failed to register process_cpu_usage_ratio")
    });

    // Initialize CPU state
    CPU_STATE.get_or_init(|| {
        Mutex::new(CpuState {
            last_utime: 0,
            last_stime: 0,
            last_timestamp: std::time::Instant::now(),
        })
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

/// Initialize OT core performance metrics
pub fn init_ot_performance_metrics() {
    // Latency histograms with exponential buckets from 1ms to 10s
    // buckets: 0.001, 0.002, 0.004, 0.008, 0.016, 0.032, 0.064, 0.128, 0.256, 0.512, 1.024, 2.048, 4.096, 8.192
    let latency_buckets =
        exponential_buckets(0.001, 2.0, 14).expect("Failed to create latency buckets");

    OT_OPERATION_E2E_LATENCY.get_or_init(|| {
        register_histogram!(histogram_opts!(
            "ot_operation_e2e_latency_seconds",
            "End-to-end latency from operation receive to application complete",
            latency_buckets.clone()
        ))
        .expect("Failed to register ot_operation_e2e_latency_seconds")
    });

    OT_TRANSFORM_LATENCY.get_or_init(|| {
        register_histogram!(histogram_opts!(
            "ot_transform_latency_seconds",
            "Time spent on operation transformation",
            latency_buckets.clone()
        ))
        .expect("Failed to register ot_transform_latency_seconds")
    });

    OT_BROADCAST_LATENCY.get_or_init(|| {
        register_histogram!(histogram_opts!(
            "ot_broadcast_latency_seconds",
            "Time to broadcast operation to all clients",
            latency_buckets.clone()
        ))
        .expect("Failed to register ot_broadcast_latency_seconds")
    });

    OT_CLIENT_RTT.get_or_init(|| {
        register_histogram!(histogram_opts!(
            "ot_client_rtt_seconds",
            "Client-server round trip time",
            latency_buckets.clone()
        ))
        .expect("Failed to register ot_client_rtt_seconds")
    });

    // Throughput counters
    OT_OPERATIONS_TOTAL.get_or_init(|| {
        register_int_counter!(opts!(
            "ot_operations_total",
            "Total number of operations processed"
        ))
        .expect("Failed to register ot_operations_total")
    });

    OT_OPERATIONS_SUCCESS.get_or_init(|| {
        register_int_counter!(opts!(
            "ot_operations_success_total",
            "Total number of operations processed successfully"
        ))
        .expect("Failed to register ot_operations_success_total")
    });

    OT_OPERATIONS_FAILED.get_or_init(|| {
        register_int_counter!(opts!(
            "ot_operations_failed_total",
            "Total number of operations that failed"
        ))
        .expect("Failed to register ot_operations_failed_total")
    });

    // Connection gauges
    OT_WEBSOCKET_CONNECTIONS.get_or_init(|| {
        register_int_gauge!(opts!(
            "ot_websocket_connections",
            "Current number of active WebSocket connections"
        ))
        .expect("Failed to register ot_websocket_connections")
    });

    OT_WEBSOCKET_CONNECTIONS_PEAK.get_or_init(|| {
        register_int_gauge!(opts!(
            "ot_websocket_connections_peak",
            "Peak WebSocket connections (high water mark)"
        ))
        .expect("Failed to register ot_websocket_connections_peak")
    });

    info!("OT performance metrics initialized");
}

/// Initialize OT lock metrics
pub fn init_ot_lock_metrics() {
    OT_LOCK_TIMEOUTS.get_or_init(|| {
        register_int_counter!(opts!(
            "ot_lock_timeouts_total",
            "Total number of lock acquisition timeouts"
        ))
        .expect("Failed to register ot_lock_timeouts_total")
    });

    // Lock retries histogram: buckets from 1 to 50 retries
    let retry_buckets = vec![1.0, 2.0, 5.0, 10.0, 20.0, 30.0, 50.0];
    OT_LOCK_RETRIES.get_or_init(|| {
        register_histogram!(histogram_opts!(
            "ot_lock_retries",
            "Number of lock acquisition retries",
            retry_buckets
        ))
        .expect("Failed to register ot_lock_retries")
    });

    // Lock wait time histogram: buckets from 1ms to 10s
    let wait_time_buckets = exponential_buckets(0.001, 2.0, 14)
        .expect("Failed to create lock wait time buckets");
    OT_LOCK_WAIT_TIME.get_or_init(|| {
        register_histogram!(histogram_opts!(
            "ot_lock_wait_time_seconds",
            "Time spent waiting to acquire lock",
            wait_time_buckets.clone()
        ))
        .expect("Failed to register ot_lock_wait_time_seconds")
    });

    // Lock hold time histogram: buckets from 1ms to 30s
    OT_LOCK_HOLD_TIME.get_or_init(|| {
        register_histogram!(histogram_opts!(
            "ot_lock_hold_time_seconds",
            "Time lock was held",
            wait_time_buckets
        ))
        .expect("Failed to register ot_lock_hold_time_seconds")
    });

    info!("OT lock metrics initialized");
}

/// Initialize OT conflict resolution metrics
pub fn init_ot_conflict_metrics() {
    OT_CONFLICTS_TOTAL.get_or_init(|| {
        register_int_counter!(opts!(
            "ot_conflicts_total",
            "Total number of conflicts detected"
        ))
        .expect("Failed to register ot_conflicts_total")
    });

    OT_CONFLICTS_RESOLVED.get_or_init(|| {
        register_int_counter!(opts!(
            "ot_conflicts_resolved_total",
            "Total number of conflicts resolved successfully"
        ))
        .expect("Failed to register ot_conflicts_resolved_total")
    });

    OT_TRANSFORM_RETRIES.get_or_init(|| {
        register_int_counter!(opts!(
            "ot_transform_retries_total",
            "Number of transform retries due to conflicts"
        ))
        .expect("Failed to register ot_transform_retries_total")
    });

    // Transform complexity histogram: buckets from 1 to 64 nesting levels
    let complexity_buckets = vec![1.0, 2.0, 4.0, 8.0, 16.0, 32.0, 64.0];
    OT_TRANSFORM_COMPLEXITY.get_or_init(|| {
        register_histogram!(histogram_opts!(
            "ot_transform_complexity",
            "Transform complexity (nesting depth)",
            complexity_buckets
        ))
        .expect("Failed to register ot_transform_complexity")
    });

    info!("OT conflict metrics initialized");
}

/// Initialize OT consistency metrics
pub fn init_ot_consistency_metrics() {
    OT_INCONSISTENCIES_DETECTED.get_or_init(|| {
        register_int_counter!(opts!(
            "ot_inconsistencies_detected_total",
            "Total number of state inconsistencies detected"
        ))
        .expect("Failed to register ot_inconsistencies_detected_total")
    });

    OT_STATE_SYNCS.get_or_init(|| {
        register_int_counter!(opts!(
            "ot_state_syncs_total",
            "Total number of state synchronizations performed"
        ))
        .expect("Failed to register ot_state_syncs_total")
    });

    OT_STATE_REPAIRS.get_or_init(|| {
        register_int_counter!(opts!(
            "ot_state_repairs_total",
            "Total number of state repairs performed"
        ))
        .expect("Failed to register ot_state_repairs_total")
    });

    // Version drift histogram: buckets from 1 to 1000 version difference
    let drift_buckets = vec![1.0, 5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1000.0];
    OT_VERSION_DRIFT.get_or_init(|| {
        register_histogram!(histogram_opts!(
            "ot_version_drift",
            "Version vector drift (difference between client and server versions)",
            drift_buckets
        ))
        .expect("Failed to register ot_version_drift")
    });

    OT_VERSION_DRIFT_REJECTIONS.get_or_init(|| {
        register_int_counter!(opts!(
            "ot_version_drift_rejections_total",
            "Total number of changesets rejected due to excessive version drift"
        ))
        .expect("Failed to register ot_version_drift_rejections_total")
    });

    OT_VERSION_RECONCILIATIONS.get_or_init(|| {
        register_int_counter!(opts!(
            "ot_version_reconciliations_total",
            "Total number of version reconciliations (cache miss with pending ops)"
        ))
        .expect("Failed to register ot_version_reconciliations_total")
    });

    info!("OT consistency metrics initialized");
}

/// Initialize OT session metrics
pub fn init_ot_session_metrics() {
    OT_ACTIVE_SESSIONS.get_or_init(|| {
        register_int_gauge!(opts!(
            "ot_active_sessions",
            "Current number of active collaboration sessions"
        ))
        .expect("Failed to register ot_active_sessions")
    });

    OT_SESSIONS_CREATED.get_or_init(|| {
        register_int_counter!(opts!(
            "ot_sessions_created_total",
            "Total number of sessions created"
        ))
        .expect("Failed to register ot_sessions_created_total")
    });

    OT_SESSIONS_CLOSED.get_or_init(|| {
        register_int_counter!(opts!(
            "ot_sessions_closed_total",
            "Total number of sessions closed"
        ))
        .expect("Failed to register ot_sessions_closed_total")
    });

    // Session duration histogram: buckets from 1 second to 24 hours
    let duration_buckets = vec![
        1.0, 5.0, 15.0, 30.0, 60.0, 300.0, 600.0, 1800.0, 3600.0, 7200.0, 14400.0, 28800.0, 86400.0,
    ];
    OT_SESSION_DURATION.get_or_init(|| {
        register_histogram!(histogram_opts!(
            "ot_session_duration_seconds",
            "Session duration in seconds",
            duration_buckets
        ))
        .expect("Failed to register ot_session_duration_seconds")
    });

    OT_USER_JOINS.get_or_init(|| {
        register_int_counter!(opts!("ot_user_joins_total", "Total number of user joins"))
            .expect("Failed to register ot_user_joins_total")
    });

    OT_USER_LEAVES.get_or_init(|| {
        register_int_counter!(opts!("ot_user_leaves_total", "Total number of user leaves"))
            .expect("Failed to register ot_user_leaves_total")
    });

    // Users per session histogram: buckets from 1 to 100 users
    let users_buckets = vec![1.0, 2.0, 5.0, 10.0, 20.0, 50.0, 100.0];
    OT_USERS_PER_SESSION.get_or_init(|| {
        register_histogram!(histogram_opts!(
            "ot_users_per_session",
            "Number of users per session",
            users_buckets
        ))
        .expect("Failed to register ot_users_per_session")
    });

    info!("OT session metrics initialized");
}

/// Initialize OT document metrics
pub fn init_ot_document_metrics() {
    // Ops per document histogram: buckets from 1 to 10000 operations
    let ops_buckets = vec![1.0, 10.0, 50.0, 100.0, 500.0, 1000.0, 5000.0, 10000.0];
    OT_OPS_PER_DOCUMENT.get_or_init(|| {
        register_histogram!(histogram_opts!(
            "ot_ops_per_document",
            "Number of operations per document",
            ops_buckets
        ))
        .expect("Failed to register ot_ops_per_document")
    });

    // Document size histogram: buckets from 1KB to 100MB
    let size_buckets = vec![
        1024.0,
        10240.0,
        102400.0,
        1048576.0,
        10485760.0,
        52428800.0,
        104857600.0,
    ];
    OT_DOCUMENT_SIZE.get_or_init(|| {
        register_histogram!(histogram_opts!(
            "ot_document_size_bytes",
            "Document size in bytes",
            size_buckets
        ))
        .expect("Failed to register ot_document_size_bytes")
    });

    OT_SNAPSHOTS_CREATED.get_or_init(|| {
        register_int_counter!(opts!(
            "ot_snapshots_created_total",
            "Total number of document snapshots created"
        ))
        .expect("Failed to register ot_snapshots_created_total")
    });

    info!("OT document metrics initialized");
}

/// Initialize write-behind caching metrics
pub fn init_writebehind_metrics() {
    WRITEBEHIND_OPS_BUFFERED.get_or_init(|| {
        register_int_counter!(opts!(
            "writebehind_ops_buffered_total",
            "Total number of operations buffered in write-behind cache"
        ))
        .expect("Failed to register writebehind_ops_buffered_total")
    });

    WRITEBEHIND_OPS_FLUSHED.get_or_init(|| {
        register_int_counter!(opts!(
            "writebehind_ops_flushed_total",
            "Total number of operations flushed from cache to database"
        ))
        .expect("Failed to register writebehind_ops_flushed_total")
    });

    // Flush latency histogram: buckets from 1ms to 10s
    let latency_buckets =
        exponential_buckets(0.001, 2.0, 14).expect("Failed to create flush latency buckets");
    WRITEBEHIND_FLUSH_LATENCY.get_or_init(|| {
        register_histogram!(histogram_opts!(
            "writebehind_flush_latency_seconds",
            "Time to flush operations from cache to database",
            latency_buckets
        ))
        .expect("Failed to register writebehind_flush_latency_seconds")
    });

    WRITEBEHIND_QUEUE_DEPTH.get_or_init(|| {
        register_int_gauge!(opts!(
            "writebehind_queue_depth",
            "Current number of documents pending flush"
        ))
        .expect("Failed to register writebehind_queue_depth")
    });

    WRITEBEHIND_CACHE_HITS.get_or_init(|| {
        register_int_counter!(opts!(
            "writebehind_cache_hits_total",
            "Total number of cache hits"
        ))
        .expect("Failed to register writebehind_cache_hits_total")
    });

    WRITEBEHIND_CACHE_MISSES.get_or_init(|| {
        register_int_counter!(opts!(
            "writebehind_cache_misses_total",
            "Total number of cache misses"
        ))
        .expect("Failed to register writebehind_cache_misses_total")
    });

    WRITEBEHIND_FALLBACKS.get_or_init(|| {
        register_int_counter!(opts!(
            "writebehind_fallbacks_total",
            "Total number of fallbacks to synchronous DB write"
        ))
        .expect("Failed to register writebehind_fallbacks_total")
    });

    WRITEBEHIND_SKIPPED.get_or_init(|| {
        register_int_counter!(opts!(
            "writebehind_skipped_total",
            "Total number of skipped flushes due to lock contention"
        ))
        .expect("Failed to register writebehind_skipped_total")
    });

    // Redis resilience metrics
    REDIS_RETRIES_TOTAL.get_or_init(|| {
        register_int_counter!(opts!(
            "redis_retries_total",
            "Total number of Redis operation retries"
        ))
        .expect("Failed to register redis_retries_total")
    });

    REDIS_TIMEOUTS_TOTAL.get_or_init(|| {
        register_int_counter!(opts!(
            "redis_timeouts_total",
            "Total number of Redis operation timeouts"
        ))
        .expect("Failed to register redis_timeouts_total")
    });

    info!("Write-behind caching metrics initialized");
}

/// Initialize all OT metrics
///
/// This is a convenience function that initializes all OT-related metrics at once.
pub fn init_all_ot_metrics() {
    init_ot_performance_metrics();
    init_ot_lock_metrics();
    init_ot_conflict_metrics();
    init_ot_consistency_metrics();
    init_ot_session_metrics();
    init_ot_document_metrics();
    init_writebehind_metrics();
    info!("All OT metrics initialized");
}

// ============================================================================
// OT Performance Metric Helpers
// ============================================================================

/// Record the end-to-end operation latency
pub fn record_operation_e2e_latency(latency_seconds: f64) {
    if let Some(histogram) = OT_OPERATION_E2E_LATENCY.get() {
        histogram.observe(latency_seconds);
    }
}

/// Record the transform latency
pub fn record_transform_latency(latency_seconds: f64) {
    if let Some(histogram) = OT_TRANSFORM_LATENCY.get() {
        histogram.observe(latency_seconds);
    }
}

/// Record the broadcast latency
pub fn record_broadcast_latency(latency_seconds: f64) {
    if let Some(histogram) = OT_BROADCAST_LATENCY.get() {
        histogram.observe(latency_seconds);
    }
}

/// Record the client round-trip time
pub fn record_client_rtt(rtt_seconds: f64) {
    if let Some(histogram) = OT_CLIENT_RTT.get() {
        histogram.observe(rtt_seconds);
    }
}

/// Increment the total operations counter
pub fn increment_operations_total() {
    if let Some(counter) = OT_OPERATIONS_TOTAL.get() {
        counter.inc();
    }
}

/// Increment the successful operations counter
pub fn increment_operations_success() {
    if let Some(counter) = OT_OPERATIONS_SUCCESS.get() {
        counter.inc();
    }
}

/// Increment the failed operations counter
pub fn increment_operations_failed() {
    if let Some(counter) = OT_OPERATIONS_FAILED.get() {
        counter.inc();
    }
}

/// Increment the WebSocket connections counter
pub fn increment_websocket_connections() {
    if let Some(gauge) = OT_WEBSOCKET_CONNECTIONS.get() {
        gauge.inc();
        // Update peak if current exceeds peak
        if let Some(peak) = OT_WEBSOCKET_CONNECTIONS_PEAK.get() {
            let current = gauge.get();
            if current > peak.get() {
                peak.set(current);
            }
        }
    }
}

/// Decrement the WebSocket connections counter
pub fn decrement_websocket_connections() {
    if let Some(gauge) = OT_WEBSOCKET_CONNECTIONS.get() {
        gauge.dec();
    }
}

/// Set the WebSocket connections counter to a specific value
pub fn set_websocket_connections(count: i64) {
    if let Some(gauge) = OT_WEBSOCKET_CONNECTIONS.get() {
        gauge.set(count);
        // Update peak if current exceeds peak
        if let Some(peak) = OT_WEBSOCKET_CONNECTIONS_PEAK.get() {
            if count > peak.get() {
                peak.set(count);
            }
        }
    }
}

// ============================================================================
// OT Lock Metric Helpers
// ============================================================================

/// Increment the lock timeouts counter
pub fn increment_lock_timeouts() {
    if let Some(counter) = OT_LOCK_TIMEOUTS.get() {
        counter.inc();
    }
}

/// Record the number of lock acquisition retries
pub fn record_lock_retries(retries: f64) {
    if let Some(histogram) = OT_LOCK_RETRIES.get() {
        histogram.observe(retries);
    }
}

/// Record the time spent waiting to acquire lock
pub fn record_lock_wait_time(wait_seconds: f64) {
    if let Some(histogram) = OT_LOCK_WAIT_TIME.get() {
        histogram.observe(wait_seconds);
    }
}

/// Record the time lock was held
pub fn record_lock_hold_time(hold_seconds: f64) {
    if let Some(histogram) = OT_LOCK_HOLD_TIME.get() {
        histogram.observe(hold_seconds);
    }
}

// ============================================================================
// OT Conflict Metric Helpers
// ============================================================================

/// Increment the conflicts total counter
pub fn increment_conflicts_total() {
    if let Some(counter) = OT_CONFLICTS_TOTAL.get() {
        counter.inc();
    }
}

/// Increment the conflicts resolved counter
pub fn increment_conflicts_resolved() {
    if let Some(counter) = OT_CONFLICTS_RESOLVED.get() {
        counter.inc();
    }
}

/// Increment the transform retries counter
pub fn increment_transform_retries() {
    if let Some(counter) = OT_TRANSFORM_RETRIES.get() {
        counter.inc();
    }
}

/// Record the transform complexity (nesting depth)
pub fn record_transform_complexity(depth: f64) {
    if let Some(histogram) = OT_TRANSFORM_COMPLEXITY.get() {
        histogram.observe(depth);
    }
}

// ============================================================================
// OT Consistency Metric Helpers
// ============================================================================

/// Increment the inconsistencies detected counter
pub fn increment_inconsistencies_detected() {
    if let Some(counter) = OT_INCONSISTENCIES_DETECTED.get() {
        counter.inc();
    }
}

/// Increment the state syncs counter
pub fn increment_state_syncs() {
    if let Some(counter) = OT_STATE_SYNCS.get() {
        counter.inc();
    }
}

/// Increment the state repairs counter
pub fn increment_state_repairs() {
    if let Some(counter) = OT_STATE_REPAIRS.get() {
        counter.inc();
    }
}

/// Record the version drift
pub fn record_version_drift(drift: f64) {
    if let Some(histogram) = OT_VERSION_DRIFT.get() {
        histogram.observe(drift);
    }
}

/// Increment the version drift rejections counter
pub fn increment_version_drift_rejections() {
    if let Some(counter) = OT_VERSION_DRIFT_REJECTIONS.get() {
        counter.inc();
    }
}

/// Increment the version reconciliations counter
/// Called when cache miss occurs but pending ops exist in cache
pub fn increment_version_reconciliations() {
    if let Some(counter) = OT_VERSION_RECONCILIATIONS.get() {
        counter.inc();
    }
}

// ============================================================================
// OT Session Metric Helpers
// ============================================================================

/// Increment the active sessions gauge
pub fn increment_active_sessions() {
    if let Some(gauge) = OT_ACTIVE_SESSIONS.get() {
        gauge.inc();
    }
    if let Some(counter) = OT_SESSIONS_CREATED.get() {
        counter.inc();
    }
}

/// Decrement the active sessions gauge
pub fn decrement_active_sessions() {
    if let Some(gauge) = OT_ACTIVE_SESSIONS.get() {
        gauge.dec();
    }
    if let Some(counter) = OT_SESSIONS_CLOSED.get() {
        counter.inc();
    }
}

/// Set the active sessions gauge to a specific value
pub fn set_active_sessions(count: i64) {
    if let Some(gauge) = OT_ACTIVE_SESSIONS.get() {
        gauge.set(count);
    }
}

/// Record the session duration
pub fn record_session_duration(duration_seconds: f64) {
    if let Some(histogram) = OT_SESSION_DURATION.get() {
        histogram.observe(duration_seconds);
    }
}

/// Increment the user joins counter
pub fn increment_user_joins() {
    if let Some(counter) = OT_USER_JOINS.get() {
        counter.inc();
    }
}

/// Increment the user leaves counter
pub fn increment_user_leaves() {
    if let Some(counter) = OT_USER_LEAVES.get() {
        counter.inc();
    }
}

/// Record the number of users in a session
pub fn record_users_per_session(count: f64) {
    if let Some(histogram) = OT_USERS_PER_SESSION.get() {
        histogram.observe(count);
    }
}

// ============================================================================
// OT Document Metric Helpers
// ============================================================================

/// Record the number of operations for a document
pub fn record_ops_per_document(count: f64) {
    if let Some(histogram) = OT_OPS_PER_DOCUMENT.get() {
        histogram.observe(count);
    }
}

/// Record the document size
pub fn record_document_size(size_bytes: f64) {
    if let Some(histogram) = OT_DOCUMENT_SIZE.get() {
        histogram.observe(size_bytes);
    }
}

/// Increment the snapshots created counter
pub fn increment_snapshots_created() {
    if let Some(counter) = OT_SNAPSHOTS_CREATED.get() {
        counter.inc();
    }
}

// ============================================================================
// Write-Behind Caching Metric Helpers
// ============================================================================

/// Increment the operations buffered counter
pub fn increment_writebehind_ops_buffered() {
    if let Some(counter) = WRITEBEHIND_OPS_BUFFERED.get() {
        counter.inc();
    }
}

/// Increment the operations buffered counter by a specific amount
pub fn increment_writebehind_ops_buffered_by(count: u64) {
    if let Some(counter) = WRITEBEHIND_OPS_BUFFERED.get() {
        counter.inc_by(count);
    }
}

/// Increment the operations flushed counter
pub fn increment_writebehind_ops_flushed() {
    if let Some(counter) = WRITEBEHIND_OPS_FLUSHED.get() {
        counter.inc();
    }
}

/// Increment the operations flushed counter by a specific amount
pub fn increment_writebehind_ops_flushed_by(count: u64) {
    if let Some(counter) = WRITEBEHIND_OPS_FLUSHED.get() {
        counter.inc_by(count);
    }
}

/// Record the flush latency
pub fn record_writebehind_flush_latency(latency_seconds: f64) {
    if let Some(histogram) = WRITEBEHIND_FLUSH_LATENCY.get() {
        histogram.observe(latency_seconds);
    }
}

/// Set the current queue depth
pub fn set_writebehind_queue_depth(depth: i64) {
    if let Some(gauge) = WRITEBEHIND_QUEUE_DEPTH.get() {
        gauge.set(depth);
    }
}

/// Increment the cache hits counter
pub fn increment_writebehind_cache_hits() {
    if let Some(counter) = WRITEBEHIND_CACHE_HITS.get() {
        counter.inc();
    }
}

/// Increment the cache misses counter
pub fn increment_writebehind_cache_misses() {
    if let Some(counter) = WRITEBEHIND_CACHE_MISSES.get() {
        counter.inc();
    }
}

/// Increment the fallbacks counter
pub fn increment_writebehind_fallbacks() {
    if let Some(counter) = WRITEBEHIND_FALLBACKS.get() {
        counter.inc();
    }
}

/// Increment the skipped flushes counter (due to lock contention)
pub fn increment_writebehind_skipped() {
    if let Some(counter) = WRITEBEHIND_SKIPPED.get() {
        counter.inc();
    }
}

// ============================================================================
// Redis Resilience Metrics Functions
// ============================================================================

/// Increment the Redis retries counter
pub fn increment_redis_retries() {
    if let Some(counter) = REDIS_RETRIES_TOTAL.get() {
        counter.inc();
    }
}

/// Increment the Redis timeouts counter
pub fn increment_redis_timeouts() {
    if let Some(counter) = REDIS_TIMEOUTS_TOTAL.get() {
        counter.inc();
    }
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
    // Update memory metrics
    #[cfg(target_os = "linux")]
    {
        // Read memory info from /proc/self/statm
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

        // Read heap info from /proc/self/status
        if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
            for line in status.lines() {
                if line.starts_with("VmData:") {
                    if let Some(heap_str) = line.split_whitespace().nth(1) {
                        if let Ok(heap_kb) = heap_str.parse::<i64>() {
                            PROCESS_MEMORY_HEAP.get().unwrap().set(heap_kb * 1024);
                        }
                    }
                    break;
                }
            }
        }

        // Read CPU info from /proc/self/stat
        if let Ok(stat) = std::fs::read_to_string("/proc/self/stat") {
            let parts: Vec<&str> = stat.split_whitespace().collect();
            // utime is at index 13, stime is at index 14 (0-based)
            if parts.len() > 14 {
                if let (Ok(utime), Ok(stime)) = (
                    parts[13].parse::<u64>(),
                    parts[14].parse::<u64>(),
                ) {
                    if let Some(cpu_state_mutex) = CPU_STATE.get() {
                        if let Ok(mut cpu_state) = cpu_state_mutex.lock() {
                            let now = std::time::Instant::now();
                            let time_elapsed = now.duration_since(cpu_state.last_timestamp).as_secs_f64();

                            if cpu_state.last_utime > 0 && time_elapsed > 0.0 {
                                // Calculate CPU time delta (in clock ticks)
                                let utime_delta = utime.saturating_sub(cpu_state.last_utime);
                                let stime_delta = stime.saturating_sub(cpu_state.last_stime);
                                let total_cpu_ticks = (utime_delta + stime_delta) as f64;

                                // Convert to seconds (assuming 100 clock ticks per second on Linux)
                                let clock_ticks_per_sec = 100.0;
                                let cpu_time_seconds = total_cpu_ticks / clock_ticks_per_sec;

                                // Calculate CPU usage ratio
                                let cpu_usage = cpu_time_seconds / time_elapsed;

                                PROCESS_CPU_USAGE.get().unwrap().set(cpu_usage);
                            }

                            // Update state for next measurement
                            cpu_state.last_utime = utime;
                            cpu_state.last_stime = stime;
                            cpu_state.last_timestamp = now;
                        }
                    }
                }
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        use std::process::Command;

        // Get memory info using ps
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

        // Get heap info using vmmap (more accurate than ps for heap)
        if let Ok(output) = Command::new("vmmap")
            .args([&std::process::id().to_string()])
            .output()
        {
            if let Ok(output_str) = String::from_utf8(output.stdout) {
                let mut heap_size: i64 = 0;
                for line in output_str.lines() {
                    if line.contains("MALLOC") || line.contains("VM_ALLOCATE") {
                        // Parse heap regions (vmmap output varies, this is a basic approach)
                        if let Some(size_part) = line.split_whitespace().nth(1) {
                            // Size is typically in format like "4096K" or "1M"
                            let size_str = size_part.trim_end_matches('K').trim_end_matches('M');
                            if let Ok(size) = size_str.parse::<i64>() {
                                if size_part.ends_with('K') {
                                    heap_size += size * 1024;
                                } else if size_part.ends_with('M') {
                                    heap_size += size * 1024 * 1024;
                                }
                            }
                        }
                    }
                }
                if heap_size > 0 {
                    PROCESS_MEMORY_HEAP.get().unwrap().set(heap_size);
                }
            }
        }

        // Get CPU info using ps with cpu percentage
        if let Ok(output) = Command::new("ps")
            .args(["-p", &std::process::id().to_string(), "-o", "%cpu="])
            .output()
        {
            if let Ok(output_str) = String::from_utf8(output.stdout) {
                if let Ok(cpu_percent) = output_str.trim().parse::<f64>() {
                    // ps returns CPU as percentage (0-100 per core), convert to ratio (0-1 per core)
                    PROCESS_CPU_USAGE.get().unwrap().set(cpu_percent / 100.0);
                }
            }
        }
    }
}

/// Start metrics collection background task
///
/// This spawns a tokio task that updates process metrics every 5 seconds
pub fn start_metrics_collection() {
    tokio::spawn(async {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));
        loop {
            interval.tick().await;
            update_process_metrics();
        }
    });
    info!("Metrics collection task started");
}

/// Get current process metrics for debugging
///
/// Returns a formatted string with current memory and CPU metrics
pub fn get_current_metrics_debug() -> String {
    update_process_metrics(); // Update metrics first

    let rss = PROCESS_MEMORY_RSS.get().map(|g| g.get()).unwrap_or(0);
    let virtual_mem = PROCESS_MEMORY_VIRTUAL.get().map(|g| g.get()).unwrap_or(0);
    let heap = PROCESS_MEMORY_HEAP.get().map(|g| g.get()).unwrap_or(0);
    let cpu = PROCESS_CPU_USAGE.get().map(|g| g.get()).unwrap_or(0.0);

    format!(
        "Process Metrics:\n\
         - RSS: {} MB\n\
         - Virtual Memory: {} MB\n\
         - Heap: {} MB\n\
         - CPU Usage: {:.2}% per core",
        rss / (1024 * 1024),
        virtual_mem / (1024 * 1024),
        heap / (1024 * 1024),
        cpu * 100.0
    )
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
