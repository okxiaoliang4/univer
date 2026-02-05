//! Migration: Add inline params support to operation_logs
//!
//! This migration enables storing small operation params directly in the database
//! instead of always going through S3. This optimization:
//! - Reduces S3 API calls for small operations (< 2KB)
//! - Improves read latency for small operations
//! - Aligns with PostgreSQL's TOAST threshold for optimal storage
//!
//! Schema changes:
//! - Add `params` BYTEA column (nullable) for inline storage
//! - Make `storage_id` nullable (was NOT NULL)
//! - Add CHECK constraint: at least one of params or storage_id must be non-null

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Step 1: Add params column (nullable) for inline storage of small operations
        manager
            .alter_table(
                Table::alter()
                    .table(OperationLogs::Table)
                    .add_column(ColumnDef::new(OperationLogs::Params).binary())
                    .to_owned(),
            )
            .await?;

        // Step 2: Make storage_id nullable (it was NOT NULL before)
        // This allows operations to be stored inline without an S3 reference
        manager
            .alter_table(
                Table::alter()
                    .table(OperationLogs::Table)
                    .modify_column(ColumnDef::new(OperationLogs::StorageId).uuid().null())
                    .to_owned(),
            )
            .await?;

        // Step 3: Add CHECK constraint to ensure data integrity
        // At least one of params or storage_id must be non-null
        // Note: PostgreSQL-specific syntax
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                ALTER TABLE operation_logs
                ADD CONSTRAINT chk_params_or_storage
                CHECK (params IS NOT NULL OR storage_id IS NOT NULL)
                "#,
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Step 1: Drop the CHECK constraint
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                ALTER TABLE operation_logs
                DROP CONSTRAINT IF EXISTS chk_params_or_storage
                "#,
            )
            .await?;

        // Step 2: Update any rows that have inline params to have a placeholder storage_id
        // This is needed before making storage_id NOT NULL again
        // Note: In a real migration, you'd want to upload params to S3 first
        // For rollback simplicity, we just delete rows without storage_id
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                DELETE FROM operation_logs WHERE storage_id IS NULL
                "#,
            )
            .await?;

        // Step 3: Make storage_id NOT NULL again
        manager
            .alter_table(
                Table::alter()
                    .table(OperationLogs::Table)
                    .modify_column(ColumnDef::new(OperationLogs::StorageId).uuid().not_null())
                    .to_owned(),
            )
            .await?;

        // Step 4: Drop the params column
        manager
            .alter_table(
                Table::alter()
                    .table(OperationLogs::Table)
                    .drop_column(OperationLogs::Params)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum OperationLogs {
    Table,
    Params,
    StorageId,
}
