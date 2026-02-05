//! Cache service for write-behind caching optimization
//!
//! This module provides a Redis-based caching layer for OT operations that enables
//! asynchronous batch writes to the database. Operations are first written to Redis
//! (fast, 1-5ms) and then flushed to PostgreSQL in batches by background workers.
//!
//! ## Redis Data Structures
//!
//! - `ot:doc:{doc_id}:version` - String containing current (logical) version number
//! - `ot:doc:{doc_id}:db_version` - String containing last persisted DB version (write-behind optimization)
//! - `ot:doc:{doc_id}:ops` - Sorted Set for operation index (member=opId, score=rev)
//! - `ot:doc:{doc_id}:ops:data` - Hash for operation data (field=opId, value=Zstd compressed JSON)
//! - `ot:writebehind:queue` - List of document IDs pending flush
//! - `ot:writebehind:lock:{doc_id}` - Lock to prevent concurrent flushes
//!
//! ## Idempotency Design
//!
//! Uses opId as natural idempotency key:
//! - opId is client-generated unique identifier for each operation
//! - ZADD with same opId automatically overwrites (no duplicates)
//! - ZSCORE check replaces separate idempotency Hash (simpler, less memory)
//!
//! ## Version Concepts
//!
//! - `version` (cached_version): The latest logical version including pending ops not yet flushed
//! - `db_version`: The last version successfully persisted to PostgreSQL
//!
//! The difference (cached_version - db_version) represents operations pending flush.
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
use crate::services::compression::{deserialize_smart, serialize_smart};
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
    pub params: Vec<u8>, // Raw params bytes - uploaded to S3 by write-behind worker if large
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
    pub params: Vec<u8>, // Raw params bytes
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
const KEY_SUFFIX_OPS_DATA: &str = ":ops:data"; // Hash for opId -> Zstd compressed JSON data
const KEY_SUFFIX_DB_VERSION: &str = ":db_version"; // Persisted DB version (for write-behind optimization)
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

    /// Get the operations index key for a document (Sorted Set: member=opId, score=rev)
    fn ops_key(doc_id: Uuid) -> String {
        format!("{}{}{}", KEY_PREFIX_OPS, doc_id, KEY_SUFFIX_OPS)
    }

    /// Get the operations data key for a document (Hash: field=opId, value=Zstd JSON)
    fn ops_data_key(doc_id: Uuid) -> String {
        format!("{}{}{}", KEY_PREFIX_OPS, doc_id, KEY_SUFFIX_OPS_DATA)
    }

    /// Get the flush lock key for a document
    fn lock_key(doc_id: &str) -> String {
        format!("{}{}", KEY_PREFIX_LOCK, doc_id)
    }

    /// Get the params cache key for a storage_id
    fn params_key(storage_id: Uuid) -> String {
        format!("{}{}", KEY_PREFIX_PARAMS, storage_id)
    }

    /// Get the DB version key for a document
    /// This stores the last flushed (persisted) version, separate from cached version
    fn db_version_key(doc_id: Uuid) -> String {
        format!("{}{}{}", KEY_PREFIX_VERSION, doc_id, KEY_SUFFIX_DB_VERSION)
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
    // DB Version Management (Write-Behind Optimization)
    // ========================================================================

    /// Get the persisted DB version from cache
    ///
    /// This is the version that has been flushed to PostgreSQL, used by write-behind
    /// workers to avoid querying the database on every flush cycle.
    ///
    /// Returns None if:
    /// - Key doesn't exist (cold start, needs DB query to initialize)
    /// - Key expired (rare, uses longer TTL than cached version)
    pub async fn get_db_version(&self, doc_id: Uuid) -> Result<Option<i64>> {
        let key = Self::db_version_key(doc_id);

        let version: Option<i64> = self
            .with_retry("get_db_version", |mut conn| {
                let key = key.clone();
                async move { conn.get(&key).await }
            })
            .await?;

        debug!(
            "Cache get_db_version: doc_id={}, version={:?}",
            doc_id, version
        );
        Ok(version)
    }

    /// Set the persisted DB version in cache
    ///
    /// Called after successful flush to PostgreSQL. Uses a longer TTL (24 hours)
    /// since this represents durable state and only changes on successful flushes.
    ///
    /// If this key expires, the write-behind worker will fall back to querying
    /// the database once and re-caching the result.
    pub async fn set_db_version(&self, doc_id: Uuid, version: i64) -> Result<()> {
        let key = Self::db_version_key(doc_id);
        // Use 24 hour TTL - longer than cached_version since DB version changes less frequently
        // and represents durable state. Expiry just means one DB query on next flush.
        let ttl = 86400u64;

        self.with_retry("set_db_version", |mut conn| {
            let key = key.clone();
            async move { conn.set_ex::<_, _, ()>(&key, version, ttl).await }
        })
        .await?;

        debug!(
            "Cache set_db_version: doc_id={}, version={}",
            doc_id, version
        );
        Ok(())
    }

    // ========================================================================
    // Idempotency Check (using Sorted Set ZSCORE)
    // ========================================================================

    /// Check if an operation was already processed (idempotency check)
    /// Returns Some(rev) if the operation exists, None otherwise
    ///
    /// Uses ZSCORE on the ops Sorted Set instead of a separate idempotency Hash.
    /// This is more efficient (one less key) and naturally idempotent.
    ///
    /// This is much faster than DB lookup (~1ms vs 10-200ms), reducing load
    /// on the database connection pool during high-concurrency scenarios.
    pub async fn check_idempotency(
        &self,
        doc_id: Uuid,
        op_id: &str,
    ) -> Result<Option<i64>> {
        let ops_key = Self::ops_key(doc_id);
        let op_id_owned = op_id.to_string();

        // ZSCORE returns the score (rev) if member exists, None otherwise
        let rev: Option<f64> = self.with_retry("check_idempotency", |mut conn| {
            let key = ops_key.clone();
            let op_id = op_id_owned.clone();
            async move { conn.zscore(&key, &op_id).await }
        }).await?;

        let rev = rev.map(|r| r as i64);

        if rev.is_some() {
            debug!(
                "Cache idempotency hit: doc_id={}, op_id={}, rev={:?}",
                doc_id, op_id, rev
            );
            metrics::increment_writebehind_cache_hits();
        }

        Ok(rev)
    }

    // ========================================================================
    // Operations Management
    // ========================================================================

    /// Write operations to cache atomically using a Lua script
    ///
    /// This atomically:
    /// 1. Updates the version
    /// 2. Adds all operations to the Sorted Set (member=opId, score=rev)
    /// 3. Stores operation data in Hash (field=opId, value=Zstd compressed JSON)
    /// 4. Sets TTL on all keys
    ///
    /// The opId-as-member design provides natural idempotency:
    /// - ZADD with same opId will overwrite (no duplicates)
    /// - HSET with same opId will overwrite
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
        let ops_data_key = Self::ops_data_key(doc_id);
        let ttl_seconds = self.config.ttl_seconds;

        // Pre-serialize operations with Zstd compression
        // Format: [new_version, ttl, ops_count, op_id1, rev1, data1, op_id2, rev2, data2, ...]
        let ops_count = ops.len();
        let mut args: Vec<Vec<u8>> = vec![
            new_version.to_string().into_bytes(),
            ttl_seconds.to_string().into_bytes(),
            ops_count.to_string().into_bytes(),
        ];

        for op in ops {
            // Serialize with smart compression (JSON + Zstd for large data)
            let serialized = serialize_smart(op).context("Failed to serialize operation")?;
            args.push(op.op_id.clone().into_bytes()); // opId as member
            args.push(op.rev.to_string().into_bytes());
            args.push(serialized); // Zstd compressed JSON
        }

        // Use Lua script for atomicity - Sorted Set + Hash structure
        let script = redis::Script::new(
            r#"
            local version_key = KEYS[1]
            local ops_key = KEYS[2]
            local ops_data_key = KEYS[3]
            local new_version = tonumber(ARGV[1])
            local ttl = tonumber(ARGV[2])
            local ops_count = tonumber(ARGV[3])

            -- Set version
            redis.call('SET', version_key, new_version, 'EX', ttl)

            -- Add operations to Sorted Set (opId as member) and Hash (opId -> data)
            for i = 1, ops_count do
                local base = 3 + (i-1)*3
                local op_id = ARGV[base + 1]
                local rev = tonumber(ARGV[base + 2])
                local data = ARGV[base + 3]

                -- Sorted Set: opId as member, rev as score (idempotent - ZADD overwrites)
                redis.call('ZADD', ops_key, rev, op_id)

                -- Hash: opId -> Zstd compressed JSON data
                redis.call('HSET', ops_data_key, op_id, data)
            end

            -- Set TTL on ops index and data keys
            redis.call('EXPIRE', ops_key, ttl)
            redis.call('EXPIRE', ops_data_key, ttl)

            return ops_count
            "#,
        );

        let result: i64 = self.with_retry("write_ops", |mut conn| {
            let version_key = version_key.clone();
            let ops_key = ops_key.clone();
            let ops_data_key = ops_data_key.clone();
            let args = args.clone();
            let script = script.clone();
            async move {
                script
                    .key(&version_key)
                    .key(&ops_key)
                    .key(&ops_data_key)
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
    ///
    /// Uses optimized Lua script to:
    /// 1. Get opId list from Sorted Set (by score range)
    /// 2. Batch get data from Hash (HMGET)
    /// 3. Return in single RTT
    ///
    /// Uses retry with exponential backoff for high concurrency resilience
    pub async fn get_ops_since(
        &self,
        doc_id: Uuid,
        since_rev: i64,
    ) -> Result<Vec<CachedOperationInfo>> {
        let ops_key = Self::ops_key(doc_id);
        let ops_data_key = Self::ops_data_key(doc_id);
        let min_score = since_rev + 1;

        // Lua script: ZRANGEBYSCORE + HMGET in single RTT
        let script = redis::Script::new(
            r#"
            local ops_key = KEYS[1]
            local data_key = KEYS[2]
            local min_score = ARGV[1]

            -- Get opId list by score range
            local op_ids = redis.call('ZRANGEBYSCORE', ops_key, min_score, '+inf')

            if #op_ids == 0 then
                return {}
            end

            -- Batch get data from Hash
            local data = redis.call('HMGET', data_key, unpack(op_ids))

            -- Return non-nil data
            local result = {}
            for i, d in ipairs(data) do
                if d then
                    table.insert(result, d)
                end
            end
            return result
            "#,
        );

        let data: Vec<Vec<u8>> = self.with_retry("get_ops_since", |mut conn| {
            let ops_key = ops_key.clone();
            let ops_data_key = ops_data_key.clone();
            let script = script.clone();
            async move {
                script
                    .key(&ops_key)
                    .key(&ops_data_key)
                    .arg(min_score)
                    .invoke_async(&mut conn)
                    .await
            }
        }).await?;

        // Deserialize with Zstd decompression
        let mut result = Vec::with_capacity(data.len());
        for bytes in data {
            let entry: OperationEntry =
                deserialize_smart(&bytes).context("Failed to deserialize cached operation")?;

            result.push(CachedOperationInfo {
                rev: entry.rev,
                user_id: entry.user_id,
                mutation_id: entry.mutation_id,
                params: entry.params,
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
    ///
    /// Uses optimized Lua script to:
    /// 1. Get opId list from Sorted Set (by score range)
    /// 2. Batch get data from Hash (HMGET)
    /// 3. Return in single RTT
    ///
    /// Uses retry with exponential backoff for high concurrency resilience
    pub async fn get_ops_range(
        &self,
        doc_id: Uuid,
        from_rev: i64,
        to_rev: i64,
    ) -> Result<Vec<OperationEntry>> {
        let ops_key = Self::ops_key(doc_id);
        let ops_data_key = Self::ops_data_key(doc_id);

        // Lua script: ZRANGEBYSCORE + HMGET in single RTT
        let script = redis::Script::new(
            r#"
            local ops_key = KEYS[1]
            local data_key = KEYS[2]
            local min_score = ARGV[1]
            local max_score = ARGV[2]

            -- Get opId list by score range
            local op_ids = redis.call('ZRANGEBYSCORE', ops_key, min_score, max_score)

            if #op_ids == 0 then
                return {}
            end

            -- Batch get data from Hash
            local data = redis.call('HMGET', data_key, unpack(op_ids))

            -- Return non-nil data
            local result = {}
            for i, d in ipairs(data) do
                if d then
                    table.insert(result, d)
                end
            end
            return result
            "#,
        );

        let data: Vec<Vec<u8>> = self.with_retry("get_ops_range", |mut conn| {
            let ops_key = ops_key.clone();
            let ops_data_key = ops_data_key.clone();
            let script = script.clone();
            async move {
                script
                    .key(&ops_key)
                    .key(&ops_data_key)
                    .arg(from_rev)
                    .arg(to_rev)
                    .invoke_async(&mut conn)
                    .await
            }
        }).await?;

        // Deserialize with Zstd decompression
        let mut result = Vec::with_capacity(data.len());
        for bytes in data {
            let entry: OperationEntry =
                deserialize_smart(&bytes).context("Failed to deserialize cached operation")?;
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
    ///
    /// Atomically removes from both:
    /// - Sorted Set (index): ZREMRANGEBYSCORE
    /// - Hash (data): HDEL for each removed opId
    ///
    /// Uses retry with exponential backoff for high concurrency resilience
    pub async fn remove_ops_up_to(&self, doc_id: Uuid, max_rev: i64) -> Result<usize> {
        let ops_key = Self::ops_key(doc_id);
        let ops_data_key = Self::ops_data_key(doc_id);

        // Lua script: get opIds first, then delete from both Hash and Sorted Set
        let script = redis::Script::new(
            r#"
            local ops_key = KEYS[1]
            local data_key = KEYS[2]
            local max_rev = ARGV[1]

            -- Get opIds to be deleted (score <= max_rev)
            local op_ids = redis.call('ZRANGEBYSCORE', ops_key, '-inf', max_rev)

            if #op_ids > 0 then
                -- Delete from Hash first
                redis.call('HDEL', data_key, unpack(op_ids))
                -- Delete from Sorted Set
                redis.call('ZREMRANGEBYSCORE', ops_key, '-inf', max_rev)
            end

            return #op_ids
            "#,
        );

        let removed: i64 = self.with_retry("remove_ops_up_to", |mut conn| {
            let ops_key = ops_key.clone();
            let ops_data_key = ops_data_key.clone();
            let script = script.clone();
            async move {
                script
                    .key(&ops_key)
                    .key(&ops_data_key)
                    .arg(max_rev)
                    .invoke_async(&mut conn)
                    .await
            }
        }).await?;

        debug!(
            "Cache remove_ops_up_to: doc_id={}, max_rev={}, removed={}",
            doc_id, max_rev, removed
        );
        Ok(removed as usize)
    }

    /// Get the minimum revision in the cache for a document
    /// Uses ZRANGE with WITHSCORES to get the first (lowest score) element
    pub async fn get_min_cached_rev(&self, doc_id: Uuid) -> Result<Option<i64>> {
        let ops_key = Self::ops_key(doc_id);

        // Get the first element (lowest score) - returns (opId, score) pairs
        let result: Vec<(String, f64)> = self.with_retry("get_min_cached_rev", |mut conn| {
            let ops_key = ops_key.clone();
            async move {
                conn.zrange_withscores(&ops_key, 0, 0).await
            }
        }).await?;

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

        // Get the last element (highest score) using ZREVRANGE - returns (opId, score) pairs
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
        let version_key = Self::version_key(doc_id);
        let ops_key = Self::ops_key(doc_id);
        let ops_data_key = Self::ops_data_key(doc_id);
        let db_version_key = Self::db_version_key(doc_id);

        self.with_retry("clear_doc_cache", |mut conn| {
            let keys = vec![
                version_key.clone(),
                ops_key.clone(),
                ops_data_key.clone(),
                db_version_key.clone(),
            ];
            async move { conn.del::<_, ()>(&keys).await }
        }).await?;

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
            params: vec![1, 2, 3, 4], // Raw params bytes
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
    fn test_operation_entry_smart_serialization() {
        // Test small data (no compression)
        let small_entry = OperationEntry {
            rev: 1,
            user_id: "u".to_string(),
            mutation_id: "m".to_string(),
            params: vec![1],
            op_id: "op-1".to_string(),
            created_at: 1704067200000,
        };

        let serialized = serialize_smart(&small_entry).unwrap();
        // Small data should not be compressed (flag byte = 0)
        assert_eq!(serialized[0], 0);

        let decoded: OperationEntry = deserialize_smart(&serialized).unwrap();
        assert_eq!(decoded.rev, small_entry.rev);
        assert_eq!(decoded.op_id, small_entry.op_id);

        // Test large data (should be compressed)
        let large_entry = OperationEntry {
            rev: 42,
            user_id: "user-123".to_string(),
            mutation_id: "sheet.mutation.set-range-values".to_string(),
            params: vec![0; 500], // Large params to trigger compression
            op_id: "op-789".to_string(),
            created_at: 1704067200000,
        };

        let serialized = serialize_smart(&large_entry).unwrap();
        // Large data should be compressed (flag byte = 1)
        assert_eq!(serialized[0], 1);

        let decoded: OperationEntry = deserialize_smart(&serialized).unwrap();
        assert_eq!(decoded.rev, large_entry.rev);
        assert_eq!(decoded.params.len(), large_entry.params.len());
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

        let ops_data_key = CacheService::ops_data_key(doc_id);
        assert_eq!(
            ops_data_key,
            "ot:doc:550e8400-e29b-41d4-a716-446655440000:ops:data"
        );

        let lock_key = CacheService::lock_key("550e8400-e29b-41d4-a716-446655440000");
        assert_eq!(
            lock_key,
            "ot:writebehind:lock:550e8400-e29b-41d4-a716-446655440000"
        );
    }
}
