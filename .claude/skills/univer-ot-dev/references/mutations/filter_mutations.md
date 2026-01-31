# Filter Mutations - Batch Analysis

This document covers sheet filter mutations (beyond SetSheetsFilterRangeMutation which has its own doc).

## Covered Mutations

### 1. SetSheetsFilterRangeMutation
**Status**: ✅ Full document exists ([set_sheets_filter_range.md](set_sheets_filter_range.md))

### 2. SetSheetsFilterCriteriaMutation
**ID**: `sheet.mutation.set-sheets-filter-criteria`
**Purpose**: Sets filter criteria for a column
**Strategy**: Column-based + Shifting
**Parameters**: `col, criteria, re_calc`

### 3. RemoveSheetsFilterMutation
**ID**: `sheet.mutation.remove-sheets-filter`
**Purpose**: Removes entire filter from worksheet
**Strategy**: Worksheet singleton removal
**Parameters**: None (worksheet-level)

### 4. ReCalcSheetsFilterMutation
**ID**: `sheet.mutation.re-calc-sheets-filter`
**Purpose**: Triggers filter recalculation
**Strategy**: Trigger mutation (identity with all)
**Parameters**: None (worksheet-level trigger)

## Conflict Dimensions Analysis

### Dimension 1: Unit ID (Workbook)
- **Conflicts when**: Same `unit_id`
- **No conflict when**: Different `unit_id` → Identity transform

### Dimension 2: Sub-Unit ID (Worksheet)
- **Conflicts when**: Same `unit_id` AND same `sub_unit_id`
- **No conflict when**: Different worksheets → Identity transform
- **Key**: Only one filter per worksheet (singleton)

### Dimension 3: Position (Row/Col/Range)
**Positions affected**:
- SetFilterRange: range
- SetFilterCriteria: col (column index)
- Remove/ReCalc: No position

### Dimension 4: Feature Plugin ID
**Feature-specific IDs**: Filter is unique per worksheet (no ID needed)

## Transform Coverage Matrix

### Column-Based (SetFilterCriteria)

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| InsertRowMutation | ❌ | Identity | Row insert doesn't affect column criteria |
| InsertColMutation | ✅ | Shifting | col field shifts if >= insert position |
| RemoveRowsMutation | ❌ | Identity | Row removal doesn't affect column criteria |
| RemoveColMutation | ✅ | Shifting + Removal | col may be removed or shifted |
| MoveColumnsMutation | ⏳ | Complex | col may move |

### Worksheet-Level (Remove, ReCalc)

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| All structural mutations | ❌ | Identity | No position in params |
| All data mutations | ❌ | Identity | Independent operations |

### Self-Transforms

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| SetSheetsFilterRangeMutation (self) | ✅ | LWW | Singleton per worksheet (see main doc) |
| SetSheetsFilterCriteriaMutation (self) | ✅ | Conflict Scope | Diff columns: independent; Same: LWW |
| RemoveSheetsFilterMutation (self) | ✅ | Idempotent | Both remove filter |
| ReCalcSheetsFilterMutation (self) | ✅ | Idempotent | Both trigger recalc |

### Cross-Filter Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| SetFilterRange vs SetFilterCriteria | ❌ | Identity | Different aspects of filter |
| SetFilterRange vs RemoveFilter | ⏳ | Conflict | Set vs Remove: Remove wins |
| SetFilterCriteria vs RemoveFilter | ⏳ | Conflict | Set vs Remove: Remove wins |
| Any vs ReCalc | ❌ | Identity | ReCalc is just trigger |

## Implementation Pattern

### Column-Based (SetFilterCriteria)
```rust
fn create_insert_col_vs_filter_criteria() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return identity(m1, m2);
        }

        let m1_params: InsertColMutationParams = parse(m1)?;
        let mut m2_params: SetSheetsFilterCriteriaMutationParams = parse(m2)?;

        let insert_start = m1_params.range.start_column;
        let insert_count = m1_params.range.end_column - insert_start + 1;

        // Shift filter column
        if m2_params.col >= insert_start {
            m2_params.col += insert_count;
        }

        TransformResultRef {
            m1_prime: MutationOutcome::Unchanged(m1),
            m2_prime: MutationOutcome::Modified(serialize(m2_params)),
            error: None,
        }
    })
}

// Self-transform: conflict scope
fn create_self_transform() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return identity(m1, m2);
        }

        let m1_params = parse(m1)?;
        let m2_params = parse(m2)?;

        // Different columns: independent
        if m1_params.col != m2_params.col {
            return identity(m1, m2);
        }

        // Same column: m2 wins (LWW)
        TransformResultRef {
            m1_prime: MutationOutcome::Removed,
            m2_prime: MutationOutcome::Unchanged(m2),
            error: None,
        }
    })
}
```

### Worksheet-Level (Remove, ReCalc)
```rust
// All mutations → identity with structural ops
// Self-transform → idempotent

fn create_self_transform() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return identity(m1, m2);
        }

        // Same worksheet: both remove/recalc (idempotent)
        TransformResultRef {
            m1_prime: MutationOutcome::Removed,
            m2_prime: MutationOutcome::Unchanged(m2),
            error: None,
        }
    })
}
```

## Key Insights

1. **Worksheet singleton**: Only one filter per worksheet
2. **Column-based criteria**: Each column can have independent filter criteria
3. **Remove is destructive**: RemoveFilter removes entire filter, all criteria
4. **ReCalc is trigger**: No data, just recalculation signal

## Implementation Checklist

- [x] SetSheetsFilterRangeMutation fully documented
- [x] SetSheetsFilterCriteriaMutation pattern defined
- [x] RemoveSheetsFilterMutation pattern defined
- [x] ReCalcSheetsFilterMutation pattern defined
- [ ] Cross-filter conflict resolution
- [ ] Tests written
- [x] This document updated

## Last Updated

**Date**: 2026-01-31
**By**: Claude Code
**Changes**: Batch document for filter mutations
