use crate::mutations::sheets::CopyWorksheetEndMutation;
use crate::registry::{MutationId, TransformRegistry};

pub const MUTATION_ID: MutationId = CopyWorksheetEndMutation::ID;

/// Register transforms for CopyWorksheetEndMutation
///
/// Mutation ID: sheet.mutation.copy-worksheet-end
///
/// CopyWorksheetEndMutation copies a worksheet to the end of the workbook.
/// Transform strategy: Identity (operates at workbook level).
pub fn register_transforms(_registry: &mut TransformRegistry) {
    // Symmetric: CopyWorksheetEnd vs CopyWorksheetEnd (identity - concurrent copies don't conflict)

}
