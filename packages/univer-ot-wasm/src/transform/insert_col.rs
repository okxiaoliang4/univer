use crate::types::{
    MutationInfoInternal, TransformResultInternal,
    InsertColMutationParams, RemoveColMutationParams,
    SetRangeValuesMutationParams, ObjectMatrixPrimitiveType,
    SubUnitParams, Range,
};
use crate::transform::mutation_transform::MutationTransform;
use serde_json;

#[derive(Default)]
pub struct InsertColTransform;

fn shift_cols_for_insert(cell_value: &mut ObjectMatrixPrimitiveType, insert_start: u32, insert_count: u32) {
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

impl MutationTransform for InsertColTransform {
    fn mutation_id() -> &'static str {
        "sheet.mutation.insert-col"
    }

    fn transform_with_set_range_values(&self, m1: &MutationInfoInternal, m2: &MutationInfoInternal) -> TransformResultInternal {
        let m1_params: InsertColMutationParams = serde_json::from_value(m1.params.clone())
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

        let insert_col = m1_params.range.start_column;
        let insert_count = m1_params.range.end_column - m1_params.range.start_column + 1;

        if let Some(ref mut cell_value) = m2_params.cell_value {
            shift_cols_for_insert(cell_value, insert_col, insert_count);
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
        // Insert col and insert row are independent
        TransformResultInternal {
            m1_prime: m1.clone(),
            m2_prime: m2.clone(),
            error: None,
        }
    }

    fn transform_with_insert_col(&self, m1: &MutationInfoInternal, m2: &MutationInfoInternal) -> TransformResultInternal {
        let m1_params: InsertColMutationParams = serde_json::from_value(m1.params.clone())
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

        let mut m2_params: InsertColMutationParams = serde_json::from_value(m2.params.clone())
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

        if m1_params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id ||
           m1_params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id {
            return TransformResultInternal {
                m1_prime: m1.clone(),
                m2_prime: m2.clone(),
                error: None,
            };
        }

        let m1_col = m1_params.range.start_column;
        let m1_count = m1_params.range.end_column - m1_params.range.start_column + 1;
        let m2_col = m2_params.range.start_column;
        let m2_count = m2_params.range.end_column - m2_params.range.start_column + 1;

        if m1_col <= m2_col {
            m2_params.range.start_column += m1_count;
            m2_params.range.end_column += m1_count;
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
            new_m1_params.range.start_column += m2_count;
            new_m1_params.range.end_column += m2_count;
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

    fn transform_with_remove_rows(&self, m1: &MutationInfoInternal, m2: &MutationInfoInternal) -> TransformResultInternal {
        // Insert col and remove rows are independent
        TransformResultInternal {
            m1_prime: m1.clone(),
            m2_prime: m2.clone(),
            error: None,
        }
    }

    fn transform_with_remove_col(&self, m1: &MutationInfoInternal, m2: &MutationInfoInternal) -> TransformResultInternal {
        let m1_params: InsertColMutationParams = serde_json::from_value(m1.params.clone())
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

        if m1_params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id ||
           m1_params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id {
            return TransformResultInternal {
                m1_prime: m1.clone(),
                m2_prime: m2.clone(),
                error: None,
            };
        }

        let insert_col = m1_params.range.start_column;
        let insert_count = m1_params.range.end_column - m1_params.range.start_column + 1;
        let remove_start = m2_params.range.start_column;
        let remove_end = m2_params.range.end_column;
        let remove_count = m2_params.range.end_column - m2_params.range.start_column + 1;

        if insert_col <= remove_start {
            let mut new_m2_params = m2_params.clone();
            new_m2_params.range.start_column += insert_count;
            new_m2_params.range.end_column += insert_count;
            TransformResultInternal {
                m1_prime: m1.clone(),
                m2_prime: MutationInfoInternal {
                    id: m2.id.clone(),
                    params: serde_json::to_value(new_m2_params).unwrap(),
                },
                error: None,
            }
        } else if insert_col > remove_end {
            let mut new_m1_params = m1_params.clone();
            new_m1_params.range.start_column -= remove_count;
            new_m1_params.range.end_column -= remove_count;
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
                error: Some("Insert col conflicts with remove col".to_string()),
            }
        }
    }
}
