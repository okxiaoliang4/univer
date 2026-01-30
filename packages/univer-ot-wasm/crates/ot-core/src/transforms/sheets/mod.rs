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

// New modules for additional mutations
pub mod visibility;
pub mod dimensions;
pub mod grid;
pub mod tab;
pub mod reorder;
pub mod empty;
pub mod worksheet_protection;
pub mod worksheet_style;
pub mod range_theme_style;
pub mod additional;

use crate::registry::{MutationId, TransformRegistry};
use crate::transforms::constants::{
    DATA_VALIDATION_MUTATIONS,
    CONDITIONAL_FORMATTING_MUTATIONS,
    FILTER_MUTATIONS,
    HYPER_LINK_MUTATIONS,
    NOTE_MUTATIONS,
    TABLE_MUTATIONS,
    PIVOT_TABLE_MUTATIONS,
    THREAD_COMMENT_MUTATIONS,
};

// Import all mutation structs used in constants
use crate::mutations::sheets::{
    InsertRowMutation, InsertColMutation, RemoveRowMutation, RemoveColMutation,
    SetRangeValuesMutation, MoveRangeMutation, MoveRowsMutation, MoveColsMutation,
    AddWorksheetMergeMutation, SetRangeProtectionMutation, SetRangeThemeMutation,
    SetFrozenMutation, SetRowDataMutation, InsertSheetMutation, SetWorkbookNameMutation,
    RemoveWorksheetMergeMutation, AddRangeProtectionMutation, DeleteRangeProtectionMutation,
    AddRangeThemeMutation, RemoveRangeThemeMutation, SetColDataMutation, RemoveSheetMutation,
    SetWorksheetNameMutation, SetWorksheetOrderMutation, SetWorksheetHideMutation,
    CopyWorksheetEndMutation, SetRowVisibleMutation, SetRowHiddenMutation,
    SetColVisibleMutation, SetColHiddenMutation, SetWorksheetRowHeightMutation,
    SetWorksheetRowIsAutoHeightMutation, SetWorksheetRowAutoHeightMutation,
    SetWorksheetColWidthMutation, SetWorksheetRowCountMutation, SetWorksheetColumnCountMutation,
    ToggleGridlinesMutation, SetGridlinesColorMutation, SetTabColorMutation,
    ReorderRangeMutation, EmptyMutation, AddWorksheetProtectionMutation,
    SetWorksheetProtectionMutation, DeleteWorksheetProtectionMutation,
    SetWorksheetPermissionPointsMutation, SetWorksheetDefaultStyleMutation,
    SetWorksheetRightToLeftMutation, SetWorksheetRangeThemeStyleMutation,
    DeleteWorksheetRangeThemeStyleMutation, RegisterWorksheetRangeThemeStyleMutation,
    UnregisterWorksheetRangeThemeStyleMutation,
};
use crate::mutations::sheets_numfmt::{SetNumfmtMutation, RemoveNumfmtMutation};

/// All existing core mutations for cross-module registration
const EXISTING_CORE_MUTATIONS: &[MutationId] = &[
    InsertRowMutation::ID,
    InsertColMutation::ID,
    RemoveRowMutation::ID,
    RemoveColMutation::ID,
    SetRangeValuesMutation::ID,
    MoveRangeMutation::ID,
    MoveRowsMutation::ID,
    MoveColsMutation::ID,
    AddWorksheetMergeMutation::ID,
    SetRangeProtectionMutation::ID,
    SetRangeThemeMutation::ID,
    SetNumfmtMutation::ID,
    RemoveNumfmtMutation::ID,
    SetFrozenMutation::ID,
    SetRowDataMutation::ID,
    InsertSheetMutation::ID,
    SetWorkbookNameMutation::ID,
];

