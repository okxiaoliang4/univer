use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::mutations::sheets::types::IRange;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NumfmtValue {
    pub ranges: Vec<IRange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NumfmtRef {
    pub pattern: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetNumfmtMutationParams {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub values: HashMap<String, NumfmtValue>,
    pub ref_map: HashMap<String, NumfmtRef>,
}

pub struct SetNumfmtMutation;

impl SetNumfmtMutation {
    pub const ID: &'static str = "sheet.mutation.set.numfmt";

    pub fn handler(_params: SetNumfmtMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveNumfmtMutationParams {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub ranges: Vec<IRange>,
}

pub struct RemoveNumfmtMutation;

impl RemoveNumfmtMutation {
    pub const ID: &'static str = "sheet.mutation.remove.numfmt";

    pub fn handler(_params: RemoveNumfmtMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}
