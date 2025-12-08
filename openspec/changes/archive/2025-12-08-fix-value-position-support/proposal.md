## Why

The `PivotEngineV2` class has a `valuePosition` property that controls whether value fields should appear in rows or columns of the pivot table. However, the `setValuePosition()` method marks the engine as dirty (triggering recalculation) but the calculation logic does not actually respect the `valuePosition` setting. This means:

1. **Current Behavior**: When `valuePosition` is changed, the pivot output structure remains unchanged
2. **Expected Behavior**: When `valuePosition` changes between `ROW` and `COLUMN`, the pivot should swap value fields between rows and columns
3. **Missing Test Coverage**: There are no test cases validating that `valuePosition` changes actually affect the output

This is a correctness issue that prevents users from toggling between row-based and column-based value positioning.

## What Changes

### 1. Core Implementation Fix
- **Location**: `pivot-engine-v2.ts` - `_buildCrossTabResult()` method
- **Change**: Implement logic to respect `valuePosition` setting when building the cross-tabulation structure
  - When `valuePosition === ROW`: Value fields should appear as row headers (swap row/column structure)
  - When `valuePosition === COLUMN`: Value fields should appear as column headers (current behavior)

### 2. Test Coverage Addition
- **Location**: `pivot-engine-v2.spec.ts`
- **New Test Scenarios**:
  1. Basic valuePosition change from COLUMN to ROW
  2. Basic valuePosition change from ROW to COLUMN
  3. Multiple value fields with different valuePosition settings
  4. Verify recalculation is triggered when valuePosition changes
  5. Verify output dimensions swap correctly
  6. Verify header structure changes appropriately

## Impact

- **Affected Specs**: `sheets-pivot-table`
- **Affected Code**:
  - `packages/sheets-pivot-table/src/models/pivot-engine-v2.ts` - Core calculation logic
  - `packages/sheets-pivot-table/src/models/__tests__/pivot-engine-v2.spec.ts` - Test coverage
  - Potentially: `packages/sheets-pivot-table/src/models/pivot-table-render-model.ts` (if render model needs adjustment)

### Dependencies

- No new external dependencies
- Follows existing patterns in pivot calculation

## Risks & Mitigations

| Risk | Mitigation |
|------|-----------|
| Breaking change in pivot output structure | Feature was already present but non-functional, so enabling it should not break existing working code |
| Render model may need adjustments | Will verify with render model tests and examples |
| Performance impact on large datasets | Structure swapping is O(n) but acceptable for typical pivot sizes |

## Implementation Summary

The fix involves:

1. **Detect valuePosition in calculation**: Check `this.valuePosition` in `_buildCrossTabResult()`
2. **Swap row/column structure when needed**: When `valuePosition === ROW`, reorder the output so value fields become row headers
3. **Add comprehensive tests**: Create test cases covering all valuePosition scenarios
4. **Validate with examples**: Ensure the example application works correctly with both positioning modes
