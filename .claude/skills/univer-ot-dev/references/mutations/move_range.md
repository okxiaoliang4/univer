# MoveRangeMutation Conflict Analysis

**Mutation ID**: `sheet.mutation.move-range`
**File**: `mutations/sheets/move_range_mutation.rs`
**Transform**: `transforms/sheets/move_range.rs`
**Tests**: `tests/move_operations_tests.rs`

## Mutation Overview

**Purpose**: Moves a 2D cell range from source to target position.

**Affects**:
- [x] Rows (cells move between rows)
- [x] Columns (cells move between columns)
- [x] Cells/Ranges (primary)
- [ ] Worksheet properties
- [ ] Workbook properties
- [ ] Feature-specific

**Parameters**:
```rust
pub struct MoveRangeMutationParams {
    pub unit_id: String,
    pub from_range: IRange,
    pub to_range: IRange,
    pub from: MoveRangeFromTo,
    pub to: MoveRangeFromTo,
}

pub struct MoveRangeFromTo {
    pub value: IObjectMatrixPrimitiveType,
    pub row_properties: Option<HashMap<String, IRowData>>,
    pub column_properties: Option<HashMap<String, IColumnData>>,
}
```

## Conflict Dimensions Analysis

### Dimension 1: Unit ID (Workbook)
- **Conflicts when**: Same `unit_id`
- **No conflict when**: Different `unit_id` → Identity transform

### Dimension 2: Sub-Unit ID (Worksheet)
- **Conflicts when**: Same `unit_id` AND same sub_unit_id in ranges
- **No conflict when**: Different worksheets → Identity transform

### Dimension 3: Position (Row/Col/Range)
**Positions affected by this mutation**:
- Source range: from_range
- Target range: to_range
- Both row and column dimensions

## Transform Coverage Matrix

### Core Sheets Mutations

#### Structural Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| InsertRowMutation | ⏳ | Complex | Both ranges may shift |
| InsertColMutation | ⏳ | Complex | Both ranges may shift |
| RemoveRowsMutation | ⏳ | Complex | Ranges may be affected |
| RemoveColMutation | ⏳ | Complex | Ranges may be affected |
| MoveRowsMutation | ⏳ | Complex | May interact |
| MoveColumnsMutation | ⏳ | Complex | May interact |
| MoveRangeMutation (self) | ⏳ | Complex | Two range moves |

## Implementation Checklist

- [ ] Self-transform implemented
- [ ] Transforms with structural mutations implemented
- [ ] Tests written
- [ ] This document updated

## Last Updated

**Date**: 2026-01-31
**By**: Claude Code
**Changes**: Initial document creation
