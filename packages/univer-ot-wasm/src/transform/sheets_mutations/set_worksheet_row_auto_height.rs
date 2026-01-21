use crate::transform::mutation_transform::MutationTransform;
use crate::transform::sheets_mutations::common::{
    shift_rows_auto_height_for_insert, shift_rows_auto_height_for_remove,
};
use crate::types::{
    InsertRowMutationParams, MutationInfoInternal, Range, RemoveRowsMutationParams, SubUnitParams,
    TransformResultInternal,
};
use serde_json;

#[derive(Default)]
pub struct SetWorksheetRowAutoHeightTransform;

impl MutationTransform for SetWorksheetRowAutoHeightTransform {
    fn mutation_id() -> &'static str {
        "sheet.mutation.set-worksheet-row-auto-height"
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
        if params.get("unitId").and_then(|v| v.as_str()) != Some(&m2_params.sub_unit_params.unit_id)
            || params.get("subUnitId").and_then(|v| v.as_str())
                != Some(&m2_params.sub_unit_params.sub_unit_id)
        {
            return TransformResultInternal {
                m1_prime: Some(m1.clone()),
                m2_prime: Some(m2.clone()),
                error: None,
            };
        }
        let insert_start = m2_params.range.start_row;
        let insert_count = m2_params.range.end_row - m2_params.range.start_row + 1;
        if let Some(rows) = params.get_mut("rowsAutoHeightInfo") {
            shift_rows_auto_height_for_insert(rows, insert_start, insert_count);
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
        if params.get("unitId").and_then(|v| v.as_str()) != Some(&m2_params.sub_unit_params.unit_id)
            || params.get("subUnitId").and_then(|v| v.as_str())
                != Some(&m2_params.sub_unit_params.sub_unit_id)
        {
            return TransformResultInternal {
                m1_prime: Some(m1.clone()),
                m2_prime: Some(m2.clone()),
                error: None,
            };
        }
        if let Some(rows) = params.get_mut("rowsAutoHeightInfo") {
            if !shift_rows_auto_height_for_remove(
                rows,
                m2_params.range.start_row,
                m2_params.range.end_row,
            ) {
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
