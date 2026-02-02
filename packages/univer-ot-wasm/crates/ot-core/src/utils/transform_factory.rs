//! Transform factory functions for unified shift conflict resolution
//!
//! This module provides factory functions to create transforms for common
//! shift conflict patterns. These reduce boilerplate when implementing
//! transforms between structural mutations (insert/remove row/col) and
//! data mutations (set values, merge, etc.).
//!
//! # Factory Types
//!
//! 1. **Range-based factories**: For mutations with `range: IRange` or `ranges: Vec<IRange>`
//! 2. **Cell value factories**: For mutations with `cell_value: ObjectMatrixPrimitiveType`
//! 3. **Array key factories**: For mutations with `row_data`/`col_data` HashMaps
//!
//! # Usage Example
//!
//! ```ignore
//! use crate::utils::transform_factory::create_insert_row_vs_ranges_transform;
//!
//! registry.register_bidirectional_ref(
//!     InsertRowMutation::ID,
//!     AddWorksheetMergeMutation::ID,
//!     create_insert_row_vs_ranges_transform::<AddWorksheetMergeMutationParams>(),
//! );
//! ```

use crate::mutations::sheets::{
    InsertRowMutationParams, InsertColMutationParams,
    RemoveRowsMutationParams, RemoveColMutationParams,
};
use crate::registry::TransformFnRef;
use crate::types::{MutationInfo, MutationOutcome, TransformResultRef, IRange};
use crate::utils::params::same_worksheet;
use crate::utils::shiftable::{
    ShiftableByRowInsert, ShiftableByRowRemove,
    ShiftableByColInsert, ShiftableByColRemove,
};
use serde::{de::DeserializeOwned, Serialize};
use std::sync::Arc;

// ============================================================================
// Helper traits for extracting worksheet params
// ============================================================================

/// Trait for mutation params that have worksheet identification
pub trait HasWorksheetParams {
    fn unit_id(&self) -> &str;
    fn sub_unit_id(&self) -> &str;
}

/// Trait for mutation params that have a single shiftable range
pub trait HasRange {
    fn range_mut(&mut self) -> &mut IRange;
}

/// Trait for mutation params that have multiple shiftable ranges
pub trait HasRanges {
    fn ranges_mut(&mut self) -> &mut Vec<IRange>;
}

// ============================================================================
// Insert Row vs Ranges Transform Factory
// ============================================================================

/// Create a transform for InsertRow vs mutation with `ranges: Vec<IRange>`
///
/// This factory creates a transform that shifts all ranges in m2 when m1 is
/// an insert-row operation. The transform:
/// 1. Checks worksheet match
/// 2. Parses both mutations
/// 3. Shifts all ranges in m2 based on insert position
///
/// # Type Parameters
/// - `T`: The target mutation params type, must implement:
///   - `DeserializeOwned + Serialize + Clone` for JSON handling
///   - `HasWorksheetParams` for worksheet matching
///   - `HasRanges` for getting mutable access to ranges
///
/// # Example
/// ```ignore
/// registry.register_bidirectional_ref(
///     InsertRowMutation::ID,
///     AddWorksheetMergeMutation::ID,
///     create_insert_row_vs_ranges_transform::<AddWorksheetMergeMutationParams>(),
/// );
/// ```
pub fn create_insert_row_vs_ranges_transform<T>() -> TransformFnRef
where
    T: DeserializeOwned + Serialize + Clone + HasWorksheetParams + HasRanges + Send + Sync + 'static,
{
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        // Quick worksheet check BEFORE parsing
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return TransformResultRef::identity(m1, m2);
        }

        // Parse InsertRow params (m1)
        let m1_params: InsertRowMutationParams = match serde_json::from_value(m1.params.clone()) {
            Ok(p) => p,
            Err(_) => return TransformResultRef::identity(m1, m2),
        };

        // Parse target params (m2)
        let mut m2_params: T = match serde_json::from_value(m2.params.clone()) {
            Ok(p) => p,
            Err(_) => return TransformResultRef::identity(m1, m2),
        };

        // Double-check worksheet match
        if m1_params.sub_unit_params.unit_id != m2_params.unit_id()
            || m1_params.sub_unit_params.sub_unit_id != m2_params.sub_unit_id()
        {
            return TransformResultRef::identity(m1, m2);
        }

        let insert_start = m1_params.range.start_row;
        let insert_count = m1_params.range.end_row - insert_start + 1;

        // Apply shift to all ranges using the Shiftable trait
        m2_params.ranges_mut().shift_for_row_insert(insert_start, insert_count);

        TransformResultRef {
            m1_prime: MutationOutcome::Unchanged(m1),
            m2_prime: MutationOutcome::Modified(MutationInfo {
                id: m2.id.clone(),
                params: serde_json::to_value(m2_params).unwrap_or_else(|_| m2.params.clone()),
            }),
            error: None,
        }
    })
}

