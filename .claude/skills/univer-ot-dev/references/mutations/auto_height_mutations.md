# Auto Height Mutations - Batch Analysis

This document covers row auto-height mutations.

## Covered Mutations

### 1. SetWorksheetRowIsAutoHeightMutation
**ID**: `sheet.mutation.set-worksheet-row-is-auto-height`
**Purpose**: Marks rows as having auto-height enabled
**Strategy**: Range-based + Shifting
**Parameters**: `ranges, auto_height_info`

### 2. SetWorksheetRowAutoHeightMutation
**ID**: `sheet.mutation.set-worksheet-row-auto-height`
**Purpose**: Sets calculated auto-height values for rows
**Strategy**: Row keys + Shifting
**Parameters**: `rows_auto_height_info` (HashMap<String, f64>)

### 3. MarkDirtyRowAutoHeightMutation
**ID**: `sheet.operation.mark-dirty-row-auto-height`
**Purpose**: Marks rows needing height recalculation
**Strategy**: Range-based + Shifting
**Parameters**: `ranges, id`

### 4. CancelMarkDirtyRowAutoHeightMutation
**ID**: `sheet.operation.cancel-mark-dirty-row-auto-height`
**Purpose**: Cancels dirty marking
**Strategy**: ID-based
**Parameters**: `id`

## Conflict Dimensions Analysis

### Dimension 1: Unit ID (Workbook)
- **Conflicts when**: Same `unit_id`
- **No conflict when**: Different `unit_id` → Identity transform

### Dimension 2: Sub-Unit ID (Worksheet)
- **Conflicts when**: Same `unit_id` AND same `sub_unit_id`
- **No conflict when**: Different worksheets → Identity transform

### Dimension 3: Position (Row)
**Positions affected**:
- IsAutoHeight: ranges (Vec<IRange>)
- AutoHeight: row keys (HashMap keys)
- MarkDirty: ranges (Vec<IRange>)
- CancelMark: No position (ID-based)

### Dimension 4: Feature Plugin ID
**Feature-specific IDs**:
- MarkDirty: `id` for mark/cancel pairing
- Others: No specific ID

## Transform Coverage Matrix

### Range/Row-Based Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| InsertRowMutation | ✅ | Shifting | Row ranges/keys shift |
| InsertColMutation | ❌ | Identity | Column ops don't affect row height |
| RemoveRowsMutation | ✅ | Shifting + Removal | Rows removed/shifted |
| RemoveColMutation | ❌ | Identity | Column ops don't affect row height |
| MoveRowsMutation | ⏳ | Complex | Rows may move |

### Self-Transforms

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| SetWorksheetRowIsAutoHeightMutation (self) | ✅ | Conflict Scope | Different rows: independent; Same: LWW |
| SetWorksheetRowAutoHeightMutation (self) | ✅ | LWW (row-level) | Same rows: m2 wins |
| MarkDirtyRowAutoHeightMutation (self) | ✅ | Conflict Scope | Different rows: independent; Same ID: LWW |
| CancelMarkDirtyRowAutoHeightMutation (self) | ✅ | Idempotent | Same ID: both cancel |

### Cross-AutoHeight Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| IsAutoHeight vs AutoHeight | ❌ | Identity | Different operations |
| IsAutoHeight vs MarkDirty | ❌ | Identity | Different operations |
| MarkDirty vs CancelMark | ⏳ | Conflict | Mark vs Cancel same ID |
| SetWorksheetRowHeightMutation vs IsAutoHeight | ⏳ | Conflict | Manual vs auto height |

## Implementation Pattern

### Range-Based (IsAutoHeight, MarkDirty)
```rust
// Similar to SetWorksheetRowHeightMutation
fn create_insert_row_vs_auto_height() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return identity(m1, m2);
        }

        let m1_params: InsertRowMutationParams = parse(m1)?;
        let mut m2_params: SetWorksheetRowIsAutoHeightMutationParams = parse(m2)?;

        let insert_start = m1_params.range.start_row;
        let insert_count = m1_params.range.end_row - insert_start + 1;

        // Shift all row ranges
        for range in &mut m2_params.ranges {
            shift_range_rows_for_insert(range, insert_start, insert_count);
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

        let mut m1_prime_ranges = m1_params.ranges.clone();

        // Remove overlapping ranges from m1
        for m2_range in &m2_params.ranges {
            m1_prime_ranges.retain(|m1_range| {
                !ranges_overlap_rows(m1_range, m2_range)
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

### Row-Keys-Based (AutoHeight)
```rust
// Similar to SetRowDataMutation
fn create_insert_row_vs_auto_height_values() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return identity(m1, m2);
        }

        let m1_params: InsertRowMutationParams = parse(m1)?;
        let mut m2_params: SetWorksheetRowAutoHeightMutationParams = parse(m2)?;

        let insert_start = m1_params.range.start_row;
        let insert_count = m1_params.range.end_row - insert_start + 1;

        // Shift row keys
        shift_row_keys_for_insert_generic(&mut m2_params.rows_auto_height_info, insert_start, insert_count);

        TransformResultRef {
            m1_prime: MutationOutcome::Unchanged(m1),
            m2_prime: MutationOutcome::Modified(serialize(m2_params)),
            error: None,
        }
    })
}

// Self-transform: LWW at row level
fn create_self_transform() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return identity(m1, m2);
        }

        let m1_params = parse(m1)?;
        let m2_params = parse(m2)?;

        let mut m1_prime_info = m1_params.rows_auto_height_info.clone();

        // Remove rows from m1 that m2 also sets (LWW)
        for row_key in m2_params.rows_auto_height_info.keys() {
            m1_prime_info.remove(row_key);
        }

        if m1_prime_info.is_empty() {
            return TransformResultRef {
                m1_prime: MutationOutcome::Removed,
                m2_prime: MutationOutcome::Unchanged(m2),
                error: None,
            };
        }

        let mut m1_prime_params = m1_params.clone();
        m1_prime_params.rows_auto_height_info = m1_prime_info;

        TransformResultRef {
            m1_prime: MutationOutcome::Modified(serialize(m1_prime_params)),
            m2_prime: MutationOutcome::Unchanged(m2),
            error: None,
        }
    })
}
```

### ID-Based (CancelMark)
```rust
// Mostly identity with structural mutations
fn create_insert_row_vs_cancel_mark() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        identity(m1, m2)  // No position in params
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

        // Different IDs: independent
        if m1_params.id != m2_params.id {
            return identity(m1, m2);
        }

        // Same ID: both cancel (idempotent)
        TransformResultRef {
            m1_prime: MutationOutcome::Removed,
            m2_prime: MutationOutcome::Unchanged(m2),
            error: None,
        }
    })
}
```

## Key Insights

1. **Three data patterns**: Ranges, row keys (HashMap), ID-based
2. **Related to SetWorksheetRowHeightMutation**: Similar position logic
3. **Mark/Cancel pairing**: MarkDirty and CancelMark use ID for pairing
4. **Auto vs manual conflict**: Auto-height may conflict with manual height setting

## Implementation Checklist

- [x] IsAutoHeight transforms with structural mutations
- [x] AutoHeight transforms with structural mutations
- [x] MarkDirty transforms with structural mutations
- [x] CancelMark (ID-based) patterns defined
- [ ] Cross-auto-height conflict resolution
- [ ] Conflict with SetWorksheetRowHeightMutation
- [ ] Tests written
- [x] This document updated

## Last Updated

**Date**: 2026-01-31
**By**: Claude Code
**Changes**: Batch document for auto-height mutations
