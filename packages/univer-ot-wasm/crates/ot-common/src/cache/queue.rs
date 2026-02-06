//! Stream-based queue service for write-behind notifications
//!
//! Uses Redis Streams with Consumer Groups to provide reliable, distributed
//! document flush notifications. Replaces the previous Pub/Sub + List approach.
//!
//! ## Redis Streams advantages over Pub/Sub + List
//!
//! - **At-least-once delivery**: Messages persist in the stream and are tracked
//!   in the Pending Entries List (PEL) until acknowledged
//! - **Consumer groups**: Natural load balancing across multiple workers -
//!   each message is delivered to exactly one consumer in the group
//! - **Dead consumer recovery**: XAUTOCLAIM allows live consumers to take over
//!   messages from crashed consumers
//! - **Backpressure**: MAXLEN~ keeps the stream bounded without blocking producers
//!
//! ## Keys
//!
//! ```text
//! Stream key:    ot:{wb}:stream        # Uses hash tag for Redis Cluster
//! Group name:    writebehind-workers
//! Consumer name: worker-{instance_id}-{thread_id}
//! ```

use anyhow::{Context, Result};
use redis::aio::ConnectionManager;

use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info, warn};

// Stream configuration
const STREAM_KEY: &str = "ot:{wb}:stream";
const GROUP_NAME: &str = "writebehind-workers";
const STREAM_MAXLEN: i64 = 100_000;

// Retry configuration (same as CacheService)
const MAX_RETRIES: u32 = 3;
const INITIAL_BACKOFF_MS: u64 = 10;
const MAX_BACKOFF_MS: u64 = 100;
const OPERATION_TIMEOUT_MS: u64 = 3000;

/// A message read from the stream
#[derive(Debug, Clone)]
pub struct StreamMessage {
    /// Redis stream message ID (e.g., "1234567890-0")
    pub id: String,
    /// Document ID extracted from the message payload
    pub doc_id: String,
}

/// Stream monitoring information
#[derive(Debug, Clone)]
pub struct StreamInfo {
    /// Total number of entries in the stream
    pub stream_length: u64,
    /// Number of pending (unacknowledged) messages across all consumers
    pub pending_count: u64,
    /// Number of consumers in the group
    pub consumer_count: u64,
}

/// Queue service using Redis Streams for write-behind notifications
///
/// This service manages the lifecycle of flush notification messages:
/// - **Producers** (ot-server) call `enqueue()` to add document IDs
/// - **Consumers** (writebehind-worker) call `read_messages()` to consume
/// - After processing, consumers call `ack()` to confirm
/// - `claim_pending()` recovers messages from dead consumers
#[derive(Clone)]
pub struct StreamQueueService {
    /// Pool of connection managers (shared with CacheService)
    connections: Vec<ConnectionManager>,
    /// Counter for round-robin connection selection
    next_conn: Arc<std::sync::atomic::AtomicUsize>,
}

impl StreamQueueService {
    /// Create a new StreamQueueService from existing connection managers
    ///
    /// Reuses the same connection pool as CacheService to avoid creating
    /// additional Redis connections.
    pub fn new(connections: Vec<ConnectionManager>) -> Self {
        Self {
            connections,
            next_conn: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        }
    }

    /// Get a connection from the pool using round-robin selection
    fn get_connection(&self) -> ConnectionManager {
        let idx = self
            .next_conn
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            % self.connections.len();
        self.connections[idx].clone()
    }

