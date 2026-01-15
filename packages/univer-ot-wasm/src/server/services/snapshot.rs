use crate::server::database::entities::{document_snapshot, operation_log};
use crate::server::services::document::OperationInfo;
use anyhow::{Context, Result};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, TransactionTrait};
use serde_json::Value as JsonValue;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct SnapshotService {
    db: Arc<DatabaseConnection>,
    snapshot_interval: u64,
}

impl SnapshotService {
    pub fn new(db: DatabaseConnection, snapshot_interval: u64) -> Self {
        Self {
            db: Arc::new(db),
            snapshot_interval,
        }
    }

    /// Check if snapshot should be updated based on operation count
    pub fn should_update_snapshot(&self, current_ops_count: u64) -> bool {
        current_ops_count % self.snapshot_interval == 0
    }

    /// Compute snapshot content by applying mutations since base snapshot version
    /// This method applies all mutations from base_snapshot_version to target_version to compute new content
    pub async fn compute_snapshot_content(
        &self,
        doc_id: Uuid,
        base_snapshot_version: i64,
        target_version: i64,
        txn: &sea_orm::DatabaseTransaction,
    ) -> Result<JsonValue> {
        // Get base snapshot content
        let base_snapshot = document_snapshot::Entity::find_by_id(doc_id)
            .one(txn)
            .await?
            .context("Base snapshot not found")?;

        let mut content = base_snapshot.content.clone();

        // Get all operations between base_snapshot_version and target_version
        let operations = operation_log::Entity::find()
            .filter(operation_log::Column::DocId.eq(doc_id))
            .filter(operation_log::Column::Rev.gt(base_snapshot_version))
            .filter(operation_log::Column::Rev.lte(target_version))
            .all(txn)
            .await?;

        // Apply each mutation to the content
        // Note: This is a simplified implementation. In a real system, you would need
        // a full document model that can apply mutations properly (e.g., for sheets,
        // this would require applying set-range-values, insert-row, etc. mutations)
        for op in operations {
            // TODO: Implement proper mutation application logic
            // For now, we'll just update the content structure to indicate mutations were applied
            // In a production system, you would:
            // 1. Parse the mutation_id and params
            // 2. Apply the mutation to the content using the appropriate transformation
            // 3. Handle different mutation types (set-range-values, insert-row, etc.)
            
            // Placeholder: Mark that mutations need to be applied
            // The actual implementation would require a full document model in Rust
            // or calling out to a JavaScript/WASM module that has the document model
        }

        Ok(content)
    }

    /// Update document snapshot with computed content
    pub async fn update_snapshot(
        &self,
        doc_id: Uuid,
        content: JsonValue,
        version: i64,
    ) -> Result<()> {
        let now = chrono::Utc::now();

        let mut snapshot: document_snapshot::ActiveModel = document_snapshot::Entity::find_by_id(doc_id)
            .one(&*self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Document not found: {}", doc_id))?
            .into();

        snapshot.content = sea_orm::Set(JsonValue::from(content));
        snapshot.version = sea_orm::Set(version); // Snapshot checkpoint version
        snapshot.updated_at = sea_orm::Set(now.into());

        snapshot.update(&*self.db).await?;

        Ok(())
    }

    /// Get snapshot interval
    pub fn get_snapshot_interval(&self) -> u64 {
        self.snapshot_interval
    }
}