/// Create a transform for RemoveRows vs mutation with `ranges: Vec<IRange>`
pub fn create_remove_row_vs_ranges_transform<T>() -> TransformFnRef
where
    T: DeserializeOwned + Serialize + Clone + HasWorksheetParams + HasRanges + Send + Sync + 'static,
{
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        // Quick worksheet check BEFORE parsing
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return TransformResultRef::identity(m1, m2);
        }

        // Parse RemoveRows params (m1)
        let m1_params: RemoveRowsMutationParams = match serde_json::from_value(m1.params.clone()) {
            Ok(p) => p,
            Err(_) => return TransformResultRef::identity(m1, m2),
        };

        // Parse target params (m2)
        let mut m2_params: T = match serde_json::from_value(m2.params.clone()) {
            Ok(p) => p,
            Err(_) => return TransformResultRef::identity(m1, m2),
        };

        // Double-check worksheet match
        if m1_params.sub_unit_params.unit_id != m2_params.unit_id()
            || m1_params.sub_unit_params.sub_unit_id != m2_params.sub_unit_id()
        {
            return TransformResultRef::identity(m1, m2);
        }

        let remove_start = m1_params.range.start_row;
        let remove_end = m1_params.range.end_row;

        // Apply shift to all ranges - returns false if all ranges removed
        let retained = m2_params.ranges_mut().shift_for_row_remove(remove_start, remove_end);

        if !retained {
            // All ranges were removed - remove the mutation
            return TransformResultRef {
                m1_prime: MutationOutcome::Unchanged(m1),
                m2_prime: MutationOutcome::Removed,
                error: None,
            };
        }

        TransformResultRef {
            m1_prime: MutationOutcome::Unchanged(m1),
            m2_prime: MutationOutcome::Modified(MutationInfo {
                id: m2.id.clone(),
                params: serde_json::to_value(m2_params).unwrap_or_else(|_| m2.params.clone()),
            }),
            error: None,
        }
    })
}

// ============================================================================
// Insert Col vs Ranges Transform Factory
// ============================================================================

/// Create a transform for InsertCol vs mutation with `ranges: Vec<IRange>`
pub fn create_insert_col_vs_ranges_transform<T>() -> TransformFnRef
where
    T: DeserializeOwned + Serialize + Clone + HasWorksheetParams + HasRanges + Send + Sync + 'static,
{
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        // Quick worksheet check BEFORE parsing
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return TransformResultRef::identity(m1, m2);
        }

        // Parse InsertCol params (m1)
        let m1_params: InsertColMutationParams = match serde_json::from_value(m1.params.clone()) {
            Ok(p) => p,
            Err(_) => return TransformResultRef::identity(m1, m2),
        };

        // Parse target params (m2)
        let mut m2_params: T = match serde_json::from_value(m2.params.clone()) {
            Ok(p) => p,
            Err(_) => return TransformResultRef::identity(m1, m2),
        };

        // Double-check worksheet match
        if m1_params.sub_unit_params.unit_id != m2_params.unit_id()
            || m1_params.sub_unit_params.sub_unit_id != m2_params.sub_unit_id()
        {
            return TransformResultRef::identity(m1, m2);
        }

        let insert_start = m1_params.range.start_column;
        let insert_count = m1_params.range.end_column - insert_start + 1;

        // Apply shift to all ranges
        m2_params.ranges_mut().shift_for_col_insert(insert_start, insert_count);

        TransformResultRef {
            m1_prime: MutationOutcome::Unchanged(m1),
            m2_prime: MutationOutcome::Modified(MutationInfo {
                id: m2.id.clone(),
                params: serde_json::to_value(m2_params).unwrap_or_else(|_| m2.params.clone()),
            }),
            error: None,
        }
    })
}

