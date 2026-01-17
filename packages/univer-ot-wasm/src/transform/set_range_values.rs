use crate::transform::mutation_transform::MutationTransform;
use crate::types::{
    InsertColMutationParams, InsertRowMutationParams, MutationInfoInternal,
    ObjectMatrixPrimitiveType, Range, RemoveColMutationParams, RemoveRowsMutationParams,
    SetRangeValuesMutationParams, SubUnitParams, TransformResultInternal,
};
use serde_json;

#[derive(Default)]
pub struct SetRangeValuesTransform;

fn shift_rows_for_insert(
    cell_value: &mut ObjectMatrixPrimitiveType,
    insert_start: u32,
    insert_count: u32,
) {
    let mut new_data = serde_json::Map::new();
    for (row_key, row_value) in cell_value.data.iter() {
        if let Ok(row_num) = row_key.parse::<u32>() {
            if row_num >= insert_start {
                let new_key = (row_num + insert_count).to_string();
                new_data.insert(new_key, row_value.clone());
            } else {
                new_data.insert(row_key.clone(), row_value.clone());
            }
        } else {
            new_data.insert(row_key.clone(), row_value.clone());
        }
    }
    cell_value.data = new_data;
}

fn shift_rows_for_remove(
    cell_value: &mut ObjectMatrixPrimitiveType,
    remove_start: u32,
    remove_end: u32,
) {
    let remove_count = remove_end - remove_start + 1;
    let mut new_data = serde_json::Map::new();
    for (row_key, row_value) in cell_value.data.iter() {
        if let Ok(row_num) = row_key.parse::<u32>() {
            if row_num > remove_end {
                let new_key = (row_num - remove_count).to_string();
                new_data.insert(new_key, row_value.clone());
            } else if row_num < remove_start {
                new_data.insert(row_key.clone(), row_value.clone());
            }
            // Rows in [remove_start, remove_end] are removed
        } else {
            new_data.insert(row_key.clone(), row_value.clone());
        }
    }
    cell_value.data = new_data;
}

fn shift_cols_for_insert(
    cell_value: &mut ObjectMatrixPrimitiveType,
    insert_start: u32,
    insert_count: u32,
) {
    let mut new_data = serde_json::Map::new();
    for (row_key, row_value) in cell_value.data.iter() {
        if let serde_json::Value::Object(cols) = row_value {
            let mut new_row = serde_json::Map::new();
            for (col_key, col_value) in cols.iter() {
                if let Ok(col_num) = col_key.parse::<u32>() {
                    if col_num >= insert_start {
                        let new_key = (col_num + insert_count).to_string();
                        new_row.insert(new_key, col_value.clone());
                    } else {
                        new_row.insert(col_key.clone(), col_value.clone());
                    }
                } else {
                    new_row.insert(col_key.clone(), col_value.clone());
                }
            }
            if !new_row.is_empty() {
                new_data.insert(row_key.clone(), serde_json::Value::Object(new_row));
            }
        } else {
            new_data.insert(row_key.clone(), row_value.clone());
        }
    }
    cell_value.data = new_data;
}

fn shift_cols_for_remove(
    cell_value: &mut ObjectMatrixPrimitiveType,
    remove_start: u32,
    remove_end: u32,
) {
    let remove_count = remove_end - remove_start + 1;
    let mut new_data = serde_json::Map::new();
    for (row_key, row_value) in cell_value.data.iter() {
        if let serde_json::Value::Object(cols) = row_value {
            let mut new_row = serde_json::Map::new();
            for (col_key, col_value) in cols.iter() {
                if let Ok(col_num) = col_key.parse::<u32>() {
                    if col_num > remove_end {
                        let new_key = (col_num - remove_count).to_string();
                        new_row.insert(new_key, col_value.clone());
                    } else if col_num < remove_start {
                        new_row.insert(col_key.clone(), col_value.clone());
                    }
                    // Columns in [remove_start, remove_end] are removed
                } else {
                    new_row.insert(col_key.clone(), col_value.clone());
                }
            }
            if !new_row.is_empty() {
                new_data.insert(row_key.clone(), serde_json::Value::Object(new_row));
            }
        } else {
            new_data.insert(row_key.clone(), row_value.clone());
        }
    }
    cell_value.data = new_data;
}

