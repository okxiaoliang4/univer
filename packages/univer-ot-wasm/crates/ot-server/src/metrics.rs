//! Metrics collection and Prometheus exporter
//!
//! This module provides OpenTelemetry integration with Prometheus exporter
//! for monitoring server metrics including memory usage, system information,
//! and OT (Operational Transform) collaboration metrics.
//!
//! ## Metric Categories
//!
//! ### Core Performance Metrics
//! - Operation latency (end-to-end, transform, broadcast)
//! - Throughput (ops/sec, operations per document)
//! - System resources (CPU, memory, WebSocket connections)
//!
//! ### Collaboration Metrics
//! - Conflict resolution (conflicts, transform complexity)
//! - Consistency (version drift, reconciliations)
//! - Session metrics (active sessions, users per session)

use axum::response::IntoResponse;
use opentelemetry::global;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use prometheus::{
    exponential_buckets, histogram_opts, opts, register_gauge, register_histogram,
    register_int_counter, register_int_gauge, Encoder, Gauge, Histogram, IntCounter, IntGauge,
    TextEncoder,
};
use sea_orm::ConnectionTrait;
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

// ============================================================================
// OT Core Performance Metrics
// ============================================================================

/// End-to-end latency from operation receive to application complete
static OT_OPERATION_E2E_LATENCY: OnceLock<Histogram> = OnceLock::new();
/// Time spent on operation transformation
static OT_TRANSFORM_LATENCY: OnceLock<Histogram> = OnceLock::new();
/// Time to broadcast operation to all clients
static OT_BROADCAST_LATENCY: OnceLock<Histogram> = OnceLock::new();

/// Total number of operations processed
static OT_OPERATIONS_TOTAL: OnceLock<IntCounter> = OnceLock::new();
/// Total number of operations processed successfully
static OT_OPERATIONS_SUCCESS: OnceLock<IntCounter> = OnceLock::new();
/// Total number of operations that failed
static OT_OPERATIONS_FAILED: OnceLock<IntCounter> = OnceLock::new();

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

// ============================================================================
// OT Conflict Resolution Metrics
// ============================================================================

/// Total number of conflicts detected
static OT_CONFLICTS_TOTAL: OnceLock<IntCounter> = OnceLock::new();
/// Total number of conflicts resolved successfully
static OT_CONFLICTS_RESOLVED: OnceLock<IntCounter> = OnceLock::new();
/// Histogram of transform complexity (nesting depth)
static OT_TRANSFORM_COMPLEXITY: OnceLock<Histogram> = OnceLock::new();

// ============================================================================
// OT Consistency Metrics
// ============================================================================

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

// ============================================================================
// Write-Behind Cache Metrics (ot-server side: buffering + cache hit/miss)
// ============================================================================

/// Total number of operations buffered in write-behind cache
static WRITEBEHIND_OPS_BUFFERED: OnceLock<IntCounter> = OnceLock::new();
/// Total number of cache hits
static WRITEBEHIND_CACHE_HITS: OnceLock<IntCounter> = OnceLock::new();
/// Total number of cache misses
static WRITEBEHIND_CACHE_MISSES: OnceLock<IntCounter> = OnceLock::new();

// ============================================================================
// Local (L1) Cache Metrics
// ============================================================================

/// Total number of local (process-in-memory) cache hits
static LOCAL_CACHE_HITS: OnceLock<IntCounter> = OnceLock::new();

// ============================================================================
// Database Connection Pool Metrics
// ============================================================================

/// Total number of database connections (from pg_stat_activity)
static DB_POOL_TOTAL_CONNECTIONS: OnceLock<IntGauge> = OnceLock::new();
/// Number of active database connections (state = 'active')
static DB_POOL_ACTIVE_CONNECTIONS: OnceLock<IntGauge> = OnceLock::new();
/// Number of idle database connections (state = 'idle')
static DB_POOL_IDLE_CONNECTIONS: OnceLock<IntGauge> = OnceLock::new();
/// Number of idle in transaction connections (state = 'idle in transaction')
static DB_POOL_IDLE_IN_TRANSACTION: OnceLock<IntGauge> = OnceLock::new();
/// Maximum configured connections (from pg_settings.max_connections)
static DB_POOL_MAX_CONNECTIONS: OnceLock<IntGauge> = OnceLock::new();
/// Connection usage percentage
static DB_POOL_USAGE_PERCENT: OnceLock<Gauge> = OnceLock::new();

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

/// Initialize OT core performance metrics
pub fn init_ot_performance_metrics() {
    // Latency histograms with exponential buckets from 1ms to 10s
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
            wait_time_buckets
        ))
        .expect("Failed to register ot_lock_wait_time_seconds")
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

    info!("OT document metrics initialized");
}

