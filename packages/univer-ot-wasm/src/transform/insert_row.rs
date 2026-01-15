use crate::types::{
    MutationInfo, TransformResult,
    InsertRowMutationParams, RemoveRowsMutationParams,
    SubUnitParams, Range,
};
#[cfg(any(test, feature = "server"))]
use crate::types::{MutationInfoInternal, TransformResultInternal};
use crate::transform::mutation_transform::MutationTransform;
#[cfg(any(test, feature = "server"))]
use serde_json;
use serde_wasm_bindgen;

#[derive(Default)]
pub struct InsertRowTransform;

impl MutationTransform for InsertRowTransform {
    fn mutation_id() -> &'static str {
        "sheet.mutation.insert-row"
    }

    fn transform_with_set_range_values(&self, m1: &MutationInfo, m2: &MutationInfo) -> TransformResult {
        // Insert row doesn't need to change when set_range_values happens
        TransformResult {
            m1_prime: m1.clone(),
            m2_prime: m2.clone(),
            error: None,
        }
    }

    fn transform_with_insert_row(&self, m1: &MutationInfo, m2: &MutationInfo) -> TransformResult {
        let m1_params: InsertRowMutationParams = serde_wasm_bindgen::from_value(m1.params.clone())
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

        let mut m2_params: InsertRowMutationParams = serde_wasm_bindgen::from_value(m2.params.clone())
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

        let m1_row = m1_params.range.start_row;

        // If m2 inserts after m1, shift m2 down
        if m2_params.range.start_row > m1_row {
            m2_params.range.start_row += 1;
            m2_params.range.end_row += 1;
        }

        TransformResult {
            m1_prime: m1.clone(),
            m2_prime: MutationInfo {
                id: m2.id.clone(),
                params: serde_wasm_bindgen::to_value(&m2_params).unwrap(),
            },
            error: None,
        }
    }

    fn transform_with_insert_col(&self, m1: &MutationInfo, m2: &MutationInfo) -> TransformResult {
        // Insert row and insert col are independent
        TransformResult {
            m1_prime: m1.clone(),
            m2_prime: m2.clone(),
            error: None,
        }
    }

    fn transform_with_remove_rows(&self, m1: &MutationInfo, m2: &MutationInfo) -> TransformResult {
        let m1_params: InsertRowMutationParams = serde_wasm_bindgen::from_value(m1.params.clone())
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

        let insert_row = m1_params.range.start_row;
        let remove_end = m2_params.range.end_row;
        let remove_count = m2_params.range.end_row - m2_params.range.start_row + 1;

        // If insert_row is after removed rows, shift up
        let mut new_m1_params = m1_params.clone();
        if insert_row > remove_end {
            new_m1_params.range.start_row -= remove_count;
            new_m1_params.range.end_row -= remove_count;
        } else if insert_row >= m2_params.range.start_row {
            // Insert row is within removed range, this is a conflict
            return TransformResult {
                m1_prime: m1.clone(),
                m2_prime: m2.clone(),
                error: Some("Insert row conflicts with remove rows".to_string()),
            };
        }

        TransformResult {
            m1_prime: MutationInfo {
                id: m1.id.clone(),
                params: serde_wasm_bindgen::to_value(&new_m1_params).unwrap(),
            },
            m2_prime: m2.clone(),
            error: None,
        }
    }

    fn transform_with_remove_col(&self, m1: &MutationInfo, m2: &MutationInfo) -> TransformResult {
        // Insert row and remove col are independent
        TransformResult {
            m1_prime: m1.clone(),
            m2_prime: m2.clone(),
            error: None,
        }
    }

    // Internal methods that work with MutationInfoInternal (for Rust unit tests and server)
    #[cfg(any(test, feature = "server"))]
    fn transform_internal_with_set_range_values(&self, m1: &MutationInfoInternal, m2: &MutationInfoInternal) -> TransformResultInternal {
        TransformResultInternal {
            m1_prime: m1.clone(),
            m2_prime: m2.clone(),
            error: None,
        }
    }

    #[cfg(any(test, feature = "server"))]
    fn transform_internal_with_insert_row(&self, m1: &MutationInfoInternal, m2: &MutationInfoInternal) -> TransformResultInternal {
        let m1_params: InsertRowMutationParams = serde_json::from_value(m1.params.clone())
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

        let mut m2_params: InsertRowMutationParams = serde_json::from_value(m2.params.clone())
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

        let m1_row = m1_params.range.start_row;

        // If m2 inserts after m1, shift m2 down
        if m2_params.range.start_row > m1_row {
            m2_params.range.start_row += 1;
            m2_params.range.end_row += 1;
        }

        TransformResultInternal {
            m1_prime: m1.clone(),
            m2_prime: MutationInfoInternal {
                id: m2.id.clone(),
                params: serde_json::to_value(m2_params).unwrap(),
            },
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
        let m1_params: InsertRowMutationParams = serde_json::from_value(m1.params.clone())
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

        let insert_row = m1_params.range.start_row;
        let remove_end = m2_params.range.end_row;
        let remove_count = m2_params.range.end_row - m2_params.range.start_row + 1;

        // If insert_row is after removed rows, shift up
        let mut new_m1_params = m1_params.clone();
        if insert_row > remove_end {
            new_m1_params.range.start_row -= remove_count;
            new_m1_params.range.end_row -= remove_count;
        } else if insert_row >= m2_params.range.start_row {
            // Insert row is within removed range, this is a conflict
            return TransformResultInternal {
                m1_prime: m1.clone(),
                m2_prime: m2.clone(),
                error: Some("Insert row conflicts with remove rows".to_string()),
            };
        }

        TransformResultInternal {
            m1_prime: MutationInfoInternal {
                id: m1.id.clone(),
                params: serde_json::to_value(new_m1_params).unwrap(),
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
