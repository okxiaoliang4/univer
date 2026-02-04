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
use crate::services::storage::StorageService;
use anyhow::{Context, Result};
use futures::future::join_all;
use sea_orm::{
    sea_query::Expr, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QuerySelect, Set,
    TransactionTrait,
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Notify;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Configuration for write-behind workers
#[derive(Debug, Clone)]
pub struct WriteBehindConfig {
    pub batch_size: usize,
    pub flush_interval_ms: u64,
    pub worker_count: usize,
    pub lock_ttl_ms: u64,
    pub max_retry_count: u32,
    pub retry_base_delay_ms: u64,
}

impl Default for WriteBehindConfig {
    fn default() -> Self {
        Self {
            batch_size: 50,      // Reduced from 200 to limit transaction size and COMMIT time
            flush_interval_ms: 50, // Reduced from 100ms for more responsive flushing
            worker_count: 8,     // Increased from 4 for higher throughput
            lock_ttl_ms: 3_000,  // Reduced from 10s to 3s to reduce lock contention
            max_retry_count: 5,  // Increased from 3 for better resilience
            retry_base_delay_ms: 50, // Reduced from 100ms for faster retries
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
        // Reduced from 10 to 5 to limit memory usage per worker
        const MAX_BATCH_SIZE: usize = 5;

        loop {
            // Check for shutdown
            if shutdown.load(Ordering::SeqCst) {
                debug!("Worker {} received shutdown signal", worker_id);
                break;
            }

            // Wait for enqueue notification or periodic timeout (non-blocking)
            // This avoids blocking Redis connections with BRPOP
            tokio::select! {
                _ = cache_service.wait_for_enqueue() => {
                    // New document enqueued, try to dequeue
                }
                _ = tokio::time::sleep(Duration::from_millis(config.flush_interval_ms.max(50))) => {
                    // Periodic check in case we missed a notification
                }
                _ = shutdown_notify.notified() => {
                    debug!("Worker {} interrupted by shutdown", worker_id);
                    break;
                }
            }

            // Process documents in batch (non-blocking dequeue)
            let mut processed = 0;
            while processed < MAX_BATCH_SIZE {
                // Check shutdown between documents
                if shutdown.load(Ordering::SeqCst) {
                    break;
                }

                // Non-blocking dequeue - immediately releases Redis connection
                let doc_id = match cache_service.dequeue_doc_nonblocking().await {
                    Ok(Some(id)) => id,
                    Ok(None) => {
                        // Queue empty, wait for next notification
                        break;
                    }
                    Err(e) => {
                        // Log with rate limiting to avoid spam
                        static LAST_ERROR_LOG: std::sync::atomic::AtomicU64 =
                            std::sync::atomic::AtomicU64::new(0);
                        let now = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs();
                        let last = LAST_ERROR_LOG.load(std::sync::atomic::Ordering::Relaxed);
                        if now - last >= 10 {
                            LAST_ERROR_LOG.store(now, std::sync::atomic::Ordering::Relaxed);
                            warn!(
                                "Worker {} failed to dequeue (suppressing for 10s): {}",
                                worker_id, e
                            );
                        }
                        // Brief pause on error before retry
                        tokio::time::sleep(Duration::from_millis(100)).await;
                        break;
                    }
                };

                // Process the document
                debug!("Worker {} processing doc {}", worker_id, doc_id);

                match Self::flush_document_with_retry(
                    &db,
                    &cache_service,
                    &storage_service,
                    &config,
                    &doc_id,
                )
                .await
                {
                    Ok(count) => {
                        debug!(
                            "Worker {} flushed {} operations for doc {}",
                            worker_id, count, doc_id
                        );
                        metrics::increment_writebehind_ops_flushed_by(count as u64);
                    }
                    Err(e) => {
                        error!(
                            "Worker {} failed to flush doc {} after retries: {}",
                            worker_id, doc_id, e
                        );
                        // Re-enqueue for another attempt
                        if let Err(e) = cache_service.enqueue_doc(&doc_id).await {
                            error!("Failed to re-enqueue doc {}: {}", doc_id, e);
                        }
                    }
                }

                processed += 1;
            }
        }
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
    /// Optimized batch flow:
    /// 1. Get ops from cache (with params)
    /// 2. Batch upload params to S3 (concurrent)
    /// 3. Batch insert storage records
    /// 4. Batch insert operation_log records with storage_ids
    /// 5. Update document version
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

        // Get current DB version
        let db_version = documents::Entity::find_by_id(uuid)
            .select_only()
            .column(documents::Column::CurrentVersion)
            .into_tuple::<i64>()
            .one(db)
            .await?
            .context("Document not found")?;

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

        // Step 1: Batch upload params to S3 (with concurrency limit)
        // Process in chunks of 10 to avoid network resource exhaustion
        const S3_UPLOAD_CHUNK_SIZE: usize = 10;
        let s3_start = Instant::now();

        let mut uploaded_params = Vec::with_capacity(ops_count);
        for chunk in ops.chunks(S3_UPLOAD_CHUNK_SIZE) {
            let upload_futures: Vec<_> = chunk
                .iter()
                .map(|op| storage_service.upload_operation_params_to_s3(uuid, op.rev, &op.params))
                .collect();

            let chunk_results = join_all(upload_futures).await;

            // Process chunk results
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
        debug!(
            "S3 uploads completed for doc {}: {} ops in {:?}",
            doc_id,
            ops_count,
            s3_start.elapsed()
        );

        // Step 2: Begin transaction for all DB operations
        let txn = db.begin().await?;

        // Step 3: Batch insert storage records (in smaller chunks to avoid DB limits)
        // Reduced to 25 to minimize per-INSERT latency and reduce COMMIT time
        const DB_INSERT_CHUNK_SIZE: usize = 25;

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

        // Step 4: Batch insert operation_log records with storage_ids
        let mut operations_to_insert = Vec::with_capacity(ops_count);
        let mut max_rev = from_rev - 1;

        // Create a map of rev -> storage_id for quick lookup
        let rev_to_storage_id: std::collections::HashMap<i64, Uuid> = uploaded_params
            .iter()
            .map(|u| (u.rev, u.storage_id))
            .collect();

        for op in &ops {
            let storage_id = rev_to_storage_id
                .get(&op.rev)
                .copied()
                .context(format!("Missing storage_id for rev {}", op.rev))?;

            operations_to_insert.push(operation_log::ActiveModel {
                doc_id: Set(uuid),
                rev: Set(op.rev),
                user_id: Set(op.user_id.clone()),
                mutation_id: Set(op.mutation_id.clone()),
                storage_id: Set(storage_id),
                client_id: Set(op.client_id.clone()),
                op_id: Set(op.op_id.clone()),
                created_at: Set(chrono::DateTime::from_timestamp_millis(op.created_at)
                    .unwrap_or_else(chrono::Utc::now)
                    .into()),
                ..Default::default()
            });
            max_rev = max_rev.max(op.rev);
        }

        // Insert all operations (in chunks to avoid DB limits)
        for chunk in operations_to_insert.chunks(DB_INSERT_CHUNK_SIZE) {
            if !chunk.is_empty() {
                operation_log::Entity::insert_many(chunk.to_vec())
                    .exec(&txn)
                    .await
                    .context("Failed to insert operations")?;
            }
        }

        // Step 5: Update document version
        documents::Entity::update_many()
            .col_expr(documents::Column::CurrentVersion, Expr::value(max_rev))
            .col_expr(
                documents::Column::UpdatedAt,
                Expr::value(chrono::Utc::now()),
            )
            .filter(documents::Column::Id.eq(uuid))
            .filter(documents::Column::CurrentVersion.lt(max_rev))
            .exec(&txn)
            .await?;

        // Commit transaction (single COMMIT for all DB operations)
        txn.commit().await?;

        // Cache params in Redis for fast subsequent reads
        for (op, uploaded) in ops.iter().zip(uploaded_params.iter()) {
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

        debug!(
            "Flushed {} operations for doc {} (revs {}-{}) in {:?}",
            ops_count, doc_id, from_rev, max_rev, elapsed
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
        assert_eq!(config.batch_size, 50); // Reduced to limit transaction size and COMMIT time
        assert_eq!(config.flush_interval_ms, 50);
        assert_eq!(config.worker_count, 8);
        assert_eq!(config.lock_ttl_ms, 3_000);
        assert_eq!(config.max_retry_count, 5);
    }
}
