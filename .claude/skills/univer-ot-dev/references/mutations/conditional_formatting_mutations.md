# Conditional Formatting Mutations - Batch Analysis

This document covers conditional formatting mutations (beyond AddConditionalRuleMutation which has its own doc).

## Covered Mutations

### 1. AddConditionalRuleMutation
**Status**: ✅ Full document exists ([add_conditional_rule.md](add_conditional_rule.md))

### 2. DeleteConditionalRuleMutation
**ID**: `sheet.mutation.delete-conditional-rule`
**Purpose**: Removes conditional formatting rule by ID
**Strategy**: ID-based (mostly identity)
**Parameters**: `cf_id`

### 3. SetConditionalRuleMutation
**ID**: `sheet.mutation.set-conditional-rule`
**Purpose**: Updates conditional formatting rule
**Strategy**: ID-based + Range shifting + LWW
**Parameters**: `cf_id, rule` (with ranges)

### 4. MoveConditionalRuleMutation
**ID**: `sheet.mutation.move-conditional-rule`
**Purpose**: Changes rule priority order
**Strategy**: Priority-based (mostly identity with position ops)
**Parameters**: `start, end` (priority indices)

### 5. ConditionalFormattingFormulaMarkDirty
**ID**: `sheet.mutation.conditional-formatting-formula-mark-dirty`
**Purpose**: Marks formulas as needing recalculation
**Strategy**: Trigger mutation (mostly identity)
**Parameters**: Complex nested data structure

## Conflict Dimensions Analysis

### Dimension 1: Unit ID (Workbook)
- **Conflicts when**: Same `unit_id`
- **No conflict when**: Different `unit_id` → Identity transform

### Dimension 2: Sub-Unit ID (Worksheet)
- **Conflicts when**: Same `unit_id` AND same `sub_unit_id`
- **No conflict when**: Different worksheets → Identity transform

### Dimension 3: Position (Row/Col/Range)
**Positions affected**:
- Add/Set: rule.ranges (Vec<IRange>)
- Delete/Move: No position in params
- MarkDirty: Complex position data

### Dimension 4: Feature Plugin ID
**Feature-specific IDs**: `cf_id` (conditional formatting rule ID)
- Different IDs → Independent
- Same ID → LWW or conflict

## Transform Coverage Matrix

### Range-Based Mutations (Add, Set)

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| InsertRowMutation | ✅ | Shifting | Rule ranges shift rows |
| InsertColMutation | ✅ | Shifting | Rule ranges shift columns |
| RemoveRowsMutation | ✅ | Shifting + Removal | Rules may be removed |
| RemoveColMutation | ✅ | Shifting + Removal | Rules may be removed |
| (covered in add_conditional_rule.md) | ✅ | - | See main document |

### ID-Based Mutations (Delete)

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| All structural mutations | ❌ | Identity | No position in params |

### Priority-Based (Move)

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| All structural mutations | ❌ | Identity | Priority order, not position |
| All data mutations | ❌ | Identity | Priority order, not data |

### Self-Transforms

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| DeleteConditionalRuleMutation (self) | ✅ | Idempotent | Same cf_id: both delete |
| SetConditionalRuleMutation (self) | ✅ | LWW | Same cf_id: m2 wins |
| MoveConditionalRuleMutation (self) | ⏳ | Priority adjustment | Complex priority logic |

### Cross-CF Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| AddConditionalRuleMutation vs DeleteConditionalRuleMutation | ⏳ | Conflict | Add vs Delete same cf_id |
| AddConditionalRuleMutation vs SetConditionalRuleMutation | ⏳ | Conflict | Add vs Set same cf_id: Set wins |
| DeleteConditionalRuleMutation vs MoveConditionalRuleMutation | ⏳ | Conflict | Delete removes from priority list |

## Implementation Pattern

### ID-Based (Delete)
```rust
// Mostly identity with structural mutations
fn create_insert_row_vs_delete_cf() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        identity(m1, m2)  // No position in Delete params
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

        // Different cf_ids: independent
        if m1_params.cf_id != m2_params.cf_id {
            return identity(m1, m2);
        }

        // Same cf_id: both delete (idempotent)
        TransformResultRef {
            m1_prime: MutationOutcome::Removed,
            m2_prime: MutationOutcome::Unchanged(m2),
            error: None,
        }
    })
}
```

### Range-Based (Set)
```rust
// Similar to AddConditionalRuleMutation
fn create_insert_row_vs_set_cf() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return identity(m1, m2);
        }

        let m1_params: InsertRowMutationParams = parse(m1)?;
        let mut m2_params: SetConditionalRuleMutationParams = parse(m2)?;

        let insert_start = m1_params.range.start_row;
        let insert_count = m1_params.range.end_row - insert_start + 1;

        // Shift all ranges in the rule
        for range in &mut m2_params.rule.ranges {
            shift_range_rows_for_insert(range, insert_start, insert_count);
        }

        TransformResultRef {
            m1_prime: MutationOutcome::Unchanged(m1),
            m2_prime: MutationOutcome::Modified(serialize(m2_params)),
            error: None,
        }
    })
}
```

### Priority-Based (Move)
```rust
// Identity with most mutations
fn create_self_transform() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return identity(m1, m2);
        }

        // Complex priority adjustment logic
        // Similar to reordering an array
        // ... priority calculation ...

        TransformResultRef {
            m1_prime: MutationOutcome::Modified(/* adjusted */),
            m2_prime: MutationOutcome::Unchanged(m2),
            error: None,
        }
    })
}
```

## Implementation Checklist

- [x] AddConditionalRuleMutation fully documented
- [ ] DeleteConditionalRuleMutation transforms
- [ ] SetConditionalRuleMutation transforms
- [ ] MoveConditionalRuleMutation transforms
- [ ] Cross-CF conflict resolution
- [ ] Tests written
- [x] This document updated

## Last Updated

**Date**: 2026-01-31
**By**: Claude Code
**Changes**: Batch document for conditional formatting mutations
