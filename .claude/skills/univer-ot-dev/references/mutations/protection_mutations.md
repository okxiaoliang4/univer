# Protection Mutations - Batch Analysis

This document covers range and worksheet protection mutations.

## Range Protection Mutations

### 1. AddRangeProtectionMutation
**ID**: `sheet.mutation.add-range-protection`
**Purpose**: Adds protection rules to ranges
**Strategy**: Range-based + Shifting
**Parameters**: `rules: Vec<IRangeProtectionRule>` with ranges

### 2. DeleteRangeProtectionMutation
**ID**: `sheet.mutation.delete-range-protection`
**Purpose**: Removes protection rules by IDs
**Strategy**: ID-based (mostly identity with position ops)
**Parameters**: `rule_ids: Vec<String>`

### 3. SetRangeProtectionMutation
**ID**: `sheet.mutation.set-range-protection`
**Purpose**: Updates protection rule
**Strategy**: ID-based + Range shifting
**Parameters**: `rule_id: String, rule: IRangeProtectionRule`

## Worksheet Protection Mutations

### 4. AddWorksheetProtectionMutation
**ID**: `sheet.mutation.add-worksheet-protection`
**Purpose**: Adds worksheet-level protection
**Strategy**: LWW at worksheet level

### 5. DeleteWorksheetProtectionMutation
**ID**: `sheet.mutation.delete-worksheet-protection`
**Purpose**: Removes worksheet protection
**Strategy**: LWW at worksheet level

### 6. SetWorksheetProtectionMutation
**ID**: `sheet.mutation.set-worksheet-protection`
**Purpose**: Updates worksheet protection
**Strategy**: LWW at worksheet level

### 7. SetWorksheetPermissionPointsMutation
**ID**: `sheet.mutation.set-worksheet-permission-points`
**Purpose**: Sets permission points for worksheet
**Strategy**: LWW at worksheet level

## Conflict Dimensions Analysis

### Range Protection
- **Dimension 1-2**: Standard worksheet matching
- **Dimension 3**: Ranges need shifting with structural mutations
- **Dimension 4**: `rule_id` for identity

### Worksheet Protection
- **Dimension 1-2**: Standard worksheet matching
- **Dimension 3**: Not applicable (worksheet-level)
- **Dimension 4**: Not applicable (singleton per worksheet)

## Transform Coverage Matrix

### Range Protection

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| InsertRowMutation | ⏳ | Shifting | Rule ranges shift rows |
| InsertColMutation | ⏳ | Shifting | Rule ranges shift columns |
| RemoveRowsMutation | ⏳ | Shifting + Removal | Rules may be removed |
| RemoveColMutation | ⏳ | Shifting + Removal | Rules may be removed |
| AddRangeProtectionMutation (self) | ✅ | Identity (diff ID) / LWW (same ID) | By rule_id |
| DeleteRangeProtectionMutation | ⏳ | Conflict | Add vs Delete same ID |

### Worksheet Protection

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| All structural mutations | ❌ | Identity | Worksheet-level property |
| All data mutations | ❌ | Identity | Independent |
| Self | ✅ | LWW | m2 wins on same worksheet |

## Implementation Pattern

### Range Protection (with ranges)
```rust
// Similar to AddConditionalRuleMutation
fn create_insert_row_vs_range_protection() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        // Shift ranges in protection rules
        for rule in &mut m2_params.rules {
            for range in &mut rule.ranges {
                shift_range_rows_for_insert(range, insert_start, insert_count);
            }
        }
        // ...
    })
}
```

### Worksheet Protection (LWW)
```rust
// Same as SetFrozenMutation pattern
fn create_self_transform() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return identity(m1, m2);
        }

        TransformResultRef {
            m1_prime: MutationOutcome::Removed,
            m2_prime: MutationOutcome::Unchanged(m2),
            error: None,
        }
    })
}
```

## Implementation Checklist

- [ ] Range protection transforms with structural mutations
- [x] Worksheet protection self-transforms (LWW)
- [ ] Add vs Delete conflict resolution
- [ ] Tests written
- [x] This document updated

## Last Updated

**Date**: 2026-01-31
**By**: Claude Code
**Changes**: Batch document for protection mutations
