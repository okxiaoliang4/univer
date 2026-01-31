use crate::mutations::sheets::SetWorksheetDefaultStyleMutation;
use crate::registry::{MutationId, TransformFnRef, TransformRegistry};
use crate::utils::transform_helpers::lww_transform;
use crate::types::{MutationInfo, MutationOutcome, TransformResultRef};

pub const MUTATION_ID: MutationId = SetWorksheetDefaultStyleMutation::ID;

/// Register transforms for SetWorksheetDefaultStyleMutation
///
/// Mutation ID: sheet.mutation.set-worksheet-default-style
///
/// SetWorksheetDefaultStyleMutation sets the default style for a worksheet.
/// Transform strategy: Last-Write-Wins (LWW) at worksheet level.
/// Identity with all other mutations (default style is independent).
pub fn register_transforms(registry: &mut TransformRegistry) {
    // Self-transform: LWW
    registry.register_symmetric_ref(MUTATION_ID, lww_transform());
}

pub fn register_cross_module_transforms(registry: &mut TransformRegistry, other_mutations: &[MutationId]) {
    for &other_id in other_mutations {
        registry.register_identity(MUTATION_ID, other_id);
    }
}