## Context

The Univer framework supports running computation-intensive operations in Web Workers to avoid blocking the main thread. This is achieved through the `@univerjs/rpc` package which provides a data synchronization mechanism between the main thread and Worker.

### RPC Mutation Sync Architecture

```
Main Thread                                  Worker Thread
     │                                            │
     │  DataSyncPrimaryController                 │  DataSyncReplicaController
     │  ├─► Filters mutations by _syncingMutations│  └─► Syncs ALL mutations back
     │  └─► Only registered mutations are synced  │      (no filtering!)
     │                                            │
     │           registerSyncingMutations()       │
     │           ─────────────────────────►       │
     │           syncMutation()                   │
     │           ─────────────────────────►       │
     │                                            │
     │           syncMutation() ALL mutations     │
     │           ◄─────────────────────────       │
```

### Problem Discovery

When `PivotTableFormulaController` executes `SetFeatureCalculationMutation` in the Worker thread, `DataSyncReplicaController` attempts to sync it back to the main thread. Since the mutation payload contains a function (`getDirtyData`), `postMessage` throws `DataCloneError`.

### Key Architectural Insight

**The mutation was unnecessary!** Analysis of `FeatureCalculationManagerService` revealed:

```typescript
// FeatureCalculationManagerService.register()
register(unitId, subUnitId, featureId, referenceExecutor) {
    // ...setup...

    // This triggers dependency updates via observable
    this._onChanged$.next({ unitId, subUnitId, featureIds: [featureId] });

    subUnitMap.set(featureId, referenceExecutor);
}
```

And `SetDependencyController` subscribes to this:

```typescript
// SetDependencyController
this._featureCalculationManagerService.onChanged$.subscribe((params) => {
    this._dependencyManagerService.removeFeatureFormulaDependency(...);
});
```

## Design Decision

**Remove redundant mutation execution** instead of adding workarounds like `onlyLocal: true`.

### Alternatives Considered

1. **Add `onlyLocal: true` to mutation execution**
   - ❌ Doesn't work because `DataSyncReplicaController` doesn't check `onlyLocal`
   - Would require modifying RPC core code

2. **Modify `DataSyncReplicaController` to check `onlyLocal`**
   - ❌ Core infrastructure change with broader implications
   - Risk of breaking other plugins

3. **Remove mutation execution entirely** ✅
   - ✅ No infrastructure changes needed
   - ✅ The `register()` method already provides the same functionality
   - ✅ Simpler and more correct solution

## Implementation

### Before (Problematic)

```typescript
// _registerPivotFeature
this._featureCalculationManagerService.register(unitId, subUnitId, pivotTableId, calculationParam);

// This causes DataCloneError in RPC environment
this._commandService.executeCommand(SetFeatureCalculationMutation.id, {
    featureId: pivotTableId,
    calculationParam,  // Contains getDirtyData function!
});
```

### After (Fixed)

```typescript
// _registerPivotFeature
// The register() method already triggers onChanged$ which notifies SetDependencyController
this._featureCalculationManagerService.register(unitId, subUnitId, pivotTableId, calculationParam);
// No mutation execution needed!
```

## Configuration Support

The `notExecuteFormula` configuration option allows fine-grained control:

- **Main thread (RPC)**: `notExecuteFormula: true` - Don't initialize `PivotTableFormulaController`
- **Worker thread**: `notExecuteFormula: false` (default) - Initialize `PivotTableFormulaController`

This follows the same pattern as `@univerjs/sheets-formula` plugin.

## Data Flow

### Formula Feature Registration (Existing)

```
Worker Thread:
┌─────────────────────────────────────────────────────┐
│ PivotTableFormulaController                         │
│   └── _registerPivotFeature()                       │
│         └── featureCalculationManagerService        │
│               .register()                           │
│                   └── onChanged$.next()             │
│                         └── SetDependencyController │
│                               └── Update dependency │
└─────────────────────────────────────────────────────┘
                      │
                      │ Formula calculation
                      │ results synced via
                      │ SetFormulaCalculationResultMutation
                      ▼
Main Thread:
┌─────────────────────────────────────────────────────┐
│ Render pivot table output via SheetInterceptorService│
│   └── PivotTableRenderController injects cell data  │
└─────────────────────────────────────────────────────┘
```

### Pivot Table Calculated Data Sync (New)

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
                      │ DataSyncReplicaController syncs
                      │ SetPivotTableCalculatedDataMutation via RPC
                      ▼
Main Thread:
┌─────────────────────────────────────────────────────────────────┐
│ SetPivotTableCalculatedDataMutation handler                     │
│   └── SheetsPivotDataSourceModel.setCalculatedData()            │
│         └── PivotTable.setCalculatedData() - Direct update      │
│               └── _calculatedData$.next() - Notify observers    │
│                     └── Render controller updates display       │
└─────────────────────────────────────────────────────────────────┘

Note: Main thread PivotTable is created with skipAutoCalculation: true,
so _initCalculatedDataListener() is NOT initialized, preventing
redundant calculations.
```

## Key Design Decisions

### 1. skipAutoCalculation Option

The `PivotTable` model accepts an `IPivotTableOptions` with `skipAutoCalculation`:
- **Worker thread (default)**: `skipAutoCalculation: false` - auto-calculates
- **Main thread (RPC)**: `skipAutoCalculation: true` - receives data via mutation

This ensures only ONE thread performs calculations, avoiding redundant work.

### 2. SetPivotTableCalculatedDataMutation

New mutation for syncing calculated data:
- Contains only serializable data (IObjectMatrixPrimitiveType)
- No functions or callbacks - safe for RPC
- Synced from Worker to Main thread via DataSyncReplicaController
- **Critical**: The mutation handler only updates the model on Main thread (`notExecuteFormula: true`)
- On Worker thread, the handler is a no-op to prevent infinite loops (data already correct from auto-calculation)

### 3. Configuration-Driven Behavior

The `notExecuteFormula` config controls:
1. Whether `PivotTableFormulaController` is registered (formula integration)
2. Whether `PivotTable` instances skip auto-calculation

Both are controlled by the same config for consistency.

### 4. Preventing Infinite Loops

The mutation handler checks `notExecuteFormula` config to determine which thread it's on:

```typescript
// SetPivotTableCalculatedDataMutation handler
const isMainThread = pluginConfig?.notExecuteFormula ?? false;
if (!isMainThread) {
    return true; // No-op on Worker thread
}
// Only update model on Main thread
dataSourceModel.setCalculatedData(...);
```

This prevents the following infinite loop on Worker thread:
1. `_initCalculatedDataListener` calculates and emits to `calculatedData$`
2. `_listenToPivotTableDataChanges` catches it and executes mutation
3. If mutation handler updated model, it would emit to `calculatedData$` again
4. This would trigger step 2 again → infinite loop!

By making the mutation handler a no-op on Worker thread, the loop is broken.

## Testing Strategy

1. **Unit test**: Verify `register()` triggers `onChanged$`
2. **Integration test**: Verify pivot table works in RPC environment without DataCloneError
3. **Manual test**: Verify pivot table renders correctly and formulas calculate properly
4. **RPC test**: Verify Main thread doesn't calculate when `notExecuteFormula: true`
5. **Sync test**: Verify `SetPivotTableCalculatedDataMutation` syncs data correctly
6. **Loop test**: Verify no infinite loop when pivot table calculates on Worker thread
