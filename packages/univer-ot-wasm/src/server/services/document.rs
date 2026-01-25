use crate::server::database::entities::{document_snapshot, documents, operation_log};
use crate::server::services::storage::{StoredSnapshot, StorageService};
use anyhow::Result;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder,
    QuerySelect,
};
use serde_json::{json, Value as JsonValue};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct DocumentService {
    db: Arc<DatabaseConnection>,
    storage_service: StorageService,
}

impl DocumentService {
    pub fn new(db: DatabaseConnection, storage_service: StorageService) -> Self {
        Self {
            db: Arc::new(db),
            storage_service,
        }
    }

    /// Create a new document with initial content
    pub async fn create_document(
        &self,
        doc_id: Uuid,
        creator_id: String,
        name: String,
        doc_type: i16,
        create_type: i16,
        initial_content: JsonValue,
    ) -> Result<i64> {
        let now = chrono::Utc::now();
        let users = json!({
            creator_id.clone(): now.timestamp() as i32,
        });

        // Create document entry in documents table
        let document = documents::ActiveModel {
            id: sea_orm::Set(doc_id),
            name: sea_orm::Set(name.clone()),
            creator_id: sea_orm::Set(creator_id),
            doc_type: sea_orm::Set(doc_type),
            create_type: sea_orm::Set(create_type),
            current_version: sea_orm::Set(0),
            created_at: sea_orm::Set(now.into()),
            updated_at: sea_orm::Set(now.into()),
        };

        documents::Entity::insert(document).exec(&*self.db).await?;

        // Create initial snapshot
        let StoredSnapshot { storage_id, size } = self
            .storage_service
            .store_snapshot_content(doc_id, 0, &initial_content)
            .await?;
        let snapshot = document_snapshot::ActiveModel {
            id: sea_orm::Set(doc_id),     // id = doc_id for snapshot
            doc_id: sea_orm::Set(doc_id), // doc_id references documents.id
            storage_id: sea_orm::Set(storage_id),
            name: sea_orm::Set(Some(name)),
            size: sea_orm::Set(Some(size)),
            users: sea_orm::Set(Some(users.into())),
            restore_from_id: sea_orm::Set(None),
            version: sea_orm::Set(0), // Snapshot version = version at which snapshot was taken
            created_at: sea_orm::Set(now.into()),
            updated_at: sea_orm::Set(now.into()),
        };

        document_snapshot::Entity::insert(snapshot)
            .exec(&*self.db)
            .await?;

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
        let response = reqwest::get(&url).await?.bytes().await?;
        let content = serde_json::from_slice::<JsonValue>(&response)?;
        self.create_document(doc_id, creator_id, name, doc_type, create_type, content)
            .await
    }

    /// Get document snapshot by doc_id
    pub async fn get_document(&self, doc_id: Uuid) -> Result<Option<(Uuid, i64)>> {
        let snapshot = document_snapshot::Entity::find()
            .filter(document_snapshot::Column::DocId.eq(doc_id))
            .order_by_desc(document_snapshot::Column::Version)
            .one(&*self.db)
            .await?;

        Ok(snapshot.map(|s| (s.storage_id, s.version)))
    }

    pub async fn get_document_metadata(&self, doc_id: Uuid) -> Result<Option<documents::Model>> {
        Ok(documents::Entity::find_by_id(doc_id).one(&*self.db).await?)
    }

