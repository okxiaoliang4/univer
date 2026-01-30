use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionalFormattingFormulaMarkDirtyParams {
    #[serde(flatten)]
    pub data: HashMap<String, HashMap<String, HashMap<String, bool>>>,
}

pub struct ConditionalFormattingFormulaMarkDirty;

impl ConditionalFormattingFormulaMarkDirty {
    pub const ID: &'static str = "sheet.mutation.conditional-formatting-formula-mark-dirty";

    pub fn handler(_params: ConditionalFormattingFormulaMarkDirtyParams) -> Result<bool, String> {
        Ok(true)
    }
}
