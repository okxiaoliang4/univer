/**
 * Copyright 2023-present DreamNum Co., Ltd.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use crate::transform::mutation_transform::MutationTransform;
use crate::transform::sheets_transform_utils::{shift_range_cols_for_insert, shift_range_cols_for_remove, shift_range_rows_for_insert, shift_range_rows_for_remove};
use crate::types::{
    InsertColMutationParams, InsertRowMutationParams, MutationInfoInternal, Range,
    RemoveColMutationParams, RemoveRowsMutationParams, SubUnitParams,
    TransformResultInternal,
};
use serde_json;

#[derive(Default)]
pub struct AddDataValidationTransform;

impl MutationTransform for AddDataValidationTransform {
    fn mutation_id() -> &'static str {
        "data-validation.mutation.addRule"
    }

    fn transform_with_set_range_values(
        &self,
        m1: &MutationInfoInternal,
        m2: &MutationInfoInternal,
    ) -> TransformResultInternal {
        // Set range values doesn't affect data validation rules
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

        let mut params = m1.params.clone();
        if let Some(ref mut rule) = params.get_mut("rule") {
            if let Some(rule_obj) = rule.as_object_mut() {
                if let Some(unit_id) = rule_obj.get("unitId").and_then(|v| v.as_str()) {
                    if unit_id != m2_params.sub_unit_params.unit_id {
                        return TransformResultInternal {
                            m1_prime: Some(m1.clone()),
                            m2_prime: Some(m2.clone()),
                            error: None,
                        };
                    }
                }
                if let Some(sub_unit_id) = rule_obj.get("subUnitId").and_then(|v| v.as_str()) {
                    if sub_unit_id != m2_params.sub_unit_params.sub_unit_id {
                        return TransformResultInternal {
                            m1_prime: Some(m1.clone()),
                            m2_prime: Some(m2.clone()),
                            error: None,
                        };
                    }
                }

                if let Some(ranges) = rule_obj.get_mut("ranges") {
                    if let Some(ranges_array) = ranges.as_array_mut() {
                        let insert_start = m2_params.range.start_row;
                        let insert_count = m2_params.range.end_row - m2_params.range.start_row + 1;

                        for range_value in ranges_array.iter_mut() {
                            if let Some(range_obj) = range_value.as_object_mut() {
                                let mut range = Range {
                                    start_row: range_obj
                                        .get("startRow")
                                        .and_then(|v| v.as_u64())
                                        .unwrap_or(0) as u32,
                                    start_column: range_obj
                                        .get("startColumn")
                                        .and_then(|v| v.as_u64())
                                        .unwrap_or(0) as u32,
                                    end_row: range_obj
                                        .get("endRow")
                                        .and_then(|v| v.as_u64())
                                        .unwrap_or(0) as u32,
                                    end_column: range_obj
                                        .get("endColumn")
                                        .and_then(|v| v.as_u64())
                                        .unwrap_or(0) as u32,
                                };

                                shift_range_rows_for_insert(&mut range, insert_start, insert_count);

                                range_obj.insert("startRow".to_string(), serde_json::json!(range.start_row));
                                range_obj.insert("startColumn".to_string(), serde_json::json!(range.start_column));
                                range_obj.insert("endRow".to_string(), serde_json::json!(range.end_row));
                                range_obj.insert("endColumn".to_string(), serde_json::json!(range.end_column));
                            }
                        }
                    }
                }
            }
        }

        TransformResultInternal {
            m1_prime: Some(MutationInfoInternal {
                id: m1.id.clone(),
                params,
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

        let mut params = m1.params.clone();
        if let Some(ref mut rule) = params.get_mut("rule") {
            if let Some(rule_obj) = rule.as_object_mut() {
                if let Some(unit_id) = rule_obj.get("unitId").and_then(|v| v.as_str()) {
                    if unit_id != m2_params.sub_unit_params.unit_id {
                        return TransformResultInternal {
                            m1_prime: Some(m1.clone()),
                            m2_prime: Some(m2.clone()),
                            error: None,
                        };
                    }
                }
                if let Some(sub_unit_id) = rule_obj.get("subUnitId").and_then(|v| v.as_str()) {
                    if sub_unit_id != m2_params.sub_unit_params.sub_unit_id {
                        return TransformResultInternal {
                            m1_prime: Some(m1.clone()),
                            m2_prime: Some(m2.clone()),
                            error: None,
                        };
                    }
                }

                if let Some(ranges) = rule_obj.get_mut("ranges") {
                    if let Some(ranges_array) = ranges.as_array_mut() {
                        let insert_start = m2_params.range.start_column;
                        let insert_count = m2_params.range.end_column - m2_params.range.start_column + 1;

                        for range_value in ranges_array.iter_mut() {
                            if let Some(range_obj) = range_value.as_object_mut() {
                                let mut range = Range {
                                    start_row: range_obj
                                        .get("startRow")
                                        .and_then(|v| v.as_u64())
                                        .unwrap_or(0) as u32,
                                    start_column: range_obj
                                        .get("startColumn")
                                        .and_then(|v| v.as_u64())
                                        .unwrap_or(0) as u32,
                                    end_row: range_obj
                                        .get("endRow")
                                        .and_then(|v| v.as_u64())
                                        .unwrap_or(0) as u32,
                                    end_column: range_obj
                                        .get("endColumn")
                                        .and_then(|v| v.as_u64())
                                        .unwrap_or(0) as u32,
                                };

                                shift_range_cols_for_insert(&mut range, insert_start, insert_count);

                                range_obj.insert("startRow".to_string(), serde_json::json!(range.start_row));
                                range_obj.insert("startColumn".to_string(), serde_json::json!(range.start_column));
                                range_obj.insert("endRow".to_string(), serde_json::json!(range.end_row));
                                range_obj.insert("endColumn".to_string(), serde_json::json!(range.end_column));
                            }
                        }
                    }
                }
            }
        }

        TransformResultInternal {
            m1_prime: Some(MutationInfoInternal {
                id: m1.id.clone(),
                params,
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

        let mut params = m1.params.clone();
        let mut should_keep = false;

        if let Some(ref mut rule) = params.get_mut("rule") {
            if let Some(rule_obj) = rule.as_object_mut() {
                if let Some(unit_id) = rule_obj.get("unitId").and_then(|v| v.as_str()) {
                    if unit_id != m2_params.sub_unit_params.unit_id {
                        return TransformResultInternal {
                            m1_prime: Some(m1.clone()),
                            m2_prime: Some(m2.clone()),
                            error: None,
                        };
                    }
                }
                if let Some(sub_unit_id) = rule_obj.get("subUnitId").and_then(|v| v.as_str()) {
                    if sub_unit_id != m2_params.sub_unit_params.sub_unit_id {
                        return TransformResultInternal {
                            m1_prime: Some(m1.clone()),
                            m2_prime: Some(m2.clone()),
                            error: None,
                        };
                    }
                }

                if let Some(ranges) = rule_obj.get_mut("ranges") {
                    if let Some(ranges_array) = ranges.as_array_mut() {
                        let remove_start = m2_params.range.start_row;
                        let remove_end = m2_params.range.end_row;

                        // Filter ranges that don't overlap with removed rows
                        let mut new_ranges = Vec::new();
                        for range_value in ranges_array.iter() {
                            if let Some(range_obj) = range_value.as_object() {
                                let mut range = Range {
                                    start_row: range_obj
                                        .get("startRow")
                                        .and_then(|v| v.as_u64())
                                        .unwrap_or(0) as u32,
                                    start_column: range_obj
                                        .get("startColumn")
                                        .and_then(|v| v.as_u64())
                                        .unwrap_or(0) as u32,
                                    end_row: range_obj
                                        .get("endRow")
                                        .and_then(|v| v.as_u64())
                                        .unwrap_or(0) as u32,
                                    end_column: range_obj
                                        .get("endColumn")
                                        .and_then(|v| v.as_u64())
                                        .unwrap_or(0) as u32,
                                };

                                if shift_range_rows_for_remove(&mut range, remove_start, remove_end) {
                                    should_keep = true;
                                    new_ranges.push(serde_json::json!({
                                        "startRow": range.start_row,
                                        "startColumn": range.start_column,
                                        "endRow": range.end_row,
                                        "endColumn": range.end_column,
                                    }));
                                }
                            }
                        }

                        *ranges = serde_json::Value::Array(new_ranges);
                    }
                }
            }
        }

        if !should_keep {
            return TransformResultInternal {
                m1_prime: None,
                m2_prime: Some(m2.clone()),
                error: None,
            };
        }

        TransformResultInternal {
            m1_prime: Some(MutationInfoInternal {
                id: m1.id.clone(),
                params,
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

        let mut params = m1.params.clone();
        let mut should_keep = false;

        if let Some(ref mut rule) = params.get_mut("rule") {
            if let Some(rule_obj) = rule.as_object_mut() {
                if let Some(unit_id) = rule_obj.get("unitId").and_then(|v| v.as_str()) {
                    if unit_id != m2_params.sub_unit_params.unit_id {
                        return TransformResultInternal {
                            m1_prime: Some(m1.clone()),
                            m2_prime: Some(m2.clone()),
                            error: None,
                        };
                    }
                }
                if let Some(sub_unit_id) = rule_obj.get("subUnitId").and_then(|v| v.as_str()) {
                    if sub_unit_id != m2_params.sub_unit_params.sub_unit_id {
                        return TransformResultInternal {
                            m1_prime: Some(m1.clone()),
                            m2_prime: Some(m2.clone()),
                            error: None,
                        };
                    }
                }

                if let Some(ranges) = rule_obj.get_mut("ranges") {
                    if let Some(ranges_array) = ranges.as_array_mut() {
                        let remove_start = m2_params.range.start_column;
                        let remove_end = m2_params.range.end_column;

                        // Filter ranges that don't overlap with removed columns
                        let mut new_ranges = Vec::new();
                        for range_value in ranges_array.iter() {
                            if let Some(range_obj) = range_value.as_object() {
                                let mut range = Range {
                                    start_row: range_obj
                                        .get("startRow")
                                        .and_then(|v| v.as_u64())
                                        .unwrap_or(0) as u32,
                                    start_column: range_obj
                                        .get("startColumn")
                                        .and_then(|v| v.as_u64())
                                        .unwrap_or(0) as u32,
                                    end_row: range_obj
                                        .get("endRow")
                                        .and_then(|v| v.as_u64())
                                        .unwrap_or(0) as u32,
                                    end_column: range_obj
                                        .get("endColumn")
                                        .and_then(|v| v.as_u64())
                                        .unwrap_or(0) as u32,
                                };

                                if shift_range_cols_for_remove(&mut range, remove_start, remove_end) {
                                    should_keep = true;
                                    new_ranges.push(serde_json::json!({
                                        "startRow": range.start_row,
                                        "startColumn": range.start_column,
                                        "endRow": range.end_row,
                                        "endColumn": range.end_column,
                                    }));
                                }
                            }
                        }

                        *ranges = serde_json::Value::Array(new_ranges);
                    }
                }
            }
        }

        if !should_keep {
            return TransformResultInternal {
                m1_prime: None,
                m2_prime: Some(m2.clone()),
                error: None,
            };
        }

        TransformResultInternal {
            m1_prime: Some(MutationInfoInternal {
                id: m1.id.clone(),
                params,
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