/// Initialize write-behind cache metrics (ot-server side only: buffering + cache hit/miss)
pub fn init_writebehind_metrics() {
    WRITEBEHIND_OPS_BUFFERED.get_or_init(|| {
        register_int_counter!(opts!(
            "writebehind_ops_buffered_total",
            "Total number of operations buffered in write-behind cache"
        ))
        .expect("Failed to register writebehind_ops_buffered_total")
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

    LOCAL_CACHE_HITS.get_or_init(|| {
        register_int_counter!(opts!(
            "local_cache_hits_total",
            "Total number of local (L1) cache hits"
        ))
        .expect("Failed to register local_cache_hits_total")
    });

    info!("Write-behind cache metrics initialized");
}

/// Initialize database connection pool metrics
pub fn init_db_pool_metrics() {
    DB_POOL_TOTAL_CONNECTIONS.get_or_init(|| {
        register_int_gauge!(opts!(
            "db_pool_total_connections",
            "Total number of database connections (from pg_stat_activity)"
        ))
        .expect("Failed to register db_pool_total_connections")
    });

    DB_POOL_ACTIVE_CONNECTIONS.get_or_init(|| {
        register_int_gauge!(opts!(
            "db_pool_active_connections",
            "Number of active database connections (state = 'active')"
        ))
        .expect("Failed to register db_pool_active_connections")
    });

    DB_POOL_IDLE_CONNECTIONS.get_or_init(|| {
        register_int_gauge!(opts!(
            "db_pool_idle_connections",
            "Number of idle database connections (state = 'idle')"
        ))
        .expect("Failed to register db_pool_idle_connections")
    });

    DB_POOL_IDLE_IN_TRANSACTION.get_or_init(|| {
        register_int_gauge!(opts!(
            "db_pool_idle_in_transaction",
            "Number of idle in transaction connections (state = 'idle in transaction')"
        ))
        .expect("Failed to register db_pool_idle_in_transaction")
    });

    DB_POOL_MAX_CONNECTIONS.get_or_init(|| {
        register_int_gauge!(opts!(
            "db_pool_max_connections",
            "Maximum configured database connections"
        ))
        .expect("Failed to register db_pool_max_connections")
    });

    DB_POOL_USAGE_PERCENT.get_or_init(|| {
        register_gauge!(opts!(
            "db_pool_usage_percent",
            "Database connection pool usage percentage"
        ))
        .expect("Failed to register db_pool_usage_percent")
    });

    info!("Database connection pool metrics initialized");
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
    init_db_pool_metrics();
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

/// Record the transform complexity (nesting depth)
pub fn record_transform_complexity(depth: f64) {
    if let Some(histogram) = OT_TRANSFORM_COMPLEXITY.get() {
        histogram.observe(depth);
    }
}

// ============================================================================
// OT Consistency Metric Helpers
// ============================================================================

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

// ============================================================================
// Write-Behind Cache Metric Helpers
// ============================================================================

/// Increment the operations buffered counter by a specific amount
pub fn increment_writebehind_ops_buffered_by(count: u64) {
    if let Some(counter) = WRITEBEHIND_OPS_BUFFERED.get() {
        counter.inc_by(count);
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

// ============================================================================
// Local (L1) Cache Metric Helpers
// ============================================================================

/// Increment the local cache hits counter
pub fn increment_local_cache_hits() {
    if let Some(counter) = LOCAL_CACHE_HITS.get() {
        counter.inc();
    }
}


// ============================================================================
// Database Connection Pool Metric Helpers
// ============================================================================

/// Update database connection pool metrics from pg_stat_activity query results
pub fn update_db_pool_metrics(
    total: i64,
    active: i64,
    idle: i64,
    idle_in_transaction: i64,
    max_connections: i64,
) {
    if let Some(gauge) = DB_POOL_TOTAL_CONNECTIONS.get() {
        gauge.set(total);
    }
    if let Some(gauge) = DB_POOL_ACTIVE_CONNECTIONS.get() {
        gauge.set(active);
    }
    if let Some(gauge) = DB_POOL_IDLE_CONNECTIONS.get() {
        gauge.set(idle);
    }
    if let Some(gauge) = DB_POOL_IDLE_IN_TRANSACTION.get() {
        gauge.set(idle_in_transaction);
    }
    if let Some(gauge) = DB_POOL_MAX_CONNECTIONS.get() {
        gauge.set(max_connections);
    }
    if let Some(gauge) = DB_POOL_USAGE_PERCENT.get() {
        if max_connections > 0 {
            let usage = (total as f64 / max_connections as f64) * 100.0;
            gauge.set(usage);
        }
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

/// Start background task for database connection pool monitoring
///
/// This spawns a tokio task that queries PostgreSQL every 15 seconds
/// to collect connection pool statistics from pg_stat_activity
pub fn start_db_pool_monitoring(db: std::sync::Arc<sea_orm::DatabaseConnection>) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(15));
        let pool = db.get_postgres_connection_pool();

        loop {
            interval.tick().await;

            // Query pg_stat_activity for connection statistics
            let sql = r#"
                SELECT
                    count(*) as total,
                    count(*) FILTER (WHERE state = 'active') as active,
                    count(*) FILTER (WHERE state = 'idle') as idle,
                    count(*) FILTER (WHERE state = 'idle in transaction') as idle_in_transaction,
                    (SELECT setting::int FROM pg_settings WHERE name = 'max_connections') as max_connections
                FROM pg_stat_activity
                WHERE datname = current_database()
            "#;

            match sqlx::query_as::<_, (i64, i64, i64, i64, i32)>(sql)
                .fetch_one(pool)
                .await
            {
                Ok((total, active, idle, idle_in_transaction, max_connections)) => {
                    update_db_pool_metrics(
                        total,
                        active,
                        idle,
                        idle_in_transaction,
                        max_connections as i64,
                    );
                }
                Err(e) => {
                    error!("Failed to query database connection pool stats: {}", e);
                }
            }
        }
    });
    info!("Database connection pool monitoring task started");
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
