//! Migration: Convert operation_logs.params from TEXT to BYTEA
//!
//! This migration changes the params column from TEXT (JSON string) to BYTEA
//! (binary data) to support MessagePack serialization for improved performance
//! and storage efficiency.
//!
//! The existing TEXT data is converted to BYTEA using PostgreSQL's text-to-bytea
//! conversion, which preserves the raw bytes. The application layer handles
//! backwards-compatible decoding of both JSON and bincode formats.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Convert TEXT to BYTEA
        // The convert_to function converts text to bytea using UTF-8 encoding,
        // preserving the original JSON string bytes for backwards-compatible decoding
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE operation_logs ALTER COLUMN params TYPE BYTEA USING convert_to(params, 'UTF8')"
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Rollback: Convert BYTEA back to TEXT
        // Note: This will only work correctly for data that was originally JSON text.
        // New bincode-encoded data will produce garbled text.
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE operation_logs ALTER COLUMN params TYPE TEXT USING convert_from(params, 'UTF8')"
            )
            .await?;
        Ok(())
    }
}
