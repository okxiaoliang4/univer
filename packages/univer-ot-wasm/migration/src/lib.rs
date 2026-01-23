pub use sea_orm_migration::prelude::*;

mod m20250101_000001_create_tables;
mod m20250102_000001_add_documents_table;
mod m20260120_000001_add_storage_table;
mod m20260123_000001_add_op_id_to_operation_logs;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250101_000001_create_tables::Migration),
            Box::new(m20250102_000001_add_documents_table::Migration),
            Box::new(m20260120_000001_add_storage_table::Migration),
            Box::new(m20260123_000001_add_op_id_to_operation_logs::Migration),
        ]
    }
}
