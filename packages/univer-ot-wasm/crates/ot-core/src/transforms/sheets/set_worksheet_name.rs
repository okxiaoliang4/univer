use crate::mutations::sheets::SetWorksheetNameMutation;
use crate::registry::{MutationId, TransformRegistry};
use crate::utils::transform_helpers::lww_transform;

pub const MUTATION_ID: MutationId = SetWorksheetNameMutation::ID;

/// Register transforms for SetWorksheetNameMutation
///
/// Mutation ID: sheet.mutation.set-worksheet-name
///
/// SetWorksheetNameMutation renames a worksheet.
/// Transform strategy: Last-Write-Wins (LWW) at worksheet level.
pub fn register_transforms(registry: &mut TransformRegistry) {
    // Self-transform: LWW
    registry.register_symmetric_ref(MUTATION_ID, lww_transform());
}

pub fn register_cross_module_transforms(registry: &mut TransformRegistry, other_mutations: &[MutationId]) {
    for &other_id in other_mutations {
        registry.register_identity(MUTATION_ID, other_id);
    }
}
