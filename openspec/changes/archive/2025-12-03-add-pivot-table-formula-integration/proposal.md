# Proposal: Add Pivot Table Formula Integration

## Problem Statement

Currently, pivot table output values are only available to the rendering layer through `CELL_CONTENT` interceptors. When a formula references a cell in the pivot table output range (e.g., `=C3+C3`), the formula engine returns 0 because:

1. **Interceptors only affect rendering**: The `CELL_CONTENT` interceptor injects values into the view model for display, but this data is not accessible to the formula engine.
2. **Formula engine reads directly from model**: The formula engine's `getCellData()` method reads from the worksheet's `cellData` matrix, which is empty for pivot output cells.
3. **No bridge between pivot calculation and formula engine**: There is currently no mechanism to make pivot table calculated values available to the formula dependency chain.

The previous implementation using `SetRangeValuesMutation` wrote values to the model, but this approach has issues:
- Values get saved to snapshot (even with `onlyLocal: true`, they persist in the model)
- Undo/redo conflicts with pivot table lifecycle
- Collaborative editing complexity

## Proposed Changes

Implement **Feature Calculation** integration to register pivot table as a formula engine feature, enabling formulas to reference pivot output cells without writing to the model.

### Solution Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         Formula Calculation Flow                         │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│   Formula "=C3+C3"  ──► ReferenceNode.execute()                         │
│                              │                                           │
│                              ▼                                           │
│                     BaseReferenceObject.getCellData(row=3, col=3)        │
│                              │                                           │
│                              ▼                                           │
│   ┌───────────────────────────────────────────────────────────────┐     │
│   │  Data Source Priority (in order):                              │     │
│   │  1. activeRuntimeData (formula runtime)                        │     │
│   │  2. activeRuntimeArrayFormulaCellData (array formula runtime)  │     │
│   │  3. getRuntimeFeatureCellValue() ◄── NEW: Pivot Table Data     │     │
│   │  4. activeArrayFormulaCellData (array formula model)           │     │
│   │  5. activeSheetData.cellData (worksheet model) ◄── Currently 0 │     │
│   └───────────────────────────────────────────────────────────────┘     │
│                              │                                           │
│                              ▼                                           │
│                     Returns pivot table value                            │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### Key Components

1. **Feature Calculation Registration**: Register pivot table as a feature in `IFeatureCalculationManagerService`
2. **Runtime Cell Data Provider**: Implement `getDirtyData()` callback to provide pivot output values
3. **Dirty Range Propagation**: Mark pivot output ranges as dirty when pivot data changes
4. **Rendering via Interceptor**: Continue using `CELL_CONTENT` interceptor for display (no change)

## Impact

### Spec Changes
- **MODIFIED**: `openspec/specs/sheets-pivot-table/spec.md` - Add new requirement for formula integration

### Code Changes
- **MODIFIED**: `packages/sheets-pivot-table/src/services/pivot-table.service.ts` - Add feature calculation registration
- **NEW**: `packages/sheets-pivot-table/src/controllers/pivot-table-formula.controller.ts` - Controller for formula integration lifecycle

### Dependencies
- `@univerjs/engine-formula`: `IFeatureCalculationManagerService`, `SetFeatureCalculationMutation`

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| Performance impact from feature calculation callback | Callback only returns pre-calculated data from `PivotTable.getOutputCellMatrix()` - O(1) lookup |
| Formula dependency cycle with pivot source range | Pivot table feature only depends on source range, not output range |
| Memory overhead from runtime cell data | Runtime data is cleared after formula calculation completes |

## Alternatives Considered

1. **Continue using SetRangeValuesMutation**: Rejected due to snapshot persistence and undo/redo issues
2. **Modify formula engine to read from interceptors**: Rejected due to performance implications and architectural violation
3. **Create custom formula function (e.g., `=PIVOT(...)`)**: Rejected as it requires users to change formulas and doesn't provide seamless integration

