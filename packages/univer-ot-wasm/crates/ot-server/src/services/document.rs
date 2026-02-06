use crate::database::entities::{document_snapshot, documents, operation_log};
use crate::metrics;
use crate::services::params_codec;
use ot_common::CacheService;
use ot_common::storage::{StorageService, StoredSnapshot};
use anyhow::{Context, Result};
use futures::TryStreamExt;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder,
    QuerySelect, TransactionTrait,
};
use serde_json::{json, Value as JsonValue};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

#[derive(Clone)]
pub struct DocumentService {
    db: Arc<DatabaseConnection>,
    storage_service: Arc<StorageService>,
    cache_service: Arc<CacheService>,
}

impl DocumentService {
    pub fn new(
        db: Arc<DatabaseConnection>,
        storage_service: Arc<StorageService>,
        cache_service: Arc<CacheService>,
    ) -> Self {
        Self {
            db,
            storage_service,
            cache_service,
        }
    }

    /// Create a new document with initial content
    /// Uses a transaction to ensure atomicity: if snapshot creation fails, document creation is rolled back
    pub async fn create_document(
        &self,
        doc_id: Uuid,
        creator_id: String,
        name: String,
        doc_type: i16,
        create_type: i16,
        initial_content: JsonValue,
    ) -> Result<i64> {
        info!("Creating document: doc_id={}, name={}, doc_type={}, creator_id={}",
            doc_id, name, doc_type, creator_id);

        // Start transaction to ensure atomicity
        let txn = (*self.db).begin().await
            .context("Failed to start transaction for document creation")?;

        let now = chrono::Utc::now();
        let users = json!({
            creator_id.clone(): now.timestamp() as i32,
        });

        // Create document entry in documents table (within transaction)
        let document = documents::ActiveModel {
            id: sea_orm::Set(doc_id),
            name: sea_orm::Set(name.clone()),
            creator_id: sea_orm::Set(creator_id.clone()),
            doc_type: sea_orm::Set(doc_type),
            create_type: sea_orm::Set(create_type),
            current_version: sea_orm::Set(1),
            created_at: sea_orm::Set(now.into()),
            updated_at: sea_orm::Set(now.into()),
        };

        if let Err(e) = documents::Entity::insert(document).exec(&txn).await {
            error!("Failed to insert document: doc_id={}, error={}", doc_id, e);
            let _ = txn.rollback().await;
            return Err(e.into());
        }
        debug!("Document inserted into database: doc_id={}", doc_id);

        // Create initial snapshot
        // Note: Storage operations are outside the transaction, but if snapshot insert fails,
        // the transaction will rollback and we'll have orphaned storage data.
        // In production, consider adding cleanup logic or making storage operations transactional.
        debug!("Storing initial snapshot content: doc_id={}", doc_id);
        let StoredSnapshot { storage_id, size } = match self
            .storage_service
            .store_snapshot_content(doc_id, 0, &initial_content)
            .await
        {
            Ok(result) => {
                info!("Snapshot content stored: doc_id={}, storage_id={}, size={}",
                    doc_id, result.storage_id, result.size);
                result
            }
            Err(e) => {
                error!("Failed to store snapshot content: doc_id={}, error={}", doc_id, e);
                let _ = txn.rollback().await;
                return Err(e);
            }
        };

        let snapshot = document_snapshot::ActiveModel {
            id: sea_orm::Set(doc_id),     // id = doc_id for snapshot
            doc_id: sea_orm::Set(doc_id), // doc_id references documents.id
            storage_id: sea_orm::Set(storage_id),
            name: sea_orm::Set(Some(name.clone())),
            size: sea_orm::Set(Some(size)),
            users: sea_orm::Set(Some(users.into())),
            restore_from_id: sea_orm::Set(None),
            version: sea_orm::Set(1), // Snapshot version = version at which snapshot was taken
            created_at: sea_orm::Set(now.into()),
            updated_at: sea_orm::Set(now.into()),
        };

        if let Err(e) = document_snapshot::Entity::insert(snapshot)
            .exec(&txn)
            .await
        {
            error!("Failed to insert snapshot: doc_id={}, error={}", doc_id, e);
            let _ = txn.rollback().await;
            return Err(e.into());
        }

        // Commit transaction
        if let Err(e) = txn.commit().await {
            error!("Failed to commit transaction: doc_id={}, error={}", doc_id, e);
            return Err(e.into());
        }

        info!("Document created successfully: doc_id={}, name={}, version=1", doc_id, name);
        Ok(0)
    }

    pub async fn create_document_from_url(
        &self,
        doc_id: Uuid,
        creator_id: String,
        name: String,
        doc_type: i16,
        create_type: i16,
        url: String,
    ) -> Result<i64> {
        info!("Creating document from URL: doc_id={}, url={}", doc_id, url);

        debug!("Fetching content from URL: url={}", url);
        let response = match reqwest::get(&url).await {
            Ok(resp) => resp,
            Err(e) => {
                error!("Failed to fetch URL: url={}, error={}", url, e);
                return Err(e.into());
            }
        };

        let bytes = match response.bytes().await {
            Ok(b) => b,
            Err(e) => {
                error!("Failed to read response bytes: url={}, error={}", url, e);
                return Err(e.into());
            }
        };

        let content = match serde_json::from_slice::<JsonValue>(&bytes) {
            Ok(c) => {
                debug!("Parsed JSON content from URL: url={}, size={}", url, bytes.len());
                c
            }
            Err(e) => {
                error!("Failed to parse JSON from URL: url={}, error={}", url, e);
                return Err(e.into());
            }
        };

        self.create_document(doc_id, creator_id, name, doc_type, create_type, content)
            .await
    }

