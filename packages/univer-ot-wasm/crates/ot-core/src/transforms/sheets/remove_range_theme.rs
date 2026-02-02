use crate::mutations::sheets::RemoveRangeThemeMutation;
use crate::registry::{MutationId, TransformRegistry};

pub const MUTATION_ID: MutationId = RemoveRangeThemeMutation::ID;

/// Register transforms for RemoveRangeThemeMutation
///
/// Mutation ID: sheet.mutation.remove-range-theme
///
/// RemoveRangeThemeMutation removes a theme from a range.
/// Transform strategy: Identity (theme operations don't conflict).
pub fn register_transforms(_registry: &mut TransformRegistry) {
    // Self-transform: identity


    // NOTE: No register_identity calls needed - registry falls back to identity automatically
}
