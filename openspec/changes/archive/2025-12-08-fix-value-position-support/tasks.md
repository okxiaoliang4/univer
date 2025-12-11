## 1. Analyze Current Implementation

- [x] **1.1 Review pivot engine calculation logic**
  - Understand how cross-tabulation structure is currently built
  - Identify where `valuePosition` should affect the output
  - **Verification**: Document the calculation flow in code comments

- [x] **1.2 Study PivotValuePosition enum**
  - Review `PivotValuePosition.ROW` vs `PivotValuePosition.COLUMN` semantics
  - Understand expected behavior for each position
  - **Verification**: Confirm understanding in design notes

- [x] **1.3 Examine render model integration**
  - Check if `PivotTableRenderModel` expects specific output structure
  - Verify compatibility with both ROW and COLUMN positioning
  - **Verification**: No render model changes needed or identified

## 2. Implement valuePosition Support in Calculation

- [x] **2.1 Implement row/column swapping logic**
  - Modify `_buildCrossTabResult()` to respect `valuePosition`
  - When `valuePosition === ROW`: Swap the roles of row and column fields
  - When `valuePosition === COLUMN`: Keep current behavior
  - **Verification**: Code review and manual testing

- [x] **2.2 Handle value field header arrangement**
  - Update `valueFieldHeaders` generation based on `valuePosition`
  - Ensure headers appear in correct dimension
  - **Verification**: Pivot output displays headers correctly

- [x] **2.3 Verify dirty flag behavior**
  - Confirm `setValuePosition()` properly triggers recalculation
  - Verify `_markDirty()` is called and cache is invalidated
  - **Verification**: Console logs or debugging show recalculation happens

- [x] **2.4 Test with render model**
  - Ensure `PivotTableRenderModel` works with new structure
  - Verify no issues in row/column visibility logic
  - **Verification**: Example application renders correctly

## 3. Add Comprehensive Test Cases

- [x] **3.1 Test: valuePosition COLUMN (baseline)**
  - Create pivot with simple row/column/value fields
  - Verify output structure has values in columns
  - **Verification**: Test passes, output structure is correct

- [x] **3.2 Test: valuePosition ROW**
  - Create same pivot but with `valuePosition: ROW`
  - Verify output structure has values in rows instead
  - **Verification**: Test passes, row and column dimensions swap

- [x] **3.3 Test: valuePosition change COLUMN → ROW**
  - Create pivot with `valuePosition: COLUMN`
  - Call `setValuePosition(PivotValuePosition.ROW)`
  - Verify recalculation is triggered
  - Verify output structure changes
  - **Verification**: Test passes, cache invalidation works

- [x] **3.4 Test: valuePosition change ROW → COLUMN**
  - Create pivot with `valuePosition: ROW`
  - Call `setValuePosition(PivotValuePosition.COLUMN)`
  - Verify output reverts to original structure
  - **Verification**: Test passes, bidirectional change works

- [x] **3.5 Test: Multiple value fields with ROW position**
  - Create pivot with multiple value fields
  - Set `valuePosition: ROW`
  - Verify each value field becomes a row
  - **Verification**: Test passes, all value fields appear as rows

- [x] **3.6 Test: Single value field with different positions**
  - Create pivot with one value field
  - Test both `valuePosition: ROW` and `valuePosition: COLUMN`
  - Verify single value appears in correct dimension
  - **Verification**: Test passes, no issues with single value

- [x] **3.7 Test: No value fields**
  - Create pivot with no value fields
  - Set `valuePosition` to either ROW or COLUMN
  - Verify empty result is returned correctly
  - **Verification**: Test passes, edge case handled

- [x] **3.8 Test: Row-only pivot (no columns) with valuePosition**
  - Create pivot with only row fields and value fields
  - Set `valuePosition: ROW` and `valuePosition: COLUMN`
  - Verify both positions work correctly
  - **Verification**: Test passes, single dimension pivot works

- [x] **3.9 Test: Column-only pivot (no rows) with valuePosition**
  - Create pivot with only column fields and value fields
  - Set `valuePosition: ROW` and `valuePosition: COLUMN`
  - Verify both positions work correctly
  - **Verification**: Test passes, single dimension pivot works

- [x] **3.10 Test: Dimension validation after position change**
  - Create pivot with known dimensions
  - Change `valuePosition`
  - Verify dimensions are correctly swapped
  - Verify `totalRows` and `totalColumns` are updated
  - **Verification**: Test passes, dimensions match structure

## 4. Validation & Integration Testing

- [x] **4.1 Run existing test suite**
  - Ensure no regression in other tests
  - All pivot engine tests should pass
  - **Verification**: `pnpm test` passes for sheets-pivot-table

- [x] **4.2 Test with example application**
  - Run example with pivot table containing both ROW and COLUMN positioning
  - Verify visual rendering is correct
  - Verify no console errors
  - **Verification**: Example renders correctly with both settings

- [x] **4.3 Test with PivotTable service**
  - Verify service properly calls `setValuePosition()`
  - Verify mutations execute correctly
  - Verify command properly handles valuePosition changes
  - **Verification**: Service integration tests pass

- [x] **4.4 Performance verification**
  - Verify no performance degradation with large datasets
  - Check memory usage is acceptable
  - **Verification**: No significant regression in performance tests

## 5. Documentation & Cleanup

- [x] **5.1 Add code comments**
  - Document valuePosition behavior in key methods
  - Explain row/column swapping logic
  - **Verification**: Code review confirms clarity

- [x] **5.2 Update JSDoc comments**
  - Ensure public API documentation is complete
  - Clarify valuePosition semantics
  - **Verification**: No TSDoc errors

- [x] **5.3 Verify no lint errors**
  - Run ESLint on modified files
  - Fix any linting issues
  - **Verification**: `pnpm lint` passes

## Success Criteria

✓ All test cases pass
✓ valuePosition changes trigger recalculation
✓ Output structure correctly reflects valuePosition setting
✓ No regression in existing functionality
✓ Example application works with both ROW and COLUMN positioning
✓ No console errors or warnings
