//! Transforms for SetRowDataMutation
//!
//! This mutation has row_data field where keys are row indices.
//! When row insert/remove operations happen, these keys need adjustment.
//! Uses shared Generic transforms for position shifting.

use crate::mutations::sheets::{
    SetRowDataMutation, InsertRowMutation, RemoveRowMutation,
    MoveRowsMutation, RemoveSheetMutation,
};
use crate::registry::{MutationId, TransformRegistry};
use crate::utils::generic_params::GenericRowDataParams;
use crate::utils::shared_transforms as shared;

pub const MUTATION_ID: MutationId = SetRowDataMutation::ID;

pub fn register_transforms(registry: &mut TransformRegistry) {
    // InsertRow vs SetRowData: shift row_data keys >= insert position
    registry.register_bidirectional_ref(
        InsertRowMutation::ID,
        MUTATION_ID,
        shared::insert_row_shift::<GenericRowDataParams>(),
    );

    // RemoveRows vs SetRowData: shift row_data keys > remove end, remove keys in range
    registry.register_bidirectional_ref(
        RemoveRowMutation::ID,
        MUTATION_ID,
        shared::remove_row_shift::<GenericRowDataParams>(),
    );

    // MoveRows vs SetRowData: shift row_data keys for moved rows
    registry.register_bidirectional_ref(
        MoveRowsMutation::ID,
        MUTATION_ID,
        shared::move_rows_shift::<GenericRowDataParams>(),
    );

    // RemoveSheet vs SetRowData: remove mutation if sheet is deleted
    registry.register_bidirectional_ref(
        RemoveSheetMutation::ID,
        MUTATION_ID,
        shared::remove_sheet_shift::<GenericRowDataParams>(),
    );
}