/// Register all sheets transforms
pub fn register_transforms(registry: &mut TransformRegistry) {
    // Register existing core modules
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

    // Register new modules
    visibility::register_transforms(registry);
    dimensions::register_transforms(registry);
    grid::register_transforms(registry);
    tab::register_transforms(registry);
    reorder::register_transforms(registry);
    empty::register_transforms(registry);
    worksheet_protection::register_transforms(registry);
    worksheet_style::register_transforms(registry);
    range_theme_style::register_transforms(registry);
    additional::register_transforms(registry);

    // Register cross-module transforms for new modules with existing core mutations
    visibility::register_cross_module_transforms(registry, EXISTING_CORE_MUTATIONS);
    dimensions::register_cross_module_transforms(registry, EXISTING_CORE_MUTATIONS);
    grid::register_cross_module_transforms(registry, EXISTING_CORE_MUTATIONS);
    tab::register_cross_module_transforms(registry, EXISTING_CORE_MUTATIONS);
    reorder::register_cross_module_transforms(registry, EXISTING_CORE_MUTATIONS);
    empty::register_cross_module_transforms(registry, EXISTING_CORE_MUTATIONS);
    worksheet_protection::register_cross_module_transforms(registry, EXISTING_CORE_MUTATIONS);
    worksheet_style::register_cross_module_transforms(registry, EXISTING_CORE_MUTATIONS);
    range_theme_style::register_cross_module_transforms(registry, EXISTING_CORE_MUTATIONS);
    additional::register_cross_module_transforms(registry, EXISTING_CORE_MUTATIONS);

    // Register cross-module transforms between new modules
    register_new_modules_cross_transforms(registry);

    // Register cross-module transforms with other feature modules (data validation, conditional formatting, etc.)
    register_cross_module_with_other_features(registry);
}

/// Register cross-module transforms between all new module mutations
fn register_new_modules_cross_transforms(registry: &mut TransformRegistry) {
    // Visibility mutations
    let visibility_mutations: &[MutationId] = &[
        SetRowVisibleMutation::ID,
        SetRowHiddenMutation::ID,
        SetColVisibleMutation::ID,
        SetColHiddenMutation::ID,
    ];

    // Dimension mutations
    let dimension_mutations: &[MutationId] = &[
        SetWorksheetRowHeightMutation::ID,
        SetWorksheetRowIsAutoHeightMutation::ID,
        SetWorksheetRowAutoHeightMutation::ID,
        SetWorksheetColWidthMutation::ID,
        SetWorksheetRowCountMutation::ID,
        SetWorksheetColumnCountMutation::ID,
    ];

    // Grid mutations
    let grid_mutations: &[MutationId] = &[
        ToggleGridlinesMutation::ID,
        SetGridlinesColorMutation::ID,
    ];

    // Tab mutations
    let tab_mutations: &[MutationId] = &[
        SetTabColorMutation::ID,
    ];

    // Reorder mutations
    let reorder_mutations: &[MutationId] = &[
        ReorderRangeMutation::ID,
    ];

    // Empty mutations
    let empty_mutations: &[MutationId] = &[
        EmptyMutation::ID,
    ];

    // Worksheet protection mutations
    let ws_protection_mutations: &[MutationId] = &[
        AddWorksheetProtectionMutation::ID,
        SetWorksheetProtectionMutation::ID,
        DeleteWorksheetProtectionMutation::ID,
        SetWorksheetPermissionPointsMutation::ID,
    ];

    // Worksheet style mutations
    let ws_style_mutations: &[MutationId] = &[
        SetWorksheetDefaultStyleMutation::ID,
        SetWorksheetRightToLeftMutation::ID,
    ];

    // Range theme style mutations
    let range_theme_style_mutations: &[MutationId] = &[
        SetWorksheetRangeThemeStyleMutation::ID,
        DeleteWorksheetRangeThemeStyleMutation::ID,
        RegisterWorksheetRangeThemeStyleMutation::ID,
        UnregisterWorksheetRangeThemeStyleMutation::ID,
    ];

    // Additional mutations
    let additional_mutations: &[MutationId] = &[
        RemoveWorksheetMergeMutation::ID,
        AddRangeProtectionMutation::ID,
        DeleteRangeProtectionMutation::ID,
        AddRangeThemeMutation::ID,
        RemoveRangeThemeMutation::ID,
        SetColDataMutation::ID,
        RemoveSheetMutation::ID,
        SetWorksheetNameMutation::ID,
        SetWorksheetOrderMutation::ID,
        SetWorksheetHideMutation::ID,
        CopyWorksheetEndMutation::ID,
    ];

    // All new module mutation groups
    let all_groups: Vec<&[MutationId]> = vec![
        visibility_mutations,
        dimension_mutations,
        grid_mutations,
        tab_mutations,
        reorder_mutations,
        empty_mutations,
        ws_protection_mutations,
        ws_style_mutations,
        range_theme_style_mutations,
        additional_mutations,
    ];

    // Register identity transforms between all groups
    for i in 0..all_groups.len() {
        for j in (i + 1)..all_groups.len() {
            for &m1 in all_groups[i] {
                for &m2 in all_groups[j] {
                    registry.register_identity(m1, m2);
                }
            }
        }
    }
}

