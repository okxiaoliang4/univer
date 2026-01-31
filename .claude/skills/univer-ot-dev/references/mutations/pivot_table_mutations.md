# Pivot Table Mutations - Batch Analysis

This document covers pivot table mutations.

## Covered Mutations

### 1. AddPivotTableMutation
**ID**: `sheet.mutation.add-pivot-table`
**Purpose**: Creates new pivot table
**Strategy**: Range-based + ID-based + Shifting
**Parameters**: `pivot_table_id, config` (with source_range, target_cell)

### 2. RemovePivotTableMutation
**ID**: `sheet.mutation.remove-pivot-table`
**Purpose**: Removes pivot table by ID
**Strategy**: ID-based (mostly identity)
**Parameters**: `pivot_table_id`

### 3. SetPivotTableSourceRangeMutation
**ID**: `sheet.mutation.set-pivot-table-source-range`
**Purpose**: Updates source data range
**Strategy**: Range-based + Shifting
**Parameters**: `pivot_table_id, source_range_info`

### 4. SetPivotTableTargetCellMutation
**ID**: `sheet.mutation.set-pivot-table-target-cell`
**Purpose**: Updates pivot output location
**Strategy**: Position-based + Shifting
**Parameters**: `pivot_table_id, target_cell_info`

### 5. SetPivotTableFieldsConfigMutation
**ID**: `sheet.mutation.set-pivot-table-fields-config`
**Purpose**: Updates pivot field configuration
**Strategy**: ID-based + LWW
**Parameters**: `pivot_table_id, fields_config`

### 6. SetPivotTableCalculatedDataMutation
**ID**: `sheet.mutation.set-pivot-table-calculated-data`
**Purpose**: Sets pivot calculation results
**Strategy**: ID-based + LWW
**Parameters**: `pivot_table_id, pivot_model`

## Conflict Dimensions Analysis

### Dimension 1: Unit ID (Workbook)
- **Conflicts when**: Same `unit_id`
- **No conflict when**: Different `unit_id` → Identity transform

### Dimension 2: Sub-Unit ID (Worksheet)
- **Conflicts when**: Same `unit_id` AND same `sub_unit_id`
- **No conflict when**: Different worksheets → Identity transform

### Dimension 3: Position (Row/Col/Range)
**Positions affected**:
- Add: source_range, target_cell
- SetSourceRange: source_range
- SetTargetCell: target_cell

### Dimension 4: Feature Plugin ID
**Feature-specific IDs**: `pivot_table_id`
- Different IDs → Independent
- Same ID → LWW or conflict

## Transform Coverage Matrix

### Range/Position-Based Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| InsertRowMutation | ⏳ | Shifting | Source range and target cell shift |
| InsertColMutation | ⏳ | Shifting | Source range and target cell shift |
| RemoveRowsMutation | ⏳ | Shifting + Removal | May invalidate pivot |
| RemoveColMutation | ⏳ | Shifting + Removal | May invalidate pivot |
| MoveRowsMutation | ⏳ | Complex | Ranges may move |
| MoveColumnsMutation | ⏳ | Complex | Ranges may move |
| MoveRangeMutation | ⏳ | Complex | May affect pivot ranges |

### ID-Based Mutations (Config, Data, Remove)

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| All structural mutations | ❌ | Identity | No position in params |

### Self-Transforms

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| AddPivotTableMutation (self) | ✅ | Identity (diff ID) / LWW (same ID) | By pivot_table_id |
| RemovePivotTableMutation (self) | ✅ | Idempotent | Same ID: both remove |
| SetPivotTableSourceRangeMutation (self) | ✅ | LWW | Same ID: m2 wins |
| SetPivotTableTargetCellMutation (self) | ✅ | LWW | Same ID: m2 wins |
| SetPivotTableFieldsConfigMutation (self) | ✅ | LWW | Same ID: m2 wins |
| SetPivotTableCalculatedDataMutation (self) | ✅ | LWW | Same ID: m2 wins |

## Implementation Pattern

### Range/Position-Based
```rust
// Similar to AddConditionalRuleMutation
fn create_insert_row_vs_pivot() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return identity(m1, m2);
        }

        let m1_params: InsertRowMutationParams = parse(m1)?;
        let mut m2_params: AddPivotTableMutationParams = parse(m2)?;

        let insert_start = m1_params.range.start_row;
        let insert_count = m1_params.range.end_row - insert_start + 1;

        // Shift source range
        shift_range_rows_for_insert(
            &mut m2_params.config.source_range, insert_start, insert_count
        );

        // Shift target cell
        if m2_params.config.target_cell.row >= insert_start {
            m2_params.config.target_cell.row += insert_count;
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

        // Different pivot IDs: independent
        if m1_params.pivot_table_id != m2_params.pivot_table_id {
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

- [ ] Range/position-based transforms with structural mutations
- [ ] ID-based self-transforms (LWW)
- [ ] Add vs Remove conflict resolution
- [ ] Tests written
- [x] This document updated

## Last Updated

**Date**: 2026-01-31
**By**: Claude Code
**Changes**: Batch document for pivot table mutations