/// Create a transform for RemoveCol vs mutation with `ranges: Vec<IRange>`
pub fn create_remove_col_vs_ranges_transform<T>() -> TransformFnRef
where
    T: DeserializeOwned + Serialize + Clone + HasWorksheetParams + HasRanges + Send + Sync + 'static,
{
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        // Quick worksheet check BEFORE parsing
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return TransformResultRef::identity(m1, m2);
        }

        // Parse RemoveCol params (m1)
        let m1_params: RemoveColMutationParams = match serde_json::from_value(m1.params.clone()) {
            Ok(p) => p,
            Err(_) => return TransformResultRef::identity(m1, m2),
        };

        // Parse target params (m2)
        let mut m2_params: T = match serde_json::from_value(m2.params.clone()) {
            Ok(p) => p,
            Err(_) => return TransformResultRef::identity(m1, m2),
        };

        // Double-check worksheet match
        if m1_params.sub_unit_params.unit_id != m2_params.unit_id()
            || m1_params.sub_unit_params.sub_unit_id != m2_params.sub_unit_id()
        {
            return TransformResultRef::identity(m1, m2);
        }

        let remove_start = m1_params.range.start_column;
        let remove_end = m1_params.range.end_column;

        // Apply shift to all ranges
        let retained = m2_params.ranges_mut().shift_for_col_remove(remove_start, remove_end);

        if !retained {
            return TransformResultRef {
                m1_prime: MutationOutcome::Unchanged(m1),
                m2_prime: MutationOutcome::Removed,
                error: None,
            };
        }

        TransformResultRef {
            m1_prime: MutationOutcome::Unchanged(m1),
            m2_prime: MutationOutcome::Modified(MutationInfo {
                id: m2.id.clone(),
                params: serde_json::to_value(m2_params).unwrap_or_else(|_| m2.params.clone()),
            }),
            error: None,
        }
    })
}

// ============================================================================
// Cell Value (2D HashMap) Transform Factories
// ============================================================================

/// Trait for mutation params that have cell_value field
pub trait HasCellValue {
    fn cell_value_mut(&mut self) -> Option<&mut crate::types::IObjectMatrixPrimitiveType>;
}

/// Create a transform for InsertRow vs mutation with `cell_value: Option<ObjectMatrixPrimitiveType>`
pub fn create_insert_row_vs_cell_value_transform<T>() -> TransformFnRef
where
    T: DeserializeOwned + Serialize + Clone + HasWorksheetParams + HasCellValue + Send + Sync + 'static,
{
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return TransformResultRef::identity(m1, m2);
        }

        let m1_params: InsertRowMutationParams = match serde_json::from_value(m1.params.clone()) {
            Ok(p) => p,
            Err(_) => return TransformResultRef::identity(m1, m2),
        };

        let mut m2_params: T = match serde_json::from_value(m2.params.clone()) {
            Ok(p) => p,
            Err(_) => return TransformResultRef::identity(m1, m2),
        };

        if m1_params.sub_unit_params.unit_id != m2_params.unit_id()
            || m1_params.sub_unit_params.sub_unit_id != m2_params.sub_unit_id()
        {
            return TransformResultRef::identity(m1, m2);
        }

        let insert_start = m1_params.range.start_row;
        let insert_count = m1_params.range.end_row - insert_start + 1;

        if let Some(cell_value) = m2_params.cell_value_mut() {
            cell_value.shift_for_row_insert(insert_start, insert_count);
        }

        TransformResultRef {
            m1_prime: MutationOutcome::Unchanged(m1),
            m2_prime: MutationOutcome::Modified(MutationInfo {
                id: m2.id.clone(),
                params: serde_json::to_value(m2_params).unwrap_or_else(|_| m2.params.clone()),
            }),
            error: None,
        }
    })
}

