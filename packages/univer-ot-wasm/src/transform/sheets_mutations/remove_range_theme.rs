use crate::transform::mutation_transform::MutationTransform;
use crate::transform::sheets_mutations::common::same_sheet;
use crate::types::{MutationInfoInternal, SubUnitParams, TransformResultInternal};

#[derive(Default)]
pub struct RemoveRangeThemeTransform;

fn parse_sub_unit_params(params: &serde_json::Value) -> SubUnitParams {
    serde_json::from_value(params.clone()).unwrap_or_else(|_| SubUnitParams {
        unit_id: params
            .get("unitId")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        sub_unit_id: params
            .get("subUnitId")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
    })
}

fn same_workbook_or_sheet(m1: &SubUnitParams, m2: &SubUnitParams) -> bool {
    if m1.sub_unit_id.is_empty() {
        m1.unit_id == m2.unit_id
    } else {
        same_sheet(m1, m2)
    }
}

fn parse_theme_name(params: &serde_json::Value) -> Option<String> {
    params
        .get("styleName")
        .and_then(|v| v.as_str())
        .map(|name| name.to_string())
}

impl MutationTransform for RemoveRangeThemeTransform {
    fn mutation_id() -> &'static str {
        "sheet.mutation.remove-range-theme"
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

    fn transform_with_add_range_theme(
        &self,
        m1: &MutationInfoInternal,
        m2: &MutationInfoInternal,
    ) -> TransformResultInternal {
        let m1_params = m1.params.clone();
        let m2_params = m2.params.clone();
        let m1_unit = parse_sub_unit_params(&m1_params);
        let m2_unit = parse_sub_unit_params(&m2_params);
        if !same_workbook_or_sheet(&m1_unit, &m2_unit) {
            return TransformResultInternal {
                m1_prime: Some(m1.clone()),
                m2_prime: Some(m2.clone()),
                error: None,
            };
        }
        let remove_name = parse_theme_name(&m1_params);
        let add_name = m2_params
            .get("styleJSON")
            .and_then(|v| v.get("name"))
            .and_then(|v| v.as_str())
            .map(|name| name.to_string());
        if remove_name.is_some() && remove_name == add_name {
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

    fn transform_with_set_range_theme(
        &self,
        m1: &MutationInfoInternal,
        m2: &MutationInfoInternal,
    ) -> TransformResultInternal {
        let m1_params = m1.params.clone();
        let m2_params = m2.params.clone();
        let m1_unit = parse_sub_unit_params(&m1_params);
        let m2_unit = parse_sub_unit_params(&m2_params);
        if !same_workbook_or_sheet(&m1_unit, &m2_unit) {
            return TransformResultInternal {
                m1_prime: Some(m1.clone()),
                m2_prime: Some(m2.clone()),
                error: None,
            };
        }
        let remove_name = parse_theme_name(&m1_params);
        let set_name = m2_params
            .get("styleName")
            .and_then(|v| v.as_str())
            .map(|name| name.to_string());
        if remove_name.is_some() && remove_name == set_name {
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
