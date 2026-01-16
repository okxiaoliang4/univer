use crate::types::{
    MutationInfoInternal, TransformResultInternal,
    InsertRowMutationParams, RemoveRowsMutationParams,
    SetRangeValuesMutationParams, ObjectMatrixPrimitiveType,
    SubUnitParams, Range,
};
use crate::transform::mutation_transform::MutationTransform;
use serde_json;

#[derive(Default)]
pub struct InsertRowTransform;

fn shift_rows_for_insert(cell_value: &mut ObjectMatrixPrimitiveType, insert_start: u32, insert_count: u32) {
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

impl MutationTransform for InsertRowTransform {
    fn mutation_id() -> &'static str {
        "sheet.mutation.insert-row"
    }

    fn transform_with_set_range_values(&self, m1: &MutationInfoInternal, m2: &MutationInfoInternal) -> TransformResultInternal {
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

        let mut m2_params: SetRangeValuesMutationParams = serde_json::from_value(m2.params.clone())
            .unwrap_or_else(|_| SetRangeValuesMutationParams {
                sub_unit_params: SubUnitParams {
                    unit_id: "".to_string(),
                    sub_unit_id: "".to_string(),
                },
                cell_value: None,
            });

        if m1_params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id ||
           m1_params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id {
            return TransformResultInternal {
                m1_prime: m1.clone(),
                m2_prime: m2.clone(),
                error: None,
            };
        }

        let insert_row = m1_params.range.start_row;
        let insert_count = m1_params.range.end_row - m1_params.range.start_row + 1;

        if let Some(ref mut cell_value) = m2_params.cell_value {
            shift_rows_for_insert(cell_value, insert_row, insert_count);
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

    fn transform_with_insert_row(&self, m1: &MutationInfoInternal, m2: &MutationInfoInternal) -> TransformResultInternal {
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

        if m1_params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id ||
           m1_params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id {
            return TransformResultInternal {
                m1_prime: m1.clone(),
                m2_prime: m2.clone(),
                error: None,
            };
        }

        let m1_row = m1_params.range.start_row;
        let m1_count = m1_params.range.end_row - m1_params.range.start_row + 1;
        let m2_row = m2_params.range.start_row;
        let m2_count = m2_params.range.end_row - m2_params.range.start_row + 1;

        if m1_row <= m2_row {
            m2_params.range.start_row += m1_count;
            m2_params.range.end_row += m1_count;
            TransformResultInternal {
                m1_prime: m1.clone(),
                m2_prime: MutationInfoInternal {
                    id: m2.id.clone(),
                    params: serde_json::to_value(m2_params).unwrap(),
                },
                error: None,
            }
        } else {
            let mut new_m1_params = m1_params.clone();
            new_m1_params.range.start_row += m2_count;
            new_m1_params.range.end_row += m2_count;
            TransformResultInternal {
                m1_prime: MutationInfoInternal {
                    id: m1.id.clone(),
                    params: serde_json::to_value(new_m1_params).unwrap(),
                },
                m2_prime: m2.clone(),
                error: None,
            }
        }
    }

    fn transform_with_insert_col(&self, m1: &MutationInfoInternal, m2: &MutationInfoInternal) -> TransformResultInternal {
        // Insert row and insert col are independent
        TransformResultInternal {
            m1_prime: m1.clone(),
            m2_prime: m2.clone(),
            error: None,
        }
    }

    fn transform_with_remove_rows(&self, m1: &MutationInfoInternal, m2: &MutationInfoInternal) -> TransformResultInternal {
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

        if m1_params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id ||
           m1_params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id {
            return TransformResultInternal {
                m1_prime: m1.clone(),
                m2_prime: m2.clone(),
                error: None,
            };
        }

        let insert_row = m1_params.range.start_row;
        let insert_count = m1_params.range.end_row - m1_params.range.start_row + 1;
        let remove_start = m2_params.range.start_row;
        let remove_end = m2_params.range.end_row;
        let remove_count = m2_params.range.end_row - m2_params.range.start_row + 1;

        if insert_row <= remove_start {
            let mut new_m2_params = m2_params.clone();
            new_m2_params.range.start_row += insert_count;
            new_m2_params.range.end_row += insert_count;
            TransformResultInternal {
                m1_prime: m1.clone(),
                m2_prime: MutationInfoInternal {
                    id: m2.id.clone(),
                    params: serde_json::to_value(new_m2_params).unwrap(),
                },
                error: None,
            }
        } else if insert_row > remove_end {
            let mut new_m1_params = m1_params.clone();
            new_m1_params.range.start_row -= remove_count;
            new_m1_params.range.end_row -= remove_count;
            TransformResultInternal {
                m1_prime: MutationInfoInternal {
                    id: m1.id.clone(),
                    params: serde_json::to_value(new_m1_params).unwrap(),
                },
                m2_prime: m2.clone(),
                error: None,
            }
        } else {
            TransformResultInternal {
                m1_prime: m1.clone(),
                m2_prime: m2.clone(),
                error: Some("Insert row conflicts with remove rows".to_string()),
            }
        }
    }

    fn transform_with_remove_col(&self, m1: &MutationInfoInternal, m2: &MutationInfoInternal) -> TransformResultInternal {
        // Insert row and remove col are independent
        TransformResultInternal {
            m1_prime: m1.clone(),
            m2_prime: m2.clone(),
            error: None,
        }
    }
}
