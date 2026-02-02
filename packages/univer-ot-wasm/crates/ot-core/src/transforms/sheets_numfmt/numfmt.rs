//! Transforms for number format mutations
//!
//! Mutations handled:
//! - SetNumfmtMutation: sheet.mutation.set.numfmt (Category C - nested values.ranges)
//! - RemoveNumfmtMutation: sheet.mutation.remove.numfmt (Category A - plain ranges)

use crate::mutations::sheets::{
    InsertRowMutation, InsertRowMutationParams,
    InsertColMutation, InsertColMutationParams,
    RemoveRowMutation, RemoveRowsMutationParams,
    RemoveColMutation, RemoveColMutationParams,
};
use crate::mutations::sheets_numfmt::{SetNumfmtMutation, SetNumfmtMutationParams, RemoveNumfmtMutation};
use crate::registry::{MutationId, TransformRegistry, TransformFnRef};
use crate::types::{MutationInfo, TransformResultRef};
use crate::utils::{ShiftResult, Shiftable, WorksheetParams, shift_ranges_vec};
use crate::utils::params::same_worksheet;
use crate::utils::shift_operations::ShiftOperation;
use crate::utils::generic_params::GenericRangesParams;
use crate::utils::shared_transforms::{self as shared, apply_shift_transform};
use std::sync::{Arc, OnceLock};

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
    // Self-transforms: Default to identity via fallback
    // (concurrent numfmt operations on different ranges don't conflict)

    // Shift transforms for SetNumfmt (Category C - has nested values.ranges)
    registry.register_bidirectional_ref(InsertRowMutation::ID, SET_NUMFMT_MUTATION_ID, insert_row_vs_set_numfmt());
    registry.register_bidirectional_ref(InsertColMutation::ID, SET_NUMFMT_MUTATION_ID, insert_col_vs_set_numfmt());
    registry.register_bidirectional_ref(RemoveRowMutation::ID, SET_NUMFMT_MUTATION_ID, remove_row_vs_set_numfmt());
    registry.register_bidirectional_ref(RemoveColMutation::ID, SET_NUMFMT_MUTATION_ID, remove_col_vs_set_numfmt());

    // Shift transforms for RemoveNumfmt (Category A - has plain ranges: Vec<IRange>)
    registry.register_bidirectional_ref(InsertRowMutation::ID, REMOVE_NUMFMT_MUTATION_ID, shared::insert_row_shift::<GenericRangesParams>());
    registry.register_bidirectional_ref(InsertColMutation::ID, REMOVE_NUMFMT_MUTATION_ID, shared::insert_col_shift::<GenericRangesParams>());
    registry.register_bidirectional_ref(RemoveRowMutation::ID, REMOVE_NUMFMT_MUTATION_ID, shared::remove_row_shift::<GenericRangesParams>());
    registry.register_bidirectional_ref(RemoveColMutation::ID, REMOVE_NUMFMT_MUTATION_ID, shared::remove_col_shift::<GenericRangesParams>());
}

// ============================================================================
// Local Transform Functions for SetNumfmt (Category C)
// ============================================================================

/// Transform: InsertRow vs SetNumfmt (values with nested ranges)
fn insert_row_vs_set_numfmt() -> TransformFnRef {
    static TRANSFORM: OnceLock<TransformFnRef> = OnceLock::new();
    TRANSFORM.get_or_init(|| {
        Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
            if let Some(false) = same_worksheet(&m1.params, &m2.params) {
                return TransformResultRef::identity(m1, m2);
            }

            let m1_params: InsertRowMutationParams = match serde_json::from_value(m1.params.clone()) {
                Ok(p) => p,
                Err(_) => return TransformResultRef::identity(m1, m2),
            };

            let shift_op = ShiftOperation::InsertRows {
                start: m1_params.range.start_row,
                count: m1_params.range.end_row - m1_params.range.start_row + 1,
            };

            apply_shift_transform::<SetNumfmtMutationParams>(m1, m2, &m1_params.sub_unit_params, shift_op)
        })
    }).clone()
}

/// Transform: RemoveRow vs SetNumfmt (values with nested ranges)
fn remove_row_vs_set_numfmt() -> TransformFnRef {
    static TRANSFORM: OnceLock<TransformFnRef> = OnceLock::new();
    TRANSFORM.get_or_init(|| {
        Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
            if let Some(false) = same_worksheet(&m1.params, &m2.params) {
                return TransformResultRef::identity(m1, m2);
            }

            let m1_params: RemoveRowsMutationParams = match serde_json::from_value(m1.params.clone()) {
                Ok(p) => p,
                Err(_) => return TransformResultRef::identity(m1, m2),
            };

            let shift_op = ShiftOperation::RemoveRows {
                start: m1_params.range.start_row,
                end: m1_params.range.end_row,
            };

            apply_shift_transform::<SetNumfmtMutationParams>(m1, m2, &m1_params.sub_unit_params, shift_op)
        })
    }).clone()
}

/// Transform: InsertCol vs SetNumfmt (values with nested ranges)
fn insert_col_vs_set_numfmt() -> TransformFnRef {
    static TRANSFORM: OnceLock<TransformFnRef> = OnceLock::new();
    TRANSFORM.get_or_init(|| {
        Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
            if let Some(false) = same_worksheet(&m1.params, &m2.params) {
                return TransformResultRef::identity(m1, m2);
            }

            let m1_params: InsertColMutationParams = match serde_json::from_value(m1.params.clone()) {
                Ok(p) => p,
                Err(_) => return TransformResultRef::identity(m1, m2),
            };

            let shift_op = ShiftOperation::InsertCols {
                start: m1_params.range.start_column,
                count: m1_params.range.end_column - m1_params.range.start_column + 1,
            };

            apply_shift_transform::<SetNumfmtMutationParams>(m1, m2, &m1_params.sub_unit_params, shift_op)
        })
    }).clone()
}

/// Transform: RemoveCol vs SetNumfmt (values with nested ranges)
fn remove_col_vs_set_numfmt() -> TransformFnRef {
    static TRANSFORM: OnceLock<TransformFnRef> = OnceLock::new();
    TRANSFORM.get_or_init(|| {
        Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
            if let Some(false) = same_worksheet(&m1.params, &m2.params) {
                return TransformResultRef::identity(m1, m2);
            }

            let m1_params: RemoveColMutationParams = match serde_json::from_value(m1.params.clone()) {
                Ok(p) => p,
                Err(_) => return TransformResultRef::identity(m1, m2),
            };

            let shift_op = ShiftOperation::RemoveCols {
                start: m1_params.range.start_column,
                end: m1_params.range.end_column,
            };

            apply_shift_transform::<SetNumfmtMutationParams>(m1, m2, &m1_params.sub_unit_params, shift_op)
        })
    }).clone()
}
