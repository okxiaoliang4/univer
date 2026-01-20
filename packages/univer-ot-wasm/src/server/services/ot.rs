use crate::server::database::entities::{document_snapshot, documents, operation_log};
use crate::server::services::document::DocumentService;
use crate::server::services::snapshot::SnapshotService;
use crate::transform::TransformServiceCore;
use crate::types::MutationInfoInternal;
use anyhow::{Context, Result};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QuerySelect, Set,
    TransactionTrait,
};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Changeset {
    #[serde(rename = "baseRev")]
    pub base_rev: i64,
    #[serde(rename = "userId")]
    pub user_id: String,
    pub mutations: Vec<MutationInfoInternal>,
    #[serde(rename = "clientId")]
    pub client_id: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChangesetApplied {
    #[serde(rename = "serverRev")]
    pub server_rev: i64,
    pub mutations: Vec<MutationInfoInternal>,
    #[serde(rename = "userId")]
    pub user_id: String,
}

#[derive(Clone)]
pub struct OTService {
    db: Arc<DatabaseConnection>,
    document_service: DocumentService,
    snapshot_service: SnapshotService,
    transform_service: Arc<TransformServiceCore>,
}

impl OTService {
    pub fn new(
        db: DatabaseConnection,
        document_service: DocumentService,
        snapshot_service: SnapshotService,
    ) -> Self {
        Self {
            db: Arc::new(db),
            document_service,
            snapshot_service,
            transform_service: Arc::new(TransformServiceCore::new()),
        }
    }

    /// Apply a changeset from a client
    /// This performs OT transformation and stores the transformed operations
    pub async fn apply_changeset(
        &self,
        doc_id: Uuid,
        changeset: Changeset,
        client_msg_id: String,
    ) -> Result<ChangesetApplied> {
        let txn = (*self.db).begin().await?;

        // Get current version from documents table (SeaORM 2.0 uses transactions for locking)
        let current_version = documents::Entity::find_by_id(doc_id)
            .lock_exclusive()
            .one(&txn)
            .await?
            .map(|d| d.current_version)
            .context("Document not found")?;

        // Validate base_rev is not greater than current version (shouldn't happen)
        if changeset.base_rev > current_version {
            return Err(anyhow::anyhow!(
                "Version mismatch: client base_rev {} is greater than server version {}",
                changeset.base_rev,
                current_version
            ));
        }

        // Check for duplicate client_msg_id (idempotency) - check once for the entire changeset
        // Use the first mutation's client_msg_id to check if the entire changeset was already processed
        let first_mutation_client_msg_id = format!("{}-0", client_msg_id);
        let existing_op = operation_log::Entity::find()
            .filter(operation_log::Column::DocId.eq(doc_id))
            .filter(operation_log::Column::ClientMsgId.eq(&first_mutation_client_msg_id))
            .one(&txn)
            .await?;

        if let Some(existing_op) = existing_op {
            // Return existing result if duplicate
            // Find all mutations for this changeset (they should be consecutive revisions)
            let existing_rev = existing_op.rev;
            let changeset_size = changeset.mutations.len() as i64;
            let existing_mutations = self
                .document_service
                .get_operations(
                    doc_id,
                    existing_rev,
                    Some(existing_rev + changeset_size - 1),
                )
                .await?;
            return Ok(ChangesetApplied {
                server_rev: existing_rev + changeset_size - 1,
                mutations: existing_mutations
                    .into_iter()
                    .map(|op| MutationInfoInternal {
                        id: op.mutation_id,
                        params: op.params,
                    })
                    .collect(),
                user_id: changeset.user_id,
            });
        }

        // Get operations that happened after base_rev (for OT transformation)
        // This will be empty if base_rev == current_version, or contain concurrent operations
        let concurrent_ops = self
            .document_service
            .get_operations_since(doc_id, changeset.base_rev)
            .await?;

        // Convert concurrent operations to MutationInfoInternal list
        let concurrent_mutations: Vec<MutationInfoInternal> = concurrent_ops
            .iter()
            .map(|op| MutationInfoInternal {
                id: op.mutation_id.clone(),
                params: op.params.clone(),
            })
            .collect();

        // Transform all mutations against concurrent operations using transform_list
        let (m1_primes, _, error) = self
            .transform_service
            .transform_list(&changeset.mutations, &concurrent_mutations);

        if let Some(err) = error {
            return Err(anyhow::anyhow!("Transform error: {}", err));
        }

        // Store all transformed operations (m1_primes) into database
        let mut transformed_mutations = Vec::new();
        let mut next_rev = current_version + 1;
        let now = chrono::Utc::now();

        for (index, m1_prime) in m1_primes.into_iter().enumerate() {
            // Generate unique client_msg_id for each mutation: {base_client_msg_id}-{index}
            let unique_client_msg_id = format!("{}-{}", client_msg_id, index);
            let operation = operation_log::ActiveModel {
                doc_id: Set(doc_id),
                rev: Set(next_rev),
                user_id: Set(changeset.user_id.clone()),
                mutation_id: Set(m1_prime.id.clone()),
                params: Set(m1_prime.params.clone()),
                client_msg_id: Set(unique_client_msg_id),
                created_at: Set(now.into()),
                ..Default::default()
            };

            operation.insert(&txn).await?;
            transformed_mutations.push(m1_prime);
            next_rev += 1;
        }

        // Update document current_version in documents table
        let new_version = next_rev - 1;
        let mut document: documents::ActiveModel = documents::Entity::find_by_id(doc_id)
            .one(&txn)
            .await?
            .context("Document not found")?
            .into();

        document.current_version = Set(new_version);
        document.updated_at = Set(chrono::Utc::now().into());
        document.update(&txn).await?;

        // Check if we should create/update snapshot checkpoint
        // Snapshot version should be the version at which snapshot was taken, not updated per operation
        let ops_count = (new_version - changeset.base_rev) as u64;
        if self.snapshot_service.should_update_snapshot(ops_count) {
            let base_snapshot = document_snapshot::Entity::find_by_id(doc_id)
                .one(&txn)
                .await?
                .context("Base snapshot not found")?;

            let base_snapshot_version = base_snapshot.version;

            // Compute new snapshot content by applying mutations since base snapshot
            let new_content = self
                .snapshot_service
                .compute_snapshot_content(doc_id, base_snapshot_version, new_version, &txn)
                .await?;

            // Update snapshot with new content and checkpoint version
            let storage_id = self
                .snapshot_service
                .storage_service()
                .store_snapshot_content(doc_id, new_version, &new_content)
                .await?;
            let mut snapshot: document_snapshot::ActiveModel = base_snapshot.into();
            snapshot.storage_id = Set(storage_id);
            snapshot.version = Set(new_version); // Snapshot checkpoint version
            snapshot.updated_at = Set(chrono::Utc::now().into());
            snapshot.update(&txn).await?;
        }

        txn.commit().await?;

        Ok(ChangesetApplied {
            server_rev: new_version,
            mutations: transformed_mutations,
            user_id: changeset.user_id,
        })
    }

    /// Transform two mutations using OT algorithms
    fn transform_mutations(
        &self,
        m1: &MutationInfoInternal,
        m2: &MutationInfoInternal,
    ) -> Result<crate::types::TransformResultInternal> {
        // Use the unified transform service (shared via Arc)
        Ok(self.transform_service.transform(m1, m2))
    }
}
