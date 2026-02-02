use crate::mutations::sheets::AddRangeThemeMutation;
use crate::registry::{MutationId, TransformRegistry};

pub const MUTATION_ID: MutationId = AddRangeThemeMutation::ID;

/// Register transforms for AddRangeThemeMutation
///
/// Mutation ID: sheet.mutation.add-range-theme
///
/// AddRangeThemeMutation adds a theme to a range.
/// Transform strategy: Identity (theme operations don't conflict).
pub fn register_transforms(_registry: &mut TransformRegistry) {
}
