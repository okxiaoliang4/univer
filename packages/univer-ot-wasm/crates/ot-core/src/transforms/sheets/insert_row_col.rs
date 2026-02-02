//! Insert row and insert column transforms
//!
//! When two insert operations occur concurrently:
//! - If they're at the same position: m2 is shifted (m2 happens after m1)
//! - If m2 is after m1's position: m2 is shifted by m1's count
//! - Otherwise: no shift needed (independent operations)

use crate::mutations::sheets::{InsertRowMutation, InsertColMutation, InsertRowMutationParams, InsertColMutationParams};
use crate::registry::{MutationId, TransformFnRef, TransformRegistry};
use crate::types::{MutationInfo, MutationOutcome, TransformResultRef};
use crate::utils::params::same_worksheet;
use std::sync::Arc;

pub const INSERT_ROW_ID: MutationId = InsertRowMutation::ID;
pub const INSERT_COL_ID: MutationId = InsertColMutation::ID;

pub fn register_transforms(registry: &mut TransformRegistry) {
    registry.register_symmetric_ref(INSERT_ROW_ID, create_insert_row_symmetric());
    registry.register_symmetric_ref(INSERT_COL_ID, create_insert_col_symmetric());

    // NOTE: No register_identity or register_cross_module_transforms needed
    // The registry automatically falls back to identity when no transform is found
}

/// Symmetric transform for insert-row vs insert-row
fn create_insert_row_symmetric() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        // Early worksheet check
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return TransformResultRef::identity(m1, m2);
        }

        // Parse params
        let m1_params: Result<InsertRowMutationParams, _> =
            serde_json::from_value(m1.params.clone());
        let m2_params: Result<InsertRowMutationParams, _> =
            serde_json::from_value(m2.params.clone());

        if let (Ok(m1_p), Ok(m2_p)) = (m1_params, m2_params) {
            let m1_start = m1_p.range.start_row;
            let m1_count = m1_p.range.end_row - m1_start + 1;
            let m2_start = m2_p.range.start_row;

            // If m2's insert position is at or after m1's, shift m2
            if m2_start >= m1_start {
                let mut m2_prime = m2_p.clone();
                m2_prime.range.start_row += m1_count;
                m2_prime.range.end_row += m1_count;

                return TransformResultRef {
                    m1_prime: MutationOutcome::Unchanged(m1),
                    m2_prime: MutationOutcome::Modified(MutationInfo {
                        id: m2.id.clone(),
                        params: serde_json::to_value(m2_prime).unwrap_or_else(|_| m2.params.clone()),
                    }),
                    error: None,
                };
            }
        }

        // If parsing failed or no shift needed, return identity
        TransformResultRef::identity(m1, m2)
    })
}

/// Symmetric transform for insert-col vs insert-col
fn create_insert_col_symmetric() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        // Early worksheet check
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return TransformResultRef::identity(m1, m2);
        }

        // Parse params
        let m1_params: Result<InsertColMutationParams, _> =
            serde_json::from_value(m1.params.clone());
        let m2_params: Result<InsertColMutationParams, _> =
            serde_json::from_value(m2.params.clone());

        if let (Ok(m1_p), Ok(m2_p)) = (m1_params, m2_params) {
            let m1_start = m1_p.range.start_column;
            let m1_count = m1_p.range.end_column - m1_start + 1;
            let m2_start = m2_p.range.start_column;

            // If m2's insert position is at or after m1's, shift m2
            if m2_start >= m1_start {
                let mut m2_prime = m2_p.clone();
                m2_prime.range.start_column += m1_count;
                m2_prime.range.end_column += m1_count;

                return TransformResultRef {
                    m1_prime: MutationOutcome::Unchanged(m1),
                    m2_prime: MutationOutcome::Modified(MutationInfo {
                        id: m2.id.clone(),
                        params: serde_json::to_value(m2_prime).unwrap_or_else(|_| m2.params.clone()),
                    }),
                    error: None,
                };
            }
        }

        // If parsing failed or no shift needed, return identity
        TransformResultRef::identity(m1, m2)
    })
}
