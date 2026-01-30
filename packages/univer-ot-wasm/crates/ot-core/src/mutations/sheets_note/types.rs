use serde::{Deserialize, Serialize};

// ISheetNote
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ISheetNote {
    pub width: f64,
    pub height: f64,
    pub note: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show: Option<bool>,
}
