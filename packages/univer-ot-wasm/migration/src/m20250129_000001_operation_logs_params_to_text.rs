use sea_orm::DbBackend;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let backend = db.get_database_backend();
        match backend {
            DbBackend::Postgres => {
                // PostgreSQL: jsonb -> text requires USING for existing data
                db.execute_unprepared(
                    "ALTER TABLE operation_logs ALTER COLUMN params TYPE text USING params::text",
                )
                .await?;
            }
            _ => {
                manager
                    .alter_table(
                        Table::alter()
                            .table(OperationLogs::Table)
                            .modify_column(
                                ColumnDef::new(OperationLogs::Params)
                                    .text()
                                    .not_null(),
                            )
                            .to_owned(),
                    )
                    .await?;
            }
        }
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let backend = db.get_database_backend();
        match backend {
            DbBackend::Postgres => {
                db.execute_unprepared(
                    "ALTER TABLE operation_logs ALTER COLUMN params TYPE jsonb USING params::jsonb",
                )
                .await?;
            }
            _ => {
                manager
                    .alter_table(
                        Table::alter()
                            .table(OperationLogs::Table)
                            .modify_column(
                                ColumnDef::new(OperationLogs::Params)
                                    .json_binary()
                                    .not_null(),
                            )
                            .to_owned(),
                    )
                    .await?;
            }
        }
        Ok(())
    }
}

#[derive(DeriveIden)]
enum OperationLogs {
    Table,
    Params,
}