    /// Execute a Redis operation with retry and timeout
    async fn with_retry<T, F, Fut>(&self, operation_name: &str, f: F) -> Result<T>
    where
        F: Fn(ConnectionManager) -> Fut,
        Fut: std::future::Future<Output = redis::RedisResult<T>>,
    {
        let mut attempts = 0u32;
        let mut backoff_ms = INITIAL_BACKOFF_MS;

        loop {
            attempts += 1;
            let conn = self.get_connection();

            let result = tokio::time::timeout(
                Duration::from_millis(OPERATION_TIMEOUT_MS),
                f(conn),
            )
            .await;

            match result {
                Ok(Ok(value)) => return Ok(value),
                Ok(Err(e)) => {
                    if attempts >= MAX_RETRIES {
                        return Err(anyhow::anyhow!(
                            "Redis {} failed after {} attempts: {}",
                            operation_name,
                            attempts,
                            e
                        ));
                    }
                    if Self::is_retriable_error(&e) {
                        debug!(
                            "Redis {} failed (attempt {}), retrying in {}ms: {}",
                            operation_name, attempts, backoff_ms, e
                        );
                        tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
                        backoff_ms = (backoff_ms * 2).min(MAX_BACKOFF_MS);
                        continue;
                    } else {
                        return Err(anyhow::anyhow!("Redis {} failed: {}", operation_name, e));
                    }
                }
                Err(_timeout) => {
                    if attempts >= MAX_RETRIES {
                        return Err(anyhow::anyhow!(
                            "Redis {} timed out after {} attempts ({}ms timeout)",
                            operation_name,
                            attempts,
                            OPERATION_TIMEOUT_MS
                        ));
                    }
                    warn!(
                        "Redis {} timed out (attempt {}), retrying in {}ms",
                        operation_name, attempts, backoff_ms
                    );
                    tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
                    backoff_ms = (backoff_ms * 2).min(MAX_BACKOFF_MS);
                }
            }
        }
    }

    fn is_retriable_error(e: &redis::RedisError) -> bool {
        use redis::ErrorKind;

        match e.kind() {
            ErrorKind::Io => true,
            ErrorKind::ClusterConnectionNotFound => true,
            ErrorKind::Server(server_kind) => {
                use redis::ServerErrorKind;
                matches!(
                    server_kind,
                    ServerErrorKind::BusyLoading
                        | ServerErrorKind::TryAgain
                        | ServerErrorKind::MasterDown
                        | ServerErrorKind::ClusterDown
                )
            }
            _ => false,
        }
    }

    // ========================================================================
    // Producer API
    // ========================================================================

    /// Enqueue a document ID for background flushing
    ///
    /// Uses `XADD` with approximate MAXLEN trimming to keep the stream bounded.
    /// Returns the stream message ID assigned by Redis.
    pub async fn enqueue(&self, doc_id: &str) -> Result<String> {
        let doc_id_owned = doc_id.to_string();

        let msg_id: String = self
            .with_retry("stream_enqueue", |mut conn| {
                let doc_id = doc_id_owned.clone();
                async move {
                    redis::cmd("XADD")
                        .arg(STREAM_KEY)
                        .arg("MAXLEN")
                        .arg("~")
                        .arg(STREAM_MAXLEN)
                        .arg("*")
                        .arg("doc_id")
                        .arg(&doc_id)
                        .query_async(&mut conn)
                        .await
                }
            })
            .await?;

        debug!("Stream enqueue: doc_id={}, msg_id={}", doc_id, msg_id);
        Ok(msg_id)
    }

    // ========================================================================
    // Consumer API
    // ========================================================================

    /// Create the consumer group (idempotent)
    ///
    /// Uses `XGROUP CREATE ... MKSTREAM` to create both the stream and group
    /// if they don't exist. Safe to call multiple times.
    pub async fn ensure_group(&self) -> Result<()> {
        let result: redis::RedisResult<String> = {
            let mut conn = self.get_connection();
            redis::cmd("XGROUP")
                .arg("CREATE")
                .arg(STREAM_KEY)
                .arg(GROUP_NAME)
                .arg("0")
                .arg("MKSTREAM")
                .query_async(&mut conn)
                .await
        };

        match result {
            Ok(_) => {
                info!(
                    "Consumer group created: stream={}, group={}",
                    STREAM_KEY, GROUP_NAME
                );
            }
            Err(e) => {
                // BUSYGROUP means group already exists - this is expected
                let err_str = format!("{}", e);
                if err_str.contains("BUSYGROUP") {
                    debug!("Consumer group already exists: {}", GROUP_NAME);
                } else {
                    return Err(e)
                        .context("Failed to create consumer group");
                }
            }
        }

        Ok(())
    }

