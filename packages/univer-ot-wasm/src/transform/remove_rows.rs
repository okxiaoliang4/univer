use crate::transform::mutation_transform::MutationTransform;
use crate::mutations::types::{
    InsertRowMutationParams, RemoveRowsMutationParams, SetRangeValuesMutationParams,
};
use crate::types::{
    MutationInfoInternal, ObjectMatrixPrimitiveType, Range, SubUnitParams, TransformResultInternal,
};
use crate::wasm_log_debug;
use serde_json;

#[derive(Default)]
pub struct RemoveRowsTransform;

fn shift_rows_for_remove(
    cell_value: &mut ObjectMatrixPrimitiveType,
    remove_start: u32,
    remove_end: u32,
) {
    wasm_log_debug!(
        "remove_rows shift_rows_for_remove start remove_start={} remove_end={} rows={}",
        remove_start,
        remove_end,
        cell_value.data.len()
    );

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

impl MutationTransform for RemoveRowsTransform {
    fn mutation_id() -> &'static str {
        "sheet.mutation.remove-rows"
    }

    fn transform_with_set_range_values(
        &self,
        m1: &MutationInfoInternal,
        m2: &MutationInfoInternal,
    ) -> TransformResultInternal {
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

        let mut m2_params: SetRangeValuesMutationParams = serde_json::from_value(m2.params.clone())
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
            return TransformResultInternal {
                m1_prime: Some(m1.clone()),
                m2_prime: Some(m2.clone()),
                error: None,
            };
        }

        let remove_start = m1_params.range.start_row;
        let remove_end = m1_params.range.end_row;

        if let Some(ref mut cell_value) = m2_params.cell_value {
            shift_rows_for_remove(cell_value, remove_start, remove_end);
        }

        TransformResultInternal {
            m1_prime: Some(m1.clone()),
            m2_prime: Some(MutationInfoInternal {
                id: m2.id.clone(),
                params: serde_json::to_value(m2_params).unwrap(),
            }),
            error: None,
        }
    }

    fn transform_with_insert_row(
        &self,
        m1: &MutationInfoInternal,
        m2: &MutationInfoInternal,
    ) -> TransformResultInternal {
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

        if m1_params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id
            || m1_params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id
        {
            return TransformResultInternal {
                m1_prime: Some(m1.clone()),
                m2_prime: Some(m2.clone()),
                error: None,
            };
        }

        let remove_start = m1_params.range.start_row;
        let remove_end = m1_params.range.end_row;
        let remove_count = m1_params.range.end_row - m1_params.range.start_row + 1;
        let insert_row = m2_params.range.start_row;
        let insert_count = m2_params.range.end_row - m2_params.range.start_row + 1;

        if insert_row < remove_start {
            let mut new_m1_params = m1_params.clone();
            new_m1_params.range.start_row += insert_count;
            new_m1_params.range.end_row += insert_count;
            TransformResultInternal {
                m1_prime: Some(MutationInfoInternal {
                    id: m1.id.clone(),
                    params: serde_json::to_value(new_m1_params).unwrap(),
                }),
                m2_prime: Some(m2.clone()),
                error: None,
            }
        } else if insert_row > remove_end {
            let mut new_m2_params = m2_params.clone();
            new_m2_params.range.start_row -= remove_count;
            new_m2_params.range.end_row -= remove_count;
            TransformResultInternal {
                m1_prime: Some(m1.clone()),
                m2_prime: Some(MutationInfoInternal {
                    id: m2.id.clone(),
                    params: serde_json::to_value(new_m2_params).unwrap(),
                }),
                error: None,
            }
        } else {
            // Insert happens inside removed range; delete wins and grows to include insert.
            let mut new_m1_params = m1_params.clone();
            new_m1_params.range.end_row += insert_count;
            TransformResultInternal {
                m1_prime: Some(MutationInfoInternal {
                    id: m1.id.clone(),
                    params: serde_json::to_value(new_m1_params).unwrap(),
                }),
                m2_prime: None,
                error: None,
            }
        }
    }

    fn transform_with_insert_col(
        &self,
        m1: &MutationInfoInternal,
        m2: &MutationInfoInternal,
    ) -> TransformResultInternal {
        // Remove rows and insert col are independent
        TransformResultInternal {
            m1_prime: Some(m1.clone()),
            m2_prime: Some(m2.clone()),
            error: None,
        }
    }

    fn transform_with_remove_rows(
        &self,
        m1: &MutationInfoInternal,
        m2: &MutationInfoInternal,
    ) -> TransformResultInternal {
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

        if m1_params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id
            || m1_params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id
        {
            return TransformResultInternal {
                m1_prime: Some(m1.clone()),
                m2_prime: Some(m2.clone()),
                error: None,
            };
        }

        let m1_prime_params = transform_remove_against_remove(&m1_params, &m2_params);
        let m2_prime_params = transform_remove_against_remove(&m2_params, &m1_params);

        TransformResultInternal {
            m1_prime: m1_prime_params.map(|params| MutationInfoInternal {
                id: m1.id.clone(),
                params: serde_json::to_value(params).unwrap(),
            }),
            m2_prime: m2_prime_params.map(|params| MutationInfoInternal {
                id: m2.id.clone(),
                params: serde_json::to_value(params).unwrap(),
            }),
            error: None,
        }
    }

    fn transform_with_remove_col(
        &self,
        m1: &MutationInfoInternal,
        m2: &MutationInfoInternal,
    ) -> TransformResultInternal {
        // Remove rows and remove col are independent
        TransformResultInternal {
            m1_prime: Some(m1.clone()),
            m2_prime: Some(m2.clone()),
            error: None,
        }
    }

    fn compose(
        &self,
        m1: &MutationInfoInternal,
        m2: &MutationInfoInternal,
    ) -> Vec<MutationInfoInternal> {
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

        if m1_params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id
            || m1_params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id
        {
            return vec![m1.clone(), m2.clone()];
        }

        let m1_start = m1_params.range.start_row;
        let m1_end = m1_params.range.end_row;
        let m1_count = m1_end - m1_start + 1;
        let m2_start = m2_params.range.start_row;
        let m2_end = m2_params.range.end_row;

        if m2_start < m1_start && m2_end >= m1_start {
            return vec![m1.clone(), m2.clone()];
        }

        let (mapped_start, mapped_end) = if m2_start >= m1_start {
            (m2_start + m1_count, m2_end + m1_count)
        } else {
            (m2_start, m2_end)
        };

        if mapped_start > m1_end + 1 || m1_start > mapped_end + 1 {
            return vec![m1.clone(), m2.clone()];
        }

        let merged_start = std::cmp::min(m1_start, mapped_start);
        let merged_end = std::cmp::max(m1_end, mapped_end);

        let composed_params = RemoveRowsMutationParams {
            sub_unit_params: m1_params.sub_unit_params,
            range: Range {
                start_row: merged_start,
                start_column: m1_params.range.start_column,
                end_row: merged_end,
                end_column: m1_params.range.end_column,
            },
        };

        vec![MutationInfoInternal {
            id: m1.id.clone(),
            params: serde_json::to_value(composed_params).unwrap(),
        }]
    }
}

