//! Write-behind worker for asynchronous database persistence
//!
//! This module provides background workers that flush cached operations from Redis
//! to PostgreSQL in batches. This enables low-latency writes (operations are ACKed
//! after Redis write) while maintaining durability through eventual DB persistence.
//!
//! ## Architecture
//!
//! ```text
//! OT Service -> Redis Cache -> WriteBehindWorker -> PostgreSQL
//!     |              |                                  |
//!     |              |                                  |
//!     +-- ACK -------+                                  |
//!                    +-------- async flush -------------+
//! ```
//!
//! ## Failure Handling
//!
//! - If Redis write fails: Fall back to synchronous DB write
//! - If worker flush fails: Re-enqueue document with exponential backoff
//! - If service crashes: Recovery on startup scans pending docs

use crate::database::entities::{documents, operation_log, storage};
use crate::metrics;
use crate::services::cache::CacheService;
use crate::services::copy_writer::{copy_operations, CopyOperationData};
use crate::services::storage::StorageService;
use anyhow::{Context, Result};
use futures::future::join_all;
use sea_orm::{
    sea_query::Expr, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder,
    QuerySelect, TransactionTrait,
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Notify;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Prepared flush data for batched COPY
/// Holds all data needed to complete a flush after preparation
struct PreparedFlushData {
    doc_id: String,
    uuid: Uuid,
    max_rev: i64,
    cached_version: i64,
    copy_data: Vec<CopyOperationData>,
    ops_count: usize,
}

/// Configuration for write-behind workers
#[derive(Debug, Clone)]
pub struct WriteBehindConfig {
    pub batch_size: usize,
    pub flush_interval_ms: u64,
    pub worker_count: usize,
    pub lock_ttl_ms: u64,
    pub max_retry_count: u32,
    pub retry_base_delay_ms: u64,
    /// Params smaller than this threshold (in bytes) are stored directly in the database.
    /// Larger params are uploaded to S3. Default: 2048 (2KB) aligns with PostgreSQL TOAST.
    pub params_inline_threshold_bytes: usize,
}

impl Default for WriteBehindConfig {
    fn default() -> Self {
        Self {
            batch_size: 200,       // Increased to accumulate more ops per flush
            flush_interval_ms: 200, // Increased to allow more ops to accumulate
            worker_count: 8,       // Keep 8 workers for parallelism
            lock_ttl_ms: 5_000,    // Increased to allow larger batches
            max_retry_count: 5,
            retry_base_delay_ms: 50,
            params_inline_threshold_bytes: 2048, // 2KB - aligns with PostgreSQL TOAST threshold
        }
    }
}

/// Write-behind worker manager
pub struct WriteBehindWorker {
    db: Arc<DatabaseConnection>,
    cache_service: Arc<CacheService>,
    storage_service: Arc<StorageService>,
    config: WriteBehindConfig,
    shutdown: Arc<AtomicBool>,
    shutdown_notify: Arc<Notify>,
}

impl WriteBehindWorker {
    /// Create a new write-behind worker
    pub fn new(
        db: Arc<DatabaseConnection>,
        cache_service: Arc<CacheService>,
        storage_service: Arc<StorageService>,
        config: WriteBehindConfig,
    ) -> Self {
        Self {
            db,
            cache_service,
            storage_service,
            config,
            shutdown: Arc::new(AtomicBool::new(false)),
            shutdown_notify: Arc::new(Notify::new()),
        }
    }

    /// Start the worker pool
    pub fn start(&self) -> Vec<tokio::task::JoinHandle<()>> {
        let mut handles = Vec::with_capacity(self.config.worker_count);

        for worker_id in 0..self.config.worker_count {
            let db = self.db.clone();
            let cache_service = self.cache_service.clone();
            let storage_service = self.storage_service.clone();
            let config = self.config.clone();
            let shutdown = self.shutdown.clone();
            let shutdown_notify = self.shutdown_notify.clone();

            let handle = tokio::spawn(async move {
                info!("WriteBehindWorker {} started", worker_id);
                Self::worker_loop(
                    worker_id,
                    db,
                    cache_service,
                    storage_service,
                    config,
                    shutdown,
                    shutdown_notify,
                )
                .await;
                info!("WriteBehindWorker {} stopped", worker_id);
            });

            handles.push(handle);
        }

        info!(
            "Started {} write-behind workers",
            self.config.worker_count
        );
        handles
    }

    /// Signal workers to shut down
    pub fn shutdown(&self) {
        info!("Signaling write-behind workers to shutdown");
        self.shutdown.store(true, Ordering::SeqCst);
        self.shutdown_notify.notify_waiters();
    }

    /// Flush all pending documents (for graceful shutdown or recovery)
    pub async fn flush_all(&self) -> Result<usize> {
        info!("Flushing all pending documents");
        let start = Instant::now();

        // Get all pending docs
        let pending_docs = self.cache_service.get_all_pending_docs().await?;
        let total = pending_docs.len();

        if total == 0 {
            info!("No pending documents to flush");
            return Ok(0);
        }

        info!("Found {} pending documents to flush", total);

        let mut flushed = 0;
        for doc_id in pending_docs {
            match self.flush_document_internal(&doc_id).await {
                Ok(count) => {
                    flushed += count;
                    debug!("Flushed {} operations for doc {}", count, doc_id);
                }
                Err(e) => {
                    error!("Failed to flush doc {}: {}", doc_id, e);
                    // Continue with other documents
                }
            }
        }

        let elapsed = start.elapsed();
        info!(
            "Flush complete: flushed {} operations from {} documents in {:?}",
            flushed, total, elapsed
        );

        Ok(flushed)
    }

    /// Worker main loop
    ///
    /// Uses non-blocking dequeue to avoid holding Redis connections.
    /// Workers wait for enqueue notifications or periodic timeout.
    /// Batches multiple documents into a single COPY for efficiency.
    async fn worker_loop(
        worker_id: usize,
        db: Arc<DatabaseConnection>,
        cache_service: Arc<CacheService>,
        storage_service: Arc<StorageService>,
        config: WriteBehindConfig,
        shutdown: Arc<AtomicBool>,
        shutdown_notify: Arc<Notify>,
    ) {
        // Maximum documents to process in one batch before yielding
        const MAX_BATCH_SIZE: usize = 10;

        loop {
            // Check for shutdown
            if shutdown.load(Ordering::SeqCst) {
                debug!("Worker {} received shutdown signal", worker_id);
                break;
            }

            // Wait for enqueue notification or periodic timeout (non-blocking)
            tokio::select! {
                _ = cache_service.wait_for_enqueue() => {
                    // New document enqueued, try to dequeue
                }
                _ = tokio::time::sleep(Duration::from_millis(config.flush_interval_ms)) => {
                    // Periodic check in case we missed a notification
                }
                _ = shutdown_notify.notified() => {
                    debug!("Worker {} interrupted by shutdown", worker_id);
                    break;
                }
            }

            // Collect documents to process in batch
            // Strategy: dequeue -> try lock -> only process if lock acquired
            // This prevents multiple workers from processing the same document
            let mut doc_ids = Vec::with_capacity(MAX_BATCH_SIZE);
            let mut dequeue_attempts = 0;
            const MAX_DEQUEUE_ATTEMPTS: usize = MAX_BATCH_SIZE * 2; // Allow some skips due to lock contention

            while doc_ids.len() < MAX_BATCH_SIZE && dequeue_attempts < MAX_DEQUEUE_ATTEMPTS {
                if shutdown.load(Ordering::SeqCst) {
                    break;
                }

                dequeue_attempts += 1;

                match cache_service.dequeue_doc_nonblocking().await {
                    Ok(Some(id)) => {
                        // Try to acquire lock immediately after dequeue
                        // If lock fails, another worker is processing - skip this doc
                        let lock_acquired = cache_service
                            .acquire_flush_lock(&id, config.lock_ttl_ms)
                            .await
                            .unwrap_or(false);

                        if lock_acquired {
                            doc_ids.push(id);
                        } else {
                            // Another worker has the lock, skip this document
                            // It will be re-enqueued by that worker if needed
                            debug!(
                                "Worker {} skipped doc {} - lock held by another worker",
                                worker_id, id
                            );
                            metrics::increment_writebehind_skipped();
                        }
                    }
                    Ok(None) => break, // Queue empty
                    Err(e) => {
                        static LAST_ERROR_LOG: std::sync::atomic::AtomicU64 =
                            std::sync::atomic::AtomicU64::new(0);
                        let now = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs();
                        let last = LAST_ERROR_LOG.load(std::sync::atomic::Ordering::Relaxed);
                        if now - last >= 10 {
                            LAST_ERROR_LOG.store(now, std::sync::atomic::Ordering::Relaxed);
                            warn!("Worker {} dequeue error: {}", worker_id, e);
                        }
                        tokio::time::sleep(Duration::from_millis(100)).await;
                        break;
                    }
                }
            }

            if doc_ids.is_empty() {
                continue;
            }

            info!("Worker {} processing {} documents in batch (locked): {:?}", worker_id, doc_ids.len(), doc_ids);

            // Prepare all documents (upload to S3, insert storage records)
            let mut prepared_data: Vec<PreparedFlushData> = Vec::with_capacity(doc_ids.len());
            let mut failed_docs: Vec<String> = Vec::new();

            for doc_id in &doc_ids {
                match Self::prepare_flush(&db, &cache_service, &storage_service, &config, doc_id).await {
                    Ok(Some(data)) => prepared_data.push(data),
                    Ok(None) => {
                        // No operations to flush for this doc
                        debug!("Worker {} no ops to flush for doc {}", worker_id, doc_id);
                    }
                    Err(e) => {
                        warn!("Worker {} prepare failed for doc {}: {}", worker_id, doc_id, e);
                        failed_docs.push(doc_id.clone());
                    }
                }
            }

            // Release locks and re-enqueue failed documents
            for doc_id in failed_docs {
                // Release the lock we acquired during dequeue
                let _ = cache_service.release_flush_lock(&doc_id).await;
                if let Err(e) = cache_service.enqueue_doc(&doc_id).await {
                    error!("Failed to re-enqueue doc {}: {}", doc_id, e);
                }
            }

            if prepared_data.is_empty() {
                continue;
            }

            // Batch COPY all prepared operations
            let total_ops: usize = prepared_data.iter().map(|d| d.ops_count).sum();
            let all_copy_data: Vec<CopyOperationData> = prepared_data
                .iter()
                .flat_map(|d| d.copy_data.clone())
                .collect();

            let copy_start = Instant::now();
            match copy_operations(&db, &all_copy_data).await {
                Ok(rows) => {
                    debug!(
                        "Worker {} COPY inserted {} rows for {} docs in {:?}",
                        worker_id, rows, prepared_data.len(), copy_start.elapsed()
                    );

                    // Update versions and cleanup for all successful documents
                    for data in &prepared_data {
                        // Update document version
                        if let Err(e) = documents::Entity::update_many()
                            .col_expr(documents::Column::CurrentVersion, Expr::value(data.max_rev))
                            .col_expr(documents::Column::UpdatedAt, Expr::value(chrono::Utc::now()))
                            .filter(documents::Column::Id.eq(data.uuid))
                            // .filter(documents::Column::CurrentVersion.lt(data.max_rev))
                            .exec(db.as_ref())
                            .await
                        {
                            error!("Failed to update version for doc {}: {}", data.doc_id, e);
                        }

                        // Update db_version cache
                        if let Err(e) = cache_service.set_db_version(data.uuid, data.max_rev).await {
                            warn!("Failed to update db_version cache for doc {}: {}", data.doc_id, e);
                        }

                        // Remove flushed operations from cache
                        if let Err(e) = cache_service.remove_ops_up_to(data.uuid, data.max_rev).await {
                            warn!("Failed to remove flushed ops for doc {}: {}", data.doc_id, e);
                        }

                        // Release flush lock after successful COPY
                        let _ = cache_service.release_flush_lock(&data.doc_id).await;

                        // Re-enqueue if more operations pending
                        if data.max_rev < data.cached_version {
                            let _ = cache_service.enqueue_doc(&data.doc_id).await;
                        }
                    }

                    metrics::increment_writebehind_ops_flushed_by(total_ops as u64);
                }
                Err(e) => {
                    let error_str = format!("{:?}", e);
                    let is_duplicate_key = error_str.contains("duplicate key")
                        || error_str.contains("23505");

                    if is_duplicate_key {
                        // Duplicate key error - db_version cache is stale
                        // Query actual max rev from operation_logs (not documents.current_version)
                        // because operation_logs may have partial data from a previous crash
                        warn!("Worker {} COPY duplicate key error, syncing from operation_logs max rev", worker_id);

                        for data in &prepared_data {
                            // Query actual max rev from operation_logs table
                            let actual_max_rev = operation_log::Entity::find()
                                .filter(operation_log::Column::DocId.eq(data.uuid))
                                .select_only()
                                .column(operation_log::Column::Rev)
                                .order_by_desc(operation_log::Column::Rev)
                                .into_tuple::<i64>()
                                .one(db.as_ref())
                                .await
                                .ok()
                                .flatten()
                                .unwrap_or(0);

                            if actual_max_rev > 0 {
                                // Update cache with actual max rev from operation_logs
                                let _ = cache_service.set_db_version(data.uuid, actual_max_rev).await;
                                // Remove already-flushed ops from Redis cache
                                let _ = cache_service.remove_ops_up_to(data.uuid, actual_max_rev).await;
                                debug!(
                                    "Synced db_version for doc {} from operation_logs: max_rev={}",
                                    data.doc_id, actual_max_rev
                                );

                                // Also update documents.current_version to match operation_logs
                                let _ = documents::Entity::update_many()
                                    .col_expr(documents::Column::CurrentVersion, Expr::value(actual_max_rev))
                                    .filter(documents::Column::Id.eq(data.uuid))
                                    .filter(documents::Column::CurrentVersion.lt(actual_max_rev))
                                    .exec(db.as_ref())
                                    .await;
                            }

                            // Release lock and re-enqueue for retry with corrected version
                            let _ = cache_service.release_flush_lock(&data.doc_id).await;
                            let _ = cache_service.enqueue_doc(&data.doc_id).await;
                        }
                    } else {
                        error!("Worker {} batch COPY failed: {}, re-enqueueing {} docs",
                               worker_id, e, prepared_data.len());
                        // Release locks and re-enqueue all documents for retry
                        for data in &prepared_data {
                            let _ = cache_service.release_flush_lock(&data.doc_id).await;
                            if let Err(e) = cache_service.enqueue_doc(&data.doc_id).await {
                                error!("Failed to re-enqueue doc {}: {}", data.doc_id, e);
                            }
                        }
                    }
                }
            }
        }
    }

    /// Prepare flush data for a document (upload S3, insert storage records)
    /// Returns PreparedFlushData ready for batched COPY, or None if no ops to flush
    ///
    /// IMPORTANT: Caller must have already acquired the flush lock for this document.
    /// The lock is acquired in worker_loop immediately after dequeue to prevent
    /// multiple workers from processing the same document.
    async fn prepare_flush(
        db: &DatabaseConnection,
        cache_service: &CacheService,
        storage_service: &StorageService,
        config: &WriteBehindConfig,
        doc_id: &str,
    ) -> Result<Option<PreparedFlushData>> {
        let uuid = Uuid::parse_str(doc_id).context("Invalid document ID")?;

        // NOTE: Lock is already acquired by worker_loop after dequeue
        // No need to acquire lock here - we already have exclusive access

        // Get current DB version
        let db_version = match cache_service.get_db_version(uuid).await {
            Ok(Some(v)) => {
                metrics::increment_db_version_cache_hits();
                v
            }
            Ok(None) | Err(_) => {
                metrics::increment_db_version_cache_misses();
                let v = documents::Entity::find_by_id(uuid)
                    .select_only()
                    .column(documents::Column::CurrentVersion)
                    .into_tuple::<i64>()
                    .one(db)
                    .await?
                    .context("Document not found")?;
                let _ = cache_service.set_db_version(uuid, v).await;
                v
            }
        };

        let cached_version = cache_service.get_version(uuid).await?.unwrap_or(db_version);

        if cached_version <= db_version {
            info!(
                "Skipping flush for doc {} - cached_version ({}) <= db_version ({})",
                doc_id, cached_version, db_version
            );
            metrics::increment_writebehind_skipped();
            // Release lock since we're not going to flush
            // Lock was acquired in worker_loop after dequeue
            let _ = cache_service.release_flush_lock(doc_id).await;
            return Ok(None);
        }

        // Get operations in batches
        let from_rev = db_version + 1;
        let to_rev = (from_rev + config.batch_size as i64 - 1).min(cached_version);
        let ops = cache_service.get_ops_range(uuid, from_rev, to_rev).await?;

        if ops.is_empty() {
            info!(
                "No cached operations found for doc {} in range [{}, {}], cached_version={}, db_version={}",
                doc_id, from_rev, to_rev, cached_version, db_version
            );
            metrics::increment_writebehind_skipped();
            // Release lock since we're not going to flush
            let _ = cache_service.release_flush_lock(doc_id).await;
            return Ok(None);
        }

        let ops_count = ops.len();
        let threshold = config.params_inline_threshold_bytes;

        // Partition by size
        let (small_ops, large_ops): (Vec<_>, Vec<_>) = ops
            .iter()
            .partition(|op| op.params.len() < threshold);

        // Upload large params to S3
        let mut uploaded_params = Vec::with_capacity(large_ops.len());
        for chunk in large_ops.chunks(10) {
            let upload_futures: Vec<_> = chunk
                .iter()
                .map(|op| storage_service.upload_operation_params_to_s3(uuid, op.rev, &op.params))
                .collect();

            let chunk_results = join_all(upload_futures).await;
            for (i, result) in chunk_results.into_iter().enumerate() {
                match result {
                    Ok(uploaded) => uploaded_params.push(uploaded),
                    Err(e) => {
                        // Release lock on S3 upload failure
                        let _ = cache_service.release_flush_lock(doc_id).await;
                        return Err(e.context(format!("S3 upload failed for rev {}", chunk[i].rev)));
                    }
                }
            }
        }

        // Insert storage records in transaction
        if !uploaded_params.is_empty() {
            let txn = db.begin().await?;
            let storage_records: Vec<storage::ActiveModel> = uploaded_params
                .iter()
                .map(|u| u.storage_record.clone())
                .collect();

            for chunk in storage_records.chunks(25) {
                if !chunk.is_empty() {
                    storage::Entity::insert_many(chunk.to_vec())
                        .exec(&txn)
                        .await
                        .context("Failed to insert storage records")?;
                }
            }
            txn.commit().await?;
        }

        // Build COPY data
        // Note: With opId-as-member Redis structure, duplicates are no longer possible
        // Each opId maps to exactly one entry in the Sorted Set
        let rev_to_storage_id: std::collections::HashMap<i64, Uuid> = uploaded_params
            .iter()
            .map(|u| (u.rev, u.storage_id))
            .collect();

        let mut copy_data = Vec::with_capacity(ops_count);
        let mut max_rev = from_rev - 1;

        for op in &ops {
            max_rev = max_rev.max(op.rev);
            let is_small = op.params.len() < threshold;
            let storage_id = if is_small {
                None
            } else {
                Some(*rev_to_storage_id.get(&op.rev).context("Missing storage_id")?)
            };

            copy_data.push(CopyOperationData {
                doc_id: uuid,
                rev: op.rev,
                user_id: op.user_id.clone(),
                mutation_id: op.mutation_id.clone(),
                storage_id,
                params: if is_small { Some(op.params.clone()) } else { None },
                op_id: op.op_id.clone(),
                created_at: chrono::DateTime::from_timestamp_millis(op.created_at)
                    .unwrap_or_else(chrono::Utc::now),
            });
        }

        // NOTE: Lock is NOT released here - it will be released after COPY succeeds
        // This prevents race conditions where another worker picks up the same doc
        // before COPY completes

        Ok(Some(PreparedFlushData {
            doc_id: doc_id.to_string(),
            uuid,
            max_rev,
            cached_version,
            copy_data,
            ops_count,
        }))
    }

    /// Flush a document with retries
    ///
    /// Lock acquisition strategy:
    /// - Try to acquire lock ONCE - if another worker has it, skip immediately
    /// - Only retry on actual flush failures (DB errors, etc.)
    /// - Document will be re-enqueued after flush if there are more ops
    async fn flush_document_with_retry(
        db: &DatabaseConnection,
        cache_service: &CacheService,
        storage_service: &StorageService,
        config: &WriteBehindConfig,
        doc_id: &str,
    ) -> Result<usize> {
        // Try to acquire flush lock - single attempt only
        // If another worker has the lock, skip this document immediately
        // The document will be re-enqueued after the current flush completes
        let lock_acquired = cache_service
            .acquire_flush_lock(doc_id, config.lock_ttl_ms)
            .await
            .unwrap_or(false);

        if !lock_acquired {
            debug!(
                "Skipping flush for doc {} - another worker is flushing",
                doc_id
            );
            // Return Ok(0) instead of error - this is expected behavior, not a failure
            // The document will be re-enqueued by the worker that has the lock
            metrics::increment_writebehind_skipped();
            return Ok(0);
        }

        // Perform flush with retries for actual failures (DB errors, etc.)
        let mut attempts = 0;
        let mut delay = Duration::from_millis(config.retry_base_delay_ms);

        let result = loop {
            attempts += 1;

            match Self::flush_document_impl(db, cache_service, storage_service, config, doc_id)
                .await
            {
                Ok(count) => break Ok(count),
                Err(e) => {
                    warn!(
                        "Flush failed for doc {}, attempt {}: {}",
                        doc_id, attempts, e
                    );
                    if attempts >= config.max_retry_count {
                        break Err(e);
                    }
                    tokio::time::sleep(delay).await;
                    delay = (delay * 2).min(Duration::from_secs(5));
                }
            }
        };

        // Always release lock
        if let Err(e) = cache_service.release_flush_lock(doc_id).await {
            warn!("Failed to release flush lock for doc {}: {}", doc_id, e);
        }

        result
    }

    /// Internal flush implementation
    ///
    /// Optimized batch flow with inline params support:
    /// 1. Get ops from cache (with params)
    /// 2. Partition ops by size: small (< threshold) vs large (>= threshold)
    /// 3. Upload large ops params to S3 (concurrent)
    /// 4. Batch insert storage records for large ops
    /// 5. Batch insert operation_log records:
    ///    - Small ops: params stored inline, storage_id = NULL
    ///    - Large ops: params = NULL, storage_id references S3
    /// 6. Update document version
    /// All DB operations in a single transaction for atomicity
    async fn flush_document_impl(
        db: &DatabaseConnection,
        cache_service: &CacheService,
        storage_service: &StorageService,
        config: &WriteBehindConfig,
        doc_id: &str,
    ) -> Result<usize> {
        let start = Instant::now();
        let uuid = Uuid::parse_str(doc_id).context("Invalid document ID")?;

        // Get current DB version - prefer Redis cache to avoid DB query on every flush
        // This is the key optimization: Redis lookup (~1ms) vs DB query (~50-100ms under load)
        let db_version = match cache_service.get_db_version(uuid).await {
            Ok(Some(v)) => {
                debug!(
                    "Cache hit for db_version: doc_id={}, version={}",
                    doc_id, v
                );
                metrics::increment_db_version_cache_hits();
                v
            }
            Ok(None) => {
                // Cache miss (cold start or key expired) - query DB and cache the result
                debug!(
                    "Cache miss for db_version: doc_id={}, querying DB",
                    doc_id
                );
                metrics::increment_db_version_cache_misses();

                let query_start = Instant::now();
                let v = documents::Entity::find_by_id(uuid)
                    .select_only()
                    .column(documents::Column::CurrentVersion)
                    .into_tuple::<i64>()
                    .one(db)
                    .await?
                    .context("Document not found")?;
                metrics::record_db_query_latency(query_start.elapsed().as_secs_f64());
                metrics::increment_db_queries();

                // Cache the DB version for future flush cycles
                if let Err(e) = cache_service.set_db_version(uuid, v).await {
                    warn!(
                        "Failed to cache db_version for doc {}: {}",
                        doc_id, e
                    );
                    // Continue anyway - caching is optimization, not critical path
                }
                v
            }
            Err(e) => {
                // Redis error - fall back to DB query
                warn!(
                    "Redis error getting db_version (falling back to DB): doc_id={}, error={}",
                    doc_id, e
                );
                metrics::increment_db_version_cache_misses();

                let query_start = Instant::now();
                let v = documents::Entity::find_by_id(uuid)
                    .select_only()
                    .column(documents::Column::CurrentVersion)
                    .into_tuple::<i64>()
                    .one(db)
                    .await?
                    .context("Document not found")?;
                metrics::record_db_query_latency(query_start.elapsed().as_secs_f64());
                metrics::increment_db_queries();
                v
            }
        };

        // Get operations to flush (from db_version + 1)
        // We use the cached version to determine the upper bound
        let cached_version = cache_service.get_version(uuid).await?.unwrap_or(db_version);

        if cached_version <= db_version {
            debug!(
                "No operations to flush for doc {}: db_version={}, cached_version={}",
                doc_id, db_version, cached_version
            );
            return Ok(0);
        }

        // Get operations in batches
        let from_rev = db_version + 1;
        let to_rev = (from_rev + config.batch_size as i64 - 1).min(cached_version);

        let ops = cache_service.get_ops_range(uuid, from_rev, to_rev).await?;

        if ops.is_empty() {
            debug!(
                "No cached operations found for doc {} in range [{}, {}]",
                doc_id, from_rev, to_rev
            );
            return Ok(0);
        }

        let ops_count = ops.len();
        let threshold = config.params_inline_threshold_bytes;

        // Step 1: Partition operations by params size
        // Small operations (< threshold): store params inline in DB
        // Large operations (>= threshold): upload to S3
        let (small_ops, large_ops): (Vec<_>, Vec<_>) = ops
            .iter()
            .partition(|op| op.params.len() < threshold);

        let small_count = small_ops.len();
        let large_count = large_ops.len();

        debug!(
            "Partitioned {} ops for doc {}: {} small (inline), {} large (S3), threshold={}",
            ops_count, doc_id, small_count, large_count, threshold
        );

        // Step 2: Upload large operation params to S3 (with concurrency limit)
        const S3_UPLOAD_CHUNK_SIZE: usize = 10;
        let s3_start = Instant::now();

        let mut uploaded_params = Vec::with_capacity(large_count);
        for chunk in large_ops.chunks(S3_UPLOAD_CHUNK_SIZE) {
            let upload_futures: Vec<_> = chunk
                .iter()
                .map(|op| storage_service.upload_operation_params_to_s3(uuid, op.rev, &op.params))
                .collect();

            let chunk_results = join_all(upload_futures).await;

            for (i, result) in chunk_results.into_iter().enumerate() {
                match result {
                    Ok(uploaded) => uploaded_params.push(uploaded),
                    Err(e) => {
                        error!(
                            "Failed to upload params to S3 for doc {}, rev {}: {}",
                            doc_id, chunk[i].rev, e
                        );
                        return Err(e.context("S3 upload failed"));
                    }
                }
            }
        }

        if large_count > 0 {
            debug!(
                "S3 uploads completed for doc {}: {} large ops in {:?}",
                doc_id,
                large_count,
                s3_start.elapsed()
            );
        }

        // Step 3: Begin transaction for all DB operations
        let txn_start = Instant::now();
        let txn = db.begin().await?;

        // Step 4: Batch insert storage records for large operations only
        const DB_INSERT_CHUNK_SIZE: usize = 25;

        if !uploaded_params.is_empty() {
            let storage_records: Vec<storage::ActiveModel> = uploaded_params
                .iter()
                .map(|u| u.storage_record.clone())
                .collect();

            for chunk in storage_records.chunks(DB_INSERT_CHUNK_SIZE) {
                if !chunk.is_empty() {
                    storage::Entity::insert_many(chunk.to_vec())
                        .exec(&txn)
                        .await
                        .context("Failed to batch insert storage records")?;
                }
            }
        }

        // Step 5: Build COPY data for operation_logs
        // Build a map of rev -> storage_id for large operations
        // Note: With opId-as-member Redis structure, duplicates are no longer possible
        // Each opId maps to exactly one entry in the Sorted Set
        let rev_to_storage_id: std::collections::HashMap<i64, Uuid> = uploaded_params
            .iter()
            .map(|u| (u.rev, u.storage_id))
            .collect();

        let mut copy_data = Vec::with_capacity(ops_count);
        let mut max_rev = from_rev - 1;

        for op in &ops {
            max_rev = max_rev.max(op.rev);

            let is_small = op.params.len() < threshold;
            let storage_id = if is_small {
                None
            } else {
                Some(
                    *rev_to_storage_id
                        .get(&op.rev)
                        .context(format!("Missing storage_id for rev {}", op.rev))?,
                )
            };

            copy_data.push(CopyOperationData {
                doc_id: uuid,
                rev: op.rev,
                user_id: op.user_id.clone(),
                mutation_id: op.mutation_id.clone(),
                storage_id,
                params: if is_small {
                    Some(op.params.clone())
                } else {
                    None
                },
                op_id: op.op_id.clone(),
                created_at: chrono::DateTime::from_timestamp_millis(op.created_at)
                    .unwrap_or_else(chrono::Utc::now),
            });
        }

        // Commit transaction (storage records only)
        // Version update moved to AFTER COPY succeeds to ensure atomicity
        txn.commit().await?;

        // Record transaction duration
        metrics::record_db_transaction_duration(txn_start.elapsed().as_secs_f64());

        // Step 6: COPY operation_logs (outside transaction for performance)
        // COPY is 5-10x faster than INSERT for batch operations
        // If COPY fails, operations are still in Redis cache and will be retried
        let copy_start = Instant::now();
        match copy_operations(db, &copy_data).await {
            Ok(rows) => {
                debug!(
                    "COPY inserted {} operation_logs for doc {} in {:?}",
                    rows, doc_id, copy_start.elapsed()
                );
            }
            Err(e) => {
                let error_str = format!("{:?}", e);
                let is_duplicate_key = error_str.contains("duplicate key")
                    || error_str.contains("23505");

                if is_duplicate_key {
                    // Duplicate key error - db_version cache is stale
                    // Query actual max rev from operation_logs (not documents.current_version)
                    // because operation_logs may have partial data from a previous crash
                    warn!(
                        "COPY duplicate key for doc {}, syncing from operation_logs max rev",
                        doc_id
                    );

                    // Get actual max rev from operation_logs table
                    let actual_max_rev = operation_log::Entity::find()
                        .filter(operation_log::Column::DocId.eq(uuid))
                        .select_only()
                        .column(operation_log::Column::Rev)
                        .order_by_desc(operation_log::Column::Rev)
                        .into_tuple::<i64>()
                        .one(db)
                        .await
                        .ok()
                        .flatten()
                        .unwrap_or(0);

                    if actual_max_rev > 0 {
                        // Update cache with actual max rev from operation_logs
                        let _ = cache_service.set_db_version(uuid, actual_max_rev).await;
                        // Remove already-flushed ops from Redis cache
                        let _ = cache_service.remove_ops_up_to(uuid, actual_max_rev).await;
                        debug!(
                            "Synced db_version for doc {} from operation_logs: max_rev={}",
                            doc_id, actual_max_rev
                        );

                        // Also update documents.current_version to match operation_logs
                        let _ = documents::Entity::update_many()
                            .col_expr(documents::Column::CurrentVersion, Expr::value(actual_max_rev))
                            .filter(documents::Column::Id.eq(uuid))
                            .filter(documents::Column::CurrentVersion.lt(actual_max_rev))
                            .exec(db)
                            .await;
                    }
                } else {
                    warn!(
                        "COPY failed for doc {}: {}, re-enqueueing for retry",
                        doc_id, e
                    );
                }

                let _ = cache_service.enqueue_doc(doc_id).await;
                return Err(e.context("COPY operation_logs failed"));
            }
        }

        // Step 7: Update document version AFTER COPY succeeds
        // This ensures atomicity: if COPY fails, version isn't updated,
        // and retry will correctly re-fetch and re-insert the same operations
        documents::Entity::update_many()
            .col_expr(documents::Column::CurrentVersion, Expr::value(max_rev))
            .col_expr(
                documents::Column::UpdatedAt,
                Expr::value(chrono::Utc::now()),
            )
            .filter(documents::Column::Id.eq(uuid))
            .filter(documents::Column::CurrentVersion.lt(max_rev))
            .exec(db)
            .await?;

        // Update db_version cache after successful flush
        // This is critical for avoiding DB queries on subsequent flush cycles
        if let Err(e) = cache_service.set_db_version(uuid, max_rev).await {
            warn!(
                "Failed to update db_version cache after flush for doc {}: {}",
                doc_id, e
            );
            // Not critical - next flush will query DB once and re-cache
        }

        // Cache params in Redis for fast subsequent reads (large ops only)
        // Small ops are stored inline in DB, so we don't need Redis caching
        for (op, uploaded) in large_ops.iter().zip(uploaded_params.iter()) {
            if let Err(e) = cache_service.cache_params(uploaded.storage_id, &op.params).await {
                debug!("Failed to cache params after flush: {}", e);
                // Non-critical - params can be fetched from S3
            }
        }

        // Remove flushed operations from cache
        if let Err(e) = cache_service.remove_ops_up_to(uuid, max_rev).await {
            warn!(
                "Failed to remove flushed ops from cache for doc {}: {}",
                doc_id, e
            );
            // Not critical - they'll be cleaned up by TTL
        }

        // Record metrics
        let elapsed = start.elapsed();
        metrics::record_writebehind_flush_latency(elapsed.as_secs_f64());
        metrics::increment_writebehind_ops_flushed_by(ops_count as u64);

        debug!(
            "Flushed {} operations for doc {} (revs {}-{}) in {:?}: {} inline, {} to S3",
            ops_count, doc_id, from_rev, max_rev, elapsed, small_count, large_count
        );

        // If there are more operations to flush, re-enqueue the document
        if max_rev < cached_version {
            debug!(
                "More operations pending for doc {}: flushed up to {}, cached version {}",
                doc_id, max_rev, cached_version
            );
            let _ = cache_service.enqueue_doc(doc_id).await;
        }

        Ok(ops_count)
    }

    /// Flush a single document (public API for manual flush)
    async fn flush_document_internal(&self, doc_id: &str) -> Result<usize> {
        Self::flush_document_with_retry(
            &self.db,
            &self.cache_service,
            &self.storage_service,
            &self.config,
            doc_id,
        )
        .await
    }

    /// Get the current queue depth (for monitoring)
    pub async fn get_queue_depth(&self) -> Result<usize> {
        self.cache_service.get_queue_depth().await
    }
}

impl Drop for WriteBehindWorker {
    fn drop(&mut self) {
        self.shutdown();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = WriteBehindConfig::default();
        assert_eq!(config.batch_size, 200); // Increased to accumulate more ops per flush
        assert_eq!(config.flush_interval_ms, 200); // Increased to allow more ops to accumulate
        assert_eq!(config.worker_count, 8);
        assert_eq!(config.lock_ttl_ms, 5_000); // Increased to allow larger batches
        assert_eq!(config.max_retry_count, 5);
        assert_eq!(config.params_inline_threshold_bytes, 2048); // 2KB - PostgreSQL TOAST threshold
    }
}
