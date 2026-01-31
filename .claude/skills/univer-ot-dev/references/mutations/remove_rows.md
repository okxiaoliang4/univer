# RemoveRowsMutation Conflict Analysis

**Mutation ID**: `sheet.mutation.remove-rows`
**File**: `mutations/sheets/remove_row_col_mutation.rs`
**Transform**: `transforms/sheets/remove_rows.rs`
**Tests**: `tests/remove_rows_tests.rs`

## Mutation Overview

**Purpose**: Removes one or more rows from a worksheet, shifting remaining rows up.

**Affects**:
- [x] Rows (primary - removes and shifts row positions)
- [ ] Columns
- [x] Cells/Ranges (indirect - cells removed or shifted)
- [ ] Worksheet properties
- [ ] Workbook properties
- [ ] Feature-specific

**Parameters**:
```rust
pub struct RemoveRowsMutationParams {
    pub unit_id: String,           // Workbook ID
    pub sub_unit_id: String,       // Worksheet ID
    pub range: IRange,             // Range defining removal (start_row, end_row)
}
```

**Key Derived Values**:
- `remove_start = range.start_row`
- `remove_end = range.end_row`
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
- Rows: Rows in [remove_start, remove_end] are deleted; rows > remove_end shift up
- Columns: Not affected
- Cells: Cells in removed rows deleted; cells below shift up

**Conflict scenarios**:
1. **Complete overlap**: Target entirely within removed range → Target removed
2. **Partial overlap (top)**: Target starts above, ends within → Target truncated
3. **Partial overlap (bottom)**: Target starts within, ends below → Target truncated and shifted
4. **Below removed**: Target below removed range → Shift up
5. **Above removed**: Target above removed range → No change

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
| InsertRowMutation | ⏳ | Shifting + Conflict | Complex: insert may be in removed range |
| InsertColMutation | ❌ | Identity | Row removal doesn't affect column positions |
| RemoveRowsMutation (self) | ✅ | Shifting + Removal | Overlapping removes need careful handling |
| RemoveColMutation | ❌ | Identity | Row removal doesn't affect column positions |
| MoveRowsMutation | ⏳ | Shifting + Removal | Source/target may be in removed range |
| MoveColumnsMutation | ❌ | Identity | Row removal doesn't affect column positions |
| MoveRangeMutation | ⏳ | Shifting + Removal | from_range/to_range may overlap removed |

#### Data Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| SetRangeValuesMutation | ✅ | Shifting + Removal | Cells in removed rows deleted, rest shifted |
| SetRowDataMutation | ⏳ | Shifting + Removal | Row data in range deleted, rest shifted |
| SetColDataMutation | ❌ | Identity | Column data not affected by row removal |
| ReorderRangeMutation | ⏳ | Shifting + Removal | Range may overlap removed rows |

#### Merge Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| AddWorksheetMergeMutation | ✅ | Shifting + Removal | Merges in removed rows deleted or adjusted |
| RemoveWorksheetMergeMutation | ⏳ | Shifting + Removal | Merges may be in removed range |

#### Dimension Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| SetWorksheetRowHeightMutation | ⏳ | Shifting + Removal | Row indices need adjustment/removal |
| SetWorksheetColWidthMutation | ❌ | Identity | Column dimensions not affected |
| SetWorksheetRowIsAutoHeightMutation | ⏳ | Shifting + Removal | Row indices need adjustment/removal |
| SetWorksheetRowAutoHeightMutation | ⏳ | Shifting + Removal | Row indices need adjustment/removal |
| MarkDirtyRowAutoHeightMutation | ⏳ | Shifting + Removal | Row indices need adjustment/removal |
| CancelMarkDirtyRowAutoHeightMutation | ⏳ | Shifting + Removal | Row indices need adjustment/removal |

#### Visibility Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| SetRowVisibleMutation | ⏳ | Shifting + Removal | Row ranges need adjustment/removal |
| SetRowHiddenMutation | ⏳ | Shifting + Removal | Row ranges need adjustment/removal |
| SetColVisibleMutation | ❌ | Identity | Column visibility not affected |
| SetColHiddenMutation | ❌ | Identity | Column visibility not affected |

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
| SetWorksheetRowCountMutation | ⏳ | Adjustment | Row count should decrease |

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

#### Conditional Formatting

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| AddConditionalRuleMutation | ⏳ | Shifting + Removal | Rule ranges may overlap |
| DeleteConditionalRuleMutation | ❌ | Identity | Deletion by ID, no position |
| SetConditionalRuleMutation | ⏳ | Shifting + Removal | Rule ranges may overlap |
| MoveConditionalRuleMutation | ❌ | Identity | Reorders rules, no position |

#### Data Validation

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| AddDataValidationMutation | ⏳ | Shifting + Removal | Rule ranges may overlap |
| RemoveDataValidationMutation | ❌ | Identity | Deletion by ID, no position |
| UpdateDataValidationMutation | ⏳ | Shifting + Removal | Rule ranges may overlap |

#### Filter

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| SetSheetsFilterRangeMutation | ⏳ | Shifting + Removal | Filter range may overlap |
| RemoveSheetsFilterMutation | ❌ | Identity | Removes filter, no position |
| SetSheetsFilterCriteriaMutation | ❌ | Identity | Column-based criteria |
| ReCalcSheetsFilterMutation | ❌ | Identity | Recalculation trigger |

