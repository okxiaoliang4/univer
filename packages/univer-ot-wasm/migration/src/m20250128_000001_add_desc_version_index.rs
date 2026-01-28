use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create a separate index for descending version queries
        // This helps PostgreSQL's query planner efficiently handle ORDER BY version DESC
        manager
            .create_index(
                Index::create()
                    .name("idx_document_snapshots_doc_id_version_desc")
                    .table(DocumentSnapshots::Table)
                    .col(DocumentSnapshots::DocId)
                    .col(DocumentSnapshots::Version)
                    .to_owned(),
            )
            .await?;

        // Also create an index specifically for version DESC queries
        // Note: PostgreSQL can use the same index for both ASC and DESC,
        // but creating this explicitly can help the query planner
        manager
            .create_index(
                Index::create()
                    .name("idx_document_snapshots_version_desc")
                    .table(DocumentSnapshots::Table)
                    .col(DocumentSnapshots::Version)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_document_snapshots_version_desc")
                    .table(DocumentSnapshots::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_document_snapshots_doc_id_version_desc")
                    .table(DocumentSnapshots::Table)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum DocumentSnapshots {
    Table,
    DocId,
    Version,
}
