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
//! - `ot:writebehind:lock:{doc_id}` - Lock to prevent concurrent flushes
//!
//! ## Queue Notifications
//!
//! Queue functionality (enqueue/dequeue for write-behind) is handled by
//! `StreamQueueService` using Redis Streams with Consumer Groups.

use super::types::{CacheConfig, CachedOperationInfo, OperationEntry};
use crate::compression::{deserialize_smart, serialize_smart};
use anyhow::{Context, Result};
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info, warn};
use uuid::Uuid;

// Redis key prefixes
const KEY_PREFIX_VERSION: &str = "ot:doc:";
const KEY_SUFFIX_VERSION: &str = ":version";
const KEY_PREFIX_OPS: &str = "ot:doc:";
const KEY_SUFFIX_OPS: &str = ":ops";
const KEY_SUFFIX_OPS_DATA: &str = ":ops:data"; // Hash for opId -> Zstd compressed JSON data
const KEY_SUFFIX_DB_VERSION: &str = ":db_version"; // Persisted DB version (for write-behind optimization)
const KEY_PREFIX_LOCK: &str = "ot:writebehind:lock:";
// Params cache key prefix for TTL caching of operation params
const KEY_PREFIX_PARAMS: &str = "ot:params:";
const PARAMS_CACHE_TTL_SECONDS: u64 = 3600; // 1 hour TTL for params cache

// Retry configuration for high concurrency resilience
const MAX_RETRIES: u32 = 3;
const INITIAL_BACKOFF_MS: u64 = 10;
const MAX_BACKOFF_MS: u64 = 100;
const OPERATION_TIMEOUT_MS: u64 = 3000;

// Connection pool configuration
const CONNECTION_POOL_SIZE: usize = 16;

