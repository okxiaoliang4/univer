use crate::mutations::sheets::SheetReorderRangeParams;
use crate::transform::mutation_transform::MutationTransform;
use crate::transform::sheets_transform_utils::{
    shift_range_cols_for_insert, shift_range_cols_for_remove, shift_range_rows_for_insert,
    shift_range_rows_for_remove,
};
use crate::types::{
    InsertColMutationParams, InsertRowMutationParams, MutationInfoInternal, Range,
    RemoveColMutationParams, RemoveRowsMutationParams, SubUnitParams, TransformResultInternal,
};
use serde_json;

#[derive(Default)]
pub struct ReorderRangeTransform;

impl MutationTransform for ReorderRangeTransform {
    fn mutation_id() -> &'static str {
        "sheet.mutation.reorder-range"
    }

    fn transform_with_set_range_values(
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

    fn transform_with_insert_row(
        &self,
        m1: &MutationInfoInternal,
        m2: &MutationInfoInternal,
    ) -> TransformResultInternal {
        let mut params: SheetReorderRangeParams = serde_json::from_value(m1.params.clone())
            .unwrap_or_else(|_| SheetReorderRangeParams {
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
                order: serde_json::Map::new(),
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
        if params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id
            || params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id
        {
            return TransformResultInternal {
                m1_prime: Some(m1.clone()),
                m2_prime: Some(m2.clone()),
                error: None,
            };
        }
        let insert_start = m2_params.range.start_row;
        let insert_count = m2_params.range.end_row - m2_params.range.start_row + 1;
        shift_range_rows_for_insert(&mut params.range, insert_start, insert_count);
        let mut new_order = serde_json::Map::new();
        for (key, value) in params.order.iter() {
            if let Ok(row_num) = key.parse::<u32>() {
                let new_key = if row_num >= insert_start {
                    row_num + insert_count
                } else {
                    row_num
                };
                new_order.insert(new_key.to_string(), value.clone());
            } else {
                new_order.insert(key.clone(), value.clone());
            }
        }
        params.order = new_order;
        TransformResultInternal {
            m1_prime: Some(MutationInfoInternal {
                id: m1.id.clone(),
                params: serde_json::to_value(params).unwrap(),
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
        let mut params: SheetReorderRangeParams = serde_json::from_value(m1.params.clone())
            .unwrap_or_else(|_| SheetReorderRangeParams {
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
                order: serde_json::Map::new(),
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
        if params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id
            || params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id
        {
            return TransformResultInternal {
                m1_prime: Some(m1.clone()),
                m2_prime: Some(m2.clone()),
                error: None,
            };
        }
        let insert_start = m2_params.range.start_column;
        let insert_count = m2_params.range.end_column - m2_params.range.start_column + 1;
        shift_range_cols_for_insert(&mut params.range, insert_start, insert_count);
        TransformResultInternal {
            m1_prime: Some(MutationInfoInternal {
                id: m1.id.clone(),
                params: serde_json::to_value(params).unwrap(),
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
        let mut params: SheetReorderRangeParams = serde_json::from_value(m1.params.clone())
            .unwrap_or_else(|_| SheetReorderRangeParams {
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
                order: serde_json::Map::new(),
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
        if params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id
            || params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id
        {
            return TransformResultInternal {
                m1_prime: Some(m1.clone()),
                m2_prime: Some(m2.clone()),
                error: None,
            };
        }
        if !shift_range_rows_for_remove(
            &mut params.range,
            m2_params.range.start_row,
            m2_params.range.end_row,
        ) {
            return TransformResultInternal {
                m1_prime: None,
                m2_prime: Some(m2.clone()),
                error: None,
            };
        }
        let mut new_order = serde_json::Map::new();
        for (key, value) in params.order.iter() {
            if let Ok(row_num) = key.parse::<u32>() {
                if row_num > m2_params.range.end_row {
                    new_order.insert(
                        (row_num - (m2_params.range.end_row - m2_params.range.start_row + 1))
                            .to_string(),
                        value.clone(),
                    );
                } else if row_num < m2_params.range.start_row {
                    new_order.insert(key.clone(), value.clone());
                }
            } else {
                new_order.insert(key.clone(), value.clone());
            }
        }
        params.order = new_order;
        TransformResultInternal {
            m1_prime: Some(MutationInfoInternal {
                id: m1.id.clone(),
                params: serde_json::to_value(params).unwrap(),
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
        let mut params: SheetReorderRangeParams = serde_json::from_value(m1.params.clone())
            .unwrap_or_else(|_| SheetReorderRangeParams {
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
                order: serde_json::Map::new(),
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
        if params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id
            || params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id
        {
            return TransformResultInternal {
                m1_prime: Some(m1.clone()),
                m2_prime: Some(m2.clone()),
                error: None,
            };
        }
        if !shift_range_cols_for_remove(
            &mut params.range,
            m2_params.range.start_column,
            m2_params.range.end_column,
        ) {
            return TransformResultInternal {
                m1_prime: None,
                m2_prime: Some(m2.clone()),
                error: None,
            };
        }
        TransformResultInternal {
            m1_prime: Some(MutationInfoInternal {
                id: m1.id.clone(),
                params: serde_json::to_value(params).unwrap(),
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
        if m1.id != m2.id {
            return vec![m1.clone(), m2.clone()];
        }
        vec![m2.clone()]
    }
}