#### Hyperlink

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| AddHyperLinkMutation | ⏳ | Shifting + Removal | Link may be in removed row |
| RemoveHyperLinkMutation | ❌ | Identity | Removal by ID, no position |
| UpdateHyperLinkMutation | ❌ | Identity | Update by ID, no position |
| UpdateHyperLinkRefMutation | ⏳ | Shifting + Removal | row may be in removed range |
| UpdateRichHyperLinkMutation | ⏳ | Shifting + Removal | row may be in removed range |

#### Note

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| UpdateNoteMutation | ⏳ | Shifting + Removal | row may be in removed range |
| RemoveNoteMutation | ⏳ | Shifting + Removal | row may be in removed range |
| ToggleNotePopupMutation | ⏳ | Shifting + Removal | row may be in removed range |
| UpdateNotePositionMutation | ⏳ | Shifting + Removal | row may be in removed range |

#### Pivot Table

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| AddPivotTableMutation | ⏳ | Shifting + Removal | Source/target ranges may overlap |
| RemovePivotTableMutation | ❌ | Identity | Removal by ID, no position |
| SetPivotTableSourceRangeMutation | ⏳ | Shifting + Removal | Source range may overlap |
| SetPivotTableTargetCellMutation | ⏳ | Shifting + Removal | Target cell may be in removed range |
| SetPivotTableFieldsConfigMutation | ❌ | Identity | Config only, no position |
| SetPivotTableCalculatedDataMutation | ❌ | Identity | Data only, no position |

#### Table

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| AddSheetTableMutation | ⏳ | Shifting + Removal | Table range may overlap |
| DeleteSheetTableMutation | ❌ | Identity | Deletion by ID, no position |
| SetSheetTableMutation | ⏳ | Shifting + Removal | Table config may have range |
| SetSheetTableFilterMutation | ❌ | Identity | Column-based filter |

#### Thread Comment

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| AddCommentMutation | ⏳ | Shifting + Removal | Comment may be in removed row |
| UpdateCommentMutation | ❌ | Identity | Update by ID, no position |
| UpdateCommentRefMutation | ⏳ | Shifting + Removal | ref may be in removed row |
| ResolveCommentMutation | ❌ | Identity | Status update, no position |
| DeleteCommentMutation | ❌ | Identity | Deletion by ID, no position |

## Detailed Conflict Resolution

### RemoveRowsMutation vs RemoveRowsMutation (Self-Transform)

**Conflict Analysis**:
- **Dimension 3 (Position)**: Overlapping removes need careful handling

**Resolution Strategy**: Shifting + Removal

**Scenarios**:
1. **No overlap**: Both removes independent, second shifts based on first
2. **Complete overlap**: One remove encompasses the other → Second becomes no-op
3. **Partial overlap**: Ranges intersect → Adjust second range

**Implementation**:
```rust
fn create_self_transform() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return identity(m1, m2);
        }

        let m1_params: RemoveRowsMutationParams = parse(m1)?;
        let mut m2_params: RemoveRowsMutationParams = parse(m2)?;

        let m1_start = m1_params.range.start_row;
        let m1_end = m1_params.range.end_row;
        let m1_count = m1_end - m1_start + 1;

        // Apply shift_range_rows_for_remove to m2's range
        let should_keep = shift_range_rows_for_remove(
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
- [x] No overlap - second shifts up
- [x] Complete overlap - second removed
- [x] Partial overlap (top) - second truncated
- [x] Partial overlap (bottom) - second truncated and shifted

**Status**: ✅ Implemented

---

### RemoveRowsMutation vs SetRangeValuesMutation

**Conflict Analysis**:
- **Dimension 3 (Position)**: Cells in removed rows should be deleted

**Resolution Strategy**: Shifting + Removal (cell-level)

**Implementation**:
```rust
fn create_transform_with_set_range_values() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return identity(m1, m2);
        }

        let m1_params: RemoveRowsMutationParams = parse(m1)?;
        let mut m2_params: SetRangeValuesMutationParams = parse(m2)?;

        let remove_start = m1_params.range.start_row;
        let remove_end = m1_params.range.end_row;

        // Shift/remove cell values
        if let Some(ref mut cell_value) = m2_params.cell_value {
            shift_row_keys_for_remove(cell_value, remove_start, remove_end);
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
- [x] Cells in removed range - deleted
- [x] Cells below removed range - shifted up
- [x] Cells above removed range - unchanged

**Status**: ✅ Implemented

---

### RemoveRowsMutation vs AddWorksheetMergeMutation

**Conflict Analysis**:
- **Dimension 3 (Position)**: Merge ranges may overlap with removed rows

**Resolution Strategy**: Shifting + Removal

**Implementation**: Uses `shift_range_rows_for_remove()` utility with `retain_mut()`

**Status**: ✅ Implemented (in merge.rs)

## Implementation Checklist

- [x] Self-transform registered in `transforms/sheets/remove_rows.rs`
- [x] Transform with SetRangeValuesMutation implemented
- [x] Transform with AddWorksheetMergeMutation implemented (in merge.rs)
- [ ] All structural mutation transforms implemented
- [ ] All feature plugin transforms implemented
- [x] Tests written for implemented scenarios
- [x] Zero-copy optimizations applied
- [x] This document updated with implementation status

## Notes

- RemoveRows is more complex than InsertRow due to removal + shifting
- Must handle partial overlaps carefully (truncation)
- Use `shift_row_keys_for_remove()` utility for HashMap-based cell data
- Use `shift_range_rows_for_remove()` utility for IRange-based data
- The utility returns `false` if range is entirely removed

## Last Updated

**Date**: 2026-01-31
**By**: Claude Code
**Changes**: Initial document creation with comprehensive coverage matrix
