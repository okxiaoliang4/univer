# MoveRowsMutation Conflict Analysis

**Mutation ID**: `sheet.mutation.move-rows`
**File**: `mutations/sheets/move_rows_mutation.rs`
**Transform**: `transforms/sheets/move_rows.rs`
**Tests**: `tests/move_operations_tests.rs`

## Mutation Overview

**Purpose**: Moves one or more rows from a source position to a target position.

**Affects**:
- [x] Rows (primary - moves row positions)
- [ ] Columns
- [x] Cells/Ranges (indirect - cells move with rows)
- [ ] Worksheet properties
- [ ] Workbook properties
- [ ] Feature-specific

**Parameters**:
```rust
pub struct MoveRowsMutationParams {
    pub unit_id: String,           // Workbook ID
    pub sub_unit_id: String,       // Worksheet ID
    pub source_range: IRange,      // Source row range
    pub target_range: IRange,      // Target row range
}
```

**Operation Logic**:
- Rows in `source_range` are moved to `target_range`
- Intermediate rows shift to fill/make space
- Complex position transformations needed

## Conflict Dimensions Analysis

### Dimension 1: Unit ID (Workbook)
- **Conflicts when**: Same `unit_id`
- **No conflict when**: Different `unit_id` → Identity transform

### Dimension 2: Sub-Unit ID (Worksheet)
- **Conflicts when**: Same `unit_id` AND same `sub_unit_id`
- **No conflict when**: Different worksheets → Identity transform

### Dimension 3: Position (Row/Col/Range)
**Positions affected by this mutation**:
- Source rows: Rows being moved
- Target rows: Where rows are moving to
- Intermediate rows: Rows between source and target shift

**Conflict scenarios**:
1. **Source overlap**: Another operation affects rows being moved
2. **Target overlap**: Another operation affects destination
3. **Path overlap**: Another operation affects intermediate rows

### Dimension 4: Feature Plugin ID
**Feature-specific IDs**: N/A

## Transform Coverage Matrix

### Core Sheets Mutations

#### Structural Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| InsertRowMutation | ⏳ | Complex | Source/target positions shift |
| InsertColMutation | ❌ | Identity | Column insert doesn't affect row move |
| RemoveRowsMutation | ⏳ | Complex + Conflict | Source/target may be affected |
| RemoveColMutation | ❌ | Identity | Column removal doesn't affect row move |
| MoveRowsMutation (self) | ⏳ | Complex | Two moves may interact |
| MoveColumnsMutation | ❌ | Identity | Row vs column operations |
| MoveRangeMutation | ⏳ | Complex | May affect same rows |

#### Data Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| SetRangeValuesMutation | ⏳ | Complex | Cells in moved rows affected |
| SetRowDataMutation | ⏳ | Complex | Row data for moved rows affected |
| SetColDataMutation | ❌ | Identity | Column data not affected |
| ReorderRangeMutation | ⏳ | Complex | May affect same rows |

#### Range-based Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| AddWorksheetMergeMutation | ⏳ | Complex | Merge ranges may move |
| SetSheetsFilterRangeMutation | ⏳ | Complex | Filter range may be affected |
| AddConditionalRuleMutation | ⏳ | Complex | Rule ranges may move |
| AddDataValidationMutation | ⏳ | Complex | Rule ranges may move |
| UpdateNoteMutation | ⏳ | Complex | Note position may move |

## Detailed Conflict Resolution

### InsertRowMutation vs MoveRowsMutation

**Conflict Analysis**:
- **Dimension 3 (Position)**: Both source_range and target_range may need shifting

**Resolution Strategy**: Complex Shifting

**Key Cases**:
1. Insert before source → source shifts down
2. Insert before target → target shifts down
3. Insert between source and target → depends on move direction

**Implementation** (Pseudocode):
```rust
fn create_insert_row_vs_move_rows() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return identity(m1, m2);
        }

        let m1_params: InsertRowMutationParams = parse(m1)?;
        let mut m2_params: MoveRowsMutationParams = parse(m2)?;

        let insert_start = m1_params.range.start_row;
        let insert_count = m1_params.range.end_row - insert_start + 1;

        // Shift source range
        shift_range_rows_for_insert(&mut m2_params.source_range, insert_start, insert_count);

        // Shift target range
        shift_range_rows_for_insert(&mut m2_params.target_range, insert_start, insert_count);

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

### MoveRowsMutation vs MoveRowsMutation (Self-Transform)

**Conflict Analysis**:
- Two concurrent row moves are extremely complex
- Source/target ranges may interact

**Resolution Strategy**: First-move-wins or complex merge

**Key Cases**:
1. Non-overlapping moves → Both apply with position adjustments
2. Overlapping sources → Conflict, one wins
3. Same target → Conflict, last wins

**Status**: ⏳ Planned (Complex)

---

### MoveRowsMutation vs SetRangeValuesMutation

**Conflict Analysis**:
- Cells in moved rows need their positions transformed

**Resolution Strategy**: Cell Position Transformation

**Key Cases**:
1. Cells in source range → Cells move to target
2. Cells in intermediate range → Cells shift
3. Cells outside → No change

**Status**: ⏳ Planned

## Implementation Complexity

MoveRowsMutation is one of the most complex transforms because:

1. **Dual Range**: Both source and target need consideration
2. **Direction Matters**: Moving down vs up has different effects
3. **Intermediate Rows**: Rows between source and target shift
4. **Cell Transformation**: Need to transform cell positions, not just shift
5. **Feature Interactions**: All range-based features need transformation

## Recommended Implementation Approach

1. **Create transformation function** for row position:
```rust
fn transform_row_for_move(
    row: i32,
    source_start: i32,
    source_end: i32,
    target_start: i32,
) -> Option<i32> {
    let source_count = source_end - source_start + 1;

    if row >= source_start && row <= source_end {
        // Row is being moved
        let offset = row - source_start;
        Some(target_start + offset)
    } else if target_start < source_start {
        // Moving up
        if row >= target_start && row < source_start {
            Some(row + source_count)
        } else {
            Some(row)
        }
    } else {
        // Moving down
        if row > source_end && row <= target_start {
            Some(row - source_count)
        } else {
            Some(row)
        }
    }
}
```

2. **Apply to HashMap keys** (cell_value, row_data)
3. **Apply to IRange** (ranges in other mutations)
4. **Apply to single positions** (row, col in notes/comments)

## Implementation Checklist

- [ ] Self-transform implemented
- [ ] Transform with InsertRowMutation implemented
- [ ] Transform with RemoveRowsMutation implemented
- [ ] Transform with SetRangeValuesMutation implemented
- [ ] Transform with SetRowDataMutation implemented
- [ ] Transform with AddWorksheetMergeMutation implemented
- [ ] Transform with all range-based feature mutations
- [ ] Tests written for all scenarios
- [ ] This document updated

## Notes

- MoveRowsMutation is high complexity due to dual-range nature
- Consider implementing after simpler mutations
- Comprehensive test coverage essential
- May benefit from shared transformation utilities

## Last Updated

**Date**: 2026-01-31
**By**: Claude Code
**Changes**: Initial document creation with implementation guidance
