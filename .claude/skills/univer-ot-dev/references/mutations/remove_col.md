# RemoveColMutation Conflict Analysis

**Mutation ID**: `sheet.mutation.remove-col`
**File**: `mutations/sheets/remove_row_col_mutation.rs`
**Transform**: `transforms/sheets/remove_col.rs`
**Tests**: `tests/remove_col_tests.rs`

## Mutation Overview

**Purpose**: Removes one or more columns from a worksheet, shifting remaining columns left.

**Affects**:
- [ ] Rows
- [x] Columns (primary - removes and shifts column positions)
- [x] Cells/Ranges (indirect - cells removed or shifted)
- [ ] Worksheet properties
- [ ] Workbook properties
- [ ] Feature-specific

**Parameters**:
```rust
pub struct RemoveColMutationParams {
    pub unit_id: String,           // Workbook ID
    pub sub_unit_id: String,       // Worksheet ID
    pub range: IRange,             // Range defining removal (start_column, end_column)
}
```

**Key Derived Values**:
- `remove_start = range.start_column`
- `remove_end = range.end_column`
- `remove_count = remove_end - remove_start + 1`

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
- Columns: Columns in [remove_start, remove_end] are deleted; columns > remove_end shift left
- Cells: Cells in removed columns deleted; cells to the right shift left

**Conflict scenarios**:
1. **Complete overlap**: Target entirely within removed range → Target removed
2. **Partial overlap (left)**: Target starts left, ends within → Target truncated
3. **Partial overlap (right)**: Target starts within, ends right → Target truncated and shifted
4. **Right of removed**: Target to the right → Shift left
5. **Left of removed**: Target to the left → No change

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
| InsertRowMutation | ❌ | Identity | Column removal doesn't affect row positions |
| InsertColMutation | ⏳ | Shifting + Conflict | Complex: insert may be in removed range |
| RemoveRowsMutation | ❌ | Identity | Column removal doesn't affect row positions |
| RemoveColMutation (self) | ✅ | Shifting + Removal | Overlapping removes need careful handling |
| MoveRowsMutation | ❌ | Identity | Column removal doesn't affect row positions |
| MoveColumnsMutation | ⏳ | Shifting + Removal | Source/target may be in removed range |
| MoveRangeMutation | ⏳ | Shifting + Removal | from_range/to_range may overlap removed |

#### Data Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| SetRangeValuesMutation | ✅ | Shifting + Removal | Cells in removed columns deleted, rest shifted |
| SetRowDataMutation | ❌ | Identity | Row metadata not affected by column removal |
| SetColDataMutation | ⏳ | Shifting + Removal | Column data in range deleted, rest shifted |
| ReorderRangeMutation | ⏳ | Shifting + Removal | Range may overlap removed columns |

#### Merge Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| AddWorksheetMergeMutation | ✅ | Shifting + Removal | Merges in removed columns deleted or adjusted |
| RemoveWorksheetMergeMutation | ⏳ | Shifting + Removal | Merges may be in removed range |

#### Dimension Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| SetWorksheetRowHeightMutation | ❌ | Identity | Row dimensions not affected |
| SetWorksheetColWidthMutation | ⏳ | Shifting + Removal | Column indices need adjustment/removal |
| SetWorksheetRowIsAutoHeightMutation | ❌ | Identity | Row dimensions not affected |
| SetWorksheetRowAutoHeightMutation | ❌ | Identity | Row dimensions not affected |
| MarkDirtyRowAutoHeightMutation | ❌ | Identity | Row dimensions not affected |
| CancelMarkDirtyRowAutoHeightMutation | ❌ | Identity | Row dimensions not affected |

#### Visibility Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| SetRowVisibleMutation | ❌ | Identity | Row visibility not affected |
| SetRowHiddenMutation | ❌ | Identity | Row visibility not affected |
| SetColVisibleMutation | ⏳ | Shifting + Removal | Column ranges need adjustment/removal |
| SetColHiddenMutation | ⏳ | Shifting + Removal | Column ranges need adjustment/removal |

#### Protection Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| AddRangeProtectionMutation | ⏳ | Shifting + Removal | Protection ranges may overlap |
| DeleteRangeProtectionMutation | ⏳ | Shifting + Removal | Protection ranges may overlap |
| SetRangeProtectionMutation | ⏳ | Shifting + Removal | Protection ranges may overlap |
| AddWorksheetProtectionMutation | ❌ | Identity | Worksheet-level, no position |
| DeleteWorksheetProtectionMutation | ❌ | Identity | Worksheet-level, no position |
| SetWorksheetProtectionMutation | ❌ | Identity | Worksheet-level, no position |
| SetWorksheetPermissionPointsMutation | ❌ | Identity | Worksheet-level, no position |

