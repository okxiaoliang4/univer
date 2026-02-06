use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "operation_logs")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    #[sea_orm(column_name = "doc_id")]
    pub doc_id: Uuid,
    #[sea_orm(column_name = "rev")]
    pub rev: i64,
    #[sea_orm(column_name = "user_id")]
    pub user_id: String,
    #[sea_orm(column_name = "mutation_id")]
    pub mutation_id: String,
    /// Storage ID for S3-stored params (large operations >= threshold)
    /// NULL when params are stored inline in the `params` column
    #[sea_orm(column_name = "storage_id")]
    pub storage_id: Option<Uuid>,
    /// Inline params for small operations (< threshold, typically 2KB)
    /// NULL when params are stored in S3 (referenced by storage_id)
    /// MessagePack encoded bytes
    #[sea_orm(column_name = "params")]
    pub params: Option<Vec<u8>>,
    #[sea_orm(column_name = "op_id")]
    pub op_id: String,
    #[sea_orm(column_name = "created_at")]
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::documents::Entity",
        from = "Column::DocId",
        to = "super::documents::Column::Id"
    )]
    Document,
}

impl Related<super::documents::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Document.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
