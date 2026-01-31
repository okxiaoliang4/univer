# InsertRowMutation Conflict Analysis

**Mutation ID**: `sheet.mutation.insert-row`
**File**: `mutations/sheets/insert_row_col_mutation.rs`
**Transform**: `transforms/sheets/insert_row.rs`
**Tests**: `tests/insert_row_tests.rs`

## Mutation Overview

**Purpose**: Inserts one or more rows at a specified position in a worksheet, shifting existing rows down.

**Affects**:
- [x] Rows (primary - shifts row positions)
- [ ] Columns
- [x] Cells/Ranges (indirect - cells shift with rows)
- [ ] Worksheet properties
- [ ] Workbook properties
- [ ] Feature-specific

**Parameters**:
```rust
pub struct InsertRowMutationParams {
    pub unit_id: String,           // Workbook ID
    pub sub_unit_id: String,       // Worksheet ID
    pub range: IRange,             // Range defining insertion (start_row, end_row)
    pub row_info: Option<IObjectArrayPrimitiveType<IRowData>>,  // Row metadata
    pub cell_value: Option<IObjectMatrixPrimitiveType>,  // Optional cell data for new rows
}
```

**Key Derived Values**:
- `insert_position = range.start_row`
- `insert_count = range.end_row - range.start_row + 1`

## Conflict Dimensions Analysis

### Dimension 1: Unit ID (Workbook)
- **Conflicts when**: Same `unit_id`
- **No conflict when**: Different `unit_id` → Identity transform

### Dimension 2: Sub-Unit ID (Worksheet)
- **Conflicts when**: Same `unit_id` AND same `sub_unit_id`
- **No conflict when**: Different worksheets → Identity transform

### Dimension 3: Position (Row/Col/Range)
**Positions affected by this mutation**:
- Rows: All rows >= `insert_position` are shifted down by `insert_count`
- Columns: Not affected
- Cells: All cells in rows >= `insert_position` shift down

