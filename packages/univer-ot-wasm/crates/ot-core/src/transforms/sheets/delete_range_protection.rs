use crate::mutations::sheets::DeleteRangeProtectionMutation;
use crate::registry::{MutationId, TransformRegistry};

pub const MUTATION_ID: MutationId = DeleteRangeProtectionMutation::ID;

/// Register transforms for DeleteRangeProtectionMutation
///
/// Mutation ID: sheet.mutation.delete-range-protection
///
/// DeleteRangeProtectionMutation removes protection from a range.
/// Transform strategy: Identity (protection operations don't conflict).
pub fn register_transforms(_registry: &mut TransformRegistry) {
    // Self-transform: identity


    // NOTE: No register_identity calls needed - registry falls back to identity automatically
}
