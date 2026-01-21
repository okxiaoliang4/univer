use crate::transform::mutation_transform::MutationTransform;
use crate::transform::sheets_mutations::common::same_sheet;
use crate::types::{MutationInfoInternal, SubUnitParams, TransformResultInternal};

#[derive(Default)]
pub struct DeleteRangeProtectionTransform;

fn parse_sub_unit_params(params: &serde_json::Value) -> SubUnitParams {
    serde_json::from_value(params.clone()).unwrap_or_else(|_| SubUnitParams {
        unit_id: "".to_string(),
        sub_unit_id: "".to_string(),
    })
}

fn parse_rule_ids(params: &serde_json::Value) -> Vec<String> {
    params
        .get("ruleIds")
        .and_then(|v| v.as_array())
        .map(|ids| {
            ids.iter()
                .filter_map(|value| value.as_str().map(|id| id.to_string()))
                .collect()
        })
        .unwrap_or_default()
}

impl MutationTransform for DeleteRangeProtectionTransform {
    fn mutation_id() -> &'static str {
        "sheet.mutation.delete-range-protection"
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
        TransformResultInternal {
            m1_prime: Some(m1.clone()),
            m2_prime: Some(m2.clone()),
            error: None,
        }
    }

    fn transform_with_insert_col(
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

    fn transform_with_remove_rows(
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

    fn transform_with_remove_col(
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

    fn transform_with_add_range_protection(
        &self,
        m1: &MutationInfoInternal,
        m2: &MutationInfoInternal,
    ) -> TransformResultInternal {
        let m1_params = m1.params.clone();
        let m2_params = m2.params.clone();
        let m1_unit = parse_sub_unit_params(&m1_params);
        let m2_unit = parse_sub_unit_params(&m2_params);
        if !same_sheet(&m1_unit, &m2_unit) {
            return TransformResultInternal {
                m1_prime: Some(m1.clone()),
                m2_prime: Some(m2.clone()),
                error: None,
            };
        }
        let rule_ids = parse_rule_ids(&m1_params);
        if rule_ids.is_empty() {
            return TransformResultInternal {
                m1_prime: Some(m1.clone()),
                m2_prime: Some(m2.clone()),
                error: None,
            };
        }
        let mut has_match = false;
        if let Some(rules) = m2_params.get("rules").and_then(|v| v.as_array()) {
            for rule in rules {
                if let Some(rule_id) = rule.get("id").and_then(|v| v.as_str()) {
                    if rule_ids.iter().any(|id| id == rule_id) {
                        has_match = true;
                        break;
                    }
                }
            }
        }
        if has_match {
            return TransformResultInternal {
                m1_prime: Some(m1.clone()),
                m2_prime: None,
                error: None,
            };
        }
        TransformResultInternal {
            m1_prime: Some(m1.clone()),
            m2_prime: Some(m2.clone()),
            error: None,
        }
    }

    fn transform_with_set_range_protection(
        &self,
        m1: &MutationInfoInternal,
        m2: &MutationInfoInternal,
    ) -> TransformResultInternal {
        let m1_params = m1.params.clone();
        let m2_params = m2.params.clone();
        let m1_unit = parse_sub_unit_params(&m1_params);
        let m2_unit = parse_sub_unit_params(&m2_params);
        if !same_sheet(&m1_unit, &m2_unit) {
            return TransformResultInternal {
                m1_prime: Some(m1.clone()),
                m2_prime: Some(m2.clone()),
                error: None,
            };
        }
        let rule_ids = parse_rule_ids(&m1_params);
        if rule_ids.is_empty() {
            return TransformResultInternal {
                m1_prime: Some(m1.clone()),
                m2_prime: Some(m2.clone()),
                error: None,
            };
        }
        let set_rule_id = m2_params.get("ruleId").and_then(|v| v.as_str());
        if let Some(rule_id) = set_rule_id {
            if rule_ids.iter().any(|id| id == rule_id) {
                return TransformResultInternal {
                    m1_prime: Some(m1.clone()),
                    m2_prime: None,
                    error: None,
                };
            }
        }
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
        if m1.id != m2.id {
            return vec![m1.clone(), m2.clone()];
        }
        vec![m2.clone()]
    }
}
