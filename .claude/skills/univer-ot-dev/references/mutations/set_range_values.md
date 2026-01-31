# SetRangeValuesMutation Conflict Analysis

**Mutation ID**: `sheet.mutation.set-range-values`
**File**: `mutations/sheets/set_range_values_mutation.rs`
**Transform**: `transforms/sheets/set_range_values.rs`
**Tests**: `tests/set_range_values_tests.rs`

## Mutation Overview

**Purpose**: Sets cell values in a range of cells. This is the primary mutation for modifying cell data.

**Affects**:
- [ ] Rows
- [ ] Columns
- [x] Cells/Ranges (primary - modifies cell values)
- [ ] Worksheet properties
- [ ] Workbook properties
- [ ] Feature-specific

**Parameters**:
```rust
pub struct SetRangeValuesMutationParams {
    pub unit_id: String,           // Workbook ID
    pub sub_unit_id: String,       // Worksheet ID
    pub cell_value: Option<IObjectMatrixPrimitiveType>,  // HashMap<row, HashMap<col, ICellData>>
    pub options: Option<SetRangeValuesOptions>,  // Optional settings
}
```

**Data Structure**:
```json
{
  "cellValue": {
    "0": {          // Row index as string
      "0": { "v": "A1" },  // Column index as string -> cell data
      "1": { "v": "B1" }
    },
    "1": {
      "0": { "v": "A2" }
    }
  }
}
```

## Conflict Dimensions Analysis

### Dimension 1: Unit ID (Workbook)
- **Conflicts when**: Same `unit_id`
- **No conflict when**: Different `unit_id` → Identity transform

### Dimension 2: Sub-Unit ID (Worksheet)
- **Conflicts when**: Same `unit_id` AND same `sub_unit_id`
- **No conflict when**: Different worksheets → Identity transform

### Dimension 3: Position (Row/Col/Range)
**Positions affected by this mutation**:
- Rows: Only rows with data in cell_value
- Columns: Only columns with data in cell_value
- Cells: Specific cells defined by row/column keys

**Conflict scenarios**:
1. **Same cell**: Both mutations modify the same cell → LWW (m2 wins)
2. **Different cells**: No conflict → Both apply independently
3. **Partial overlap**: Some cells conflict → Cell-level LWW

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
| InsertRowMutation | ✅ | Shifting | Cell row keys shifted (registered in insert_row.rs) |
| InsertColMutation | ✅ | Shifting | Cell column keys shifted (registered in insert_col.rs) |
| RemoveRowsMutation | ✅ | Shifting + Removal | Cells in removed rows deleted (registered in remove_rows.rs) |
| RemoveColMutation | ✅ | Shifting + Removal | Cells in removed columns deleted (registered in remove_col.rs) |
| MoveRowsMutation | ⏳ | Complex | Cells in source/target affected |
| MoveColumnsMutation | ⏳ | Complex | Cells in source/target affected |
| MoveRangeMutation | ⏳ | Complex | Cells in source/target affected |

#### Data Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| SetRangeValuesMutation (self) | ✅ | LWW (cell-level) | Conflicting cells: m2 wins |
| SetRowDataMutation | ❌ | Identity | Row metadata, not cell values |
| SetColDataMutation | ❌ | Identity | Column metadata, not cell values |
| ReorderRangeMutation | ⏳ | Complex | Cell positions may change |

#### Merge Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| AddWorksheetMergeMutation | ❌ | Identity | Merge doesn't affect cell values |
| RemoveWorksheetMergeMutation | ❌ | Identity | Unmerge doesn't affect cell values |

#### Dimension Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| SetWorksheetRowHeightMutation | ❌ | Identity | Row dimensions don't affect values |
| SetWorksheetColWidthMutation | ❌ | Identity | Column dimensions don't affect values |
| SetWorksheetRowIsAutoHeightMutation | ❌ | Identity | Row dimensions don't affect values |
| SetWorksheetRowAutoHeightMutation | ❌ | Identity | Row dimensions don't affect values |
| MarkDirtyRowAutoHeightMutation | ❌ | Identity | Row dimensions don't affect values |
| CancelMarkDirtyRowAutoHeightMutation | ❌ | Identity | Row dimensions don't affect values |

#### Visibility Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| SetRowVisibleMutation | ❌ | Identity | Visibility doesn't affect values |
| SetRowHiddenMutation | ❌ | Identity | Visibility doesn't affect values |
| SetColVisibleMutation | ❌ | Identity | Visibility doesn't affect values |
| SetColHiddenMutation | ❌ | Identity | Visibility doesn't affect values |

