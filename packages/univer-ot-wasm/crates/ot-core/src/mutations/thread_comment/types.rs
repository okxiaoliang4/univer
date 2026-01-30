use serde::{Deserialize, Serialize};

// IDocumentBody - document body structure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IDocumentBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_stream: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_runs: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paragraphs: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub section_breaks: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_ranges: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_blocks: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tables: Option<Vec<serde_json::Value>>,
}

// IThreadCommentMention
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IThreadCommentMention {
    pub label: String,
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
}

// IBaseComment
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IBaseComment {
    pub id: String,
    pub thread_id: String,
    #[serde(rename = "dT")]
    pub d_t: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_t: Option<String>,
    pub person_id: String,
    pub text: IDocumentBody,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachments: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mentions: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved: Option<bool>,
    pub unit_id: String,
    pub sub_unit_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<IBaseComment>>,
}

// IThreadComment
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IThreadComment {
    pub id: String,
    pub thread_id: String,
    #[serde(rename = "dT")]
    pub d_t: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_t: Option<String>,
    pub person_id: String,
    pub text: IDocumentBody,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachments: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mentions: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved: Option<bool>,
    pub unit_id: String,
    pub sub_unit_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<IBaseComment>>,
    #[serde(rename = "ref")]
    pub ref_field: String,
}
