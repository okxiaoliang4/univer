use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::OnceLock;
use crate::registry::TransformFnRef;
use crate::types::{IRowData, IObjectArrayPrimitiveType};
use crate::utils::transform_factory::{
    HasWorksheetParams, HasRowData,
    create_insert_row_vs_row_data_transform,
    create_remove_row_vs_row_data_transform,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetRowDataMutationParams {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub row_data: IObjectArrayPrimitiveType<Option<IRowData>>,
}

// Implement traits for transform factory
impl HasWorksheetParams for SetRowDataMutationParams {
    fn unit_id(&self) -> &str {
        &self.unit_id
    }
    fn sub_unit_id(&self) -> &str {
        &self.sub_unit_id
    }
}

impl HasRowData for SetRowDataMutationParams {
    type Value = Option<IRowData>;
    fn row_data_mut(&mut self) -> &mut HashMap<String, Self::Value> {
        &mut self.row_data
    }
}

/// Cached transform instances for SetRowDataMutationParams
impl SetRowDataMutationParams {
    /// Get cached InsertRow vs SetRowData transform (singleton)
    pub fn insert_row_transform() -> TransformFnRef {
        static TRANSFORM: OnceLock<TransformFnRef> = OnceLock::new();
        TRANSFORM.get_or_init(|| {
            create_insert_row_vs_row_data_transform::<Self>()
        }).clone()
    }

    /// Get cached RemoveRow vs SetRowData transform (singleton)
    pub fn remove_row_transform() -> TransformFnRef {
        static TRANSFORM: OnceLock<TransformFnRef> = OnceLock::new();
        TRANSFORM.get_or_init(|| {
            create_remove_row_vs_row_data_transform::<Self>()
        }).clone()
    }
}

pub struct SetRowDataMutation;

impl SetRowDataMutation {
    pub const ID: &'static str = "sheet.mutation.set-row-data";

    pub fn handler(_params: SetRowDataMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}