    /// Get document snapshot by doc_id
    pub async fn get_document(&self, doc_id: Uuid) -> Result<Option<(Uuid, i64)>> {
        info!("get_document called with ORDER BY version DESC: doc_id={}", doc_id);

        let snapshot = match document_snapshot::Entity::find()
            .filter(document_snapshot::Column::DocId.eq(doc_id))
            .order_by_desc(document_snapshot::Column::Version)
            .one(&*self.db)
            .await
        {
            Ok(s) => {
                info!("Query result: doc_id={}, found={}, version={:?}",
                    doc_id, s.is_some(), s.as_ref().map(|sn| sn.version));
                s
            }
            Err(e) => {
                error!("Failed to query document snapshot: doc_id={}, error={}", doc_id, e);
                return Err(e.into());
            }
        };

        match snapshot {
            Some(s) => {
                info!("Found document snapshot: doc_id={}, storage_id={}, version={}",
                    doc_id, s.storage_id, s.version);
                Ok(Some((s.storage_id, s.version)))
            }
            None => {
                debug!("Document snapshot not found: doc_id={}", doc_id);
                Ok(None)
            }
        }
    }

    pub async fn get_document_metadata(&self, doc_id: Uuid) -> Result<Option<documents::Model>> {
        debug!("Getting document metadata: doc_id={}", doc_id);

        match documents::Entity::find_by_id(doc_id).one(&*self.db).await {
            Ok(Some(doc)) => {
                debug!("Found document metadata: doc_id={}, name={}, version={}",
                    doc_id, doc.name, doc.current_version);
                Ok(Some(doc))
            }
            Ok(None) => {
                debug!("Document metadata not found: doc_id={}", doc_id);
                Ok(None)
            }
            Err(e) => {
                error!("Failed to query document metadata: doc_id={}, error={}", doc_id, e);
                Err(e.into())
            }
        }
    }

    pub async fn clone_document(
        &self,
        source_doc_id: Uuid,
        creator_id: String,
        snapshot_id: Option<Uuid>,
        doc_type: Option<i16>,
    ) -> Result<(Uuid, i64)> {
        info!("Cloning document: source_doc_id={}, snapshot_id={:?}, creator_id={}",
            source_doc_id, snapshot_id, creator_id);

        let source_document = match documents::Entity::find_by_id(source_doc_id)
            .one(&*self.db)
            .await
        {
            Ok(Some(doc)) => doc,
            Ok(None) => {
                error!("Source document not found: source_doc_id={}", source_doc_id);
                return Err(anyhow::anyhow!("Document not found: {}", source_doc_id));
            }
            Err(e) => {
                error!("Failed to query source document: source_doc_id={}, error={}", source_doc_id, e);
                return Err(e.into());
            }
        };

        debug!("Fetching source snapshot: source_doc_id={}, snapshot_id={:?}",
            source_doc_id, snapshot_id);
        let source_snapshot = match self
            .get_snapshot_model(source_doc_id, snapshot_id)
            .await
        {
            Ok(Some(snapshot)) => snapshot,
            Ok(None) => {
                error!("Snapshot not found: source_doc_id={}, snapshot_id={:?}",
                    source_doc_id, snapshot_id);
                return Err(anyhow::anyhow!("Snapshot not found for document: {}", source_doc_id));
            }
            Err(e) => {
                error!("Failed to get snapshot model: source_doc_id={}, error={}", source_doc_id, e);
                return Err(e);
            }
        };

        debug!("Fetching snapshot content: storage_id={}", source_snapshot.storage_id);
        let content = match self
            .storage_service
            .fetch_snapshot_content(source_snapshot.storage_id)
            .await
        {
            Ok(c) => {
                info!("Fetched snapshot content: storage_id={}, size={}",
                    source_snapshot.storage_id, source_snapshot.size.unwrap_or(0));
                c
            }
            Err(e) => {
                error!("Failed to fetch snapshot content: storage_id={}, error={}",
                    source_snapshot.storage_id, e);
                return Err(e);
            }
        };

        let new_doc_id = Uuid::new_v4();
        let doc_name = format!("{} (Clone)", source_document.name);
        let doc_type = doc_type.unwrap_or(source_document.doc_type);

        info!("Creating cloned document: new_doc_id={}, name={}", new_doc_id, doc_name);
        if let Err(e) = self.create_document(
            new_doc_id,
            creator_id,
            doc_name,
            doc_type,
            source_document.create_type,
            content,
        )
        .await
        {
            error!("Failed to create cloned document: new_doc_id={}, error={}", new_doc_id, e);
            return Err(e);
        }

        let size = source_snapshot.size.unwrap_or(0);
        info!("Document cloned successfully: source_doc_id={}, new_doc_id={}, size={}",
            source_doc_id, new_doc_id, size);
        Ok((new_doc_id, size))
    }

