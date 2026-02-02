use crate::mutations::sheets::SetWorkbookNameMutation;
use crate::registry::{MutationId, TransformRegistry};
use crate::utils::transform_helpers::lww_transform;

pub const MUTATION_ID: MutationId = SetWorkbookNameMutation::ID;

/// Register transforms for SetWorkbookNameMutation
///
/// Mutation ID: sheet.mutation.set-workbook-name
///
/// SetWorkbookNameMutation renames the workbook.
/// Transform strategy: Last-Write-Wins (LWW) at workbook level.
/// Identity with all sheet-level and cell-level mutations (workbook name is independent).
pub fn register_transforms(registry: &mut TransformRegistry) {
    // Self-transform: LWW
    registry.register_symmetric_ref(MUTATION_ID, lww_transform());

    // NOTE: No register_identity calls needed - registry falls back to identity automatically
}
