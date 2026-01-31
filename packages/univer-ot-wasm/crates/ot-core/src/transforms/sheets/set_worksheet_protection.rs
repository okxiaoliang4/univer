use crate::mutations::sheets::SetWorksheetProtectionMutation;
use crate::registry::{MutationId, TransformFnRef, TransformRegistry};
use crate::utils::transform_helpers::lww_transform;
use crate::types::{MutationInfo, MutationOutcome, TransformResultRef};

pub const MUTATION_ID: MutationId = SetWorksheetProtectionMutation::ID;

/// Register transforms for SetWorksheetProtectionMutation
///
/// Mutation ID: sheet.mutation.set-worksheet-protection
///
/// SetWorksheetProtectionMutation sets protection settings for a worksheet.
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