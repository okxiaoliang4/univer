# Design Document: Value Position Support in Pivot Tables

## Problem Statement

The `PivotEngineV2` class has a `valuePosition` property that is initialized and can be changed via `setValuePosition()`, but the calculation logic in `_buildCrossTabResult()` does not respect this setting. This results in:

1. The position change being ignored in the output
2. No test coverage validating position changes work correctly
3. Users unable to toggle between row-based and column-based value positioning

## Current Implementation Analysis

### Current Structure

The pivot table output follows this pattern:
```
                  Column Header 1   Column Header 2   ...
Row Header 1      [Value 1,1]       [Value 1,2]       ...
Row Header 2      [Value 2,1]       [Value 2,2]       ...
...
```

Where:
- Row headers come from row fields
- Column headers include column fields + value fields (when `valuePosition: COLUMN`)
- Values are organized in a 2D matrix

### What valuePosition Controls

According to the spec, `valuePosition` should control where value fields appear:

- **`PivotValuePosition.COLUMN`** (default): Value fields are column headers
  - Output has many columns (one per value field per column combo)
  - Output has fewer row headers

- **`PivotValuePosition.ROW`**: Value fields are row headers
  - Output has many rows (one per value field per row combo)
  - Output has fewer column headers

## Implementation Strategy

### Phase 1: Core Logic Implementation

#### Step 1.1: Analyze _buildCrossTabResult()

The method currently:
1. Builds column headers from column field combinations
2. Adds value field headers if multiple value fields exist
3. Processes row groups and builds row headers
4. Aggregates values into the output matrix

**Key Insight**: The method needs to detect `this.valuePosition` and adapt the building logic:
- When `ROW`: Value fields should be treated like additional row field levels
- When `COLUMN`: Keep current behavior (value fields as column headers)

#### Step 1.2: Implement Position-Aware Logic

**For `valuePosition === COLUMN` (current behavior)**:
- Keep existing logic mostly unchanged
- Ensure value field headers appear as column headers
- Values organized in columns

**For `valuePosition === ROW` (new behavior)**:
- Generate row combinations that include value field positions
- Place value field names in row headers
- Transpose value field headers from column to row
- Reorganize the data matrix so values are distributed across rows

#### Step 1.3: Handle Dimension Changes

When position changes, dimensions must be recalculated:
```typescript
// Before (valuePosition: COLUMN)
dimensions = {
  totalRows: 5,      // row combos
  totalColumns: 6,   // column combos * value fields
  dataRowCount: 4,   // excluding subtotals
  dataColumnCount: 5
}

// After (valuePosition: ROW)
dimensions = {
  totalRows: 20,     // row combos * value fields
  totalColumns: 6,   // column combos (no value field columns)
  dataRowCount: 16,
  dataColumnCount: 5
}
```

### Phase 2: Test Coverage

#### Test Categories

**1. Basic Positioning Tests**
- Test with `valuePosition: COLUMN` baseline
- Test with `valuePosition: ROW` produces different structure
- Test both positions work correctly

**2. Position Change Tests**
- Test `COLUMN → ROW` transition
- Test `ROW → COLUMN` transition
- Verify recalculation is triggered

**3. Multi-Value Field Tests**
- Test multiple value fields with `ROW` position
- Test multiple value fields with `COLUMN` position
- Verify each value field gets its own row/column

**4. Edge Case Tests**
- Row-only pivot (no column fields)
- Column-only pivot (no row fields)
- No value fields (should return empty)
- Empty source data

**5. Feature Integration Tests**
- Position changes with subtotals enabled
- Position changes with filters applied
- Dimension validation after position change

## Technical Details

### Row/Column Swapping Strategy

When `valuePosition === ROW`, we need to logically "swap" the role of value fields:

```typescript
// Pseudo-code for the adaptation
if (this.valuePosition === PivotValuePosition.ROW) {
  // For each row group:
  //   For each value field:
  //     Create a new row header entry with value field name
  //     Copy the values from that value field position
  //
  // This effectively expands rows to include value field rows
  // and removes value field columns
} else {
  // Keep current behavior (COLUMN position)
}
```

### Data Structure Changes

**Current (COLUMN) Layout**:
```
rows = [row1, row2, row3]
columns = [col1_value1, col1_value2, col2_value1, col2_value2]
values = [
  [[row1_col1_value1, row1_col1_value2, row1_col2_value1, row1_col2_value2]],
  [[row2_col1_value1, row2_col1_value2, row2_col2_value1, row2_col2_value2]],
  [[row3_col1_value1, row3_col1_value2, row3_col2_value1, row3_col2_value2]],
]
```

**New (ROW) Layout**:
```
rows = [row1, row1_value1, row1_value2, row2, row2_value1, row2_value2, row3, row3_value1, row3_value2]
columns = [col1, col2]
values = [
  [[row1_col1], [row1_col2]],                    // row1 row (totals)
  [[row1_col1_value1], [row1_col2_value1]],      // row1 value1
  [[row1_col1_value2], [row1_col2_value2]],      // row1 value2
  [[row2_col1], [row2_col2]],
  [[row2_col1_value1], [row2_col2_value1]],
  [[row2_col1_value2], [row2_col2_value2]],
  ...
]
```

## Implementation Considerations

### Performance
- The position-aware logic adds minimal overhead (one check per calculation)
- Row/column reorganization is O(n) where n = number of data cells
- Acceptable for typical pivot table sizes (< 100k cells)

### Backward Compatibility
- Default `valuePosition: COLUMN` maintains existing behavior
- Existing code that doesn't set `valuePosition` continues to work
- No breaking changes to public APIs

### Render Model Compatibility
- `PivotTableRenderModel` should work with both positions
- May need minor adjustments if it has hardcoded row/column assumptions
- Tests will verify compatibility

## Verification Plan

1. **Unit Tests**: All test cases in 3.1-3.10 pass
2. **Integration Tests**: Example app works with both positions
3. **Regression Tests**: All existing tests continue to pass
4. **Performance Tests**: No significant degradation with large datasets
5. **Visual Tests**: Render model produces correct output for both positions

## Rollout Plan

1. Implement core logic with `valuePosition` support
2. Add comprehensive test coverage
3. Verify with existing tests (regression)
4. Test with example application
5. Code review and approval
6. Merge to main branch
7. Archive change proposal

## Related Code Areas

- `packages/sheets-pivot-table/src/models/pivot-engine-v2.ts` - Core calculation engine
- `packages/sheets-pivot-table/src/models/__tests__/pivot-engine-v2.spec.ts` - Unit tests
- `packages/sheets-pivot-table/src/models/pivot-table-render-model.ts` - Render model (may need updates)
- `packages/sheets-pivot-table/src/models/pivot-table.ts` - Pivot table model (may need updates)
- `packages/sheets-pivot-table/src/types/type.ts` - Type definitions
- `packages/sheets-pivot-table/src/types/enum.ts` - PivotValuePosition enum
