# Mutation Conflict Analysis Document Index

This index tracks all mutation conflict analysis documents and their completion status.

## Status Legend

- ✅ Document complete with full coverage matrix
- 📦 Batch document covering multiple mutations
- ⏳ Document planned
- ❌ Document not needed (trivial mutation)

## Core Sheets Mutations (55 total)

### Structural Mutations (Insert/Remove/Move)

| Mutation | Document | Status |
|----------|----------|--------|
| InsertRowMutation | [insert_row.md](insert_row.md) | ✅ |
| InsertColMutation | [insert_col.md](insert_col.md) | ✅ |
| RemoveRowsMutation | [remove_rows.md](remove_rows.md) | ✅ |
| RemoveColMutation | [remove_col.md](remove_col.md) | ✅ |
| MoveRowsMutation | [move_rows.md](move_rows.md) | ✅ |
| MoveColumnsMutation | [move_columns.md](move_columns.md) | ✅ |
| MoveRangeMutation | [move_range.md](move_range.md) | ✅ |

### Data Mutations

| Mutation | Document | Status |
|----------|----------|--------|
| SetRangeValuesMutation | [set_range_values.md](set_range_values.md) | ✅ |
| SetRowDataMutation | [set_row_data.md](set_row_data.md) | ✅ |
| SetColDataMutation | [set_col_data.md](set_col_data.md) | ✅ |
| ReorderRangeMutation | [reorder_range.md](reorder_range.md) | ✅ |

### Merge Mutations

| Mutation | Document | Status |
|----------|----------|--------|
| AddWorksheetMergeMutation | [add_worksheet_merge.md](add_worksheet_merge.md) | ✅ |
| RemoveWorksheetMergeMutation | [remove_worksheet_merge.md](remove_worksheet_merge.md) | ✅ |

### Dimension Mutations

| Mutation | Document | Status |
|----------|----------|--------|
| SetWorksheetRowHeightMutation | [set_worksheet_row_height.md](set_worksheet_row_height.md) | ✅ |
| SetWorksheetColWidthMutation | [set_worksheet_col_width.md](set_worksheet_col_width.md) | ✅ |
| SetWorksheetRowIsAutoHeightMutation | [auto_height_mutations.md](auto_height_mutations.md) | 📦 |
| SetWorksheetRowAutoHeightMutation | [auto_height_mutations.md](auto_height_mutations.md) | 📦 |
| MarkDirtyRowAutoHeightMutation | [auto_height_mutations.md](auto_height_mutations.md) | 📦 |
| CancelMarkDirtyRowAutoHeightMutation | [auto_height_mutations.md](auto_height_mutations.md) | 📦 |

### Visibility Mutations

| Mutation | Document | Status |
|----------|----------|--------|
| SetRowVisibleMutation | [visibility_mutations.md](visibility_mutations.md) | 📦 |
| SetRowHiddenMutation | [visibility_mutations.md](visibility_mutations.md) | 📦 |
| SetColVisibleMutation | [visibility_mutations.md](visibility_mutations.md) | 📦 |
| SetColHiddenMutation | [visibility_mutations.md](visibility_mutations.md) | 📦 |

### Protection Mutations

| Mutation | Document | Status |
|----------|----------|--------|
| AddRangeProtectionMutation | [protection_mutations.md](protection_mutations.md) | 📦 |
| DeleteRangeProtectionMutation | [protection_mutations.md](protection_mutations.md) | 📦 |
| SetRangeProtectionMutation | [protection_mutations.md](protection_mutations.md) | 📦 |
| AddWorksheetProtectionMutation | [protection_mutations.md](protection_mutations.md) | 📦 |
| DeleteWorksheetProtectionMutation | [protection_mutations.md](protection_mutations.md) | 📦 |
| SetWorksheetProtectionMutation | [protection_mutations.md](protection_mutations.md) | 📦 |
| SetWorksheetPermissionPointsMutation | [protection_mutations.md](protection_mutations.md) | 📦 |

### Theme/Style Mutations

| Mutation | Document | Status |
|----------|----------|--------|
| AddRangeThemeMutation | [theme_mutations.md](theme_mutations.md) | 📦 |
| RemoveRangeThemeMutation | [theme_mutations.md](theme_mutations.md) | 📦 |
| SetRangeThemeMutation | [theme_mutations.md](theme_mutations.md) | 📦 |
| RegisterWorksheetRangeThemeStyleMutation | [theme_mutations.md](theme_mutations.md) | 📦 |
| UnregisterWorksheetRangeThemeStyleMutation | [theme_mutations.md](theme_mutations.md) | 📦 |
| SetWorksheetRangeThemeStyleMutation | [theme_mutations.md](theme_mutations.md) | 📦 |
| DeleteWorksheetRangeThemeStyleMutation | [theme_mutations.md](theme_mutations.md) | 📦 |

