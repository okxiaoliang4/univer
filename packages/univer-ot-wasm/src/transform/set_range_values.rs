use crate::types::{
    MutationInfo, TransformResult,
    SetRangeValuesMutationParams, InsertRowMutationParams,
    RemoveRowsMutationParams, SubUnitParams, Range,
};
#[cfg(any(test, feature = "server"))]
use crate::types::{MutationInfoInternal, TransformResultInternal};
use crate::transform::mutation_transform::MutationTransform;
#[cfg(any(test, feature = "server"))]
use serde_json;
use serde_wasm_bindgen;

#[derive(Default)]
pub struct SetRangeValuesTransform;

impl MutationTransform for SetRangeValuesTransform {
    fn mutation_id() -> &'static str {
        "sheet.mutation.set-range-values"
    }

    fn transform_with_set_range_values(&self, m1: &MutationInfo, m2: &MutationInfo) -> TransformResult {
        // SetRangeValues with SetRangeValues: resolve conflicts
        // m2 is the server operation that was applied first, so m2 wins for conflicting cells
        // m1' should only contain cells that don't conflict with m2
        // m2' remains unchanged
        
        let mut m1_params: SetRangeValuesMutationParams = serde_wasm_bindgen::from_value(m1.params.clone())
            .unwrap_or_else(|_| SetRangeValuesMutationParams {
                sub_unit_params: SubUnitParams {
                    unit_id: "".to_string(),
                    sub_unit_id: "".to_string(),
                },
                cell_value: None,
            });

        let m2_params: SetRangeValuesMutationParams = serde_wasm_bindgen::from_value(m2.params.clone())
            .unwrap_or_else(|_| SetRangeValuesMutationParams {
                sub_unit_params: SubUnitParams {
                    unit_id: "".to_string(),
                    sub_unit_id: "".to_string(),
                },
                cell_value: None,
            });

        // Check if they're on the same sheet
        if m1_params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id ||
           m1_params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id {
            // Different sheets, no conflict
            return TransformResult {
                m1_prime: m1.clone(),
                m2_prime: m2.clone(),
                error: None,
            };
        }

        // Remove conflicting cells from m1
        // m2 wins for any cell that both operations modify
        if let (Some(ref mut m1_cell_value), Some(ref m2_cell_value)) = (&mut m1_params.cell_value, &m2_params.cell_value) {
            let mut new_data = serde_json::Map::new();
            
            for (row_key, row_value) in m1_cell_value.data.iter() {
                if let Some(m2_row) = m2_cell_value.data.get(row_key) {
                    // Both have this row, check each column
                    if let (serde_json::Value::Object(m1_cols), serde_json::Value::Object(m2_cols)) = (row_value, m2_row) {
                        let mut new_row = serde_json::Map::new();
                        for (col_key, col_value) in m1_cols.iter() {
                            if !m2_cols.contains_key(col_key) {
                                // m2 doesn't modify this cell, keep m1's value
                                new_row.insert(col_key.clone(), col_value.clone());
                            }
                            // If m2 also modifies this cell, m2 wins, so we drop m1's value
                        }
                        if !new_row.is_empty() {
                            new_data.insert(row_key.clone(), serde_json::Value::Object(new_row));
                        }
                    } else {
                        // Row value is not an object, keep as is
                        new_data.insert(row_key.clone(), row_value.clone());
                    }
                } else {
                    // m2 doesn't have this row, keep m1's entire row
                    new_data.insert(row_key.clone(), row_value.clone());
                }
            }
            
            m1_cell_value.data = new_data;
        }

        TransformResult {
            m1_prime: MutationInfo {
                id: m1.id.clone(),
                params: serde_wasm_bindgen::to_value(&m1_params).unwrap(),
            },
            m2_prime: m2.clone(),
            error: None,
        }
    }

    fn transform_with_insert_row(&self, m1: &MutationInfo, m2: &MutationInfo) -> TransformResult {
        let mut m1_params: SetRangeValuesMutationParams = serde_wasm_bindgen::from_value(m1.params.clone())
            .unwrap_or_else(|_| SetRangeValuesMutationParams {
                sub_unit_params: SubUnitParams {
                    unit_id: "".to_string(),
                    sub_unit_id: "".to_string(),
                },
                cell_value: None,
            });

        let m2_params: InsertRowMutationParams = serde_wasm_bindgen::from_value(m2.params.clone())
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

        let insert_row = m2_params.range.start_row;

        // Adjust cell positions: if row >= insert_row, shift down by 1
        if let Some(ref mut cell_value) = m1_params.cell_value {
            let mut new_data = serde_json::Map::new();
            for (row_key, row_value) in cell_value.data.iter() {
                if let Ok(row_num) = row_key.parse::<u32>() {
                    if row_num >= insert_row {
                        let new_key = (row_num + 1).to_string();
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

        TransformResult {
            m1_prime: MutationInfo {
                id: m1.id.clone(),
                params: serde_wasm_bindgen::to_value(&m1_params).unwrap(),
            },
            m2_prime: m2.clone(),
            error: None,
        }
    }

    fn transform_with_insert_col(&self, m1: &MutationInfo, m2: &MutationInfo) -> TransformResult {
        // Similar to insert_row but for columns
        // For now, return identity transform as column transformation logic needs to be implemented
        TransformResult {
            m1_prime: m1.clone(),
            m2_prime: m2.clone(),
            error: None,
        }
    }

    fn transform_with_remove_rows(&self, m1: &MutationInfo, m2: &MutationInfo) -> TransformResult {
        let mut m1_params: SetRangeValuesMutationParams = serde_wasm_bindgen::from_value(m1.params.clone())
            .unwrap_or_else(|_| SetRangeValuesMutationParams {
                sub_unit_params: SubUnitParams {
                    unit_id: "".to_string(),
                    sub_unit_id: "".to_string(),
                },
                cell_value: None,
            });

        let m2_params: RemoveRowsMutationParams = serde_wasm_bindgen::from_value(m2.params.clone())
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

        let remove_start = m2_params.range.start_row;
        let remove_end = m2_params.range.end_row;
        let remove_count = remove_end - remove_start + 1;

        // Adjust cell positions: if row > remove_end, shift up by remove_count
        if let Some(ref mut cell_value) = m1_params.cell_value {
            let mut new_data = serde_json::Map::new();
            for (row_key, row_value) in cell_value.data.iter() {
                if let Ok(row_num) = row_key.parse::<u32>() {
                    if row_num > remove_end {
                        let new_key = (row_num - remove_count).to_string();
                        new_data.insert(new_key, row_value.clone());
                    } else if row_num < remove_start {
                        new_data.insert(row_key.clone(), row_value.clone());
                    }
                    // Rows in [remove_start, remove_end] are removed, so skip them
                } else {
                    new_data.insert(row_key.clone(), row_value.clone());
                }
            }
            cell_value.data = new_data;
        }

        TransformResult {
            m1_prime: MutationInfo {
                id: m1.id.clone(),
                params: serde_wasm_bindgen::to_value(&m1_params).unwrap(),
            },
            m2_prime: m2.clone(),
            error: None,
        }
    }

    fn transform_with_remove_col(&self, m1: &MutationInfo, m2: &MutationInfo) -> TransformResult {
        // Similar to remove_rows but for columns
        // For now, return identity transform as column transformation logic needs to be implemented
        TransformResult {
            m1_prime: m1.clone(),
            m2_prime: m2.clone(),
            error: None,
        }
    }

    // Internal methods that work with MutationInfoInternal (for Rust unit tests and server)
    #[cfg(any(test, feature = "server"))]
    fn transform_internal_with_set_range_values(&self, m1: &MutationInfoInternal, m2: &MutationInfoInternal) -> TransformResultInternal {
        // SetRangeValues with SetRangeValues: resolve conflicts
        // m2 is the server operation that was applied first, so m2 wins for conflicting cells
        
        let mut m1_params: SetRangeValuesMutationParams = serde_json::from_value(m1.params.clone())
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

        // Check if they're on the same sheet
        if m1_params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id ||
           m1_params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id {
            // Different sheets, no conflict
            return TransformResultInternal {
                m1_prime: m1.clone(),
                m2_prime: m2.clone(),
                error: None,
            };
        }

        // Remove conflicting cells from m1
        if let (Some(ref mut m1_cell_value), Some(ref m2_cell_value)) = (&mut m1_params.cell_value, &m2_params.cell_value) {
            let mut new_data = serde_json::Map::new();
            
            for (row_key, row_value) in m1_cell_value.data.iter() {
                if let Some(m2_row) = m2_cell_value.data.get(row_key) {
                    if let (serde_json::Value::Object(m1_cols), serde_json::Value::Object(m2_cols)) = (row_value, m2_row) {
                        let mut new_row = serde_json::Map::new();
                        for (col_key, col_value) in m1_cols.iter() {
                            if !m2_cols.contains_key(col_key) {
                                new_row.insert(col_key.clone(), col_value.clone());
                            }
                        }
                        if !new_row.is_empty() {
                            new_data.insert(row_key.clone(), serde_json::Value::Object(new_row));
                        }
                    } else {
                        new_data.insert(row_key.clone(), row_value.clone());
                    }
                } else {
                    new_data.insert(row_key.clone(), row_value.clone());
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

    #[cfg(any(test, feature = "server"))]
    fn transform_internal_with_insert_row(&self, m1: &MutationInfoInternal, m2: &MutationInfoInternal) -> TransformResultInternal {
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

        let insert_row = m2_params.range.start_row;

        // Adjust cell positions: if row >= insert_row, shift down by 1
        if let Some(ref mut cell_value) = m1_params.cell_value {
            let mut new_data = serde_json::Map::new();
            for (row_key, row_value) in cell_value.data.iter() {
                if let Ok(row_num) = row_key.parse::<u32>() {
                    if row_num >= insert_row {
                        let new_key = (row_num + 1).to_string();
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

        TransformResultInternal {
            m1_prime: MutationInfoInternal {
                id: m1.id.clone(),
                params: serde_json::to_value(m1_params).unwrap(),
            },
            m2_prime: m2.clone(),
            error: None,
        }
    }

    #[cfg(any(test, feature = "server"))]
    fn transform_internal_with_insert_col(&self, m1: &MutationInfoInternal, m2: &MutationInfoInternal) -> TransformResultInternal {
        TransformResultInternal {
            m1_prime: m1.clone(),
            m2_prime: m2.clone(),
            error: None,
        }
    }

    #[cfg(any(test, feature = "server"))]
    fn transform_internal_with_remove_rows(&self, m1: &MutationInfoInternal, m2: &MutationInfoInternal) -> TransformResultInternal {
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

        let remove_start = m2_params.range.start_row;
        let remove_end = m2_params.range.end_row;
        let remove_count = remove_end - remove_start + 1;

        // Adjust cell positions: if row > remove_end, shift up by remove_count
        if let Some(ref mut cell_value) = m1_params.cell_value {
            let mut new_data = serde_json::Map::new();
            for (row_key, row_value) in cell_value.data.iter() {
                if let Ok(row_num) = row_key.parse::<u32>() {
                    if row_num > remove_end {
                        let new_key = (row_num - remove_count).to_string();
                        new_data.insert(new_key, row_value.clone());
                    } else if row_num < remove_start {
                        new_data.insert(row_key.clone(), row_value.clone());
                    }
                    // Rows in [remove_start, remove_end] are removed, so skip them
                } else {
                    new_data.insert(row_key.clone(), row_value.clone());
                }
            }
            cell_value.data = new_data;
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

    #[cfg(any(test, feature = "server"))]
    fn transform_internal_with_remove_col(&self, m1: &MutationInfoInternal, m2: &MutationInfoInternal) -> TransformResultInternal {
        TransformResultInternal {
            m1_prime: m1.clone(),
            m2_prime: m2.clone(),
            error: None,
        }
    }
}
