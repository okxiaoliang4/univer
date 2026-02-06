use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "documents")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub name: String,
    #[sea_orm(column_name = "creator_id")]
    pub creator_id: String,
    #[sea_orm(column_name = "doc_type")]
    pub doc_type: i16,
    #[sea_orm(column_name = "create_type")]
    pub create_type: i16,
    #[sea_orm(column_name = "current_version")]
    pub current_version: i64,
    #[sea_orm(column_name = "created_at")]
    pub created_at: DateTimeWithTimeZone,
    #[sea_orm(column_name = "updated_at")]
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::operation_log::Entity")]
    OperationLogs,
    #[sea_orm(has_many = "super::document_snapshot::Entity")]
    DocumentSnapshots,
}

impl Related<super::operation_log::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::OperationLogs.def()
    }
}

impl Related<super::document_snapshot::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DocumentSnapshots.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
