use crate::mutations::sheets::{AddRangeThemeMutation, AddRangeThemeMutationParams, RemoveSheetMutation};
use crate::registry::{MutationId, TransformRegistry};
use crate::utils::{ShiftOperation, ShiftResult, Shiftable, WorksheetParams};
use crate::utils::shared_transforms as shared;

pub const MUTATION_ID: MutationId = AddRangeThemeMutation::ID;

impl WorksheetParams for AddRangeThemeMutationParams {
    fn unit_id(&self) -> &str { &self.unit_id }
    fn sub_unit_id(&self) -> &str { &self.sub_unit_id }
}

impl Shiftable for AddRangeThemeMutationParams {
    fn apply_shift(&mut self, op: &ShiftOperation) -> ShiftResult {
        match op {
            ShiftOperation::RemoveSheet { sub_unit_id } => {
                if &self.sub_unit_id == sub_unit_id {
                    ShiftResult::Removed
                } else {
                    ShiftResult::Unchanged
                }
            }
            // No position data to shift for other operations
            _ => ShiftResult::Unchanged,
        }
    }
}

/// Register transforms for AddRangeThemeMutation
///
/// Mutation ID: sheet.mutation.add-range-theme
///
/// AddRangeThemeMutation adds a theme to a range.
/// Transform strategy: Identity (theme operations don't conflict).
/// RemoveSheet: Remove if the sheet is deleted.
pub fn register_transforms(registry: &mut TransformRegistry) {
    // RemoveSheet vs AddRangeTheme: remove mutation if sheet is deleted
    registry.register_bidirectional_ref(
        RemoveSheetMutation::ID,
        MUTATION_ID,
        shared::remove_sheet_shift::<AddRangeThemeMutationParams>(),
    );
}
