use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
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
                    .add_column(
                        ColumnDef::new(OperationLogs::OpId)
                            .string()
                            .not_null()
                            .default(""),
                    )
                    .to_owned(),
            )
            .await?;

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

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_operation_logs_doc_client_op")
                    .table(OperationLogs::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(OperationLogs::Table)
                    .drop_column(OperationLogs::OpId)
                    .drop_column(OperationLogs::ClientId)
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
