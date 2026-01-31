# InsertColMutation Conflict Analysis

**Mutation ID**: `sheet.mutation.insert-col`
**File**: `mutations/sheets/insert_row_col_mutation.rs`
**Transform**: `transforms/sheets/insert_col.rs`
**Tests**: `tests/insert_col_tests.rs`

## Mutation Overview

**Purpose**: Inserts one or more columns at a specified position in a worksheet, shifting existing columns right.

**Affects**:
- [ ] Rows
- [x] Columns (primary - shifts column positions)
- [x] Cells/Ranges (indirect - cells shift with columns)
- [ ] Worksheet properties
- [ ] Workbook properties
- [ ] Feature-specific

**Parameters**:
```rust
pub struct InsertColMutationParams {
    pub unit_id: String,           // Workbook ID
    pub sub_unit_id: String,       // Worksheet ID
    pub range: IRange,             // Range defining insertion (start_column, end_column)
    pub col_info: Option<IObjectArrayPrimitiveType<IColumnData>>,  // Column metadata
    pub cell_value: Option<IObjectMatrixPrimitiveType>,  // Optional cell data for new columns
}
```

**Key Derived Values**:
- `insert_position = range.start_column`
- `insert_count = range.end_column - range.start_column + 1`

## Conflict Dimensions Analysis

### Dimension 1: Unit ID (Workbook)
- **Conflicts when**: Same `unit_id`
- **No conflict when**: Different `unit_id` → Identity transform

### Dimension 2: Sub-Unit ID (Worksheet)
- **Conflicts when**: Same `unit_id` AND same `sub_unit_id`
- **No conflict when**: Different worksheets → Identity transform

### Dimension 3: Position (Row/Col/Range)
**Positions affected by this mutation**:
- Rows: Not affected
- Columns: All columns >= `insert_position` are shifted right by `insert_count`
- Cells: All cells in columns >= `insert_position` shift right

**Conflict scenarios**:
1. **Same insert position**: Both inserts at same column → Order matters
2. **m1 before m2**: m1's insert shifts m2's position right
3. **m1 after m2**: No shift needed for m2

### Dimension 4: Feature Plugin ID
**Feature-specific IDs**: N/A

## Transform Coverage Matrix

**Status Legend**:
- ✅ Implemented and tested
- 🚧 Implementation in progress
- ⏳ Planned
- ❌ Not needed (identity)

### Core Sheets Mutations (53 total)

#### Structural Mutations (Insert/Remove)

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| InsertRowMutation | ❌ | Identity | Column insert doesn't affect row positions |
| InsertColMutation (self) | ✅ | Shifting | m2's column shifts if >= m1's position |
| RemoveRowsMutation | ❌ | Identity | Column insert doesn't affect row positions |
| RemoveColMutation | ⏳ | Shifting + Conflict | Complex: may remove the inserted columns |
| MoveRowsMutation | ❌ | Identity | Column insert doesn't affect row positions |
| MoveColumnsMutation | ⏳ | Shifting | Source/target ranges need column adjustment |
| MoveRangeMutation | ⏳ | Shifting | from_range/to_range need column adjustment |

#### Data Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| SetRangeValuesMutation | ✅ | Shifting | Cell column keys >= insert position shifted |
| SetRowDataMutation | ❌ | Identity | Row data organized by row, not column |
| SetColDataMutation | ⏳ | Shifting | Column data keys need shifting |
| ReorderRangeMutation | ⏳ | Shifting | Range positions need adjustment |

#### Merge Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| AddWorksheetMergeMutation | ✅ | Shifting | Merge ranges shift with columns |
| RemoveWorksheetMergeMutation | ⏳ | Shifting | Merge ranges shift with columns |

#### Dimension Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| SetWorksheetRowHeightMutation | ❌ | Identity | Row dimensions not affected |
| SetWorksheetColWidthMutation | ⏳ | Shifting | Column indices need adjustment |
| SetWorksheetRowIsAutoHeightMutation | ❌ | Identity | Row dimensions not affected |
| SetWorksheetRowAutoHeightMutation | ❌ | Identity | Row dimensions not affected |
| MarkDirtyRowAutoHeightMutation | ❌ | Identity | Row dimensions not affected |
| CancelMarkDirtyRowAutoHeightMutation | ❌ | Identity | Row dimensions not affected |

#### Visibility Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| SetRowVisibleMutation | ❌ | Identity | Row visibility not affected |
| SetRowHiddenMutation | ❌ | Identity | Row visibility not affected |
| SetColVisibleMutation | ⏳ | Shifting | Column ranges need adjustment |
| SetColHiddenMutation | ⏳ | Shifting | Column ranges need adjustment |

