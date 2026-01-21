use crate::mutations::sheets::{
    SheetMoveRowsColsParams, SheetMutationRangeParams, SheetMutationRangesParams,
    SheetReorderRangeParams, SheetRowColumnDataParams, SheetSetFrozenParams,
};
use crate::transform::sheets_transform_utils::{
    parse_move_range_params, parse_range_params, shift_move_range_value_for_insert,
    shift_move_range_value_for_remove, shift_range_cols_for_insert, shift_range_cols_for_remove,
    shift_range_rows_for_insert, shift_range_rows_for_remove, shift_ranges_for_insert,
    shift_ranges_for_remove, shift_ranges_value_for_insert, shift_ranges_value_for_remove,
    shift_set_frozen_for_insert, shift_set_frozen_for_remove,
};
use crate::types::{
    InsertColMutationParams, InsertRowMutationParams, MutationInfoInternal, Range,
    RemoveColMutationParams, RemoveRowsMutationParams, SetRangeValuesMutationParams, SubUnitParams,
    TransformResultInternal,
};
use crate::wasm_log_warn;
use serde_json;

pub(crate) fn same_sheet(params1: &SubUnitParams, params2: &SubUnitParams) -> bool {
    params1.unit_id == params2.unit_id && params1.sub_unit_id == params2.sub_unit_id
}

pub(crate) fn ranges_intersect(r1: &Range, r2: &Range) -> bool {
    !(r1.end_row < r2.start_row
        || r1.start_row > r2.end_row
        || r1.end_column < r2.start_column
        || r1.start_column > r2.end_column)
}

pub(crate) fn parse_ranges_params(
    _id: &str,
    params: &serde_json::Value,
) -> SheetMutationRangesParams {
    serde_json::from_value(params.clone()).unwrap_or_else(|_| {
        wasm_log_warn!("invalid params");
        SheetMutationRangesParams {
            sub_unit_params: SubUnitParams {
                unit_id: "".to_string(),
                sub_unit_id: "".to_string(),
            },
            ranges: vec![],
        }
    })
}

pub(crate) fn shift_index_map_for_insert(
    data: &mut serde_json::Map<String, serde_json::Value>,
    insert_start: u32,
    insert_count: u32,
) {
    let mut new_data = serde_json::Map::new();
    for (key, value) in data.iter() {
        if let Ok(index) = key.parse::<u32>() {
            let new_key = if index >= insert_start {
                (index + insert_count).to_string()
            } else {
                index.to_string()
            };
            new_data.insert(new_key, value.clone());
        } else {
            new_data.insert(key.clone(), value.clone());
        }
    }
    *data = new_data;
}

pub(crate) fn shift_index_map_for_remove(
    data: &mut serde_json::Map<String, serde_json::Value>,
    remove_start: u32,
    remove_end: u32,
) {
    let remove_count = remove_end - remove_start + 1;
    let mut new_data = serde_json::Map::new();
    for (key, value) in data.iter() {
        if let Ok(index) = key.parse::<u32>() {
            if index > remove_end {
                new_data.insert((index - remove_count).to_string(), value.clone());
            } else if index < remove_start {
                new_data.insert(index.to_string(), value.clone());
            }
        } else {
            new_data.insert(key.clone(), value.clone());
        }
    }
    *data = new_data;
}

pub(crate) fn shift_rule_ranges_for_insert(
    rule_value: &mut serde_json::Value,
    insert_row: Option<(u32, u32)>,
    insert_col: Option<(u32, u32)>,
) {
    if let Some(ranges) = rule_value.get_mut("ranges") {
        shift_ranges_value_for_insert(ranges, insert_row, insert_col);
    }
}

pub(crate) fn shift_rule_ranges_for_remove(
    rule_value: &mut serde_json::Value,
    remove_row: Option<(u32, u32)>,
    remove_col: Option<(u32, u32)>,
) -> bool {
    if let Some(ranges) = rule_value.get_mut("ranges") {
        return shift_ranges_value_for_remove(ranges, remove_row, remove_col);
    }
    true
}

pub(crate) fn shift_ranges_params_for_insert(
    params: &mut serde_json::Value,
    insert_row: Option<(u32, u32)>,
    insert_col: Option<(u32, u32)>,
) {
    if let Some(ranges) = params.get_mut("ranges") {
        shift_ranges_value_for_insert(ranges, insert_row, insert_col);
    }
}

pub(crate) fn shift_ranges_params_for_remove(
    params: &mut serde_json::Value,
    remove_row: Option<(u32, u32)>,
    remove_col: Option<(u32, u32)>,
) -> bool {
    if let Some(ranges) = params.get_mut("ranges") {
        return shift_ranges_value_for_remove(ranges, remove_row, remove_col);
    }
    true
}

pub(crate) fn shift_rows_auto_height_for_insert(
    rows_value: &mut serde_json::Value,
    insert_start: u32,
    insert_count: u32,
) {
    if let Some(rows) = rows_value.as_array_mut() {
        for row_info in rows.iter_mut() {
            if let Some(row_value) = row_info.get_mut("row") {
                if let Some(row_num) = row_value.as_u64() {
                    if row_num >= insert_start as u64 {
                        *row_value = serde_json::Value::Number(serde_json::Number::from(
                            row_num + insert_count as u64,
                        ));
                    }
                }
            }
        }
    }
}

pub(crate) fn shift_rows_auto_height_for_remove(
    rows_value: &mut serde_json::Value,
    remove_start: u32,
    remove_end: u32,
) -> bool {
    let remove_count = remove_end - remove_start + 1;
    if let Some(rows) = rows_value.as_array_mut() {
        let mut new_rows = Vec::with_capacity(rows.len());
        for mut row_info in rows.drain(..) {
            let row_num = row_info.get("row").and_then(|value| value.as_u64());
            if let Some(row_num) = row_num {
                if row_num < remove_start as u64 {
                    new_rows.push(row_info);
                } else if row_num > remove_end as u64 {
                    if let Some(row_value) = row_info.get_mut("row") {
                        *row_value = serde_json::Value::Number(serde_json::Number::from(
                            row_num - remove_count as u64,
                        ));
                    }
                    new_rows.push(row_info);
                }
            } else {
                new_rows.push(row_info);
            }
        }
        *rows = new_rows;
        return !rows.is_empty();
    }
    true
}

