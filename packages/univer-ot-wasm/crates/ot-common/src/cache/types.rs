//! Cache types and configuration

use serde::{Deserialize, Serialize};

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
