//! Shared mutation constants for all transform modules
//!
//! This module provides the complete list of all mutations for cross-module registration.

use crate::registry::MutationId;

// Sheets mutations
use crate::mutations::sheets::{
    InsertRowMutation, InsertColMutation, RemoveRowMutation, RemoveColMutation,
    SetRangeValuesMutation, MoveRangeMutation, MoveRowsMutation, MoveColsMutation,
    AddWorksheetMergeMutation, SetRangeProtectionMutation, SetRangeThemeMutation,
    SetFrozenMutation, SetRowDataMutation,
    InsertSheetMutation, SetWorkbookNameMutation, RemoveWorksheetMergeMutation,
    AddRangeProtectionMutation, DeleteRangeProtectionMutation, AddRangeThemeMutation,
    RemoveRangeThemeMutation, SetColDataMutation, RemoveSheetMutation,
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

// Numfmt mutations (from sheets module)
use crate::mutations::sheets_numfmt::{
    SetNumfmtMutation, RemoveNumfmtMutation,
};

// Data validation mutations
use crate::mutations::data_validation::{
    AddDataValidationMutation, RemoveDataValidationMutation, UpdateDataValidationMutation,
};

// Conditional formatting mutations
use crate::mutations::sheets_conditional_formatting::{
    AddConditionalRuleMutation, DeleteConditionalRuleMutation, SetConditionalRuleMutation,
    MoveConditionalRuleMutation,
};

// Filter mutations
use crate::mutations::sheets_filter::{
    SetSheetsFilterRangeMutation, SetSheetsFilterCriteriaMutation, RemoveSheetsFilterMutation,
    ReCalcSheetsFilterMutation,
};

// HyperLink mutations (sheets version, not docs version)
use crate::mutations::sheets_hyper_link::{
    AddHyperLinkMutation, RemoveHyperLinkMutation, UpdateHyperLinkMutation,
    UpdateHyperLinkRefMutation, UpdateRichHyperLinkMutation,
};

// Note mutations
use crate::mutations::sheets_note::{
    UpdateNoteMutation, RemoveNoteMutation, ToggleNotePopupMutation, UpdateNotePositionMutation,
};

// Table mutations
use crate::mutations::sheets_table::{
    AddSheetTableMutation, SetSheetTableMutation, SetSheetTableFilterMutation, DeleteSheetTableMutation,
};

// Pivot table mutations
use crate::mutations::sheets_pivot_table::{
    AddPivotTableMutation, RemovePivotTableMutation, SetPivotTableSourceRangeMutation,
    SetPivotTableTargetCellMutation, SetPivotTableFieldsConfigMutation,
    SetPivotTableCalculatedDataMutation,
};

// Thread comment mutations
use crate::mutations::thread_comment::{
    AddCommentMutation, UpdateCommentMutation, UpdateCommentRefMutation,
    ResolveCommentMutation, DeleteCommentMutation,
};

/// All sheets core mutations (53 total)
pub const ALL_SHEETS_CORE_MUTATIONS: &[MutationId] = &[
    // Original 17 core mutations (including remove.numfmt)
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
    // Additional 36 mutations
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
    SetRowVisibleMutation::ID,
    SetRowHiddenMutation::ID,
    SetColVisibleMutation::ID,
    SetColHiddenMutation::ID,
    SetWorksheetRowHeightMutation::ID,
    SetWorksheetRowIsAutoHeightMutation::ID,
    SetWorksheetRowAutoHeightMutation::ID,
    SetWorksheetColWidthMutation::ID,
    SetWorksheetRowCountMutation::ID,
    SetWorksheetColumnCountMutation::ID,
    ToggleGridlinesMutation::ID,
    SetGridlinesColorMutation::ID,
    SetTabColorMutation::ID,
    ReorderRangeMutation::ID,
    EmptyMutation::ID,
    AddWorksheetProtectionMutation::ID,
    SetWorksheetProtectionMutation::ID,
    DeleteWorksheetProtectionMutation::ID,
    SetWorksheetPermissionPointsMutation::ID,
    SetWorksheetDefaultStyleMutation::ID,
    SetWorksheetRightToLeftMutation::ID,
    SetWorksheetRangeThemeStyleMutation::ID,
    DeleteWorksheetRangeThemeStyleMutation::ID,
    RegisterWorksheetRangeThemeStyleMutation::ID,
    UnregisterWorksheetRangeThemeStyleMutation::ID,
];

/// Data validation mutations (3 total)
pub const DATA_VALIDATION_MUTATIONS: &[MutationId] = &[
    AddDataValidationMutation::ID,
    RemoveDataValidationMutation::ID,
    UpdateDataValidationMutation::ID,
];

/// Conditional formatting mutations (4 total)
pub const CONDITIONAL_FORMATTING_MUTATIONS: &[MutationId] = &[
    AddConditionalRuleMutation::ID,
    DeleteConditionalRuleMutation::ID,
    SetConditionalRuleMutation::ID,
    MoveConditionalRuleMutation::ID,
];

/// Filter mutations (4 total)
pub const FILTER_MUTATIONS: &[MutationId] = &[
    SetSheetsFilterRangeMutation::ID,
    SetSheetsFilterCriteriaMutation::ID,
    RemoveSheetsFilterMutation::ID,
    ReCalcSheetsFilterMutation::ID,
];

/// HyperLink mutations (5 total)
pub const HYPER_LINK_MUTATIONS: &[MutationId] = &[
    AddHyperLinkMutation::ID,
    RemoveHyperLinkMutation::ID,
    UpdateHyperLinkMutation::ID,
    UpdateHyperLinkRefMutation::ID,
    UpdateRichHyperLinkMutation::ID,
];

/// Note mutations (4 total)
pub const NOTE_MUTATIONS: &[MutationId] = &[
    UpdateNoteMutation::ID,
    RemoveNoteMutation::ID,
    ToggleNotePopupMutation::ID,
    UpdateNotePositionMutation::ID,
];

/// Table mutations (4 total)
pub const TABLE_MUTATIONS: &[MutationId] = &[
    AddSheetTableMutation::ID,
    SetSheetTableMutation::ID,
    SetSheetTableFilterMutation::ID,
    DeleteSheetTableMutation::ID,
];

/// Pivot Table mutations (6 total)
pub const PIVOT_TABLE_MUTATIONS: &[MutationId] = &[
    AddPivotTableMutation::ID,
    RemovePivotTableMutation::ID,
    SetPivotTableSourceRangeMutation::ID,
    SetPivotTableTargetCellMutation::ID,
    SetPivotTableFieldsConfigMutation::ID,
    SetPivotTableCalculatedDataMutation::ID,
];

/// Thread comment mutations (5 total)
pub const THREAD_COMMENT_MUTATIONS: &[MutationId] = &[
    AddCommentMutation::ID,
    UpdateCommentMutation::ID,
    UpdateCommentRefMutation::ID,
    ResolveCommentMutation::ID,
    DeleteCommentMutation::ID,
];
