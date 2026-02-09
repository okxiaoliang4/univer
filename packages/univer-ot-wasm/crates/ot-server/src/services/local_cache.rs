use moka::future::Cache;
use ot_common::CachedOperationInfo;
use std::sync::Arc;
use std::time::Duration;
use tracing::debug;
use uuid::Uuid;

/// Maximum number of ops to retain per document in the local cache.
/// This bounds memory usage while covering the typical concurrent editing window.
const MAX_OPS_PER_DOC: usize = 500;

/// Process-local L1 cache for recent operations.
///
/// This cache solves a critical issue: the write-behind worker (a separate process)
/// calls `cache_service.remove_ops_up_to(doc_id, max_rev)` after flushing ops to DB,
/// which clears the Redis ops cache. This causes `get_ops_since()` to always miss
/// in Redis (cache_hits=0), falling back to expensive DB queries (70-120ms).
///
/// Since this cache lives in the ot-server process memory, it is unaffected by
/// the write-behind worker's Redis cleanup. The cache hierarchy becomes:
///   L1: LocalCache (moka, ~0ms) → L2: Redis CacheService → L3: DB
///
/// **Multi-instance safety**: This cache only stores ops, NOT version. Version is
/// always read from Redis/DB (cross-instance authoritative source). The caller
/// validates L1 ops coverage against the authoritative version by checking both
/// start gap (first_rev <= since_rev+1) and end gap (last_rev >= current_version).
/// If L1 has incomplete coverage, it falls through to L2/L3.
#[derive(Clone)]
pub struct LocalCache {
    /// doc_id -> recent ops list (ordered by rev, wrapped in Arc for cheap cloning)
    ops_cache: Cache<Uuid, Arc<Vec<CachedOperationInfo>>>,
}

impl LocalCache {
    /// Create a new LocalCache.
    ///
    /// - `max_docs`: maximum number of documents to cache (LRU eviction beyond this)
    /// - `ttl_secs`: time-to-live for cache entries in seconds
    pub fn new(max_docs: u64, ttl_secs: u64) -> Self {
        let ttl = Duration::from_secs(ttl_secs);
        Self {
            ops_cache: Cache::builder()
                .max_capacity(max_docs)
                .time_to_live(ttl)
                .build(),
        }
    }

    /// Get cached operations with rev > since_rev.
    /// Returns None if no matching ops are found (caller should fall through to L2/L3).
    ///
    /// **Important**: The caller MUST validate coverage completeness against the
    /// authoritative `current_version` from Redis/DB. This method only filters by
    /// start rev — it does not know whether the ops cover up to current_version.
    pub async fn get_ops_since(
        &self,
        doc_id: Uuid,
        since_rev: i64,
    ) -> Option<Vec<CachedOperationInfo>> {
        let ops = self.ops_cache.get(&doc_id).await?;
        let filtered: Vec<_> = ops.iter().filter(|op| op.rev > since_rev).cloned().collect();
        if filtered.is_empty() {
            None
        } else {
            Some(filtered)
        }
    }

    /// Write-through: store new ops after a successful changeset.
    ///
    /// Merges with any existing cached ops for this document, keeping only the most
    /// recent MAX_OPS_PER_DOC entries (by rev) to bound memory usage.
    pub async fn put_ops(
        &self,
        doc_id: Uuid,
        new_ops: &[CachedOperationInfo],
    ) {
        let mut all_ops = match self.ops_cache.get(&doc_id).await {
            Some(existing) => (*existing).clone(),
            None => Vec::new(),
        };

        all_ops.extend(new_ops.iter().cloned());

        // Keep only the most recent ops (by rev) to bound memory
        if all_ops.len() > MAX_OPS_PER_DOC {
            all_ops.sort_by_key(|op| op.rev);
            all_ops.drain(0..all_ops.len() - MAX_OPS_PER_DOC);
        }

        debug!(
            "LocalCache put_ops: doc_id={}, new_ops={}, total_cached={}",
            doc_id,
            new_ops.len(),
            all_ops.len(),
        );

        self.ops_cache.insert(doc_id, Arc::new(all_ops)).await;
    }
}
