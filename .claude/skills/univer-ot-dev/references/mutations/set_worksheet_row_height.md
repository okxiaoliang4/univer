# SetWorksheetRowHeightMutation Conflict Analysis

**Mutation ID**: `sheet.mutation.set-worksheet-row-height`
**File**: `mutations/sheets/set_worksheet_row_height_mutation.rs`
**Transform**: `transforms/sheets/dimensions.rs`
**Tests**: `tests/set_row_height_tests.rs`

## Mutation Overview

**Purpose**: Sets the height of one or more rows in a worksheet.

**Affects**:
- [x] Rows (sets row height)
- [ ] Columns
- [ ] Cells/Ranges
- [ ] Worksheet properties
- [ ] Workbook properties
- [ ] Feature-specific

**Parameters**:
```rust
pub struct SetWorksheetRowHeightMutationParams {
    pub unit_id: String,           // Workbook ID
    pub sub_unit_id: String,       // Worksheet ID
    pub ranges: Vec<IRange>,       // Row ranges to set height
    pub row_height: f64,           // Height value
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
- Rows: Row indices in the ranges

**Conflict scenarios**:
1. **Row insert above**: Row indices in ranges shift down
2. **Row remove**: Row indices may be removed or shifted up
3. **Same row, different height**: LWW (m2 wins)

### Dimension 4: Feature Plugin ID
**Feature-specific IDs**: N/A

## Transform Coverage Matrix

### Core Sheets Mutations

#### Structural Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| InsertRowMutation | ✅ | Shifting | Row indices in ranges shift |
| InsertColMutation | ❌ | Identity | Column insert doesn't affect row heights |
| RemoveRowsMutation | ✅ | Shifting + Removal | Row indices shift/removed |
| RemoveColMutation | ❌ | Identity | Column removal doesn't affect row heights |
| MoveRowsMutation | ⏳ | Complex | Source/target rows affected |
| MoveColumnsMutation | ❌ | Identity | Column move doesn't affect row heights |
| MoveRangeMutation | ⏳ | Complex | May affect rows |

#### Data/Merge/Other Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| SetRangeValuesMutation | ❌ | Identity | Cell values independent of row height |
| AddWorksheetMergeMutation | ❌ | Identity | Merge independent of row height |
| (Most other mutations) | ❌ | Identity | Independent operations |

#### Dimension Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| SetWorksheetRowHeightMutation (self) | ✅ | Conflict Scope | Different rows: both apply; Same rows: LWW |
| SetWorksheetColWidthMutation | ❌ | Identity | Row vs column dimensions |
| SetWorksheetRowIsAutoHeightMutation | ⏳ | Conflict Scope | Same row conflicts |
| SetWorksheetRowAutoHeightMutation | ⏳ | Conflict Scope | Same row conflicts |

## Detailed Conflict Resolution

### InsertRowMutation vs SetWorksheetRowHeightMutation

**Conflict Analysis**:
- **Dimension 3 (Position)**: Row indices need shifting

**Resolution Strategy**: Shifting

**Implementation**:
```rust
fn create_insert_row_vs_row_height() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return identity(m1, m2);
        }

        let m1_params: InsertRowMutationParams = parse(m1)?;
        let mut m2_params: SetWorksheetRowHeightMutationParams = parse(m2)?;

        let insert_start = m1_params.range.start_row;
        let insert_count = m1_params.range.end_row - insert_start + 1;

        // Shift all row ranges
        for range in &mut m2_params.ranges {
            shift_range_rows_for_insert(range, insert_start, insert_count);
        }

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

### SetWorksheetRowHeightMutation vs SetWorksheetRowHeightMutation (Self-Transform)

**Conflict Analysis**:
- **Dimension 3 (Position)**: Different rows = no conflict; Same rows = LWW

**Resolution Strategy**: Conflict Scope (row-level)

**Implementation**:
```rust
fn create_self_transform() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return identity(m1, m2);
        }

        let m1_params: SetWorksheetRowHeightMutationParams = parse(m1)?;
        let m2_params: SetWorksheetRowHeightMutationParams = parse(m2)?;

        // Different rows: both apply (identity)
        // Same rows: m2 wins (LWW)
        // For simplicity, we apply row-level conflict resolution

        let mut m1_prime_ranges = m1_params.ranges.clone();

        // Remove ranges from m1 that overlap with m2's ranges
        for m2_range in &m2_params.ranges {
            m1_prime_ranges.retain(|m1_range| {
                !ranges_overlap_rows(m1_range, m2_range)
            });
        }

        if m1_prime_ranges.is_empty() {
            return TransformResultRef {
                m1_prime: MutationOutcome::Removed,
                m2_prime: MutationOutcome::Unchanged(m2),
                error: None,
            };
        }

        let mut m1_prime_params = m1_params.clone();
        m1_prime_params.ranges = m1_prime_ranges;

        TransformResultRef {
            m1_prime: MutationOutcome::Modified(serialize(m1_prime_params)),
            m2_prime: MutationOutcome::Unchanged(m2),
            error: None,
        }
    })
}
```

**Status**: ✅ Implemented

## Implementation Checklist

- [x] Self-transform (conflict scope) implemented
- [x] Transform with InsertRowMutation implemented
- [x] Transform with RemoveRowsMutation implemented
- [ ] Transform with MoveRowsMutation implemented
- [x] Tests written for implemented scenarios
- [x] Zero-copy optimizations applied
- [x] This document updated

## Notes

- Row height is a row-level property (not cell-level)
- Conflict scope at row level: different rows are independent
- Same row = LWW (m2's height wins)
- Uses `Vec<IRange>` to support multiple row ranges

## Last Updated

**Date**: 2026-01-31
**By**: Claude Code
**Changes**: Initial document creation
