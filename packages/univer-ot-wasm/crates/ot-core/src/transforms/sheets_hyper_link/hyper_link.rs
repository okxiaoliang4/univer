use crate::mutations::sheets::{
    InsertRowMutation, InsertColMutation, RemoveRowMutation, RemoveColMutation,
    MoveRowsMutation, MoveColsMutation, MoveRangeMutation, RemoveSheetMutation,
};
use crate::mutations::sheets_hyper_link::{
    AddHyperLinkMutation, AddHyperLinkMutationParams,
    RemoveHyperLinkMutation, UpdateHyperLinkMutation,
    UpdateHyperLinkRefMutation, UpdateHyperLinkRefMutationParams,
    UpdateRichHyperLinkMutation, UpdateRichHyperLinkMutationParams,
};
use crate::registry::{MutationId, TransformRegistry};
use crate::utils::shared_transforms as shared;
use crate::utils::transform_helpers::{lww_transform, identity_transform};

pub const ADD_HYPER_LINK_ID: MutationId = AddHyperLinkMutation::ID;
pub const REMOVE_HYPER_LINK_ID: MutationId = RemoveHyperLinkMutation::ID;
pub const UPDATE_HYPER_LINK_ID: MutationId = UpdateHyperLinkMutation::ID;
pub const UPDATE_HYPER_LINK_REF_ID: MutationId = UpdateHyperLinkRefMutation::ID;
pub const UPDATE_RICH_HYPER_LINK_ID: MutationId = UpdateRichHyperLinkMutation::ID;

