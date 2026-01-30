use serde::{Deserialize, Serialize};
use crate::types::IWorksheetProtectionPointRule;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetWorksheetPermissionPointsMutationParams {
    pub rule: IWorksheetProtectionPointRule,
    pub unit_id: String,
    pub sub_unit_id: String,
}

pub struct SetWorksheetPermissionPointsMutation;

impl SetWorksheetPermissionPointsMutation {
    pub const ID: &'static str = "sheet.mutation.set-worksheet-permission-points";

    pub fn handler(_params: SetWorksheetPermissionPointsMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}
