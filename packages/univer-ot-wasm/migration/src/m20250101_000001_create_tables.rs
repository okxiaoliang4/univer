use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create document_snapshots table
        manager
            .create_table(
                Table::create()
                    .table(DocumentSnapshots::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(DocumentSnapshots::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(DocumentSnapshots::Content)
                            .json_binary()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(DocumentSnapshots::Version)
                            .big_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(DocumentSnapshots::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(DocumentSnapshots::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // Create operation_logs table
        manager
            .create_table(
                Table::create()
                    .table(OperationLogs::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(OperationLogs::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(OperationLogs::DocId).uuid().not_null())
                    .col(ColumnDef::new(OperationLogs::Rev).big_integer().not_null())
                    .col(ColumnDef::new(OperationLogs::UserId).string().not_null())
                    .col(
                        ColumnDef::new(OperationLogs::MutationId)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(OperationLogs::Params)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(OperationLogs::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_operation_logs_doc_id")
                            .from(OperationLogs::Table, OperationLogs::DocId)
                            .to(DocumentSnapshots::Table, DocumentSnapshots::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create indexes
        manager
            .create_index(
                Index::create()
                    .name("idx_operation_logs_doc_id_rev")
                    .table(OperationLogs::Table)
                    .col(OperationLogs::DocId)
                    .col(OperationLogs::Rev)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_operation_logs_doc_id")
                    .table(OperationLogs::Table)
                    .col(OperationLogs::DocId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(OperationLogs::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(DocumentSnapshots::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum DocumentSnapshots {
    Table,
    Id,
    Content,
    Version,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum OperationLogs {
    Table,
    Id,
    DocId,
    Rev,
    UserId,
    MutationId,
    Params,
    CreatedAt,
}