    /// Read messages from the stream as a consumer in the group
    ///
    /// Uses `XREADGROUP` with `>` to read only new (undelivered) messages.
    /// When `block_ms > 0`, blocks until messages arrive or timeout.
    /// Returns empty Vec on timeout (not an error).
    pub async fn read_messages(
        &self,
        consumer: &str,
        count: usize,
        block_ms: u64,
    ) -> Result<Vec<StreamMessage>> {
        let consumer_owned = consumer.to_string();

        // Use a longer timeout for the operation when blocking
        let op_timeout = if block_ms > 0 {
            block_ms + 1000 // Add 1s buffer for network latency
        } else {
            OPERATION_TIMEOUT_MS
        };

        let mut conn = self.get_connection();

        let result: redis::RedisResult<redis::Value> = tokio::time::timeout(
            Duration::from_millis(op_timeout),
            redis::cmd("XREADGROUP")
                .arg("GROUP")
                .arg(GROUP_NAME)
                .arg(&consumer_owned)
                .arg("COUNT")
                .arg(count)
                .arg("BLOCK")
                .arg(block_ms)
                .arg("STREAMS")
                .arg(STREAM_KEY)
                .arg(">")
                .query_async(&mut conn),
        )
        .await
        .unwrap_or(Ok(redis::Value::Nil)); // Timeout → treat as no messages

        match result {
            Ok(value) => Self::parse_xreadgroup_response(value),
            Err(e) => {
                if e.is_timeout() {
                    Ok(Vec::new())
                } else {
                    Err(anyhow::anyhow!("XREADGROUP failed: {}", e))
                }
            }
        }
    }

    /// Acknowledge messages as processed
    ///
    /// Removes messages from the consumer's Pending Entries List (PEL).
    /// Returns the number of messages acknowledged.
    pub async fn ack(&self, msg_ids: &[String]) -> Result<u64> {
        if msg_ids.is_empty() {
            return Ok(0);
        }

        let ids = msg_ids.to_vec();
        let count: u64 = self
            .with_retry("stream_ack", |mut conn| {
                let ids = ids.clone();
                async move {
                    let mut cmd = redis::cmd("XACK");
                    cmd.arg(STREAM_KEY).arg(GROUP_NAME);
                    for id in &ids {
                        cmd.arg(id);
                    }
                    cmd.query_async(&mut conn).await
                }
            })
            .await?;

        debug!("Stream ack: {} messages acknowledged", count);
        Ok(count)
    }

    /// Claim pending messages from dead or slow consumers
    ///
    /// Uses `XAUTOCLAIM` to atomically find and claim messages that have been
    /// idle (unacknowledged) for at least `min_idle_ms` milliseconds.
    /// This is the primary mechanism for recovering from consumer failures.
    pub async fn claim_pending(
        &self,
        consumer: &str,
        min_idle_ms: u64,
        count: usize,
    ) -> Result<Vec<StreamMessage>> {
        let consumer_owned = consumer.to_string();

        let value: redis::Value = self
            .with_retry("stream_claim_pending", |mut conn| {
                let consumer = consumer_owned.clone();
                async move {
                    redis::cmd("XAUTOCLAIM")
                        .arg(STREAM_KEY)
                        .arg(GROUP_NAME)
                        .arg(&consumer)
                        .arg(min_idle_ms)
                        .arg("0-0")
                        .arg("COUNT")
                        .arg(count)
                        .query_async(&mut conn)
                        .await
                }
            })
            .await?;

        Self::parse_xautoclaim_response(value)
    }

    /// Get stream monitoring information
    pub async fn get_info(&self) -> Result<StreamInfo> {
        let mut conn = self.get_connection();

        // Get stream length
        let stream_length: u64 = redis::cmd("XLEN")
            .arg(STREAM_KEY)
            .query_async(&mut conn)
            .await
            .unwrap_or(0);

        // Get group info
        let group_info: redis::RedisResult<redis::Value> = redis::cmd("XINFO")
            .arg("GROUPS")
            .arg(STREAM_KEY)
            .query_async(&mut conn)
            .await;

        let (pending_count, consumer_count) = match group_info {
            Ok(value) => Self::parse_xinfo_groups(value),
            Err(_) => (0, 0),
        };

        Ok(StreamInfo {
            stream_length,
            pending_count,
            consumer_count,
        })
    }

    // ========================================================================
    // Response Parsers (support both RESP2 and RESP3 formats)
    // ========================================================================