pub(crate) fn transform_ranges_set_range_values(
    m1: &MutationInfoInternal,
    m2: &MutationInfoInternal,
) -> TransformResultInternal {
    let params = parse_ranges_params(&m1.id, &m1.params);
    let m2_params: SetRangeValuesMutationParams = serde_json::from_value(m2.params.clone())
        .unwrap_or_else(|_| SetRangeValuesMutationParams {
            sub_unit_params: SubUnitParams {
                unit_id: "".to_string(),
                sub_unit_id: "".to_string(),
            },
            cell_value: None,
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
    TransformResultInternal {
        m1_prime: Some(m1.clone()),
        m2_prime: Some(m2.clone()),
        error: None,
    }
}

pub(crate) fn transform_ranges_insert_row(
    m1: &MutationInfoInternal,
    m2: &MutationInfoInternal,
) -> TransformResultInternal {
    let mut params = parse_ranges_params(&m1.id, &m1.params);
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
    shift_ranges_for_insert(&mut params.ranges, Some((insert_start, insert_count)), None);
    TransformResultInternal {
        m1_prime: Some(MutationInfoInternal {
            id: m1.id.clone(),
            params: serde_json::to_value(params).unwrap(),
        }),
        m2_prime: Some(m2.clone()),
        error: None,
    }
}

pub(crate) fn transform_ranges_insert_col(
    m1: &MutationInfoInternal,
    m2: &MutationInfoInternal,
) -> TransformResultInternal {
    let mut params = parse_ranges_params(&m1.id, &m1.params);
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
    shift_ranges_for_insert(&mut params.ranges, None, Some((insert_start, insert_count)));
    TransformResultInternal {
        m1_prime: Some(MutationInfoInternal {
            id: m1.id.clone(),
            params: serde_json::to_value(params).unwrap(),
        }),
        m2_prime: Some(m2.clone()),
        error: None,
    }
}

pub(crate) fn transform_ranges_remove_rows(
    m1: &MutationInfoInternal,
    m2: &MutationInfoInternal,
) -> TransformResultInternal {
    let mut params = parse_ranges_params(&m1.id, &m1.params);
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
    if !shift_ranges_for_remove(
        &mut params.ranges,
        Some((m2_params.range.start_row, m2_params.range.end_row)),
        None,
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

pub(crate) fn transform_ranges_remove_cols(
    m1: &MutationInfoInternal,
    m2: &MutationInfoInternal,
) -> TransformResultInternal {
    let mut params = parse_ranges_params(&m1.id, &m1.params);
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
    if !shift_ranges_for_remove(
        &mut params.ranges,
        None,
        Some((m2_params.range.start_column, m2_params.range.end_column)),
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

#[macro_export]
macro_rules! impl_ranges_transform {
    ($name:ident, $id:expr) => {
        impl crate::transform::mutation_transform::MutationTransform for $name {
            fn mutation_id() -> &'static str {
                $id
            }

            fn transform_with_set_range_values(
                &self,
                m1: &crate::types::MutationInfoInternal,
                m2: &crate::types::MutationInfoInternal,
            ) -> crate::types::TransformResultInternal {
                $crate::transform::sheets_mutations::common::transform_ranges_set_range_values(
                    m1, m2,
                )
            }

            fn transform_with_insert_row(
                &self,
                m1: &crate::types::MutationInfoInternal,
                m2: &crate::types::MutationInfoInternal,
            ) -> crate::types::TransformResultInternal {
                $crate::transform::sheets_mutations::common::transform_ranges_insert_row(m1, m2)
            }

            fn transform_with_insert_col(
                &self,
                m1: &crate::types::MutationInfoInternal,
                m2: &crate::types::MutationInfoInternal,
            ) -> crate::types::TransformResultInternal {
                $crate::transform::sheets_mutations::common::transform_ranges_insert_col(m1, m2)
            }

            fn transform_with_remove_rows(
                &self,
                m1: &crate::types::MutationInfoInternal,
                m2: &crate::types::MutationInfoInternal,
            ) -> crate::types::TransformResultInternal {
                $crate::transform::sheets_mutations::common::transform_ranges_remove_rows(m1, m2)
            }

            fn transform_with_remove_col(
                &self,
                m1: &crate::types::MutationInfoInternal,
                m2: &crate::types::MutationInfoInternal,
            ) -> crate::types::TransformResultInternal {
                $crate::transform::sheets_mutations::common::transform_ranges_remove_cols(m1, m2)
            }

            fn compose(
                &self,
                m1: &crate::types::MutationInfoInternal,
                m2: &crate::types::MutationInfoInternal,
            ) -> Vec<crate::types::MutationInfoInternal> {
                if m1.id != m2.id {
                    return vec![m1.clone(), m2.clone()];
                }
                vec![m2.clone()]
            }
        }
    };
}

#[macro_export]
macro_rules! impl_range_transform {
    ($name:ident, $id:expr) => {
        impl crate::transform::mutation_transform::MutationTransform for $name {
            fn mutation_id() -> &'static str {
                $id
            }

            fn transform_with_set_range_values(
                &self,
                m1: &crate::types::MutationInfoInternal,
                m2: &crate::types::MutationInfoInternal,
            ) -> crate::types::TransformResultInternal {
                $crate::transform::sheets_mutations::common::transform_ranges_set_range_values(
                    m1, m2,
                )
            }

            fn transform_with_insert_row(
                &self,
                m1: &crate::types::MutationInfoInternal,
                m2: &crate::types::MutationInfoInternal,
            ) -> crate::types::TransformResultInternal {
                let mut params: $crate::mutations::sheets::SheetMutationRangeParams =
                    $crate::transform::sheets_transform_utils::parse_range_params(
                        &m1.id, &m1.params,
                    );
                let m2_params: $crate::types::InsertRowMutationParams =
                    serde_json::from_value(m2.params.clone()).unwrap_or_else(|_| {
                        $crate::types::InsertRowMutationParams {
                            sub_unit_params: $crate::types::SubUnitParams {
                                unit_id: "".to_string(),
                                sub_unit_id: "".to_string(),
                            },
                            range: $crate::types::Range {
                                start_row: 0,
                                start_column: 0,
                                end_row: 0,
                                end_column: 0,
                            },
                            row_info: None,
                        }
                    });
                if params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id
                    || params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id
                {
                    return crate::types::TransformResultInternal {
                        m1_prime: Some(m1.clone()),
                        m2_prime: Some(m2.clone()),
                        error: None,
                    };
                }
                let insert_start = m2_params.range.start_row;
                let insert_count = m2_params.range.end_row - m2_params.range.start_row + 1;
                $crate::transform::sheets_transform_utils::shift_range_rows_for_insert(
                    &mut params.range,
                    insert_start,
                    insert_count,
                );
                crate::types::TransformResultInternal {
                    m1_prime: Some(crate::types::MutationInfoInternal {
                        id: m1.id.clone(),
                        params: serde_json::to_value(params).unwrap(),
                    }),
                    m2_prime: Some(m2.clone()),
                    error: None,
                }
            }

            fn transform_with_insert_col(
                &self,
                m1: &crate::types::MutationInfoInternal,
                m2: &crate::types::MutationInfoInternal,
            ) -> crate::types::TransformResultInternal {
                let mut params: $crate::mutations::sheets::SheetMutationRangeParams =
                    $crate::transform::sheets_transform_utils::parse_range_params(
                        &m1.id, &m1.params,
                    );
                let m2_params: $crate::types::InsertColMutationParams =
                    serde_json::from_value(m2.params.clone()).unwrap_or_else(|_| {
                        $crate::types::InsertColMutationParams {
                            sub_unit_params: $crate::types::SubUnitParams {
                                unit_id: "".to_string(),
                                sub_unit_id: "".to_string(),
                            },
                            range: $crate::types::Range {
                                start_row: 0,
                                start_column: 0,
                                end_row: 0,
                                end_column: 0,
                            },
                            col_info: None,
                        }
                    });
                if params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id
                    || params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id
                {
                    return crate::types::TransformResultInternal {
                        m1_prime: Some(m1.clone()),
                        m2_prime: Some(m2.clone()),
                        error: None,
                    };
                }
                let insert_start = m2_params.range.start_column;
                let insert_count = m2_params.range.end_column - m2_params.range.start_column + 1;
                $crate::transform::sheets_transform_utils::shift_range_cols_for_insert(
                    &mut params.range,
                    insert_start,
                    insert_count,
                );
                crate::types::TransformResultInternal {
                    m1_prime: Some(crate::types::MutationInfoInternal {
                        id: m1.id.clone(),
                        params: serde_json::to_value(params).unwrap(),
                    }),
                    m2_prime: Some(m2.clone()),
                    error: None,
                }
            }

            fn transform_with_remove_rows(
                &self,
                m1: &crate::types::MutationInfoInternal,
                m2: &crate::types::MutationInfoInternal,
            ) -> crate::types::TransformResultInternal {
                let mut params: $crate::mutations::sheets::SheetMutationRangeParams =
                    $crate::transform::sheets_transform_utils::parse_range_params(
                        &m1.id, &m1.params,
                    );
                let m2_params: $crate::types::RemoveRowsMutationParams =
                    serde_json::from_value(m2.params.clone()).unwrap_or_else(|_| {
                        $crate::types::RemoveRowsMutationParams {
                            sub_unit_params: $crate::types::SubUnitParams {
                                unit_id: "".to_string(),
                                sub_unit_id: "".to_string(),
                            },
                            range: $crate::types::Range {
                                start_row: 0,
                                start_column: 0,
                                end_row: 0,
                                end_column: 0,
                            },
                        }
                    });
                if params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id
                    || params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id
                {
                    return crate::types::TransformResultInternal {
                        m1_prime: Some(m1.clone()),
                        m2_prime: Some(m2.clone()),
                        error: None,
                    };
                }
                if !$crate::transform::sheets_transform_utils::shift_range_rows_for_remove(
                    &mut params.range,
                    m2_params.range.start_row,
                    m2_params.range.end_row,
                ) {
                    return crate::types::TransformResultInternal {
                        m1_prime: None,
                        m2_prime: Some(m2.clone()),
                        error: None,
                    };
                }
                crate::types::TransformResultInternal {
                    m1_prime: Some(crate::types::MutationInfoInternal {
                        id: m1.id.clone(),
                        params: serde_json::to_value(params).unwrap(),
                    }),
                    m2_prime: Some(m2.clone()),
                    error: None,
                }
            }

            fn transform_with_remove_col(
                &self,
                m1: &crate::types::MutationInfoInternal,
                m2: &crate::types::MutationInfoInternal,
            ) -> crate::types::TransformResultInternal {
                let mut params: $crate::mutations::sheets::SheetMutationRangeParams =
                    $crate::transform::sheets_transform_utils::parse_range_params(
                        &m1.id, &m1.params,
                    );
                let m2_params: $crate::types::RemoveColMutationParams =
                    serde_json::from_value(m2.params.clone()).unwrap_or_else(|_| {
                        $crate::types::RemoveColMutationParams {
                            sub_unit_params: $crate::types::SubUnitParams {
                                unit_id: "".to_string(),
                                sub_unit_id: "".to_string(),
                            },
                            range: $crate::types::Range {
                                start_row: 0,
                                start_column: 0,
                                end_row: 0,
                                end_column: 0,
                            },
                        }
                    });
                if params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id
                    || params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id
                {
                    return crate::types::TransformResultInternal {
                        m1_prime: Some(m1.clone()),
                        m2_prime: Some(m2.clone()),
                        error: None,
                    };
                }
                if !$crate::transform::sheets_transform_utils::shift_range_cols_for_remove(
                    &mut params.range,
                    m2_params.range.start_column,
                    m2_params.range.end_column,
                ) {
                    return crate::types::TransformResultInternal {
                        m1_prime: None,
                        m2_prime: Some(m2.clone()),
                        error: None,
                    };
                }
                crate::types::TransformResultInternal {
                    m1_prime: Some(crate::types::MutationInfoInternal {
                        id: m1.id.clone(),
                        params: serde_json::to_value(params).unwrap(),
                    }),
                    m2_prime: Some(m2.clone()),
                    error: None,
                }
            }

            fn compose(
                &self,
                m1: &crate::types::MutationInfoInternal,
                m2: &crate::types::MutationInfoInternal,
            ) -> Vec<crate::types::MutationInfoInternal> {
                if m1.id != m2.id {
                    return vec![m1.clone(), m2.clone()];
                }
                vec![m2.clone()]
            }
        }
    };
}

#[macro_export]
macro_rules! impl_identity_transform {
    ($name:ident, $id:expr) => {
        impl crate::transform::mutation_transform::MutationTransform for $name {
            fn mutation_id() -> &'static str {
                $id
            }

            fn transform_with_set_range_values(
                &self,
                m1: &crate::types::MutationInfoInternal,
                m2: &crate::types::MutationInfoInternal,
            ) -> crate::types::TransformResultInternal {
                crate::types::TransformResultInternal {
                    m1_prime: Some(m1.clone()),
                    m2_prime: Some(m2.clone()),
                    error: None,
                }
            }

            fn transform_with_insert_row(
                &self,
                m1: &crate::types::MutationInfoInternal,
                m2: &crate::types::MutationInfoInternal,
            ) -> crate::types::TransformResultInternal {
                crate::types::TransformResultInternal {
                    m1_prime: Some(m1.clone()),
                    m2_prime: Some(m2.clone()),
                    error: None,
                }
            }

            fn transform_with_insert_col(
                &self,
                m1: &crate::types::MutationInfoInternal,
                m2: &crate::types::MutationInfoInternal,
            ) -> crate::types::TransformResultInternal {
                crate::types::TransformResultInternal {
                    m1_prime: Some(m1.clone()),
                    m2_prime: Some(m2.clone()),
                    error: None,
                }
            }

            fn transform_with_remove_rows(
                &self,
                m1: &crate::types::MutationInfoInternal,
                m2: &crate::types::MutationInfoInternal,
            ) -> crate::types::TransformResultInternal {
                crate::types::TransformResultInternal {
                    m1_prime: Some(m1.clone()),
                    m2_prime: Some(m2.clone()),
                    error: None,
                }
            }

            fn transform_with_remove_col(
                &self,
                m1: &crate::types::MutationInfoInternal,
                m2: &crate::types::MutationInfoInternal,
            ) -> crate::types::TransformResultInternal {
                crate::types::TransformResultInternal {
                    m1_prime: Some(m1.clone()),
                    m2_prime: Some(m2.clone()),
                    error: None,
                }
            }

            fn compose(
                &self,
                m1: &crate::types::MutationInfoInternal,
                m2: &crate::types::MutationInfoInternal,
            ) -> Vec<crate::types::MutationInfoInternal> {
                if m1.id != m2.id {
                    return vec![m1.clone(), m2.clone()];
                }
                vec![m2.clone()]
            }
        }
    };
}

#[macro_export]
macro_rules! impl_row_data_transform {
    ($name:ident, $id:expr) => {
        impl crate::transform::mutation_transform::MutationTransform for $name {
            fn mutation_id() -> &'static str {
                $id
            }

            fn transform_with_set_range_values(
                &self,
                m1: &$crate::types::MutationInfoInternal,
                m2: &$crate::types::MutationInfoInternal,
            ) -> $crate::types::TransformResultInternal {
                $crate::types::TransformResultInternal {
                    m1_prime: Some(m1.clone()),
                    m2_prime: Some(m2.clone()),
                    error: None,
                }
            }

            fn transform_with_insert_row(
                &self,
                m1: &$crate::types::MutationInfoInternal,
                m2: &$crate::types::MutationInfoInternal,
            ) -> $crate::types::TransformResultInternal {
                let mut params: $crate::mutations::sheets::SheetRowColumnDataParams =
                    serde_json::from_value(m1.params.clone()).unwrap_or_else(|_| {
                        $crate::mutations::sheets::SheetRowColumnDataParams {
                            sub_unit_params: $crate::types::SubUnitParams {
                                unit_id: "".to_string(),
                                sub_unit_id: "".to_string(),
                            },
                            row_data: None,
                            column_data: None,
                        }
                    });
                let m2_params: $crate::types::InsertRowMutationParams =
                    serde_json::from_value(m2.params.clone()).unwrap_or_else(|_| {
                        $crate::types::InsertRowMutationParams {
                            sub_unit_params: $crate::types::SubUnitParams {
                                unit_id: "".to_string(),
                                sub_unit_id: "".to_string(),
                            },
                            range: $crate::types::Range {
                                start_row: 0,
                                start_column: 0,
                                end_row: 0,
                                end_column: 0,
                            },
                            row_info: None,
                        }
                    });
                if params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id
                    || params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id
                {
                    return $crate::types::TransformResultInternal {
                        m1_prime: Some(m1.clone()),
                        m2_prime: Some(m2.clone()),
                        error: None,
                    };
                }
                let insert_start = m2_params.range.start_row;
                let insert_count = m2_params.range.end_row - m2_params.range.start_row + 1;
                if let Some(ref mut row_data) = params.row_data {
                    let mut new_data = serde_json::Map::new();
                    for (row_key, row_value) in row_data.iter() {
                        if let Ok(row_num) = row_key.parse::<u32>() {
                            let new_key = if row_num >= insert_start {
                                row_num + insert_count
                            } else {
                                row_num
                            };
                            new_data.insert(new_key.to_string(), row_value.clone());
                        } else {
                            new_data.insert(row_key.clone(), row_value.clone());
                        }
                    }
                    *row_data = new_data;
                }
                $crate::types::TransformResultInternal {
                    m1_prime: Some($crate::types::MutationInfoInternal {
                        id: m1.id.clone(),
                        params: serde_json::to_value(params).unwrap(),
                    }),
                    m2_prime: Some(m2.clone()),
                    error: None,
                }
            }

            fn transform_with_insert_col(
                &self,
                m1: &$crate::types::MutationInfoInternal,
                m2: &$crate::types::MutationInfoInternal,
            ) -> $crate::types::TransformResultInternal {
                $crate::types::TransformResultInternal {
                    m1_prime: Some(m1.clone()),
                    m2_prime: Some(m2.clone()),
                    error: None,
                }
            }

            fn transform_with_remove_rows(
                &self,
                m1: &$crate::types::MutationInfoInternal,
                m2: &$crate::types::MutationInfoInternal,
            ) -> $crate::types::TransformResultInternal {
                let mut params: $crate::mutations::sheets::SheetRowColumnDataParams =
                    serde_json::from_value(m1.params.clone()).unwrap_or_else(|_| {
                        $crate::mutations::sheets::SheetRowColumnDataParams {
                            sub_unit_params: $crate::types::SubUnitParams {
                                unit_id: "".to_string(),
                                sub_unit_id: "".to_string(),
                            },
                            row_data: None,
                            column_data: None,
                        }
                    });
                let m2_params: $crate::types::RemoveRowsMutationParams =
                    serde_json::from_value(m2.params.clone()).unwrap_or_else(|_| {
                        $crate::types::RemoveRowsMutationParams {
                            sub_unit_params: $crate::types::SubUnitParams {
                                unit_id: "".to_string(),
                                sub_unit_id: "".to_string(),
                            },
                            range: $crate::types::Range {
                                start_row: 0,
                                start_column: 0,
                                end_row: 0,
                                end_column: 0,
                            },
                        }
                    });
                if params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id
                    || params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id
                {
                    return $crate::types::TransformResultInternal {
                        m1_prime: Some(m1.clone()),
                        m2_prime: Some(m2.clone()),
                        error: None,
                    };
                }
                let remove_start = m2_params.range.start_row;
                let remove_end = m2_params.range.end_row;
                if let Some(ref mut row_data) = params.row_data {
                    let mut new_data = serde_json::Map::new();
                    for (row_key, row_value) in row_data.iter() {
                        if let Ok(row_num) = row_key.parse::<u32>() {
                            if row_num > remove_end {
                                new_data.insert(
                                    (row_num - (remove_end - remove_start + 1)).to_string(),
                                    row_value.clone(),
                                );
                            } else if row_num < remove_start {
                                new_data.insert(row_key.clone(), row_value.clone());
                            }
                        } else {
                            new_data.insert(row_key.clone(), row_value.clone());
                        }
                    }
                    *row_data = new_data;
                }
                $crate::types::TransformResultInternal {
                    m1_prime: Some($crate::types::MutationInfoInternal {
                        id: m1.id.clone(),
                        params: serde_json::to_value(params).unwrap(),
                    }),
                    m2_prime: Some(m2.clone()),
                    error: None,
                }
            }

            fn transform_with_remove_col(
                &self,
                m1: &$crate::types::MutationInfoInternal,
                m2: &$crate::types::MutationInfoInternal,
            ) -> $crate::types::TransformResultInternal {
                $crate::types::TransformResultInternal {
                    m1_prime: Some(m1.clone()),
                    m2_prime: Some(m2.clone()),
                    error: None,
                }
            }

            fn compose(
                &self,
                m1: &$crate::types::MutationInfoInternal,
                m2: &$crate::types::MutationInfoInternal,
            ) -> Vec<$crate::types::MutationInfoInternal> {
                if m1.id != m2.id {
                    return vec![m1.clone(), m2.clone()];
                }
                vec![m2.clone()]
            }
        }
    };
}

#[macro_export]
macro_rules! impl_col_data_transform {
    ($name:ident, $id:expr) => {
        impl crate::transform::mutation_transform::MutationTransform for $name {
            fn mutation_id() -> &'static str {
                $id
            }

            fn transform_with_set_range_values(
                &self,
                m1: &$crate::types::MutationInfoInternal,
                m2: &$crate::types::MutationInfoInternal,
            ) -> $crate::types::TransformResultInternal {
                $crate::types::TransformResultInternal {
                    m1_prime: Some(m1.clone()),
                    m2_prime: Some(m2.clone()),
                    error: None,
                }
            }

            fn transform_with_insert_row(
                &self,
                m1: &$crate::types::MutationInfoInternal,
                m2: &$crate::types::MutationInfoInternal,
            ) -> $crate::types::TransformResultInternal {
                $crate::types::TransformResultInternal {
                    m1_prime: Some(m1.clone()),
                    m2_prime: Some(m2.clone()),
                    error: None,
                }
            }

            fn transform_with_insert_col(
                &self,
                m1: &$crate::types::MutationInfoInternal,
                m2: &$crate::types::MutationInfoInternal,
            ) -> $crate::types::TransformResultInternal {
                let mut params: $crate::mutations::sheets::SheetRowColumnDataParams =
                    serde_json::from_value(m1.params.clone()).unwrap_or_else(|_| {
                        $crate::mutations::sheets::SheetRowColumnDataParams {
                            sub_unit_params: $crate::types::SubUnitParams {
                                unit_id: "".to_string(),
                                sub_unit_id: "".to_string(),
                            },
                            row_data: None,
                            column_data: None,
                        }
                    });
                let m2_params: $crate::types::InsertColMutationParams =
                    serde_json::from_value(m2.params.clone()).unwrap_or_else(|_| {
                        $crate::types::InsertColMutationParams {
                            sub_unit_params: $crate::types::SubUnitParams {
                                unit_id: "".to_string(),
                                sub_unit_id: "".to_string(),
                            },
                            range: $crate::types::Range {
                                start_row: 0,
                                start_column: 0,
                                end_row: 0,
                                end_column: 0,
                            },
                            col_info: None,
                        }
                    });
                if params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id
                    || params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id
                {
                    return $crate::types::TransformResultInternal {
                        m1_prime: Some(m1.clone()),
                        m2_prime: Some(m2.clone()),
                        error: None,
                    };
                }
                let insert_start = m2_params.range.start_column;
                let insert_count = m2_params.range.end_column - m2_params.range.start_column + 1;
                if let Some(ref mut column_data) = params.column_data {
                    let mut new_data = serde_json::Map::new();
                    for (col_key, col_value) in column_data.iter() {
                        if let Ok(col_num) = col_key.parse::<u32>() {
                            let new_key = if col_num >= insert_start {
                                col_num + insert_count
                            } else {
                                col_num
                            };
                            new_data.insert(new_key.to_string(), col_value.clone());
                        } else {
                            new_data.insert(col_key.clone(), col_value.clone());
                        }
                    }
                    *column_data = new_data;
                }
                $crate::types::TransformResultInternal {
                    m1_prime: Some($crate::types::MutationInfoInternal {
                        id: m1.id.clone(),
                        params: serde_json::to_value(params).unwrap(),
                    }),
                    m2_prime: Some(m2.clone()),
                    error: None,
                }
            }

            fn transform_with_remove_rows(
                &self,
                m1: &$crate::types::MutationInfoInternal,
                m2: &$crate::types::MutationInfoInternal,
            ) -> $crate::types::TransformResultInternal {
                $crate::types::TransformResultInternal {
                    m1_prime: Some(m1.clone()),
                    m2_prime: Some(m2.clone()),
                    error: None,
                }
            }

            fn transform_with_remove_col(
                &self,
                m1: &$crate::types::MutationInfoInternal,
                m2: &$crate::types::MutationInfoInternal,
            ) -> $crate::types::TransformResultInternal {
                let mut params: $crate::mutations::sheets::SheetRowColumnDataParams =
                    serde_json::from_value(m1.params.clone()).unwrap_or_else(|_| {
                        $crate::mutations::sheets::SheetRowColumnDataParams {
                            sub_unit_params: $crate::types::SubUnitParams {
                                unit_id: "".to_string(),
                                sub_unit_id: "".to_string(),
                            },
                            row_data: None,
                            column_data: None,
                        }
                    });
                let m2_params: $crate::types::RemoveColMutationParams =
                    serde_json::from_value(m2.params.clone()).unwrap_or_else(|_| {
                        $crate::types::RemoveColMutationParams {
                            sub_unit_params: $crate::types::SubUnitParams {
                                unit_id: "".to_string(),
                                sub_unit_id: "".to_string(),
                            },
                            range: $crate::types::Range {
                                start_row: 0,
                                start_column: 0,
                                end_row: 0,
                                end_column: 0,
                            },
                        }
                    });
                if params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id
                    || params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id
                {
                    return $crate::types::TransformResultInternal {
                        m1_prime: Some(m1.clone()),
                        m2_prime: Some(m2.clone()),
                        error: None,
                    };
                }
                let remove_start = m2_params.range.start_column;
                let remove_end = m2_params.range.end_column;
                if let Some(ref mut column_data) = params.column_data {
                    let mut new_data = serde_json::Map::new();
                    for (col_key, col_value) in column_data.iter() {
                        if let Ok(col_num) = col_key.parse::<u32>() {
                            if col_num > remove_end {
                                new_data.insert(
                                    (col_num - (remove_end - remove_start + 1)).to_string(),
                                    col_value.clone(),
                                );
                            } else if col_num < remove_start {
                                new_data.insert(col_key.clone(), col_value.clone());
                            }
                        } else {
                            new_data.insert(col_key.clone(), col_value.clone());
                        }
                    }
                    *column_data = new_data;
                }
                $crate::types::TransformResultInternal {
                    m1_prime: Some($crate::types::MutationInfoInternal {
                        id: m1.id.clone(),
                        params: serde_json::to_value(params).unwrap(),
                    }),
                    m2_prime: Some(m2.clone()),
                    error: None,
                }
            }

            fn compose(
                &self,
                m1: &$crate::types::MutationInfoInternal,
                m2: &$crate::types::MutationInfoInternal,
            ) -> Vec<$crate::types::MutationInfoInternal> {
                if m1.id != m2.id {
                    return vec![m1.clone(), m2.clone()];
                }
                vec![m2.clone()]
            }
        }
    };
}

#[macro_export]
macro_rules! impl_set_frozen_transform {
    ($name:ident, $id:expr) => {
        impl crate::transform::mutation_transform::MutationTransform for $name {
            fn mutation_id() -> &'static str {
                $id
            }

            fn transform_with_set_range_values(
                &self,
                m1: &$crate::types::MutationInfoInternal,
                m2: &$crate::types::MutationInfoInternal,
            ) -> $crate::types::TransformResultInternal {
                $crate::types::TransformResultInternal {
                    m1_prime: Some(m1.clone()),
                    m2_prime: Some(m2.clone()),
                    error: None,
                }
            }

            fn transform_with_insert_row(
                &self,
                m1: &$crate::types::MutationInfoInternal,
                m2: &$crate::types::MutationInfoInternal,
            ) -> $crate::types::TransformResultInternal {
                let mut params: $crate::mutations::sheets::SheetSetFrozenParams =
                    serde_json::from_value(m1.params.clone()).unwrap_or_else(|_| {
                        $crate::mutations::sheets::SheetSetFrozenParams {
                            sub_unit_params: $crate::types::SubUnitParams {
                                unit_id: "".to_string(),
                                sub_unit_id: "".to_string(),
                            },
                            start_row: 0,
                            start_column: 0,
                            y_split: 0,
                            x_split: 0,
                        }
                    });
                let m2_params: $crate::types::InsertRowMutationParams =
                    serde_json::from_value(m2.params.clone()).unwrap_or_else(|_| {
                        $crate::types::InsertRowMutationParams {
                            sub_unit_params: $crate::types::SubUnitParams {
                                unit_id: "".to_string(),
                                sub_unit_id: "".to_string(),
                            },
                            range: $crate::types::Range {
                                start_row: 0,
                                start_column: 0,
                                end_row: 0,
                                end_column: 0,
                            },
                            row_info: None,
                        }
                    });
                if params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id
                    || params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id
                {
                    return $crate::types::TransformResultInternal {
                        m1_prime: Some(m1.clone()),
                        m2_prime: Some(m2.clone()),
                        error: None,
                    };
                }
                let insert_start = m2_params.range.start_row;
                let insert_count = m2_params.range.end_row - m2_params.range.start_row + 1;
                $crate::transform::sheets_transform_utils::shift_set_frozen_for_insert(
                    &mut params,
                    Some((insert_start, insert_count)),
                    None,
                );
                $crate::types::TransformResultInternal {
                    m1_prime: Some($crate::types::MutationInfoInternal {
                        id: m1.id.clone(),
                        params: serde_json::to_value(params).unwrap(),
                    }),
                    m2_prime: Some(m2.clone()),
                    error: None,
                }
            }

            fn transform_with_insert_col(
                &self,
                m1: &$crate::types::MutationInfoInternal,
                m2: &$crate::types::MutationInfoInternal,
            ) -> $crate::types::TransformResultInternal {
                let mut params: $crate::mutations::sheets::SheetSetFrozenParams =
                    serde_json::from_value(m1.params.clone()).unwrap_or_else(|_| {
                        $crate::mutations::sheets::SheetSetFrozenParams {
                            sub_unit_params: $crate::types::SubUnitParams {
                                unit_id: "".to_string(),
                                sub_unit_id: "".to_string(),
                            },
                            start_row: 0,
                            start_column: 0,
                            y_split: 0,
                            x_split: 0,
                        }
                    });
                let m2_params: $crate::types::InsertColMutationParams =
                    serde_json::from_value(m2.params.clone()).unwrap_or_else(|_| {
                        $crate::types::InsertColMutationParams {
                            sub_unit_params: $crate::types::SubUnitParams {
                                unit_id: "".to_string(),
                                sub_unit_id: "".to_string(),
                            },
                            range: $crate::types::Range {
                                start_row: 0,
                                start_column: 0,
                                end_row: 0,
                                end_column: 0,
                            },
                            col_info: None,
                        }
                    });
                if params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id
                    || params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id
                {
                    return $crate::types::TransformResultInternal {
                        m1_prime: Some(m1.clone()),
                        m2_prime: Some(m2.clone()),
                        error: None,
                    };
                }
                let insert_start = m2_params.range.start_column;
                let insert_count = m2_params.range.end_column - m2_params.range.start_column + 1;
                $crate::transform::sheets_transform_utils::shift_set_frozen_for_insert(
                    &mut params,
                    None,
                    Some((insert_start, insert_count)),
                );
                $crate::types::TransformResultInternal {
                    m1_prime: Some($crate::types::MutationInfoInternal {
                        id: m1.id.clone(),
                        params: serde_json::to_value(params).unwrap(),
                    }),
                    m2_prime: Some(m2.clone()),
                    error: None,
                }
            }

            fn transform_with_remove_rows(
                &self,
                m1: &$crate::types::MutationInfoInternal,
                m2: &$crate::types::MutationInfoInternal,
            ) -> $crate::types::TransformResultInternal {
                let mut params: $crate::mutations::sheets::SheetSetFrozenParams =
                    serde_json::from_value(m1.params.clone()).unwrap_or_else(|_| {
                        $crate::mutations::sheets::SheetSetFrozenParams {
                            sub_unit_params: $crate::types::SubUnitParams {
                                unit_id: "".to_string(),
                                sub_unit_id: "".to_string(),
                            },
                            start_row: 0,
                            start_column: 0,
                            y_split: 0,
                            x_split: 0,
                        }
                    });
                let m2_params: $crate::types::RemoveRowsMutationParams =
                    serde_json::from_value(m2.params.clone()).unwrap_or_else(|_| {
                        $crate::types::RemoveRowsMutationParams {
                            sub_unit_params: $crate::types::SubUnitParams {
                                unit_id: "".to_string(),
                                sub_unit_id: "".to_string(),
                            },
                            range: $crate::types::Range {
                                start_row: 0,
                                start_column: 0,
                                end_row: 0,
                                end_column: 0,
                            },
                        }
                    });
                if params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id
                    || params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id
                {
                    return $crate::types::TransformResultInternal {
                        m1_prime: Some(m1.clone()),
                        m2_prime: Some(m2.clone()),
                        error: None,
                    };
                }
                $crate::transform::sheets_transform_utils::shift_set_frozen_for_remove(
                    &mut params,
                    Some((m2_params.range.start_row, m2_params.range.end_row)),
                    None,
                );
                $crate::types::TransformResultInternal {
                    m1_prime: Some($crate::types::MutationInfoInternal {
                        id: m1.id.clone(),
                        params: serde_json::to_value(params).unwrap(),
                    }),
                    m2_prime: Some(m2.clone()),
                    error: None,
                }
            }

            fn transform_with_remove_col(
                &self,
                m1: &$crate::types::MutationInfoInternal,
                m2: &$crate::types::MutationInfoInternal,
            ) -> $crate::types::TransformResultInternal {
                let mut params: $crate::mutations::sheets::SheetSetFrozenParams =
                    serde_json::from_value(m1.params.clone()).unwrap_or_else(|_| {
                        $crate::mutations::sheets::SheetSetFrozenParams {
                            sub_unit_params: $crate::types::SubUnitParams {
                                unit_id: "".to_string(),
                                sub_unit_id: "".to_string(),
                            },
                            start_row: 0,
                            start_column: 0,
                            y_split: 0,
                            x_split: 0,
                        }
                    });
                let m2_params: $crate::types::RemoveColMutationParams =
                    serde_json::from_value(m2.params.clone()).unwrap_or_else(|_| {
                        $crate::types::RemoveColMutationParams {
                            sub_unit_params: $crate::types::SubUnitParams {
                                unit_id: "".to_string(),
                                sub_unit_id: "".to_string(),
                            },
                            range: $crate::types::Range {
                                start_row: 0,
                                start_column: 0,
                                end_row: 0,
                                end_column: 0,
                            },
                        }
                    });
                if params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id
                    || params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id
                {
                    return $crate::types::TransformResultInternal {
                        m1_prime: Some(m1.clone()),
                        m2_prime: Some(m2.clone()),
                        error: None,
                    };
                }
                $crate::transform::sheets_transform_utils::shift_set_frozen_for_remove(
                    &mut params,
                    None,
                    Some((m2_params.range.start_column, m2_params.range.end_column)),
                );
                $crate::types::TransformResultInternal {
                    m1_prime: Some($crate::types::MutationInfoInternal {
                        id: m1.id.clone(),
                        params: serde_json::to_value(params).unwrap(),
                    }),
                    m2_prime: Some(m2.clone()),
                    error: None,
                }
            }

            fn compose(
                &self,
                m1: &$crate::types::MutationInfoInternal,
                m2: &$crate::types::MutationInfoInternal,
            ) -> Vec<$crate::types::MutationInfoInternal> {
                if m1.id != m2.id {
                    return vec![m1.clone(), m2.clone()];
                }
                vec![m2.clone()]
            }
        }
    };
}

