use serde::{Deserialize, Serialize};
use super::types::ISheetNote;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateNoteMutationParams {
    pub unit_id: String,
    pub sheet_id: String,
    pub row: i32,
    pub col: i32,
    pub note: ISheetNote,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub silent: Option<bool>,
}

pub struct UpdateNoteMutation;

impl UpdateNoteMutation {
    pub const ID: &'static str = "sheet.mutation.update-note";

    pub fn handler(_params: UpdateNoteMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveNoteMutationParams {
    pub unit_id: String,
    pub sheet_id: String,
    pub row: i32,
    pub col: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub silent: Option<bool>,
}

pub struct RemoveNoteMutation;

impl RemoveNoteMutation {
    pub const ID: &'static str = "sheet.mutation.remove-note";

    pub fn handler(_params: RemoveNoteMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToggleNotePopupMutationParams {
    pub unit_id: String,
    pub sheet_id: String,
    pub row: i32,
    pub col: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub silent: Option<bool>,
}

pub struct ToggleNotePopupMutation;

impl ToggleNotePopupMutation {
    pub const ID: &'static str = "sheet.mutation.toggle-note-popup";

    pub fn handler(_params: ToggleNotePopupMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewPosition {
    pub row: i32,
    pub col: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateNotePositionMutationParams {
    pub unit_id: String,
    pub sheet_id: String,
    pub row: i32,
    pub col: i32,
    pub new_position: NewPosition,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub silent: Option<bool>,
}

pub struct UpdateNotePositionMutation;

impl UpdateNotePositionMutation {
    pub const ID: &'static str = "sheet.mutation.update-note-position";

    pub fn handler(_params: UpdateNotePositionMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}
