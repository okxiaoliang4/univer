use crate::server::database::entities::{document_snapshot, operation_log};
use crate::server::services::storage::StorageService;
use anyhow::{Context, Result};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde_json::Value as JsonValue;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct SnapshotService {
    db: Arc<DatabaseConnection>,
    snapshot_interval: u64,
    storage_service: StorageService,
}

impl SnapshotService {
    pub fn new(
        db: DatabaseConnection,
        snapshot_interval: u64,
        storage_service: StorageService,
    ) -> Self {
        Self {
            db: Arc::new(db),
            snapshot_interval,
            storage_service,
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
        // Get base snapshot content - find snapshot with matching doc_id and version
        let base_snapshot = document_snapshot::Entity::find()
            .filter(document_snapshot::Column::DocId.eq(doc_id))
            .filter(document_snapshot::Column::Version.eq(base_snapshot_version))
            .one(txn)
            .await?
            .context("Base snapshot not found")?;

        let content = self
            .storage_service
            .fetch_snapshot_content(base_snapshot.storage_id)
            .await?;

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
        for _op in operations {
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

        let storage_id = self
            .storage_service
            .store_snapshot_content(doc_id, version, &content)
            .await?;

        // Find existing snapshot with matching doc_id and version, or create new one
        let snapshot = document_snapshot::Entity::find()
            .filter(document_snapshot::Column::DocId.eq(doc_id))
            .filter(document_snapshot::Column::Version.eq(version))
            .one(&*self.db)
            .await?;

        if let Some(existing) = snapshot {
            // Update existing snapshot
            let mut snapshot: document_snapshot::ActiveModel = existing.into();
            snapshot.storage_id = sea_orm::Set(storage_id);
            snapshot.updated_at = sea_orm::Set(now.into());
            snapshot.update(&*self.db).await?;
        } else {
            // Create new snapshot
            let new_snapshot = document_snapshot::ActiveModel {
                id: sea_orm::Set(uuid::Uuid::new_v4()),
                doc_id: sea_orm::Set(doc_id),
                storage_id: sea_orm::Set(storage_id),
                version: sea_orm::Set(version),
                created_at: sea_orm::Set(now.into()),
                updated_at: sea_orm::Set(now.into()),
            };
            document_snapshot::Entity::insert(new_snapshot)
                .exec(&*self.db)
                .await?;
        }

        Ok(())
    }

    /// Get snapshot interval
    pub fn get_snapshot_interval(&self) -> u64 {
        self.snapshot_interval
    }

    pub fn storage_service(&self) -> &StorageService {
        &self.storage_service
    }
}
