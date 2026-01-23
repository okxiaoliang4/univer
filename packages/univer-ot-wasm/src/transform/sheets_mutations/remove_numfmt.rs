use crate::transform::mutation_transform::MutationTransform;
use crate::transform::sheets_mutations::common::{
    parse_ranges_params, ranges_intersect, same_sheet, transform_ranges_insert_col,
    transform_ranges_insert_row, transform_ranges_remove_cols, transform_ranges_remove_rows,
    transform_ranges_set_range_values,
};
use crate::types::{MutationInfoInternal, TransformResultInternal};

#[derive(Default)]
pub struct RemoveNumfmtTransform;

impl MutationTransform for RemoveNumfmtTransform {
    fn mutation_id() -> &'static str {
        "sheet.mutation.remove.numfmt"
    }

    fn transform_with_set_range_values(
        &self,
        m1: &MutationInfoInternal,
        m2: &MutationInfoInternal,
    ) -> TransformResultInternal {
        transform_ranges_set_range_values(m1, m2)
    }

    fn transform_with_insert_row(
        &self,
        m1: &MutationInfoInternal,
        m2: &MutationInfoInternal,
    ) -> TransformResultInternal {
        transform_ranges_insert_row(m1, m2)
    }

    fn transform_with_insert_col(
        &self,
        m1: &MutationInfoInternal,
        m2: &MutationInfoInternal,
    ) -> TransformResultInternal {
        transform_ranges_insert_col(m1, m2)
    }

    fn transform_with_remove_rows(
        &self,
        m1: &MutationInfoInternal,
        m2: &MutationInfoInternal,
    ) -> TransformResultInternal {
        transform_ranges_remove_rows(m1, m2)
    }

    fn transform_with_remove_col(
        &self,
        m1: &MutationInfoInternal,
        m2: &MutationInfoInternal,
    ) -> TransformResultInternal {
        transform_ranges_remove_cols(m1, m2)
    }

    fn transform_with_set_numfmt(
        &self,
        m1: &MutationInfoInternal,
        m2: &MutationInfoInternal,
    ) -> TransformResultInternal {
        let params = parse_ranges_params(&m1.id, &m1.params);
        if params.ranges.is_empty() {
            return TransformResultInternal {
                m1_prime: Some(m1.clone()),
                m2_prime: Some(m2.clone()),
                error: None,
            };
        }
        let m2_params: serde_json::Value = m2.params.clone();
        let m2_unit = m2_params
            .get("unitId")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let m2_sub_unit = m2_params
            .get("subUnitId")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let m2_sub_unit_params = crate::types::SubUnitParams {
            unit_id: m2_unit.to_string(),
            sub_unit_id: m2_sub_unit.to_string(),
        };
        if !same_sheet(&params.sub_unit_params, &m2_sub_unit_params) {
            return TransformResultInternal {
                m1_prime: Some(m1.clone()),
                m2_prime: Some(m2.clone()),
                error: None,
            };
        }
        let values = match m2_params.get("values").and_then(|v| v.as_object()) {
            Some(values) => values,
            None => {
                return TransformResultInternal {
                    m1_prime: Some(m1.clone()),
                    m2_prime: Some(m2.clone()),
                    error: None,
                };
            }
        };
        let mut has_intersection = false;
        for range in &params.ranges {
            for value in values.values() {
                if let Some(ranges) = value.get("ranges").and_then(|v| v.as_array()) {
                    for set_range in ranges {
                        let set_range: crate::types::Range = serde_json::from_value(
                            set_range.clone(),
                        )
                        .unwrap_or(crate::types::Range {
                            start_row: 0,
                            start_column: 0,
                            end_row: 0,
                            end_column: 0,
                        });
                        if ranges_intersect(range, &set_range) {
                            has_intersection = true;
                            break;
                        }
                    }
                }
                if has_intersection {
                    break;
                }
            }
            if has_intersection {
                break;
            }
        }
        if has_intersection {
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
