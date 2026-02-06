//! Write-behind worker implementation
//!
//! This module provides background workers that flush cached operations from Redis
//! to PostgreSQL in batches. Uses Redis Streams with Consumer Groups for reliable,
//! distributed message processing.
//!
//! ## Architecture
//!
//! - Each worker reads from a shared Redis Stream via XREADGROUP
//! - Messages are document IDs that need flushing
//! - After successful flush, messages are ACKed
//! - Failed messages stay in the PEL and are reclaimed via XAUTOCLAIM

use crate::config::WriteBehindConfig;
use crate::metrics;
use anyhow::{Context, Result};
use futures::future::join_all;
use ot_common::cache::{CacheService, StreamQueueService};
use ot_common::copy_writer::{copy_operations_atomic, CopyOperationData, VersionUpdate};
use ot_common::database::entities::{documents, operation_log, storage};
use ot_common::StreamMessage;
use ot_common::StorageService;
use sea_orm::{
    sea_query::Expr, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder,
    QuerySelect, TransactionTrait,
};
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Notify;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Prepared flush data for batched COPY
struct PreparedFlushData {
    doc_id: String,
    uuid: Uuid,
    max_rev: i64,
    cached_version: i64,
    copy_data: Vec<CopyOperationData>,
    ops_count: usize,
}

