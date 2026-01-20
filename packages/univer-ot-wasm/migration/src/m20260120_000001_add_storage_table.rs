use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Storages::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Storages::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Storages::Endpoint).string().not_null())
                    .col(ColumnDef::new(Storages::Region).string().not_null())
                    .col(ColumnDef::new(Storages::Bucket).string().not_null())
                    .col(ColumnDef::new(Storages::Path).string().not_null())
                    .col(ColumnDef::new(Storages::Size).big_integer().not_null())
                    .col(ColumnDef::new(Storages::Hash).string().not_null())
                    .col(
                        ColumnDef::new(Storages::HashAlgorithm)
                            .string()
                            .not_null()
                            .default("md5"),
                    )
                    .col(ColumnDef::new(Storages::Filename).string().not_null())
                    .col(ColumnDef::new(Storages::ContentType).string().null())
                    .col(ColumnDef::new(Storages::Metadata).json_binary().null())
                    .col(ColumnDef::new(Storages::VersionId).string().not_null())
                    .col(ColumnDef::new(Storages::Compressed).small_integer().null())
                    .col(
                        ColumnDef::new(Storages::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Storages::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(DocumentSnapshots::Table)
                    .add_column(ColumnDef::new(DocumentSnapshots::StorageId).uuid().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_document_snapshots_storage_id")
                    .from(DocumentSnapshots::Table, DocumentSnapshots::StorageId)
                    .to(Storages::Table, Storages::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(DocumentSnapshots::Table)
                    .drop_column(DocumentSnapshots::Content)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_storages_hash")
                    .table(Storages::Table)
                    .col(Storages::Hash)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_storages_version_filename")
                    .table(Storages::Table)
                    .col(Storages::VersionId)
                    .col(Storages::Filename)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_document_snapshots_doc_id_version")
                    .table(DocumentSnapshots::Table)
                    .col(DocumentSnapshots::DocId)
                    .col(DocumentSnapshots::Version)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_document_snapshots_storage_id")
                    .table(DocumentSnapshots::Table)
                    .col(DocumentSnapshots::StorageId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_document_snapshots_storage_id")
                    .table(DocumentSnapshots::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_document_snapshots_doc_id_version")
                    .table(DocumentSnapshots::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_storages_version_filename")
                    .table(Storages::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_storages_hash")
                    .table(Storages::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk_document_snapshots_storage_id")
                    .table(DocumentSnapshots::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(DocumentSnapshots::Table)
                    .add_column(ColumnDef::new(DocumentSnapshots::Content).json_binary().not_null())
                    .drop_column(DocumentSnapshots::StorageId)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(Storages::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Storages {
    Table,
    Id,
    Endpoint,
    Region,
    Bucket,
    Path,
    Size,
    Hash,
    HashAlgorithm,
    Filename,
    ContentType,
    Metadata,
    VersionId,
    Compressed,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum DocumentSnapshots {
    Table,
    Content,
    DocId,
    StorageId,
    Version,
}
