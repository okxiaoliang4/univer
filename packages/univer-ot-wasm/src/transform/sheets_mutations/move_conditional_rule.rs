use crate::transform::mutation_transform::MutationTransform;
use crate::transform::sheets_mutations::common::{
    shift_rule_ranges_for_insert, shift_rule_ranges_for_remove, same_sheet,
};
use crate::types::{
    InsertColMutationParams, InsertRowMutationParams, MutationInfoInternal, Range,
    RemoveColMutationParams, RemoveRowsMutationParams, SubUnitParams, TransformResultInternal,
};
use serde_json;

#[derive(Default)]
pub struct MoveConditionalRuleTransform;

fn parse_sub_unit_params(params: &serde_json::Value) -> SubUnitParams {
    serde_json::from_value(params.clone()).unwrap_or_else(|_| SubUnitParams {
        unit_id: "".to_string(),
        sub_unit_id: "".to_string(),
    })
}

impl MutationTransform for MoveConditionalRuleTransform {
    fn mutation_id() -> &'static str {
        "sheet.mutation.move-conditional-rule"
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
        let mut params: serde_json::Value = m1.params.clone();
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
        let m1_unit = parse_sub_unit_params(&params);
        if !same_sheet(&m1_unit, &m2_params.sub_unit_params) {
            return TransformResultInternal {
                m1_prime: Some(m1.clone()),
                m2_prime: Some(m2.clone()),
                error: None,
            };
        }
        let insert_start = m2_params.range.start_row;
        let insert_count = m2_params.range.end_row - m2_params.range.start_row + 1;
        if let Some(rule) = params.get_mut("rule") {
            shift_rule_ranges_for_insert(rule, Some((insert_start, insert_count)), None);
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
        let mut params: serde_json::Value = m1.params.clone();
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
        let m1_unit = parse_sub_unit_params(&params);
        if !same_sheet(&m1_unit, &m2_params.sub_unit_params) {
            return TransformResultInternal {
                m1_prime: Some(m1.clone()),
                m2_prime: Some(m2.clone()),
                error: None,
            };
        }
        let insert_start = m2_params.range.start_column;
        let insert_count = m2_params.range.end_column - m2_params.range.start_column + 1;
        if let Some(rule) = params.get_mut("rule") {
            shift_rule_ranges_for_insert(rule, None, Some((insert_start, insert_count)));
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
        let mut params: serde_json::Value = m1.params.clone();
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
        let m1_unit = parse_sub_unit_params(&params);
        if !same_sheet(&m1_unit, &m2_params.sub_unit_params) {
            return TransformResultInternal {
                m1_prime: Some(m1.clone()),
                m2_prime: Some(m2.clone()),
                error: None,
            };
        }
        let remove_start = m2_params.range.start_row;
        let remove_end = m2_params.range.end_row;
        if let Some(rule) = params.get_mut("rule") {
            if !shift_rule_ranges_for_remove(rule, Some((remove_start, remove_end)), None) {
                return TransformResultInternal {
                    m1_prime: None,
                    m2_prime: Some(m2.clone()),
                    error: None,
                };
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

    fn transform_with_remove_col(
        &self,
        m1: &MutationInfoInternal,
        m2: &MutationInfoInternal,
    ) -> TransformResultInternal {
        let mut params: serde_json::Value = m1.params.clone();
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
        let m1_unit = parse_sub_unit_params(&params);
        if !same_sheet(&m1_unit, &m2_params.sub_unit_params) {
            return TransformResultInternal {
                m1_prime: Some(m1.clone()),
                m2_prime: Some(m2.clone()),
                error: None,
            };
        }
        let remove_start = m2_params.range.start_column;
        let remove_end = m2_params.range.end_column;
        if let Some(rule) = params.get_mut("rule") {
            if !shift_rule_ranges_for_remove(rule, None, Some((remove_start, remove_end))) {
                return TransformResultInternal {
                    m1_prime: None,
                    m2_prime: Some(m2.clone()),
                    error: None,
                };
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
