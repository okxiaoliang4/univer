use crate::mutations::SetRangeProtectionMutationParams;
use crate::mutations::sheets::SetWorksheetProtectionMutation;
use crate::registry::TransformRegistry;
use crate::utils::transform_helpers::lww_transform;
use crate::utils::{ShiftOperation, ShiftResult, Shiftable, WorksheetParams, shift_ranges_vec};


impl WorksheetParams for SetRangeProtectionMutationParams {
  fn unit_id(&self) -> &str { &self.unit_id }
  fn sub_unit_id(&self) -> &str { &self.sub_unit_id }
}

impl Shiftable for SetRangeProtectionMutationParams {
  fn apply_shift(&mut self, op: &ShiftOperation) -> ShiftResult {
      // Handle RemoveSheet at the mutation level
      if let ShiftOperation::RemoveSheet { sub_unit_id } = op {
          if &self.sub_unit_id == sub_unit_id {
              return ShiftResult::Removed;
          }
          return ShiftResult::Unchanged;
      }

      // Only shift if the rule is for the same sheet as the mutation
      if self.rule.sub_unit_id != self.sub_unit_id {
          return ShiftResult::Unchanged;
      }

      let ranges_remain = shift_ranges_vec(&mut self.rule.ranges, op);
      if !ranges_remain {
          ShiftResult::Removed
      } else {
          ShiftResult::Modified
      }
  }
}

/// Register transforms for SetWorksheetProtectionMutation
///
/// Mutation ID: sheet.mutation.set-worksheet-protection
///
/// SetWorksheetProtectionMutation sets protection settings for a worksheet.
/// Transform strategy: Last-Write-Wins (LWW) at worksheet level.
pub fn register_transforms(registry: &mut TransformRegistry) {
    // Self-transform: LWW
    registry.register_symmetric_ref(SetWorksheetProtectionMutation::ID, lww_transform());

    // NOTE: No register_identity calls needed - registry falls back to identity automatically
}
