use crate::mutations::AddRangeProtectionMutationParams;
use crate::mutations::sheets::AddWorksheetProtectionMutation;
use crate::registry::{MutationId, TransformRegistry};
use crate::utils::{ShiftOperation, ShiftResult, Shiftable, WorksheetParams, shift_ranges_vec};

pub const MUTATION_ID: MutationId = AddWorksheetProtectionMutation::ID;


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

/// Register transforms for AddWorksheetProtectionMutation
///
/// Mutation ID: sheet.mutation.add-worksheet-protection
///
/// AddWorksheetProtectionMutation adds protection to a worksheet.
/// Transform strategy: Identity (add operations don't conflict).
pub fn register_transforms(_registry: &mut TransformRegistry) {
    // Symmetric: AddWorksheetProtection vs AddWorksheetProtection (identity - concurrent adds don't conflict)

}