#### Protection Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| AddRangeProtectionMutation | ❌ | Identity | Protection doesn't affect values |
| DeleteRangeProtectionMutation | ❌ | Identity | Protection doesn't affect values |
| SetRangeProtectionMutation | ❌ | Identity | Protection doesn't affect values |
| AddWorksheetProtectionMutation | ❌ | Identity | Protection doesn't affect values |
| DeleteWorksheetProtectionMutation | ❌ | Identity | Protection doesn't affect values |
| SetWorksheetProtectionMutation | ❌ | Identity | Protection doesn't affect values |
| SetWorksheetPermissionPointsMutation | ❌ | Identity | Protection doesn't affect values |

#### Theme/Style Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| AddRangeThemeMutation | ❌ | Identity | Theme doesn't affect cell values |
| RemoveRangeThemeMutation | ❌ | Identity | Theme doesn't affect cell values |
| SetRangeThemeMutation | ❌ | Identity | Theme doesn't affect cell values |
| RegisterWorksheetRangeThemeStyleMutation | ❌ | Identity | Theme doesn't affect cell values |
| UnregisterWorksheetRangeThemeStyleMutation | ❌ | Identity | Theme doesn't affect cell values |
| SetWorksheetRangeThemeStyleMutation | ❌ | Identity | Theme doesn't affect cell values |
| DeleteWorksheetRangeThemeStyleMutation | ❌ | Identity | Theme doesn't affect cell values |

#### Worksheet Configuration

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| SetFrozenMutation | ❌ | Identity | Frozen state doesn't affect values |
| SetGridlinesColorMutation | ❌ | Identity | Gridlines don't affect values |
| ToggleGridlinesMutation | ❌ | Identity | Gridlines don't affect values |
| SetTabColorMutation | ❌ | Identity | Tab color doesn't affect values |
| SetWorksheetHideMutation | ❌ | Identity | Visibility doesn't affect values |
| SetWorksheetNameMutation | ❌ | Identity | Name doesn't affect values |
| SetWorksheetOrderMutation | ❌ | Identity | Order doesn't affect values |
| SetWorksheetRightToLeftMutation | ❌ | Identity | RTL doesn't affect values |
| SetWorksheetDefaultStyleMutation | ❌ | Identity | Default style doesn't affect values |

#### Sheet Management

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| InsertSheetMutation | ❌ | Identity | Different worksheet |
| RemoveSheetMutation | ❌ | Identity | Different worksheet |
| CopyWorksheetEndMutation | ❌ | Identity | Different worksheet |

#### Worksheet Size

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| SetWorksheetColumnCountMutation | ❌ | Identity | Count doesn't affect existing values |
| SetWorksheetRowCountMutation | ❌ | Identity | Count doesn't affect existing values |

#### Workbook

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| SetWorkbookNameMutation | ❌ | Identity | Workbook name doesn't affect values |

#### Number Format

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| SetNumfmtMutation | ❌ | Identity | Format doesn't change stored values |
| RemoveNumfmtMutation | ❌ | Identity | Format doesn't change stored values |

#### Other

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| EmptyMutation | ❌ | Identity | No-op mutation |

### Feature Plugin Mutations

#### Conditional Formatting

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| AddConditionalRuleMutation | ❌ | Identity | Formatting rules, not values |
| DeleteConditionalRuleMutation | ❌ | Identity | Formatting rules, not values |
| SetConditionalRuleMutation | ❌ | Identity | Formatting rules, not values |
| MoveConditionalRuleMutation | ❌ | Identity | Formatting rules, not values |

#### Data Validation

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| AddDataValidationMutation | ❌ | Identity | Validation rules, not values |
| RemoveDataValidationMutation | ❌ | Identity | Validation rules, not values |
| UpdateDataValidationMutation | ❌ | Identity | Validation rules, not values |

#### Filter

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| SetSheetsFilterRangeMutation | ❌ | Identity | Filter config, not values |
| RemoveSheetsFilterMutation | ❌ | Identity | Filter config, not values |
| SetSheetsFilterCriteriaMutation | ❌ | Identity | Filter config, not values |
| ReCalcSheetsFilterMutation | ❌ | Identity | Filter trigger, not values |

#### Hyperlink

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| AddHyperLinkMutation | ❌ | Identity | Hyperlink metadata, not cell values |
| RemoveHyperLinkMutation | ❌ | Identity | Hyperlink metadata, not cell values |
| UpdateHyperLinkMutation | ❌ | Identity | Hyperlink metadata, not cell values |
| UpdateHyperLinkRefMutation | ❌ | Identity | Hyperlink metadata, not cell values |
| UpdateRichHyperLinkMutation | ❌ | Identity | Hyperlink metadata, not cell values |

#### Note

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| UpdateNoteMutation | ❌ | Identity | Note content, not cell values |
| RemoveNoteMutation | ❌ | Identity | Note content, not cell values |
| ToggleNotePopupMutation | ❌ | Identity | Note state, not cell values |
| UpdateNotePositionMutation | ❌ | Identity | Note position, not cell values |

