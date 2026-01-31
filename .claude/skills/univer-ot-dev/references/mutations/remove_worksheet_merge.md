# RemoveWorksheetMergeMutation Conflict Analysis

**Mutation ID**: `sheet.mutation.remove-worksheet-merge`
**File**: `mutations/sheets/remove_worksheet_merge_mutation.rs`
**Transform**: `transforms/sheets/remove_merge.rs`
**Tests**: `tests/merge_tests.rs`

## Mutation Overview

**Purpose**: Unmerges previously merged cell ranges.

**Affects**:
- [ ] Rows
- [ ] Columns
- [x] Cells/Ranges (unmerges cell ranges)
- [ ] Worksheet properties
- [ ] Workbook properties
- [ ] Feature-specific

**Parameters**:
```rust
pub struct RemoveWorksheetMergeMutationParams {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub ranges: Vec<IRange>,
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
- Ranges: Unmerge ranges

**Conflict scenarios**:
1. **Row/column insert/remove**: Range positions shift
2. **Add vs Remove same range**: Conflict
3. **Different ranges**: Independent

## Transform Coverage Matrix

### Core Sheets Mutations

#### Structural Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| InsertRowMutation | ✅ | Shifting | Unmerge ranges shift with rows |
| InsertColMutation | ✅ | Shifting | Unmerge ranges shift with columns |
| RemoveRowsMutation | ✅ | Shifting + Removal | Ranges shrink/removed |
| RemoveColMutation | ✅ | Shifting + Removal | Ranges shrink/removed |

#### Merge Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| AddWorksheetMergeMutation | ⏳ | Conflict | Add vs Remove same range |
| RemoveWorksheetMergeMutation (self) | ✅ | Identity | Both unmerge (idempotent) |

## Implementation Checklist

- [x] Self-transform (identity) implemented
- [x] Transforms with structural mutations implemented
- [ ] Transform with AddWorksheetMergeMutation
- [x] Tests written for implemented scenarios
- [x] This document updated

## Last Updated

**Date**: 2026-01-31
**By**: Claude Code
**Changes**: Initial document creation
