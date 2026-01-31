use crate::mutations::sheets::ReorderRangeMutation;
use crate::registry::{MutationId, TransformFnRef, TransformRegistry};
use crate::utils::transform_helpers::identity_transform;
use crate::types::{MutationInfo, TransformResultRef};

pub const REORDER_RANGE_ID: MutationId = ReorderRangeMutation::ID;

/// Register transforms for ReorderRangeMutation
///
/// Mutation ID: sheet.mutation.reorder-range
///
/// ReorderRangeMutation reorders cells within a range.
/// Transform strategy: Identity (TODO: needs proper implementation for conflicts).
///
/// TODO: Implement proper conflict resolution:
/// - Self-transform: Need to analyze range overlap and reorder conflicts
/// - vs structural mutations: Need to adjust range positions
/// - vs data mutations: Need to determine precedence
pub fn register_transforms(registry: &mut TransformRegistry) {
    // Self-transform: currently identity (needs proper implementation)
    registry.register_symmetric_ref(REORDER_RANGE_ID, identity_transform());
}

pub fn register_cross_module_transforms(registry: &mut TransformRegistry, other_mutations: &[MutationId]) {
    for &other_id in other_mutations {
        registry.register_identity(REORDER_RANGE_ID, other_id);
    }
}