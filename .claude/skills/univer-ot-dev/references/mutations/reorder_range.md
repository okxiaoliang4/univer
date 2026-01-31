# ReorderRangeMutation Conflict Analysis

**Mutation ID**: `sheet.mutation.reorder-range`
**File**: `mutations/sheets/reorder_range_mutation.rs`
**Transform**: `transforms/sheets/reorder.rs`
**Tests**: `tests/reorder_tests.rs`

## Mutation Overview

**Purpose**: Reorders cells within a range (e.g., for sorting).

**Affects**:
- [ ] Rows
- [ ] Columns
- [x] Cells/Ranges (cell positions change within range)
- [ ] Worksheet properties
- [ ] Workbook properties
- [ ] Feature-specific

**Parameters**:
```rust
pub struct ReorderRangeMutationParams {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub range: IRange,
    pub order: Vec<i32>,  // New order of cells
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
- Range: Cells within the range are reordered

**Conflict scenarios**:
1. **Row/column insert/remove**: Range positions shift
2. **Overlapping reorders**: Complex conflict
3. **Non-overlapping reorders**: Independent

## Transform Coverage Matrix

### Core Sheets Mutations

#### Structural Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| InsertRowMutation | ⏳ | Shifting | Range rows shift |
| InsertColMutation | ⏳ | Shifting | Range columns shift |
| RemoveRowsMutation | ⏳ | Shifting + Removal | Range may be affected |
| RemoveColMutation | ⏳ | Shifting + Removal | Range may be affected |
| MoveRowsMutation | ⏳ | Complex | Range may move |
| MoveColumnsMutation | ⏳ | Complex | Range may move |
| MoveRangeMutation | ⏳ | Complex | May affect same range |

#### Data Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| SetRangeValuesMutation | ⏳ | Complex | Cells being reordered may be modified |
| ReorderRangeMutation (self) | ⏳ | Complex | Overlapping reorders |

## Implementation Checklist

- [ ] Self-transform implemented
- [ ] Transforms with structural mutations implemented
- [ ] Tests written
- [ ] This document updated

## Last Updated

**Date**: 2026-01-31
**By**: Claude Code
**Changes**: Initial document creation