    /// Parse XREADGROUP response
    ///
    /// RESP2 format: `[[stream_key, [[msg_id, [field, value, ...]], ...]]]`
    /// RESP3 format: `Map([(stream_key, [[msg_id, Map([(field, value)])]])])`
    ///
    /// Returns Nil when BLOCK times out.
    fn parse_xreadgroup_response(value: redis::Value) -> Result<Vec<StreamMessage>> {
        // Nil = timeout or no messages
        if matches!(value, redis::Value::Nil) {
            return Ok(Vec::new());
        }

        let mut messages = Vec::new();

        match value {
            // RESP2: Array of [stream_key, entries] pairs
            redis::Value::Array(streams) => {
                for stream in streams {
                    let stream_data = match stream {
                        redis::Value::Array(data) if data.len() == 2 => data,
                        _ => continue,
                    };

                    let entries = match stream_data.into_iter().nth(1) {
                        Some(redis::Value::Array(msgs)) => msgs,
                        _ => continue,
                    };

                    for entry in entries {
                        Self::parse_stream_entry(entry, &mut messages);
                    }
                }
            }
            // RESP3: Map of stream_key => entries
            redis::Value::Map(stream_map) => {
                for (_key, entries_value) in stream_map {
                    let entries = match entries_value {
                        redis::Value::Array(msgs) => msgs,
                        _ => continue,
                    };

                    for entry in entries {
                        Self::parse_stream_entry(entry, &mut messages);
                    }
                }
            }
            other => {
                debug!(
                    "XREADGROUP unexpected response type: {:?}",
                    std::mem::discriminant(&other)
                );
            }
        }

        Ok(messages)
    }

    /// Parse XAUTOCLAIM response
    ///
    /// RESP2: `[next_start_id, [[msg_id, [field, value, ...]], ...], [deleted_ids]]`
    /// RESP3: `[next_start_id, [[msg_id, Map([(field, value)])], ...], [deleted_ids]]`
    fn parse_xautoclaim_response(value: redis::Value) -> Result<Vec<StreamMessage>> {
        let parts = match value {
            redis::Value::Array(parts) if parts.len() >= 2 => parts,
            _ => return Ok(Vec::new()),
        };

        // parts[0] = next start ID (for pagination)
        // parts[1] = claimed messages
        // parts[2] = deleted IDs (optional)
        let msgs = match parts.into_iter().nth(1) {
            Some(redis::Value::Array(msgs)) => msgs,
            _ => return Ok(Vec::new()),
        };

        let mut messages = Vec::new();

        for entry in msgs {
            Self::parse_stream_entry(entry, &mut messages);
        }

        Ok(messages)
    }

    /// Parse a single stream entry from either RESP2 or RESP3 format
    ///
    /// A stream entry is always `[msg_id, fields]` where fields can be:
    /// - RESP2: flat Array `[field, value, field, value, ...]`
    /// - RESP3: Map `[(field, value), ...]`
    fn parse_stream_entry(entry: redis::Value, messages: &mut Vec<StreamMessage>) {
        let msg_data = match entry {
            redis::Value::Array(data) if data.len() == 2 => data,
            _ => return,
        };

        let mut msg_iter = msg_data.into_iter();
        let id = match msg_iter.next() {
            Some(redis::Value::BulkString(bytes)) => {
                String::from_utf8_lossy(&bytes).to_string()
            }
            _ => return,
        };

        // Extract doc_id from fields (handles both RESP2 Array and RESP3 Map)
        let doc_id = match msg_iter.next() {
            Some(redis::Value::Array(fields)) => {
                Self::extract_field_value_from_array(&fields, "doc_id")
            }
            Some(redis::Value::Map(pairs)) => {
                Self::extract_field_value_from_map(&pairs, "doc_id")
            }
            _ => None,
        };

        if let Some(doc_id) = doc_id {
            messages.push(StreamMessage { id, doc_id });
        }
    }