/// Create a transform for RemoveRows vs mutation with `cell_value`
pub fn create_remove_row_vs_cell_value_transform<T>() -> TransformFnRef
where
    T: DeserializeOwned + Serialize + Clone + HasWorksheetParams + HasCellValue + Send + Sync + 'static,
{
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return TransformResultRef::identity(m1, m2);
        }

        let m1_params: RemoveRowsMutationParams = match serde_json::from_value(m1.params.clone()) {
            Ok(p) => p,
            Err(_) => return TransformResultRef::identity(m1, m2),
        };

        let mut m2_params: T = match serde_json::from_value(m2.params.clone()) {
            Ok(p) => p,
            Err(_) => return TransformResultRef::identity(m1, m2),
        };

        if m1_params.sub_unit_params.unit_id != m2_params.unit_id()
            || m1_params.sub_unit_params.sub_unit_id != m2_params.sub_unit_id()
        {
            return TransformResultRef::identity(m1, m2);
        }

        let remove_start = m1_params.range.start_row;
        let remove_end = m1_params.range.end_row;

        if let Some(cell_value) = m2_params.cell_value_mut() {
            cell_value.shift_for_row_remove(remove_start, remove_end);
        }

        TransformResultRef {
            m1_prime: MutationOutcome::Unchanged(m1),
            m2_prime: MutationOutcome::Modified(MutationInfo {
                id: m2.id.clone(),
                params: serde_json::to_value(m2_params).unwrap_or_else(|_| m2.params.clone()),
            }),
            error: None,
        }
    })
}

/// Create a transform for InsertCol vs mutation with `cell_value`
pub fn create_insert_col_vs_cell_value_transform<T>() -> TransformFnRef
where
    T: DeserializeOwned + Serialize + Clone + HasWorksheetParams + HasCellValue + Send + Sync + 'static,
{
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return TransformResultRef::identity(m1, m2);
        }

        let m1_params: InsertColMutationParams = match serde_json::from_value(m1.params.clone()) {
            Ok(p) => p,
            Err(_) => return TransformResultRef::identity(m1, m2),
        };

        let mut m2_params: T = match serde_json::from_value(m2.params.clone()) {
            Ok(p) => p,
            Err(_) => return TransformResultRef::identity(m1, m2),
        };

        if m1_params.sub_unit_params.unit_id != m2_params.unit_id()
            || m1_params.sub_unit_params.sub_unit_id != m2_params.sub_unit_id()
        {
            return TransformResultRef::identity(m1, m2);
        }

        let insert_start = m1_params.range.start_column;
        let insert_count = m1_params.range.end_column - insert_start + 1;

        if let Some(cell_value) = m2_params.cell_value_mut() {
            cell_value.shift_for_col_insert(insert_start, insert_count);
        }

        TransformResultRef {
            m1_prime: MutationOutcome::Unchanged(m1),
            m2_prime: MutationOutcome::Modified(MutationInfo {
                id: m2.id.clone(),
                params: serde_json::to_value(m2_params).unwrap_or_else(|_| m2.params.clone()),
            }),
            error: None,
        }
    })
}

