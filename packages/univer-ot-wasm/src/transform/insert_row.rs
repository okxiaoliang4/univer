use crate::mutations::types::{
    InsertRowMutationParams, RemoveRowsMutationParams, RowData, SetRangeValuesMutationParams,
};
use crate::transform::mutation_transform::MutationTransform;
use crate::transform::sheets_mutations::common::shift_rule_ranges_for_insert;
use crate::transform::sheets_transform_utils::shift_row_keys_for_insert;
use crate::types::{
    MutationInfoInternal, ObjectMatrixPrimitiveType, Range, SubUnitParams, TransformResultInternal,
};
use crate::wasm_log_debug;
use serde_json;

#[derive(Default)]
pub struct InsertRowTransform;

impl MutationTransform for InsertRowTransform {
    fn mutation_id() -> &'static str {
        "sheet.mutation.insert-row"
    }

    fn transform_with_set_range_values(
        &self,
        m1: &MutationInfoInternal,
        m2: &MutationInfoInternal,
    ) -> TransformResultInternal {
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

        if m1_params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id
            || m1_params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id
        {
            return TransformResultInternal {
                m1_prime: Some(m1.clone()),
                m2_prime: Some(m2.clone()),
                error: None,
            };
        }

        let insert_row = m1_params.range.start_row;
        let insert_count = m1_params.range.end_row - m1_params.range.start_row + 1;

        if let Some(ref mut cell_value) = m2_params.cell_value {
            shift_row_keys_for_insert(cell_value, insert_row, insert_count);
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

        if m1_params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id
            || m1_params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id
        {
            return TransformResultInternal {
                m1_prime: Some(m1.clone()),
                m2_prime: Some(m2.clone()),
                error: None,
            };
        }

        let m1_row = m1_params.range.start_row;
        let m1_count = m1_params.range.end_row - m1_params.range.start_row + 1;
        let m2_row = m2_params.range.start_row;
        let m2_count = m2_params.range.end_row - m2_params.range.start_row + 1;

        if m1_row < m2_row {
            m2_params.range.start_row += m1_count;
            m2_params.range.end_row += m1_count;
            TransformResultInternal {
                m1_prime: Some(m1.clone()),
                m2_prime: Some(MutationInfoInternal {
                    id: m2.id.clone(),
                    params: serde_json::to_value(m2_params).unwrap(),
                }),
                error: None,
            }
        } else {
            let mut new_m1_params = m1_params.clone();
            new_m1_params.range.start_row += m2_count;
            new_m1_params.range.end_row += m2_count;
            TransformResultInternal {
                m1_prime: Some(MutationInfoInternal {
                    id: m1.id.clone(),
                    params: serde_json::to_value(new_m1_params).unwrap(),
                }),
                m2_prime: Some(m2.clone()),
                error: None,
            }
        }
    }

    fn transform_with_insert_col(
        &self,
        m1: &MutationInfoInternal,
        m2: &MutationInfoInternal,
    ) -> TransformResultInternal {
        // Insert row and insert col are independent
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

        if m1_params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id
            || m1_params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id
        {
            return TransformResultInternal {
                m1_prime: Some(m1.clone()),
                m2_prime: Some(m2.clone()),
                error: None,
            };
        }

        let insert_row = m1_params.range.start_row;
        let insert_count = m1_params.range.end_row - m1_params.range.start_row + 1;
        let remove_start = m2_params.range.start_row;
        let remove_end = m2_params.range.end_row;
        let remove_count = m2_params.range.end_row - m2_params.range.start_row + 1;

        if remove_end < insert_row {
            let mut new_m1_params = m1_params.clone();
            new_m1_params.range.start_row -= remove_count;
            new_m1_params.range.end_row -= remove_count;
            TransformResultInternal {
                m1_prime: Some(MutationInfoInternal {
                    id: m1.id.clone(),
                    params: serde_json::to_value(new_m1_params).unwrap(),
                }),
                m2_prime: Some(m2.clone()),
                error: None,
            }
        } else if remove_start > insert_row {
            let mut new_m2_params = m2_params.clone();
            new_m2_params.range.start_row += insert_count;
            new_m2_params.range.end_row += insert_count;
            TransformResultInternal {
                m1_prime: Some(m1.clone()),
                m2_prime: Some(MutationInfoInternal {
                    id: m2.id.clone(),
                    params: serde_json::to_value(new_m2_params).unwrap(),
                }),
                error: None,
            }
        } else {
            // Insert happens inside removed range; delete wins.
            let mut new_m2_params = m2_params.clone();
            new_m2_params.range.end_row += insert_count;
            TransformResultInternal {
                m1_prime: None,
                m2_prime: Some(MutationInfoInternal {
                    id: m2.id.clone(),
                    params: serde_json::to_value(new_m2_params).unwrap(),
                }),
                error: None,
            }
        }
    }

    fn transform_with_remove_col(
        &self,
        m1: &MutationInfoInternal,
        m2: &MutationInfoInternal,
    ) -> TransformResultInternal {
        // Insert row and remove col are independent
        TransformResultInternal {
            m1_prime: Some(m1.clone()),
            m2_prime: Some(m2.clone()),
            error: None,
        }
    }

    fn transform_with_add_conditional_rule(
        &self,
        m1: &MutationInfoInternal,
        m2: &MutationInfoInternal,
    ) -> TransformResultInternal {
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
        let mut m2_params: serde_json::Value = m2.params.clone();
        if m2_params.get("unitId").and_then(|v| v.as_str())
            != Some(&m1_params.sub_unit_params.unit_id)
            || m2_params.get("subUnitId").and_then(|v| v.as_str())
                != Some(&m1_params.sub_unit_params.sub_unit_id)
        {
            return TransformResultInternal {
                m1_prime: Some(m1.clone()),
                m2_prime: Some(m2.clone()),
                error: None,
            };
        }
        let insert_start = m1_params.range.start_row;
        let insert_count = m1_params.range.end_row - m1_params.range.start_row + 1;
        if let Some(rule) = m2_params.get_mut("rule") {
            shift_rule_ranges_for_insert(rule, Some((insert_start, insert_count)), None);
        }
        TransformResultInternal {
            m1_prime: Some(m1.clone()),
            m2_prime: Some(MutationInfoInternal {
                id: m2.id.clone(),
                params: m2_params,
            }),
            error: None,
        }
    }

    fn transform_with_set_conditional_rule(
        &self,
        m1: &MutationInfoInternal,
        m2: &MutationInfoInternal,
    ) -> TransformResultInternal {
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
        let mut m2_params: serde_json::Value = m2.params.clone();
        if m2_params.get("unitId").and_then(|v| v.as_str())
            != Some(&m1_params.sub_unit_params.unit_id)
            || m2_params.get("subUnitId").and_then(|v| v.as_str())
                != Some(&m1_params.sub_unit_params.sub_unit_id)
        {
            return TransformResultInternal {
                m1_prime: Some(m1.clone()),
                m2_prime: Some(m2.clone()),
                error: None,
            };
        }
        let insert_start = m1_params.range.start_row;
        let insert_count = m1_params.range.end_row - m1_params.range.start_row + 1;
        if let Some(rule) = m2_params.get_mut("rule") {
            shift_rule_ranges_for_insert(rule, Some((insert_start, insert_count)), None);
        }
        TransformResultInternal {
            m1_prime: Some(m1.clone()),
            m2_prime: Some(MutationInfoInternal {
                id: m2.id.clone(),
                params: m2_params,
            }),
            error: None,
        }
    }

    fn transform_with_delete_conditional_rule(
        &self,
        m1: &MutationInfoInternal,
        m2: &MutationInfoInternal,
    ) -> TransformResultInternal {
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
        let mut m2_params: serde_json::Value = m2.params.clone();
        if m2_params.get("unitId").and_then(|v| v.as_str())
            != Some(&m1_params.sub_unit_params.unit_id)
            || m2_params.get("subUnitId").and_then(|v| v.as_str())
                != Some(&m1_params.sub_unit_params.sub_unit_id)
        {
            return TransformResultInternal {
                m1_prime: Some(m1.clone()),
                m2_prime: Some(m2.clone()),
                error: None,
            };
        }
        let insert_start = m1_params.range.start_row;
        let insert_count = m1_params.range.end_row - m1_params.range.start_row + 1;
        if let Some(rule) = m2_params.get_mut("rule") {
            shift_rule_ranges_for_insert(rule, Some((insert_start, insert_count)), None);
        }
        TransformResultInternal {
            m1_prime: Some(m1.clone()),
            m2_prime: Some(MutationInfoInternal {
                id: m2.id.clone(),
                params: m2_params,
            }),
            error: None,
        }
    }

    fn transform_with_move_conditional_rule(
        &self,
        m1: &MutationInfoInternal,
        m2: &MutationInfoInternal,
    ) -> TransformResultInternal {
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
        let mut m2_params: serde_json::Value = m2.params.clone();
        if m2_params.get("unitId").and_then(|v| v.as_str())
            != Some(&m1_params.sub_unit_params.unit_id)
            || m2_params.get("subUnitId").and_then(|v| v.as_str())
                != Some(&m1_params.sub_unit_params.sub_unit_id)
        {
            return TransformResultInternal {
                m1_prime: Some(m1.clone()),
                m2_prime: Some(m2.clone()),
                error: None,
            };
        }
        let insert_start = m1_params.range.start_row;
        let insert_count = m1_params.range.end_row - m1_params.range.start_row + 1;
        if let Some(rule) = m2_params.get_mut("rule") {
            shift_rule_ranges_for_insert(rule, Some((insert_start, insert_count)), None);
        }
        TransformResultInternal {
            m1_prime: Some(m1.clone()),
            m2_prime: Some(MutationInfoInternal {
                id: m2.id.clone(),
                params: m2_params,
            }),
            error: None,
        }
    }

    fn transform_with_conditional_formatting_formula_mark_dirty(
        &self,
        m1: &MutationInfoInternal,
        m2: &MutationInfoInternal,
    ) -> TransformResultInternal {
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
            return vec![m1.clone(), m2.clone()];
        }

        let m1_start = m1_params.range.start_row;
        let m1_count = m1_params.range.end_row - m1_params.range.start_row + 1;
        let m2_start = m2_params.range.start_row;
        let m2_count = m2_params.range.end_row - m2_params.range.start_row + 1;

        if m2_start < m1_start || m2_start > m1_start + m1_count {
            return vec![m1.clone(), m2.clone()];
        }

        let offset = m2_start - m1_start;
        let mut merged_info: Option<Vec<RowData>> = None;

        if m1_params.row_info.is_some() || m2_params.row_info.is_some() {
            let _m1_len = m1_params.row_info.as_ref().map(|v| v.len()).unwrap_or(0);
            let _m2_len = m2_params.row_info.as_ref().map(|v| v.len()).unwrap_or(0);
            let total_len = (m1_count + m2_count) as usize;
            let mut merged_vec = Vec::with_capacity(total_len);

            // Initialize with default values
            for _ in 0..total_len {
                merged_vec.push(RowData { h: None, hd: None });
            }

            // Copy m1 row_info, shifting items at offset and beyond
            if let Some(row_info) = &m1_params.row_info {
                for (index, row_data) in row_info.iter().enumerate() {
                    let target_index = if (index as u32) >= offset {
                        (index as u32 + m2_count) as usize
                    } else {
                        index
                    };
                    if target_index < merged_vec.len() {
                        merged_vec[target_index] = row_data.clone();
                    }
                }
            }

            // Insert m2 row_info at offset position
            if let Some(row_info) = &m2_params.row_info {
                for (index, row_data) in row_info.iter().enumerate() {
                    let target_index = (offset as usize) + index;
                    if target_index < merged_vec.len() {
                        merged_vec[target_index] = row_data.clone();
                    }
                }
            }

            merged_info = Some(merged_vec);
        }

        let composed_params = InsertRowMutationParams {
            sub_unit_params: m1_params.sub_unit_params,
            range: Range {
                start_row: m1_start,
                start_column: m1_params.range.start_column,
                end_row: m1_start + m1_count + m2_count - 1,
                end_column: m1_params.range.end_column,
            },
            row_info: merged_info,
        };

        vec![MutationInfoInternal {
            id: m1.id.clone(),
            params: serde_json::to_value(composed_params).unwrap(),
        }]
    }
}
