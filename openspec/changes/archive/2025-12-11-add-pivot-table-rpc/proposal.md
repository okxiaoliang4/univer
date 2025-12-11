## Why

When using Univer with Web Workers (RPC), the pivot table plugin throws a `DataCloneError` because `SetFeatureCalculationMutation` contains a function (`getDirtyData`) that cannot be serialized through `postMessage`. This prevents the pivot table formula integration from working in distributed computing environments.

The error occurs because:
1. `PivotTableFormulaController` registers pivot tables as formula features with a callback function (`getDirtyData`)
2. When executing `SetFeatureCalculationMutation`, Worker's `DataSyncReplicaController` tries to sync it back to main thread
3. `DataSyncReplicaController` syncs ALL mutations back to main thread (unlike `DataSyncPrimaryController` which filters mutations)
4. When the mutation containing a function is passed to `postMessage`, it throws `DataCloneError`

**Root Cause Analysis**:
- The mutation `SetFeatureCalculationMutation` is designed for inter-thread communication
- Its comment states: "It requires setting local to true during execution"
- However, the key insight is that **this mutation is unnecessary** because:
  - `FeatureCalculationManagerService.register()` already triggers `onChanged$` event
  - `SetDependencyController` listens to `onChanged$` and updates dependency tree
  - The mutation was only providing redundant communication that gets blocked by RPC

## What Changes

### Primary Fix (Implemented)
Remove execution of `SetFeatureCalculationMutation` and `RemoveFeatureCalculationMutation` in `PivotTableFormulaController`:
- The `register()` and `remove()` methods on `FeatureCalculationManagerService` already trigger `onChanged$` events
- `SetDependencyController` listens to `onChanged$` to update dependency trees
- The mutations were redundant and caused serialization issues in RPC environment

### Configuration Support (Previously Implemented)
- `notExecuteFormula` configuration option in `sheets-pivot-table` plugin
- Conditionally register `PivotTableFormulaController` based on environment

## Impact

- **Affected specs**: `sheets-pivot-table`
- **Affected code**:
  - `packages/sheets-pivot-table/src/controllers/pivot-table-formula.controller.ts` - Remove mutation executions
  - `packages/sheets-pivot-table/src/plugin.ts` - Conditional controller registration
  - `packages/sheets-pivot-table/src/controllers/config.schema.ts` - Configuration option

### Dependencies

- No new dependencies required
- Follows existing patterns from `@univerjs/sheets-formula` plugin

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| Other plugins may depend on listening to these mutations | Code review shows only `SetFeatureCalculationController` and `SetDependencyController` listen to these mutations, and both get updates via `onChanged$` |
| Breaking change for existing users | Default value `false` for `notExecuteFormula` preserves existing behavior |

## Implementation Summary

The fix is surprisingly simple - just remove the redundant mutation executions:

```typescript
// Before (causing DataCloneError):
this._featureCalculationManagerService.register(...);
this._commandService.executeCommand(SetFeatureCalculationMutation.id, {...});

// After (working correctly):
this._featureCalculationManagerService.register(...);
// The register() method already triggers onChanged$ which notifies SetDependencyController
```

This works because:
1. `register()` calls `this._onChanged$.next(...)` internally
2. `SetDependencyController` subscribes to `onChanged$` and handles dependency updates
3. The mutation was just redundant communication that failed in RPC environment