/// Write-behind worker manager
pub struct WriteBehindWorker {
    db: Arc<DatabaseConnection>,
    cache_service: Arc<CacheService>,
    queue_service: Arc<StreamQueueService>,
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
        queue_service: Arc<StreamQueueService>,
        storage_service: Arc<StorageService>,
        config: WriteBehindConfig,
    ) -> Self {
        Self {
            db,
            cache_service,
            queue_service,
            storage_service,
            config,
            shutdown: Arc::new(AtomicBool::new(false)),
            shutdown_notify: Arc::new(Notify::new()),
        }
    }

    /// Start the worker pool
    ///
    /// Creates the consumer group and spawns worker tasks.
    pub async fn start(&self) -> Result<Vec<tokio::task::JoinHandle<()>>> {
        // Ensure consumer group exists before starting workers
        self.queue_service.ensure_group().await?;

        let mut handles = Vec::with_capacity(self.config.worker_count);

        for worker_id in 0..self.config.worker_count {
            let db = self.db.clone();
            let cache_service = self.cache_service.clone();
            let queue_service = self.queue_service.clone();
            let storage_service = self.storage_service.clone();
            let config = self.config.clone();
            let shutdown = self.shutdown.clone();
            let shutdown_notify = self.shutdown_notify.clone();

            let consumer_name = format!(
                "worker-{}-{}",
                config.instance_id, worker_id
            );

            let handle = tokio::spawn(async move {
                info!("WriteBehindWorker {} started (consumer: {})", worker_id, consumer_name);
                Self::worker_loop(
                    worker_id,
                    &consumer_name,
                    db,
                    cache_service,
                    queue_service,
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

        info!("Started {} write-behind workers", self.config.worker_count);
        Ok(handles)
    }

    /// Signal workers to shut down
    pub fn shutdown(&self) {
        info!("Signaling write-behind workers to shutdown");
        self.shutdown.store(true, Ordering::SeqCst);
        self.shutdown_notify.notify_waiters();
    }

    /// Flush all pending documents (for graceful shutdown)
    ///
    /// Claims all pending messages and undelivered messages from the stream,
    /// then flushes each unique document.
    pub async fn flush_all(&self) -> Result<usize> {
        info!("Flushing all pending documents");
        let start = Instant::now();

        let shutdown_consumer = format!("shutdown-{}", self.config.instance_id);

        // Claim all pending messages from any consumer
        let claimed = self
            .queue_service
            .claim_pending(&shutdown_consumer, 0, 1000)
            .await
            .unwrap_or_default();

        // Also read any undelivered messages
        let undelivered = self
            .queue_service
            .read_messages(&shutdown_consumer, 1000, 0)
            .await
            .unwrap_or_default();

        // Collect unique doc_ids
        let mut seen = HashSet::new();
        let mut all_msg_ids = Vec::new();

        for msg in claimed.iter().chain(undelivered.iter()) {
            all_msg_ids.push(msg.id.clone());
            seen.insert(msg.doc_id.clone());
        }

        let unique_docs: Vec<String> = seen.into_iter().collect();
        let total = unique_docs.len();

        if total == 0 {
            info!("No pending documents to flush");
            return Ok(0);
        }

        info!("Found {} pending documents to flush", total);

        let mut flushed = 0;
        for doc_id in unique_docs {
            match Self::flush_document_with_retry(
                &self.db,
                &self.cache_service,
                &self.storage_service,
                &self.config,
                &doc_id,
            )
            .await
            {
                Ok(count) => {
                    flushed += count;
                    debug!("Flushed {} operations for doc {}", count, doc_id);
                }
                Err(e) => {
                    error!("Failed to flush doc {}: {}", doc_id, e);
                }
            }
        }

        // ACK all messages
        if !all_msg_ids.is_empty() {
            let _ = self.queue_service.ack(&all_msg_ids).await;
        }

        let elapsed = start.elapsed();
        info!(
            "Flush complete: flushed {} operations from {} documents in {:?}",
            flushed, total, elapsed
        );

        Ok(flushed)
    }

    /// Worker main loop using Redis Streams (XREADGROUP)
    ///
    /// Single-loop design replaces the old double-loop (connection + pubsub):
    /// 1. On startup: XAUTOCLAIM dead consumer messages
    /// 2. Main loop: XREADGROUP with BLOCK
    /// 3. Periodic XAUTOCLAIM every claim_interval_secs
    async fn worker_loop(
        worker_id: usize,
        consumer_name: &str,
        db: Arc<DatabaseConnection>,
        cache_service: Arc<CacheService>,
        queue_service: Arc<StreamQueueService>,
        storage_service: Arc<StorageService>,
        config: WriteBehindConfig,
        shutdown: Arc<AtomicBool>,
        shutdown_notify: Arc<Notify>,
    ) {
        // Step 1: On startup, claim any dead consumer messages
        match queue_service
            .claim_pending(consumer_name, config.claim_min_idle_ms, config.stream_batch_size)
            .await
        {
            Ok(claimed) if !claimed.is_empty() => {
                info!(
                    "Worker {} claimed {} messages from dead consumers on startup",
                    worker_id,
                    claimed.len()
                );
                metrics::increment_stream_messages_claimed_by(claimed.len() as u64);
                Self::process_messages(
                    worker_id,
                    claimed,
                    &db,
                    &cache_service,
                    &queue_service,
                    &storage_service,
                    &config,
                )
                .await;
            }
            Ok(_) => {}
            Err(e) => {
                warn!("Worker {} failed to claim on startup: {}", worker_id, e);
            }
        }

        let mut last_claim_time = Instant::now();

        // Step 2: Main loop
        loop {
            if shutdown.load(Ordering::SeqCst) {
                debug!("Worker {} received shutdown signal", worker_id);
                break;
            }

            // Read messages from stream
            let messages = tokio::select! {
                biased;

                _ = shutdown_notify.notified() => {
                    debug!("Worker {} interrupted by shutdown", worker_id);
                    break;
                }
                result = queue_service.read_messages(
                    consumer_name,
                    config.stream_batch_size,
                    config.flush_interval_ms,
                ) => {
                    match result {
                        Ok(msgs) => msgs,
                        Err(e) => {
                            warn!("Worker {} XREADGROUP error: {}", worker_id, e);
                            tokio::time::sleep(Duration::from_secs(1)).await;
                            continue;
                        }
                    }
                }
            };

            if !messages.is_empty() {
                metrics::increment_stream_messages_received_by(messages.len() as u64);
                Self::process_messages(
                    worker_id,
                    messages,
                    &db,
                    &cache_service,
                    &queue_service,
                    &storage_service,
                    &config,
                )
                .await;
            }

            // Step 3: Periodic XAUTOCLAIM
            if last_claim_time.elapsed() >= Duration::from_secs(config.claim_interval_secs) {
                last_claim_time = Instant::now();

                match queue_service
                    .claim_pending(consumer_name, config.claim_min_idle_ms, config.stream_batch_size)
                    .await
                {
                    Ok(claimed) if !claimed.is_empty() => {
                        info!(
                            "Worker {} claimed {} messages from dead consumers",
                            worker_id,
                            claimed.len()
                        );
                        metrics::increment_stream_messages_claimed_by(claimed.len() as u64);
                        Self::process_messages(
                            worker_id,
                            claimed,
                            &db,
                            &cache_service,
                            &queue_service,
                            &storage_service,
                            &config,
                        )
                        .await;
                    }
                    Ok(_) => {}
                    Err(e) => {
                        debug!("Worker {} claim_pending error: {}", worker_id, e);
                    }
                }
            }
        }
    }

    /// Process a batch of stream messages
    ///
    /// For each message:
    /// - Try to acquire flush lock
    /// - If lock fails: ACK (another worker is handling it)
    /// - Prepare flush data
    /// - Batch COPY to database
    /// - On success: update version, cleanup, ACK
    /// - On failure: don't ACK (stays in PEL for XAUTOCLAIM)
    async fn process_messages(
        worker_id: usize,
        messages: Vec<StreamMessage>,
        db: &DatabaseConnection,
        cache_service: &CacheService,
        queue_service: &StreamQueueService,
        storage_service: &StorageService,
        config: &WriteBehindConfig,
    ) {
        info!(
            "Worker {} processing {} messages",
            worker_id,
            messages.len()
        );

        let mut ack_ids: Vec<String> = Vec::new();
        let mut prepared_data: Vec<(String, PreparedFlushData)> = Vec::new(); // (msg_id, data)
        let mut skipped_count = 0usize;

        // Deduplicate: if multiple messages for the same doc_id, only process once
        let mut seen_docs: HashSet<String> = HashSet::new();

        for msg in &messages {
            if seen_docs.contains(&msg.doc_id) {
                // Duplicate doc_id in batch - just ACK the extra message
                ack_ids.push(msg.id.clone());
                continue;
            }
            seen_docs.insert(msg.doc_id.clone());

            // Try to acquire flush lock
            let lock_acquired = cache_service
                .acquire_flush_lock(&msg.doc_id, config.lock_ttl_ms)
                .await
                .unwrap_or(false);

            if !lock_acquired {
                // Another worker is handling this doc - ACK since it will be processed
                debug!(
                    "Worker {} skipped doc {} - lock held by another worker",
                    worker_id, msg.doc_id
                );
                metrics::increment_writebehind_skipped();
                ack_ids.push(msg.id.clone());
                skipped_count += 1;
                continue;
            }

            // Prepare flush data
            match Self::prepare_flush(db, cache_service, storage_service, config, &msg.doc_id).await
            {
                Ok(Some(data)) => {
                    prepared_data.push((msg.id.clone(), data));
                }
                Ok(None) => {
                    // Nothing to flush (version already current)
                    let _ = cache_service.release_flush_lock(&msg.doc_id).await;
                    ack_ids.push(msg.id.clone());
                    skipped_count += 1;
                }
                Err(e) => {
                    warn!(
                        "Worker {} prepare failed for doc {}: {}",
                        worker_id, msg.doc_id, e
                    );
                    let _ = cache_service.release_flush_lock(&msg.doc_id).await;
                    // Don't ACK - leave in PEL for retry via XAUTOCLAIM
                }
            }
        }

        if prepared_data.is_empty() {
            // ACK any messages we decided to skip
            if !ack_ids.is_empty() {
                info!(
                    "Worker {} nothing to flush ({} skipped), ACKing {} messages",
                    worker_id, skipped_count, ack_ids.len()
                );
                if let Err(e) = queue_service.ack(&ack_ids).await {
                    warn!("Worker {} failed to ACK {} messages: {}", worker_id, ack_ids.len(), e);
                } else {
                    metrics::increment_stream_messages_acked_by(ack_ids.len() as u64);
                }
            }
            return;
        }

        // Batch COPY all prepared operations atomically with version updates
        let total_ops: usize = prepared_data.iter().map(|(_, d)| d.ops_count).sum();
        let all_copy_data: Vec<CopyOperationData> = prepared_data
            .iter()
            .flat_map(|(_, d)| d.copy_data.clone())
            .collect();
        let version_updates: Vec<VersionUpdate> = prepared_data
            .iter()
            .map(|(_, d)| VersionUpdate {
                doc_id: d.uuid,
                max_rev: d.max_rev,
            })
            .collect();

        let copy_start = Instant::now();
        match copy_operations_atomic(db, &all_copy_data, &version_updates).await {
            Ok(rows) => {
                debug!(
                    "Worker {} atomic COPY+UPDATE: {} rows for {} docs in {:?}",
                    worker_id,
                    rows,
                    prepared_data.len(),
                    copy_start.elapsed()
                );

                // Update cache and cleanup for each document
                for (msg_id, data) in &prepared_data {
                    let _ = cache_service.set_db_version(data.uuid, data.max_rev).await;
                    let _ = cache_service
                        .remove_ops_up_to(data.uuid, data.max_rev)
                        .await;
                    let _ = cache_service.release_flush_lock(&data.doc_id).await;

                    // Re-enqueue if more operations pending
                    if data.max_rev < data.cached_version {
                        let _ = queue_service.enqueue(&data.doc_id).await;
                    }

                    ack_ids.push(msg_id.clone());
                }

                metrics::increment_writebehind_ops_flushed_by(total_ops as u64);
                metrics::record_writebehind_flush_latency(copy_start.elapsed().as_secs_f64());
            }
            Err(e) => {
                let error_str = format!("{:?}", e);
                let is_duplicate_key =
                    error_str.contains("duplicate key") || error_str.contains("23505");

                if is_duplicate_key {
                    warn!(
                        "Worker {} COPY duplicate key error, syncing from operation_logs",
                        worker_id
                    );

                    for (msg_id, data) in &prepared_data {
                        let actual_max_rev = operation_log::Entity::find()
                            .filter(operation_log::Column::DocId.eq(data.uuid))
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
                            let _ = cache_service
                                .set_db_version(data.uuid, actual_max_rev)
                                .await;
                            let _ = cache_service
                                .remove_ops_up_to(data.uuid, actual_max_rev)
                                .await;

                            let _ = documents::Entity::update_many()
                                .col_expr(
                                    documents::Column::CurrentVersion,
                                    Expr::value(actual_max_rev),
                                )
                                .filter(documents::Column::Id.eq(data.uuid))
                                .filter(documents::Column::CurrentVersion.lt(actual_max_rev))
                                .exec(db)
                                .await;
                        }

                        let _ = cache_service.release_flush_lock(&data.doc_id).await;
                        ack_ids.push(msg_id.clone());
                    }
                } else {
                    error!(
                        "Worker {} batch COPY failed: {}, leaving {} messages in PEL for retry",
                        worker_id,
                        e,
                        prepared_data.len()
                    );
                    metrics::increment_writebehind_error("copy_failed");

                    // Release locks but don't ACK - messages stay in PEL
                    for (_, data) in &prepared_data {
                        let _ = cache_service.release_flush_lock(&data.doc_id).await;
                    }
                }
            }
        }

        // ACK all successful messages
        if !ack_ids.is_empty() {
            if let Err(e) = queue_service.ack(&ack_ids).await {
                warn!("Worker {} failed to ACK {} messages: {}", worker_id, ack_ids.len(), e);
            } else {
                metrics::increment_stream_messages_acked_by(ack_ids.len() as u64);
            }
        }
    }

    /// Prepare flush data for a document
    async fn prepare_flush(
        db: &DatabaseConnection,
        cache_service: &CacheService,
        storage_service: &StorageService,
        config: &WriteBehindConfig,
        doc_id: &str,
    ) -> Result<Option<PreparedFlushData>> {
        let uuid = Uuid::parse_str(doc_id).context("Invalid document ID")?;

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

        // Get cached version - if version key is missing, check ops sorted set
        let cached_version = match cache_service.get_version(uuid).await {
            Ok(Some(v)) => v,
            Ok(None) | Err(_) => {
                match cache_service.get_max_cached_rev(uuid).await {
                    Ok(Some(max_rev)) => {
                        debug!(
                            "Version key missing for doc {}, using max_cached_rev={}",
                            doc_id, max_rev
                        );
                        max_rev
                    }
                    Ok(None) | Err(_) => {
                        debug!(
                            "No version or ops in cache for doc {}, skipping",
                            doc_id
                        );
                        metrics::increment_writebehind_skipped();
                        return Ok(None);
                    }
                }
            }
        };

        if cached_version <= db_version {
            debug!(
                "Skipping flush for doc {} - cached_version ({}) <= db_version ({})",
                doc_id, cached_version, db_version
            );
            metrics::increment_writebehind_skipped();
            return Ok(None);
        }

        // Get operations
        let from_rev = db_version + 1;
        let to_rev = (from_rev + config.batch_size as i64 - 1).min(cached_version);
        let ops = cache_service.get_ops_range(uuid, from_rev, to_rev).await?;

        if ops.is_empty() {
            info!(
                "No cached operations found for doc {} in range [{}, {}]",
                doc_id, from_rev, to_rev
            );
            metrics::increment_writebehind_skipped();
            return Ok(None);
        }

        let ops_count = ops.len();
        let threshold = config.params_inline_threshold_bytes;

        // Partition by size
        let (_small_ops, large_ops): (Vec<_>, Vec<_>) =
            ops.iter().partition(|op| op.params.len() < threshold);

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
                        return Err(e.context(format!("S3 upload failed for rev {}", chunk[i].rev)));
                    }
                }
            }
        }

        // Insert storage records
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

        Ok(Some(PreparedFlushData {
            doc_id: doc_id.to_string(),
            uuid,
            max_rev,
            cached_version,
            copy_data,
            ops_count,
        }))
    }

    /// Flush a document with retries (used by flush_all for graceful shutdown)
    async fn flush_document_with_retry(
        db: &DatabaseConnection,
        cache_service: &CacheService,
        storage_service: &StorageService,
        config: &WriteBehindConfig,
        doc_id: &str,
    ) -> Result<usize> {
        let lock_acquired = cache_service
            .acquire_flush_lock(doc_id, config.lock_ttl_ms)
            .await
            .unwrap_or(false);

        if !lock_acquired {
            debug!(
                "Skipping flush for doc {} - another worker is flushing",
                doc_id
            );
            metrics::increment_writebehind_skipped();
            return Ok(0);
        }

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

        if let Err(e) = cache_service.release_flush_lock(doc_id).await {
            warn!("Failed to release flush lock for doc {}: {}", doc_id, e);
        }

        result
    }

    /// Internal flush implementation (for graceful shutdown flush_all)
    async fn flush_document_impl(
        db: &DatabaseConnection,
        cache_service: &CacheService,
        storage_service: &StorageService,
        config: &WriteBehindConfig,
        doc_id: &str,
    ) -> Result<usize> {
        let start = Instant::now();
        let uuid = Uuid::parse_str(doc_id).context("Invalid document ID")?;

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

        let cached_version = match cache_service.get_version(uuid).await {
            Ok(Some(v)) => v,
            Ok(None) | Err(_) => {
                cache_service
                    .get_max_cached_rev(uuid)
                    .await?
                    .unwrap_or(db_version)
            }
        };

        if cached_version <= db_version {
            return Ok(0);
        }

        let from_rev = db_version + 1;
        let to_rev = (from_rev + config.batch_size as i64 - 1).min(cached_version);
        let ops = cache_service.get_ops_range(uuid, from_rev, to_rev).await?;

        if ops.is_empty() {
            return Ok(0);
        }

        let ops_count = ops.len();
        let threshold = config.params_inline_threshold_bytes;

        let (_small_ops, large_ops): (Vec<_>, Vec<_>) =
            ops.iter().partition(|op| op.params.len() < threshold);

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
                        return Err(e.context(format!("S3 upload failed for rev {}", chunk[i].rev)));
                    }
                }
            }
        }

        // Insert storage records
        let txn = db.begin().await?;

        if !uploaded_params.is_empty() {
            let storage_records: Vec<storage::ActiveModel> = uploaded_params
                .iter()
                .map(|u| u.storage_record.clone())
                .collect();

            for chunk in storage_records.chunks(25) {
                if !chunk.is_empty() {
                    storage::Entity::insert_many(chunk.to_vec())
                        .exec(&txn)
                        .await
                        .context("Failed to batch insert storage records")?;
                }
            }
        }

        txn.commit().await?;

        // Build COPY data
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

        // Atomic COPY operation_logs + UPDATE document version
        let version_updates = vec![VersionUpdate {
            doc_id: uuid,
            max_rev,
        }];
        copy_operations_atomic(db, &copy_data, &version_updates)
            .await
            .context("Atomic COPY + UPDATE failed")?;

        let _ = cache_service.set_db_version(uuid, max_rev).await;
        let _ = cache_service.remove_ops_up_to(uuid, max_rev).await;

        // Cache large params
        for (op, uploaded) in large_ops.iter().zip(uploaded_params.iter()) {
            let _ = cache_service
                .cache_params(uploaded.storage_id, &op.params)
                .await;
        }

        let elapsed = start.elapsed();
        metrics::record_writebehind_flush_latency(elapsed.as_secs_f64());
        metrics::increment_writebehind_ops_flushed_by(ops_count as u64);

        debug!(
            "Flushed {} operations for doc {} (revs {}-{}) in {:?}",
            ops_count, doc_id, from_rev, max_rev, elapsed
        );

        Ok(ops_count)
    }

    /// Get queue depth (for monitoring)
    pub async fn get_queue_depth(&self) -> Result<usize> {
        let info = self.queue_service.get_info().await?;
        Ok(info.pending_count as usize)
    }
}

impl Drop for WriteBehindWorker {
    fn drop(&mut self) {
        self.shutdown();
    }
}
