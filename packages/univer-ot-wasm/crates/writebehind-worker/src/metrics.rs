//! Prometheus metrics for writebehind worker

use once_cell::sync::Lazy;
use prometheus::{
    register_counter, register_counter_vec, register_histogram, register_int_gauge, Counter,
    CounterVec, Histogram, IntGauge,
};

// Write-behind metrics
pub static WRITEBEHIND_OPS_FLUSHED: Lazy<Counter> = Lazy::new(|| {
    register_counter!(
        "writebehind_ops_flushed_total",
        "Total number of operations flushed to database"
    )
    .unwrap()
});

pub static WRITEBEHIND_FLUSH_LATENCY: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!(
        "writebehind_flush_latency_seconds",
        "Latency of write-behind flush operations",
        vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]
    )
    .unwrap()
});

pub static WRITEBEHIND_SKIPPED: Lazy<Counter> = Lazy::new(|| {
    register_counter!(
        "writebehind_skipped_total",
        "Total number of skipped flush operations (lock contention or no ops)"
    )
    .unwrap()
});

pub static WRITEBEHIND_ERRORS: Lazy<CounterVec> = Lazy::new(|| {
    register_counter_vec!(
        "writebehind_errors_total",
        "Total number of write-behind errors by type",
        &["error_type"]
    )
    .unwrap()
});

pub static WRITEBEHIND_QUEUE_DEPTH: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "writebehind_queue_depth",
        "Current depth of the write-behind queue"
    )
    .unwrap()
});

// Stream metrics
pub static STREAM_MESSAGES_RECEIVED: Lazy<Counter> = Lazy::new(|| {
    register_counter!(
        "writebehind_stream_messages_received_total",
        "Total number of stream messages received"
    )
    .unwrap()
});

pub static STREAM_MESSAGES_ACKED: Lazy<Counter> = Lazy::new(|| {
    register_counter!(
        "writebehind_stream_messages_acked_total",
        "Total number of stream messages acknowledged"
    )
    .unwrap()
});

pub static STREAM_MESSAGES_CLAIMED: Lazy<Counter> = Lazy::new(|| {
    register_counter!(
        "writebehind_stream_messages_claimed_total",
        "Total number of stream messages claimed from dead consumers"
    )
    .unwrap()
});

pub static STREAM_LENGTH: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "writebehind_stream_length",
        "Current length of the write-behind stream"
    )
    .unwrap()
});

pub static STREAM_PENDING_COUNT: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "writebehind_stream_pending_count",
        "Number of pending (unacknowledged) messages in the stream"
    )
    .unwrap()
});

pub static STREAM_CONSUMER_COUNT: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "writebehind_stream_consumer_count",
        "Number of consumers in the stream consumer group"
    )
    .unwrap()
});

// DB version cache metrics
pub static DB_VERSION_CACHE_HITS: Lazy<Counter> = Lazy::new(|| {
    register_counter!(
        "db_version_cache_hits_total",
        "Total number of DB version cache hits"
    )
    .unwrap()
});

pub static DB_VERSION_CACHE_MISSES: Lazy<Counter> = Lazy::new(|| {
    register_counter!(
        "db_version_cache_misses_total",
        "Total number of DB version cache misses"
    )
    .unwrap()
});

// Helper functions
pub fn increment_writebehind_ops_flushed_by(count: u64) {
    WRITEBEHIND_OPS_FLUSHED.inc_by(count as f64);
}

pub fn record_writebehind_flush_latency(seconds: f64) {
    WRITEBEHIND_FLUSH_LATENCY.observe(seconds);
}

pub fn increment_writebehind_skipped() {
    WRITEBEHIND_SKIPPED.inc();
}

pub fn increment_writebehind_error(error_type: &str) {
    WRITEBEHIND_ERRORS.with_label_values(&[error_type]).inc();
}

pub fn set_queue_depth(depth: i64) {
    WRITEBEHIND_QUEUE_DEPTH.set(depth);
}

pub fn increment_stream_messages_received_by(count: u64) {
    STREAM_MESSAGES_RECEIVED.inc_by(count as f64);
}

pub fn increment_stream_messages_acked_by(count: u64) {
    STREAM_MESSAGES_ACKED.inc_by(count as f64);
}

pub fn increment_stream_messages_claimed_by(count: u64) {
    STREAM_MESSAGES_CLAIMED.inc_by(count as f64);
}

pub fn set_stream_info(length: u64, pending: u64, consumers: u64) {
    STREAM_LENGTH.set(length as i64);
    STREAM_PENDING_COUNT.set(pending as i64);
    STREAM_CONSUMER_COUNT.set(consumers as i64);
}

pub fn increment_db_version_cache_hits() {
    DB_VERSION_CACHE_HITS.inc();
}

pub fn increment_db_version_cache_misses() {
    DB_VERSION_CACHE_MISSES.inc();
}

