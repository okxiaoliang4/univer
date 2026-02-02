//! Transforms for SetColDataMutation
//!
//! This mutation has column_data field where keys are column indices.
//! When column insert/remove operations happen, these keys need adjustment.
//! Uses shared Generic transforms for position shifting.

use crate::mutations::sheets::{SetColDataMutation, InsertColMutation, RemoveColMutation};
use crate::registry::{MutationId, TransformRegistry};
use crate::utils::generic_params::GenericColDataParams;
use crate::utils::shared_transforms as shared;

pub const MUTATION_ID: MutationId = SetColDataMutation::ID;

pub fn register_transforms(registry: &mut TransformRegistry) {
    // InsertCol vs SetColData: shift column_data keys >= insert position
    registry.register_bidirectional_ref(
        InsertColMutation::ID,
        MUTATION_ID,
        shared::insert_col_shift::<GenericColDataParams>(),
    );

    // RemoveCol vs SetColData: shift column_data keys > remove end, remove keys in range
    registry.register_bidirectional_ref(
        RemoveColMutation::ID,
        MUTATION_ID,
        shared::remove_col_shift::<GenericColDataParams>(),
    );
}