/// Create a transform for RemoveCol vs mutation with `cell_value`
pub fn create_remove_col_vs_cell_value_transform<T>() -> TransformFnRef
where
    T: DeserializeOwned + Serialize + Clone + HasWorksheetParams + HasCellValue + Send + Sync + 'static,
{
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return TransformResultRef::identity(m1, m2);
        }

        let m1_params: RemoveColMutationParams = match serde_json::from_value(m1.params.clone()) {
            Ok(p) => p,
            Err(_) => return TransformResultRef::identity(m1, m2),
        };

        let mut m2_params: T = match serde_json::from_value(m2.params.clone()) {
            Ok(p) => p,
            Err(_) => return TransformResultRef::identity(m1, m2),
        };

        if m1_params.sub_unit_params.unit_id != m2_params.unit_id()
            || m1_params.sub_unit_params.sub_unit_id != m2_params.sub_unit_id()
        {
            return TransformResultRef::identity(m1, m2);
        }

        let remove_start = m1_params.range.start_column;
        let remove_end = m1_params.range.end_column;

        if let Some(cell_value) = m2_params.cell_value_mut() {
            cell_value.shift_for_col_remove(remove_start, remove_end);
        }

        TransformResultRef {
            m1_prime: MutationOutcome::Unchanged(m1),
            m2_prime: MutationOutcome::Modified(MutationInfo {
                id: m2.id.clone(),
                params: serde_json::to_value(m2_params).unwrap_or_else(|_| m2.params.clone()),
            }),
            error: None,
        }
    })
}

// ============================================================================
// Row/Col Data (1D HashMap) Transform Factories
// ============================================================================

use crate::utils::shiftable::{
    shift_hashmap_keys_for_row_insert, shift_hashmap_keys_for_row_remove,
    shift_hashmap_keys_for_col_insert, shift_hashmap_keys_for_col_remove,
};
use std::collections::HashMap;

/// Trait for mutation params that have row_data field (HashMap with row indices as keys)
pub trait HasRowData {
    type Value;
    fn row_data_mut(&mut self) -> &mut HashMap<String, Self::Value>;
}

/// Trait for mutation params that have column_data field (HashMap with column indices as keys)
pub trait HasColData {
    type Value;
    fn col_data_mut(&mut self) -> &mut HashMap<String, Self::Value>;
}

/// Create a transform for InsertRow vs mutation with `row_data: HashMap<String, V>`
pub fn create_insert_row_vs_row_data_transform<T>() -> TransformFnRef
where
    T: DeserializeOwned + Serialize + Clone + HasWorksheetParams + HasRowData + Send + Sync + 'static,
{
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return TransformResultRef::identity(m1, m2);
        }

        let m1_params: InsertRowMutationParams = match serde_json::from_value(m1.params.clone()) {
            Ok(p) => p,
            Err(_) => return TransformResultRef::identity(m1, m2),
        };

        let mut m2_params: T = match serde_json::from_value(m2.params.clone()) {
            Ok(p) => p,
            Err(_) => return TransformResultRef::identity(m1, m2),
        };

        if m1_params.sub_unit_params.unit_id != m2_params.unit_id()
            || m1_params.sub_unit_params.sub_unit_id != m2_params.sub_unit_id()
        {
            return TransformResultRef::identity(m1, m2);
        }

        let insert_start = m1_params.range.start_row;
        let insert_count = m1_params.range.end_row - insert_start + 1;

        shift_hashmap_keys_for_row_insert(m2_params.row_data_mut(), insert_start, insert_count);

        TransformResultRef {
            m1_prime: MutationOutcome::Unchanged(m1),
            m2_prime: MutationOutcome::Modified(MutationInfo {
                id: m2.id.clone(),
                params: serde_json::to_value(m2_params).unwrap_or_else(|_| m2.params.clone()),
            }),
            error: None,
        }
    })
}