impl MutationTransform for SetRangeValuesTransform {
    fn mutation_id() -> &'static str {
        "sheet.mutation.set-range-values"
    }

    /// Transform SetRangeValues against another SetRangeValues
    ///
    /// Implements Last-Writer-Wins (LWW) conflict resolution strategy:
    /// - When two operations modify the same cell, the one accepted by server first (m2) wins
    /// - The later operation (m1) sets conflicting cells to null to indicate conflict resolution
    ///
    /// This ensures OT consistency property:
    /// $State + m1 + m2' = $State + m2 + m1'
    ///
    /// Example:
    /// - m1: SetRangeValues(A1, "World")  // Client B's operation
    /// - m2: SetRangeValues(A1, "Hello")  // Client A's operation (arrived at server first)
    /// - Result: m1' sets A1 = null (conflict), m2' unchanged
    /// - Server state: $State + m2 + m1' = A1 = "Hello" (m2 wins)
    /// - Both clients end up with A1 = "Hello"
    fn transform_with_set_range_values(
        &self,
        m1: &MutationInfoInternal,
        m2: &MutationInfoInternal,
    ) -> TransformResultInternal {
        let mut m1_params: SetRangeValuesMutationParams =
            serde_json::from_value(m1.params.clone()).unwrap();
        let m2_params: SetRangeValuesMutationParams =
            serde_json::from_value(m2.params.clone()).unwrap();

        // Check if they're on the same sheet
        if m1_params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id
            || m1_params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id
        {
            // Different sheets, no conflict
            return TransformResultInternal {
                m1_prime: m1.clone(),
                m2_prime: m2.clone(),
                error: None,
            };
        }

        // LWW (Last-Writer-Wins) conflict resolution:
        // m2 was accepted by the server first (higher priority)
        // m1 needs to set conflicting cells to null (instead of removing them)

        // If both have cell_value, check for conflicts
        if let (Some(ref mut m1_cell_value), Some(ref m2_cell_value)) =
            (&mut m1_params.cell_value, &m2_params.cell_value)
        {
            // Set conflicting cells to null in m1_prime
            // A cell conflicts if both m1 and m2 modify it
            let mut new_data = serde_json::Map::new();

            for (row_key, row_value) in m1_cell_value.data.iter() {
                if let serde_json::Value::Object(m1_cols) = row_value {
                    let mut new_row = serde_json::Map::new();

                    for (col_key, col_value) in m1_cols.iter() {
                        // Check if m2 also modifies this cell
                        let has_conflict = if let Some(serde_json::Value::Object(m2_cols)) =
                            m2_cell_value.data.get(row_key)
                        {
                            m2_cols.contains_key(col_key)
                        } else {
                            false
                        };

                        if has_conflict {
                            // Set conflicting cell to null
                            new_row.insert(col_key.clone(), serde_json::Value::Null);
                        } else {
                            // Keep non-conflicting cell as-is
                            new_row.insert(col_key.clone(), col_value.clone());
                        }
                    }

                    // Always add the row if it has any cells
                    if !new_row.is_empty() {
                        new_data.insert(row_key.clone(), serde_json::Value::Object(new_row));
                    }
                }
            }

            m1_cell_value.data = new_data;
        }

        TransformResultInternal {
            m1_prime: MutationInfoInternal {
                id: m1.id.clone(),
                params: serde_json::to_value(m1_params).unwrap(),
            },
            m2_prime: m2.clone(),
            error: None,
        }
    }

    fn transform_with_insert_row(
        &self,
        m1: &MutationInfoInternal,
        m2: &MutationInfoInternal,
    ) -> TransformResultInternal {
        let mut m1_params: SetRangeValuesMutationParams = serde_json::from_value(m1.params.clone())
            .unwrap_or_else(|_| SetRangeValuesMutationParams {
                sub_unit_params: SubUnitParams {
                    unit_id: "".to_string(),
                    sub_unit_id: "".to_string(),
                },
                cell_value: None,
            });

        let m2_params: InsertRowMutationParams = serde_json::from_value(m2.params.clone())
            .unwrap_or_else(|_| InsertRowMutationParams {
                sub_unit_params: SubUnitParams {
                    unit_id: "".to_string(),
                    sub_unit_id: "".to_string(),
                },
                range: Range {
                    start_row: 0,
                    start_column: 0,
                    end_row: 0,
                    end_column: 0,
                },
                row_info: None,
            });

        if m1_params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id
            || m1_params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id
        {
            return TransformResultInternal {
                m1_prime: m1.clone(),
                m2_prime: m2.clone(),
                error: None,
            };
        }

        let insert_row = m2_params.range.start_row;
        let insert_count = m2_params.range.end_row - m2_params.range.start_row + 1;

        if let Some(ref mut cell_value) = m1_params.cell_value {
            shift_rows_for_insert(cell_value, insert_row, insert_count);
        }

        TransformResultInternal {
            m1_prime: MutationInfoInternal {
                id: m1.id.clone(),
                params: serde_json::to_value(m1_params).unwrap(),
            },
            m2_prime: m2.clone(),
            error: None,
        }
    }

    fn transform_with_insert_col(
        &self,
        m1: &MutationInfoInternal,
        m2: &MutationInfoInternal,
    ) -> TransformResultInternal {
        let mut m1_params: SetRangeValuesMutationParams = serde_json::from_value(m1.params.clone())
            .unwrap_or_else(|_| SetRangeValuesMutationParams {
                sub_unit_params: SubUnitParams {
                    unit_id: "".to_string(),
                    sub_unit_id: "".to_string(),
                },
                cell_value: None,
            });

        let m2_params: InsertColMutationParams = serde_json::from_value(m2.params.clone())
            .unwrap_or_else(|_| InsertColMutationParams {
                sub_unit_params: SubUnitParams {
                    unit_id: "".to_string(),
                    sub_unit_id: "".to_string(),
                },
                range: Range {
                    start_row: 0,
                    start_column: 0,
                    end_row: 0,
                    end_column: 0,
                },
                col_info: None,
            });

        if m1_params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id
            || m1_params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id
        {
            return TransformResultInternal {
                m1_prime: m1.clone(),
                m2_prime: m2.clone(),
                error: None,
            };
        }

        let insert_col = m2_params.range.start_column;
        let insert_count = m2_params.range.end_column - m2_params.range.start_column + 1;

        if let Some(ref mut cell_value) = m1_params.cell_value {
            shift_cols_for_insert(cell_value, insert_col, insert_count);
        }

        TransformResultInternal {
            m1_prime: MutationInfoInternal {
                id: m1.id.clone(),
                params: serde_json::to_value(m1_params).unwrap(),
            },
            m2_prime: m2.clone(),
            error: None,
        }
    }

    fn transform_with_remove_rows(
        &self,
        m1: &MutationInfoInternal,
        m2: &MutationInfoInternal,
    ) -> TransformResultInternal {
        let mut m1_params: SetRangeValuesMutationParams = serde_json::from_value(m1.params.clone())
            .unwrap_or_else(|_| SetRangeValuesMutationParams {
                sub_unit_params: SubUnitParams {
                    unit_id: "".to_string(),
                    sub_unit_id: "".to_string(),
                },
                cell_value: None,
            });

        let m2_params: RemoveRowsMutationParams = serde_json::from_value(m2.params.clone())
            .unwrap_or_else(|_| RemoveRowsMutationParams {
                sub_unit_params: SubUnitParams {
                    unit_id: "".to_string(),
                    sub_unit_id: "".to_string(),
                },
                range: Range {
                    start_row: 0,
                    start_column: 0,
                    end_row: 0,
                    end_column: 0,
                },
            });

        if m1_params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id
            || m1_params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id
        {
            return TransformResultInternal {
                m1_prime: m1.clone(),
                m2_prime: m2.clone(),
                error: None,
            };
        }

        let remove_start = m2_params.range.start_row;
        let remove_end = m2_params.range.end_row;

        if let Some(ref mut cell_value) = m1_params.cell_value {
            shift_rows_for_remove(cell_value, remove_start, remove_end);
        }

        TransformResultInternal {
            m1_prime: MutationInfoInternal {
                id: m1.id.clone(),
                params: serde_json::to_value(m1_params).unwrap(),
            },
            m2_prime: m2.clone(),
            error: None,
        }
    }

    fn transform_with_remove_col(
        &self,
        m1: &MutationInfoInternal,
        m2: &MutationInfoInternal,
    ) -> TransformResultInternal {
        let mut m1_params: SetRangeValuesMutationParams = serde_json::from_value(m1.params.clone())
            .unwrap_or_else(|_| SetRangeValuesMutationParams {
                sub_unit_params: SubUnitParams {
                    unit_id: "".to_string(),
                    sub_unit_id: "".to_string(),
                },
                cell_value: None,
            });

        let m2_params: RemoveColMutationParams = serde_json::from_value(m2.params.clone())
            .unwrap_or_else(|_| RemoveColMutationParams {
                sub_unit_params: SubUnitParams {
                    unit_id: "".to_string(),
                    sub_unit_id: "".to_string(),
                },
                range: Range {
                    start_row: 0,
                    start_column: 0,
                    end_row: 0,
                    end_column: 0,
                },
            });

        if m1_params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id
            || m1_params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id
        {
            return TransformResultInternal {
                m1_prime: m1.clone(),
                m2_prime: m2.clone(),
                error: None,
            };
        }

        let remove_start = m2_params.range.start_column;
        let remove_end = m2_params.range.end_column;

        if let Some(ref mut cell_value) = m1_params.cell_value {
            shift_cols_for_remove(cell_value, remove_start, remove_end);
        }

        TransformResultInternal {
            m1_prime: MutationInfoInternal {
                id: m1.id.clone(),
                params: serde_json::to_value(m1_params).unwrap(),
            },
            m2_prime: m2.clone(),
            error: None,
        }
    }
}
