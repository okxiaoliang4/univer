use crate::mutations::{AddRangeProtectionMutationParams, AddWorksheetProtectionMutation};
use crate::mutations::{InsertRowMutation, InsertColMutation, RemoveRowMutation, RemoveColMutation, MoveColsMutation, MoveRowsMutation, MoveRangeMutation, RemoveSheetMutation};
use crate::registry::TransformRegistry;
use crate::utils::{ShiftOperation, ShiftResult, Shiftable, WorksheetParams, shift_ranges_vec};
use crate::utils::shared_transforms::{insert_col_shift, insert_row_shift, remove_col_shift, remove_row_shift, move_cols_shift, move_rows_shift, move_range_shift, remove_sheet_shift};

impl WorksheetParams for AddRangeProtectionMutationParams {
  fn unit_id(&self) -> &str { &self.unit_id }
  fn sub_unit_id(&self) -> &str { &self.sub_unit_id }
}

impl Shiftable for AddRangeProtectionMutationParams {
  fn apply_shift(&mut self, op: &ShiftOperation) -> ShiftResult {
      if self.rules.is_empty() {
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
      let mut rules_to_remove = Vec::new();

      for (idx, rule) in self.rules.iter_mut().enumerate() {
          // Only shift rules for the same sheet as the mutation
          if rule.sub_unit_id != self.sub_unit_id {
              continue;
          }

          let ranges_remain = shift_ranges_vec(&mut rule.ranges, op);
          if !ranges_remain {
              rules_to_remove.push(idx);
          }
          any_modified = true;
      }

      // Remove rules with empty ranges (from back to front to preserve indices)
      for idx in rules_to_remove.into_iter().rev() {
          self.rules.remove(idx);
      }

      if self.rules.is_empty() {
          ShiftResult::Removed
      } else if any_modified {
          ShiftResult::Modified
      } else {
          ShiftResult::Unchanged
      }
  }
}

pub fn register_transforms(registry: &mut TransformRegistry) {
  registry.register_bidirectional_ref(InsertRowMutation::ID, AddWorksheetProtectionMutation::ID, insert_row_shift::<AddRangeProtectionMutationParams>());
  registry.register_bidirectional_ref(InsertColMutation::ID, AddWorksheetProtectionMutation::ID, insert_col_shift::<AddRangeProtectionMutationParams>());
  registry.register_bidirectional_ref(RemoveRowMutation::ID, AddWorksheetProtectionMutation::ID, remove_row_shift::<AddRangeProtectionMutationParams>());
  registry.register_bidirectional_ref(RemoveColMutation::ID, AddWorksheetProtectionMutation::ID, remove_col_shift::<AddRangeProtectionMutationParams>());
  registry.register_bidirectional_ref(MoveColsMutation::ID, AddWorksheetProtectionMutation::ID, move_cols_shift::<AddRangeProtectionMutationParams>());
  registry.register_bidirectional_ref(MoveRowsMutation::ID, AddWorksheetProtectionMutation::ID, move_rows_shift::<AddRangeProtectionMutationParams>());
  registry.register_bidirectional_ref(MoveRangeMutation::ID, AddWorksheetProtectionMutation::ID, move_range_shift::<AddRangeProtectionMutationParams>());
  registry.register_bidirectional_ref(RemoveSheetMutation::ID, AddWorksheetProtectionMutation::ID, remove_sheet_shift::<AddRangeProtectionMutationParams>());
}