/// Create a transform for RemoveRows vs mutation with `row_data: HashMap<String, V>`
pub fn create_remove_row_vs_row_data_transform<T>() -> TransformFnRef
where
    T: DeserializeOwned + Serialize + Clone + HasWorksheetParams + HasRowData + Send + Sync + 'static,
{
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return TransformResultRef::identity(m1, m2);
        }

        let m1_params: RemoveRowsMutationParams = match serde_json::from_value(m1.params.clone()) {
            Ok(p) => p,
            Err(_) => return TransformResultRef::identity(m1, m2),
        };

        let mut m2_params: T = match serde_json::from_value(m2.params.clone()) {
            Ok(p) => p,
            Err(_) => return TransformResultRef::identity(m1, m2),
        };

        if m1_params.sub_unit_params.unit_id != m2_params.unit_id()
            || m1_params.sub_unit_params.sub_unit_id != m2_params.sub_unit_id()
        {
            return TransformResultRef::identity(m1, m2);
        }

        let remove_start = m1_params.range.start_row;
        let remove_end = m1_params.range.end_row;

        shift_hashmap_keys_for_row_remove(m2_params.row_data_mut(), remove_start, remove_end);

        TransformResultRef {
            m1_prime: MutationOutcome::Unchanged(m1),
            m2_prime: MutationOutcome::Modified(MutationInfo {
                id: m2.id.clone(),
                params: serde_json::to_value(m2_params).unwrap_or_else(|_| m2.params.clone()),
            }),
            error: None,
        }
    })
}

/// Create a transform for InsertCol vs mutation with `column_data: HashMap<String, V>`
pub fn create_insert_col_vs_col_data_transform<T>() -> TransformFnRef
where
    T: DeserializeOwned + Serialize + Clone + HasWorksheetParams + HasColData + Send + Sync + 'static,
{
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return TransformResultRef::identity(m1, m2);
        }

        let m1_params: InsertColMutationParams = match serde_json::from_value(m1.params.clone()) {
            Ok(p) => p,
            Err(_) => return TransformResultRef::identity(m1, m2),
        };

        let mut m2_params: T = match serde_json::from_value(m2.params.clone()) {
            Ok(p) => p,
            Err(_) => return TransformResultRef::identity(m1, m2),
        };

        if m1_params.sub_unit_params.unit_id != m2_params.unit_id()
            || m1_params.sub_unit_params.sub_unit_id != m2_params.sub_unit_id()
        {
            return TransformResultRef::identity(m1, m2);
        }

        let insert_start = m1_params.range.start_column;
        let insert_count = m1_params.range.end_column - insert_start + 1;

        shift_hashmap_keys_for_col_insert(m2_params.col_data_mut(), insert_start, insert_count);

        TransformResultRef {
            m1_prime: MutationOutcome::Unchanged(m1),
            m2_prime: MutationOutcome::Modified(MutationInfo {
                id: m2.id.clone(),
                params: serde_json::to_value(m2_params).unwrap_or_else(|_| m2.params.clone()),
            }),
            error: None,
        }
    })
}

/// Create a transform for RemoveCol vs mutation with `column_data: HashMap<String, V>`
pub fn create_remove_col_vs_col_data_transform<T>() -> TransformFnRef
where
    T: DeserializeOwned + Serialize + Clone + HasWorksheetParams + HasColData + Send + Sync + 'static,
{
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return TransformResultRef::identity(m1, m2);
        }

        let m1_params: RemoveColMutationParams = match serde_json::from_value(m1.params.clone()) {
            Ok(p) => p,
            Err(_) => return TransformResultRef::identity(m1, m2),
        };

        let mut m2_params: T = match serde_json::from_value(m2.params.clone()) {
            Ok(p) => p,
            Err(_) => return TransformResultRef::identity(m1, m2),
        };

        if m1_params.sub_unit_params.unit_id != m2_params.unit_id()
            || m1_params.sub_unit_params.sub_unit_id != m2_params.sub_unit_id()
        {
            return TransformResultRef::identity(m1, m2);
        }

        let remove_start = m1_params.range.start_column;
        let remove_end = m1_params.range.end_column;

        shift_hashmap_keys_for_col_remove(m2_params.col_data_mut(), remove_start, remove_end);

        TransformResultRef {
            m1_prime: MutationOutcome::Unchanged(m1),
            m2_prime: MutationOutcome::Modified(MutationInfo {
                id: m2.id.clone(),
                params: serde_json::to_value(m2_params).unwrap_or_else(|_| m2.params.clone()),
            }),
            error: None,
        }
    })
}

// ============================================================================
// Simplified functional helpers for non-generic use cases
// ============================================================================