#### Protection Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| AddRangeProtectionMutation | ⏳ | Shifting | Protection ranges need column shift |
| DeleteRangeProtectionMutation | ⏳ | Shifting | Protection ranges need column shift |
| SetRangeProtectionMutation | ❌ | Identity | Registered as identity |
| AddWorksheetProtectionMutation | ❌ | Identity | Worksheet-level, no position |
| DeleteWorksheetProtectionMutation | ❌ | Identity | Worksheet-level, no position |
| SetWorksheetProtectionMutation | ❌ | Identity | Worksheet-level, no position |
| SetWorksheetPermissionPointsMutation | ❌ | Identity | Worksheet-level, no position |

#### Theme/Style Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| AddRangeThemeMutation | ⏳ | Shifting | Theme ranges need column shift |
| RemoveRangeThemeMutation | ⏳ | Shifting | Theme ranges need column shift |
| SetRangeThemeMutation | ❌ | Identity | Registered as identity |
| RegisterWorksheetRangeThemeStyleMutation | ❌ | Identity | Unit-level, no position |
| UnregisterWorksheetRangeThemeStyleMutation | ❌ | Identity | Unit-level, no position |
| SetWorksheetRangeThemeStyleMutation | ⏳ | Shifting | Range needs column shift |
| DeleteWorksheetRangeThemeStyleMutation | ⏳ | Shifting | Range needs column shift |

#### Worksheet Configuration

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| SetFrozenMutation | ⏳ | Shifting | start_column, x_split may need adjustment |
| SetGridlinesColorMutation | ❌ | Identity | Worksheet property, no position |
| ToggleGridlinesMutation | ❌ | Identity | Worksheet property, no position |
| SetTabColorMutation | ❌ | Identity | Worksheet property, no position |
| SetWorksheetHideMutation | ❌ | Identity | Worksheet property, no position |
| SetWorksheetNameMutation | ❌ | Identity | Worksheet property, no position |
| SetWorksheetOrderMutation | ❌ | Identity | Worksheet property, no position |
| SetWorksheetRightToLeftMutation | ❌ | Identity | Worksheet property, no position |
| SetWorksheetDefaultStyleMutation | ❌ | Identity | Worksheet property, no position |

#### Sheet Management

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| InsertSheetMutation | ❌ | Identity | Different worksheet |
| RemoveSheetMutation | ❌ | Identity | Different worksheet |
| CopyWorksheetEndMutation | ❌ | Identity | Different worksheet |

#### Worksheet Size

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| SetWorksheetColumnCountMutation | ⏳ | Adjustment | May need to increase column count |
| SetWorksheetRowCountMutation | ❌ | Identity | Row count not affected |

#### Workbook

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| SetWorkbookNameMutation | ❌ | Identity | Workbook level, no position |

#### Number Format

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| SetNumfmtMutation | ⏳ | Shifting | ref_map ranges need column shift |
| RemoveNumfmtMutation | ⏳ | Shifting | ranges need column shift |

#### Other

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| EmptyMutation | ❌ | Identity | No-op mutation |

### Feature Plugin Mutations

#### Conditional Formatting

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| AddConditionalRuleMutation | ⏳ | Shifting | Rule ranges need column shift |
| DeleteConditionalRuleMutation | ❌ | Identity | Deletion by ID, no position |
| SetConditionalRuleMutation | ⏳ | Shifting | Rule ranges need column shift |
| MoveConditionalRuleMutation | ❌ | Identity | Reorders rules, no position |

#### Data Validation

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| AddDataValidationMutation | ⏳ | Shifting | Rule ranges need column shift |
| RemoveDataValidationMutation | ❌ | Identity | Deletion by ID, no position |
| UpdateDataValidationMutation | ⏳ | Shifting | Rule ranges need column shift |

#### Filter

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| SetSheetsFilterRangeMutation | ⏳ | Shifting | Filter range needs column shift |
| RemoveSheetsFilterMutation | ❌ | Identity | Removes filter, no position |
| SetSheetsFilterCriteriaMutation | ⏳ | Shifting | col field needs shift |
| ReCalcSheetsFilterMutation | ❌ | Identity | Recalculation trigger |

#### Hyperlink

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| AddHyperLinkMutation | ⏳ | Shifting | Link column position needs shift |
| RemoveHyperLinkMutation | ❌ | Identity | Removal by ID, no position |
| UpdateHyperLinkMutation | ❌ | Identity | Update by ID, no position |
| UpdateHyperLinkRefMutation | ⏳ | Shifting | column field needs shift |
| UpdateRichHyperLinkMutation | ⏳ | Shifting | col field needs shift |