fn transform_remove_against_remove(
    m1_params: &RemoveRowsMutationParams,
    m2_params: &RemoveRowsMutationParams,
) -> Option<RemoveRowsMutationParams> {
    let m1_start = m1_params.range.start_row;
    let m1_end = m1_params.range.end_row;
    let m2_start = m2_params.range.start_row;
    let m2_end = m2_params.range.end_row;

    let m1_count = m1_end - m1_start + 1;
    let m2_count = m2_end - m2_start + 1;

    if m2_end < m1_start {
        let mut new_m1_params = m1_params.clone();
        new_m1_params.range.start_row -= m2_count;
        new_m1_params.range.end_row -= m2_count;
        return Some(new_m1_params);
    }

    if m2_start > m1_end {
        return Some(m1_params.clone());
    }

    let overlap_start = std::cmp::max(m1_start, m2_start);
    let overlap_end = std::cmp::min(m1_end, m2_end);
    let overlap_count = overlap_end - overlap_start + 1;
    let remaining_count = m1_count - overlap_count;

    if remaining_count == 0 {
        return None;
    }

    let removed_before_m1 = if m2_start < m1_start {
        m1_start - m2_start
    } else {
        0
    };
    let new_start = m1_start - removed_before_m1;
    let new_end = new_start + remaining_count - 1;
    let mut new_m1_params = m1_params.clone();
    new_m1_params.range.start_row = new_start;
    new_m1_params.range.end_row = new_end;
    Some(new_m1_params)
}
