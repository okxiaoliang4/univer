use crate::types::MutationInfoInternal;
use serde::{Deserialize, Serialize};

/// Request structure for changeset event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangesetRequest {
    #[serde(rename = "docId")]
    #[serde(default)]
    pub doc_id: String,
    #[serde(rename = "baseRev")]
    pub base_rev: i64,
    #[serde(rename = "clientMsgId")]
    pub client_msg_id: String,
    pub mutations: Vec<MutationInfoInternal>,
}

/// Ack response for changeset event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangesetAck {
    pub status: String, // "ok" | "error"
    #[serde(rename = "serverRev")]
    pub server_rev: Option<i64>,
    /// The mutations after server-side OT transformation
    /// This is the "authoritative" result that clients should use to ensure consistency
    pub mutations: Option<Vec<MutationInfoInternal>>,
    pub message: Option<String>,
}

/// Broadcast event for changeset_pushed (sent to observers)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangesetPushed {
    #[serde(rename = "docId")]
    pub doc_id: String,
    #[serde(rename = "serverRev")]
    pub server_rev: i64,
    #[serde(rename = "userId")]
    pub user_id: String,
    pub mutations: Vec<MutationInfoInternal>,
}

/// Request structure for join_doc event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinDocRequest {
    #[serde(rename = "docId")]
    pub doc_id: String,
}

/// Ack response for join_doc event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinDocAck {
    pub status: String, // "ok" | "error"
    pub version: Option<i64>,
    pub content: Option<serde_json::Value>,
    pub message: Option<String>,
}

/// Request structure for leave_doc event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaveDocRequest {
    #[serde(rename = "docId")]
    pub doc_id: String,
}

/// Request structure for fetch_ops event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchOpsRequest {
    #[serde(rename = "docId")]
    pub doc_id: String,
    #[serde(rename = "startRev")]
    pub start_rev: i64,
}

/// Operation info for fetch_ops response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationInfo {
    pub rev: i64,
    #[serde(rename = "userId")]
    pub user_id: String,
    pub mutations: Vec<MutationInfoInternal>,
}

/// Ack response for fetch_ops event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchOpsAck {
    pub status: String, // "ok" | "error"
    pub operations: Option<Vec<OperationInfo>>,
    pub message: Option<String>,
}

/// Request structure for presence_update event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceUpdateRequest {
    #[serde(rename = "docId")]
    pub doc_id: String,
    pub cursor: Option<serde_json::Value>,
    pub selection: Option<serde_json::Value>,
}