**Conflict scenarios**:
1. **Same insert position**: Both inserts at same row → Order matters, one shifts the other
2. **m1 before m2**: m1's insert shifts m2's position down
3. **m1 after m2**: No shift needed for m2
4. **Overlapping ranges**: Not applicable (inserts don't overlap)

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
| InsertRowMutation (self) | ✅ | Shifting | m2's position shifts if >= m1's position |
| InsertColMutation | ❌ | Identity | Row insert doesn't affect column positions |
| RemoveRowsMutation | ⏳ | Shifting + Conflict | Complex: may remove the inserted rows |
| RemoveColMutation | ❌ | Identity | Row insert doesn't affect column positions |
| MoveRowsMutation | ⏳ | Shifting | Source/target ranges need row adjustment |
| MoveColumnsMutation | ❌ | Identity | Row insert doesn't affect column positions |
| MoveRangeMutation | ⏳ | Shifting | from_range/to_range need row adjustment |

#### Data Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| SetRangeValuesMutation | ✅ | Shifting | Cell row keys >= insert position shifted |
| SetRowDataMutation | ⏳ | Shifting | Row data keys need shifting |
| SetColDataMutation | ❌ | Identity | Column data not affected by row insert |
| ReorderRangeMutation | ⏳ | Shifting | Range positions need adjustment |

#### Merge Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| AddWorksheetMergeMutation | ✅ | Shifting | Merge ranges shift with rows |
| RemoveWorksheetMergeMutation | ⏳ | Shifting | Merge ranges shift with rows |

#### Dimension Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| SetWorksheetRowHeightMutation | ⏳ | Shifting | Row indices need adjustment |
| SetWorksheetColWidthMutation | ❌ | Identity | Column dimensions not affected |
| SetWorksheetRowIsAutoHeightMutation | ⏳ | Shifting | Row indices need adjustment |
| SetWorksheetRowAutoHeightMutation | ⏳ | Shifting | Row indices need adjustment |
| MarkDirtyRowAutoHeightMutation | ⏳ | Shifting | Row indices need adjustment |
| CancelMarkDirtyRowAutoHeightMutation | ⏳ | Shifting | Row indices need adjustment |

#### Visibility Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| SetRowVisibleMutation | ⏳ | Shifting | Row ranges need adjustment |
| SetRowHiddenMutation | ⏳ | Shifting | Row ranges need adjustment |
| SetColVisibleMutation | ❌ | Identity | Column visibility not affected |
| SetColHiddenMutation | ❌ | Identity | Column visibility not affected |

#### Protection Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| AddRangeProtectionMutation | ⏳ | Shifting | Protection ranges need row shift |
| DeleteRangeProtectionMutation | ⏳ | Shifting | Protection ranges need row shift |
| SetRangeProtectionMutation | ❌ | Identity | Registered as identity |
| AddWorksheetProtectionMutation | ❌ | Identity | Worksheet-level, no position |
| DeleteWorksheetProtectionMutation | ❌ | Identity | Worksheet-level, no position |
| SetWorksheetProtectionMutation | ❌ | Identity | Worksheet-level, no position |
| SetWorksheetPermissionPointsMutation | ❌ | Identity | Worksheet-level, no position |

#### Theme/Style Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| AddRangeThemeMutation | ⏳ | Shifting | Theme ranges need row shift |
| RemoveRangeThemeMutation | ⏳ | Shifting | Theme ranges need row shift |
| SetRangeThemeMutation | ❌ | Identity | Registered as identity |
| RegisterWorksheetRangeThemeStyleMutation | ❌ | Identity | Unit-level, no position |
| UnregisterWorksheetRangeThemeStyleMutation | ❌ | Identity | Unit-level, no position |
| SetWorksheetRangeThemeStyleMutation | ⏳ | Shifting | Range needs row shift |
| DeleteWorksheetRangeThemeStyleMutation | ⏳ | Shifting | Range needs row shift |

#### Worksheet Configuration

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| SetFrozenMutation | ⏳ | Shifting | start_row, y_split may need adjustment |
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
| SetWorksheetColumnCountMutation | ❌ | Identity | Column count not affected |
| SetWorksheetRowCountMutation | ⏳ | Adjustment | May need to increase row count |

#### Workbook

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| SetWorkbookNameMutation | ❌ | Identity | Workbook level, no position |

#### Number Format

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| SetNumfmtMutation | ⏳ | Shifting | ref_map ranges need row shift |
| RemoveNumfmtMutation | ⏳ | Shifting | ranges need row shift |

#### Other

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| EmptyMutation | ❌ | Identity | No-op mutation |

### Feature Plugin Mutations

#### Conditional Formatting

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| AddConditionalRuleMutation | ⏳ | Shifting | Rule ranges need row shift |
| DeleteConditionalRuleMutation | ❌ | Identity | Deletion by ID, no position |
| SetConditionalRuleMutation | ⏳ | Shifting | Rule ranges need row shift |
| MoveConditionalRuleMutation | ❌ | Identity | Reorders rules, no position |

#### Data Validation

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| AddDataValidationMutation | ⏳ | Shifting | Rule ranges need row shift |
| RemoveDataValidationMutation | ❌ | Identity | Deletion by ID, no position |
| UpdateDataValidationMutation | ⏳ | Shifting | Rule ranges need row shift |

#### Filter

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| SetSheetsFilterRangeMutation | ⏳ | Shifting | Filter range needs row shift |
| RemoveSheetsFilterMutation | ❌ | Identity | Removes filter, no position |
| SetSheetsFilterCriteriaMutation | ❌ | Identity | Column-based criteria |
| ReCalcSheetsFilterMutation | ❌ | Identity | Recalculation trigger |

#### Hyperlink

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| AddHyperLinkMutation | ⏳ | Shifting | Link row position needs shift |
| RemoveHyperLinkMutation | ❌ | Identity | Removal by ID, no position |
| UpdateHyperLinkMutation | ❌ | Identity | Update by ID, no position |
| UpdateHyperLinkRefMutation | ⏳ | Shifting | row field needs shift |
| UpdateRichHyperLinkMutation | ⏳ | Shifting | row field needs shift |

#### Note

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| UpdateNoteMutation | ⏳ | Shifting | row field needs shift |
| RemoveNoteMutation | ⏳ | Shifting | row field needs shift |
| ToggleNotePopupMutation | ⏳ | Shifting | row field needs shift |
| UpdateNotePositionMutation | ⏳ | Shifting | row field needs shift |

#### Pivot Table

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| AddPivotTableMutation | ⏳ | Shifting | Source/target ranges need shift |
| RemovePivotTableMutation | ❌ | Identity | Removal by ID, no position |
| SetPivotTableSourceRangeMutation | ⏳ | Shifting | Source range needs shift |
| SetPivotTableTargetCellMutation | ⏳ | Shifting | Target cell row needs shift |
| SetPivotTableFieldsConfigMutation | ❌ | Identity | Config only, no position |
| SetPivotTableCalculatedDataMutation | ❌ | Identity | Data only, no position |

#### Table

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| AddSheetTableMutation | ⏳ | Shifting | Table range needs row shift |
| DeleteSheetTableMutation | ❌ | Identity | Deletion by ID, no position |
| SetSheetTableMutation | ⏳ | Shifting | Table config may have range |
| SetSheetTableFilterMutation | ❌ | Identity | Column-based filter |

#### Thread Comment

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| AddCommentMutation | ⏳ | Shifting | Comment position needs shift |
| UpdateCommentMutation | ❌ | Identity | Update by ID, no position |
| UpdateCommentRefMutation | ⏳ | Shifting | ref position needs shift |
| ResolveCommentMutation | ❌ | Identity | Status update, no position |
| DeleteCommentMutation | ❌ | Identity | Deletion by ID, no position |

## Detailed Conflict Resolution

### InsertRowMutation vs InsertRowMutation (Self-Transform)

**Conflict Analysis**:
- **Dimension 1 (unitId)**: Same workbook required for conflict
- **Dimension 2 (subUnitId)**: Same worksheet required for conflict
- **Dimension 3 (Position)**: Order of inserts matters - earlier insert shifts later one
- **Dimension 4 (Feature ID)**: N/A

**Resolution Strategy**: Shifting

**Implementation**:
```rust
fn create_self_transform() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        // Early worksheet check (zero-copy for different sheets)
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return identity(m1, m2);
        }

        let m1_params: InsertRowMutationParams = parse(m1)?;
        let mut m2_params: InsertRowMutationParams = parse(m2)?;

        let m1_start = m1_params.range.start_row;
        let m1_count = m1_params.range.end_row - m1_start + 1;

        // If m2's insert position >= m1's insert position, shift m2 down
        if m2_params.range.start_row >= m1_start {
            m2_params.range.start_row += m1_count;
            m2_params.range.end_row += m1_count;
        }

        // m1 unchanged, m2 shifted
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
- [ ] Multiple rows inserted

**Status**: ✅ Implemented

---

### InsertRowMutation vs SetRangeValuesMutation

**Conflict Analysis**:
- **Dimension 1 (unitId)**: Same workbook required
- **Dimension 2 (subUnitId)**: Same worksheet required
- **Dimension 3 (Position)**: SetRangeValues cell positions need shifting

**Resolution Strategy**: Shifting (cell row keys)

**Implementation**:
```rust
fn create_transform_with_set_range_values() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return identity(m1, m2);
        }

        let m1_params: InsertRowMutationParams = parse(m1)?;
        let mut m2_params: SetRangeValuesMutationParams = parse(m2)?;

        let insert_row = m1_params.range.start_row;
        let insert_count = m1_params.range.end_row - insert_row + 1;

        // Shift cell value row keys
        if let Some(ref mut cell_value) = m2_params.cell_value {
            shift_row_keys_for_insert(cell_value, insert_row, insert_count);
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
- [x] Cells at insert position - shift down
- [x] Cells below insert - shift down
- [ ] Cells above insert - no shift
- [ ] Empty cell_value

**Status**: ✅ Implemented

---

### InsertRowMutation vs AddWorksheetMergeMutation

**Conflict Analysis**:
- **Dimension 3 (Position)**: Merge ranges need row shifting

**Resolution Strategy**: Shifting (range rows)

**Implementation**: Uses `shift_range_rows_for_insert()` utility

**Test Cases**:
- [x] Merge range below insert - shifts
- [x] Merge range spanning insert - expands
- [ ] Merge range above insert - no change

**Status**: ✅ Implemented (in merge.rs)

## Implementation Checklist

- [x] Self-transform registered in `transforms/sheets/insert_row.rs`
- [x] Transform with SetRangeValuesMutation implemented
- [x] Transform with AddWorksheetMergeMutation implemented (in merge.rs)
- [ ] All structural mutation transforms (move/remove) implemented
- [ ] All data mutation transforms implemented
- [ ] All feature plugin transforms implemented
- [x] Tests written for implemented scenarios
- [x] Zero-copy optimizations applied
- [x] This document updated with implementation status

## Notes

- InsertRow is one of the most frequently used mutations and affects many other mutations
- The key optimization is the early worksheet check before parsing
- Use `shift_row_keys_for_insert()` utility for HashMap-based cell data
- Use `shift_range_rows_for_insert()` utility for IRange-based data

## Last Updated

**Date**: 2026-01-31
**By**: Claude Code
**Changes**: Initial document creation with comprehensive coverage matrix
