//! Cache service for write-behind caching optimization
//!
//! This module provides a Redis-based caching layer for OT operations that enables
//! asynchronous batch writes to the database. Operations are first written to Redis
//! (fast, 1-5ms) and then flushed to PostgreSQL in batches by background workers.
//!
//! ## Redis Data Structures
//!
//! - `ot:doc:{doc_id}:version` - String containing current version number
//! - `ot:doc:{doc_id}:ops` - Sorted Set of pending operations (score = rev)
//! - `ot:writebehind:queue` - List of document IDs pending flush
//! - `ot:writebehind:set` - Set for deduplication of pending docs
//! - `ot:writebehind:lock:{doc_id}` - Lock to prevent concurrent flushes
//!
//! ## Connection Management
//!
//! Uses `redis::aio::ConnectionManager` for efficient connection handling:
//! - Single connection shared across all operations (multiplexed)
//! - Automatic reconnection on connection loss
//! - Zero-cost cloning (internally Arc-based)
//! - Thread-safe (Send + Sync)
//!
//! ## High Concurrency Resilience
//!
//! Critical operations use retry with exponential backoff to handle:
//! - Transient network failures
//! - Redis server temporary unavailability
//! - Connection pool exhaustion
//!
//! Configuration: MAX_RETRIES=3, initial backoff=5ms, max backoff=50ms

use crate::metrics;
use anyhow::{Context, Result};
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Notify;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Entry representing a single operation to be cached
/// Note: params are stored directly in the cache entry for low-latency writes.
/// S3 upload happens asynchronously in the write-behind worker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationEntry {
    pub rev: i64,
    pub user_id: String,
    pub mutation_id: String,
    pub params: Vec<u8>, // MessagePack encoded params - uploaded to S3 by write-behind worker
    pub client_id: String,
    pub op_id: String,
    pub created_at: i64, // Unix timestamp in milliseconds
}

/// Information about a cached operation (for reading)
/// Contains decoded params directly from the cache
#[derive(Debug, Clone)]
pub struct CachedOperationInfo {
    pub rev: i64,
    pub user_id: String,
    pub mutation_id: String,
    pub params: Vec<u8>, // MessagePack encoded params
    pub client_id: String,
    pub op_id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Configuration for the cache service
#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub ttl_seconds: u64,
    pub batch_size: usize,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            ttl_seconds: 3600,
            batch_size: 1000,
        }
    }
}

/// Cache service for write-behind caching
///
/// Uses a pool of `ConnectionManager` instances for efficient Redis access:
/// - Multiple connections to handle high concurrency
/// - Automatic reconnection on connection loss
/// - Round-robin load balancing across connections
/// - Thread-safe (Send + Sync)
#[derive(Clone)]
pub struct CacheService {
    /// Pool of connection managers for high concurrency
    /// Multiple connections prevent single-connection bottleneck
    connections: Vec<ConnectionManager>,
    /// Counter for round-robin connection selection
    next_conn: Arc<std::sync::atomic::AtomicUsize>,
    config: CacheConfig,
    /// Notification for workers when a document is enqueued
    /// This enables non-blocking worker loops instead of BRPOP
    enqueue_notify: Arc<Notify>,
}

// Redis key prefixes
const KEY_PREFIX_VERSION: &str = "ot:doc:";
const KEY_SUFFIX_VERSION: &str = ":version";
const KEY_PREFIX_OPS: &str = "ot:doc:";
const KEY_SUFFIX_OPS: &str = ":ops";
const KEY_SUFFIX_IDEMPOTENCY: &str = ":idempotency"; // Hash for {client_id}:{op_id} -> rev
// Use hash tag {wb} to ensure queue operations work in Redis Cluster
const KEY_WRITEBEHIND_QUEUE: &str = "ot:{wb}:queue";
const KEY_PREFIX_LOCK: &str = "ot:writebehind:lock:";
// Params cache key prefix for TTL caching of operation params
const KEY_PREFIX_PARAMS: &str = "ot:params:";
const PARAMS_CACHE_TTL_SECONDS: u64 = 3600; // 1 hour TTL for params cache

