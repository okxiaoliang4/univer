use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Documents::Table)
                    .add_column(
                        ColumnDef::new(Documents::CreatorId)
                            .string()
                            .not_null()
                            .default(""),
                    )
                    .add_column(
                        ColumnDef::new(Documents::DocType)
                            .small_integer()
                            .not_null()
                            .default(0),
                    )
                    .add_column(
                        ColumnDef::new(Documents::CreateType)
                            .small_integer()
                            .not_null()
                            .default(0),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(DocumentSnapshots::Table)
                    .add_column(ColumnDef::new(DocumentSnapshots::Name).string().null())
                    .add_column(ColumnDef::new(DocumentSnapshots::Size).big_integer().null())
                    .add_column(ColumnDef::new(DocumentSnapshots::Users).json_binary().null())
                    .add_column(ColumnDef::new(DocumentSnapshots::RestoreFromId).uuid().null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_document_snapshots_restore_from_id")
                    .from(DocumentSnapshots::Table, DocumentSnapshots::RestoreFromId)
                    .to(DocumentSnapshots::Table, DocumentSnapshots::Id)
                    .on_delete(ForeignKeyAction::SetNull)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_document_snapshots_restore_from_id")
                    .table(DocumentSnapshots::Table)
                    .col(DocumentSnapshots::RestoreFromId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_document_snapshots_restore_from_id")
                    .table(DocumentSnapshots::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk_document_snapshots_restore_from_id")
                    .table(DocumentSnapshots::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(DocumentSnapshots::Table)
                    .drop_column(DocumentSnapshots::RestoreFromId)
                    .drop_column(DocumentSnapshots::Users)
                    .drop_column(DocumentSnapshots::Size)
                    .drop_column(DocumentSnapshots::Name)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Documents::Table)
                    .drop_column(Documents::CreateType)
                    .drop_column(Documents::DocType)
                    .drop_column(Documents::CreatorId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Documents {
    Table,
    CreatorId,
    DocType,
    CreateType,
}

#[derive(DeriveIden)]
enum DocumentSnapshots {
    Table,
    Id,
    Name,
    Size,
    Users,
    RestoreFromId,
}
