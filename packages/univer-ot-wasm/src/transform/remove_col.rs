use crate::types::{
    MutationInfoInternal, TransformResultInternal,
    InsertColMutationParams, RemoveColMutationParams,
    SetRangeValuesMutationParams, ObjectMatrixPrimitiveType,
    SubUnitParams, Range,
};
use crate::transform::mutation_transform::MutationTransform;
use serde_json;

#[derive(Default)]
pub struct RemoveColTransform;

fn shift_cols_for_remove(cell_value: &mut ObjectMatrixPrimitiveType, remove_start: u32, remove_end: u32) {
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

impl MutationTransform for RemoveColTransform {
    fn mutation_id() -> &'static str {
        "sheet.mutation.remove-col"
    }

    fn transform_with_set_range_values(&self, m1: &MutationInfoInternal, m2: &MutationInfoInternal) -> TransformResultInternal {
        let m1_params: RemoveColMutationParams = serde_json::from_value(m1.params.clone())
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

        let remove_start = m1_params.range.start_column;
        let remove_end = m1_params.range.end_column;

        if let Some(ref mut cell_value) = m2_params.cell_value {
            shift_cols_for_remove(cell_value, remove_start, remove_end);
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
        // Remove col and insert row are independent
        TransformResultInternal {
            m1_prime: m1.clone(),
            m2_prime: m2.clone(),
            error: None,
        }
    }

    fn transform_with_insert_col(&self, m1: &MutationInfoInternal, m2: &MutationInfoInternal) -> TransformResultInternal {
        let m1_params: RemoveColMutationParams = serde_json::from_value(m1.params.clone())
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

        if m1_params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id ||
           m1_params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id {
            return TransformResultInternal {
                m1_prime: m1.clone(),
                m2_prime: m2.clone(),
                error: None,
            };
        }

        let remove_start = m1_params.range.start_column;
        let remove_end = m1_params.range.end_column;
        let remove_count = m1_params.range.end_column - m1_params.range.start_column + 1;
        let insert_col = m2_params.range.start_column;
        let insert_count = m2_params.range.end_column - m2_params.range.start_column + 1;

        if insert_col <= remove_start {
            let mut new_m1_params = m1_params.clone();
            new_m1_params.range.start_column += insert_count;
            new_m1_params.range.end_column += insert_count;
            TransformResultInternal {
                m1_prime: MutationInfoInternal {
                    id: m1.id.clone(),
                    params: serde_json::to_value(new_m1_params).unwrap(),
                },
                m2_prime: m2.clone(),
                error: None,
            }
        } else if insert_col > remove_end {
            let mut new_m2_params = m2_params.clone();
            new_m2_params.range.start_column -= remove_count;
            new_m2_params.range.end_column -= remove_count;
            TransformResultInternal {
                m1_prime: m1.clone(),
                m2_prime: MutationInfoInternal {
                    id: m2.id.clone(),
                    params: serde_json::to_value(new_m2_params).unwrap(),
                },
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

    fn transform_with_remove_rows(&self, m1: &MutationInfoInternal, m2: &MutationInfoInternal) -> TransformResultInternal {
        // Remove col and remove rows are independent
        TransformResultInternal {
            m1_prime: m1.clone(),
            m2_prime: m2.clone(),
            error: None,
        }
    }

    fn transform_with_remove_col(&self, m1: &MutationInfoInternal, m2: &MutationInfoInternal) -> TransformResultInternal {
        let m1_params: RemoveColMutationParams = serde_json::from_value(m1.params.clone())
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

        let m1_start = m1_params.range.start_column;
        let m1_end = m1_params.range.end_column;
        let m2_start = m2_params.range.start_column;
        let m2_end = m2_params.range.end_column;

        if (m2_start >= m1_start && m2_start <= m1_end) ||
           (m2_end >= m1_start && m2_end <= m1_end) ||
           (m2_start <= m1_start && m2_end >= m1_end) {
            return TransformResultInternal {
                m1_prime: m1.clone(),
                m2_prime: m2.clone(),
                error: Some("Conflicting remove operations".to_string()),
            };
        }

        let m1_count = m1_end - m1_start + 1;
        let m2_count = m2_end - m2_start + 1;

        if m1_end < m2_start {
            let mut new_m2_params = m2_params.clone();
            new_m2_params.range.start_column -= m1_count;
            new_m2_params.range.end_column -= m1_count;
            return TransformResultInternal {
                m1_prime: m1.clone(),
                m2_prime: MutationInfoInternal {
                    id: m2.id.clone(),
                    params: serde_json::to_value(new_m2_params).unwrap(),
                },
                error: None,
            };
        }

        if m2_end < m1_start {
            let mut new_m1_params = m1_params.clone();
            new_m1_params.range.start_column -= m2_count;
            new_m1_params.range.end_column -= m2_count;
            return TransformResultInternal {
                m1_prime: MutationInfoInternal {
                    id: m1.id.clone(),
                    params: serde_json::to_value(new_m1_params).unwrap(),
                },
                m2_prime: m2.clone(),
                error: None,
            };
        }

        TransformResultInternal {
            m1_prime: m1.clone(),
            m2_prime: m2.clone(),
            error: None,
        }
    }
}
