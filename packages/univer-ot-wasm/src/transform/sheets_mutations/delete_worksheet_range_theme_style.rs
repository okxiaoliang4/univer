use crate::mutations::sheets::SheetMutationRangeParams;
use crate::transform::mutation_transform::MutationTransform;
use crate::transform::sheets_mutations::common::{ranges_intersect, same_sheet};
use crate::transform::sheets_transform_utils::parse_range_params;
use crate::types::{MutationInfoInternal, TransformResultInternal};

#[derive(Default)]
pub struct DeleteWorksheetRangeThemeStyleTransform;

impl MutationTransform for DeleteWorksheetRangeThemeStyleTransform {
    fn mutation_id() -> &'static str {
        "sheet.mutation.remove-worksheet-range-theme-style"
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
        let mut params: SheetMutationRangeParams = parse_range_params(&m1.id, &m1.params);
        let m2_params: crate::types::InsertRowMutationParams =
            serde_json::from_value(m2.params.clone()).unwrap_or_else(|_| {
                crate::types::InsertRowMutationParams {
                    sub_unit_params: crate::types::SubUnitParams {
                        unit_id: "".to_string(),
                        sub_unit_id: "".to_string(),
                    },
                    range: crate::types::Range {
                        start_row: 0,
                        start_column: 0,
                        end_row: 0,
                        end_column: 0,
                    },
                    row_info: None,
                }
            });
        if !same_sheet(&params.sub_unit_params, &m2_params.sub_unit_params) {
            return TransformResultInternal {
                m1_prime: Some(m1.clone()),
                m2_prime: Some(m2.clone()),
                error: None,
            };
        }
        let insert_start = m2_params.range.start_row;
        let insert_count = m2_params.range.end_row - m2_params.range.start_row + 1;
        crate::transform::sheets_transform_utils::shift_range_rows_for_insert(
            &mut params.range,
            insert_start,
            insert_count,
        );
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
        let mut params: SheetMutationRangeParams = parse_range_params(&m1.id, &m1.params);
        let m2_params: crate::types::InsertColMutationParams =
            serde_json::from_value(m2.params.clone()).unwrap_or_else(|_| {
                crate::types::InsertColMutationParams {
                    sub_unit_params: crate::types::SubUnitParams {
                        unit_id: "".to_string(),
                        sub_unit_id: "".to_string(),
                    },
                    range: crate::types::Range {
                        start_row: 0,
                        start_column: 0,
                        end_row: 0,
                        end_column: 0,
                    },
                    col_info: None,
                }
            });
        if !same_sheet(&params.sub_unit_params, &m2_params.sub_unit_params) {
            return TransformResultInternal {
                m1_prime: Some(m1.clone()),
                m2_prime: Some(m2.clone()),
                error: None,
            };
        }
        let insert_start = m2_params.range.start_column;
        let insert_count = m2_params.range.end_column - m2_params.range.start_column + 1;
        crate::transform::sheets_transform_utils::shift_range_cols_for_insert(
            &mut params.range,
            insert_start,
            insert_count,
        );
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
        let mut params: SheetMutationRangeParams = parse_range_params(&m1.id, &m1.params);
        let m2_params: crate::types::RemoveRowsMutationParams =
            serde_json::from_value(m2.params.clone()).unwrap_or_else(|_| {
                crate::types::RemoveRowsMutationParams {
                    sub_unit_params: crate::types::SubUnitParams {
                        unit_id: "".to_string(),
                        sub_unit_id: "".to_string(),
                    },
                    range: crate::types::Range {
                        start_row: 0,
                        start_column: 0,
                        end_row: 0,
                        end_column: 0,
                    },
                }
            });
        if !same_sheet(&params.sub_unit_params, &m2_params.sub_unit_params) {
            return TransformResultInternal {
                m1_prime: Some(m1.clone()),
                m2_prime: Some(m2.clone()),
                error: None,
            };
        }
        if !crate::transform::sheets_transform_utils::shift_range_rows_for_remove(
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
        let mut params: SheetMutationRangeParams = parse_range_params(&m1.id, &m1.params);
        let m2_params: crate::types::RemoveColMutationParams =
            serde_json::from_value(m2.params.clone()).unwrap_or_else(|_| {
                crate::types::RemoveColMutationParams {
                    sub_unit_params: crate::types::SubUnitParams {
                        unit_id: "".to_string(),
                        sub_unit_id: "".to_string(),
                    },
                    range: crate::types::Range {
                        start_row: 0,
                        start_column: 0,
                        end_row: 0,
                        end_column: 0,
                    },
                }
            });
        if !same_sheet(&params.sub_unit_params, &m2_params.sub_unit_params) {
            return TransformResultInternal {
                m1_prime: Some(m1.clone()),
                m2_prime: Some(m2.clone()),
                error: None,
            };
        }
        if !crate::transform::sheets_transform_utils::shift_range_cols_for_remove(
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

    fn transform_with_set_worksheet_range_theme_style(
        &self,
        m1: &MutationInfoInternal,
        m2: &MutationInfoInternal,
    ) -> TransformResultInternal {
        let params = parse_range_params(&m1.id, &m1.params);
        let m2_params = parse_range_params(&m2.id, &m2.params);
        if !same_sheet(&params.sub_unit_params, &m2_params.sub_unit_params) {
            return TransformResultInternal {
                m1_prime: Some(m1.clone()),
                m2_prime: Some(m2.clone()),
                error: None,
            };
        }
        if ranges_intersect(&params.range, &m2_params.range) {
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
