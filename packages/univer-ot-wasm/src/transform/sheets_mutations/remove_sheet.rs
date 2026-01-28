use crate::transform::mutation_transform::MutationTransform;
use crate::transform::sheets_mutations::common::same_sheet;
use crate::types::{MutationInfoInternal, SubUnitParams, TransformResultInternal};

#[derive(Default)]
pub struct RemoveSheetTransform;

fn parse_sub_unit_params(params: &serde_json::Value) -> SubUnitParams {
    serde_json::from_value(params.clone()).unwrap_or_else(|_| SubUnitParams {
        unit_id: "".to_string(),
        sub_unit_id: "".to_string(),
    })
}

macro_rules! impl_remove_sheet_conflict {
    ($method:ident) => {
        fn $method(
            &self,
            m1: &MutationInfoInternal,
            m2: &MutationInfoInternal,
        ) -> TransformResultInternal {
            let m1_params = parse_sub_unit_params(&m1.params);
            let m2_params = parse_sub_unit_params(&m2.params);

            if same_sheet(&m1_params, &m2_params) {
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
    };
}

impl MutationTransform for RemoveSheetTransform {
    fn mutation_id() -> &'static str {
        "sheet.mutation.remove-sheet"
    }

    impl_remove_sheet_conflict!(transform_with_set_range_values);
    impl_remove_sheet_conflict!(transform_with_insert_row);
    impl_remove_sheet_conflict!(transform_with_insert_col);
    impl_remove_sheet_conflict!(transform_with_remove_rows);
    impl_remove_sheet_conflict!(transform_with_remove_col);
    impl_remove_sheet_conflict!(transform_with_add_conditional_rule);
    impl_remove_sheet_conflict!(transform_with_add_range_protection);
    impl_remove_sheet_conflict!(transform_with_add_range_theme);
    impl_remove_sheet_conflict!(transform_with_add_worksheet_merge);
    impl_remove_sheet_conflict!(transform_with_add_worksheet_protection);
    impl_remove_sheet_conflict!(transform_with_copy_worksheet_end);
    impl_remove_sheet_conflict!(transform_with_delete_conditional_rule);
    impl_remove_sheet_conflict!(transform_with_delete_range_protection);
    impl_remove_sheet_conflict!(transform_with_delete_worksheet_protection);
    impl_remove_sheet_conflict!(transform_with_remove_worksheet_range_theme_style);
    impl_remove_sheet_conflict!(transform_with_empty);
    impl_remove_sheet_conflict!(transform_with_insert_sheet);
    impl_remove_sheet_conflict!(transform_with_move_columns);
    impl_remove_sheet_conflict!(transform_with_move_conditional_rule);
    impl_remove_sheet_conflict!(transform_with_move_rows);
    impl_remove_sheet_conflict!(transform_with_move_range);
    impl_remove_sheet_conflict!(transform_with_register_worksheet_range_theme_style);
    impl_remove_sheet_conflict!(transform_with_remove_range_theme);
    impl_remove_sheet_conflict!(transform_with_remove_sheet);
    impl_remove_sheet_conflict!(transform_with_remove_worksheet_merge);
    impl_remove_sheet_conflict!(transform_with_reorder_range);
    impl_remove_sheet_conflict!(transform_with_set_col_data);
    impl_remove_sheet_conflict!(transform_with_set_col_hidden);
    impl_remove_sheet_conflict!(transform_with_set_col_visible);
    impl_remove_sheet_conflict!(transform_with_set_frozen);
    impl_remove_sheet_conflict!(transform_with_set_gridlines_color);
    impl_remove_sheet_conflict!(transform_with_set_numfmt);
    impl_remove_sheet_conflict!(transform_with_remove_numfmt);
    impl_remove_sheet_conflict!(transform_with_set_conditional_rule);
    impl_remove_sheet_conflict!(transform_with_set_range_protection);
    impl_remove_sheet_conflict!(transform_with_set_range_theme);
    impl_remove_sheet_conflict!(transform_with_set_row_data);
    impl_remove_sheet_conflict!(transform_with_set_row_hidden);
    impl_remove_sheet_conflict!(transform_with_set_row_visible);
    impl_remove_sheet_conflict!(transform_with_set_tab_color);
    impl_remove_sheet_conflict!(transform_with_set_workbook_name);
    impl_remove_sheet_conflict!(transform_with_set_worksheet_col_width);
    impl_remove_sheet_conflict!(transform_with_set_worksheet_column_count);
    impl_remove_sheet_conflict!(transform_with_set_worksheet_default_style);
    impl_remove_sheet_conflict!(transform_with_set_worksheet_hidden);
    impl_remove_sheet_conflict!(transform_with_set_worksheet_name);
    impl_remove_sheet_conflict!(transform_with_set_worksheet_order);
    impl_remove_sheet_conflict!(transform_with_set_worksheet_permission_points);
    impl_remove_sheet_conflict!(transform_with_set_worksheet_protection);
    impl_remove_sheet_conflict!(transform_with_set_worksheet_range_theme_style);
    impl_remove_sheet_conflict!(transform_with_set_worksheet_right_to_left);
    impl_remove_sheet_conflict!(transform_with_set_worksheet_row_auto_height);
    impl_remove_sheet_conflict!(transform_with_set_worksheet_row_count);
    impl_remove_sheet_conflict!(transform_with_set_worksheet_row_height);
    impl_remove_sheet_conflict!(transform_with_set_worksheet_row_is_auto_height);
    impl_remove_sheet_conflict!(transform_with_conditional_formatting_formula_mark_dirty);
    impl_remove_sheet_conflict!(transform_with_toggle_gridlines);
    impl_remove_sheet_conflict!(transform_with_unregister_worksheet_range_theme_style);

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
