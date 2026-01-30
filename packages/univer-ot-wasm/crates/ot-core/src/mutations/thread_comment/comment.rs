use serde::{Deserialize, Serialize};
use super::types::{IThreadComment, IDocumentBody};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddCommentMutationParams {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub comment: IThreadComment,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sync: Option<bool>,
}

pub struct AddCommentMutation;

impl AddCommentMutation {
    pub const ID: &'static str = "thread-comment.mutation.add-comment";

    pub fn handler(_params: AddCommentMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCommentPayload {
    pub comment_id: String,
    pub text: IDocumentBody,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachments: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_t: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCommentMutationParams {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub payload: UpdateCommentPayload,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub silent: Option<bool>,
}

pub struct UpdateCommentMutation;

impl UpdateCommentMutation {
    pub const ID: &'static str = "thread-comment.mutation.update-comment";

    pub fn handler(_params: UpdateCommentMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCommentRefPayload {
    pub comment_id: String,
    #[serde(rename = "ref")]
    pub ref_field: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCommentRefMutationParams {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub payload: UpdateCommentRefPayload,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub silent: Option<bool>,
}

pub struct UpdateCommentRefMutation;

impl UpdateCommentRefMutation {
    pub const ID: &'static str = "thread-comment.mutation.update-comment-ref";

    pub fn handler(_params: UpdateCommentRefMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolveCommentMutationParams {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub comment_id: String,
    pub resolved: bool,
}

pub struct ResolveCommentMutation;

impl ResolveCommentMutation {
    pub const ID: &'static str = "thread-comment.mutation.resolve-comment";

    pub fn handler(_params: ResolveCommentMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteCommentMutationParams {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub comment_id: String,
}

pub struct DeleteCommentMutation;

impl DeleteCommentMutation {
    pub const ID: &'static str = "thread-comment.mutation.delete-comment";

    pub fn handler(_params: DeleteCommentMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}
