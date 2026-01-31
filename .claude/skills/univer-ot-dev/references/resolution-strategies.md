# Transform Resolution Strategies

This document details the 5 core conflict resolution strategies used in Univer OT transforms.

## The 5 Resolution Strategies

### 1. Identity Transform (Zero-Copy)
**When to use**: Mutations don't interfere with each other

**Characteristics:**
- Most common strategy (~40% of transforms)
- Zero-copy implementation (returns references)
- Used for different worksheets, different positions, different feature domains

**Implementation pattern:**
```rust
fn create_identity() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        TransformResultRef::identity(m1, m2)
    })
}

// Or use helper
registry.register_identity(MUTATION_ID, OTHER_MUTATION_ID);
```

**When to apply:**
- Different `unitId` (different workbooks)
- Different `subUnitId` (different worksheets)
- No position overlap (different rows/columns/ranges)
- Different feature types with no dependencies
- Data vs formatting (independent concerns)

**Example registrations:**
```rust
// Different feature types
registry.register_identity(SET_RANGE_VALUES_ID, ADD_MERGE_ID);
registry.register_identity(SET_FILTER_ID, SET_CONDITIONAL_FORMATTING_ID);

// Data vs styling
registry.register_identity(SET_RANGE_VALUES_ID, SET_RANGE_THEME_ID);
```

### 2. Shifting/Adjustment
**When to use**: Structural mutations (insert/remove) affect positions in other mutations

**Characteristics:**
- Second most common strategy for transforms with insert/remove
- Adjusts row/column indices, range boundaries, HashMap keys
- Maintains operation intent by shifting references

**Implementation pattern:**
```rust
fn create_insert_row_vs_set_range() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        let m1_params: InsertRowMutationParams = parse(m1)?;
        let m2_params: SetRangeValuesMutationParams = parse(m2)?;

        if !same_worksheet(&m1_params, &m2_params) {
            return identity(m1, m2);
        }

        let insert_start = m1_params.range.start_row;
        let insert_count = m1_params.range.end_row - insert_start + 1;

        let mut m2_prime = m2_params.clone();

        // Shift cell value HashMap keys
        if let Some(cell_value) = &mut m2_prime.cell_value {
            shift_row_keys_for_insert(cell_value, insert_start, insert_count);
        }

        TransformResultRef {
            m1_prime: MutationOutcome::Unchanged(m1),
            m2_prime: MutationOutcome::Modified(serialize(m2_prime)),
            error: None,
        }
    })
}
```

**Shift utilities** (from `utils/shift.rs`):

```rust
// Shift HashMap keys (uses std::mem::take for zero-copy)
shift_row_keys_for_insert(cell_value, insert_start, insert_count)
shift_row_keys_for_remove(cell_value, remove_start, remove_end)
shift_col_keys_for_insert(cell_value, insert_start, insert_count)
shift_col_keys_for_remove(cell_value, remove_start, remove_end)

// Shift range boundaries (direct mutation)
shift_range_rows_for_insert(range, insert_start, insert_count)
shift_range_rows_for_remove(range, remove_start, remove_end) -> bool
shift_range_cols_for_insert(range, insert_start, insert_count)
shift_range_cols_for_remove(range, remove_start, remove_end) -> bool
```

**Insert row scenarios:**
```rust
// Scenario 1: Insert before target
if m2_start_row >= m1_insert_start {
    m2_params.range.start_row += insert_count;
    m2_params.range.end_row += insert_count;
}

// Scenario 2: Insert after target
if m2_end_row < m1_insert_start {
    // No shift needed
}
```

**Remove row scenarios:**
```rust
// Scenario 1: Remove before target
if m2_end < m1_start {
    // No change
} else if m2_start > m1_end {
    // Shift down
    m2_start -= remove_count;
    m2_end -= remove_count;
} else if m2_start >= m1_start && m2_end <= m1_end {
    // Complete removal
    return MutationOutcome::Removed;
} else {
    // Partial overlap - adjust boundaries
    if m2_start < m1_start && m2_end <= m1_end {
        m2_end = m1_start - 1;
    } else if m2_start >= m1_start && m2_end > m1_end {
        m2_start = m1_start;
        m2_end -= remove_count;
    } else {
        // Spans across removal
        m2_end -= remove_count;
    }
}
```

### 3. Last-Write-Wins (LWW)
**When to use**: Both mutations modify the same property or cell

**Characteristics:**
- m2 "wins" (keeps its value)
- m1 either removed completely or has conflicting portion removed
- Used for single-value properties and cell-level conflicts

**Implementation pattern for property:**
```rust
fn create_lww() -> TransformFnRef {
    Arc::new(|_m1: &MutationInfo, m2: &MutationInfo| {
        TransformResultRef {
            m1_prime: MutationOutcome::Removed,  // m1 loses
            m2_prime: MutationOutcome::Unchanged(m2),
            error: None,
        }
    })
}
```

