use crate::types::{MutationInfoInternal, TransformResultInternal};

#[macro_export]
macro_rules! sheet_mutation_transform_list {
    ($macro:ident) => {
        $macro!(
            transform_with_set_range_values,
            "sheet.mutation.set-range-values"
        );
        $macro!(transform_with_insert_row, "sheet.mutation.insert-row");
        $macro!(transform_with_insert_col, "sheet.mutation.insert-col");
        $macro!(transform_with_remove_rows, "sheet.mutation.remove-rows");
        $macro!(transform_with_remove_col, "sheet.mutation.remove-col");
        $macro!(
            transform_with_add_conditional_rule,
            "sheet.mutation.add-conditional-rule"
        );
        $macro!(
            transform_with_add_range_protection,
            "sheet.mutation.add-range-protection"
        );
        $macro!(
            transform_with_add_range_theme,
            "sheet.mutation.add-range-theme"
        );
        $macro!(
            transform_with_add_worksheet_merge,
            "sheet.mutation.add-worksheet-merge"
        );
        $macro!(
            transform_with_add_worksheet_protection,
            "sheet.mutation.add-worksheet-protection"
        );
        $macro!(
            transform_with_conditional_formatting_formula_mark_dirty,
            "sheet.mutation.conditional-formatting-formula-mark-dirty"
        );
        $macro!(
            transform_with_copy_worksheet_end,
            "sheet.mutation.copy-worksheet-end"
        );
        $macro!(
            transform_with_delete_conditional_rule,
            "sheet.mutation.delete-conditional-rule"
        );
        $macro!(
            transform_with_delete_range_protection,
            "sheet.mutation.delete-range-protection"
        );
        $macro!(
            transform_with_delete_worksheet_protection,
            "sheet.mutation.delete-worksheet-protection"
        );
        $macro!(
            transform_with_remove_worksheet_range_theme_style,
            "sheet.mutation.remove-worksheet-range-theme-style"
        );
        $macro!(transform_with_empty, "sheet.mutation.empty");
        $macro!(transform_with_insert_sheet, "sheet.mutation.insert-sheet");
        $macro!(transform_with_move_columns, "sheet.mutation.move-columns");
        $macro!(
            transform_with_move_conditional_rule,
            "sheet.mutation.move-conditional-rule"
        );
        $macro!(transform_with_move_rows, "sheet.mutation.move-rows");
        $macro!(transform_with_move_range, "sheet.mutation.move-range");
        $macro!(
            transform_with_register_worksheet_range_theme_style,
            "sheet.mutation.register-worksheet-range-theme-style"
        );
        $macro!(
            transform_with_remove_range_theme,
            "sheet.mutation.remove-range-theme"
        );
        $macro!(transform_with_remove_sheet, "sheet.mutation.remove-sheet");
        $macro!(
            transform_with_remove_worksheet_merge,
            "sheet.mutation.remove-worksheet-merge"
        );
        $macro!(transform_with_reorder_range, "sheet.mutation.reorder-range");
        $macro!(transform_with_set_col_data, "sheet.mutation.set-col-data");
        $macro!(
            transform_with_set_col_hidden,
            "sheet.mutation.set-col-hidden"
        );
        $macro!(
            transform_with_set_col_visible,
            "sheet.mutation.set-col-visible"
        );
        $macro!(transform_with_set_frozen, "sheet.mutation.set-frozen");
        $macro!(
            transform_with_set_gridlines_color,
            "sheet.mutation.set-gridlines-color"
        );
        $macro!(transform_with_set_numfmt, "sheet.mutation.set.numfmt");
        $macro!(transform_with_remove_numfmt, "sheet.mutation.remove.numfmt");
        $macro!(
            transform_with_set_range_protection,
            "sheet.mutation.set-range-protection"
        );
        $macro!(
            transform_with_set_conditional_rule,
            "sheet.mutation.set-conditional-rule"
        );
        $macro!(
            transform_with_set_range_theme,
            "sheet.mutation.set-range-theme"
        );
        $macro!(transform_with_set_row_data, "sheet.mutation.set-row-data");
        $macro!(
            transform_with_set_row_hidden,
            "sheet.mutation.set-row-hidden"
        );
        $macro!(
            transform_with_set_row_visible,
            "sheet.mutation.set-row-visible"
        );
        $macro!(transform_with_set_tab_color, "sheet.mutation.set-tab-color");
        $macro!(
            transform_with_set_workbook_name,
            "sheet.mutation.set-workbook-name"
        );
        $macro!(
            transform_with_set_worksheet_col_width,
            "sheet.mutation.set-worksheet-col-width"
        );
        $macro!(
            transform_with_set_worksheet_column_count,
            "sheet.mutation.set-worksheet-column-count"
        );
        $macro!(
            transform_with_set_worksheet_default_style,
            "sheet.mutation.set-worksheet-default-style"
        );
        $macro!(
            transform_with_set_worksheet_hidden,
            "sheet.mutation.set-worksheet-hidden"
        );
        $macro!(
            transform_with_set_worksheet_name,
            "sheet.mutation.set-worksheet-name"
        );
        $macro!(
            transform_with_set_worksheet_order,
            "sheet.mutation.set-worksheet-order"
        );
        $macro!(
            transform_with_set_worksheet_permission_points,
            "sheet.mutation.set-worksheet-permission-points"
        );
        $macro!(
            transform_with_set_worksheet_protection,
            "sheet.mutation.set-worksheet-protection"
        );
        $macro!(
            transform_with_set_worksheet_range_theme_style,
            "sheet.mutation.set-worksheet-range-theme-style"
        );
        $macro!(
            transform_with_set_worksheet_right_to_left,
            "sheet.mutation.set-worksheet-right-to-left"
        );
        $macro!(
            transform_with_set_worksheet_row_auto_height,
            "sheet.mutation.set-worksheet-row-auto-height"
        );
        $macro!(
            transform_with_set_worksheet_row_count,
            "sheet.mutation.set-worksheet-row-count"
        );
        $macro!(
            transform_with_set_worksheet_row_height,
            "sheet.mutation.set-worksheet-row-height"
        );
        $macro!(
            transform_with_set_worksheet_row_is_auto_height,
            "sheet.mutation.set-worksheet-row-is-auto-height"
        );
        $macro!(
            transform_with_toggle_gridlines,
            "sheet.mutation.toggle-gridlines"
        );
        $macro!(
            transform_with_unregister_worksheet_range_theme_style,
            "sheet.mutation.unregister-worksheet-range-theme-style"
        );
        $macro!(
            transform_with_add_data_validation,
            "data-validation.mutation.addRule"
        );
        $macro!(
            transform_with_remove_data_validation,
            "data-validation.mutation.removeRule"
        );
        $macro!(
            transform_with_update_data_validation,
            "data-validation.mutation.updateRule"
        );
    };
}

macro_rules! identity_transform_method {
    ($method:ident, $id:expr) => {
        fn $method(
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
    };
}

/// Core trait for mutation transforms
/// All transforms operate on core types (MutationInfoInternal/TransformResultInternal)
/// which use serde_json::Value internally for zero-copy efficiency
pub trait MutationTransform: Default {
    /// Get the mutation ID string for this mutation type
    fn mutation_id() -> &'static str;

    /// Compose two mutations of the same type
    fn compose(
        &self,
        m1: &MutationInfoInternal,
        m2: &MutationInfoInternal,
    ) -> Vec<MutationInfoInternal>;

    sheet_mutation_transform_list!(identity_transform_method);
}