    /// Parse XINFO GROUPS response to extract pending count and consumer count
    ///
    /// RESP2: `[["name", "group-name", "consumers", 8, "pel-count", 10, ...], ...]`
    /// RESP3: `[Map([("name", "group-name"), ("consumers", 8), ("pel-count", 10), ...]), ...]`
    fn parse_xinfo_groups(value: redis::Value) -> (u64, u64) {
        let groups = match value {
            redis::Value::Array(groups) => groups,
            _ => return (0, 0),
        };

        for group in groups {
            match group {
                // RESP2: flat array of field-value pairs
                redis::Value::Array(fields) => {
                    let name = Self::extract_field_value_from_array(&fields, "name");
                    if name.as_deref() != Some(GROUP_NAME) {
                        continue;
                    }
                    let pending =
                        Self::extract_field_i64_from_array(&fields, "pel-count").unwrap_or(0)
                            as u64;
                    let consumers =
                        Self::extract_field_i64_from_array(&fields, "consumers").unwrap_or(0)
                            as u64;
                    return (pending, consumers);
                }
                // RESP3: map of field-value pairs
                redis::Value::Map(pairs) => {
                    let name = Self::extract_field_value_from_map(&pairs, "name");
                    if name.as_deref() != Some(GROUP_NAME) {
                        continue;
                    }
                    let pending =
                        Self::extract_field_i64_from_map(&pairs, "pel-count").unwrap_or(0) as u64;
                    let consumers =
                        Self::extract_field_i64_from_map(&pairs, "consumers").unwrap_or(0) as u64;
                    return (pending, consumers);
                }
                _ => continue,
            }
        }

        (0, 0)
    }

    // ========================================================================
    // Field Extractors (RESP2: flat Array, RESP3: Map)
    // ========================================================================

    /// Extract a string value from RESP2 flat array: [field, value, field, value, ...]
    fn extract_field_value_from_array(fields: &[redis::Value], key: &str) -> Option<String> {
        let mut iter = fields.iter();
        while let Some(field) = iter.next() {
            let field_name = match field {
                redis::Value::BulkString(bytes) => String::from_utf8_lossy(bytes),
                _ => {
                    iter.next(); // skip value
                    continue;
                }
            };

            if let Some(value) = iter.next() {
                if field_name == key {
                    return Self::value_to_string(value);
                }
            }
        }
        None
    }

    /// Extract a string value from RESP3 map: [(field, value), ...]
    fn extract_field_value_from_map(
        pairs: &[(redis::Value, redis::Value)],
        key: &str,
    ) -> Option<String> {
        for (field, value) in pairs {
            let field_name = match field {
                redis::Value::BulkString(bytes) => String::from_utf8_lossy(bytes),
                redis::Value::SimpleString(s) => std::borrow::Cow::Borrowed(s.as_str()),
                _ => continue,
            };
            if field_name == key {
                return Self::value_to_string(value);
            }
        }
        None
    }

    /// Extract an integer value from RESP2 flat array
    fn extract_field_i64_from_array(fields: &[redis::Value], key: &str) -> Option<i64> {
        let mut iter = fields.iter();
        while let Some(field) = iter.next() {
            let field_name = match field {
                redis::Value::BulkString(bytes) => String::from_utf8_lossy(bytes),
                _ => {
                    iter.next(); // skip value
                    continue;
                }
            };

            if let Some(value) = iter.next() {
                if field_name == key {
                    return Self::value_to_i64(value);
                }
            }
        }
        None
    }

    /// Extract an integer value from RESP3 map
    fn extract_field_i64_from_map(
        pairs: &[(redis::Value, redis::Value)],
        key: &str,
    ) -> Option<i64> {
        for (field, value) in pairs {
            let field_name = match field {
                redis::Value::BulkString(bytes) => String::from_utf8_lossy(bytes),
                redis::Value::SimpleString(s) => std::borrow::Cow::Borrowed(s.as_str()),
                _ => continue,
            };
            if field_name == key {
                return Self::value_to_i64(value);
            }
        }
        None
    }

    /// Convert a Redis Value to String
    fn value_to_string(value: &redis::Value) -> Option<String> {
        match value {
            redis::Value::BulkString(bytes) => {
                Some(String::from_utf8_lossy(bytes).to_string())
            }
            redis::Value::SimpleString(s) => Some(s.clone()),
            redis::Value::Int(i) => Some(i.to_string()),
            _ => None,
        }
    }

    /// Convert a Redis Value to i64
    fn value_to_i64(value: &redis::Value) -> Option<i64> {
        match value {
            redis::Value::Int(i) => Some(*i),
            redis::Value::BulkString(bytes) => {
                String::from_utf8_lossy(bytes).parse().ok()
            }
            redis::Value::Double(f) => Some(*f as i64),
            _ => None,
        }
    }
}
