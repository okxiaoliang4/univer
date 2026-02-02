use crate::database::entities::{documents, operation_log};
use crate::metrics;
use crate::services::document::DocumentService;
use crate::services::op_queue::OpQueueService;
use anyhow::{Context, Result};
use ot_core::{MutationInfo, MutationInfoWithOpId, TransformService};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QuerySelect, Set,
    TransactionTrait,
};
use std::sync::Arc;
use std::time::Instant;
use uuid::Uuid;

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

#[derive(Clone)]
pub struct OTService {
    db: Arc<DatabaseConnection>,
    document_service: DocumentService,
    transform_service: Arc<TransformService>,
    op_queue_service: OpQueueService,
}

impl OTService {
    pub fn new(
        db: DatabaseConnection,
        document_service: DocumentService,
        op_queue_service: OpQueueService,
    ) -> Self {
        Self {
            db: Arc::new(db),
            document_service,
            transform_service: Arc::new(TransformService::new()),
            op_queue_service,
        }
    }

    /// Apply a changeset from a client
    /// This performs OT transformation and stores the transformed operations
    pub async fn apply_changeset(
        &self,
        doc_id: Uuid,
        changeset: Changeset,
    ) -> Result<ChangesetApplied> {
        // Start timing for end-to-end latency
        let e2e_start = Instant::now();

        // Increment total operations counter
        metrics::increment_operations_total();

        let txn = (*self.db).begin().await?;

        // Get current version from documents table (SeaORM 2.0 uses transactions for locking)
        let current_version = documents::Entity::find_by_id(doc_id)
            .lock_exclusive()
            .one(&txn)
            .await?
            .map(|d| d.current_version)
            .context("Document not found")?;

        // Deduplicate by (doc_id, client_id, op_id) per mutation (idempotent behavior)
        if let Some(first_mutation) = changeset.mutations.first() {
            let existing_op = operation_log::Entity::find()
                .filter(operation_log::Column::DocId.eq(doc_id))
                .filter(operation_log::Column::ClientId.eq(&changeset.client_id))
                .filter(operation_log::Column::OpId.eq(&first_mutation.op_id))
                .one(&txn)
                .await?;
            if let Some(existing_op) = existing_op {
                let existing_rev = existing_op.rev;
                let changeset_size = changeset.mutations.len() as i64;
                let existing_mutations = self
                    .document_service
                    .get_operations(
                        doc_id,
                        existing_rev,
                        Some(existing_rev + changeset_size - 1),
                        None, // No limit needed as range is bounded by changeset_size
                    )
                    .await?;
                let ack_op_ids = changeset
                    .mutations
                    .iter()
                    .map(|mutation| mutation.op_id.clone())
                    .collect();
                return Ok(ChangesetApplied {
                    server_rev: existing_rev + changeset_size - 1,
                    mutations: existing_mutations
                        .into_iter()
                        .map(|op| MutationInfo {
                            id: op.mutation_id,
                            params: op.params,
                        })
                        .collect(),
                    op_ids: ack_op_ids,
                    user_id: changeset.user_id,
                });
            }
        }

        // Validate base_rev is not greater than current version (shouldn't happen)
        if changeset.base_rev > current_version {
            // Record failed operation
            metrics::increment_operations_failed();
            metrics::record_operation_e2e_latency(e2e_start.elapsed().as_secs_f64());
            return Err(anyhow::anyhow!(
                "Version mismatch: client base_rev {} is greater than server version {}",
                changeset.base_rev,
                current_version
            ));
        }

        // Record version drift (difference between client base_rev and server current_version)
        let version_drift = (current_version - changeset.base_rev) as f64;
        metrics::record_version_drift(version_drift);

        // Get operations that happened after base_rev (for OT transformation)
        // This will be empty if base_rev == current_version, or contain concurrent operations
        let concurrent_ops = self
            .document_service
            .get_operations_since(doc_id, changeset.base_rev)
            .await?;

        // Convert concurrent operations to MutationInfo list
        let concurrent_mutations: Vec<MutationInfo> = concurrent_ops
            .iter()
            .map(|op| MutationInfo {
                id: op.mutation_id.clone(),
                params: op.params.clone(),
            })
            .collect();

        // Record conflict if there are concurrent operations
        let has_conflict = !concurrent_mutations.is_empty();
        if has_conflict {
            metrics::increment_conflicts_total();
        }

        // Transform all mutations against concurrent operations using transform_list
        let m1_internal: Vec<MutationInfo> = changeset
            .mutations
            .iter()
            .map(|mutation| MutationInfo {
                id: mutation.id.clone(),
                params: mutation.params.clone(),
            })
            .collect();

        // Start timing for transform latency
        let transform_start = Instant::now();

        let (m1_primes, _, error) = self
            .transform_service
            .transform_list(&m1_internal, &concurrent_mutations);

        // Record transform latency
        metrics::record_transform_latency(transform_start.elapsed().as_secs_f64());

        // Record transform complexity (number of concurrent mutations to transform against)
        if !concurrent_mutations.is_empty() {
            metrics::record_transform_complexity(concurrent_mutations.len() as f64);
        }

        if let Some(err) = error {
            // Record failed operation
            metrics::increment_operations_failed();
            metrics::record_operation_e2e_latency(e2e_start.elapsed().as_secs_f64());
            return Err(anyhow::anyhow!("Transform error: {}", err));
        }

        // Record conflict resolved if there was a conflict
        if has_conflict {
            metrics::increment_conflicts_resolved();
        }

        // Store all transformed operations (m1_primes) into database
        let mut transformed_mutations = Vec::new();
        let mut applied_op_ids = Vec::new();
        let mut next_rev = current_version + 1;
        let now = chrono::Utc::now();

        for (index, m1_prime) in m1_primes.into_iter().enumerate() {
            let op_id = changeset
                .mutations
                .get(index)
                .map(|mutation| mutation.op_id.clone())
                .unwrap_or_default();
            let params = serde_json::to_string(&m1_prime.params)
                .context("Failed to serialize mutation params for operation_log")?;
            let operation = operation_log::ActiveModel {
                doc_id: Set(doc_id),
                rev: Set(next_rev),
                user_id: Set(changeset.user_id.clone()),
                mutation_id: Set(m1_prime.id.clone()),
                params: Set(params),
                client_id: Set(changeset.client_id.clone()),
                op_id: Set(op_id.clone()),
                created_at: Set(now.into()),
                ..Default::default()
            };

            operation.insert(&txn).await?;
            let _ = self.op_queue_service.enqueue_doc(&doc_id.to_string()).await;
            transformed_mutations.push(m1_prime);
            applied_op_ids.push(op_id);
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

        // Enqueue snapshot job asynchronously
        // Do not enqueue snapshot job here; only enqueue doc_id to Redis queue

        txn.commit().await?;

        // Record successful operation
        metrics::increment_operations_success();

        // Record end-to-end latency
        metrics::record_operation_e2e_latency(e2e_start.elapsed().as_secs_f64());

        Ok(ChangesetApplied {
            server_rev: new_version,
            mutations: transformed_mutations,
            op_ids: applied_op_ids,
            user_id: changeset.user_id,
        })
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
