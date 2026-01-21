use crate::transform::mutation_transform::MutationTransform;
use crate::transform::sheets_mutations::common::{
    parse_ranges_params, ranges_intersect, same_sheet, transform_ranges_insert_col,
    transform_ranges_insert_row, transform_ranges_remove_cols, transform_ranges_remove_rows,
    transform_ranges_set_range_values,
};
use crate::types::{MutationInfoInternal, TransformResultInternal};

#[derive(Default)]
pub struct RemoveWorksheetMergeTransform;

impl MutationTransform for RemoveWorksheetMergeTransform {
    fn mutation_id() -> &'static str {
        "sheet.mutation.remove-worksheet-merge"
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

    fn transform_with_add_worksheet_merge(
        &self,
        m1: &MutationInfoInternal,
        m2: &MutationInfoInternal,
    ) -> TransformResultInternal {
        let params = parse_ranges_params(&m1.id, &m1.params);
        let m2_params = parse_ranges_params(&m2.id, &m2.params);
        if !same_sheet(&params.sub_unit_params, &m2_params.sub_unit_params) {
            return TransformResultInternal {
                m1_prime: Some(m1.clone()),
                m2_prime: Some(m2.clone()),
                error: None,
            };
        }
        let mut has_intersection = false;
        for range in &params.ranges {
            for other in &m2_params.ranges {
                if ranges_intersect(range, other) {
                    has_intersection = true;
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