#### Note

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| UpdateNoteMutation | ⏳ | Shifting | col field needs shift |
| RemoveNoteMutation | ⏳ | Shifting | col field needs shift |
| ToggleNotePopupMutation | ⏳ | Shifting | col field needs shift |
| UpdateNotePositionMutation | ⏳ | Shifting | col field needs shift |

#### Pivot Table

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| AddPivotTableMutation | ⏳ | Shifting | Source/target ranges need shift |
| RemovePivotTableMutation | ❌ | Identity | Removal by ID, no position |
| SetPivotTableSourceRangeMutation | ⏳ | Shifting | Source range needs shift |
| SetPivotTableTargetCellMutation | ⏳ | Shifting | Target cell column needs shift |
| SetPivotTableFieldsConfigMutation | ❌ | Identity | Config only, no position |
| SetPivotTableCalculatedDataMutation | ❌ | Identity | Data only, no position |

#### Table

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| AddSheetTableMutation | ⏳ | Shifting | Table range needs column shift |
| DeleteSheetTableMutation | ❌ | Identity | Deletion by ID, no position |
| SetSheetTableMutation | ⏳ | Shifting | Table config may have range |
| SetSheetTableFilterMutation | ⏳ | Shifting | column field needs shift |

#### Thread Comment

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| AddCommentMutation | ⏳ | Shifting | Comment position needs shift |
| UpdateCommentMutation | ❌ | Identity | Update by ID, no position |
| UpdateCommentRefMutation | ⏳ | Shifting | ref position needs shift |
| ResolveCommentMutation | ❌ | Identity | Status update, no position |
| DeleteCommentMutation | ❌ | Identity | Deletion by ID, no position |

## Detailed Conflict Resolution

### InsertColMutation vs InsertColMutation (Self-Transform)

**Conflict Analysis**:
- **Dimension 3 (Position)**: Order of inserts matters - earlier insert shifts later one

**Resolution Strategy**: Shifting

**Implementation**:
```rust
fn create_self_transform() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return identity(m1, m2);
        }

        let m1_params: InsertColMutationParams = parse(m1)?;
        let mut m2_params: InsertColMutationParams = parse(m2)?;

        let m1_start = m1_params.range.start_column;
        let m1_count = m1_params.range.end_column - m1_start + 1;

        // If m2's insert position >= m1's insert position, shift m2 right
        if m2_params.range.start_column >= m1_start {
            m2_params.range.start_column += m1_count;
            m2_params.range.end_column += m1_count;
        }

        TransformResultRef {
            m1_prime: MutationOutcome::Unchanged(m1),
            m2_prime: MutationOutcome::Modified(serialize(m2_params)),
            error: None,
        }
    })
}
```

**Test Cases**:
- [x] Different worksheets (identity)
- [x] Same position - m2 shifts by m1's count
- [x] m1 before m2 - m2 shifts
- [ ] m1 after m2 - no shift

**Status**: ✅ Implemented

---

### InsertColMutation vs SetRangeValuesMutation

**Conflict Analysis**:
- **Dimension 3 (Position)**: SetRangeValues cell column keys need shifting

**Resolution Strategy**: Shifting (cell column keys)

**Implementation**: Uses `shift_col_keys_for_insert()` utility

**Test Cases**:
- [x] Cells at insert position - shift right
- [x] Cells right of insert - shift right
- [ ] Cells left of insert - no shift

**Status**: ✅ Implemented

---

### InsertColMutation vs AddWorksheetMergeMutation

**Conflict Analysis**:
- **Dimension 3 (Position)**: Merge ranges need column shifting

**Resolution Strategy**: Shifting (range columns)

**Implementation**: Uses `shift_range_cols_for_insert()` utility

**Status**: ✅ Implemented (in merge.rs)

## Implementation Checklist

- [x] Self-transform registered in `transforms/sheets/insert_col.rs`
- [x] Transform with SetRangeValuesMutation implemented
- [x] Transform with AddWorksheetMergeMutation implemented (in merge.rs)
- [ ] All structural mutation transforms implemented
- [ ] All feature plugin transforms implemented
- [x] Tests written for implemented scenarios
- [x] Zero-copy optimizations applied
- [x] This document updated with implementation status

## Notes

- InsertCol is symmetric to InsertRow but affects columns instead of rows
- Use `shift_col_keys_for_insert()` utility for HashMap-based cell data
- Use `shift_range_cols_for_insert()` utility for IRange-based data

## Last Updated

**Date**: 2026-01-31
**By**: Claude Code
**Changes**: Initial document creation with comprehensive coverage matrix
