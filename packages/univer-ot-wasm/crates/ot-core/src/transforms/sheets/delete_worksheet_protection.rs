use crate::mutations::sheets::DeleteWorksheetProtectionMutation;
use crate::registry::{MutationId, TransformRegistry};

pub const MUTATION_ID: MutationId = DeleteWorksheetProtectionMutation::ID;

/// Register transforms for DeleteWorksheetProtectionMutation
///
/// Mutation ID: sheet.mutation.delete-worksheet-protection
///
/// DeleteWorksheetProtectionMutation removes protection from a worksheet.
/// Transform strategy: Identity (delete operations don't conflict).
pub fn register_transforms(_registry: &mut TransformRegistry) {
    // Self-transform: identity


    // NOTE: No register_identity calls needed - registry falls back to identity automatically
}
