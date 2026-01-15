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
pub struct RemoveRowsTransform;

impl MutationTransform for RemoveRowsTransform {
    fn mutation_id() -> &'static str {
        "sheet.mutation.remove-rows"
    }

    fn transform_with_set_range_values(&self, m1: &MutationInfo, m2: &MutationInfo) -> TransformResult {
        // Remove rows doesn't need to change when set_range_values happens
        TransformResult {
            m1_prime: m1.clone(),
            m2_prime: m2.clone(),
            error: None,
        }
    }

    fn transform_with_insert_row(&self, m1: &MutationInfo, m2: &MutationInfo) -> TransformResult {
        let m1_params: RemoveRowsMutationParams = serde_wasm_bindgen::from_value(m1.params.clone())
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

        let remove_start = m1_params.range.start_row;
        let remove_end = m1_params.range.end_row;
        let insert_row = m2_params.range.start_row;

        // If insert_row is before remove_start, shift remove range down
        let mut new_m1_params = m1_params.clone();
        if insert_row <= remove_start {
            new_m1_params.range.start_row += 1;
            new_m1_params.range.end_row += 1;
        } else if insert_row <= remove_end {
            // Insert row is within removed range, extend remove range
            new_m1_params.range.end_row += 1;
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

    fn transform_with_insert_col(&self, m1: &MutationInfo, m2: &MutationInfo) -> TransformResult {
        // Remove rows and insert col are independent
        TransformResult {
            m1_prime: m1.clone(),
            m2_prime: m2.clone(),
            error: None,
        }
    }

    fn transform_with_remove_rows(&self, m1: &MutationInfo, m2: &MutationInfo) -> TransformResult {
        let m1_params: RemoveRowsMutationParams = serde_wasm_bindgen::from_value(m1.params.clone())
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

        // Check for conflict: overlapping ranges
        let m1_start = m1_params.range.start_row;
        let m1_end = m1_params.range.end_row;
        let m2_start = m2_params.range.start_row;
        let m2_end = m2_params.range.end_row;

        if (m2_start >= m1_start && m2_start <= m1_end) ||
           (m2_end >= m1_start && m2_end <= m1_end) ||
           (m2_start <= m1_start && m2_end >= m1_end) {
            return TransformResult {
                m1_prime: m1.clone(),
                m2_prime: m2.clone(),
                error: Some("Conflicting remove operations".to_string()),
            };
        }

        TransformResult {
            m1_prime: m1.clone(),
            m2_prime: m2.clone(),
            error: None,
        }
    }

    fn transform_with_remove_col(&self, m1: &MutationInfo, m2: &MutationInfo) -> TransformResult {
        // Remove rows and remove col are independent
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
        let m1_params: RemoveRowsMutationParams = serde_json::from_value(m1.params.clone())
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

        let remove_start = m1_params.range.start_row;
        let remove_end = m1_params.range.end_row;
        let insert_row = m2_params.range.start_row;

        // If insert_row is before remove_start, shift remove range down
        let mut new_m1_params = m1_params.clone();
        if insert_row <= remove_start {
            new_m1_params.range.start_row += 1;
            new_m1_params.range.end_row += 1;
        } else if insert_row <= remove_end {
            // Insert row is within removed range, extend remove range
            new_m1_params.range.end_row += 1;
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
    fn transform_internal_with_insert_col(&self, m1: &MutationInfoInternal, m2: &MutationInfoInternal) -> TransformResultInternal {
        TransformResultInternal {
            m1_prime: m1.clone(),
            m2_prime: m2.clone(),
            error: None,
        }
    }

    #[cfg(any(test, feature = "server"))]
    fn transform_internal_with_remove_rows(&self, m1: &MutationInfoInternal, m2: &MutationInfoInternal) -> TransformResultInternal {
        let m1_params: RemoveRowsMutationParams = serde_json::from_value(m1.params.clone())
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

        // Check for conflict: overlapping ranges
        let m1_start = m1_params.range.start_row;
        let m1_end = m1_params.range.end_row;
        let m2_start = m2_params.range.start_row;
        let m2_end = m2_params.range.end_row;

        if (m2_start >= m1_start && m2_start <= m1_end) ||
           (m2_end >= m1_start && m2_end <= m1_end) ||
           (m2_start <= m1_start && m2_end >= m1_end) {
            return TransformResultInternal {
                m1_prime: m1.clone(),
                m2_prime: m2.clone(),
                error: Some("Conflicting remove operations".to_string()),
            };
        }

        TransformResultInternal {
            m1_prime: m1.clone(),
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