/// Extract insert row info from InsertRowMutationParams
///
/// Returns (insert_start, insert_count) or None if parsing fails
pub fn extract_insert_row_info(params: &serde_json::Value) -> Option<(i32, i32)> {
    let p: InsertRowMutationParams = serde_json::from_value(params.clone()).ok()?;
    let insert_start = p.range.start_row;
    let insert_count = p.range.end_row - insert_start + 1;
    Some((insert_start, insert_count))
}

/// Extract remove row info from RemoveRowsMutationParams
///
/// Returns (remove_start, remove_end) or None if parsing fails
pub fn extract_remove_row_info(params: &serde_json::Value) -> Option<(i32, i32)> {
    let p: RemoveRowsMutationParams = serde_json::from_value(params.clone()).ok()?;
    Some((p.range.start_row, p.range.end_row))
}

/// Extract insert col info from InsertColMutationParams
///
/// Returns (insert_start, insert_count) or None if parsing fails
pub fn extract_insert_col_info(params: &serde_json::Value) -> Option<(i32, i32)> {
    let p: InsertColMutationParams = serde_json::from_value(params.clone()).ok()?;
    let insert_start = p.range.start_column;
    let insert_count = p.range.end_column - insert_start + 1;
    Some((insert_start, insert_count))
}

/// Extract remove col info from RemoveColMutationParams
///
/// Returns (remove_start, remove_end) or None if parsing fails
pub fn extract_remove_col_info(params: &serde_json::Value) -> Option<(i32, i32)> {
    let p: RemoveColMutationParams = serde_json::from_value(params.clone()).ok()?;
    Some((p.range.start_column, p.range.end_column))
}

// ============================================================================
// Macros for common transform patterns (optional, for advanced use)
// ============================================================================

/// Macro to implement HasWorksheetParams for mutation params structs
///
/// # Example
/// ```ignore
/// impl_has_worksheet_params!(AddWorksheetMergeMutationParams, unit_id, sub_unit_id);
/// ```
#[macro_export]
macro_rules! impl_has_worksheet_params {
    ($type:ty, $unit_id_field:ident, $sub_unit_id_field:ident) => {
        impl $crate::utils::transform_factory::HasWorksheetParams for $type {
            fn unit_id(&self) -> &str {
                &self.$unit_id_field
            }
            fn sub_unit_id(&self) -> &str {
                &self.$sub_unit_id_field
            }
        }
    };
}

/// Macro to implement HasRanges for mutation params structs
///
/// # Example
/// ```ignore
/// impl_has_ranges!(AddWorksheetMergeMutationParams, ranges);
/// ```
#[macro_export]
macro_rules! impl_has_ranges {
    ($type:ty, $ranges_field:ident) => {
        impl $crate::utils::transform_factory::HasRanges for $type {
            fn ranges_mut(&mut self) -> &mut Vec<$crate::types::IRange> {
                &mut self.$ranges_field
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_extract_insert_row_info() {
        let params = json!({
            "unitId": "test",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 5,
                "endRow": 7,
                "startColumn": 0,
                "endColumn": 10
            }
        });

        let (start, count) = extract_insert_row_info(&params).unwrap();
        assert_eq!(start, 5);
        assert_eq!(count, 3); // 7 - 5 + 1 = 3
    }

    #[test]
    fn test_extract_remove_row_info() {
        let params = json!({
            "unitId": "test",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 3,
                "endRow": 5,
                "startColumn": 0,
                "endColumn": 10
            }
        });

        let (start, end) = extract_remove_row_info(&params).unwrap();
        assert_eq!(start, 3);
        assert_eq!(end, 5);
    }

    #[test]
    fn test_extract_insert_col_info() {
        let params = json!({
            "unitId": "test",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 0,
                "endRow": 10,
                "startColumn": 2,
                "endColumn": 4
            }
        });

        let (start, count) = extract_insert_col_info(&params).unwrap();
        assert_eq!(start, 2);
        assert_eq!(count, 3); // 4 - 2 + 1 = 3
    }
}
