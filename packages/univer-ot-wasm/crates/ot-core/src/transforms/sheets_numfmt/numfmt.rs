//! Transforms for number format mutations
//!
//! Mutations handled:
//! - SetNumfmtMutation: sheet.mutation.set.numfmt (Category C - nested values.ranges)
//! - RemoveNumfmtMutation: sheet.mutation.remove.numfmt (Category A - plain ranges)

use crate::mutations::{MoveColsMutation, MoveRangeMutation, MoveRowsMutation, RemoveSheetMutation};
use crate::mutations::sheets::{
    InsertRowMutation,
    InsertColMutation,
    RemoveRowMutation,
    RemoveColMutation,
};
use crate::mutations::sheets_numfmt::{SetNumfmtMutation, SetNumfmtMutationParams, RemoveNumfmtMutation};
use crate::registry::{MutationId, TransformRegistry};
use crate::utils::{ShiftResult, Shiftable, WorksheetParams, shift_ranges_vec};
use crate::utils::shift_operations::ShiftOperation;
use crate::utils::generic_params::GenericRangesParams;
use crate::utils::shared_transforms::{self as shared};

pub const SET_NUMFMT_MUTATION_ID: MutationId = SetNumfmtMutation::ID;
pub const REMOVE_NUMFMT_MUTATION_ID: MutationId = RemoveNumfmtMutation::ID;

impl WorksheetParams for SetNumfmtMutationParams {
  fn unit_id(&self) -> &str { &self.unit_id }
  fn sub_unit_id(&self) -> &str { &self.sub_unit_id }
}

impl Shiftable for SetNumfmtMutationParams {
  fn apply_shift(&mut self, op: &ShiftOperation) -> ShiftResult {
      if self.values.is_empty() {
          return ShiftResult::Unchanged;
      }

      // Handle RemoveSheet at the mutation level
      if let ShiftOperation::RemoveSheet { sub_unit_id } = op {
          if &self.sub_unit_id == sub_unit_id {
              return ShiftResult::Removed;
          }
          return ShiftResult::Unchanged;
      }

      let mut any_modified = false;
      let mut keys_to_remove = Vec::new();

      for (key, numfmt_ranges) in self.values.iter_mut() {
          let ranges_remain = shift_ranges_vec(&mut numfmt_ranges.ranges, op);
          if !ranges_remain {
              keys_to_remove.push(key.clone());
          }
          any_modified = true;
      }

      // Remove entries with empty ranges
      for key in keys_to_remove {
          self.values.remove(&key);
      }

      if self.values.is_empty() {
          ShiftResult::Removed
      } else if any_modified {
          ShiftResult::Modified
      } else {
          ShiftResult::Unchanged
      }
  }
}

/// Register transforms for number format mutations
///
/// Mutations handled:
/// - SetNumfmtMutation: sheet.mutation.set.numfmt
/// - RemoveNumfmtMutation: sheet.mutation.remove.numfmt
pub fn register_transforms(registry: &mut TransformRegistry) {
    // Shift transforms for SetNumfmt (Category C - has nested values.ranges)
    registry.register_bidirectional_ref(InsertRowMutation::ID, SET_NUMFMT_MUTATION_ID, shared::insert_row_shift::<SetNumfmtMutationParams>());
    registry.register_bidirectional_ref(InsertColMutation::ID, SET_NUMFMT_MUTATION_ID, shared::insert_col_shift::<SetNumfmtMutationParams>());
    registry.register_bidirectional_ref(RemoveRowMutation::ID, SET_NUMFMT_MUTATION_ID, shared::remove_row_shift::<SetNumfmtMutationParams>());
    registry.register_bidirectional_ref(RemoveColMutation::ID, SET_NUMFMT_MUTATION_ID, shared::remove_col_shift::<SetNumfmtMutationParams>());
    registry.register_bidirectional_ref(MoveRowsMutation::ID, SET_NUMFMT_MUTATION_ID, shared::move_rows_shift::<SetNumfmtMutationParams>());
    registry.register_bidirectional_ref(MoveColsMutation::ID, SET_NUMFMT_MUTATION_ID, shared::move_cols_shift::<SetNumfmtMutationParams>());
    registry.register_bidirectional_ref(MoveRangeMutation::ID, SET_NUMFMT_MUTATION_ID, shared::move_range_shift::<SetNumfmtMutationParams>());
    registry.register_bidirectional_ref(RemoveSheetMutation::ID, SET_NUMFMT_MUTATION_ID, shared::remove_sheet_shift::<SetNumfmtMutationParams>());

    // Shift transforms for RemoveNumfmt (Category A - has plain ranges: Vec<IRange>)
    registry.register_bidirectional_ref(InsertRowMutation::ID, REMOVE_NUMFMT_MUTATION_ID, shared::insert_row_shift::<GenericRangesParams>());
    registry.register_bidirectional_ref(InsertColMutation::ID, REMOVE_NUMFMT_MUTATION_ID, shared::insert_col_shift::<GenericRangesParams>());
    registry.register_bidirectional_ref(RemoveRowMutation::ID, REMOVE_NUMFMT_MUTATION_ID, shared::remove_row_shift::<GenericRangesParams>());
    registry.register_bidirectional_ref(RemoveColMutation::ID, REMOVE_NUMFMT_MUTATION_ID, shared::remove_col_shift::<GenericRangesParams>());
    registry.register_bidirectional_ref(MoveRowsMutation::ID, REMOVE_NUMFMT_MUTATION_ID, shared::move_rows_shift::<GenericRangesParams>());
    registry.register_bidirectional_ref(MoveColsMutation::ID, REMOVE_NUMFMT_MUTATION_ID, shared::move_cols_shift::<GenericRangesParams>());
    registry.register_bidirectional_ref(MoveRangeMutation::ID, REMOVE_NUMFMT_MUTATION_ID, shared::move_range_shift::<GenericRangesParams>());
    registry.register_bidirectional_ref(RemoveSheetMutation::ID, REMOVE_NUMFMT_MUTATION_ID, shared::remove_sheet_shift::<GenericRangesParams>());
}