/// Cache service for write-behind caching
///
/// Uses a pool of `ConnectionManager` instances for efficient Redis access:
/// - Multiple connections to handle high concurrency
/// - Automatic reconnection on connection loss
/// - Round-robin load balancing across connections
/// - Thread-safe (Send + Sync)
/// - Cross-process notification via Redis Pub/Sub
#[derive(Clone)]
pub struct CacheService {
    /// Pool of connection managers for high concurrency
    connections: Vec<ConnectionManager>,
    /// Counter for round-robin connection selection
    next_conn: Arc<std::sync::atomic::AtomicUsize>,
    config: CacheConfig,
    /// Redis client for creating Pub/Sub connections
    redis_client: redis::Client,
}

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
            redis_client,
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
    pub fn from_connection_managers(
        connections: Vec<ConnectionManager>,
        config: CacheConfig,
        redis_client: redis::Client,
    ) -> Self {
        Self {
            connections,
            next_conn: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            config,
            redis_client,
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

    /// Get a clone of the first connection manager for use in other services
    pub fn connection_manager(&self) -> ConnectionManager {
        self.connections[0].clone()
    }

    /// Get the Redis client for creating additional connections
    pub fn redis_client(&self) -> &redis::Client {
        &self.redis_client
    }

    /// Get a clone of all connection managers (for sharing with StreamQueueService)
    pub fn connection_managers(&self) -> Vec<ConnectionManager> {
        self.connections.clone()
    }

    // Key generation helpers
    fn version_key(doc_id: Uuid) -> String {
        format!("{}{}{}", KEY_PREFIX_VERSION, doc_id, KEY_SUFFIX_VERSION)
    }

    fn ops_key(doc_id: Uuid) -> String {
        format!("{}{}{}", KEY_PREFIX_OPS, doc_id, KEY_SUFFIX_OPS)
    }

    fn ops_data_key(doc_id: Uuid) -> String {
        format!("{}{}{}", KEY_PREFIX_OPS, doc_id, KEY_SUFFIX_OPS_DATA)
    }

    fn lock_key(doc_id: &str) -> String {
        format!("{}{}", KEY_PREFIX_LOCK, doc_id)
    }

    fn params_key(storage_id: Uuid) -> String {
        format!("{}{}", KEY_PREFIX_PARAMS, storage_id)
    }

    fn db_version_key(doc_id: Uuid) -> String {
        format!("{}{}{}", KEY_PREFIX_VERSION, doc_id, KEY_SUFFIX_DB_VERSION)
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
    // Version Management
    // ========================================================================

    pub async fn get_version(&self, doc_id: Uuid) -> Result<Option<i64>> {
        let key = Self::version_key(doc_id);

        let version: Option<i64> = self
            .with_retry("get_version", |mut conn| {
                let key = key.clone();
                async move { conn.get(&key).await }
            })
            .await?;

        debug!("Cache get_version: doc_id={}, version={:?}", doc_id, version);
        Ok(version)
    }

    pub async fn set_version(&self, doc_id: Uuid, version: i64) -> Result<()> {
        let key = Self::version_key(doc_id);
        let ttl = self.config.ttl_seconds;

        self.with_retry("set_version", |mut conn| {
            let key = key.clone();
            async move { conn.set_ex::<_, _, ()>(&key, version, ttl).await }
        })
        .await?;

        debug!("Cache set_version: doc_id={}, version={}", doc_id, version);
        Ok(())
    }

    // ========================================================================
    // DB Version Management (Write-Behind Optimization)
    // ========================================================================

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

    pub async fn set_db_version(&self, doc_id: Uuid, version: i64) -> Result<()> {
        let key = Self::db_version_key(doc_id);
        let ttl = 86400u64; // 24 hour TTL

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
    // Idempotency Check
    // ========================================================================

    pub async fn check_idempotency(&self, doc_id: Uuid, op_id: &str) -> Result<Option<i64>> {
        let ops_key = Self::ops_key(doc_id);
        let op_id_owned = op_id.to_string();

        let rev: Option<f64> = self
            .with_retry("check_idempotency", |mut conn| {
                let key = ops_key.clone();
                let op_id = op_id_owned.clone();
                async move { conn.zscore(&key, &op_id).await }
            })
            .await?;

        let rev = rev.map(|r| r as i64);

        if rev.is_some() {
            debug!(
                "Cache idempotency hit: doc_id={}, op_id={}, rev={:?}",
                doc_id, op_id, rev
            );
        }

        Ok(rev)
    }

    // ========================================================================
    // Operations Management
    // ========================================================================

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

        let ops_count = ops.len();
        let mut args: Vec<Vec<u8>> = vec![
            new_version.to_string().into_bytes(),
            ttl_seconds.to_string().into_bytes(),
            ops_count.to_string().into_bytes(),
        ];

        for op in ops {
            let serialized = serialize_smart(op).context("Failed to serialize operation")?;
            args.push(op.op_id.clone().into_bytes());
            args.push(op.rev.to_string().into_bytes());
            args.push(serialized);
        }

        let script = redis::Script::new(
            r#"
            local version_key = KEYS[1]
            local ops_key = KEYS[2]
            local ops_data_key = KEYS[3]
            local new_version = tonumber(ARGV[1])
            local ttl = tonumber(ARGV[2])
            local ops_count = tonumber(ARGV[3])

            redis.call('SET', version_key, new_version, 'EX', ttl)

            for i = 1, ops_count do
                local base = 3 + (i-1)*3
                local op_id = ARGV[base + 1]
                local rev = tonumber(ARGV[base + 2])
                local data = ARGV[base + 3]

                redis.call('ZADD', ops_key, rev, op_id)
                redis.call('HSET', ops_data_key, op_id, data)
            end

            redis.call('EXPIRE', ops_key, ttl)
            redis.call('EXPIRE', ops_data_key, ttl)

            return ops_count
            "#,
        );

        let result: i64 = self
            .with_retry("write_ops", |mut conn| {
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
            })
            .await?;

        debug!(
            "Cache write_ops: doc_id={}, ops_count={}, new_version={}",
            doc_id, result, new_version
        );
        Ok(())
    }

    pub async fn get_ops_since(&self, doc_id: Uuid, since_rev: i64) -> Result<Vec<CachedOperationInfo>> {
        let ops_key = Self::ops_key(doc_id);
        let ops_data_key = Self::ops_data_key(doc_id);
        let min_score = since_rev + 1;

        let script = redis::Script::new(
            r#"
            local ops_key = KEYS[1]
            local data_key = KEYS[2]
            local min_score = ARGV[1]

            local op_ids = redis.call('ZRANGEBYSCORE', ops_key, min_score, '+inf')

            if #op_ids == 0 then
                return {}
            end

            local data = redis.call('HMGET', data_key, unpack(op_ids))

            local result = {}
            for i, d in ipairs(data) do
                if d then
                    table.insert(result, d)
                end
            end
            return result
            "#,
        );

        let data: Vec<Vec<u8>> = self
            .with_retry("get_ops_since", |mut conn| {
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
            })
            .await?;

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

    pub async fn get_ops_range(
        &self,
        doc_id: Uuid,
        from_rev: i64,
        to_rev: i64,
    ) -> Result<Vec<OperationEntry>> {
        let ops_key = Self::ops_key(doc_id);
        let ops_data_key = Self::ops_data_key(doc_id);

        let script = redis::Script::new(
            r#"
            local ops_key = KEYS[1]
            local data_key = KEYS[2]
            local min_score = ARGV[1]
            local max_score = ARGV[2]

            local op_ids = redis.call('ZRANGEBYSCORE', ops_key, min_score, max_score)

            if #op_ids == 0 then
                return {}
            end

            local data = redis.call('HMGET', data_key, unpack(op_ids))

            local result = {}
            for i, d in ipairs(data) do
                if d then
                    table.insert(result, d)
                end
            end
            return result
            "#,
        );

        let data: Vec<Vec<u8>> = self
            .with_retry("get_ops_range", |mut conn| {
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
            })
            .await?;

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

    pub async fn remove_ops_up_to(&self, doc_id: Uuid, max_rev: i64) -> Result<usize> {
        let ops_key = Self::ops_key(doc_id);
        let ops_data_key = Self::ops_data_key(doc_id);

        let script = redis::Script::new(
            r#"
            local ops_key = KEYS[1]
            local data_key = KEYS[2]
            local max_rev = ARGV[1]

            local op_ids = redis.call('ZRANGEBYSCORE', ops_key, '-inf', max_rev)

            if #op_ids > 0 then
                redis.call('HDEL', data_key, unpack(op_ids))
                redis.call('ZREMRANGEBYSCORE', ops_key, '-inf', max_rev)
            end

            return #op_ids
            "#,
        );

        let removed: i64 = self
            .with_retry("remove_ops_up_to", |mut conn| {
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
            })
            .await?;

        debug!(
            "Cache remove_ops_up_to: doc_id={}, max_rev={}, removed={}",
            doc_id, max_rev, removed
        );
        Ok(removed as usize)
    }

    pub async fn get_min_cached_rev(&self, doc_id: Uuid) -> Result<Option<i64>> {
        let ops_key = Self::ops_key(doc_id);

        let result: Vec<(String, f64)> = self
            .with_retry("get_min_cached_rev", |mut conn| {
                let ops_key = ops_key.clone();
                async move { conn.zrange_withscores(&ops_key, 0, 0).await }
            })
            .await?;

        let min_rev = result.first().map(|(_, score)| *score as i64);
        debug!(
            "Cache get_min_cached_rev: doc_id={}, min_rev={:?}",
            doc_id, min_rev
        );
        Ok(min_rev)
    }

    pub async fn get_max_cached_rev(&self, doc_id: Uuid) -> Result<Option<i64>> {
        let ops_key = Self::ops_key(doc_id);

        let result: Vec<(String, f64)> = self
            .with_retry("get_max_cached_rev", |mut conn| {
                let ops_key = ops_key.clone();
                async move { conn.zrevrange_withscores(&ops_key, 0, 0).await }
            })
            .await?;

        let max_rev = result.first().map(|(_, score)| *score as i64);
        debug!(
            "Cache get_max_cached_rev: doc_id={}, max_rev={:?}",
            doc_id, max_rev
        );
        Ok(max_rev)
    }

    pub async fn get_pending_ops_count(&self, doc_id: Uuid) -> Result<usize> {
        let mut conn = self.get_connection();
        let ops_key = Self::ops_key(doc_id);
        let count: usize = conn
            .zcard(&ops_key)
            .await
            .context("Failed to get pending ops count")?;
        Ok(count)
    }

    // ========================================================================
    // Lock Management
    // ========================================================================

    pub async fn acquire_flush_lock(&self, doc_id: &str, ttl_ms: u64) -> Result<bool> {
        let lock_key = Self::lock_key(doc_id);

        let result: Option<String> = self
            .with_retry("acquire_flush_lock", |mut conn| {
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
            })
            .await?;

        let acquired = result.is_some();
        debug!(
            "Cache acquire_flush_lock: doc_id={}, acquired={}",
            doc_id, acquired
        );
        Ok(acquired)
    }

    pub async fn release_flush_lock(&self, doc_id: &str) -> Result<()> {
        let lock_key = Self::lock_key(doc_id);

        self.with_retry("release_flush_lock", |mut conn| {
            let lock_key = lock_key.clone();
            async move { conn.del::<_, ()>(&lock_key).await }
        })
        .await?;

        debug!("Cache release_flush_lock: doc_id={}", doc_id);
        Ok(())
    }

    // ========================================================================
    // Params Caching
    // ========================================================================

    pub async fn get_params_cached(&self, storage_id: Uuid) -> Result<Option<Vec<u8>>> {
        let key = Self::params_key(storage_id);

        let params: Option<Vec<u8>> = self
            .with_retry("get_params_cached", |mut conn| {
                let key = key.clone();
                async move { conn.get(&key).await }
            })
            .await?;

        if params.is_some() {
            debug!("Cache hit for params: storage_id={}", storage_id);
        }
        Ok(params)
    }

    pub async fn cache_params(&self, storage_id: Uuid, params: &[u8]) -> Result<()> {
        let key = Self::params_key(storage_id);
        let params_vec = params.to_vec();

        self.with_retry("cache_params", |mut conn| {
            let key = key.clone();
            let params = params_vec.clone();
            async move {
                conn.set_ex::<_, _, ()>(&key, &params, PARAMS_CACHE_TTL_SECONDS)
                    .await
            }
        })
        .await?;

        debug!(
            "Cache cached params: storage_id={}, size={}",
            storage_id,
            params.len()
        );
        Ok(())
    }

    pub async fn get_params_cached_batch(
        &self,
        storage_ids: &[Uuid],
    ) -> Result<std::collections::HashMap<Uuid, Vec<u8>>> {
        if storage_ids.is_empty() {
            return Ok(std::collections::HashMap::new());
        }

        let mut conn = self.get_connection();
        let keys: Vec<String> = storage_ids.iter().map(|id| Self::params_key(*id)).collect();

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

    pub async fn health_check(&self) -> Result<bool> {
        let mut conn = self.get_connection();

        let pong: String = redis::cmd("PING")
            .query_async(&mut conn)
            .await
            .context("Redis health check failed")?;

        Ok(pong == "PONG")
    }

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
        })
        .await?;

        debug!("Cache clear_doc_cache: doc_id={}", doc_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operation_entry_serialization() {
        let entry = OperationEntry {
            rev: 42,
            user_id: "user-123".to_string(),
            mutation_id: "sheet.mutation.set-range-values".to_string(),
            params: vec![1, 2, 3, 4],
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
