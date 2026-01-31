# Visibility Mutations - Batch Analysis

This document covers row/column visibility mutations that follow similar patterns to dimension mutations.

## Covered Mutations

### 1. SetRowVisibleMutation
**ID**: `sheet.mutation.set-row-visible`
**Purpose**: Shows previously hidden rows
**Strategy**: Conflict Scope at row level + Shifting

### 2. SetRowHiddenMutation
**ID**: `sheet.mutation.set-row-hidden`
**Purpose**: Hides rows
**Strategy**: Conflict Scope at row level + Shifting

### 3. SetColVisibleMutation
**ID**: `sheet.mutation.set-col-visible`
**Purpose**: Shows previously hidden columns
**Strategy**: Conflict Scope at column level + Shifting

### 4. SetColHiddenMutation
**ID**: `sheet.mutation.set-col-hidden`
**Purpose**: Hides columns
**Strategy**: Conflict Scope at column level + Shifting

## Common Parameters Pattern

```rust
pub struct SetRowVisibleMutationParams {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub ranges: Vec<IRange>,  // Row ranges to show/hide
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
**Positions affected**: Row or column indices in ranges

**Conflict scenarios**:
1. **Insert before**: Indices shift
2. **Remove**: Indices shift or removed
3. **Same row/col, different visibility**: LWW

## Transform Coverage Matrix

### Row Visibility Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| InsertRowMutation | ✅ | Shifting | Row ranges shift |
| RemoveRowsMutation | ✅ | Shifting + Removal | Ranges may be removed |
| InsertColMutation | ❌ | Identity | Column ops don't affect row visibility |
| RemoveColMutation | ❌ | Identity | Column ops don't affect row visibility |
| SetRowVisibleMutation (self) | ✅ | Conflict Scope | Different rows: independent; Same: LWW |
| SetRowHiddenMutation | ⏳ | Conflict | Visible vs Hidden |

### Column Visibility Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| InsertColMutation | ✅ | Shifting | Column ranges shift |
| RemoveColMutation | ✅ | Shifting + Removal | Ranges may be removed |
| InsertRowMutation | ❌ | Identity | Row ops don't affect column visibility |
| RemoveRowsMutation | ❌ | Identity | Row ops don't affect column visibility |
| SetColVisibleMutation (self) | ✅ | Conflict Scope | Different columns: independent; Same: LWW |
| SetColHiddenMutation | ⏳ | Conflict | Visible vs Hidden |

## Standard Implementation Pattern

Similar to SetWorksheetRowHeightMutation:

```rust
fn create_self_transform() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return identity(m1, m2);
        }

        let m1_params = parse(m1)?;
        let m2_params = parse(m2)?;

        let mut m1_prime_ranges = m1_params.ranges.clone();

        // Remove ranges from m1 that overlap with m2's ranges
        for m2_range in &m2_params.ranges {
            m1_prime_ranges.retain(|m1_range| {
                !ranges_overlap(m1_range, m2_range)
            });
        }

        if m1_prime_ranges.is_empty() {
            return TransformResultRef {
                m1_prime: MutationOutcome::Removed,
                m2_prime: MutationOutcome::Unchanged(m2),
                error: None,
            };
        }

        let mut m1_prime_params = m1_params.clone();
        m1_prime_params.ranges = m1_prime_ranges;

        TransformResultRef {
            m1_prime: MutationOutcome::Modified(serialize(m1_prime_params)),
            m2_prime: MutationOutcome::Unchanged(m2),
            error: None,
        }
    })
}
```

## Implementation Checklist

- [x] Self-transforms (conflict scope) implemented
- [x] Transforms with InsertRow/Col implemented
- [x] Transforms with RemoveRows/Col implemented
- [ ] Transforms between Visible and Hidden mutations
- [x] Tests written for basic scenarios
- [x] This document updated

## Last Updated

**Date**: 2026-01-31
**By**: Claude Code
**Changes**: Batch document for visibility mutations
