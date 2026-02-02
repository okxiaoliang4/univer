use crate::mutations::sheets::InsertSheetMutation;
use crate::registry::{MutationId, TransformRegistry};

pub const MUTATION_ID: MutationId = InsertSheetMutation::ID;

/// Register transforms for InsertSheetMutation
///
/// Mutation ID: sheet.mutation.insert-sheet
///
/// InsertSheetMutation inserts a new worksheet into the workbook.
/// Transform strategy: Identity (operates at workbook level).
/// Identity with all sheet-level operations (doesn't interfere with cell-level operations).
pub fn register_transforms(_registry: &mut TransformRegistry) {
    // Self-transform: identity


    // NOTE: No register_identity calls needed - registry falls back to identity automatically
}
