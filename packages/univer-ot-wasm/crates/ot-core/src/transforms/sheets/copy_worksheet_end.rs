use crate::mutations::sheets::CopyWorksheetEndMutation;
use crate::registry::{MutationId, TransformFnRef, TransformRegistry};
use crate::utils::transform_helpers::identity_transform;
use crate::types::{MutationInfo, TransformResultRef};

pub const MUTATION_ID: MutationId = CopyWorksheetEndMutation::ID;

/// Register transforms for CopyWorksheetEndMutation
///
/// Mutation ID: sheet.mutation.copy-worksheet-end
///
/// CopyWorksheetEndMutation copies a worksheet to the end of the workbook.
/// Transform strategy: Identity (operates at workbook level).
pub fn register_transforms(registry: &mut TransformRegistry) {
    // Self-transform: identity
    registry.register_symmetric_ref(MUTATION_ID, identity_transform());
}

pub fn register_cross_module_transforms(registry: &mut TransformRegistry, other_mutations: &[MutationId]) {
    for &other_id in other_mutations {
        registry.register_identity(MUTATION_ID, other_id);
    }
}