#### Pivot Table

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| AddPivotTableMutation | ❌ | Identity | Pivot config, not source values |
| RemovePivotTableMutation | ❌ | Identity | Pivot config, not source values |
| SetPivotTableSourceRangeMutation | ❌ | Identity | Pivot config, not source values |
| SetPivotTableTargetCellMutation | ❌ | Identity | Pivot config, not source values |
| SetPivotTableFieldsConfigMutation | ❌ | Identity | Pivot config, not source values |
| SetPivotTableCalculatedDataMutation | ❌ | Identity | Pivot output, not source values |

#### Table

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| AddSheetTableMutation | ❌ | Identity | Table config, not cell values |
| DeleteSheetTableMutation | ❌ | Identity | Table config, not cell values |
| SetSheetTableMutation | ❌ | Identity | Table config, not cell values |
| SetSheetTableFilterMutation | ❌ | Identity | Table filter, not cell values |

#### Thread Comment

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| AddCommentMutation | ❌ | Identity | Comment content, not cell values |
| UpdateCommentMutation | ❌ | Identity | Comment content, not cell values |
| UpdateCommentRefMutation | ❌ | Identity | Comment ref, not cell values |
| ResolveCommentMutation | ❌ | Identity | Comment state, not cell values |
| DeleteCommentMutation | ❌ | Identity | Comment removal, not cell values |

## Detailed Conflict Resolution

### SetRangeValuesMutation vs SetRangeValuesMutation (Self-Transform)

**Conflict Analysis**:
- **Dimension 3 (Position)**: Cell-level conflict when both modify same cell

**Resolution Strategy**: Last-Write-Wins (LWW) at cell level

**Key Principle**:
- Only conflicting cells use LWW (m2 wins)
- Non-conflicting cells in m1 are preserved
- This is **cell-level granularity**, not range-level

**Implementation**:
```rust
fn create_self_transform() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return identity(m1, m2);
        }

        let m1_params: SetRangeValuesMutationParams = parse(m1)?;
        let m2_params: SetRangeValuesMutationParams = parse(m2)?;

        let mut m1_prime_params = m1_params.clone();
        let m2_prime_params = m2_params.clone();  // m2 unchanged (wins)

        if let (Some(m1_cells), Some(m2_cells)) = (&m1_params.cell_value, &m2_params.cell_value) {
            let mut m1_prime_cells = m1_cells.clone();

            // For each cell in m1, check if m2 also modifies it
            for (row_key, m1_row) in m1_cells {
                if let Some(m2_row) = m2_cells.get(row_key) {
                    let mut m1_prime_row = m1_row.clone();

                    for col_key in m1_row.keys() {
                        if m2_row.contains_key(col_key) {
                            // Conflict! LWW: m2 wins, remove from m1_prime
                            m1_prime_row.remove(col_key);
                        }
                    }

                    // Update or remove the row
                    if m1_prime_row.is_empty() {
                        m1_prime_cells.remove(row_key);
                    } else {
                        m1_prime_cells.insert(row_key.clone(), m1_prime_row);
                    }
                }
            }

            m1_prime_params.cell_value = if m1_prime_cells.is_empty() {
                None
            } else {
                Some(m1_prime_cells)
            };
        }

        TransformResultRef {
            m1_prime: MutationOutcome::Modified(serialize(m1_prime_params)),
            m2_prime: MutationOutcome::Modified(serialize(m2_prime_params)),
            error: None,
        }
    })
}
```

**Test Cases**:
- [x] Different worksheets (identity)
- [x] Same cell - m2 wins, m1 cell removed
- [x] Different cells - both preserved
- [x] Partial overlap - only conflicting cells removed from m1
- [x] m1 empty after conflict - cell_value becomes None
- [ ] Empty cell_value in one mutation

**Status**: ✅ Implemented

## Implementation Checklist

- [x] Self-transform registered in `transforms/sheets/set_range_values.rs`
- [x] Transform with InsertRowMutation implemented (in insert_row.rs)
- [x] Transform with InsertColMutation implemented (in insert_col.rs)
- [x] Transform with RemoveRowsMutation implemented (in remove_rows.rs)
- [x] Transform with RemoveColMutation implemented (in remove_col.rs)
- [ ] Transform with MoveRowsMutation implemented
- [ ] Transform with MoveColsMutation implemented
- [ ] Transform with MoveRangeMutation implemented
- [x] Tests written for implemented scenarios
- [x] Zero-copy optimizations applied
- [x] This document updated with implementation status

## Notes

- SetRangeValues is the most commonly used mutation
- The key insight is **cell-level LWW**, not mutation-level
- Non-conflicting cells are always preserved
- This mutation is a "passive" target - structural mutations shift its cells
- Most other mutations are identity transforms (no data conflict)

## Last Updated

**Date**: 2026-01-31
**By**: Claude Code
**Changes**: Initial document creation with comprehensive coverage matrix