// Retry configuration for high concurrency resilience
const MAX_RETRIES: u32 = 3; // Increased from 2 to 3 for better resilience
const INITIAL_BACKOFF_MS: u64 = 10;
const MAX_BACKOFF_MS: u64 = 100;
const OPERATION_TIMEOUT_MS: u64 = 3000; // Increased from 2s to 3s for high concurrency

// Connection pool configuration
const CONNECTION_POOL_SIZE: usize = 16; // Increased from 8 to 16 for high concurrency

impl CacheService {
    /// Create a new cache service with a pool of ConnectionManagers
    ///
    /// Multiple connections prevent single-connection bottleneck under high concurrency:
    /// - Creates CONNECTION_POOL_SIZE connections
    /// - Round-robin load balancing across connections
    /// - Each ConnectionManager handles automatic reconnection
    pub async fn new(redis_client: redis::Client, config: CacheConfig) -> Result<Self> {
        info!(
            "Creating CacheService with {} connections",
            CONNECTION_POOL_SIZE
        );

        // Create multiple connection managers for high concurrency
        let mut connections = Vec::with_capacity(CONNECTION_POOL_SIZE);
        for i in 0..CONNECTION_POOL_SIZE {
            let conn = ConnectionManager::new(redis_client.clone())
                .await
                .with_context(|| format!("Failed to create Redis ConnectionManager {}", i))?;
            connections.push(conn);
        }

        let service = Self {
            connections,
            next_conn: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            config,
            enqueue_notify: Arc::new(Notify::new()),
        };

        // Validate connection
        if !service.health_check().await.unwrap_or(false) {
            return Err(anyhow::anyhow!(
                "Failed to validate Redis connection for CacheService"
            ));
        }

        info!(
            "CacheService: {} connections created and validated successfully",
            CONNECTION_POOL_SIZE
        );
        Ok(service)
    }

    /// Create from existing ConnectionManagers (for sharing across services)
    pub fn from_connection_managers(connections: Vec<ConnectionManager>, config: CacheConfig) -> Self {
        Self {
            connections,
            next_conn: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            config,
            enqueue_notify: Arc::new(Notify::new()),
        }
    }

    /// Get a connection from the pool using round-robin selection
    ///
    /// This distributes load across multiple connections to prevent
    /// single-connection bottleneck under high concurrency
    fn get_connection(&self) -> ConnectionManager {
        let idx = self
            .next_conn
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            % self.connections.len();
        self.connections[idx].clone()
    }

    /// Get a clone of the first connection manager for use in other services
    pub fn connection_manager(&self) -> ConnectionManager {
        self.connections[0].clone()
    }

    /// Get the version key for a document
    fn version_key(doc_id: Uuid) -> String {
        format!("{}{}{}", KEY_PREFIX_VERSION, doc_id, KEY_SUFFIX_VERSION)
    }

    /// Get the operations key for a document
    fn ops_key(doc_id: Uuid) -> String {
        format!("{}{}{}", KEY_PREFIX_OPS, doc_id, KEY_SUFFIX_OPS)
    }

    /// Get the flush lock key for a document
    fn lock_key(doc_id: &str) -> String {
        format!("{}{}", KEY_PREFIX_LOCK, doc_id)
    }

    /// Get the params cache key for a storage_id
    fn params_key(storage_id: Uuid) -> String {
        format!("{}{}", KEY_PREFIX_PARAMS, storage_id)
    }

    /// Get the idempotency hash key for a document
    fn idempotency_key(doc_id: Uuid) -> String {
        format!("{}{}{}", KEY_PREFIX_OPS, doc_id, KEY_SUFFIX_IDEMPOTENCY)
    }

    /// Get the idempotency field name for a client_id and op_id
    fn idempotency_field(client_id: &str, op_id: &str) -> String {
        format!("{}:{}", client_id, op_id)
    }

