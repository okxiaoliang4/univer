# Data Validation Mutations - Batch Analysis

This document covers data validation mutations (beyond AddDataValidationMutation which has its own doc).

## Covered Mutations

### 1. AddDataValidationMutation
**Status**: ✅ Full document exists ([add_data_validation.md](add_data_validation.md))

### 2. RemoveDataValidationMutation
**ID**: `data-validation.mutation.removeRule`
**Purpose**: Removes validation rule by ID
**Strategy**: ID-based (mostly identity)
**Parameters**: `rule_id, source`

### 3. UpdateDataValidationMutation
**ID**: `data-validation.mutation.updateRule`
**Purpose**: Updates validation rule
**Strategy**: ID-based + Range shifting + LWW
**Parameters**: `rule_id, payload` (with ranges)

## Conflict Dimensions Analysis

### Dimension 1: Unit ID (Workbook)
- **Conflicts when**: Same `unit_id`
- **No conflict when**: Different `unit_id` → Identity transform

### Dimension 2: Sub-Unit ID (Worksheet)
- **Conflicts when**: Same `unit_id` AND same `sub_unit_id`
- **No conflict when**: Different worksheets → Identity transform

### Dimension 3: Position (Row/Col/Range)
**Positions affected**:
- Add/Update: rule.ranges (Vec<IRange>)
- Remove: No position in params

### Dimension 4: Feature Plugin ID
**Feature-specific IDs**: `uid` (data validation rule ID)
- Different IDs → Independent
- Same ID → LWW or conflict

## Transform Coverage Matrix

### Range-Based Mutations (Add, Update)

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| InsertRowMutation | ✅ | Shifting | Rule ranges shift rows |
| InsertColMutation | ✅ | Shifting | Rule ranges shift columns |
| RemoveRowsMutation | ✅ | Shifting + Removal | Rules may be removed |
| RemoveColMutation | ✅ | Shifting + Removal | Rules may be removed |
| (covered in add_data_validation.md) | ✅ | - | See main document |

### ID-Based Mutations (Remove)

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| All structural mutations | ❌ | Identity | No position in params |

### Self-Transforms

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| RemoveDataValidationMutation (self) | ✅ | Idempotent | Same uid: both remove |
| UpdateDataValidationMutation (self) | ✅ | LWW | Same uid: m2 wins |

### Cross-DV Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| AddDataValidationMutation vs RemoveDataValidationMutation | ⏳ | Conflict | Add vs Remove same uid |
| AddDataValidationMutation vs UpdateDataValidationMutation | ⏳ | Conflict | Add vs Update same uid: Update wins |

## Implementation Pattern

### ID-Based (Remove)
```rust
// Mostly identity with structural mutations
fn create_insert_row_vs_remove_dv() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        identity(m1, m2)  // No position in Remove params
    })
}

// Self-transform: idempotent
fn create_self_transform() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return identity(m1, m2);
        }

        let m1_params = parse(m1)?;
        let m2_params = parse(m2)?;

        // Different uids: independent
        if m1_params.rule_id != m2_params.rule_id {
            return identity(m1, m2);
        }

        // Same uid: both remove (idempotent)
        TransformResultRef {
            m1_prime: MutationOutcome::Removed,
            m2_prime: MutationOutcome::Unchanged(m2),
            error: None,
        }
    })
}
```

### Range-Based (Update)
```rust
// Similar to AddDataValidationMutation
fn create_insert_row_vs_update_dv() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return identity(m1, m2);
        }

        let m1_params: InsertRowMutationParams = parse(m1)?;
        let mut m2_params: UpdateDataValidationMutationParams = parse(m2)?;

        let insert_start = m1_params.range.start_row;
        let insert_count = m1_params.range.end_row - insert_start + 1;

        // Shift all ranges in the payload
        if let Some(ref mut ranges) = m2_params.payload.ranges {
            for range in ranges {
                shift_range_rows_for_insert(range, insert_start, insert_count);
            }
        }

        TransformResultRef {
            m1_prime: MutationOutcome::Unchanged(m1),
            m2_prime: MutationOutcome::Modified(serialize(m2_params)),
            error: None,
        }
    })
}

// Self-transform: LWW
fn create_self_transform() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return identity(m1, m2);
        }

        let m1_params = parse(m1)?;
        let m2_params = parse(m2)?;

        // Different uids: independent
        if m1_params.rule_id != m2_params.rule_id {
            return identity(m1, m2);
        }

        // Same uid: m2 wins (LWW)
        TransformResultRef {
            m1_prime: MutationOutcome::Removed,
            m2_prime: MutationOutcome::Unchanged(m2),
            error: None,
        }
    })
}
```

## Key Insights

1. **ID-based system**: Rules identified by `uid` not position
2. **Range-based application**: Rules apply to Vec<IRange>
3. **Independent rules**: Different uids = independent operations
4. **LWW for updates**: Same uid = last write wins

## Implementation Checklist

- [x] AddDataValidationMutation fully documented
- [ ] RemoveDataValidationMutation transforms
- [ ] UpdateDataValidationMutation transforms
- [ ] Cross-DV conflict resolution
- [ ] Tests written
- [x] This document updated

## Last Updated

**Date**: 2026-01-31
**By**: Claude Code
**Changes**: Batch document for data validation mutations