**Implementation pattern for cell-level LWW:**
```rust
// SetRangeValues vs SetRangeValues
fn create_self_transform() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        let mut m1_prime = m1_params.clone();

        if let (Some(m1_cells), Some(m2_cells)) = (&m1_params.cell_value, &m2_params.cell_value) {
            let mut m1_prime_cells = m1_cells.clone();

            // Remove conflicting cells from m1
            for (row_key, m1_row) in m1_cells {
                if let Some(m2_row) = m2_cells.get(row_key) {
                    let mut m1_prime_row = m1_row.clone();

                    for col_key in m1_row.keys() {
                        if m2_row.contains_key(col_key) {
                            m1_prime_row.remove(col_key);  // LWW: m2 wins
                        }
                    }

                    if m1_prime_row.is_empty() {
                        m1_prime_cells.remove(row_key);
                    } else {
                        m1_prime_cells.insert(row_key.clone(), m1_prime_row);
                    }
                }
            }

            m1_prime.cell_value = if m1_prime_cells.is_empty() {
                None
            } else {
                Some(m1_prime_cells)
            };
        }

        TransformResultRef {
            m1_prime: MutationOutcome::Modified(serialize(m1_prime)),
            m2_prime: MutationOutcome::Unchanged(m2),
            error: None,
        }
    })
}
```

**Use cases:**
- SetWorksheetName vs SetWorksheetName → Full LWW
- SetRangeValues vs SetRangeValues → Cell-level LWW
- SetTabColor vs SetTabColor → Full LWW
- SetFilterRange vs SetFilterRange → Full LWW
- AddConditionalFormatting (same ruleId) → Full LWW

### 4. Conflict Scope Resolution
**When to use**: Operations on different rows/columns are independent even in same range context

**Characteristics:**
- Fine-grained conflict detection
- Operations on different rows/columns don't conflict
- Preserves both operations when they target different scopes

**Implementation pattern:**
```rust
// SetRowHeight vs SetRowHeight (from dimensions.rs)
fn create_set_row_height_vs_set_row_height() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        let mut m1_prime = m1_params.clone();
        let mut m2_prime = m2_params.clone();

        // Remove rows from m1 that m2 also sets
        if let Some(m1_ranges) = &mut m1_prime.ranges {
            m1_ranges.retain(|m1_range| {
                // Check if m2 sets this row
                !m2_params.ranges.iter().any(|m2_range| {
                    m1_range.start_row == m2_range.start_row
                })
            });
        }

        // m2 keeps all its ranges (wins on conflicts)

        TransformResultRef {
            m1_prime: if m1_prime.ranges.is_empty() {
                MutationOutcome::Removed
            } else {
                MutationOutcome::Modified(serialize(m1_prime))
            },
            m2_prime: MutationOutcome::Unchanged(m2),
            error: None,
        }
    })
}
```

**Use cases:**
- SetRowHeight vs SetRowHeight (row scope)
- SetColWidth vs SetColWidth (column scope)
- SetRowData vs SetRowData (row scope with LWW per row)
- SetColVisible vs SetColVisible (column scope)

### 5. Removal/Cancellation
**When to use**: One mutation negates or removes what the other references

**Characteristics:**
- Result in `MutationOutcome::Removed` for affected mutation
- Used when operations conflict completely
- Maintains consistency by removing invalidated operations

**Implementation pattern:**
```rust
// RemoveRows completely contains target range
if m2_start >= m1_start && m2_end <= m1_end {
    return TransformResultRef {
        m1_prime: MutationOutcome::Unchanged(m1),
        m2_prime: MutationOutcome::Removed,  // Completely removed
        error: None,
    };
}
```

**Scenarios:**
1. **Complete range removal**: RemoveRows removes all rows a SetRangeValues targets
2. **Merge removal**: RemoveMerge removes a merge range that AddMerge references
3. **Full property reset**: One mutation resets what another sets (rare, usually LWW instead)
4. **Sheet deletion**: RemoveSheet makes sheet-specific operations invalid (handled by service layer)

**Partial removal with adjustment:**
```rust
// Range partially removed - keep remaining portion
if m2_start < m1_start && m2_end > m1_start && m2_end <= m1_end {
    // Keep portion before removal
    m2_prime.range.end_row = m1_start - 1;
} else if m2_start >= m1_start && m2_start <= m1_end && m2_end > m1_end {
    // Keep portion after removal
    m2_prime.range.start_row = m1_start;
    m2_prime.range.end_row -= remove_count;
}
```

## Strategy Selection Decision Tree

