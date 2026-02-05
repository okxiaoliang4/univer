pub use sea_orm_migration::prelude::*;

mod m20250101_000001_create_tables;
mod m20250102_000001_add_documents_table;
mod m20260120_000001_add_storage_table;
mod m20260123_000001_add_op_id_to_operation_logs;
mod m20260125_000001_add_document_metadata;
mod m20250128_000001_add_desc_version_index;
mod m20250129_000001_operation_logs_params_to_text;
mod m20260202_000001_operation_logs_params_to_binary;
mod m20260204_000001_operation_logs_params_to_storage_id;
mod m20260205_000001_drop_redundant_indexes;
mod m20260206_000001_operation_logs_inline_params;
mod m20260207_000001_remove_client_id;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250101_000001_create_tables::Migration),
            Box::new(m20250102_000001_add_documents_table::Migration),
            Box::new(m20260120_000001_add_storage_table::Migration),
            Box::new(m20260123_000001_add_op_id_to_operation_logs::Migration),
            Box::new(m20260125_000001_add_document_metadata::Migration),
            Box::new(m20250128_000001_add_desc_version_index::Migration),
            Box::new(m20250129_000001_operation_logs_params_to_text::Migration),
            Box::new(m20260202_000001_operation_logs_params_to_binary::Migration),
            Box::new(m20260204_000001_operation_logs_params_to_storage_id::Migration),
            Box::new(m20260205_000001_drop_redundant_indexes::Migration),
            Box::new(m20260206_000001_operation_logs_inline_params::Migration),
            Box::new(m20260207_000001_remove_client_id::Migration),
        ]
    }
}
