use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "document_snapshots")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(column_name = "doc_id")]
    pub doc_id: Uuid,
    #[sea_orm(column_name = "storage_id")]
    pub storage_id: Uuid,
    #[sea_orm(column_name = "version")]
    pub version: i64,
    #[sea_orm(column_name = "created_at")]
    pub created_at: DateTimeWithTimeZone,
    #[sea_orm(column_name = "updated_at")]
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::documents::Entity",
        from = "Column::DocId",
        to = "super::documents::Column::Id"
    )]
    Document,
    #[sea_orm(
        belongs_to = "super::storage::Entity",
        from = "Column::StorageId",
        to = "super::storage::Column::Id"
    )]
    Storage,
}

impl Related<super::documents::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Document.def()
    }
}

impl Related<super::storage::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Storage.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
