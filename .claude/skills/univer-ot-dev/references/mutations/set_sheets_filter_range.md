# SetSheetsFilterRangeMutation Conflict Analysis

**Mutation ID**: `sheet.mutation.set-sheets-filter-range`
**File**: `mutations/sheets_filter/set_filter_range_mutation.rs`
**Transform**: `transforms/sheets_filter/filter.rs`
**Tests**: `tests/sheets_filter_tests.rs`

## Mutation Overview

**Purpose**: Sets or updates the filter range for a worksheet, defining which cells are included in the auto-filter.

**Affects**:
- [ ] Rows
- [ ] Columns
- [x] Cells/Ranges (defines filter area)
- [ ] Worksheet properties
- [ ] Workbook properties
- [x] Feature-specific (filter)

**Parameters**:
```rust
pub struct SetSheetsFilterRangeMutationParams {
    pub unit_id: String,           // Workbook ID
    pub sub_unit_id: String,       // Worksheet ID
    pub range: IRange,             // Filter range
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
- Range: The filter range (start_row, end_row, start_column, end_column)

**Conflict scenarios**:
1. **Row insert/remove**: Filter range rows need adjustment
2. **Column insert/remove**: Filter range columns need adjustment
3. **Same filter range**: LWW - last write wins

### Dimension 4: Feature Plugin ID
**Feature-specific IDs**: Filter is unique per worksheet (only one filter range)

**Conflicts when**: Same worksheet → LWW for filter range

## Transform Coverage Matrix

### Core Sheets Mutations

#### Structural Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| InsertRowMutation | ✅ | Shifting | Filter range rows shift |
| InsertColMutation | ✅ | Shifting | Filter range columns shift |
| RemoveRowsMutation | ✅ | Shifting + Removal | Filter range shrinks/removed |
| RemoveColMutation | ✅ | Shifting + Removal | Filter range shrinks/removed |
| MoveRowsMutation | ⏳ | Complex | Filter range may be affected |
| MoveColumnsMutation | ⏳ | Complex | Filter range may be affected |
| MoveRangeMutation | ⏳ | Complex | Filter range may be affected |

#### Data Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| SetRangeValuesMutation | ❌ | Identity | Cell values independent of filter |
| SetRowDataMutation | ❌ | Identity | Row metadata independent of filter |
| SetColDataMutation | ❌ | Identity | Column metadata independent of filter |

#### Most Other Core Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| (Most mutations) | ❌ | Identity | Filter range independent |

### Filter Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| SetSheetsFilterRangeMutation (self) | ✅ | LWW | m2 wins on same worksheet |
| SetSheetsFilterCriteriaMutation | ❌ | Identity | Different aspects of filter |
| RemoveSheetsFilterMutation | ⏳ | Conflict | Set vs Remove - set wins |
| ReCalcSheetsFilterMutation | ❌ | Identity | Trigger only |

## Detailed Conflict Resolution

### InsertRowMutation vs SetSheetsFilterRangeMutation

**Conflict Analysis**:
- **Dimension 3 (Position)**: Filter range needs row shifting

**Resolution Strategy**: Shifting

**Implementation**:
```rust
fn create_insert_row_vs_filter() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return identity(m1, m2);
        }

        let m1_params: InsertRowMutationParams = parse(m1)?;
        let mut m2_params: SetSheetsFilterRangeMutationParams = parse(m2)?;

        let insert_start = m1_params.range.start_row;
        let insert_count = m1_params.range.end_row - insert_start + 1;

        shift_range_rows_for_insert(&mut m2_params.range, insert_start, insert_count);

        TransformResultRef {
            m1_prime: MutationOutcome::Unchanged(m1),
            m2_prime: MutationOutcome::Modified(serialize(m2_params)),
            error: None,
        }
    })
}
```

**Status**: ✅ Implemented

---

### SetSheetsFilterRangeMutation vs SetSheetsFilterRangeMutation (Self-Transform)

**Conflict Analysis**:
- Only one filter per worksheet
- Same worksheet = conflict

**Resolution Strategy**: LWW (m2 wins)

**Implementation**:
```rust
fn create_self_transform() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return identity(m1, m2);
        }

        // Same worksheet: m2 wins, m1 is no-op
        TransformResultRef {
            m1_prime: MutationOutcome::Removed,  // m1 removed (m2 wins)
            m2_prime: MutationOutcome::Unchanged(m2),
            error: None,
        }
    })
}
```

**Status**: ✅ Implemented

## Implementation Checklist

- [x] Self-transform (LWW) implemented
- [x] Transforms with structural mutations implemented
- [x] Tests written for implemented scenarios
- [x] Zero-copy optimizations applied
- [x] This document updated

## Notes

- Only one filter range per worksheet
- Filter range is a "singleton" resource - LWW is appropriate
- Structural mutations always shift the filter range

## Last Updated

**Date**: 2026-01-31
**By**: Claude Code
**Changes**: Initial document creation