    /// Execute a Redis operation with retry and timeout
    ///
    /// This helper provides resilience for high-concurrency scenarios:
    /// - Uses connection pool with round-robin load balancing
    /// - Retries transient failures with exponential backoff
    /// - Applies operation timeout to prevent indefinite blocking
    /// - Logs retries for observability
    /// - Records metrics for monitoring
    async fn with_retry<T, F, Fut>(&self, operation_name: &str, f: F) -> Result<T>
    where
        F: Fn(ConnectionManager) -> Fut,
        Fut: std::future::Future<Output = redis::RedisResult<T>>,
    {
        let mut attempts = 0u32;
        let mut backoff_ms = INITIAL_BACKOFF_MS;

        loop {
            attempts += 1;
            // Get connection from pool (round-robin)
            let conn = self.get_connection();

            // Apply timeout to the operation
            let result = tokio::time::timeout(
                Duration::from_millis(OPERATION_TIMEOUT_MS),
                f(conn)
            ).await;

            match result {
                Ok(Ok(value)) => return Ok(value),
                Ok(Err(e)) => {
                    // Redis error
                    if attempts >= MAX_RETRIES {
                        return Err(anyhow::anyhow!(
                            "Redis {} failed after {} attempts: {}",
                            operation_name, attempts, e
                        ));
                    }
                    // Check if error is retriable
                    if Self::is_retriable_error(&e) {
                        metrics::increment_redis_retries();
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
                    // Timeout
                    metrics::increment_redis_timeouts();
                    if attempts >= MAX_RETRIES {
                        return Err(anyhow::anyhow!(
                            "Redis {} timed out after {} attempts ({}ms timeout)",
                            operation_name, attempts, OPERATION_TIMEOUT_MS
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

    /// Check if a Redis error is retriable (transient)
    ///
    /// Retriable errors include:
    /// - I/O errors (network issues, connection drops)
    /// - Cluster connection not found (cluster resharding)
    /// - Server errors that are transient (BusyLoading, TryAgain, MasterDown, ClusterDown)
    fn is_retriable_error(e: &redis::RedisError) -> bool {
        use redis::ErrorKind;

        match e.kind() {
            // I/O errors are retriable (network issues, connection drops)
            ErrorKind::Io => true,
            // Cluster-related transient errors
            ErrorKind::ClusterConnectionNotFound => true,
            // Server-side transient errors
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
            // All other errors are not retriable
            _ => false,
        }
    }

    // ========================================================================
    // Version Management
    // ========================================================================

    /// Get the current version from cache
    /// Uses retry with exponential backoff for high concurrency resilience
    pub async fn get_version(&self, doc_id: Uuid) -> Result<Option<i64>> {
        let key = Self::version_key(doc_id);

        let version: Option<i64> = self.with_retry("get_version", |mut conn| {
            let key = key.clone();
            async move { conn.get(&key).await }
        }).await?;

        debug!("Cache get_version: doc_id={}, version={:?}", doc_id, version);
        Ok(version)
    }

    /// Set the current version in cache
    /// Uses retry with exponential backoff for high concurrency resilience
    pub async fn set_version(&self, doc_id: Uuid, version: i64) -> Result<()> {
        let key = Self::version_key(doc_id);
        let ttl = self.config.ttl_seconds;

        self.with_retry("set_version", |mut conn| {
            let key = key.clone();
            async move { conn.set_ex::<_, _, ()>(&key, version, ttl).await }
        }).await?;

        debug!(
            "Cache set_version: doc_id={}, version={}",
            doc_id, version
        );
        Ok(())
    }

    // ========================================================================
    // Idempotency Cache (fast duplicate detection)
    // ========================================================================

    /// Check if an operation was already processed (idempotency check)
    /// Returns Some(rev) if the operation exists, None otherwise
    ///
    /// This is much faster than DB lookup (~1ms vs 10-200ms), reducing load
    /// on the database connection pool during high-concurrency scenarios.
    pub async fn check_idempotency(
        &self,
        doc_id: Uuid,
        client_id: &str,
        op_id: &str,
    ) -> Result<Option<i64>> {
        let key = Self::idempotency_key(doc_id);
        let field = Self::idempotency_field(client_id, op_id);

        let rev: Option<i64> = self.with_retry("check_idempotency", |mut conn| {
            let key = key.clone();
            let field = field.clone();
            async move { conn.hget(&key, &field).await }
        }).await?;

        if rev.is_some() {
            debug!(
                "Cache idempotency hit: doc_id={}, client_id={}, op_id={}, rev={:?}",
                doc_id, client_id, op_id, rev
            );
            metrics::increment_writebehind_cache_hits();
        }

        Ok(rev)
    }

    /// Set idempotency entry for an operation (called after successful processing)
    /// Uses retry with exponential backoff for high concurrency resilience
    pub async fn set_idempotency(
        &self,
        doc_id: Uuid,
        client_id: &str,
        op_id: &str,
        rev: i64,
    ) -> Result<()> {
        let key = Self::idempotency_key(doc_id);
        let field = Self::idempotency_field(client_id, op_id);
        let ttl = self.config.ttl_seconds;

        // Use HSET + EXPIRE atomically via Lua script
        let script = redis::Script::new(
            r#"
            redis.call('HSET', KEYS[1], ARGV[1], ARGV[2])
            redis.call('EXPIRE', KEYS[1], ARGV[3])
            return 1
            "#,
        );

        self.with_retry("set_idempotency", |mut conn| {
            let key = key.clone();
            let field = field.clone();
            let script = script.clone();
            async move {
                script
                    .key(&key)
                    .arg(&field)
                    .arg(rev)
                    .arg(ttl)
                    .invoke_async::<i64>(&mut conn)
                    .await
            }
        }).await?;

        debug!(
            "Cache set_idempotency: doc_id={}, client_id={}, op_id={}, rev={}",
            doc_id, client_id, op_id, rev
        );
        Ok(())
    }

    // ========================================================================
    // Operations Management
    // ========================================================================

    /// Write operations to cache atomically using a Lua script
    ///
    /// This atomically:
    /// 1. Updates the version
    /// 2. Adds all operations to the sorted set
    /// 3. Populates idempotency hash for fast duplicate detection
    /// 4. Sets TTL on all keys
    ///
    /// Uses retry with exponential backoff for high concurrency resilience
    pub async fn write_ops(
        &self,
        doc_id: Uuid,
        ops: &[OperationEntry],
        new_version: i64,
    ) -> Result<()> {
        if ops.is_empty() {
            return Ok(());
        }

        let version_key = Self::version_key(doc_id);
        let ops_key = Self::ops_key(doc_id);
        let idempotency_key = Self::idempotency_key(doc_id);
        let ttl_seconds = self.config.ttl_seconds;

        // Pre-serialize operations to avoid repeated serialization in retry loop
        // Format: [new_version, ttl, ops_count, rev1, data1, field1, rev2, data2, field2, ...]
        let ops_count = ops.len();
        let mut args: Vec<String> = vec![
            new_version.to_string(),
            ttl_seconds.to_string(),
            ops_count.to_string(),
        ];

        for op in ops {
            let serialized =
                serde_json::to_string(op).context("Failed to serialize operation")?;
            let idempotency_field = Self::idempotency_field(&op.client_id, &op.op_id);
            args.push(op.rev.to_string());
            args.push(serialized);
            args.push(idempotency_field);
        }

        // Use Lua script for atomicity - updates ops, version, AND idempotency hash
        let script = redis::Script::new(
            r#"
            local version_key = KEYS[1]
            local ops_key = KEYS[2]
            local idempotency_key = KEYS[3]
            local new_version = tonumber(ARGV[1])
            local ttl = tonumber(ARGV[2])
            local ops_count = tonumber(ARGV[3])

            -- Set version
            redis.call('SET', version_key, new_version, 'EX', ttl)

            -- Add operations to sorted set and idempotency hash
            for i = 1, ops_count do
                local base = 3 + (i-1)*3
                local rev = tonumber(ARGV[base + 1])
                local data = ARGV[base + 2]
                local field = ARGV[base + 3]

                -- Add to operations sorted set
                redis.call('ZADD', ops_key, rev, data)

                -- Add to idempotency hash (client_id:op_id -> rev)
                redis.call('HSET', idempotency_key, field, rev)
            end

            -- Set TTL on ops and idempotency keys
            redis.call('EXPIRE', ops_key, ttl)
            redis.call('EXPIRE', idempotency_key, ttl)

            return ops_count
            "#,
        );

        let result: i64 = self.with_retry("write_ops", |mut conn| {
            let version_key = version_key.clone();
            let ops_key = ops_key.clone();
            let idempotency_key = idempotency_key.clone();
            let args = args.clone();
            let script = script.clone();
            async move {
                script
                    .key(&version_key)
                    .key(&ops_key)
                    .key(&idempotency_key)
                    .arg(args)
                    .invoke_async(&mut conn)
                    .await
            }
        }).await?;

        debug!(
            "Cache write_ops: doc_id={}, ops_count={}, new_version={}",
            doc_id, result, new_version
        );
        Ok(())
    }

    /// Get operations since a specific revision
    /// Uses retry with exponential backoff for high concurrency resilience
    pub async fn get_ops_since(
        &self,
        doc_id: Uuid,
        since_rev: i64,
    ) -> Result<Vec<CachedOperationInfo>> {
        let ops_key = Self::ops_key(doc_id);
        let min_score = (since_rev + 1) as f64;

        // Get all operations with score > since_rev
        let ops: Vec<(String, f64)> = self.with_retry("get_ops_since", |mut conn| {
            let ops_key = ops_key.clone();
            async move {
                conn.zrangebyscore_withscores(
                    &ops_key,
                    min_score,
                    "+inf",
                ).await
            }
        }).await?;

        let mut result = Vec::with_capacity(ops.len());
        for (data, _score) in ops {
            let entry: OperationEntry =
                serde_json::from_str(&data).context("Failed to deserialize cached operation")?;

            result.push(CachedOperationInfo {
                rev: entry.rev,
                user_id: entry.user_id,
                mutation_id: entry.mutation_id,
                params: entry.params,
                client_id: entry.client_id,
                op_id: entry.op_id,
                created_at: chrono::DateTime::from_timestamp_millis(entry.created_at)
                    .unwrap_or_else(chrono::Utc::now),
            });
        }

        debug!(
            "Cache get_ops_since: doc_id={}, since_rev={}, count={}",
            doc_id,
            since_rev,
            result.len()
        );
        Ok(result)
    }

    /// Get operations within a revision range for flushing
    /// Uses retry with exponential backoff for high concurrency resilience
    pub async fn get_ops_range(
        &self,
        doc_id: Uuid,
        from_rev: i64,
        to_rev: i64,
    ) -> Result<Vec<OperationEntry>> {
        let ops_key = Self::ops_key(doc_id);
        let min_score = from_rev as f64;
        let max_score = to_rev as f64;

        let ops: Vec<(String, f64)> = self.with_retry("get_ops_range", |mut conn| {
            let ops_key = ops_key.clone();
            async move {
                conn.zrangebyscore_withscores(&ops_key, min_score, max_score).await
            }
        }).await?;

        let mut result = Vec::with_capacity(ops.len());
        for (data, _score) in ops {
            let entry: OperationEntry =
                serde_json::from_str(&data).context("Failed to deserialize cached operation")?;
            result.push(entry);
        }

        debug!(
            "Cache get_ops_range: doc_id={}, from={}, to={}, count={}",
            doc_id,
            from_rev,
            to_rev,
            result.len()
        );
        Ok(result)
    }

    /// Remove operations up to a specific revision (after successful flush)
    /// Uses retry with exponential backoff for high concurrency resilience
    pub async fn remove_ops_up_to(&self, doc_id: Uuid, max_rev: i64) -> Result<usize> {
        let ops_key = Self::ops_key(doc_id);
        let max_score = max_rev as f64;

        // Remove all entries with score <= max_rev
        let removed: usize = self.with_retry("remove_ops_up_to", |mut conn| {
            let ops_key = ops_key.clone();
            async move { conn.zrembyscore(&ops_key, "-inf", max_score).await }
        }).await?;

        debug!(
            "Cache remove_ops_up_to: doc_id={}, max_rev={}, removed={}",
            doc_id, max_rev, removed
        );
        Ok(removed)
    }

    /// Get the minimum revision in the cache for a document
    pub async fn get_min_cached_rev(&self, doc_id: Uuid) -> Result<Option<i64>> {
        // Get connection from pool
        let mut conn = self.get_connection();

        let ops_key = Self::ops_key(doc_id);

        // Get the first element (lowest score)
        let result: Vec<(String, f64)> = conn
            .zrange_withscores(&ops_key, 0, 0)
            .await
            .context("Failed to get min revision from cache")?;

        let min_rev = result.first().map(|(_, score)| *score as i64);
        debug!(
            "Cache get_min_cached_rev: doc_id={}, min_rev={:?}",
            doc_id, min_rev
        );
        Ok(min_rev)
    }

    /// Get the maximum revision in the cache for a document (pending ops)
    /// This is useful when the version key has expired but ops are still cached
    /// Returns None if no pending ops exist
    pub async fn get_max_cached_rev(&self, doc_id: Uuid) -> Result<Option<i64>> {
        let ops_key = Self::ops_key(doc_id);

        // Get the last element (highest score) using ZREVRANGE
        let result: Vec<(String, f64)> = self.with_retry("get_max_cached_rev", |mut conn| {
            let ops_key = ops_key.clone();
            async move {
                conn.zrevrange_withscores(&ops_key, 0, 0).await
            }
        }).await?;

        let max_rev = result.first().map(|(_, score)| *score as i64);
        debug!(
            "Cache get_max_cached_rev: doc_id={}, max_rev={:?}",
            doc_id, max_rev
        );
        Ok(max_rev)
    }

    /// Get count of pending operations in cache
    pub async fn get_pending_ops_count(&self, doc_id: Uuid) -> Result<usize> {
        // Get connection from pool
        let mut conn = self.get_connection();

        let ops_key = Self::ops_key(doc_id);
        let count: usize = conn
            .zcard(&ops_key)
            .await
            .context("Failed to get pending ops count")?;

        Ok(count)
    }

    // ========================================================================
    // Queue Management
    // ========================================================================

    /// Enqueue a document for background flushing
    ///
    /// Always enqueues the document - the flush lock prevents duplicate flushes.
    /// We removed the dedup SET check because it caused a race condition:
    /// Worker could BRPOP (removing from queue) while SET still had the doc,
    /// causing subsequent enqueue attempts to be silently dropped.
    ///
    /// Uses retry with exponential backoff for high concurrency resilience
    pub async fn enqueue_doc(&self, doc_id: &str) -> Result<bool> {
        let doc_id_owned = doc_id.to_string();

        // Always add to queue - the flush lock prevents duplicate flushes
        // Don't use dedup SET here due to race condition with BRPOP
        self.with_retry("enqueue_doc", |mut conn| {
            let doc_id = doc_id_owned.clone();
            async move { conn.lpush::<_, _, ()>(KEY_WRITEBEHIND_QUEUE, &doc_id).await }
        }).await?;

        // Notify waiting workers that a new document is available
        self.enqueue_notify.notify_one();

        debug!("Cache enqueue_doc: doc_id={} added to queue", doc_id);
        Ok(true)
    }

    /// Dequeue a document for flushing (blocking with timeout)
    pub async fn dequeue_doc(&self, timeout_ms: u64) -> Result<Option<String>> {
        // Get connection from pool
        let mut conn = self.get_connection();

        // BRPOP with timeout (in seconds, minimum 1)
        // Use integer timeout for better Redis compatibility
        let timeout_secs = ((timeout_ms / 1000).max(1)) as usize;

        // Use redis::cmd directly for more control over BRPOP
        let result: Option<(String, String)> = match redis::cmd("BRPOP")
            .arg(KEY_WRITEBEHIND_QUEUE)
            .arg(timeout_secs)
            .query_async(&mut conn)
            .await
        {
            Ok(r) => r,
            Err(e) => {
                // Check if it's a timeout (which is normal) vs actual error
                if e.is_timeout() {
                    return Ok(None);
                }
                return Err(anyhow::anyhow!("BRPOP failed: {} (kind: {:?})", e, e.kind()));
            }
        };

        match result {
            Some((_, doc_id)) => {
                debug!("Cache dequeue_doc: got doc_id={}", doc_id);
                Ok(Some(doc_id))
            }
            None => {
                // Timeout, no document available
                Ok(None)
            }
        }
    }

    /// Dequeue a document for flushing (non-blocking)
    ///
    /// Uses RPOP instead of BRPOP to avoid blocking the connection.
    /// Returns immediately with None if the queue is empty.
    /// Workers should use wait_for_enqueue() to wait for notifications.
    pub async fn dequeue_doc_nonblocking(&self) -> Result<Option<String>> {
        let result: Option<String> = self.with_retry("dequeue_doc_nonblocking", |mut conn| {
            async move { conn.rpop(KEY_WRITEBEHIND_QUEUE, None).await }
        }).await?;

        if let Some(ref doc_id) = result {
            debug!("Cache dequeue_doc_nonblocking: got doc_id={}", doc_id);
        }
        Ok(result)
    }

    /// Wait for a document to be enqueued
    ///
    /// Returns a future that completes when enqueue_doc() is called.
    /// Use with tokio::select! for non-blocking worker loops.
    pub async fn wait_for_enqueue(&self) {
        self.enqueue_notify.notified().await
    }

    /// Get the current queue depth
    pub async fn get_queue_depth(&self) -> Result<usize> {
        // Get connection from pool
        let mut conn = self.get_connection();

        let depth: usize = conn
            .llen(KEY_WRITEBEHIND_QUEUE)
            .await
            .context("Failed to get queue depth")?;

        Ok(depth)
    }

    /// Get all documents currently in the queue (for recovery)
    /// Returns unique doc IDs from the queue
    pub async fn get_all_pending_docs(&self) -> Result<Vec<String>> {
        // Get connection from pool
        let mut conn = self.get_connection();

        // Get all items from queue without removing them
        let docs: Vec<String> = conn
            .lrange(KEY_WRITEBEHIND_QUEUE, 0, -1)
            .await
            .context("Failed to get pending docs from queue")?;

        // Deduplicate (same doc may be in queue multiple times)
        let unique_docs: Vec<String> = docs
            .into_iter()
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        debug!("Cache get_all_pending_docs: count={}", unique_docs.len());
        Ok(unique_docs)
    }

    // ========================================================================
    // Lock Management
    // ========================================================================

    /// Acquire a flush lock for a document (prevents concurrent flushes)
    /// Uses retry with exponential backoff for high concurrency resilience
    pub async fn acquire_flush_lock(&self, doc_id: &str, ttl_ms: u64) -> Result<bool> {
        let lock_key = Self::lock_key(doc_id);

        // SET NX with expiry (atomic lock acquisition)
        let result: Option<String> = self.with_retry("acquire_flush_lock", |mut conn| {
            let lock_key = lock_key.clone();
            async move {
                redis::cmd("SET")
                    .arg(&lock_key)
                    .arg("1")
                    .arg("NX")
                    .arg("PX")
                    .arg(ttl_ms)
                    .query_async(&mut conn)
                    .await
            }
        }).await?;

        let acquired = result.is_some();
        debug!(
            "Cache acquire_flush_lock: doc_id={}, acquired={}",
            doc_id, acquired
        );
        Ok(acquired)
    }

    /// Release a flush lock
    /// Uses retry with exponential backoff for high concurrency resilience
    pub async fn release_flush_lock(&self, doc_id: &str) -> Result<()> {
        let lock_key = Self::lock_key(doc_id);

        self.with_retry("release_flush_lock", |mut conn| {
            let lock_key = lock_key.clone();
            async move { conn.del::<_, ()>(&lock_key).await }
        }).await?;

        debug!("Cache release_flush_lock: doc_id={}", doc_id);
        Ok(())
    }

    // ========================================================================
    // Params Caching (for S3-stored operation params)
    // ========================================================================

    /// Get operation params from Redis cache
    /// Returns None if not cached (caller should fetch from S3 and cache)
    /// Uses retry with exponential backoff for high concurrency resilience
    pub async fn get_params_cached(&self, storage_id: Uuid) -> Result<Option<Vec<u8>>> {
        let key = Self::params_key(storage_id);

        let params: Option<Vec<u8>> = self.with_retry("get_params_cached", |mut conn| {
            let key = key.clone();
            async move { conn.get(&key).await }
        }).await?;

        if params.is_some() {
            debug!("Cache hit for params: storage_id={}", storage_id);
        }
        Ok(params)
    }

    /// Cache operation params in Redis with TTL
    /// Called after fetching from S3 to populate cache
    /// Uses retry with exponential backoff for high concurrency resilience
    pub async fn cache_params(&self, storage_id: Uuid, params: &[u8]) -> Result<()> {
        let key = Self::params_key(storage_id);
        let params_vec = params.to_vec(); // Clone once for retry loop

        self.with_retry("cache_params", |mut conn| {
            let key = key.clone();
            let params = params_vec.clone();
            async move { conn.set_ex::<_, _, ()>(&key, &params, PARAMS_CACHE_TTL_SECONDS).await }
        }).await?;

        debug!(
            "Cache cached params: storage_id={}, size={}",
            storage_id,
            params.len()
        );
        Ok(())
    }

    /// Batch get params from Redis cache
    /// Returns a map of storage_id -> params for cache hits
    pub async fn get_params_cached_batch(
        &self,
        storage_ids: &[Uuid],
    ) -> Result<std::collections::HashMap<Uuid, Vec<u8>>> {
        if storage_ids.is_empty() {
            return Ok(std::collections::HashMap::new());
        }

        // Get connection from pool
        let mut conn = self.get_connection();

        let keys: Vec<String> = storage_ids.iter().map(|id| Self::params_key(*id)).collect();

        // Use MGET for batch retrieval
        let values: Vec<Option<Vec<u8>>> = conn
            .mget(&keys)
            .await
            .context("Failed to batch get params from cache")?;

        let mut result = std::collections::HashMap::new();
        for (storage_id, value) in storage_ids.iter().zip(values.into_iter()) {
            if let Some(params) = value {
                result.insert(*storage_id, params);
            }
        }

        debug!(
            "Cache batch get params: requested={}, hits={}",
            storage_ids.len(),
            result.len()
        );
        Ok(result)
    }

    // ========================================================================
    // Utility Methods
    // ========================================================================

    /// Check if cache is healthy (can connect to Redis)
    pub async fn health_check(&self) -> Result<bool> {
        // Get connection from pool
        let mut conn = self.get_connection();

        let pong: String = redis::cmd("PING")
            .query_async(&mut conn)
            .await
            .context("Redis health check failed")?;

        Ok(pong == "PONG")
    }

    /// Clear all cache data for a document (for testing or cleanup)
    pub async fn clear_doc_cache(&self, doc_id: Uuid) -> Result<()> {
        // Get connection from pool
        let mut conn = self.get_connection();

        let version_key = Self::version_key(doc_id);
        let ops_key = Self::ops_key(doc_id);

        let _: () = conn
            .del(&[&version_key, &ops_key])
            .await
            .context("Failed to clear document cache")?;

        debug!("Cache clear_doc_cache: doc_id={}", doc_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests would require a Redis instance - typically run in integration tests
    // These are placeholder tests that verify struct creation

    #[test]
    fn test_operation_entry_serialization() {
        let entry = OperationEntry {
            rev: 42,
            user_id: "user-123".to_string(),
            mutation_id: "sheet.mutation.set-range-values".to_string(),
            params: vec![1, 2, 3, 4], // MessagePack encoded params
            client_id: "client-456".to_string(),
            op_id: "op-789".to_string(),
            created_at: 1704067200000,
        };

        let json = serde_json::to_string(&entry).unwrap();
        let decoded: OperationEntry = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded.rev, entry.rev);
        assert_eq!(decoded.user_id, entry.user_id);
        assert_eq!(decoded.op_id, entry.op_id);
        assert_eq!(decoded.params, entry.params);
    }

    #[test]
    fn test_cache_config_default() {
        let config = CacheConfig::default();
        assert_eq!(config.ttl_seconds, 3600);
        assert_eq!(config.batch_size, 1000);
    }

    #[test]
    fn test_key_generation() {
        let doc_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();

        let version_key = CacheService::version_key(doc_id);
        assert_eq!(
            version_key,
            "ot:doc:550e8400-e29b-41d4-a716-446655440000:version"
        );

        let ops_key = CacheService::ops_key(doc_id);
        assert_eq!(
            ops_key,
            "ot:doc:550e8400-e29b-41d4-a716-446655440000:ops"
        );

        let lock_key = CacheService::lock_key("550e8400-e29b-41d4-a716-446655440000");
        assert_eq!(
            lock_key,
            "ot:writebehind:lock:550e8400-e29b-41d4-a716-446655440000"
        );
    }
}