#### Theme/Style Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| AddRangeThemeMutation | ⏳ | Shifting + Removal | Theme ranges may overlap |
| RemoveRangeThemeMutation | ⏳ | Shifting + Removal | Theme ranges may overlap |
| SetRangeThemeMutation | ⏳ | Shifting + Removal | Theme ranges may overlap |
| RegisterWorksheetRangeThemeStyleMutation | ❌ | Identity | Unit-level, no position |
| UnregisterWorksheetRangeThemeStyleMutation | ❌ | Identity | Unit-level, no position |
| SetWorksheetRangeThemeStyleMutation | ⏳ | Shifting + Removal | Range may overlap |
| DeleteWorksheetRangeThemeStyleMutation | ⏳ | Shifting + Removal | Range may overlap |

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
| SetWorksheetColumnCountMutation | ⏳ | Adjustment | Column count should decrease |
| SetWorksheetRowCountMutation | ❌ | Identity | Row count not affected |

#### Workbook

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| SetWorkbookNameMutation | ❌ | Identity | Workbook level, no position |

#### Number Format

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| SetNumfmtMutation | ⏳ | Shifting + Removal | ref_map ranges may overlap |
| RemoveNumfmtMutation | ⏳ | Shifting + Removal | ranges may overlap |

#### Other

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| EmptyMutation | ❌ | Identity | No-op mutation |

### Feature Plugin Mutations

(Similar to RemoveRowsMutation - all position-based mutations need column shifting/removal)

#### Filter

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| SetSheetsFilterRangeMutation | ⏳ | Shifting + Removal | Filter range may overlap |
| RemoveSheetsFilterMutation | ❌ | Identity | Removes filter, no position |
| SetSheetsFilterCriteriaMutation | ⏳ | Shifting + Removal | col field may be in removed range |
| ReCalcSheetsFilterMutation | ❌ | Identity | Recalculation trigger |

#### Table

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| SetSheetTableFilterMutation | ⏳ | Shifting + Removal | column field may be in removed range |

## Detailed Conflict Resolution

### RemoveColMutation vs RemoveColMutation (Self-Transform)

**Conflict Analysis**:
- **Dimension 3 (Position)**: Overlapping removes need careful handling

**Resolution Strategy**: Shifting + Removal

**Implementation**:
```rust
fn create_self_transform() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return identity(m1, m2);
        }

        let m1_params: RemoveColMutationParams = parse(m1)?;
        let mut m2_params: RemoveColMutationParams = parse(m2)?;

        let m1_start = m1_params.range.start_column;
        let m1_end = m1_params.range.end_column;

        // Apply shift_range_cols_for_remove to m2's range
        let should_keep = shift_range_cols_for_remove(
            &mut m2_params.range, m1_start, m1_end
        );

        if !should_keep {
            // m2's range was entirely within m1's removed range
            return TransformResultRef {
                m1_prime: MutationOutcome::Unchanged(m1),
                m2_prime: MutationOutcome::Removed,
                error: None,
            };
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
- [x] No overlap - second shifts left
- [x] Complete overlap - second removed
- [x] Partial overlap - second truncated

**Status**: ✅ Implemented

---

### RemoveColMutation vs SetRangeValuesMutation

**Conflict Analysis**:
- **Dimension 3 (Position)**: Cells in removed columns should be deleted

**Resolution Strategy**: Shifting + Removal (cell-level)

**Implementation**: Uses `shift_col_keys_for_remove()` utility

**Status**: ✅ Implemented

---

### RemoveColMutation vs AddWorksheetMergeMutation

**Conflict Analysis**:
- **Dimension 3 (Position)**: Merge ranges may overlap with removed columns

**Resolution Strategy**: Shifting + Removal

**Implementation**: Uses `shift_range_cols_for_remove()` utility

**Status**: ✅ Implemented (in merge.rs)

## Implementation Checklist

- [x] Self-transform registered in `transforms/sheets/remove_col.rs`
- [x] Transform with SetRangeValuesMutation implemented
- [x] Transform with AddWorksheetMergeMutation implemented (in merge.rs)
- [ ] All structural mutation transforms implemented
- [ ] All feature plugin transforms implemented
- [x] Tests written for implemented scenarios
- [x] Zero-copy optimizations applied
- [x] This document updated with implementation status

## Notes

- RemoveCol is symmetric to RemoveRows but affects columns
- Use `shift_col_keys_for_remove()` utility for HashMap-based cell data
- Use `shift_range_cols_for_remove()` utility for IRange-based data
- The utility returns `false` if range is entirely removed

## Last Updated

**Date**: 2026-01-31
**By**: Claude Code
**Changes**: Initial document creation with comprehensive coverage matrix