```
1. Same worksheet?
   NO  → Identity (Strategy 1)
   YES → Continue

2. Is m1 or m2 a structural mutation (insert/remove)?
   YES → Shifting (Strategy 2)
   NO  → Continue

3. Same property/cell/position?
   NO  → Check if independent scopes
     - Different rows/cols? → Conflict Scope (Strategy 4) or Identity
     - Different ranges? → Identity (Strategy 1)
   YES → Continue

4. One removes/resets other's target?
   YES → Removal (Strategy 5)
   NO  → LWW (Strategy 3)

5. Fine-grained conflict detection needed?
   YES → Cell-level LWW (Strategy 3 variant) or Conflict Scope (Strategy 4)
   NO  → Full LWW (Strategy 3)
```

## Strategy Combinations

Some transforms use multiple strategies:

### Example: InsertRow vs MoveRange
```rust
// Strategy 1: Identity if different worksheets
// Strategy 2: Shift source and target ranges if affected
fn create_transform() -> TransformFnRef {
    Arc::new(|m1, m2| {
        if !same_worksheet(m1, m2) {
            return identity(m1, m2);  // Strategy 1
        }

        let mut m2_prime = m2_params.clone();

        // Strategy 2: Shift source range
        if m2_prime.from_range.start_row >= insert_start {
            shift_range_rows_for_insert(&mut m2_prime.from_range, insert_start, count);
        }

        // Strategy 2: Shift target range
        if m2_prime.to_range.start_row >= insert_start {
            shift_range_rows_for_insert(&mut m2_prime.to_range, insert_start, count);
        }

        // Return modified
        TransformResultRef {
            m1_prime: MutationOutcome::Unchanged(m1),
            m2_prime: MutationOutcome::Modified(serialize(m2_prime)),
            error: None,
        }
    })
}
```

### Example: RemoveRows vs SetRangeValues
```rust
// Combination of Strategy 2 (Shifting), Strategy 5 (Removal), and partial adjustment
fn create_transform() -> TransformFnRef {
    Arc::new(|m1, m2| {
        let mut m2_prime = m2_params.clone();

        if let Some(cell_value) = &mut m2_prime.cell_value {
            // Strategy 5: Remove keys within removal range
            // Strategy 2: Shift keys after removal range
            shift_row_keys_for_remove(cell_value, remove_start, remove_end);

            if cell_value.is_empty() {
                return TransformResultRef {
                    m1_prime: MutationOutcome::Unchanged(m1),
                    m2_prime: MutationOutcome::Removed,  // Strategy 5
                    error: None,
                };
            }
        }

        TransformResultRef {
            m1_prime: MutationOutcome::Unchanged(m1),
            m2_prime: MutationOutcome::Modified(serialize(m2_prime)),
            error: None,
        }
    })
}
```

## Performance Considerations

### Strategy Efficiency Ranking (Best to Worst)

1. **Identity** - Zero-copy, O(1)
2. **LWW (full property)** - O(1), single assignment
3. **Conflict Scope** - O(n) where n = number of ranges/items
4. **LWW (cell-level)** - O(n*m) where n = rows, m = columns in conflict
5. **Shifting** - O(n) for HashMap reconstruction, O(1) for range adjustment
6. **Removal** - O(1) for full removal, O(n) for partial

### Optimization Patterns

**1. Early identity checks (zero-copy):**
```rust
// Check before parsing
if let Some(false) = same_worksheet(&m1.params, &m2.params) {
    return identity(m1, m2);  // Zero clones!
}
```

**2. Conditional cloning:**
```rust
// Only clone if modification needed
let m2_prime = if needs_shift {
    let mut modified = m2_params.clone();
    shift_range(&mut modified.range, start, count);
    MutationOutcome::Modified(serialize(modified))
} else {
    MutationOutcome::Unchanged(m2)
};
```

**3. Use `std::mem::take` for HashMap shifts:**
```rust
// Zero-copy move, reconstruct with shifts
let mut new_map = HashMap::with_capacity(old_map.len());
for (key, value) in std::mem::take(old_map) {
    let new_key = compute_shifted_key(key, shift);
    new_map.insert(new_key, value);
}
*old_map = new_map;
```

**4. Batch identity registrations:**
```rust
// Register all feature mutations as identity with core
for &feature_id in FEATURE_MUTATIONS {
    for &core_id in CORE_MUTATIONS {
        registry.register_identity(feature_id, core_id);
    }
}
```

## Strategy Testing Patterns

Each strategy requires specific test cases:

### Identity Tests
- Different worksheets
- Different positions (no overlap)
- Different feature types

### Shifting Tests
- Insert before target
- Insert after target
- Insert within target
- Remove before target
- Remove after target
- Remove overlapping target (partial)
- Remove containing target (complete)

### LWW Tests
- Same property, different values
- Same cells, different values
- Cell-level: partial overlap
- Cell-level: complete overlap

### Conflict Scope Tests
- Same operation, different rows/columns
- Same operation, overlapping rows/columns
- Empty result after conflict removal

### Removal Tests
- Complete removal
- Partial removal with adjustment
- No removal (no overlap)
