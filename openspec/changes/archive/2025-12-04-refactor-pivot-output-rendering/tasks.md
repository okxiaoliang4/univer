## 1. Core Package (@sheets-pivot-table) Refactoring

- [x] 1.1 Create `PivotTableRangeService` spatial indexing service
  - Use RTree to manage pivot table output ranges
  - Implement `registerPivotRange`, `updatePivotRange`, `removePivotRange`
  - Implement O(log n) `isPivotOutputCell` and `getPivotTableIdByCell`
  - Listen to `pivotTableAdded$`, `pivotTableRemoved$`, `tableRangeChanged$` events
  - Provide public API for UI package to use

- [x] 1.2 Fix `PivotTable.getOutputCellMatrix()` offset issue
  - Remove `_moveMatrix` call, return relative position matrix
  - Add `getAbsoluteOutputRange()` method to return absolute range
  - Update all call sites to adapt to new interface

- [x] 1.3 Refactor `PivotTablePermissionController`
  - Remove rendering-related code
  - Keep only permission check logic
  - Use `PivotTableRangeService` to replace `isPivotOutputCell`

## 2. UI Package (@sheets-pivot-table-ui) Refactoring

- [x] 2.1 Create `PivotTableStyleService` style calculation service
  - Define green theme palette constants
  - Implement `getCellStyle()` to return styles based on cell type
  - Support multi-level header color graduation
  - Calculate zebra stripe styles

- [x] 2.2 Create `PivotTableRenderController` render interceptor
  - Inject pivot table values and styles logic
  - Depend on `PivotTableRangeService` for range detection
  - Inject pivot table values (correctly access matrix using relative positions)
  - Inject pivot table styles (call `PivotTableStyleService`)

## 3. Output Range Clearing Mechanism

- [x] 3.1 Implement pre-output clearing logic
  - Trigger on `calculatedData$` changes
  - Calculate difference between old and new ranges
  - Execute `SetRangeValuesMutation` to clear target area
  - Use `{ onlyLocal: true }` to avoid broadcasting

- [x] 3.2 Update RTree index synchronization
  - Update RTree after range changes
  - Ensure index consistency with actual output ranges

## 4. Formula Integration Adaptation

- [x] 4.1 Update `PivotTableFormulaController`
  - Use `getAbsoluteOutputRange()` to get absolute range
  - Fix position conversion in `_buildRuntimeCellData`
  - Ensure formula references return empty values when range shrinks

## 5. Plugin Registration and Dependency Injection

- [x] 5.1 Update core package `plugin.ts` to register business logic services
  - Register `PivotTableRangeService`
  - Adjust controller initialization order

- [x] 5.2 Update UI package `plugin.ts` to register rendering services
  - Register `PivotTableStyleService`
  - Register `PivotTableRenderController`

## 6. Testing and Documentation

- [ ] 6.1 Add unit tests
  - `PivotTableRangeService` RTree operation tests
  - `PivotTableStyleService` style calculation tests
  - Render interceptor integration tests
  - Range shrink clearing tests

- [x] 6.2 Update code comments
  - Add JSDoc comments for new services
  - Update existing controller comments
  - Add architecture explanation comments
