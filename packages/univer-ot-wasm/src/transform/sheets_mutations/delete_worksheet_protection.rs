use crate::transform::mutation_transform::MutationTransform;
use crate::transform::sheets_mutations::common::same_sheet;
use crate::types::{MutationInfoInternal, SubUnitParams, TransformResultInternal};

#[derive(Default)]
pub struct DeleteWorksheetProtectionTransform;

fn parse_sub_unit_params(params: &serde_json::Value) -> SubUnitParams {
    serde_json::from_value(params.clone()).unwrap_or_else(|_| SubUnitParams {
        unit_id: "".to_string(),
        sub_unit_id: "".to_string(),
    })
}

impl MutationTransform for DeleteWorksheetProtectionTransform {
    fn mutation_id() -> &'static str {
        "sheet.mutation.delete-worksheet-protection"
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

    fn transform_with_add_worksheet_protection(
        &self,
        m1: &MutationInfoInternal,
        m2: &MutationInfoInternal,
    ) -> TransformResultInternal {
        let m1_params = m1.params.clone();
        let m2_params = m2.params.clone();
        let m1_unit = parse_sub_unit_params(&m1_params);
        let m2_unit = parse_sub_unit_params(&m2_params);
        if same_sheet(&m1_unit, &m2_unit) {
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

    fn transform_with_set_worksheet_protection(
        &self,
        m1: &MutationInfoInternal,
        m2: &MutationInfoInternal,
    ) -> TransformResultInternal {
        let m1_params = m1.params.clone();
        let m2_params = m2.params.clone();
        let m1_unit = parse_sub_unit_params(&m1_params);
        let m2_unit = parse_sub_unit_params(&m2_params);
        if same_sheet(&m1_unit, &m2_unit) {
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
