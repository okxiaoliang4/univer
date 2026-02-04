use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Step 1: Add storage_id column (nullable initially)
        manager
            .alter_table(
                Table::alter()
                    .table(OperationLogs::Table)
                    .add_column(ColumnDef::new(OperationLogs::StorageId).uuid())
                    .to_owned(),
            )
            .await?;

        // Step 2: Drop the params column (clean break - user confirmed no backward compatibility needed)
        manager
            .alter_table(
                Table::alter()
                    .table(OperationLogs::Table)
                    .drop_column(OperationLogs::Params)
                    .to_owned(),
            )
            .await?;

        // Step 3: Make storage_id NOT NULL after dropping params
        // Note: This will fail if there are existing rows without storage_id
        // For a clean break, the table should be empty or data migrated separately
        manager
            .alter_table(
                Table::alter()
                    .table(OperationLogs::Table)
                    .modify_column(ColumnDef::new(OperationLogs::StorageId).uuid().not_null())
                    .to_owned(),
            )
            .await?;

        // Step 4: Add index on storage_id for efficient lookups
        manager
            .create_index(
                Index::create()
                    .name("idx_operation_logs_storage_id")
                    .table(OperationLogs::Table)
                    .col(OperationLogs::StorageId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop index first
        manager
            .drop_index(
                Index::drop()
                    .name("idx_operation_logs_storage_id")
                    .table(OperationLogs::Table)
                    .to_owned(),
            )
            .await?;

        // Add params column back
        manager
            .alter_table(
                Table::alter()
                    .table(OperationLogs::Table)
                    .add_column(
                        ColumnDef::new(OperationLogs::Params)
                            .binary()
                            .not_null()
                            .default(""),
                    )
                    .to_owned(),
            )
            .await?;

        // Drop storage_id column
        manager
            .alter_table(
                Table::alter()
                    .table(OperationLogs::Table)
                    .drop_column(OperationLogs::StorageId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum OperationLogs {
    Table,
    StorageId,
    Params,
}
