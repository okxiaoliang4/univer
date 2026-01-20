use crate::transform::mutation_transform::MutationTransform;
use crate::mutations::types::{
    InsertColMutationParams, InsertRowMutationParams, RemoveColMutationParams,
    RemoveRowsMutationParams, SetRangeValuesMutationParams,
};
use crate::types::{
    MutationInfoInternal, ObjectMatrixPrimitiveType, Range, SubUnitParams, TransformResultInternal,
};
use crate::{wasm_log_debug, wasm_log_warn};
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
    wasm_log_debug!(
        "set_range_values shift_cols_for_insert start insert_start={} insert_count={} rows={}",
        insert_start,
        insert_count,
        cell_value.data.len()
    );

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
    /// - When two operations modify the same cell, the later operation (m1) wins
    /// - m2' is kept unchanged (applied first)
    /// - m1' removes cells that conflict with m2, keeping only m1's unique changes
    ///
    /// This ensures OT consistency property:
    /// $State + m2' + m1' = $State + m1 + m2' where m1 wins on conflicts
    ///
    /// Example:
    /// - m1: SetRangeValues(A1=3, A2=2)  // Later operation
    /// - m2: SetRangeValues(A1=1)        // Earlier operation (arrived first)
    /// - Result:
    ///   - m2' = SetRangeValues(A1=1)    // unchanged
    ///   - m1' = SetRangeValues(A1=3, A2=2)  // keeps all, will overwrite A1
    /// - Final state: A1=3 (m1 wins), A2=2 (m1 only)
    fn transform_with_set_range_values(
        &self,
        m1: &MutationInfoInternal,
        m2: &MutationInfoInternal,
    ) -> TransformResultInternal {
        let m1_params: SetRangeValuesMutationParams =
            serde_json::from_value(m1.params.clone()).unwrap_or_else(|_| {
                wasm_log_warn!("set_range_values transform set_range_values: invalid m1 params");
                SetRangeValuesMutationParams {
                    sub_unit_params: SubUnitParams {
                        unit_id: "".to_string(),
                        sub_unit_id: "".to_string(),
                    },
                    cell_value: None,
                }
            });

        let m2_params: SetRangeValuesMutationParams =
            serde_json::from_value(m2.params.clone()).unwrap_or_else(|_| {
                wasm_log_warn!("set_range_values transform set_range_values: invalid m2 params");
                SetRangeValuesMutationParams {
                    sub_unit_params: SubUnitParams {
                        unit_id: "".to_string(),
                        sub_unit_id: "".to_string(),
                    },
                    cell_value: None,
                }
            });

        // Check if they're on the same sheet
        if m1_params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id
            || m1_params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id
        {
            // Different sheets, no conflict
            return TransformResultInternal {
                m1_prime: Some(m1.clone()),
                m2_prime: Some(m2.clone()),
                error: None,
            };
        }

        // LWW (Last-Writer-Wins) conflict resolution:
        // m2' keeps only cells that do NOT conflict with m1
        // m1' keeps all its values (applied second, overwrites conflicts)

        wasm_log_debug!(
            "set_range_values transform set_range_values m1_cells={} m2_cells={}",
            m1_params.cell_value.as_ref().map(|c| c.data.len()).unwrap_or(0),
            m2_params.cell_value.as_ref().map(|c| c.data.len()).unwrap_or(0)
        );

        let mut m2_prime = Some(m2.clone());
        if let (Some(m1_value), Some(m2_value)) = (
            m1_params.cell_value.as_ref(),
            m2_params.cell_value.as_ref(),
        ) {
            let mut new_data = serde_json::Map::new();
            for (row_key, row_value) in m2_value.data.iter() {
                if let serde_json::Value::Object(m2_cols) = row_value {
                    let mut new_row = serde_json::Map::new();
                    let m1_row = m1_value.data.get(row_key);
                    for (col_key, col_value) in m2_cols.iter() {
                        let has_conflict = match m1_row {
                            Some(serde_json::Value::Object(m1_cols)) => m1_cols.contains_key(col_key),
                            _ => false,
                        };
                        if !has_conflict {
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

            if new_data.is_empty() {
                m2_prime = None;
            } else {
                let m2_prime_params = SetRangeValuesMutationParams {
                    sub_unit_params: m2_params.sub_unit_params.clone(),
                    cell_value: Some(ObjectMatrixPrimitiveType { data: new_data }),
                };
                m2_prime = Some(MutationInfoInternal {
                    id: m2.id.clone(),
                    params: serde_json::to_value(m2_prime_params).unwrap(),
                });
            }
        }

        TransformResultInternal {
            m1_prime: Some(m1.clone()),
            m2_prime,
            error: None,
        }
    }

    fn transform_with_insert_row(
        &self,
        m1: &MutationInfoInternal,
        m2: &MutationInfoInternal,
    ) -> TransformResultInternal {
        let mut m1_params: SetRangeValuesMutationParams = serde_json::from_value(m1.params.clone())
            .unwrap_or_else(|_| {
                wasm_log_warn!("set_range_values transform insert_row: invalid m1 params");
                SetRangeValuesMutationParams {
                    sub_unit_params: SubUnitParams {
                        unit_id: "".to_string(),
                        sub_unit_id: "".to_string(),
                    },
                    cell_value: None,
                }
            });

        let m2_params: InsertRowMutationParams = serde_json::from_value(m2.params.clone())
            .unwrap_or_else(|_| {
                wasm_log_warn!("set_range_values transform insert_row: invalid m2 params");
                InsertRowMutationParams {
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
                }
            });

        if m1_params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id
            || m1_params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id
        {
            return TransformResultInternal {
                m1_prime: Some(m1.clone()),
                m2_prime: Some(m2.clone()),
                error: None,
            };
        }

        let insert_row = m2_params.range.start_row;
        let insert_count = m2_params.range.end_row - m2_params.range.start_row + 1;

        wasm_log_debug!(
            "set_range_values transform insert_row insert_row={} insert_count={} has_cell_value={}",
            insert_row,
            insert_count,
            m1_params.cell_value.is_some()
        );

        if let Some(ref mut cell_value) = m1_params.cell_value {
            shift_rows_for_insert(cell_value, insert_row, insert_count);
        }

        TransformResultInternal {
            m1_prime: Some(MutationInfoInternal {
                id: m1.id.clone(),
                params: serde_json::to_value(m1_params).unwrap(),
            }),
            m2_prime: Some(m2.clone()),
            error: None,
        }
    }

    fn transform_with_insert_col(
        &self,
        m1: &MutationInfoInternal,
        m2: &MutationInfoInternal,
    ) -> TransformResultInternal {
        let mut m1_params: SetRangeValuesMutationParams = serde_json::from_value(m1.params.clone())
            .unwrap_or_else(|_| {
                wasm_log_warn!("set_range_values transform insert_col: invalid m1 params");
                SetRangeValuesMutationParams {
                    sub_unit_params: SubUnitParams {
                        unit_id: "".to_string(),
                        sub_unit_id: "".to_string(),
                    },
                    cell_value: None,
                }
            });

        let m2_params: InsertColMutationParams = serde_json::from_value(m2.params.clone())
            .unwrap_or_else(|_| {
                wasm_log_warn!("set_range_values transform insert_col: invalid m2 params");
                InsertColMutationParams {
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
                }
            });

        if m1_params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id
            || m1_params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id
        {
            return TransformResultInternal {
                m1_prime: Some(m1.clone()),
                m2_prime: Some(m2.clone()),
                error: None,
            };
        }

        let insert_col = m2_params.range.start_column;
        let insert_count = m2_params.range.end_column - m2_params.range.start_column + 1;

        wasm_log_debug!(
            "set_range_values transform insert_col insert_col={} insert_count={} has_cell_value={}",
            insert_col,
            insert_count,
            m1_params.cell_value.is_some()
        );

        if let Some(ref mut cell_value) = m1_params.cell_value {
            shift_cols_for_insert(cell_value, insert_col, insert_count);
        }

        TransformResultInternal {
            m1_prime: Some(MutationInfoInternal {
                id: m1.id.clone(),
                params: serde_json::to_value(m1_params).unwrap(),
            }),
            m2_prime: Some(m2.clone()),
            error: None,
        }
    }

    fn transform_with_remove_rows(
        &self,
        m1: &MutationInfoInternal,
        m2: &MutationInfoInternal,
    ) -> TransformResultInternal {
        let mut m1_params: SetRangeValuesMutationParams = serde_json::from_value(m1.params.clone())
            .unwrap_or_else(|_| {
                wasm_log_warn!("set_range_values transform remove_rows: invalid m1 params");
                SetRangeValuesMutationParams {
                    sub_unit_params: SubUnitParams {
                        unit_id: "".to_string(),
                        sub_unit_id: "".to_string(),
                    },
                    cell_value: None,
                }
            });

        let m2_params: RemoveRowsMutationParams = serde_json::from_value(m2.params.clone())
            .unwrap_or_else(|_| {
                wasm_log_warn!("set_range_values transform remove_rows: invalid m2 params");
                RemoveRowsMutationParams {
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
                }
            });

        if m1_params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id
            || m1_params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id
        {
            return TransformResultInternal {
                m1_prime: Some(m1.clone()),
                m2_prime: Some(m2.clone()),
                error: None,
            };
        }

        let remove_start = m2_params.range.start_row;
        let remove_end = m2_params.range.end_row;

        wasm_log_debug!(
            "set_range_values transform remove_rows remove_start={} remove_end={} has_cell_value={}",
            remove_start,
            remove_end,
            m1_params.cell_value.is_some()
        );

        if let Some(ref mut cell_value) = m1_params.cell_value {
            shift_rows_for_remove(cell_value, remove_start, remove_end);
        }

        TransformResultInternal {
            m1_prime: Some(MutationInfoInternal {
                id: m1.id.clone(),
                params: serde_json::to_value(m1_params).unwrap(),
            }),
            m2_prime: Some(m2.clone()),
            error: None,
        }
    }

    fn transform_with_remove_col(
        &self,
        m1: &MutationInfoInternal,
        m2: &MutationInfoInternal,
    ) -> TransformResultInternal {
        let mut m1_params: SetRangeValuesMutationParams = serde_json::from_value(m1.params.clone())
            .unwrap_or_else(|_| {
                wasm_log_warn!("set_range_values transform remove_col: invalid m1 params");
                SetRangeValuesMutationParams {
                    sub_unit_params: SubUnitParams {
                        unit_id: "".to_string(),
                        sub_unit_id: "".to_string(),
                    },
                    cell_value: None,
                }
            });

        let m2_params: RemoveColMutationParams = serde_json::from_value(m2.params.clone())
            .unwrap_or_else(|_| {
                wasm_log_warn!("set_range_values transform remove_col: invalid m2 params");
                RemoveColMutationParams {
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
                }
            });

        if m1_params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id
            || m1_params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id
        {
            return TransformResultInternal {
                m1_prime: Some(m1.clone()),
                m2_prime: Some(m2.clone()),
                error: None,
            };
        }

        let remove_start = m2_params.range.start_column;
        let remove_end = m2_params.range.end_column;

        wasm_log_debug!(
            "set_range_values transform remove_col remove_start={} remove_end={} has_cell_value={}",
            remove_start,
            remove_end,
            m1_params.cell_value.is_some()
        );

        if let Some(ref mut cell_value) = m1_params.cell_value {
            shift_cols_for_remove(cell_value, remove_start, remove_end);
        }

        TransformResultInternal {
            m1_prime: Some(MutationInfoInternal {
                id: m1.id.clone(),
                params: serde_json::to_value(m1_params).unwrap(),
            }),
            m2_prime: Some(m2.clone()),
            error: None,
        }
    }

    fn compose(
        &self,
        m1: &MutationInfoInternal,
        m2: &MutationInfoInternal,
    ) -> Vec<MutationInfoInternal> {
        let m1_params: SetRangeValuesMutationParams = serde_json::from_value(m1.params.clone())
            .unwrap_or_else(|_| SetRangeValuesMutationParams {
                sub_unit_params: SubUnitParams {
                    unit_id: "".to_string(),
                    sub_unit_id: "".to_string(),
                },
                cell_value: None,
            });

        let m2_params: SetRangeValuesMutationParams = serde_json::from_value(m2.params.clone())
            .unwrap_or_else(|_| SetRangeValuesMutationParams {
                sub_unit_params: SubUnitParams {
                    unit_id: "".to_string(),
                    sub_unit_id: "".to_string(),
                },
                cell_value: None,
            });

        if m1_params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id
            || m1_params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id
        {
            return vec![m1.clone(), m2.clone()];
        }

        let merged_cell_value = match (m1_params.cell_value, m2_params.cell_value) {
            (None, None) => None,
            (Some(cell_value), None) => Some(cell_value),
            (None, Some(cell_value)) => Some(cell_value),
            (Some(m1_value), Some(m2_value)) => {
                let mut new_data = m1_value.data.clone();
                for (row_key, row_value) in m2_value.data.iter() {
                    match row_value {
                        serde_json::Value::Object(m2_cols) => {
                            let mut merged_row = match new_data.get(row_key) {
                                Some(serde_json::Value::Object(existing_cols)) => {
                                    existing_cols.clone()
                                }
                                _ => serde_json::Map::new(),
                            };
                            for (col_key, col_value) in m2_cols.iter() {
                                merged_row.insert(col_key.clone(), col_value.clone());
                            }
                            new_data.insert(row_key.clone(), serde_json::Value::Object(merged_row));
                        }
                        _ => {
                            new_data.insert(row_key.clone(), row_value.clone());
                        }
                    }
                }
                Some(ObjectMatrixPrimitiveType { data: new_data })
            }
        };

        let composed_params = SetRangeValuesMutationParams {
            sub_unit_params: m1_params.sub_unit_params,
            cell_value: merged_cell_value,
        };

        vec![MutationInfoInternal {
            id: m1.id.clone(),
            params: serde_json::to_value(composed_params).unwrap(),
        }]
    }
}