pub fn register_transforms(registry: &mut TransformRegistry) {
    // Register symmetric transforms for hyper link mutations
    registry.register_symmetric_ref(ADD_HYPER_LINK_ID, identity_transform());
    registry.register_symmetric_ref(REMOVE_HYPER_LINK_ID, identity_transform());
    registry.register_symmetric_ref(UPDATE_HYPER_LINK_ID, lww_transform());
    registry.register_symmetric_ref(UPDATE_HYPER_LINK_REF_ID, lww_transform());
    registry.register_symmetric_ref(UPDATE_RICH_HYPER_LINK_ID, lww_transform());

    // Register bidirectional transforms within hyper link module
    registry.register_bidirectional_ref(ADD_HYPER_LINK_ID, REMOVE_HYPER_LINK_ID, identity_transform());
    registry.register_bidirectional_ref(ADD_HYPER_LINK_ID, UPDATE_HYPER_LINK_ID, identity_transform());
    registry.register_bidirectional_ref(ADD_HYPER_LINK_ID, UPDATE_HYPER_LINK_REF_ID, identity_transform());
    registry.register_bidirectional_ref(ADD_HYPER_LINK_ID, UPDATE_RICH_HYPER_LINK_ID, identity_transform());
    registry.register_bidirectional_ref(REMOVE_HYPER_LINK_ID, UPDATE_HYPER_LINK_ID, identity_transform());
    registry.register_bidirectional_ref(REMOVE_HYPER_LINK_ID, UPDATE_HYPER_LINK_REF_ID, identity_transform());
    registry.register_bidirectional_ref(REMOVE_HYPER_LINK_ID, UPDATE_RICH_HYPER_LINK_ID, identity_transform());
    registry.register_bidirectional_ref(UPDATE_HYPER_LINK_ID, UPDATE_HYPER_LINK_REF_ID, identity_transform());
    registry.register_bidirectional_ref(UPDATE_HYPER_LINK_ID, UPDATE_RICH_HYPER_LINK_ID, identity_transform());
    registry.register_bidirectional_ref(UPDATE_HYPER_LINK_REF_ID, UPDATE_RICH_HYPER_LINK_ID, identity_transform());

    // Shift transforms for AddHyperLink (has link.row, link.column)
    registry.register_bidirectional_ref(InsertRowMutation::ID, ADD_HYPER_LINK_ID, shared::insert_row_shift::<AddHyperLinkMutationParams>());
    registry.register_bidirectional_ref(InsertColMutation::ID, ADD_HYPER_LINK_ID, shared::insert_col_shift::<AddHyperLinkMutationParams>());
    registry.register_bidirectional_ref(RemoveRowMutation::ID, ADD_HYPER_LINK_ID, shared::remove_row_shift::<AddHyperLinkMutationParams>());
    registry.register_bidirectional_ref(RemoveColMutation::ID, ADD_HYPER_LINK_ID, shared::remove_col_shift::<AddHyperLinkMutationParams>());
    registry.register_bidirectional_ref(MoveRowsMutation::ID, ADD_HYPER_LINK_ID, shared::move_rows_shift::<AddHyperLinkMutationParams>());
    registry.register_bidirectional_ref(MoveColsMutation::ID, ADD_HYPER_LINK_ID, shared::move_cols_shift::<AddHyperLinkMutationParams>());
    registry.register_bidirectional_ref(MoveRangeMutation::ID, ADD_HYPER_LINK_ID, shared::move_range_shift::<AddHyperLinkMutationParams>());
    registry.register_bidirectional_ref(RemoveSheetMutation::ID, ADD_HYPER_LINK_ID, shared::remove_sheet_shift::<AddHyperLinkMutationParams>());

    // Shift transforms for UpdateHyperLinkRef (has row, column)
    registry.register_bidirectional_ref(InsertRowMutation::ID, UPDATE_HYPER_LINK_REF_ID, shared::insert_row_shift::<UpdateHyperLinkRefMutationParams>());
    registry.register_bidirectional_ref(InsertColMutation::ID, UPDATE_HYPER_LINK_REF_ID, shared::insert_col_shift::<UpdateHyperLinkRefMutationParams>());
    registry.register_bidirectional_ref(RemoveRowMutation::ID, UPDATE_HYPER_LINK_REF_ID, shared::remove_row_shift::<UpdateHyperLinkRefMutationParams>());
    registry.register_bidirectional_ref(RemoveColMutation::ID, UPDATE_HYPER_LINK_REF_ID, shared::remove_col_shift::<UpdateHyperLinkRefMutationParams>());
    registry.register_bidirectional_ref(MoveRowsMutation::ID, UPDATE_HYPER_LINK_REF_ID, shared::move_rows_shift::<UpdateHyperLinkRefMutationParams>());
    registry.register_bidirectional_ref(MoveColsMutation::ID, UPDATE_HYPER_LINK_REF_ID, shared::move_cols_shift::<UpdateHyperLinkRefMutationParams>());
    registry.register_bidirectional_ref(MoveRangeMutation::ID, UPDATE_HYPER_LINK_REF_ID, shared::move_range_shift::<UpdateHyperLinkRefMutationParams>());
    registry.register_bidirectional_ref(RemoveSheetMutation::ID, UPDATE_HYPER_LINK_REF_ID, shared::remove_sheet_shift::<UpdateHyperLinkRefMutationParams>());

    // Shift transforms for UpdateRichHyperLink (has row, col)
    registry.register_bidirectional_ref(InsertRowMutation::ID, UPDATE_RICH_HYPER_LINK_ID, shared::insert_row_shift::<UpdateRichHyperLinkMutationParams>());
    registry.register_bidirectional_ref(InsertColMutation::ID, UPDATE_RICH_HYPER_LINK_ID, shared::insert_col_shift::<UpdateRichHyperLinkMutationParams>());
    registry.register_bidirectional_ref(RemoveRowMutation::ID, UPDATE_RICH_HYPER_LINK_ID, shared::remove_row_shift::<UpdateRichHyperLinkMutationParams>());
    registry.register_bidirectional_ref(RemoveColMutation::ID, UPDATE_RICH_HYPER_LINK_ID, shared::remove_col_shift::<UpdateRichHyperLinkMutationParams>());
    registry.register_bidirectional_ref(MoveRowsMutation::ID, UPDATE_RICH_HYPER_LINK_ID, shared::move_rows_shift::<UpdateRichHyperLinkMutationParams>());
    registry.register_bidirectional_ref(MoveColsMutation::ID, UPDATE_RICH_HYPER_LINK_ID, shared::move_cols_shift::<UpdateRichHyperLinkMutationParams>());
    registry.register_bidirectional_ref(MoveRangeMutation::ID, UPDATE_RICH_HYPER_LINK_ID, shared::move_range_shift::<UpdateRichHyperLinkMutationParams>());
    registry.register_bidirectional_ref(RemoveSheetMutation::ID, UPDATE_RICH_HYPER_LINK_ID, shared::remove_sheet_shift::<UpdateRichHyperLinkMutationParams>());

    // NOTE: RemoveHyperLink and UpdateHyperLink don't have position fields, no shift needed
}