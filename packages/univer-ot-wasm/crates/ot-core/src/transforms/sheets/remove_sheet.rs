use crate::mutations::sheets::RemoveSheetMutation;
use crate::registry::{MutationId, TransformRegistry};

pub const MUTATION_ID: MutationId = RemoveSheetMutation::ID;

/// Register transforms for RemoveSheetMutation
///
/// Mutation ID: sheet.mutation.remove-sheet
///
/// RemoveSheetMutation removes a worksheet from the workbook.
/// Transform strategy: Identity (operates at workbook level).
pub fn register_transforms(_registry: &mut TransformRegistry) {
    // Self-transform: identity


    // NOTE: No register_identity calls needed - registry falls back to identity automatically
}
