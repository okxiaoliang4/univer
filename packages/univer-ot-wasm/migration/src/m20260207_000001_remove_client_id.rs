//! Migration: Remove client_id from operation_logs
//!
//! This migration removes the `client_id` field from operation_logs table.
//! The idempotency check now uses only `op_id` (which should be unique per document).
//!
//! Changes:
//! - Drop index `idx_operation_logs_doc_client_op` (doc_id, client_id, op_id)
//! - Create index `idx_operation_logs_doc_op` (doc_id, op_id) UNIQUE
//! - Drop column `client_id`

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Step 1: Drop existing unique index (doc_id, client_id, op_id)
        manager
            .drop_index(
                Index::drop()
                    .name("idx_operation_logs_doc_client_op")
                    .table(OperationLogs::Table)
                    .to_owned(),
            )
            .await?;

        // Step 2: Create new unique index (doc_id, op_id)
        manager
            .create_index(
                Index::create()
                    .name("idx_operation_logs_doc_op")
                    .table(OperationLogs::Table)
                    .col(OperationLogs::DocId)
                    .col(OperationLogs::OpId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Step 3: Drop client_id column
        manager
            .alter_table(
                Table::alter()
                    .table(OperationLogs::Table)
                    .drop_column(OperationLogs::ClientId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Step 1: Add client_id column back
        manager
            .alter_table(
                Table::alter()
                    .table(OperationLogs::Table)
                    .add_column(
                        ColumnDef::new(OperationLogs::ClientId)
                            .string()
                            .not_null()
                            .default(""),
                    )
                    .to_owned(),
            )
            .await?;

        // Step 2: Drop new index
        manager
            .drop_index(
                Index::drop()
                    .name("idx_operation_logs_doc_op")
                    .table(OperationLogs::Table)
                    .to_owned(),
            )
            .await?;

        // Step 3: Recreate original index
        manager
            .create_index(
                Index::create()
                    .name("idx_operation_logs_doc_client_op")
                    .table(OperationLogs::Table)
                    .col(OperationLogs::DocId)
                    .col(OperationLogs::ClientId)
                    .col(OperationLogs::OpId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum OperationLogs {
    Table,
    DocId,
    ClientId,
    OpId,
}
