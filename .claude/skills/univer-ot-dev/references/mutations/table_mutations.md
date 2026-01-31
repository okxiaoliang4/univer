# Table Mutations - Batch Analysis

This document covers sheet table mutations (structured tables/lists).

## Covered Mutations

### 1. AddSheetTableMutation
**ID**: `sheet.mutation.add-table`
**Purpose**: Creates new table
**Strategy**: Range-based + ID-based + Shifting
**Parameters**: `table_id, name, range, header, options`

### 2. DeleteSheetTableMutation
**ID**: `sheet.mutation.delete-table`
**Purpose**: Removes table by ID
**Strategy**: ID-based (mostly identity)
**Parameters**: `table_id`

### 3. SetSheetTableMutation
**ID**: `sheet.mutation.set-sheet-table`
**Purpose**: Updates table configuration
**Strategy**: ID-based + Range shifting + LWW
**Parameters**: `table_id, config` (may include range)

### 4. SetSheetTableFilterMutation
**ID**: `sheet.mutation.set-table-filter`
**Purpose**: Sets filter for table column
**Strategy**: Column-based + Shifting
**Parameters**: `table_id, column, table_filter`

## Conflict Dimensions Analysis

### Dimension 1: Unit ID (Workbook)
- **Conflicts when**: Same `unit_id`
- **No conflict when**: Different `unit_id` → Identity transform

### Dimension 2: Sub-Unit ID (Worksheet)
- **Conflicts when**: Same `unit_id` AND same `sub_unit_id`
- **No conflict when**: Different worksheets → Identity transform

### Dimension 3: Position (Row/Col/Range)
**Positions affected**:
- Add: range (table area)
- SetTable: may have range in config
- SetTableFilter: column index

### Dimension 4: Feature Plugin ID
**Feature-specific IDs**: `table_id`
- Different IDs → Independent
- Same ID → LWW or conflict

## Transform Coverage Matrix

### Range/Position-Based Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| InsertRowMutation | ⏳ | Shifting | Table range rows shift |
| InsertColMutation | ⏳ | Shifting | Table range columns shift; filter column shifts |
| RemoveRowsMutation | ⏳ | Shifting + Removal | Table may be invalidated |
| RemoveColMutation | ⏳ | Shifting + Removal | Table may be invalidated; filter column shifts |
| MoveRowsMutation | ⏳ | Complex | Table range may move |
| MoveColumnsMutation | ⏳ | Complex | Table range may move |
| MoveRangeMutation | ⏳ | Complex | May affect table range |

### ID-Based Mutations (Delete, SetTable)

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| All structural mutations (when no range in params) | ❌ | Identity | No position |
| SetSheetTableMutation with range | ⏳ | Shifting | If config has range |

### Self-Transforms

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| AddSheetTableMutation (self) | ✅ | Identity (diff ID) / LWW (same ID) | By table_id |
| DeleteSheetTableMutation (self) | ✅ | Idempotent | Same ID: both delete |
| SetSheetTableMutation (self) | ✅ | LWW | Same ID: m2 wins |
| SetSheetTableFilterMutation (self) | ✅ | Conflict Scope | Diff columns: independent; Same: LWW |

## Implementation Pattern

### Range-Based (Add, SetTable)
```rust
// Similar to AddConditionalRuleMutation
fn create_insert_row_vs_table() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return identity(m1, m2);
        }

        let m1_params: InsertRowMutationParams = parse(m1)?;
        let mut m2_params: AddSheetTableMutationParams = parse(m2)?;

        let insert_start = m1_params.range.start_row;
        let insert_count = m1_params.range.end_row - insert_start + 1;

        // Shift table range
        shift_range_rows_for_insert(&mut m2_params.range, insert_start, insert_count);

        TransformResultRef {
            m1_prime: MutationOutcome::Unchanged(m1),
            m2_prime: MutationOutcome::Modified(serialize(m2_params)),
            error: None,
        }
    })
}
```

### Column-Based Filter
```rust
fn create_insert_col_vs_table_filter() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return identity(m1, m2);
        }

        let m1_params: InsertColMutationParams = parse(m1)?;
        let mut m2_params: SetSheetTableFilterMutationParams = parse(m2)?;

        let insert_start = m1_params.range.start_column;
        let insert_count = m1_params.range.end_column - insert_start + 1;

        // Shift filter column index
        if m2_params.column >= insert_start {
            m2_params.column += insert_count;
        }

        TransformResultRef {
            m1_prime: MutationOutcome::Unchanged(m1),
            m2_prime: MutationOutcome::Modified(serialize(m2_params)),
            error: None,
        }
    })
}
```

### ID-Based (LWW)
```rust
fn create_self_transform() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return identity(m1, m2);
        }

        let m1_params = parse(m1)?;
        let m2_params = parse(m2)?;

        // Different table IDs: independent
        if m1_params.table_id != m2_params.table_id {
            return identity(m1, m2);
        }

        // Same ID: m2 wins (LWW)
        TransformResultRef {
            m1_prime: MutationOutcome::Removed,
            m2_prime: MutationOutcome::Unchanged(m2),
            error: None,
        }
    })
}
```

## Implementation Checklist

- [ ] Range-based transforms with structural mutations
- [ ] Column-based filter transforms
- [ ] ID-based self-transforms (LWW)
- [ ] Add vs Delete conflict resolution
- [ ] Tests written
- [x] This document updated

## Last Updated

**Date**: 2026-01-31
**By**: Claude Code
**Changes**: Batch document for table mutations