### Worksheet Configuration

| Mutation | Document | Status |
|----------|----------|--------|
| SetFrozenMutation | [set_frozen.md](set_frozen.md) | ✅ |
| SetGridlinesColorMutation | [worksheet_properties.md](worksheet_properties.md) | 📦 |
| ToggleGridlinesMutation | [worksheet_properties.md](worksheet_properties.md) | 📦 |
| SetTabColorMutation | [worksheet_properties.md](worksheet_properties.md) | 📦 |
| SetWorksheetHideMutation | [worksheet_properties.md](worksheet_properties.md) | 📦 |
| SetWorksheetNameMutation | [worksheet_properties.md](worksheet_properties.md) | 📦 |
| SetWorksheetOrderMutation | [worksheet_properties.md](worksheet_properties.md) | 📦 |
| SetWorksheetRightToLeftMutation | [worksheet_properties.md](worksheet_properties.md) | 📦 |
| SetWorksheetDefaultStyleMutation | [worksheet_properties.md](worksheet_properties.md) | 📦 |

### Sheet Management

| Mutation | Document | Status |
|----------|----------|--------|
| InsertSheetMutation | [sheet_management_mutations.md](sheet_management_mutations.md) | 📦 |
| RemoveSheetMutation | [sheet_management_mutations.md](sheet_management_mutations.md) | 📦 |
| CopyWorksheetEndMutation | [sheet_management_mutations.md](sheet_management_mutations.md) | 📦 |

### Worksheet Size

| Mutation | Document | Status |
|----------|----------|--------|
| SetWorksheetColumnCountMutation | [worksheet_properties.md](worksheet_properties.md) | 📦 |
| SetWorksheetRowCountMutation | [worksheet_properties.md](worksheet_properties.md) | 📦 |

### Workbook

| Mutation | Document | Status |
|----------|----------|--------|
| SetWorkbookNameMutation | ❌ | ❌ |

### Number Format

| Mutation | Document | Status |
|----------|----------|--------|
| SetNumfmtMutation | [numfmt_mutations.md](numfmt_mutations.md) | 📦 |
| RemoveNumfmtMutation | [numfmt_mutations.md](numfmt_mutations.md) | 📦 |

### Other

| Mutation | Document | Status |
|----------|----------|--------|
| EmptyMutation | ❌ | ❌ |

## Feature Plugin Mutations

### Data Validation (3 total)

| Mutation | Document | Status |
|----------|----------|--------|
| AddDataValidationMutation | [add_data_validation.md](add_data_validation.md) | ✅ |
| RemoveDataValidationMutation | [data_validation_mutations.md](data_validation_mutations.md) | 📦 |
| UpdateDataValidationMutation | [data_validation_mutations.md](data_validation_mutations.md) | 📦 |

### Conditional Formatting (5 total)

| Mutation | Document | Status |
|----------|----------|--------|
| AddConditionalRuleMutation | [add_conditional_rule.md](add_conditional_rule.md) | ✅ |
| DeleteConditionalRuleMutation | [conditional_formatting_mutations.md](conditional_formatting_mutations.md) | 📦 |
| SetConditionalRuleMutation | [conditional_formatting_mutations.md](conditional_formatting_mutations.md) | 📦 |
| MoveConditionalRuleMutation | [conditional_formatting_mutations.md](conditional_formatting_mutations.md) | 📦 |
| ConditionalFormattingFormulaMarkDirty | [conditional_formatting_mutations.md](conditional_formatting_mutations.md) | 📦 |

### Filter (4 total)

| Mutation | Document | Status |
|----------|----------|--------|
| SetSheetsFilterRangeMutation | [set_sheets_filter_range.md](set_sheets_filter_range.md) | ✅ |
| SetSheetsFilterCriteriaMutation | [filter_mutations.md](filter_mutations.md) | 📦 |
| RemoveSheetsFilterMutation | [filter_mutations.md](filter_mutations.md) | 📦 |
| ReCalcSheetsFilterMutation | [filter_mutations.md](filter_mutations.md) | 📦 |

### Hyperlink (5 total)

| Mutation | Document | Status |
|----------|----------|--------|
| AddHyperLinkMutation | [hyperlink_mutations.md](hyperlink_mutations.md) | 📦 |
| RemoveHyperLinkMutation | [hyperlink_mutations.md](hyperlink_mutations.md) | 📦 |
| UpdateHyperLinkMutation | [hyperlink_mutations.md](hyperlink_mutations.md) | 📦 |
| UpdateHyperLinkRefMutation | [hyperlink_mutations.md](hyperlink_mutations.md) | 📦 |
| UpdateRichHyperLinkMutation | [hyperlink_mutations.md](hyperlink_mutations.md) | 📦 |

