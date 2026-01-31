# SetFrozenMutation Conflict Analysis

**Mutation ID**: `sheet.mutation.set-frozen`
**File**: `mutations/sheets/set_frozen_mutation.rs`
**Transform**: `transforms/sheets/frozen.rs`
**Tests**: `tests/frozen_tests.rs`

## Mutation Overview

**Purpose**: Sets the frozen rows and columns for a worksheet (freeze panes).

**Affects**:
- [x] Rows (frozen rows)
- [x] Columns (frozen columns)
- [ ] Cells/Ranges
- [x] Worksheet properties (freeze state)
- [ ] Workbook properties
- [ ] Feature-specific

**Parameters**:
```rust
pub struct SetFrozenMutationParams {
    pub unit_id: String,           // Workbook ID
    pub sub_unit_id: String,       // Worksheet ID
    pub start_row: i32,            // First frozen row (freeze starts after this)
    pub start_column: i32,         // First frozen column
    pub y_split: i32,              // Number of rows to freeze
    pub x_split: i32,              // Number of columns to freeze
    pub reset_scroll: Option<bool>, // Reset scroll position
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
- Row position: start_row (after which freeze begins)
- Column position: start_column (after which freeze begins)

**Conflict scenarios**:
1. **Row insert before freeze line**: Freeze line shifts down
2. **Row remove before freeze line**: Freeze line shifts up
3. **Column insert before freeze line**: Freeze line shifts right
4. **Column remove before freeze line**: Freeze line shifts left

### Dimension 4: Feature Plugin ID
**Feature-specific IDs**: N/A (one freeze state per worksheet)

## Transform Coverage Matrix

### Core Sheets Mutations

#### Structural Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| InsertRowMutation | ⏳ | Shifting | start_row may shift |
| InsertColMutation | ⏳ | Shifting | start_column may shift |
| RemoveRowsMutation | ⏳ | Shifting | start_row may shift or be invalid |
| RemoveColMutation | ⏳ | Shifting | start_column may shift or be invalid |
| MoveRowsMutation | ⏳ | Complex | Freeze line may be affected |
| MoveColumnsMutation | ⏳ | Complex | Freeze line may be affected |
| MoveRangeMutation | ❌ | Identity | Freeze is row/col based, not range |

#### Data/Merge/Other Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| SetRangeValuesMutation | ❌ | Identity | Cell values independent of freeze |
| AddWorksheetMergeMutation | ❌ | Identity | Merge independent of freeze |
| SetWorksheetRowHeightMutation | ❌ | Identity | Dimensions independent of freeze |
| SetWorksheetColWidthMutation | ❌ | Identity | Dimensions independent of freeze |
| (Most other mutations) | ❌ | Identity | Independent operations |

### Worksheet Configuration

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| SetFrozenMutation (self) | ✅ | LWW | m2 wins on same worksheet |

## Detailed Conflict Resolution

### InsertRowMutation vs SetFrozenMutation

**Conflict Analysis**:
- **Dimension 3 (Position)**: start_row may need shifting

**Resolution Strategy**: Shifting

**Implementation**:
```rust
fn create_insert_row_vs_frozen() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return identity(m1, m2);
        }

        let m1_params: InsertRowMutationParams = parse(m1)?;
        let mut m2_params: SetFrozenMutationParams = parse(m2)?;

        let insert_start = m1_params.range.start_row;
        let insert_count = m1_params.range.end_row - insert_start + 1;

        // Shift freeze line if at or below insert position
        if m2_params.start_row >= insert_start {
            m2_params.start_row += insert_count;
        }

        TransformResultRef {
            m1_prime: MutationOutcome::Unchanged(m1),
            m2_prime: MutationOutcome::Modified(serialize(m2_params)),
            error: None,
        }
    })
}
```

**Status**: ⏳ Planned

---

### SetFrozenMutation vs SetFrozenMutation (Self-Transform)

**Conflict Analysis**:
- Only one freeze state per worksheet
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
            m1_prime: MutationOutcome::Removed,
            m2_prime: MutationOutcome::Unchanged(m2),
            error: None,
        }
    })
}
```

**Status**: ✅ Implemented

## Implementation Checklist

- [x] Self-transform (LWW) implemented
- [ ] Transform with InsertRowMutation implemented
- [ ] Transform with InsertColMutation implemented
- [ ] Transform with RemoveRowsMutation implemented
- [ ] Transform with RemoveColMutation implemented
- [ ] Tests written for all scenarios
- [x] This document updated

## Notes

- Freeze is a singleton per worksheet (only one freeze state)
- start_row/start_column define where freeze begins
- y_split/x_split define how many rows/columns are frozen
- May need to validate that freeze position is still valid after row/col removal

## Last Updated

**Date**: 2026-01-31
**By**: Claude Code
**Changes**: Initial document creation
