# Conflict Dimensions in Univer OT

This document details the 4 conflict dimensions used to determine when two mutations require operational transformation.

## The 4 Conflict Dimensions

### 1. Unit ID Dimension (`unitId`)
**Scope**: Workbook level

**When conflicts occur**: Same workbook (`unitId` matches)

**When NO conflict**: Different workbooks

```rust
// Quick check - different workbooks never conflict
if m1_params.unit_id != m2_params.unit_id {
    return identity(m1, m2);  // Zero-copy
}
```

### 2. Sub-Unit ID Dimension (`subUnitId`)
**Scope**: Worksheet level

**When conflicts occur**: Same worksheet (`unitId` AND `subUnitId` match)

**When NO conflict**: Different worksheets

```rust
// Standard worksheet check pattern
if m1_params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id
    || m1_params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id
{
    return identity(m1, m2);  // Zero-copy
}
```

**Optimization**: Use `same_worksheet()` utility for early check:
```rust
// Avoid parsing if different worksheets
if let Some(false) = same_worksheet(&m1.params, &m2.params) {
    return identity(m1, m2);
}
```

### 3. Row/Column/Range Dimension
**Scope**: Cell position level

**Sub-dimensions:**
- **Row range**: `start_row`, `end_row`
- **Column range**: `start_column`, `end_column`
- **Cell coordinates**: HashMap keys (e.g., "0", "5", "10")
- **Single row/column**: Specific row or column index

**Conflict scenarios:**
1. **Overlapping ranges**: Require adjustment or LWW
2. **Adjacent ranges**: May require shifting (insert/remove)
3. **Separated ranges**: No conflict (identity)

**Conflict granularity examples:**

```rust
// Cell-level granularity (set_range_values.rs)
// ONLY conflicting cells use LWW, non-conflicting cells in same range preserved
for col_key in m1_row.keys() {
    if m2_row.contains_key(col_key) {
        m1_prime_row.remove(col_key);  // Conflict: LWW
    } else {
        // No conflict: preserve both operations
    }
}
```

```rust
// Row-level granularity (insert_row.rs)
// Rows >= insert position shift, others unchanged
if target_row >= insert_start {
    target_row += insert_count;  // Shift
}
```

```rust
// Range overlap detection (remove_rows.rs)
if m2_end < m1_start {
    // No conflict: m2 before m1
} else if m2_start > m1_end {
    // Shift: m2 after m1, shift down
} else if m2_start >= m1_start && m2_end <= m1_end {
    // Complete overlap: m2 removed
} else {
    // Partial overlap: adjust remaining portion
}
```

### 4. Feature Plugin ID Dimension
**Scope**: Feature-specific identifiers within plugins

**Examples:**
- `ruleId` in conditional formatting
- `filterId` in sheets filter
- `pivotTableId` in pivot tables
- `protectionId` in range protection
- `mergeId` (implicit, via range) in merge cells

**When conflicts occur**: Same feature ID

**When NO conflict**: Different feature IDs or different feature types

```rust
// Example: conditional formatting rules
if m1_params.rule_id == m2_params.rule_id {
    // Same rule: LWW or merge logic
} else {
    // Different rules: likely identity
}
```

## Conflict Analysis Decision Tree

Use this tree to determine conflict resolution strategy:

```
1. Same unitId?
   NO  → Identity (different workbooks)
   YES → Continue to 2

2. Same subUnitId?
   NO  → Identity (different worksheets)
   YES → Continue to 3

3. Does mutation affect rows/columns/ranges?
   NO  → Check property conflicts (dimension 4)
   YES → Continue to 4

4. Row/Column/Range overlap?
   NO  → Identity (different positions)
   YES → Continue to 5

5. Type of overlap?
   - Structural mutation (insert/remove) → Shifting strategy
   - Data mutation on same cells → LWW or merge
   - Property mutation on same range → LWW
   - Feature mutation with same ID → LWW or feature-specific logic

6. Does mutation have feature plugin ID?
   YES → Check if same ID
     - Same ID → LWW or feature logic
     - Different ID → Depends on position conflict
   NO  → Use position-based resolution from step 5
```

## Common Conflict Patterns

### Pattern 1: Structural + Data
**Example**: InsertRow + SetRangeValues

- **Dimension 1-2**: Must be same worksheet
- **Dimension 3**: Check if range >= insert position
- **Resolution**: Shift range keys in cell_value HashMap

