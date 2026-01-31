# SetColDataMutation Conflict Analysis

**Mutation ID**: `sheet.mutation.set-col-data`
**File**: `mutations/sheets/set_col_data_mutation.rs`
**Transform**: `transforms/sheets/col_data.rs`
**Tests**: `tests/set_col_data_tests.rs`

## Mutation Overview

**Purpose**: Sets column-level metadata (width, visibility, custom data).

**Affects**:
- [ ] Rows
- [x] Columns (column metadata)
- [ ] Cells/Ranges
- [ ] Worksheet properties
- [ ] Workbook properties
- [ ] Feature-specific

**Parameters**:
```rust
pub struct SetColDataMutationParams {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub column_data: IObjectArrayPrimitiveType<IColumnData>,
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
- Column indices: Keys in the column_data HashMap

**Conflict scenarios**:
1. **Column insert left**: Column keys shift right
2. **Column remove**: Column keys may be deleted or shifted left
3. **Same column**: LWW at column level

## Transform Coverage Matrix

### Core Sheets Mutations

#### Structural Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| InsertRowMutation | ❌ | Identity | Row insert doesn't affect column data |
| InsertColMutation | ✅ | Shifting | Column keys shift |
| RemoveRowsMutation | ❌ | Identity | Row removal doesn't affect column data |
| RemoveColMutation | ✅ | Shifting + Removal | Column keys removed/shifted |
| MoveRowsMutation | ❌ | Identity | Row move doesn't affect column data |
| MoveColumnsMutation | ⏳ | Complex | Column data may move |
| MoveRangeMutation | ⏳ | Complex | May affect columns |

#### Data Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| SetRangeValuesMutation | ❌ | Identity | Cell values independent of column metadata |
| SetRowDataMutation | ❌ | Identity | Row vs column metadata |
| SetColDataMutation (self) | ✅ | LWW (column-level) | Same columns: m2 wins |

## Implementation Checklist

- [x] Self-transform (LWW at column level) implemented
- [x] Transform with InsertColMutation implemented
- [x] Transform with RemoveColMutation implemented
- [ ] Transform with MoveColumnsMutation implemented
- [x] Tests written for implemented scenarios
- [x] This document updated

## Last Updated

**Date**: 2026-01-31
**By**: Claude Code
**Changes**: Initial document creation
