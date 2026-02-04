//! Migration: Drop redundant indexes
//!
//! This migration removes indexes that are either:
//! 1. Redundant (left-prefix of another composite index)
//! 2. Duplicate (same columns as another index)
//! 3. Unused (no queries filter by these columns)
//!
//! Indexes dropped:
//! - idx_operation_logs_doc_id: redundant, left-prefix of idx_operation_logs_doc_id_rev
//! - idx_operation_logs_storage_id: no SELECT queries filter by storage_id
//! - idx_document_snapshots_doc_id_version_desc: duplicate of idx_document_snapshots_doc_id_version
//! - idx_document_snapshots_version_desc: no queries filter by version alone
//! - idx_documents_id: redundant with primary key
//! - idx_storages_hash: no queries filter by hash

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop redundant operation_logs indexes
        manager
            .drop_index(
                Index::drop()
                    .name("idx_operation_logs_doc_id")
                    .table(OperationLogs::Table)
                    .if_exists()
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_operation_logs_storage_id")
                    .table(OperationLogs::Table)
                    .if_exists()
                    .to_owned(),
            )
            .await?;

        // Drop redundant document_snapshots indexes
        manager
            .drop_index(
                Index::drop()
                    .name("idx_document_snapshots_doc_id_version_desc")
                    .table(DocumentSnapshots::Table)
                    .if_exists()
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_document_snapshots_version_desc")
                    .table(DocumentSnapshots::Table)
                    .if_exists()
                    .to_owned(),
            )
            .await?;

        // Drop redundant documents index (primary key already has unique index)
        manager
            .drop_index(
                Index::drop()
                    .name("idx_documents_id")
                    .table(Documents::Table)
                    .if_exists()
                    .to_owned(),
            )
            .await?;

        // Drop unused storages index
        manager
            .drop_index(
                Index::drop()
                    .name("idx_storages_hash")
                    .table(Storages::Table)
                    .if_exists()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Recreate indexes if rollback is needed

        // idx_operation_logs_doc_id
        manager
            .create_index(
                Index::create()
                    .name("idx_operation_logs_doc_id")
                    .table(OperationLogs::Table)
                    .col(OperationLogs::DocId)
                    .to_owned(),
            )
            .await?;

        // idx_operation_logs_storage_id
        manager
            .create_index(
                Index::create()
                    .name("idx_operation_logs_storage_id")
                    .table(OperationLogs::Table)
                    .col(OperationLogs::StorageId)
                    .to_owned(),
            )
            .await?;

        // idx_document_snapshots_doc_id_version_desc
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

        // idx_document_snapshots_version_desc
        manager
            .create_index(
                Index::create()
                    .name("idx_document_snapshots_version_desc")
                    .table(DocumentSnapshots::Table)
                    .col(DocumentSnapshots::Version)
                    .to_owned(),
            )
            .await?;

        // idx_documents_id
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

        // idx_storages_hash
        manager
            .create_index(
                Index::create()
                    .name("idx_storages_hash")
                    .table(Storages::Table)
                    .col(Storages::Hash)
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
    StorageId,
}

#[derive(DeriveIden)]
enum DocumentSnapshots {
    Table,
    DocId,
    Version,
}

#[derive(DeriveIden)]
enum Documents {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Storages {
    Table,
    Hash,
}
