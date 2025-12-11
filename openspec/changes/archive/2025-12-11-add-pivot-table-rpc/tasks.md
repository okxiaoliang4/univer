## 1. Primary Fix (Required)

- [x] **1.1 Remove `SetFeatureCalculationMutation` execution**
  - Modified `pivot-table-formula.controller.ts` `_registerPivotFeature` method
  - Removed `executeCommand(SetFeatureCalculationMutation.id, ...)` call
  - Added detailed comment explaining why mutation execution is unnecessary
  - **Verification**: No DataCloneError when pivot table is created in RPC environment

- [x] **1.2 Remove `RemoveFeatureCalculationMutation` execution**
  - Modified `pivot-table-formula.controller.ts` `_unregisterPivotFeature` method
  - Removed `executeCommand(RemoveFeatureCalculationMutation.id, ...)` call
  - Added comment explaining the design rationale
  - **Verification**: No DataCloneError when pivot table is deleted in RPC environment

- [x] **1.3 Clean up unused imports**
  - Removed `SetFeatureCalculationMutation` and `RemoveFeatureCalculationMutation` imports
  - **Verification**: No linter errors

## 2. Configuration Support (Previously Implemented)

- [x] **2.1 Add `notExecuteFormula` configuration option**
  - Added to `packages/sheets-pivot-table/src/controllers/config.schema.ts`
  - Default value: `false` (preserves existing behavior)

- [x] **2.2 Conditional controller registration**
  - Modified `packages/sheets-pivot-table/src/plugin.ts`
  - `PivotTableFormulaController` only registered when `notExecuteFormula: false`

## 3. Worker-Side Calculation Sync (New)

- [x] **3.1 Add `SetPivotTableCalculatedDataMutation`**
  - Added to `packages/sheets-pivot-table/src/commands/mutations/pivot-table.mutation.ts`
  - Mutation syncs calculated data from Worker thread to Main thread via RPC
  - Contains serializable data (IObjectMatrixPrimitiveType) that can pass through postMessage
  - **Key design**: Worker calculates, Main receives via mutation

- [x] **3.2 Add `skipAutoCalculation` option to PivotTable model**
  - Modified `packages/sheets-pivot-table/src/models/pivot-table.ts`
  - Added `IPivotTableOptions` interface with `skipAutoCalculation` option
  - When `skipAutoCalculation: true`, the `_initCalculatedDataListener` is not initialized
  - Added `setCalculatedData()` method for directly setting calculated data
  - This ensures Main thread doesn't perform redundant calculations

- [x] **3.3 Update SheetsPivotDataSourceModel for RPC support**
  - Modified `packages/sheets-pivot-table/src/models/sheets-pivot-data-source-model.ts`
  - Added `_shouldSkipAutoCalculation()` private method to check `notExecuteFormula` config
  - Added `setCalculatedData()` method to receive data from mutation
  - Updated `fromJSON()` to pass `skipAutoCalculation` option when creating PivotTable instances

- [x] **3.4 Update AddPivotTableMutation for RPC support**
  - Modified `packages/sheets-pivot-table/src/commands/mutations/pivot-table.mutation.ts`
  - Now checks `notExecuteFormula` config and passes `skipAutoCalculation` option

- [x] **3.5 Update PivotTableFormulaController to emit sync mutation**
  - Modified `packages/sheets-pivot-table/src/controllers/pivot-table-formula.controller.ts`
  - In `_listenToPivotTableDataChanges`, now executes `SetPivotTableCalculatedDataMutation`
  - Worker thread calculates and emits mutation, Main thread receives via RPC

- [x] **3.6 Register and export new mutation**
  - Modified `packages/sheets-pivot-table/src/plugin.ts`
  - Registered `SetPivotTableCalculatedDataMutation` in plugin
  - Exported mutation and type for external use

## 4. Example Configuration

- [x] **4.1 Main thread configuration**
  ```typescript
  // main.tsx
  univer.registerPlugin(UniverSheetsFormulaPlugin, { notExecuteFormula: true });
  univer.registerPlugin(UniverSheetsPivotTablePlugin, { notExecuteFormula: true });
  ```

- [x] **4.2 Worker thread configuration**
  ```typescript
  // worker.ts
  univer.registerPlugin(UniverRemoteSheetsFormulaPlugin);
  univer.registerPlugin(UniverSheetsPivotTablePlugin); // notExecuteFormula: false (default)
  ```

## 5. Testing

- [ ] **5.1 Verify no duplicate calculations**
  - Main thread should NOT calculate pivot data when `notExecuteFormula: true`
  - Only Worker thread performs calculations
  - **Verification**: No console log from `_initCalculatedDataListener` on Main thread

- [ ] **5.2 Verify mutation sync**
  - Worker thread calculates and emits `SetPivotTableCalculatedDataMutation`
  - Main thread receives mutation via RPC and updates model
  - **Verification**: Pivot table renders correctly on Main thread

- [ ] **5.3 Verify no DataCloneError**
  - All mutations should be serializable
  - **Verification**: No errors in console during pivot table operations

## Technical Notes

### Why removing mutation execution works

The `SetFeatureCalculationMutation` was redundant because:

1. **`FeatureCalculationManagerService.register()`** already triggers `onChanged$`:
   ```typescript
   register(unitId, subUnitId, featureId, referenceExecutor) {
       // ...
       this._onChanged$.next({ unitId, subUnitId, featureIds: [featureId] });
       // ...
   }
   ```

2. **`SetDependencyController`** subscribes to `onChanged$`:
   ```typescript
   this._featureCalculationManagerService.onChanged$.subscribe((params) => {
       const { unitId, subUnitId, featureIds } = params;
       this._dependencyManagerService.removeFeatureFormulaDependency(unitId, subUnitId, featureIds);
   });
   ```

3. The mutation was only duplicating the work that `register()` already does via `onChanged$`.

### RPC Architecture Insight

- **Main → Worker**: `DataSyncPrimaryController` filters mutations by `_syncingMutations`
- **Worker → Main**: `DataSyncReplicaController` syncs ALL mutations (no filtering!)
- This asymmetry means mutations with functions can't be executed in Worker without special handling

### New Architecture for Calculated Data Sync

```
Worker Thread:
┌─────────────────────────────────────────────────────────────────┐
│ PivotTable (skipAutoCalculation: false)                         │
│   └── _initCalculatedDataListener() - Auto-calculates           │
│         └── calculatedData$ emits new data                      │
│               └── PivotTableFormulaController listens           │
│                     └── Executes SetPivotTableCalculatedDataMutation │
└─────────────────────────────────────────────────────────────────┘
                      │
                      │ DataSyncReplicaController syncs mutation via RPC
                      ▼
Main Thread:
┌─────────────────────────────────────────────────────────────────┐
│ PivotTable (skipAutoCalculation: true)                          │
│   └── NO _initCalculatedDataListener() - No auto-calculation    │
│         └── setCalculatedData() receives data from mutation     │
│               └── Renders pivot table output correctly          │
└─────────────────────────────────────────────────────────────────┘
```
