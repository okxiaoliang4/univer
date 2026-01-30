//! Shared mutation constants for all transform modules
//!
//! This module provides the complete list of all mutations for cross-module registration.

use crate::registry::MutationId;

/// All sheets core mutations (53 total)
pub const ALL_SHEETS_CORE_MUTATIONS: &[MutationId] = &[
    // Original 17 core mutations (including remove.numfmt)
    "sheet.mutation.insert-row",
    "sheet.mutation.insert-col",
    "sheet.mutation.remove-rows",
    "sheet.mutation.remove-col",
    "sheet.mutation.set-range-values",
    "sheet.mutation.move-range",
    "sheet.mutation.move-rows",
    "sheet.mutation.move-columns",
    "sheet.mutation.add-worksheet-merge",
    "sheet.mutation.set-range-protection",
    "sheet.mutation.set-range-theme",
    "sheet.mutation.set.numfmt",
    "sheet.mutation.remove.numfmt",
    "sheet.mutation.set-frozen",
    "sheet.mutation.set-row-data",
    "sheet.mutation.insert-sheet",
    "sheet.mutation.set-workbook-name",
    // Additional 36 mutations
    "sheet.mutation.remove-worksheet-merge",
    "sheet.mutation.add-range-protection",
    "sheet.mutation.delete-range-protection",
    "sheet.mutation.add-range-theme",
    "sheet.mutation.remove-range-theme",
    "sheet.mutation.set-col-data",
    "sheet.mutation.remove-sheet",
    "sheet.mutation.set-worksheet-name",
    "sheet.mutation.set-worksheet-order",
    "sheet.mutation.set-worksheet-hidden",
    "sheet.mutation.copy-worksheet-end",
    "sheet.mutation.set-row-visible",
    "sheet.mutation.set-row-hidden",
    "sheet.mutation.set-col-visible",
    "sheet.mutation.set-col-hidden",
    "sheet.mutation.set-worksheet-row-height",
    "sheet.mutation.set-worksheet-row-is-auto-height",
    "sheet.mutation.set-worksheet-row-auto-height",
    "sheet.mutation.set-worksheet-col-width",
    "sheet.mutation.set-worksheet-row-count",
    "sheet.mutation.set-worksheet-column-count",
    "sheet.mutation.toggle-gridlines",
    "sheet.mutation.set-gridlines-color",
    "sheet.mutation.set-tab-color",
    "sheet.mutation.reorder-range",
    "sheet.mutation.empty",
    "sheet.mutation.add-worksheet-protection",
    "sheet.mutation.set-worksheet-protection",
    "sheet.mutation.delete-worksheet-protection",
    "sheet.mutation.set-worksheet-permission-points",
    "sheet.mutation.set-worksheet-default-style",
    "sheet.mutation.set-worksheet-right-to-left",
    "sheet.mutation.set-worksheet-range-theme-style",
    "sheet.mutation.remove-worksheet-range-theme-style",
    "sheet.mutation.register-worksheet-range-theme-style",
    "sheet.mutation.unregister-worksheet-range-theme-style",
];

/// Data validation mutations (3 total)
pub const DATA_VALIDATION_MUTATIONS: &[MutationId] = &[
    "data-validation.mutation.addRule",
    "data-validation.mutation.removeRule",
    "data-validation.mutation.updateRule",
];

/// Conditional formatting mutations (4 total)
pub const CONDITIONAL_FORMATTING_MUTATIONS: &[MutationId] = &[
    "sheet.mutation.add-conditional-rule",
    "sheet.mutation.delete-conditional-rule",
    "sheet.mutation.set-conditional-rule",
    "sheet.mutation.move-conditional-rule",
];

/// Filter mutations (4 total)
pub const FILTER_MUTATIONS: &[MutationId] = &[
    "sheet.mutation.set-filter-range",
    "sheet.mutation.set-filter-criteria",
    "sheet.mutation.remove-filter",
    "sheet.mutation.re-calc-filter",
];

/// HyperLink mutations (5 total)
pub const HYPER_LINK_MUTATIONS: &[MutationId] = &[
    "sheets.mutation.add-hyper-link",
    "sheets.mutation.remove-hyper-link",
    "sheets.mutation.update-hyper-link",
    "sheets.mutation.update-hyper-link-ref",
    "sheets.mutation.update-rich-hyper-link",
];

/// Note mutations (4 total)
pub const NOTE_MUTATIONS: &[MutationId] = &[
    "sheet.mutation.update-note",
    "sheet.mutation.remove-note",
    "sheet.mutation.toggle-note-popup",
    "sheet.mutation.update-note-position",
];

/// Table mutations (4 total)
pub const TABLE_MUTATIONS: &[MutationId] = &[
    "sheet.mutation.add-table",
    "sheet.mutation.set-sheet-table",
    "sheet.mutation.set-table-filter",
    "sheet.mutation.delete-table",
];

/// Pivot Table mutations (6 total)
pub const PIVOT_TABLE_MUTATIONS: &[MutationId] = &[
    "sheet.mutation.add-pivot-table",
    "sheet.mutation.remove-pivot-table",
    "sheet.mutation.set-pivot-table-source-range",
    "sheet.mutation.set-pivot-table-target-cell",
    "sheet.mutation.set-pivot-table-fields-config",
    "sheet.mutation.set-pivot-table-calculated-data",
];

/// Thread comment mutations (5 total)
pub const THREAD_COMMENT_MUTATIONS: &[MutationId] = &[
    "thread-comment.mutation.add-comment",
    "thread-comment.mutation.update-comment",
    "thread-comment.mutation.update-comment-ref",
    "thread-comment.mutation.resolve-comment",
    "thread-comment.mutation.delete-comment",
];
