// Core sheet mutations
pub mod insert_row;
pub mod insert_col;
pub mod remove_rows;
pub mod remove_col;
pub mod set_range_values;
pub mod move_range;
pub mod move_rows;
pub mod move_columns;

// Additional sheet features
pub mod merge;
pub mod protection;
pub mod theme;
pub mod numfmt;
pub mod frozen;
pub mod row_col_data;
pub mod worksheet;
pub mod workbook;

use crate::registry::TransformRegistry;

/// Register all sheets transforms
pub fn register_transforms(registry: &mut TransformRegistry) {
    insert_row::register_transforms(registry);
    insert_col::register_transforms(registry);
    remove_rows::register_transforms(registry);
    remove_col::register_transforms(registry);
    set_range_values::register_transforms(registry);
    move_range::register_transforms(registry);
    move_rows::register_transforms(registry);
    move_columns::register_transforms(registry);
    merge::register_transforms(registry);
    protection::register_transforms(registry);
    theme::register_transforms(registry);
    numfmt::register_transforms(registry);
    frozen::register_transforms(registry);
    row_col_data::register_transforms(registry);
    worksheet::register_transforms(registry);
    workbook::register_transforms(registry);
}