/// Register cross-module transforms between all sheets-core mutations and other feature modules
fn register_cross_module_with_other_features(registry: &mut TransformRegistry) {
    // All new module mutations that need cross-registration with other features
    let new_module_mutations: &[MutationId] = &[
        // Visibility mutations
        SetRowVisibleMutation::ID,
        SetRowHiddenMutation::ID,
        SetColVisibleMutation::ID,
        SetColHiddenMutation::ID,
        // Dimension mutations
        SetWorksheetRowHeightMutation::ID,
        SetWorksheetRowIsAutoHeightMutation::ID,
        SetWorksheetRowAutoHeightMutation::ID,
        SetWorksheetColWidthMutation::ID,
        SetWorksheetRowCountMutation::ID,
        SetWorksheetColumnCountMutation::ID,
        // Grid mutations
        ToggleGridlinesMutation::ID,
        SetGridlinesColorMutation::ID,
        // Tab mutations
        SetTabColorMutation::ID,
        // Reorder mutations
        ReorderRangeMutation::ID,
        // Empty mutations
        EmptyMutation::ID,
        // Worksheet protection mutations
        AddWorksheetProtectionMutation::ID,
        SetWorksheetProtectionMutation::ID,
        DeleteWorksheetProtectionMutation::ID,
        SetWorksheetPermissionPointsMutation::ID,
        // Worksheet style mutations
        SetWorksheetDefaultStyleMutation::ID,
        SetWorksheetRightToLeftMutation::ID,
        // Range theme style mutations
        SetWorksheetRangeThemeStyleMutation::ID,
        DeleteWorksheetRangeThemeStyleMutation::ID,
        RegisterWorksheetRangeThemeStyleMutation::ID,
        UnregisterWorksheetRangeThemeStyleMutation::ID,
        // Additional mutations
        RemoveWorksheetMergeMutation::ID,
        AddRangeProtectionMutation::ID,
        DeleteRangeProtectionMutation::ID,
        AddRangeThemeMutation::ID,
        RemoveRangeThemeMutation::ID,
        SetColDataMutation::ID,
        RemoveSheetMutation::ID,
        SetWorksheetNameMutation::ID,
        SetWorksheetOrderMutation::ID,
        SetWorksheetHideMutation::ID,
        CopyWorksheetEndMutation::ID,
    ];

    // All other feature module mutations
    let all_other_modules: &[&[MutationId]] = &[
        DATA_VALIDATION_MUTATIONS,
        CONDITIONAL_FORMATTING_MUTATIONS,
        FILTER_MUTATIONS,
        HYPER_LINK_MUTATIONS,
        NOTE_MUTATIONS,
        TABLE_MUTATIONS,
        PIVOT_TABLE_MUTATIONS,
        THREAD_COMMENT_MUTATIONS,
    ];

    // Register identity transforms between new sheets-core mutations and all other modules
    for &sheets_mutation in new_module_mutations {
        for &other_module in all_other_modules {
            for &other_mutation in other_module {
                registry.register_identity(sheets_mutation, other_mutation);
            }
        }
    }
}
