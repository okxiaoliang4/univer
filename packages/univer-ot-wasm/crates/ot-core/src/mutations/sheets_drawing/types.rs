use serde::{Deserialize, Serialize};

// DrawingApplyType enum (already defined in set_drawing_apply.rs, but we'll define it here for consistency)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum DrawingApplyType {
    Insert,
    Remove,
    Update,
    Arrange,
    Group,
    Ungroup,
}

// IDrawingSearch
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IDrawingSearch {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub drawing_id: String,
}

// IDrawingOrderMapParam
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IDrawingOrderMapParam {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub drawing_ids: Vec<String>,
}

// IDrawingParam - simplified, using serde_json::Value for the full drawing data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IDrawingParam {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub drawing_id: String,
    // The full drawing object is complex and varies by type, so we use Value
    #[serde(flatten)]
    pub data: serde_json::Value,
}

// IDrawingGroupUpdateParam
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IDrawingGroupUpdateParam {
    pub parent: IDrawingParam,
    pub children: Vec<IDrawingParam>,
}

// Drawing objects union type - can be IDrawingSearch[], IDrawingOrderMapParam, or IDrawingGroupUpdateParam[]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DrawingObjects {
    SearchList(Vec<IDrawingSearch>),
    OrderMap(IDrawingOrderMapParam),
    GroupUpdateList(Vec<IDrawingGroupUpdateParam>),
}
