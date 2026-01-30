use serde::{Deserialize, Serialize};
use super::types::{IRange, NumfmtRanges, NumfmtRefItem};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetNumfmtMutationParams {
    pub values: HashMap<String, NumfmtRanges>,
    pub ref_map: HashMap<String, NumfmtRefItem>,
    pub unit_id: String,
    pub sub_unit_id: String,
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
    pub ranges: Vec<IRange>,
    pub unit_id: String,
    pub sub_unit_id: String,
}

pub struct RemoveNumfmtMutation;

impl RemoveNumfmtMutation {
    pub const ID: &'static str = "sheet.mutation.remove.numfmt";

    pub fn handler(_params: RemoveNumfmtMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}
