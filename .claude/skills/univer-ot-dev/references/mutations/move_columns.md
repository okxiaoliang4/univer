# MoveColumnsMutation Conflict Analysis

**Mutation ID**: `sheet.mutation.move-columns`
**File**: `mutations/sheets/move_columns_mutation.rs`
**Transform**: `transforms/sheets/move_columns.rs`
**Tests**: `tests/move_operations_tests.rs`

## Mutation Overview

**Purpose**: Moves one or more columns from a source position to a target position.

**Affects**:
- [ ] Rows
- [x] Columns (primary - moves column positions)
- [x] Cells/Ranges (indirect - cells move with columns)
- [ ] Worksheet properties
- [ ] Workbook properties
- [ ] Feature-specific

**Parameters**:
```rust
pub struct MoveColumnsMutationParams {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub source_range: IRange,
    pub target_range: IRange,
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
- Source columns: Columns being moved
- Target columns: Where columns are moving to
- Intermediate columns: Columns between source and target shift

### Dimension 4: Feature Plugin ID
**Feature-specific IDs**: N/A

## Transform Coverage Matrix

### Core Sheets Mutations

#### Structural Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| InsertRowMutation | ❌ | Identity | Row insert doesn't affect column move |
| InsertColMutation | ⏳ | Complex | Source/target positions shift |
| RemoveRowsMutation | ❌ | Identity | Row removal doesn't affect column move |
| RemoveColMutation | ⏳ | Complex + Conflict | Source/target may be affected |
| MoveRowsMutation | ❌ | Identity | Row vs column operations |
| MoveColumnsMutation (self) | ⏳ | Complex | Two moves may interact |
| MoveRangeMutation | ⏳ | Complex | May affect same columns |

#### Data Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| SetRangeValuesMutation | ⏳ | Complex | Cells in moved columns affected |
| SetRowDataMutation | ❌ | Identity | Row data not affected |
| SetColDataMutation | ⏳ | Complex | Column data for moved columns affected |

## Implementation Checklist

- [ ] Self-transform implemented
- [ ] Transforms with structural mutations implemented
- [ ] Tests written
- [ ] This document updated

## Last Updated

**Date**: 2026-01-31
**By**: Claude Code
**Changes**: Initial document creation