    pub async fn delete_document(&self, doc_id: Uuid, is_soft: bool) -> Result<()> {
        info!("Deleting document: doc_id={}, is_soft={}", doc_id, is_soft);

        match documents::Entity::delete_by_id(doc_id).exec(&*self.db).await {
            Ok(_) => {
                info!("Document deleted successfully: doc_id={}, is_soft={}", doc_id, is_soft);
                Ok(())
            }
            Err(e) => {
                error!("Failed to delete document: doc_id={}, error={}", doc_id, e);
                Err(e.into())
            }
        }
    }

    pub async fn get_doc_snapshot(
        &self,
        doc_id: Uuid,
        snapshot_id: Option<Uuid>,
    ) -> Result<Option<DocumentSnapshotInfo>> {
        debug!("Getting document snapshot: doc_id={}, snapshot_id={:?}", doc_id, snapshot_id);

        let snapshot = match self.get_snapshot_model(doc_id, snapshot_id).await {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to get snapshot model: doc_id={}, snapshot_id={:?}, error={}",
                    doc_id, snapshot_id, e);
                return Err(e);
            }
        };

        let Some(snapshot) = snapshot else {
            debug!("Snapshot not found: doc_id={}, snapshot_id={:?}", doc_id, snapshot_id);
            return Ok(None);
        };

        let restore_from = if let Some(restore_from_id) = snapshot.restore_from_id {
            debug!("Fetching restore_from snapshot: restore_from_id={}", restore_from_id);
            match document_snapshot::Entity::find_by_id(restore_from_id)
                .one(&*self.db)
                .await
            {
                Ok(Some(model)) => Some(DocumentSnapshotInfo::from_model(model, None)),
                Ok(None) => {
                    warn!("Restore_from snapshot not found: restore_from_id={}", restore_from_id);
                    None
                }
                Err(e) => {
                    error!("Failed to query restore_from snapshot: restore_from_id={}, error={}",
                        restore_from_id, e);
                    None
                }
            }
        } else {
            None
        };

        info!("Retrieved document snapshot: doc_id={}, snapshot_id={:?}, version={}",
            doc_id, snapshot_id, snapshot.version);
        Ok(Some(DocumentSnapshotInfo::from_model(snapshot, restore_from)))
    }

    pub async fn list_snapshots(
        &self,
        doc_id: Uuid,
        limit: i32,
        cursor: Option<i64>,
        desc: bool,
    ) -> Result<(Vec<DocumentSnapshotInfo>, Option<i64>)> {
        let limit = if limit <= 0 { 10 } else { limit } as u64;
        info!("list_snapshots called: doc_id={}, limit={}, cursor={:?}, desc={}",
            doc_id, limit, cursor, desc);
        debug!("Filter will be: doc_id={}, cursor_filter={}, order={}",
            doc_id,
            if let Some(c) = cursor { if desc { format!("version < {}", c) } else { format!("version > {}", c) } } else { "none".to_string() },
            if desc { "DESC" } else { "ASC" }
        );

        let mut query = document_snapshot::Entity::find()
            .filter(document_snapshot::Column::DocId.eq(doc_id));

        // Only apply cursor filter if cursor > 0 (all versions are positive)
        // When cursor is 0 or negative, it means we're starting from the beginning
        if let Some(cursor) = cursor {
            if cursor > 0 {
                if desc {
                    query = query.filter(document_snapshot::Column::Version.lt(cursor));
                } else {
                    query = query.filter(document_snapshot::Column::Version.gt(cursor));
                }
            }
        }

        query = if desc {
            query.order_by_desc(document_snapshot::Column::Version)
        } else {
            query.order_by_asc(document_snapshot::Column::Version)
        };

        let snapshots: Vec<document_snapshot::Model> = match query.limit(limit).all(&*self.db).await {
            Ok(s) => {
                info!("Query executed successfully: doc_id={}, results_count={}, desc={}",
                    doc_id, s.len(), desc);
                s
            }
            Err(e) => {
                error!("Failed to query snapshots: doc_id={}, error={}", doc_id, e);
                return Err(e.into());
            }
        };

        let next_cursor = snapshots.last().map(|snapshot| snapshot.version);
        let infos: Vec<DocumentSnapshotInfo> = snapshots
            .into_iter()
            .map(|snapshot| DocumentSnapshotInfo::from_model(snapshot, None))
            .collect();

        info!("Listed snapshots: doc_id={}, count={}, next_cursor={:?}",
            doc_id, infos.len(), next_cursor);
        Ok((infos, next_cursor))
    }

    pub async fn update_snapshot_name(
        &self,
        doc_id: Uuid,
        snapshot_id: Uuid,
        name: String,
    ) -> Result<()> {
        info!("Updating snapshot name: doc_id={}, snapshot_id={}, name={}",
            doc_id, snapshot_id, name);

        let mut snapshot: document_snapshot::ActiveModel = match document_snapshot::Entity::find_by_id(snapshot_id)
            .filter(document_snapshot::Column::DocId.eq(doc_id))
            .one(&*self.db)
            .await
        {
            Ok(Some(s)) => s.into(),
            Ok(None) => {
                error!("Snapshot not found: doc_id={}, snapshot_id={}", doc_id, snapshot_id);
                return Err(anyhow::anyhow!("Snapshot not found: {}", snapshot_id));
            }
            Err(e) => {
                error!("Failed to query snapshot: doc_id={}, snapshot_id={}, error={}",
                    doc_id, snapshot_id, e);
                return Err(e.into());
            }
        };

        snapshot.name = sea_orm::Set(Some(name.clone()));
        snapshot.updated_at = sea_orm::Set(chrono::Utc::now().into());

        match snapshot.update(&*self.db).await {
            Ok(_) => {
                info!("Snapshot name updated successfully: doc_id={}, snapshot_id={}, name={}",
                    doc_id, snapshot_id, name);
                Ok(())
            }
            Err(e) => {
                error!("Failed to update snapshot name: doc_id={}, snapshot_id={}, error={}",
                    doc_id, snapshot_id, e);
                Err(e.into())
            }
        }
    }

    pub async fn get_latest_snapshots(
        &self,
        doc_ids: Vec<Uuid>,
    ) -> Result<HashMap<Uuid, DocumentSnapshotInfo>> {
        info!("Getting latest snapshots for {} documents", doc_ids.len());

        let mut results = HashMap::new();
        for doc_id in doc_ids.iter() {
            match self.get_doc_snapshot(*doc_id, None).await {
                Ok(Some(snapshot)) => {
                    debug!("Found latest snapshot: doc_id={}, version={}", doc_id, snapshot.version);
                    results.insert(*doc_id, snapshot);
                }
                Ok(None) => {
                    debug!("No snapshot found for document: doc_id={}", doc_id);
                }
                Err(e) => {
                    error!("Failed to get snapshot for document: doc_id={}, error={}", doc_id, e);
                    // Continue processing other documents
                }
            }
        }

        info!("Retrieved latest snapshots: requested={}, found={}",
            doc_ids.len(), results.len());
        Ok(results)
    }

    pub async fn sign_object_urls(
        &self,
        storage_ids: Vec<Uuid>,
    ) -> Result<HashMap<Uuid, String>> {
        info!("Signing object URLs for {} storage IDs", storage_ids.len());

        let mut urls = HashMap::new();
        for storage_id in storage_ids.iter() {
            match self.storage_service.get_storage_location(*storage_id).await {
                Ok(storage) => {
                    match self.storage_service.get_signed_url(&storage).await {
                        Ok(signed_url) => {
                            debug!("Signed URL generated: storage_id={}", storage_id);
                            urls.insert(*storage_id, signed_url);
                        }
                        Err(e) => {
                            error!("Failed to sign URL: storage_id={}, error={}", storage_id, e);
                            return Err(e);
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to get storage location: storage_id={}, error={}", storage_id, e);
                    return Err(e);
                }
            }
        }

        info!("Signed {} URLs successfully", urls.len());
        Ok(urls)
    }

    pub async fn get_doc_id_by_storage_id(&self, storage_id: Uuid) -> Result<Option<Uuid>> {
        debug!("Getting doc_id by storage_id: storage_id={}", storage_id);

        let snapshot = match document_snapshot::Entity::find()
            .filter(document_snapshot::Column::StorageId.eq(storage_id))
            .one(&*self.db)
            .await
        {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to query snapshot by storage_id: storage_id={}, error={}",
                    storage_id, e);
                return Err(e.into());
            }
        };

        match snapshot {
            Some(model) => {
                info!("Found doc_id for storage_id: storage_id={}, doc_id={}",
                    storage_id, model.doc_id);
                Ok(Some(model.doc_id))
            }
            None => {
                debug!("No document found for storage_id: storage_id={}", storage_id);
                Ok(None)
            }
        }
    }

    /// Restore a document from a snapshot
    /// Uses a transaction to ensure atomicity: if document update fails, snapshot creation is rolled back
    pub async fn restore_document(
        &self,
        doc_id: Uuid,
        snapshot_id: Uuid,
    ) -> Result<DocumentSnapshotInfo> {
        info!("Restoring document: doc_id={}, snapshot_id={}", doc_id, snapshot_id);

        // Start transaction to ensure atomicity
        let txn = (*self.db).begin().await
            .context("Failed to start transaction for document restore")?;

        let document = match documents::Entity::find_by_id(doc_id)
            .one(&txn)
            .await
        {
            Ok(Some(doc)) => doc,
            Ok(None) => {
                error!("Document not found: doc_id={}", doc_id);
                let _ = txn.rollback().await;
                return Err(anyhow::anyhow!("Document not found: {}", doc_id));
            }
            Err(e) => {
                error!("Failed to query document: doc_id={}, error={}", doc_id, e);
                let _ = txn.rollback().await;
                return Err(e.into());
            }
        };

        let target_snapshot = match document_snapshot::Entity::find_by_id(snapshot_id)
            .filter(document_snapshot::Column::DocId.eq(doc_id))
            .one(&txn)
            .await
        {
            Ok(Some(snapshot)) => snapshot,
            Ok(None) => {
                error!("Snapshot not found: doc_id={}, snapshot_id={}", doc_id, snapshot_id);
                let _ = txn.rollback().await;
                return Err(anyhow::anyhow!("Snapshot not found: {}", snapshot_id));
            }
            Err(e) => {
                error!("Failed to query snapshot: doc_id={}, snapshot_id={}, error={}",
                    doc_id, snapshot_id, e);
                let _ = txn.rollback().await;
                return Err(e.into());
            }
        };

        let new_version = document.current_version + 1;
        info!("Creating restored snapshot: doc_id={}, snapshot_id={}, new_version={}",
            doc_id, snapshot_id, new_version);

        let now = chrono::Utc::now();
        let restored_snapshot = document_snapshot::ActiveModel {
            id: sea_orm::Set(Uuid::new_v4()),
            doc_id: sea_orm::Set(doc_id),
            storage_id: sea_orm::Set(target_snapshot.storage_id),
            name: sea_orm::Set(target_snapshot.name.clone()),
            size: sea_orm::Set(target_snapshot.size),
            users: sea_orm::Set(target_snapshot.users.clone()),
            restore_from_id: sea_orm::Set(Some(target_snapshot.id)),
            version: sea_orm::Set(new_version),
            created_at: sea_orm::Set(now.into()),
            updated_at: sea_orm::Set(now.into()),
        };

        let restored_model = match restored_snapshot.insert(&txn).await {
            Ok(model) => {
                info!("Restored snapshot inserted: doc_id={}, new_version={}", doc_id, new_version);
                model
            }
            Err(e) => {
                error!("Failed to insert restored snapshot: doc_id={}, error={}", doc_id, e);
                let _ = txn.rollback().await;
                return Err(e.into());
            }
        };

        let mut document: documents::ActiveModel = document.into();
        document.current_version = sea_orm::Set(new_version);
        document.updated_at = sea_orm::Set(now.into());

        match document.update(&txn).await {
            Ok(_) => {
                info!("Document version updated: doc_id={}, new_version={}", doc_id, new_version);
            }
            Err(e) => {
                error!("Failed to update document version: doc_id={}, error={}", doc_id, e);
                let _ = txn.rollback().await;
                return Err(e.into());
            }
        }

        // Commit transaction
        if let Err(e) = txn.commit().await {
            error!("Failed to commit transaction: doc_id={}, error={}", doc_id, e);
            return Err(e.into());
        }

        info!("Document restored successfully: doc_id={}, snapshot_id={}, new_version={}",
            doc_id, snapshot_id, new_version);
        Ok(DocumentSnapshotInfo::from_model(restored_model, None))
    }

    async fn get_snapshot_model(
        &self,
        doc_id: Uuid,
        snapshot_id: Option<Uuid>,
    ) -> Result<Option<document_snapshot::Model>> {
        if let Some(snapshot_id) = snapshot_id {
            debug!("Getting snapshot by ID: snapshot_id={}", snapshot_id);
            match document_snapshot::Entity::find_by_id(snapshot_id)
                .one(&*self.db)
                .await
            {
                Ok(snapshot) => Ok(snapshot),
                Err(e) => {
                    error!("Failed to query snapshot by ID: snapshot_id={}, error={}", snapshot_id, e);
                    Err(e.into())
                }
            }
        } else {
            info!("Getting latest snapshot with ORDER BY version DESC: doc_id={}", doc_id);
            match document_snapshot::Entity::find()
                .filter(document_snapshot::Column::DocId.eq(doc_id))
                .order_by_desc(document_snapshot::Column::Version)
                .one(&*self.db)
                .await
            {
                Ok(snapshot) => {
                    info!("Latest snapshot query result: doc_id={}, found={}, version={:?}",
                        doc_id, snapshot.is_some(), snapshot.as_ref().map(|s| s.version));
                    Ok(snapshot)
                }
                Err(e) => {
                    error!("Failed to query latest snapshot: doc_id={}, error={}", doc_id, e);
                    Err(e.into())
                }
            }
        }
    }

    /// Maximum number of operations that can be fetched in a single query.
    /// This prevents memory explosion when querying large operation histories.
    pub const MAX_OPERATIONS_LIMIT: u64 = 10000;

    /// Default limit for operation queries when no limit is specified (for API calls).
    pub const DEFAULT_OPERATIONS_LIMIT: u64 = 1000;

    /// Get operations for a document within a revision range.
    /// The `limit` parameter controls the maximum number of operations returned:
    /// - `Some(n)`: Return at most n operations (capped at MAX_OPERATIONS_LIMIT)
    /// - `None`: No limit (used internally for OT transformation which needs all ops)
    ///
    /// Params retrieval priority:
    /// 1. Inline params (stored directly in DB for small operations)
    /// 2. Redis cache (for recently accessed large operations)
    /// 3. S3 storage (fallback for large operations)
    pub async fn get_operations(
        &self,
        doc_id: Uuid,
        from_rev: i64,
        to_rev: Option<i64>,
        limit: Option<u64>,
    ) -> Result<Vec<OperationInfo>> {
        debug!("Getting operations: doc_id={}, from_rev={}, to_rev={:?}, limit={:?}",
            doc_id, from_rev, to_rev, limit);

        let mut query = operation_log::Entity::find()
            .filter(operation_log::Column::DocId.eq(doc_id))
            .filter(operation_log::Column::Rev.gte(from_rev))
            .order_by_asc(operation_log::Column::Rev);

        if let Some(to_rev) = to_rev {
            query = query.filter(operation_log::Column::Rev.lte(to_rev));
        }

        // Apply limit if specified (capped at MAX_OPERATIONS_LIMIT)
        if let Some(limit) = limit {
            let effective_limit = limit.min(Self::MAX_OPERATIONS_LIMIT);
            query = query.limit(effective_limit);
        }

        let operations = match query.all(&*self.db).await {
            Ok(ops) => ops,
            Err(e) => {
                error!("Failed to query operations: doc_id={}, error={}", doc_id, e);
                return Err(e.into());
            }
        };

        let count = operations.len();

        // Warn if fetching a large number of operations (potential memory concern)
        if count > 1000 {
            warn!("Large operation query: doc_id={}, count={}, limit={:?}", doc_id, count, limit);
        }

        // Collect storage_ids for operations that need S3 fetch (no inline params)
        let storage_ids: Vec<Uuid> = operations
            .iter()
            .filter_map(|op| {
                // Only need storage_id if params are not stored inline
                if op.params.is_none() {
                    op.storage_id
                } else {
                    None
                }
            })
            .collect();

        // Batch fetch params from Redis cache for large operations
        let cached_params = if !storage_ids.is_empty() {
            self.cache_service
                .get_params_cached_batch(&storage_ids)
                .await
                .unwrap_or_default()
        } else {
            std::collections::HashMap::new()
        };

        let mut result = Vec::with_capacity(count);
        let mut inline_count = 0;
        let mut cache_hit_count = 0;
        let mut s3_fetch_count = 0;

        for op in operations {
            // Priority 1: Inline params (small operations stored directly in DB)
            let params_bytes = if let Some(ref inline_params) = op.params {
                inline_count += 1;
                inline_params.clone()
            } else if let Some(storage_id) = op.storage_id {
                // Priority 2: Redis cache
                if let Some(bytes) = cached_params.get(&storage_id) {
                    cache_hit_count += 1;
                    bytes.clone()
                } else {
                    // Priority 3: Fetch from S3
                    s3_fetch_count += 1;
                    let bytes = self
                        .storage_service
                        .fetch_operation_params(storage_id)
                        .await
                        .with_context(|| {
                            format!(
                                "Failed to fetch params from S3: doc_id={}, rev={}, storage_id={}",
                                doc_id, op.rev, storage_id
                            )
                        })?;

                    // Cache in Redis for future reads
                    if let Err(e) = self.cache_service.cache_params(storage_id, &bytes).await {
                        warn!("Failed to cache params after S3 fetch: {}", e);
                    }

                    bytes
                }
            } else {
                // Neither inline params nor storage_id - data integrity issue
                return Err(anyhow::anyhow!(
                    "Operation has neither inline params nor storage_id: doc_id={}, rev={}",
                    doc_id, op.rev
                ));
            };

            // Decode params (try MessagePack first, fall back to JSON for legacy data)
            let params = params_codec::decode_params_compat(&params_bytes).with_context(|| {
                format!(
                    "Invalid operation params: doc_id={}, rev={}, storage_id={:?}",
                    doc_id, op.rev, op.storage_id
                )
            })?;

            result.push(OperationInfo {
                rev: op.rev,
                user_id: op.user_id.clone(),
                mutation_id: op.mutation_id.clone(),
                params,
                op_id: op.op_id.clone(),
                created_at: op.created_at.into(),
            });
        }

        info!(
            "Retrieved operations: doc_id={}, count={}, inline={}, cache_hits={}, s3_fetches={}",
            doc_id, count, inline_count, cache_hit_count, s3_fetch_count
        );
        Ok(result)
    }

    /// Get the current version of a document
    /// Tries cache first, falls back to database with reconciliation
    ///
    /// IMPORTANT: When cache miss occurs, we must reconcile with pending ops
    /// to prevent the race condition where:
    /// 1. join_doc returns cached version (e.g., 616)
    /// 2. Version key expires
    /// 3. changeset reads stale DB version (e.g., 612)
    pub async fn get_current_version(&self, doc_id: Uuid) -> Result<Option<i64>> {
        debug!("Getting current version: doc_id={}", doc_id);

        // Try cache first
        match self.cache_service.get_version(doc_id).await {
            Ok(Some(v)) => {
                debug!("Cache hit for version: doc_id={}, version={}", doc_id, v);
                return Ok(Some(v));
            }
            Ok(None) => {
                // Cache miss (key expired) - need to reconcile with pending ops
                debug!("Cache miss for version: doc_id={}", doc_id);
                // Fall back to database with reconciliation
                return self.reconcile_version_on_cache_miss(doc_id, true).await;
            }
            Err(e) => {
                // Redis error (timeout/connection issue) - skip reconciliation
                // If get_version failed, get_max_cached_rev will also fail
                debug!("Cache error getting version: doc_id={}, error={}", doc_id, e);
                // Fall back directly to DB without reconciliation
                return self.get_current_version_from_db(doc_id).await;
            }
        }
    }

    /// Reconcile version when cache misses (version key expired)
    ///
    /// This handles the race condition where the version key expires but pending
    /// operations still exist in the cache. The real version is:
    /// max(db_version, max_cached_rev)
    ///
    /// `try_cache_ops`: if true, attempt to read pending ops from cache
    /// Set to false when Redis is known to be unavailable to avoid redundant timeouts
    async fn reconcile_version_on_cache_miss(&self, doc_id: Uuid, try_cache_ops: bool) -> Result<Option<i64>> {
        // Step 1: Get version from database
        let db_version = match self.get_current_version_from_db(doc_id).await? {
            Some(v) => v,
            None => return Ok(None), // Document doesn't exist
        };

        // Step 2: Check for pending ops in cache (ops sorted set might outlive version key)
        // Skip if Redis is known to be unavailable
        if !try_cache_ops {
            return Ok(Some(db_version));
        }

        let max_cached_rev = match self.cache_service.get_max_cached_rev(doc_id).await {
            Ok(Some(rev)) => rev,
            Ok(None) => {
                // No pending ops - DB version is authoritative
                debug!(
                    "No pending ops in cache: doc_id={}, using db_version={}",
                    doc_id, db_version
                );
                // Initialize cache with DB version (ignore error - non-critical)
                let _ = self.cache_service.set_version(doc_id, db_version).await;
                return Ok(Some(db_version));
            }
            Err(e) => {
                // Redis error checking ops - use DB version as fallback
                // Use debug level to reduce log noise during Redis issues
                debug!(
                    "Failed to check pending ops (using db_version): doc_id={}, error={}",
                    doc_id, e
                );
                return Ok(Some(db_version));
            }
        };

        // Step 3: Real version is the maximum of DB version and cached ops
        let real_version = db_version.max(max_cached_rev);

        if real_version > db_version {
            info!(
                "Version reconciliation: doc_id={}, db_version={}, max_cached_rev={}, real_version={}",
                doc_id, db_version, max_cached_rev, real_version
            );
            metrics::increment_version_reconciliations();
        }

        // Step 4: Update cache with reconciled version (ignore error - non-critical)
        if let Err(e) = self.cache_service.set_version(doc_id, real_version).await {
            debug!(
                "Failed to update cache with reconciled version: doc_id={}, error={}",
                doc_id, e
            );
        }

        Ok(Some(real_version))
    }

    /// Get the current version directly from database (internal use)
    async fn get_current_version_from_db(&self, doc_id: Uuid) -> Result<Option<i64>> {
        let version = match documents::Entity::find_by_id(doc_id)
            .select_only()
            .column(documents::Column::CurrentVersion)
            .into_tuple::<i64>()
            .one(&*self.db)
            .await
        {
            Ok(v) => v,
            Err(e) => {
                error!("Failed to query document version: doc_id={}, error={}", doc_id, e);
                return Err(e.into());
            }
        };

        match version {
            Some(v) => {
                debug!("Found current version from DB: doc_id={}, version={}", doc_id, v);
                Ok(Some(v))
            }
            None => {
                debug!("Document not found: doc_id={}", doc_id);
                Ok(None)
            }
        }
    }

    /// Update the current version of a document
    pub async fn update_current_version(&self, doc_id: Uuid, new_version: i64) -> Result<()> {
        info!("Updating current version: doc_id={}, new_version={}", doc_id, new_version);

        let mut document: documents::ActiveModel = match documents::Entity::find_by_id(doc_id)
            .one(&*self.db)
            .await
        {
            Ok(Some(doc)) => doc.into(),
            Ok(None) => {
                error!("Document not found: doc_id={}", doc_id);
                return Err(anyhow::anyhow!("Document not found: {}", doc_id));
            }
            Err(e) => {
                error!("Failed to query document: doc_id={}, error={}", doc_id, e);
                return Err(e.into());
            }
        };

        document.current_version = sea_orm::Set(new_version);
        document.updated_at = sea_orm::Set(chrono::Utc::now().into());

        match document.update(&*self.db).await {
            Ok(_) => {
                info!("Current version updated successfully: doc_id={}, new_version={}",
                    doc_id, new_version);
                Ok(())
            }
            Err(e) => {
                error!("Failed to update current version: doc_id={}, error={}", doc_id, e);
                Err(e.into())
            }
        }
    }

    /// Get operations since a specific revision (for OT transformation).
    /// Note: This returns ALL operations since the revision without limit,
    /// as OT transformation requires complete operation history for correctness.
    ///
    /// When write-behind caching is enabled, this method:
    /// 1. First reads from the Redis cache
    /// 2. Supplements from DB if cache doesn't cover the full range
    /// 3. Fetches params from Redis cache or S3 as needed
    pub async fn get_operations_since(
        &self,
        doc_id: Uuid,
        since_rev: i64,
    ) -> Result<Vec<OperationInfo>> {
        debug!("Getting operations since revision: doc_id={}, since_rev={}", doc_id, since_rev);

        // Try cache first
        match self.cache_service.get_ops_since(doc_id, since_rev).await {
            Ok(cached_ops) => {
                if cached_ops.is_empty() {
                    // Nothing in cache - use DB
                    return self
                        .get_operations(doc_id, since_rev + 1, None, None)
                        .await;
                }

                let first_cached_rev = cached_ops.first().map(|o| o.rev).unwrap_or(i64::MAX);

                if first_cached_rev > since_rev + 1 {
                    // Gap between requested and cached - need to supplement from DB
                    debug!(
                        "Cache gap detected: since_rev={}, first_cached_rev={}",
                        since_rev, first_cached_rev
                    );

                    // Get operations from DB for the gap
                    let db_ops = self
                        .get_operations(doc_id, since_rev + 1, Some(first_cached_rev - 1), None)
                        .await?;

                    // Convert cached ops and combine with DB ops
                    let mut result = db_ops;
                    let cached_converted = self.convert_cached_ops_to_operation_info(cached_ops)?;
                    result.extend(cached_converted);

                    debug!(
                        "Combined DB + cache operations: doc_id={}, total={}",
                        doc_id,
                        result.len()
                    );
                    return Ok(result);
                }

                // Cache has all the ops we need
                debug!(
                    "Cache hit for operations: doc_id={}, count={}",
                    doc_id,
                    cached_ops.len()
                );

                self.convert_cached_ops_to_operation_info(cached_ops)
            }
            Err(e) => {
                // Cache error - fall back to DB
                warn!("Cache error getting operations, falling back to DB: {}", e);
                self.get_operations(doc_id, since_rev + 1, None, None).await
            }
        }
    }

    /// Convert cached operations to OperationInfo
    /// Cached ops have params directly (they haven't been flushed to S3 yet)
    fn convert_cached_ops_to_operation_info(
        &self,
        ops: Vec<ot_common::CachedOperationInfo>,
    ) -> Result<Vec<OperationInfo>> {
        let mut result = Vec::with_capacity(ops.len());

        for op in ops {
            // Decode MessagePack params to JSON
            let params = params_codec::decode_params(&op.params)
                .with_context(|| format!("Failed to decode params for rev={}", op.rev))?;

            result.push(OperationInfo {
                rev: op.rev,
                user_id: op.user_id,
                mutation_id: op.mutation_id,
                params,
                op_id: op.op_id,
                created_at: op.created_at,
            });
        }

        Ok(result)
    }

    /// Stream operations for memory-efficient processing of large result sets.
    /// This avoids loading all operations into memory at once, which is beneficial
    /// when processing thousands of operations.
    ///
    /// # Arguments
    /// * `doc_id` - The document ID
    /// * `from_rev` - Start revision (inclusive)
    /// * `to_rev` - Optional end revision (inclusive)
    /// * `processor` - Async callback to process each operation
    ///
    /// # Example
    /// ```ignore
    /// service.stream_operations(doc_id, 1, None, |op| async {
    ///     println!("Processing op: rev={}", op.rev);
    ///     Ok(())
    /// }).await?;
    /// ```
    pub async fn stream_operations<F, Fut>(
        &self,
        doc_id: Uuid,
        from_rev: i64,
        to_rev: Option<i64>,
        mut processor: F,
    ) -> Result<()>
    where
        F: FnMut(OperationInfo) -> Fut,
        Fut: std::future::Future<Output = Result<()>>,
    {
        debug!(
            "Streaming operations: doc_id={}, from_rev={}, to_rev={:?}",
            doc_id, from_rev, to_rev
        );

        let mut query = operation_log::Entity::find()
            .filter(operation_log::Column::DocId.eq(doc_id))
            .filter(operation_log::Column::Rev.gte(from_rev))
            .order_by_asc(operation_log::Column::Rev);

        if let Some(to_rev) = to_rev {
            query = query.filter(operation_log::Column::Rev.lte(to_rev));
        }

        let mut stream = query.stream(&*self.db).await?;
        let mut count = 0;

        while let Some(op) = stream.try_next().await? {
            // Priority 1: Inline params (small operations stored directly in DB)
            let params_bytes = if let Some(ref inline_params) = op.params {
                inline_params.clone()
            } else if let Some(storage_id) = op.storage_id {
                // Priority 2: Redis cache
                match self.cache_service.get_params_cached(storage_id).await {
                    Ok(Some(bytes)) => bytes,
                    _ => {
                        // Priority 3: Fetch from S3
                        let bytes = self
                            .storage_service
                            .fetch_operation_params(storage_id)
                            .await
                            .with_context(|| {
                                format!(
                                    "Failed to fetch params during stream: doc_id={}, rev={}, storage_id={}",
                                    doc_id, op.rev, storage_id
                                )
                            })?;

                        // Cache in Redis for future reads
                        if let Err(e) = self.cache_service.cache_params(storage_id, &bytes).await {
                            warn!("Failed to cache params after S3 fetch: {}", e);
                        }

                        bytes
                    }
                }
            } else {
                // Neither inline params nor storage_id - data integrity issue
                return Err(anyhow::anyhow!(
                    "Operation has neither inline params nor storage_id: doc_id={}, rev={}",
                    doc_id, op.rev
                ));
            };

            let params = params_codec::decode_params_compat(&params_bytes).with_context(|| {
                format!(
                    "Invalid operation params during stream: doc_id={}, rev={}, storage_id={:?}",
                    doc_id, op.rev, op.storage_id
                )
            })?;

            processor(OperationInfo {
                rev: op.rev,
                user_id: op.user_id,
                mutation_id: op.mutation_id,
                params,
                op_id: op.op_id,
                created_at: op.created_at.into(),
            })
            .await?;

            count += 1;
        }

        debug!("Streamed {} operations for doc_id={}", count, doc_id);
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct OperationInfo {
    pub rev: i64,
    pub user_id: String,
    pub mutation_id: String,
    pub params: JsonValue,
    pub op_id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct DocumentSnapshotInfo {
    pub id: Uuid,
    pub doc_id: Uuid,
    pub storage_id: Uuid,
    pub name: Option<String>,
    pub size: Option<i64>,
    pub users: HashMap<String, i32>,
    pub version: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub restore_from_id: Option<Uuid>,
    pub restore_from: Option<Box<DocumentSnapshotInfo>>,
}

impl DocumentSnapshotInfo {
    fn from_model(model: document_snapshot::Model, restore_from: Option<DocumentSnapshotInfo>) -> Self {
        let users = model
            .users
            .and_then(|value| serde_json::from_value(value.into()).ok())
            .unwrap_or_default();
        Self {
            id: model.id,
            doc_id: model.doc_id,
            storage_id: model.storage_id,
            name: model.name,
            size: model.size,
            users,
            version: model.version,
            created_at: model.created_at.into(),
            updated_at: model.updated_at.into(),
            restore_from_id: model.restore_from_id,
            restore_from: restore_from.map(Box::new),
        }
    }
}