### Note (4 total)

| Mutation | Document | Status |
|----------|----------|--------|
| UpdateNoteMutation | [update_note.md](update_note.md) | ✅ |
| RemoveNoteMutation | [note_mutations.md](note_mutations.md) | 📦 |
| ToggleNotePopupMutation | [note_mutations.md](note_mutations.md) | 📦 |
| UpdateNotePositionMutation | [note_mutations.md](note_mutations.md) | 📦 |

### Pivot Table (6 total)

| Mutation | Document | Status |
|----------|----------|--------|
| AddPivotTableMutation | [pivot_table_mutations.md](pivot_table_mutations.md) | 📦 |
| RemovePivotTableMutation | [pivot_table_mutations.md](pivot_table_mutations.md) | 📦 |
| SetPivotTableSourceRangeMutation | [pivot_table_mutations.md](pivot_table_mutations.md) | 📦 |
| SetPivotTableTargetCellMutation | [pivot_table_mutations.md](pivot_table_mutations.md) | 📦 |
| SetPivotTableFieldsConfigMutation | [pivot_table_mutations.md](pivot_table_mutations.md) | 📦 |
| SetPivotTableCalculatedDataMutation | [pivot_table_mutations.md](pivot_table_mutations.md) | 📦 |

### Table (4 total)

| Mutation | Document | Status |
|----------|----------|--------|
| AddSheetTableMutation | [table_mutations.md](table_mutations.md) | 📦 |
| DeleteSheetTableMutation | [table_mutations.md](table_mutations.md) | 📦 |
| SetSheetTableMutation | [table_mutations.md](table_mutations.md) | 📦 |
| SetSheetTableFilterMutation | [table_mutations.md](table_mutations.md) | 📦 |

### Thread Comment (5 total)

| Mutation | Document | Status |
|----------|----------|--------|
| AddCommentMutation | [add_comment.md](add_comment.md) | ✅ |
| UpdateCommentMutation | [thread_comment_mutations.md](thread_comment_mutations.md) | 📦 |
| UpdateCommentRefMutation | [thread_comment_mutations.md](thread_comment_mutations.md) | 📦 |
| ResolveCommentMutation | [thread_comment_mutations.md](thread_comment_mutations.md) | 📦 |
| DeleteCommentMutation | [thread_comment_mutations.md](thread_comment_mutations.md) | 📦 |

## Drawing, Docs, Formula, and Other Modules

The following modules are not included in the current documentation as they have different OT requirements or are out of scope:

- **sheets_drawing** (1 mutation): SetDrawingApplyMutation - Complex drawing operations
- **docs** (2 mutations): RichTextEditingMutation, DocsRenameMutation - Document OT (not sheets)
- **docs_hyper_link** (3 mutations): Document hyperlinks (different from sheets)
- **engine_formula** (29 mutations): Formula calculation system (special handling)

## Summary Statistics

| Category | Total | Individual Docs | Batch Docs | Not Needed |
|----------|-------|-----------------|------------|------------|
| Core Sheets | 55 | 15 | 38 | 2 |
| Data Validation | 3 | 1 | 2 | 0 |
| Conditional Formatting | 5 | 1 | 4 | 0 |
| Filter | 4 | 1 | 3 | 0 |
| Hyperlink | 5 | 0 | 5 | 0 |
| Note | 4 | 1 | 3 | 0 |
| Pivot Table | 6 | 0 | 6 | 0 |
| Table | 4 | 0 | 4 | 0 |
| Thread Comment | 5 | 1 | 4 | 0 |
| **Total** | **91** | **20** | **69** | **2** |

## Document Types

### Individual Documents (20)
Detailed documents with complete transform coverage matrices, implementation code, and test cases.

### Batch Documents (15)
Documents covering multiple related mutations that follow similar patterns:
- worksheet_properties.md (10 mutations)
- visibility_mutations.md (4 mutations)
- protection_mutations.md (7 mutations)
- theme_mutations.md (7 mutations)
- sheet_management_mutations.md (3 mutations)
- auto_height_mutations.md (4 mutations)
- hyperlink_mutations.md (5 mutations)
- note_mutations.md (3 mutations)
- pivot_table_mutations.md (6 mutations)
- table_mutations.md (4 mutations)
- conditional_formatting_mutations.md (4 mutations)
- filter_mutations.md (3 mutations)
- data_validation_mutations.md (2 mutations)
- thread_comment_mutations.md (4 mutations)
- numfmt_mutations.md (2 mutations)

## Last Updated

**Date**: 2026-01-31
**Total Documents**: 35 (20 individual + 15 batch)
**Mutations Documented**: 89 out of 91 sheets/feature mutations
**Coverage**: 98% complete
