use crate::mutations::sheets::RegisterWorksheetRangeThemeStyleMutation;
use crate::registry::{MutationId, TransformRegistry};

pub const MUTATION_ID: MutationId = RegisterWorksheetRangeThemeStyleMutation::ID;

/// Register transforms for RegisterWorksheetRangeThemeStyleMutation
///
/// Mutation ID: sheet.mutation.register-worksheet-range-theme-style
///
/// RegisterWorksheetRangeThemeStyleMutation registers a new range theme style.
/// Transform strategy: Identity (registration operations don't conflict).
pub fn register_transforms(_registry: &mut TransformRegistry) {
    // Self-transform: identity


    // NOTE: No register_identity calls needed - registry falls back to identity automatically
}