#[macro_export]
macro_rules! impl_move_rows_cols_transform {
    ($name:ident, $id:expr, $is_row:expr) => {
        impl crate::transform::mutation_transform::MutationTransform for $name {
            fn mutation_id() -> &'static str {
                $id
            }

            fn transform_with_set_range_values(
                &self,
                m1: &$crate::types::MutationInfoInternal,
                m2: &$crate::types::MutationInfoInternal,
            ) -> $crate::types::TransformResultInternal {
                $crate::types::TransformResultInternal {
                    m1_prime: Some(m1.clone()),
                    m2_prime: Some(m2.clone()),
                    error: None,
                }
            }

            fn transform_with_insert_row(
                &self,
                m1: &$crate::types::MutationInfoInternal,
                m2: &$crate::types::MutationInfoInternal,
            ) -> $crate::types::TransformResultInternal {
                let mut params: $crate::mutations::sheets::SheetMoveRowsColsParams =
                    serde_json::from_value(m1.params.clone()).unwrap_or_else(|_| {
                        $crate::mutations::sheets::SheetMoveRowsColsParams {
                            sub_unit_params: $crate::types::SubUnitParams {
                                unit_id: "".to_string(),
                                sub_unit_id: "".to_string(),
                            },
                            source_range: $crate::types::Range {
                                start_row: 0,
                                start_column: 0,
                                end_row: 0,
                                end_column: 0,
                            },
                            target_range: $crate::types::Range {
                                start_row: 0,
                                start_column: 0,
                                end_row: 0,
                                end_column: 0,
                            },
                        }
                    });
                let m2_params: $crate::types::InsertRowMutationParams =
                    serde_json::from_value(m2.params.clone()).unwrap_or_else(|_| {
                        $crate::types::InsertRowMutationParams {
                            sub_unit_params: $crate::types::SubUnitParams {
                                unit_id: "".to_string(),
                                sub_unit_id: "".to_string(),
                            },
                            range: $crate::types::Range {
                                start_row: 0,
                                start_column: 0,
                                end_row: 0,
                                end_column: 0,
                            },
                            row_info: None,
                        }
                    });
                if params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id
                    || params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id
                {
                    return $crate::types::TransformResultInternal {
                        m1_prime: Some(m1.clone()),
                        m2_prime: Some(m2.clone()),
                        error: None,
                    };
                }
                if $is_row {
                    let insert_start = m2_params.range.start_row;
                    let insert_count = m2_params.range.end_row - m2_params.range.start_row + 1;
                    $crate::transform::sheets_transform_utils::shift_range_rows_for_insert(
                        &mut params.source_range,
                        insert_start,
                        insert_count,
                    );
                    $crate::transform::sheets_transform_utils::shift_range_rows_for_insert(
                        &mut params.target_range,
                        insert_start,
                        insert_count,
                    );
                }
                $crate::types::TransformResultInternal {
                    m1_prime: Some($crate::types::MutationInfoInternal {
                        id: m1.id.clone(),
                        params: serde_json::to_value(params).unwrap(),
                    }),
                    m2_prime: Some(m2.clone()),
                    error: None,
                }
            }

            fn transform_with_insert_col(
                &self,
                m1: &$crate::types::MutationInfoInternal,
                m2: &$crate::types::MutationInfoInternal,
            ) -> $crate::types::TransformResultInternal {
                let mut params: $crate::mutations::sheets::SheetMoveRowsColsParams =
                    serde_json::from_value(m1.params.clone()).unwrap_or_else(|_| {
                        $crate::mutations::sheets::SheetMoveRowsColsParams {
                            sub_unit_params: $crate::types::SubUnitParams {
                                unit_id: "".to_string(),
                                sub_unit_id: "".to_string(),
                            },
                            source_range: $crate::types::Range {
                                start_row: 0,
                                start_column: 0,
                                end_row: 0,
                                end_column: 0,
                            },
                            target_range: $crate::types::Range {
                                start_row: 0,
                                start_column: 0,
                                end_row: 0,
                                end_column: 0,
                            },
                        }
                    });
                let m2_params: $crate::types::InsertColMutationParams =
                    serde_json::from_value(m2.params.clone()).unwrap_or_else(|_| {
                        $crate::types::InsertColMutationParams {
                            sub_unit_params: $crate::types::SubUnitParams {
                                unit_id: "".to_string(),
                                sub_unit_id: "".to_string(),
                            },
                            range: $crate::types::Range {
                                start_row: 0,
                                start_column: 0,
                                end_row: 0,
                                end_column: 0,
                            },
                            col_info: None,
                        }
                    });
                if params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id
                    || params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id
                {
                    return $crate::types::TransformResultInternal {
                        m1_prime: Some(m1.clone()),
                        m2_prime: Some(m2.clone()),
                        error: None,
                    };
                }
                if !$is_row {
                    let insert_start = m2_params.range.start_column;
                    let insert_count =
                        m2_params.range.end_column - m2_params.range.start_column + 1;
                    $crate::transform::sheets_transform_utils::shift_range_cols_for_insert(
                        &mut params.source_range,
                        insert_start,
                        insert_count,
                    );
                    $crate::transform::sheets_transform_utils::shift_range_cols_for_insert(
                        &mut params.target_range,
                        insert_start,
                        insert_count,
                    );
                }
                $crate::types::TransformResultInternal {
                    m1_prime: Some($crate::types::MutationInfoInternal {
                        id: m1.id.clone(),
                        params: serde_json::to_value(params).unwrap(),
                    }),
                    m2_prime: Some(m2.clone()),
                    error: None,
                }
            }

            fn transform_with_remove_rows(
                &self,
                m1: &$crate::types::MutationInfoInternal,
                m2: &$crate::types::MutationInfoInternal,
            ) -> $crate::types::TransformResultInternal {
                let mut params: $crate::mutations::sheets::SheetMoveRowsColsParams =
                    serde_json::from_value(m1.params.clone()).unwrap_or_else(|_| {
                        $crate::mutations::sheets::SheetMoveRowsColsParams {
                            sub_unit_params: $crate::types::SubUnitParams {
                                unit_id: "".to_string(),
                                sub_unit_id: "".to_string(),
                            },
                            source_range: $crate::types::Range {
                                start_row: 0,
                                start_column: 0,
                                end_row: 0,
                                end_column: 0,
                            },
                            target_range: $crate::types::Range {
                                start_row: 0,
                                start_column: 0,
                                end_row: 0,
                                end_column: 0,
                            },
                        }
                    });
                let m2_params: $crate::types::RemoveRowsMutationParams =
                    serde_json::from_value(m2.params.clone()).unwrap_or_else(|_| {
                        $crate::types::RemoveRowsMutationParams {
                            sub_unit_params: $crate::types::SubUnitParams {
                                unit_id: "".to_string(),
                                sub_unit_id: "".to_string(),
                            },
                            range: $crate::types::Range {
                                start_row: 0,
                                start_column: 0,
                                end_row: 0,
                                end_column: 0,
                            },
                        }
                    });
                if params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id
                    || params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id
                {
                    return $crate::types::TransformResultInternal {
                        m1_prime: Some(m1.clone()),
                        m2_prime: Some(m2.clone()),
                        error: None,
                    };
                }
                if $is_row {
                    if !$crate::transform::sheets_transform_utils::shift_range_rows_for_remove(
                        &mut params.source_range,
                        m2_params.range.start_row,
                        m2_params.range.end_row,
                    ) || !$crate::transform::sheets_transform_utils::shift_range_rows_for_remove(
                        &mut params.target_range,
                        m2_params.range.start_row,
                        m2_params.range.end_row,
                    ) {
                        return $crate::types::TransformResultInternal {
                            m1_prime: None,
                            m2_prime: Some(m2.clone()),
                            error: None,
                        };
                    }
                }
                $crate::types::TransformResultInternal {
                    m1_prime: Some($crate::types::MutationInfoInternal {
                        id: m1.id.clone(),
                        params: serde_json::to_value(params).unwrap(),
                    }),
                    m2_prime: Some(m2.clone()),
                    error: None,
                }
            }

            fn transform_with_remove_col(
                &self,
                m1: &$crate::types::MutationInfoInternal,
                m2: &$crate::types::MutationInfoInternal,
            ) -> $crate::types::TransformResultInternal {
                let mut params: $crate::mutations::sheets::SheetMoveRowsColsParams =
                    serde_json::from_value(m1.params.clone()).unwrap_or_else(|_| {
                        $crate::mutations::sheets::SheetMoveRowsColsParams {
                            sub_unit_params: $crate::types::SubUnitParams {
                                unit_id: "".to_string(),
                                sub_unit_id: "".to_string(),
                            },
                            source_range: $crate::types::Range {
                                start_row: 0,
                                start_column: 0,
                                end_row: 0,
                                end_column: 0,
                            },
                            target_range: $crate::types::Range {
                                start_row: 0,
                                start_column: 0,
                                end_row: 0,
                                end_column: 0,
                            },
                        }
                    });
                let m2_params: $crate::types::RemoveColMutationParams =
                    serde_json::from_value(m2.params.clone()).unwrap_or_else(|_| {
                        $crate::types::RemoveColMutationParams {
                            sub_unit_params: $crate::types::SubUnitParams {
                                unit_id: "".to_string(),
                                sub_unit_id: "".to_string(),
                            },
                            range: $crate::types::Range {
                                start_row: 0,
                                start_column: 0,
                                end_row: 0,
                                end_column: 0,
                            },
                        }
                    });
                if params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id
                    || params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id
                {
                    return $crate::types::TransformResultInternal {
                        m1_prime: Some(m1.clone()),
                        m2_prime: Some(m2.clone()),
                        error: None,
                    };
                }
                if !$is_row {
                    if !$crate::transform::sheets_transform_utils::shift_range_cols_for_remove(
                        &mut params.source_range,
                        m2_params.range.start_column,
                        m2_params.range.end_column,
                    ) || !$crate::transform::sheets_transform_utils::shift_range_cols_for_remove(
                        &mut params.target_range,
                        m2_params.range.start_column,
                        m2_params.range.end_column,
                    ) {
                        return $crate::types::TransformResultInternal {
                            m1_prime: None,
                            m2_prime: Some(m2.clone()),
                            error: None,
                        };
                    }
                }
                $crate::types::TransformResultInternal {
                    m1_prime: Some($crate::types::MutationInfoInternal {
                        id: m1.id.clone(),
                        params: serde_json::to_value(params).unwrap(),
                    }),
                    m2_prime: Some(m2.clone()),
                    error: None,
                }
            }

            fn compose(
                &self,
                m1: &$crate::types::MutationInfoInternal,
                m2: &$crate::types::MutationInfoInternal,
            ) -> Vec<$crate::types::MutationInfoInternal> {
                if m1.id != m2.id {
                    return vec![m1.clone(), m2.clone()];
                }
                vec![m2.clone()]
            }
        }
    };
}

pub(crate) use crate::{
    impl_col_data_transform, impl_identity_transform, impl_move_rows_cols_transform,
    impl_range_transform, impl_ranges_transform, impl_row_data_transform,
    impl_set_frozen_transform,
};
