# Tasks: Pivot Table Formula Integration

## Overview

Implementation tasks for integrating pivot table output into the formula calculation chain.

## Phase 1: Core Implementation

- [x] **1.1 Create PivotTableFormulaController**
  - Create `packages/sheets-pivot-table/src/controllers/pivot-table-formula.controller.ts`
  - Inject `IFeatureCalculationManagerService`, `ICommandService`, `ISheetsPivotTableService`
  - Implement basic controller structure with lifecycle management
  - **Verification**: Controller compiles without errors

- [x] **1.2 Implement Feature Registration**
  - Add `_registerPivotFeature(pivotTable: PivotTable)` method
  - Build `IFeatureCalculationManagerParam` with `getDirtyData` callback
  - Execute `SetFeatureCalculationMutation` on pivot table creation
  - **Verification**: Feature registered in `IFeatureCalculationManagerService` after pivot creation

- [x] **1.3 Implement Runtime Cell Data Builder**
  - Create `_buildRuntimeCellData(pivotTable: PivotTable)` method
  - Transform `getOutputCellMatrix()` to absolute positions
  - Return `{ runtimeCellData, dirtyRanges }` structure
  - **Verification**: Returned data matches expected format from `IFeatureCalculationManagerParam`

- [x] **1.4 Implement Feature Unregistration**
  - Add `_unregisterPivotFeature(pivotTableId: string)` method
  - Execute `RemoveFeatureCalculationMutation` on pivot table deletion
  - Handle cleanup of all features when controller is disposed
  - **Verification**: Feature removed from manager after pivot deletion

## Phase 2: Lifecycle Integration

- [x] **2.1 Register Controller in Plugin**
  - Add `PivotTableFormulaController` to `@sheets-pivot-table` plugin dependencies
  - Initialize controller in `onStarting()` lifecycle hook
  - **Verification**: Controller instantiated when plugin loads

- [x] **2.2 Subscribe to Pivot Table Events**
  - Listen to `pivotTableAdded$` for new pivot tables
  - Listen to pivot table deletion events
  - Register/unregister features accordingly
  - **Verification**: Features registered/unregistered on pivot lifecycle events

- [x] **2.3 Handle Pivot Data Updates**
  - Verify `getDirtyData` is called when source data changes
  - Ensure formula engine receives updated pivot values
  - **Verification**: Formulas recalculate when pivot source data changes

- [x] **2.4 Handle Pivot Range Changes**
  - Update feature's `dependencyRanges` when pivot target moves
  - Update dirty ranges when pivot output size changes
  - **Verification**: Feature ranges updated when pivot configuration changes

## Phase 3: Testing

- [x] **3.1 Unit Tests for Runtime Data Builder**
  - Test `_buildRuntimeCellData` with various pivot output sizes
  - Test absolute position calculation
  - Test empty pivot output handling
  - **Verification**: All unit tests pass

- [x] **3.2 Integration Tests for Formula Calculation**
  - Test simple formula `=C3` referencing pivot output
  - Test formula with operations `=C3+D3`
  - Test formula referencing multiple pivot tables
  - Test formula in different worksheet referencing pivot output
  - **Verification**: All formulas calculate correctly

- [x] **3.3 Lifecycle Tests**
  - Test feature registration on pivot creation
  - Test feature unregistration on pivot deletion
  - Test feature update on pivot configuration change
  - **Verification**: Features correctly managed through pivot lifecycle

- [ ] **3.4 Performance Tests**
  - Test formula calculation with large pivot (1000+ output cells)
  - Measure calculation time impact
  - **Verification**: No significant performance regression

## Phase 4: Documentation

- [x] **4.1 Code Comments**
  - Add JSDoc comments to `PivotTableFormulaController`
  - Document `_buildRuntimeCellData` algorithm
  - Explain feature calculation lifecycle
  - **Verification**: All public methods have JSDoc comments

- [x] **4.2 Update Spec**
  - Update `openspec/specs/sheets-pivot-table/spec.md` with new requirement
  - Add scenarios for formula integration
  - **Verification**: `openspec validate --strict` passes

## Dependencies

- Phase 1.1 → Phase 1.2, 1.3, 1.4
- Phase 1.2 + 1.3 → Phase 2.2
- Phase 1.4 → Phase 2.2
- Phase 2.1 → Phase 2.2, 2.3, 2.4
- Phase 2.* → Phase 3.*
- Phase 3.* → Phase 4.*

## Notes

- Feature calculation mechanism is already battle-tested in formula engine
- No changes needed to formula engine itself
- CELL_CONTENT interceptor continues to handle rendering (no change)
- Permission controller remains unchanged

