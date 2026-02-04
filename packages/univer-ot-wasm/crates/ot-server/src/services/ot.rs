use crate::database::entities::{documents, operation_log};
use crate::metrics;
use crate::services::cache::{CacheService, OperationEntry};
use crate::services::document::DocumentService;
use crate::services::op_queue::OpQueueService;
use crate::services::params_codec;
use anyhow::{Context, Result};
use ot_core::{MutationInfo, MutationInfoWithOpId, TransformService};
use rand::Rng;
use rslock::LockManager;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QuerySelect};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Lock configuration - optimized for reduced contention
const LOCK_TTL_MS: u64 = 10_000; // 10 seconds (reduced from 30s)
const LOCK_INITIAL_RETRY_MS: u64 = 2; // 2ms initial retry (reduced from 5ms)
const LOCK_MAX_RETRY_INTERVAL_MS: u64 = 100; // 100ms max interval (reduced from 200ms)
const LOCK_MAX_WAIT_MS: u64 = 5_000; // 5 seconds max wait (reduced from 10s)

/// Maximum allowed version drift before rejecting changeset
/// If client is more than this many versions behind, they should refresh their document state
/// This prevents O(n) queries where n can grow unboundedly
const MAX_VERSION_DRIFT: i64 = 200;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Changeset {
    #[serde(rename = "baseRev")]
    pub base_rev: i64,
    #[serde(rename = "userId")]
    pub user_id: String,
    pub mutations: Vec<MutationInfoWithOpId>,
    #[serde(rename = "clientId")]
    pub client_id: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChangesetApplied {
    #[serde(rename = "serverRev")]
    pub server_rev: i64,
    pub mutations: Vec<MutationInfo>,
    pub op_ids: Vec<String>,
    #[serde(rename = "userId")]
    pub user_id: String,
}

use crate::services::storage::StorageService;

#[derive(Clone)]
pub struct OTService {
    db: Arc<DatabaseConnection>,
    document_service: Arc<DocumentService>,
    transform_service: Arc<TransformService>,
    op_queue_service: Arc<OpQueueService>,
    lock_manager: Arc<LockManager>,
    cache_service: Arc<CacheService>,
    storage_service: Arc<StorageService>,
}

impl OTService {
    pub fn new(
        db: Arc<DatabaseConnection>,
        document_service: Arc<DocumentService>,
        op_queue_service: Arc<OpQueueService>,
        redis_url: &str,
        cache_service: Arc<CacheService>,
        storage_service: Arc<StorageService>,
    ) -> Self {
        Self {
            db,
            document_service,
            transform_service: Arc::new(TransformService::new()),
            op_queue_service,
            lock_manager: Arc::new(LockManager::new(vec![redis_url.to_string()])),
            cache_service,
            storage_service,
        }
    }

    /// Apply a changeset from a client
    /// This performs OT transformation and stores the transformed operations
    ///
    /// Optimized flow:
    /// 1. Phase 1 (no lock): Idempotency check - return early if already processed
    /// 2. Phase 2 (with lock): Acquire lock, process changeset, release lock
    pub async fn apply_changeset(
        &self,
        doc_id: Uuid,
        changeset: Changeset,
    ) -> Result<ChangesetApplied> {
        // Start timing for end-to-end latency
        let e2e_start = Instant::now();

        // Increment total operations counter
        metrics::increment_operations_total();

        // ========== Phase 1: Lock-free idempotency check ==========
        // Check if this changeset was already processed (duplicate request)
        // This avoids acquiring the lock for duplicate requests
        //
        // Performance optimization: Only check Redis cache (~1ms), no DB fallback
        // Redis TTL is 1 hour - sufficient for all reasonable retry scenarios
        // If cache misses, OT transform handles correctness via version checking
        if let Some(first_mutation) = changeset.mutations.first() {
            // Check Redis idempotency cache (no DB fallback for performance)
            // Redis TTL is 1 hour - sufficient for all reasonable retry scenarios
            // If Redis misses, treat as new operation - OT transform handles correctness
            let cached_rev = self.cache_service
                .check_idempotency(doc_id, &changeset.client_id, &first_mutation.op_id)
                .await
                .ok()
                .flatten();

            if let Some(rev) = cached_rev {
                debug!(
                    "Idempotency cache hit: doc_id={}, client_id={}, op_id={}, rev={}",
                    doc_id, changeset.client_id, first_mutation.op_id, rev
                );
                return self.build_duplicate_response(doc_id, rev, &changeset).await;
            }
            // Redis miss = treat as new operation, proceed with processing
        }

        // ========== Phase 2: Acquire lock and process ==========
        let lock_key = format!("ot:lock:doc:{}", doc_id);
        let lock = self.acquire_lock_with_backoff(&lock_key).await?;

        // Execute the changeset logic, ensuring lock is released on all paths
        let result = self
            .apply_changeset_inner(doc_id, changeset, e2e_start)
            .await;

        // Always release the lock
        self.lock_manager.unlock(&lock).await;

        result
    }

    /// Acquire distributed lock with exponential backoff and jitter
    async fn acquire_lock_with_backoff(&self, lock_key: &str) -> Result<rslock::Lock> {
        let lock_ttl = Duration::from_millis(LOCK_TTL_MS);
        let max_wait = Duration::from_millis(LOCK_MAX_WAIT_MS);
        let mut current_interval = Duration::from_millis(LOCK_INITIAL_RETRY_MS);
        let max_interval = Duration::from_millis(LOCK_MAX_RETRY_INTERVAL_MS);
        let lock_start = Instant::now();
        let mut attempts = 0u32;

        loop {
            attempts += 1;
            match self.lock_manager.lock(lock_key.as_bytes(), lock_ttl).await {
                Ok(lock) => {
                    // Record lock acquisition metrics
                    if attempts > 1 {
                        metrics::record_lock_retries(attempts as f64);
                    }
                    metrics::record_lock_wait_time(lock_start.elapsed().as_secs_f64());
                    return Ok(lock);
                }
                Err(_) => {
                    let elapsed = lock_start.elapsed();
                    if elapsed >= max_wait {
                        metrics::increment_lock_timeouts();
                        warn!(
                            "Lock acquisition timeout: key={}, attempts={}, elapsed={:?}",
                            lock_key, attempts, elapsed
                        );
                        return Err(anyhow::anyhow!("Failed to acquire document lock"));
                    }
                    // Exponential backoff with jitter (±50%)
                    let jitter: f64 = { rand::thread_rng().gen_range(0.5..1.5) };
                    let sleep_ms = (current_interval.as_millis() as f64 * jitter) as u64;
                    tokio::time::sleep(Duration::from_millis(sleep_ms)).await;
                    current_interval = (current_interval * 2).min(max_interval);
                }
            }
        }
    }

    /// Build response for duplicate/already-processed changeset
    async fn build_duplicate_response(
        &self,
        doc_id: Uuid,
        existing_rev: i64,
        changeset: &Changeset,
    ) -> Result<ChangesetApplied> {
        let changeset_size = changeset.mutations.len() as i64;
        let existing_mutations = self
            .document_service
            .get_operations(
                doc_id,
                existing_rev,
                Some(existing_rev + changeset_size - 1),
                None,
            )
            .await?;

        let ack_op_ids: Vec<String> = changeset
            .mutations
            .iter()
            .map(|mutation| mutation.op_id.clone())
            .collect();

        Ok(ChangesetApplied {
            server_rev: existing_rev + changeset_size - 1,
            mutations: existing_mutations
                .into_iter()
                .map(|op| MutationInfo {
                    id: op.mutation_id,
                    params: op.params,
                })
                .collect(),
            op_ids: ack_op_ids,
            user_id: changeset.user_id.clone(),
        })
    }

    /// Reconcile version when cache misses (version key expired)
    ///
    /// This handles the race condition where:
    /// 1. join_doc returns version from cache (e.g., 616)
    /// 2. Version key expires due to TTL
    /// 3. changeset reads from DB which is behind (e.g., 612)
    ///
    /// Solution: Check pending ops in cache (ops sorted set may still exist)
    /// Real version = max(db_version, max_cached_rev)
    ///
    /// `try_cache_ops`: if true, attempt to read pending ops from cache
    /// Set to false when Redis is known to be unavailable
    async fn reconcile_version_on_cache_miss(&self, doc_id: Uuid, try_cache_ops: bool) -> Result<i64> {
        // Step 1: Get version from database
        let db_version = documents::Entity::find_by_id(doc_id)
            .select_only()
            .column(documents::Column::CurrentVersion)
            .into_tuple::<i64>()
            .one(&*self.db)
            .await?
            .context("Document not found")?;

        // Step 2: Skip cache check if Redis is known to be unavailable
        if !try_cache_ops {
            return Ok(db_version);
        }

        // Step 3: Check if there are pending ops in cache (ops sorted set might outlive version key)
        let max_cached_rev = match self.cache_service.get_max_cached_rev(doc_id).await {
            Ok(Some(rev)) => rev,
            Ok(None) => {
                // No pending ops - DB version is authoritative
                debug!(
                    "No pending ops in cache: doc_id={}, using db_version={}",
                    doc_id, db_version
                );
                // Initialize cache with DB version (ignore error)
                let _ = self.cache_service.set_version(doc_id, db_version).await;
                return Ok(db_version);
            }
            Err(e) => {
                // Redis error checking ops - use DB version as fallback
                // Use debug level to reduce log noise during Redis issues
                debug!(
                    "Failed to check pending ops (using db_version): doc_id={}, error={}",
                    doc_id, e
                );
                return Ok(db_version);
            }
        };

        // Step 4: Real version is the maximum of DB version and cached ops
        let real_version = db_version.max(max_cached_rev);

        if real_version > db_version {
            info!(
                "Version reconciliation: doc_id={}, db_version={}, max_cached_rev={}, real_version={}",
                doc_id, db_version, max_cached_rev, real_version
            );
            metrics::increment_version_reconciliations();
        }

        // Step 5: Update cache with reconciled version (ignore error)
        if let Err(e) = self.cache_service.set_version(doc_id, real_version).await {
            debug!(
                "Failed to update cache with reconciled version: doc_id={}, error={}",
                doc_id, e
            );
        }

        Ok(real_version)
    }

    /// Inner implementation of apply_changeset (called with lock held)
    /// Note: Idempotency check is done in apply_changeset() before acquiring lock
    ///
    /// Uses write-behind caching: writes to Redis cache first, then asynchronously flushed to DB
    async fn apply_changeset_inner(
        &self,
        doc_id: Uuid,
        changeset: Changeset,
        e2e_start: Instant,
    ) -> Result<ChangesetApplied> {
        // Step 1: Get current version - prefer cache, fall back to DB on cache miss OR error
        // Graceful degradation: Redis errors fall back to DB read to maintain availability
        //
        // IMPORTANT: When cache miss occurs (version key expired), we must check for pending
        // operations in the ops sorted set. The real version = max(db_version, max_cached_rev)
        // This prevents race conditions where join_doc returns a cached version but changeset
        // reads a stale DB version after the version key expires.
        let current_version = match self.cache_service.get_version(doc_id).await {
            Ok(Some(v)) => {
                metrics::increment_writebehind_cache_hits();
                debug!("Cache hit for version: doc_id={}, version={}", doc_id, v);
                v
            }
            Ok(None) => {
                // Cache miss (key expired) - need to reconcile with pending cached ops
                metrics::increment_writebehind_cache_misses();
                let current_version = self.reconcile_version_on_cache_miss(doc_id, true).await?;
                debug!(
                    "Cache miss reconciled: doc_id={}, version={}",
                    doc_id, current_version
                );
                current_version
            }
            Err(e) => {
                // Redis error (timeout/connection issue) - skip reconciliation
                // If get_version failed, get_max_cached_rev will also fail
                debug!(
                    "Redis error getting version (falling back to DB): doc_id={}, error={}",
                    doc_id, e
                );
                metrics::increment_writebehind_cache_misses();

                // Skip reconciliation since Redis is unavailable
                let db_version = documents::Entity::find_by_id(doc_id)
                    .select_only()
                    .column(documents::Column::CurrentVersion)
                    .into_tuple::<i64>()
                    .one(&*self.db)
                    .await?
                    .context("Document not found")?;
                db_version
            }
        };

        // Validate base_rev
        // If base_rev > current_version, try to reconcile from ops cache
        // This handles race conditions where version key read returns stale data
        let current_version = if changeset.base_rev > current_version {
            // Try to get max_cached_rev from ops sorted set as a fallback
            // The ops set may have a newer version than the version key
            match self.cache_service.get_max_cached_rev(doc_id).await {
                Ok(Some(max_rev)) if max_rev >= changeset.base_rev => {
                    // Ops exist with higher rev, use that as current version
                    warn!(
                        "Version key stale: doc_id={}, version_key={}, max_cached_rev={}, using max_cached_rev",
                        doc_id, current_version, max_rev
                    );
                    metrics::increment_version_reconciliations();
                    max_rev
                }
                _ => {
                    // No ops found or max_rev still < base_rev, this is a real error
                    metrics::increment_operations_failed();
                    metrics::record_operation_e2e_latency(e2e_start.elapsed().as_secs_f64());
                    return Err(anyhow::anyhow!(
                        "Version mismatch: client base_rev {} is greater than server version {}",
                        changeset.base_rev,
                        current_version
                    ));
                }
            }
        } else {
            current_version
        };

        // Record version drift
        let version_drift = current_version - changeset.base_rev;
        metrics::record_version_drift(version_drift as f64);

        // Check for excessive version drift - reject if client is too far behind
        // This prevents O(n) queries where n can grow unboundedly
        if version_drift > MAX_VERSION_DRIFT {
            metrics::increment_operations_failed();
            metrics::increment_version_drift_rejections();
            metrics::record_operation_e2e_latency(e2e_start.elapsed().as_secs_f64());
            warn!(
                "Version drift too large: doc_id={}, base_rev={}, current_version={}, drift={}",
                doc_id, changeset.base_rev, current_version, version_drift
            );
            // Include current_version in error so client can recover without rejoining
            return Err(anyhow::anyhow!(
                "VERSION_DRIFT_TOO_LARGE:{}:{}: client is {} versions behind (max: {})",
                current_version, MAX_VERSION_DRIFT, version_drift, MAX_VERSION_DRIFT
            ));
        }

        // Step 2: Get concurrent operations for OT transformation
        // Fast path: skip query entirely when no version drift (most common case)
        let (concurrent_mutations, has_conflict) = if version_drift == 0 {
            // No concurrent operations - client is up to date
            debug!(
                "No version drift, skipping concurrent ops query: doc_id={}, base_rev={}",
                doc_id, changeset.base_rev
            );
            (Vec::new(), false)
        } else {
            // Slow path: need to fetch concurrent ops for OT transform
            let concurrent_ops = self
                .get_concurrent_ops(doc_id, changeset.base_rev)
                .await?;

            let mutations: Vec<MutationInfo> = concurrent_ops
                .into_iter()
                .map(|op| MutationInfo {
                    id: op.mutation_id,
                    params: op.params,
                })
                .collect();

            let has_conflict = !mutations.is_empty();
            if has_conflict {
                metrics::increment_conflicts_total();
            }
            (mutations, has_conflict)
        };

        // Extract metadata before consuming
        let op_ids: Vec<String> = changeset
            .mutations
            .iter()
            .map(|m| m.op_id.clone())
            .collect();
        let user_id = changeset.user_id;
        let client_id = changeset.client_id;

        // Convert to internal format
        let m1_internal: Vec<MutationInfo> = changeset
            .mutations
            .into_iter()
            .map(|mutation| MutationInfo {
                id: mutation.id,
                params: mutation.params,
            })
            .collect();

        // Step 3: Transform operations
        let transform_start = Instant::now();
        let (m1_primes, _, error) = self
            .transform_service
            .transform_list(&m1_internal, &concurrent_mutations);

        metrics::record_transform_latency(transform_start.elapsed().as_secs_f64());

        if !concurrent_mutations.is_empty() {
            metrics::record_transform_complexity(concurrent_mutations.len() as f64);
        }

        if let Some(err) = error {
            metrics::increment_operations_failed();
            metrics::record_operation_e2e_latency(e2e_start.elapsed().as_secs_f64());
            return Err(anyhow::anyhow!("Transform error: {}", err));
        }

        if has_conflict {
            metrics::increment_conflicts_resolved();
        }

        // Step 4: Build cache entries
        let mutations_count = m1_primes.len();
        let mut cache_entries = Vec::with_capacity(mutations_count);
        let mut transformed_mutations = Vec::with_capacity(mutations_count);
        let mut applied_op_ids = Vec::with_capacity(mutations_count);
        let now = chrono::Utc::now();

        for (index, m1_prime) in m1_primes.into_iter().enumerate() {
            let op_id = op_ids.get(index).cloned().unwrap_or_default();
            let rev = current_version + 1 + index as i64;
            let params_bytes = params_codec::encode_params(&m1_prime.params)
                .context("Failed to encode mutation params")?;

            // Store params directly in cache entry - S3 upload happens in write-behind worker
            cache_entries.push(OperationEntry {
                rev,
                user_id: user_id.clone(),
                mutation_id: m1_prime.id.clone(),
                params: params_bytes, // Stored directly, uploaded to S3 asynchronously
                client_id: client_id.clone(),
                op_id: op_id.clone(),
                created_at: now.timestamp_millis(),
            });

            transformed_mutations.push(m1_prime);
            applied_op_ids.push(op_id);
        }

        let new_version = current_version + mutations_count as i64;

        // Step 5: Write to Redis cache atomically
        self.cache_service
            .write_ops(doc_id, &cache_entries, new_version)
            .await
            .map_err(|e| {
                warn!("Redis write failed: doc_id={}, error={}", doc_id, e);
                metrics::increment_operations_failed();
                metrics::record_operation_e2e_latency(e2e_start.elapsed().as_secs_f64());
                anyhow::anyhow!("Redis cache write failed: {}", e)
            })?;

        // Step 6: Enqueue for background flush
        if let Err(e) = self.cache_service.enqueue_doc(&doc_id.to_string()).await {
            warn!("Failed to enqueue doc for flush: {}", e);
            // Not critical - operations are in cache and will be picked up eventually
        }

        // Also enqueue for snapshot processing
        let _ = self
            .op_queue_service
            .enqueue_doc(&doc_id.to_string())
            .await;

        // Record metrics
        metrics::increment_operations_success();
        metrics::increment_writebehind_ops_buffered_by(mutations_count as u64);
        metrics::record_operation_e2e_latency(e2e_start.elapsed().as_secs_f64());

        debug!(
            "Write-behind complete: doc_id={}, new_version={}, ops_buffered={}",
            doc_id, new_version, mutations_count
        );

        Ok(ChangesetApplied {
            server_rev: new_version,
            mutations: transformed_mutations,
            op_ids: applied_op_ids,
            user_id,
        })
    }

    /// Get concurrent operations for OT transformation
    /// Tries cache first, supplements from DB if there's a gap
    /// Cached ops have params directly, DB ops use storage_id
    async fn get_concurrent_ops(
        &self,
        doc_id: Uuid,
        since_rev: i64,
    ) -> Result<Vec<crate::services::document::OperationInfo>> {
        // Try to get from cache first
        match self.cache_service.get_ops_since(doc_id, since_rev).await {
            Ok(cached_ops) if !cached_ops.is_empty() => {
                // Check if we need to supplement from DB
                let first_cached_rev = cached_ops.first().map(|o| o.rev).unwrap_or(i64::MAX);

                if first_cached_rev > since_rev + 1 {
                    // Gap between requested and cached - need DB ops
                    let db_ops = self
                        .document_service
                        .get_operations(doc_id, since_rev + 1, Some(first_cached_rev - 1), None)
                        .await?;

                    // Convert cached ops and combine with DB ops
                    let mut result = db_ops;
                    let cached_converted = self.convert_cached_ops_to_operation_info(cached_ops)?;
                    result.extend(cached_converted);
                    return Ok(result);
                }

                // Cache has all the ops we need
                metrics::increment_writebehind_cache_hits();
                return self.convert_cached_ops_to_operation_info(cached_ops);
            }
            Ok(_) => {
                // Empty cache - fall through to DB
                metrics::increment_writebehind_cache_misses();
            }
            Err(e) => {
                warn!("Cache error getting ops, falling back to DB: {}", e);
                metrics::increment_writebehind_cache_misses();
            }
        }

        // Fall back to DB
        self.document_service
            .get_operations_since(doc_id, since_rev)
            .await
    }

    /// Convert cached operations to OperationInfo
    /// Cached ops have params directly, so just decode them
    fn convert_cached_ops_to_operation_info(
        &self,
        ops: Vec<crate::services::cache::CachedOperationInfo>,
    ) -> Result<Vec<crate::services::document::OperationInfo>> {
        let mut result = Vec::with_capacity(ops.len());

        for op in ops {
            // Decode MessagePack params to JSON
            let params = params_codec::decode_params(&op.params)
                .with_context(|| format!("Failed to decode params for rev={}", op.rev))?;

            result.push(crate::services::document::OperationInfo {
                rev: op.rev,
                user_id: op.user_id,
                mutation_id: op.mutation_id,
                params,
                client_id: op.client_id,
                op_id: op.op_id,
                created_at: op.created_at,
            });
        }

        Ok(result)
    }

    /// Transform two mutations using OT algorithms
    fn transform_mutations(
        &self,
        m1: &MutationInfo,
        m2: &MutationInfo,
    ) -> Result<ot_core::TransformResult> {
        // Use the unified transform service (shared via Arc)
        Ok(self.transform_service.transform(m1, m2))
    }
}
