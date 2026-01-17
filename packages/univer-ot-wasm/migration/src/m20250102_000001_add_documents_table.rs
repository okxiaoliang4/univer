use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create documents table to track document metadata and current version
        manager
            .create_table(
                Table::create()
                    .table(Documents::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Documents::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Documents::Name).string().not_null())
                    .col(
                        ColumnDef::new(Documents::CurrentVersion)
                            .big_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Documents::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Documents::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // Add doc_id foreign key to document_snapshots (if not already exists)
        // Note: document_snapshots.id currently references doc_id directly
        // We'll add a doc_id column to link to documents table
        manager
            .alter_table(
                Table::alter()
                    .table(DocumentSnapshots::Table)
                    .add_column(ColumnDef::new(DocumentSnapshots::DocId).uuid().not_null())
                    .to_owned(),
            )
            .await?;

        // Add foreign key from document_snapshots.doc_id to documents.id
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_document_snapshots_doc_id")
                    .from(DocumentSnapshots::Table, DocumentSnapshots::DocId)
                    .to(Documents::Table, Documents::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        // Update operation_logs foreign key to reference documents instead of document_snapshots
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .table(OperationLogs::Table)
                    .name("fk_operation_logs_doc_id")
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_operation_logs_doc_id")
                    .from(OperationLogs::Table, OperationLogs::DocId)
                    .to(Documents::Table, Documents::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        // Create index on documents.current_version for faster lookups
        manager
            .create_index(
                Index::create()
                    .name("idx_documents_id")
                    .table(Documents::Table)
                    .col(Documents::Id)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop foreign keys first
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .table(OperationLogs::Table)
                    .name("fk_operation_logs_doc_id")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .table(DocumentSnapshots::Table)
                    .name("fk_document_snapshots_doc_id")
                    .to_owned(),
            )
            .await?;

        // Drop doc_id column from document_snapshots
        manager
            .alter_table(
                Table::alter()
                    .table(DocumentSnapshots::Table)
                    .drop_column(DocumentSnapshots::DocId)
                    .to_owned(),
            )
            .await?;

        // Restore original foreign key
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_operation_logs_doc_id")
                    .from(OperationLogs::Table, OperationLogs::DocId)
                    .to(DocumentSnapshots::Table, DocumentSnapshots::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        // Drop documents table
        manager
            .drop_table(Table::drop().table(Documents::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Documents {
    Table,
    Id,
    Name,
    CurrentVersion,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum DocumentSnapshots {
    Table,
    Id,
    DocId,
}

#[derive(DeriveIden)]
enum OperationLogs {
    Table,
    DocId,
}
