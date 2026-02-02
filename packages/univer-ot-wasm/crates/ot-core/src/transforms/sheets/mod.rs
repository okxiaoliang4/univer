// All sheet mutations - one file per mutation (46 modules)
// Files are named to match mutations/sheets/*.rs (without _mutation suffix)
//
// NOTE: The registry automatically falls back to identity transform when no
// transform is registered. Therefore, we only need to register transforms that
// actually modify mutations (shift, LWW, etc.), not identity transforms.

// A
pub mod add_range_protection;
pub mod add_range_theme;
pub mod add_worksheet_merge;
pub mod add_worksheet_protection;
pub mod add_worksheet_range_theme;

// C
pub mod copy_worksheet_end;

// D
pub mod delete_range_protection;
pub mod delete_worksheet_protection;
pub mod delete_worksheet_range_theme;

// E
pub mod empty;

// I
pub mod insert_row_col;
pub mod insert_sheet;

// M
pub mod mark_dirty_auto_height;
pub mod move_range;
pub mod move_rows_cols;

// R
pub mod register_range_theme;
pub mod remove_range_theme;
pub mod remove_row_col;
pub mod remove_sheet;
pub mod remove_worksheet_merge;
pub mod reorder_range;

// S
pub mod set_col_data;
pub mod set_col_visible;
pub mod set_frozen;
pub mod set_gridlines_color;
pub mod set_range_protection;
pub mod set_range_theme;
pub mod set_range_values;
pub mod set_row_data;
pub mod set_row_visible;
pub mod set_tab_color;
pub mod set_workbook_name;
pub mod set_worksheet_col_width;
pub mod set_worksheet_column_count;
pub mod set_worksheet_default_style;
pub mod set_worksheet_hide;
pub mod set_worksheet_name;
pub mod set_worksheet_order;
pub mod set_worksheet_permission_points;
pub mod set_worksheet_protection;
pub mod set_worksheet_right_to_left;
pub mod set_worksheet_row_count;
pub mod set_worksheet_row_height;

// T
pub mod toggle_gridlines;

// U
pub mod unregister_range_theme_style;

use crate::registry::TransformRegistry;

/// Register all sheets transforms
///
/// This function registers transforms for all sheet mutations.
/// Each mutation has its own dedicated file for symmetric transforms.
///
/// NOTE: We only register transforms that modify mutations (shift, LWW, etc.).
/// Identity transforms are NOT registered - the registry automatically falls back
/// to identity when no transform is found.
pub fn register_transforms(registry: &mut TransformRegistry) {
    // Register all individual mutation transforms (alphabetical order)
    // Each file registers symmetric transforms and specific bidirectional transforms
    add_range_protection::register_transforms(registry);
    add_range_theme::register_transforms(registry);
    add_worksheet_merge::register_transforms(registry);
    add_worksheet_protection::register_transforms(registry);
    add_worksheet_range_theme::register_transforms(registry);

    copy_worksheet_end::register_transforms(registry);

    delete_range_protection::register_transforms(registry);
    delete_worksheet_protection::register_transforms(registry);
    delete_worksheet_range_theme::register_transforms(registry);

    empty::register_transforms(registry);

    insert_row_col::register_transforms(registry);
    insert_sheet::register_transforms(registry);

    mark_dirty_auto_height::register_transforms(registry);
    move_range::register_transforms(registry);
    move_rows_cols::register_transforms(registry);

    register_range_theme::register_transforms(registry);
    remove_range_theme::register_transforms(registry);
    remove_row_col::register_transforms(registry);
    remove_sheet::register_transforms(registry);
    remove_worksheet_merge::register_transforms(registry);
    reorder_range::register_transforms(registry);

    set_col_data::register_transforms(registry);
    set_col_visible::register_transforms(registry);
    set_frozen::register_transforms(registry);
    set_gridlines_color::register_transforms(registry);
    set_range_protection::register_transforms(registry);
    set_range_theme::register_transforms(registry);
    set_range_values::register_transforms(registry);
    set_row_data::register_transforms(registry);
    set_row_visible::register_transforms(registry);
    set_tab_color::register_transforms(registry);
    set_workbook_name::register_transforms(registry);
    set_worksheet_col_width::register_transforms(registry);
    set_worksheet_column_count::register_transforms(registry);
    set_worksheet_default_style::register_transforms(registry);
    set_worksheet_hide::register_transforms(registry);
    set_worksheet_name::register_transforms(registry);
    set_worksheet_order::register_transforms(registry);
    set_worksheet_permission_points::register_transforms(registry);
    set_worksheet_protection::register_transforms(registry);
    set_worksheet_right_to_left::register_transforms(registry);
    set_worksheet_row_count::register_transforms(registry);
    set_worksheet_row_height::register_transforms(registry);

    toggle_gridlines::register_transforms(registry);

    unregister_range_theme_style::register_transforms(registry);

    // NOTE: No need for register_identity calls!
    // The registry automatically falls back to identity transform when no transform is found.
}