    pub async fn clone_document(
        &self,
        source_doc_id: Uuid,
        creator_id: String,
        snapshot_id: Option<Uuid>,
        doc_type: Option<i16>,
    ) -> Result<(Uuid, i64)> {
        let source_document = documents::Entity::find_by_id(source_doc_id)
            .one(&*self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Document not found: {}", source_doc_id))?;

        let source_snapshot = self
            .get_snapshot_model(source_doc_id, snapshot_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Snapshot not found for document: {}", source_doc_id))?;

        let content = self
            .storage_service
            .fetch_snapshot_content(source_snapshot.storage_id)
            .await?;

        let new_doc_id = Uuid::new_v4();
        let doc_name = format!("{} (Clone)", source_document.name);
        let doc_type = doc_type.unwrap_or(source_document.doc_type);
        self.create_document(
            new_doc_id,
            creator_id,
            doc_name,
            doc_type,
            source_document.create_type,
            content,
        )
        .await?;

        Ok((new_doc_id, source_snapshot.size.unwrap_or(0)))
    }

    pub async fn delete_document(&self, doc_id: Uuid, _is_soft: bool) -> Result<()> {
        documents::Entity::delete_by_id(doc_id).exec(&*self.db).await?;
        Ok(())
    }

    pub async fn get_doc_snapshot(
        &self,
        doc_id: Uuid,
        snapshot_id: Option<Uuid>,
    ) -> Result<Option<DocumentSnapshotInfo>> {
        let snapshot = self.get_snapshot_model(doc_id, snapshot_id).await?;
        let Some(snapshot) = snapshot else { return Ok(None) };

        let restore_from = if let Some(restore_from_id) = snapshot.restore_from_id {
            let model = document_snapshot::Entity::find_by_id(restore_from_id)
                .one(&*self.db)
                .await?;
            model.map(|model| DocumentSnapshotInfo::from_model(model, None))
        } else {
            None
        };

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
        let mut query = document_snapshot::Entity::find()
            .filter(document_snapshot::Column::DocId.eq(doc_id));

        if let Some(cursor) = cursor {
            if desc {
                query = query.filter(document_snapshot::Column::Version.lt(cursor));
            } else {
                query = query.filter(document_snapshot::Column::Version.gt(cursor));
            }
        }

        query = if desc {
            query.order_by_desc(document_snapshot::Column::Version)
        } else {
            query.order_by_asc(document_snapshot::Column::Version)
        };

        let snapshots: Vec<document_snapshot::Model> =
            query.limit(limit).all(&*self.db).await?;
        let next_cursor = snapshots.last().map(|snapshot| snapshot.version);
        let infos = snapshots
            .into_iter()
            .map(|snapshot| DocumentSnapshotInfo::from_model(snapshot, None))
            .collect();

        Ok((infos, next_cursor))
    }

    pub async fn update_snapshot_name(
        &self,
        doc_id: Uuid,
        snapshot_id: Uuid,
        name: String,
    ) -> Result<()> {
        let mut snapshot: document_snapshot::ActiveModel = document_snapshot::Entity::find_by_id(snapshot_id)
            .filter(document_snapshot::Column::DocId.eq(doc_id))
            .one(&*self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Snapshot not found: {}", snapshot_id))?
            .into();
        snapshot.name = sea_orm::Set(Some(name));
        snapshot.updated_at = sea_orm::Set(chrono::Utc::now().into());
        snapshot.update(&*self.db).await?;
        Ok(())
    }

    pub async fn get_latest_snapshots(
        &self,
        doc_ids: Vec<Uuid>,
    ) -> Result<HashMap<Uuid, DocumentSnapshotInfo>> {
        let mut results = HashMap::new();
        for doc_id in doc_ids {
            if let Some(snapshot) = self.get_doc_snapshot(doc_id, None).await? {
                results.insert(doc_id, snapshot);
            }
        }
        Ok(results)
    }

    pub async fn sign_object_urls(
        &self,
        storage_ids: Vec<Uuid>,
    ) -> Result<HashMap<Uuid, String>> {
        let mut urls = HashMap::new();
        for storage_id in storage_ids {
            let storage = self.storage_service.get_storage_location(storage_id).await?;
            let signed_url = self.storage_service.get_signed_url(&storage).await?;
            urls.insert(storage_id, signed_url);
        }
        Ok(urls)
    }

    pub async fn get_doc_id_by_storage_id(&self, storage_id: Uuid) -> Result<Option<Uuid>> {
        let snapshot = document_snapshot::Entity::find()
            .filter(document_snapshot::Column::StorageId.eq(storage_id))
            .one(&*self.db)
            .await?;
        Ok(snapshot.map(|model| model.doc_id))
    }

    pub async fn restore_document(
        &self,
        doc_id: Uuid,
        snapshot_id: Uuid,
    ) -> Result<DocumentSnapshotInfo> {
        let document = documents::Entity::find_by_id(doc_id)
            .one(&*self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Document not found: {}", doc_id))?;

        let target_snapshot = document_snapshot::Entity::find_by_id(snapshot_id)
            .filter(document_snapshot::Column::DocId.eq(doc_id))
            .one(&*self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Snapshot not found: {}", snapshot_id))?;

        let new_version = document.current_version + 1;
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
        let restored_model = restored_snapshot.insert(&*self.db).await?;

        let mut document: documents::ActiveModel = document.into();
        document.current_version = sea_orm::Set(new_version);
        document.updated_at = sea_orm::Set(now.into());
        document.update(&*self.db).await?;

        Ok(DocumentSnapshotInfo::from_model(restored_model, None))
    }

    async fn get_snapshot_model(
        &self,
        doc_id: Uuid,
        snapshot_id: Option<Uuid>,
    ) -> Result<Option<document_snapshot::Model>> {
        if let Some(snapshot_id) = snapshot_id {
            Ok(document_snapshot::Entity::find_by_id(snapshot_id)
                .one(&*self.db)
                .await?)
        } else {
            Ok(document_snapshot::Entity::find()
                .filter(document_snapshot::Column::DocId.eq(doc_id))
                .order_by_desc(document_snapshot::Column::Version)
                .one(&*self.db)
                .await?)
        }
    }

    /// Get operations for a document within a revision range
    pub async fn get_operations(
        &self,
        doc_id: Uuid,
        from_rev: i64,
        to_rev: Option<i64>,
    ) -> Result<Vec<OperationInfo>> {
        let mut query = operation_log::Entity::find()
            .filter(operation_log::Column::DocId.eq(doc_id))
            .filter(operation_log::Column::Rev.gte(from_rev))
            .order_by_asc(operation_log::Column::Rev);

        if let Some(to_rev) = to_rev {
            query = query.filter(operation_log::Column::Rev.lte(to_rev));
        }

        let operations = query.all(&*self.db).await?;

        Ok(operations
            .into_iter()
            .map(|op| OperationInfo {
                rev: op.rev,
                user_id: op.user_id,
                mutation_id: op.mutation_id,
                params: op.params,
                client_id: op.client_id,
                op_id: op.op_id,
                created_at: op.created_at.into(),
            })
            .collect())
    }

    /// Get the current version of a document from documents table
    pub async fn get_current_version(&self, doc_id: Uuid) -> Result<Option<i64>> {
        let document = documents::Entity::find_by_id(doc_id).one(&*self.db).await?;

        Ok(document.map(|d| d.current_version))
    }

    /// Update the current version of a document
    pub async fn update_current_version(&self, doc_id: Uuid, new_version: i64) -> Result<()> {
        let mut document: documents::ActiveModel = documents::Entity::find_by_id(doc_id)
            .one(&*self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Document not found: {}", doc_id))?
            .into();

        document.current_version = sea_orm::Set(new_version);
        document.updated_at = sea_orm::Set(chrono::Utc::now().into());

        document.update(&*self.db).await?;

        Ok(())
    }

    /// Get operations since a specific revision (for OT transformation)
    pub async fn get_operations_since(
        &self,
        doc_id: Uuid,
        since_rev: i64,
    ) -> Result<Vec<OperationInfo>> {
        self.get_operations(doc_id, since_rev + 1, None).await
    }
}

#[derive(Debug, Clone)]
pub struct OperationInfo {
    pub rev: i64,
    pub user_id: String,
    pub mutation_id: String,
    pub params: JsonValue,
    pub client_id: String,
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
