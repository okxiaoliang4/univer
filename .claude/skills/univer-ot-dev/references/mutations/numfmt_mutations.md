# Number Format Mutations - Batch Analysis

This document covers number format mutations.

## Covered Mutations

### 1. SetNumfmtMutation
**ID**: `sheet.mutation.set.numfmt`
**Purpose**: Sets number format for cells
**Strategy**: Range-based + Shifting
**Parameters**: `values, ref_map, unit_id, sub_unit_id`

**Data Structure**:
- `values`: HashMap of format patterns
- `ref_map`: HashMap<String, Vec<IRange>> mapping format IDs to ranges

### 2. RemoveNumfmtMutation
**ID**: `sheet.mutation.remove.numfmt`
**Purpose**: Removes number format from cells
**Strategy**: Range-based + Shifting
**Parameters**: `ranges: Vec<IRange>`

## Conflict Dimensions Analysis

### Dimension 1: Unit ID (Workbook)
- **Conflicts when**: Same `unit_id`
- **No conflict when**: Different `unit_id` → Identity transform

### Dimension 2: Sub-Unit ID (Worksheet)
- **Conflicts when**: Same `unit_id` AND same `sub_unit_id`
- **No conflict when**: Different worksheets → Identity transform

### Dimension 3: Position (Row/Col/Range)
**Positions affected**:
- SetNumfmt: ref_map ranges (Vec<IRange> per format ID)
- RemoveNumfmt: ranges (Vec<IRange>)

**Conflict scenarios**:
1. **Row/column insert**: All ranges shift
2. **Row/column remove**: Ranges may be removed or shifted
3. **Overlapping Set vs Remove**: Complex conflict

### Dimension 4: Feature Plugin ID
**Feature-specific IDs**: Format pattern IDs in values HashMap

## Transform Coverage Matrix

### With Structural Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| InsertRowMutation | ⏳ | Shifting | All ranges shift rows |
| InsertColMutation | ⏳ | Shifting | All ranges shift columns |
| RemoveRowsMutation | ⏳ | Shifting + Removal | Ranges may be removed |
| RemoveColMutation | ⏳ | Shifting + Removal | Ranges may be removed |
| MoveRowsMutation | ⏳ | Complex | Ranges may move |
| MoveColumnsMutation | ⏳ | Complex | Ranges may move |
| MoveRangeMutation | ⏳ | Complex | May affect format ranges |

### Self-Transforms

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| SetNumfmtMutation (self) | ⏳ | Complex | Overlapping ranges: LWW at range level |
| RemoveNumfmtMutation (self) | ✅ | Union | Both remove: union of ranges |
| SetNumfmtMutation vs RemoveNumfmtMutation | ⏳ | Conflict | Set vs Remove overlaps |

## Implementation Pattern

### Range-Based Shifting (Set)
```rust
fn create_insert_row_vs_numfmt() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return identity(m1, m2);
        }

        let m1_params: InsertRowMutationParams = parse(m1)?;
        let mut m2_params: SetNumfmtMutationParams = parse(m2)?;

        let insert_start = m1_params.range.start_row;
        let insert_count = m1_params.range.end_row - insert_start + 1;

        // Shift all ranges in ref_map
        for (_format_id, ranges) in m2_params.ref_map.iter_mut() {
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
```

### Range-Based Shifting (Remove)
```rust
fn create_remove_row_vs_remove_numfmt() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return identity(m1, m2);
        }

        let m1_params: RemoveRowsMutationParams = parse(m1)?;
        let mut m2_params: RemoveNumfmtMutationParams = parse(m2)?;

        let remove_start = m1_params.range.start_row;
        let remove_end = m1_params.range.end_row;

        // Shift/remove format ranges
        m2_params.ranges.retain_mut(|range| {
            shift_range_rows_for_remove(range, remove_start, remove_end)
        });

        TransformResultRef {
            m1_prime: MutationOutcome::Unchanged(m1),
            m2_prime: MutationOutcome::Modified(serialize(m2_params)),
            error: None,
        }
    })
}
```

### Self-Transform (SetNumfmt)
```rust
// Complex: overlapping ranges with different formats
fn create_self_transform() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return identity(m1, m2);
        }

        let m1_params = parse(m1)?;
        let m2_params = parse(m2)?;

        // For each format in m1, check if any range overlaps with m2's ranges
        // If overlap: m2 wins (LWW) - remove overlapping portions from m1

        // This is complex range arithmetic
        // ... overlap detection and subtraction logic ...

        TransformResultRef {
            m1_prime: MutationOutcome::Modified(/* adjusted m1 */),
            m2_prime: MutationOutcome::Unchanged(m2),
            error: None,
        }
    })
}
```

### Self-Transform (RemoveNumfmt)
```rust
// Simpler: union of ranges
fn create_self_transform() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return identity(m1, m2);
        }

        // Both remove: can merge or keep both (idempotent at cell level)
        // Simplest: keep m2 only (both achieve same result)
        TransformResultRef {
            m1_prime: MutationOutcome::Removed,
            m2_prime: MutationOutcome::Unchanged(m2),
            error: None,
        }
    })
}
```

## Key Insights

1. **Complex data structure**: SetNumfmt has HashMap<formatId, Vec<IRange>>
2. **Range-based operations**: All about shifting Vec<IRange>
3. **Overlap complexity**: SetNumfmt self-transform requires range arithmetic
4. **Remove is simpler**: Just shift/remove ranges

## Implementation Checklist

- [ ] SetNumfmtMutation transforms with structural mutations
- [ ] RemoveNumfmtMutation transforms with structural mutations
- [ ] SetNumfmtMutation self-transform (range overlap)
- [ ] RemoveNumfmtMutation self-transform (union)
- [ ] Set vs Remove conflict resolution
- [ ] Tests written
- [x] This document updated

## Last Updated

**Date**: 2026-01-31
**By**: Claude Code
**Changes**: Batch document for number format mutations
