use crate::server::database::entities::{document_snapshot, documents, operation_log};
use anyhow::Result;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder,
    QuerySelect,
};
use serde_json::Value as JsonValue;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct DocumentService {
    db: Arc<DatabaseConnection>,
}

impl DocumentService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db: Arc::new(db) }
    }

    /// Create a new document with initial content
    pub async fn create_document(
        &self,
        doc_id: Uuid,
        initial_content: JsonValue,
        name: Option<String>,
    ) -> Result<i64> {
        let now = chrono::Utc::now();

        // Create document entry in documents table
        let document = documents::ActiveModel {
            id: sea_orm::Set(doc_id),
            name: sea_orm::Set(name.unwrap_or_else(|| "Untitled Document".to_string())),
            current_version: sea_orm::Set(0),
            created_at: sea_orm::Set(now.into()),
            updated_at: sea_orm::Set(now.into()),
        };

        documents::Entity::insert(document).exec(&*self.db).await?;

        // Create initial snapshot
        let snapshot = document_snapshot::ActiveModel {
            id: sea_orm::Set(doc_id),     // id = doc_id for snapshot
            doc_id: sea_orm::Set(doc_id), // doc_id references documents.id
            content: sea_orm::Set(JsonValue::from(initial_content)),
            version: sea_orm::Set(0), // Snapshot version = version at which snapshot was taken
            created_at: sea_orm::Set(now.into()),
            updated_at: sea_orm::Set(now.into()),
        };

        document_snapshot::Entity::insert(snapshot)
            .exec(&*self.db)
            .await?;

        Ok(0)
    }

    /// Get document snapshot by doc_id
    pub async fn get_document(&self, doc_id: Uuid) -> Result<Option<(JsonValue, i64)>> {
        let snapshot = document_snapshot::Entity::find_by_id(doc_id)
            .one(&*self.db)
            .await?;

        Ok(snapshot.map(|s| (s.content, s.version)))
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
                client_msg_id: op.client_msg_id,
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
    pub client_msg_id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