### Pattern 2: Structural + Structural
**Example**: InsertRow + InsertRow

- **Dimension 1-2**: Must be same worksheet
- **Dimension 3**: Check relative positions
- **Resolution**: Shift second insert position if >= first

### Pattern 3: Data + Data
**Example**: SetRangeValues + SetRangeValues

- **Dimension 1-2**: Must be same worksheet
- **Dimension 3**: Check cell-level conflicts
- **Resolution**: LWW at cell granularity, preserve non-conflicting

### Pattern 4: Property + Property
**Example**: SetWorksheetName + SetWorksheetName

- **Dimension 1-2**: Must be same worksheet
- **Dimension 3**: N/A (worksheet-level property)
- **Resolution**: LWW (m1 removed, m2 wins)

### Pattern 5: Cross-Feature Independence
**Example**: SetFilter + SetConditionalFormatting

- **Dimension 1-2**: Same worksheet
- **Dimension 3**: May overlap in range
- **Dimension 4**: Different feature types
- **Resolution**: Identity (features independent)

### Pattern 6: Same Feature, Different IDs
**Example**: AddPivotTable(id1) + AddPivotTable(id2)

- **Dimension 1-2**: Same worksheet
- **Dimension 3**: May overlap in range
- **Dimension 4**: Different pivot table IDs
- **Resolution**: Depends on range conflict + feature-specific rules

## Zero-Copy Optimization Strategy

Prioritize checks by cost (cheapest first):

```rust
// 1. Cheapest: Early worksheet check (no parsing)
if let Some(false) = same_worksheet(&m1.params, &m2.params) {
    return identity(m1, m2);  // Cost: ~0 clones
}

// 2. Parse parameters (required for same worksheet)
let m1_params = parse_params(m1)?;
let m2_params = parse_params(m2)?;

// 3. Check dimension 3 (position) - often avoids modification
if no_position_conflict(&m1_params, &m2_params) {
    return identity(m1, m2);  // Cost: 2 clones (parsing)
}

// 4. Complex conflict resolution (modify params)
let mut m1_prime = m1_params.clone();
let mut m2_prime = m2_params.clone();
// ... apply transforms
```

## Conflict Dimension Checklist

When implementing a new transform, check all 4 dimensions:

- [ ] **Dimension 1**: Different unitId → Identity
- [ ] **Dimension 2**: Different subUnitId → Identity
- [ ] **Dimension 3**: Position conflict?
  - [ ] No overlap → Identity
  - [ ] Insert/remove → Shifting
  - [ ] Same cells → LWW or merge
  - [ ] Adjacent → May need shift
- [ ] **Dimension 4**: Feature ID conflict?
  - [ ] Different feature types → Likely identity
  - [ ] Same feature, same ID → LWW or feature logic
  - [ ] Same feature, different IDs → Check position conflict

## Examples by Mutation Type

### Insert Row Transforms
- vs InsertRow: Shift if position >= (dimension 3)
- vs RemoveRows: Complex overlap (dimension 3)
- vs SetRangeValues: Shift cell row keys (dimension 3)
- vs SetRowData: Shift array keys (dimension 3)
- vs AddMerge: Shift merge ranges (dimension 3)
- vs SetFilter: Shift filter range (dimensions 3 + 4)
- vs AddConditionalFormatting: Shift rule range (dimensions 3 + 4)

### Set Range Values Transforms
- vs SetRangeValues: Cell-level LWW (dimension 3 granular)
- vs InsertRow/Col: Shift cell keys (dimension 3)
- vs RemoveRows/Cols: Drop/shift cells (dimension 3)
- vs SetRowData: Independent operations (different data types)
- vs SetStyle: Independent (values vs formatting)
- vs AddMerge: Independent (values vs merge)

### Add Merge Transforms
- vs AddMerge: Multiple merges allowed if no overlap (dimension 3)
- vs RemoveMerge: May cancel if same range (dimension 3)
- vs InsertRow/Col: Shift merge range (dimension 3)
- vs SetRangeValues: Independent operations
- vs MoveRange: Complex - may need to move merge

### Feature Plugin Transforms
- vs Core mutations: Typically identity (different concerns)
- vs Same feature type: Check IDs (dimension 4)
- vs Structural mutations: Shift feature ranges (dimension 3)
- vs Other feature plugins: Identity (dimension 4 independence)
