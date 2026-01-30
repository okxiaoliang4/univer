use serde::{Deserialize, Serialize};

// ICellLinkContent
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ICellLinkContent {
    pub payload: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display: Option<String>,
}

// ISheetHyperLink
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ISheetHyperLink {
    pub id: String,
    pub row: i32,
    pub column: i32,
    pub payload: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display: Option<String>,
}
