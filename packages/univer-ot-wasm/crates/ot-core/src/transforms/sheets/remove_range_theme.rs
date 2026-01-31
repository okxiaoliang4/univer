use crate::mutations::sheets::RemoveRangeThemeMutation;
use crate::registry::{MutationId, TransformFnRef, TransformRegistry};
use crate::utils::transform_helpers::identity_transform;
use crate::types::{MutationInfo, TransformResultRef};

pub const MUTATION_ID: MutationId = RemoveRangeThemeMutation::ID;

/// Register transforms for RemoveRangeThemeMutation
///
/// Mutation ID: sheet.mutation.remove-range-theme
///
/// RemoveRangeThemeMutation removes a theme from a range.
/// Transform strategy: Identity (theme operations don't conflict).
pub fn register_transforms(registry: &mut TransformRegistry) {
    // Self-transform: identity
    registry.register_symmetric_ref(MUTATION_ID, identity_transform());
}

pub fn register_cross_module_transforms(registry: &mut TransformRegistry, other_mutations: &[MutationId]) {
    for &other_id in other_mutations {
        registry.register_identity(MUTATION_ID, other_id);
    }
}