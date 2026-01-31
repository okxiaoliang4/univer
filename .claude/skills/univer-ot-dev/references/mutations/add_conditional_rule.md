# AddConditionalRuleMutation Conflict Analysis

**Mutation ID**: `sheet.mutation.add-conditional-rule`
**File**: `mutations/sheets_conditional_formatting/conditional_formatting_mutation.rs`
**Transform**: `transforms/sheets_conditional_formatting/conditional_formatting.rs`
**Tests**: `tests/conditional_formatting_tests.rs`

## Mutation Overview

**Purpose**: Adds a conditional formatting rule to a worksheet.

**Affects**:
- [ ] Rows
- [ ] Columns
- [x] Cells/Ranges (rule applies to ranges)
- [ ] Worksheet properties
- [ ] Workbook properties
- [x] Feature-specific (conditional formatting)

**Parameters**:
```rust
pub struct AddConditionalRuleMutationParams {
    pub unit_id: String,           // Workbook ID
    pub sub_unit_id: String,       // Worksheet ID
    pub rule: IConditionFormattingRule,  // The rule to add
}

pub struct IConditionFormattingRule {
    pub cf_id: String,             // Unique rule ID
    pub ranges: Vec<IRange>,       // Ranges this rule applies to
    pub rule: ConditionRule,       // The actual condition
    pub stop_if_true: Option<bool>,
    pub priority: Option<i32>,     // Rule priority
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
- Ranges: Rule.ranges define where formatting applies

**Conflict scenarios**:
1. **Row insert/remove**: Rule ranges need adjustment
2. **Column insert/remove**: Rule ranges need adjustment

### Dimension 4: Feature Plugin ID
**Feature-specific IDs**: `cf_id` (conditional formatting rule ID)

**Conflicts when**: Same `cf_id` → LWW (rare, usually different IDs)

## Transform Coverage Matrix

### Core Sheets Mutations

#### Structural Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| InsertRowMutation | ✅ | Shifting | Rule ranges shift rows |
| InsertColMutation | ✅ | Shifting | Rule ranges shift columns |
| RemoveRowsMutation | ✅ | Shifting + Removal | Rule ranges shrink/removed |
| RemoveColMutation | ✅ | Shifting + Removal | Rule ranges shrink/removed |
| MoveRowsMutation | ⏳ | Complex | Rule ranges may be affected |
| MoveColumnsMutation | ⏳ | Complex | Rule ranges may be affected |
| MoveRangeMutation | ⏳ | Complex | Rule ranges may be affected |

#### Data/Merge/Other Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| SetRangeValuesMutation | ❌ | Identity | Cell values independent of CF rules |
| AddWorksheetMergeMutation | ❌ | Identity | Merge independent of CF rules |
| (Most other mutations) | ❌ | Identity | Independent operations |

### Conditional Formatting Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| AddConditionalRuleMutation (self) | ✅ | Identity (diff ID) / LWW (same ID) | Both rules added |
| DeleteConditionalRuleMutation | ⏳ | Conflict | Add vs Delete same cf_id |
| SetConditionalRuleMutation | ⏳ | Conflict | Add vs Set same cf_id |
| MoveConditionalRuleMutation | ❌ | Identity | Priority ordering, not positions |

## Detailed Conflict Resolution

### InsertRowMutation vs AddConditionalRuleMutation

**Conflict Analysis**:
- **Dimension 3 (Position)**: Rule ranges need row shifting

**Resolution Strategy**: Shifting

**Implementation**:
```rust
fn create_insert_row_vs_cf() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return identity(m1, m2);
        }

        let m1_params: InsertRowMutationParams = parse(m1)?;
        let mut m2_params: AddConditionalRuleMutationParams = parse(m2)?;

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

**Status**: ✅ Implemented

---

### AddConditionalRuleMutation vs AddConditionalRuleMutation (Self-Transform)

**Conflict Analysis**:
- **Dimension 4 (Feature ID)**: Different cf_id = no conflict; Same cf_id = LWW

**Resolution Strategy**: Identity (different IDs) / LWW (same ID)

**Implementation**:
```rust
fn create_self_transform() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return identity(m1, m2);
        }

        let m1_params: AddConditionalRuleMutationParams = parse(m1)?;
        let m2_params: AddConditionalRuleMutationParams = parse(m2)?;

        // Different cf_id: both rules add (identity)
        if m1_params.rule.cf_id != m2_params.rule.cf_id {
            return identity(m1, m2);
        }

        // Same cf_id: m2 wins (LWW)
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

- [x] Self-transform implemented
- [x] Transform with InsertRowMutation implemented
- [x] Transform with InsertColMutation implemented
- [x] Transform with RemoveRowsMutation implemented
- [x] Transform with RemoveColMutation implemented
- [ ] Transform with Delete/Set conditional rule mutations
- [x] Tests written for implemented scenarios
- [x] This document updated

## Notes

- Conditional formatting rules have unique `cf_id` identifiers
- Rules can have multiple ranges (Vec<IRange>)
- Different rules are independent (identity transform)
- Same `cf_id` = LWW (rare case, usually server generates unique IDs)

## Last Updated

**Date**: 2026-01-31
**By**: Claude Code
**Changes**: Initial document creation
