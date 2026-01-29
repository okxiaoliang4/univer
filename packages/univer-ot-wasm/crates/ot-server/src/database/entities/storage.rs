use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "storages")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub endpoint: String,
    pub region: String,
    pub bucket: String,
    pub path: String,
    pub size: i64,
    pub hash: String,
    #[sea_orm(column_name = "hash_algorithm")]
    pub hash_algorithm: String,
    pub filename: String,
    #[sea_orm(column_name = "content_type")]
    pub content_type: Option<String>,
    pub metadata: Option<Json>,
    #[sea_orm(column_name = "version_id")]
    pub version_id: String,
    pub compressed: Option<i32>,
    #[sea_orm(column_name = "created_at")]
    pub created_at: DateTimeWithTimeZone,
    #[sea_orm(column_name = "updated_at")]
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::document_snapshot::Entity")]
    DocumentSnapshots,
}

impl Related<super::document_snapshot::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DocumentSnapshots.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
