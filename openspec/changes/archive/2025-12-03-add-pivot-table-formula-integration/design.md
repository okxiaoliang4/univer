# Design: Pivot Table Formula Integration

## Context

This design document captures architectural decisions for integrating pivot table output values into the formula calculation chain using the Feature Calculation mechanism.

## Decision Drivers

1. **Formula correctness**: Formulas referencing pivot output cells must calculate correctly
2. **No model pollution**: Pivot output should not be written to worksheet model (snapshot)
3. **Automatic updates**: Formula results must update when pivot data changes
4. **Performance**: No significant impact on formula calculation speed

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                            Pivot Table System                                │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌─────────────────────┐      ┌─────────────────────┐                       │
│  │  Source Range       │──────│  PivotTable Model   │                       │
│  │  (A1:D100)          │      │  - PivotEngineV2    │                       │
│  └─────────────────────┘      │  - calculatedData$  │                       │
│                               └──────────┬──────────┘                       │
│                                          │                                   │
│         ┌────────────────────────────────┼────────────────────────────────┐ │
│         │                                │                                │ │
│         ▼                                ▼                                │ │
│  ┌─────────────────────┐      ┌─────────────────────┐                     │ │
│  │  CELL_CONTENT       │      │  Feature Calculation │                     │ │
│  │  Interceptor        │      │  Registration        │                     │ │
│  │  (Rendering Layer)  │      │  (Formula Layer)     │                     │ │
│  │                     │      │                      │                     │ │
│  │  Injects:           │      │  Provides:           │                     │ │
│  │  - v (display value)│      │  - runtimeCellData   │                     │ │
│  │  - isPivotOutput    │      │  - dirtyRanges       │                     │ │
│  │  - selectionProtect │      │                      │                     │ │
│  └─────────────────────┘      └─────────────────────┘                     │ │
│         │                                │                                │ │
│         ▼                                ▼                                │ │
│  ┌─────────────────────┐      ┌─────────────────────┐                     │ │
│  │  Canvas Rendering   │      │  Formula Engine      │                     │ │
│  │  (User sees values) │      │  (=C3+C3 = correct) │                     │ │
│  └─────────────────────┘      └─────────────────────┘                     │ │
│                                                                           │ │
└───────────────────────────────────────────────────────────────────────────┘ │
                                                                               │
```

## Detailed Design

### 1. Feature Calculation Registration

When a pivot table is created, register it with `IFeatureCalculationManagerService`:

```typescript
// In PivotTableFormulaController
private _registerPivotFeature(pivotTable: PivotTable): void {
    const pivotTableId = pivotTable.getId();
    const targetCellInfo = pivotTable.getTargetCellInfo();
    const featureId = `pivot-table-${pivotTableId}`;

    this._commandService.executeCommand(SetFeatureCalculationMutation.id, {
        featureId,
        calculationParam: {
            unitId: targetCellInfo.unitId,
            subUnitId: targetCellInfo.subUnitId,
            dependencyRanges: this._getOutputRangesAsUnitRange(pivotTable),
            getDirtyData: (dirtyData, runtimeData) => {
                return this._buildRuntimeCellData(pivotTable);
            },
        },
    } as ISetFeatureCalculationMutation, { onlyLocal: true });
}
```

### 2. Runtime Cell Data Building

The `getDirtyData` callback transforms pivot output into formula-compatible format:

```typescript
private _buildRuntimeCellData(pivotTable: PivotTable): {
    runtimeCellData: IRuntimeUnitDataType;
    dirtyRanges: IFeatureDirtyRangeType;
} {
    const outputCellMatrix = pivotTable.getOutputCellMatrix();
    const targetCellInfo = pivotTable.getTargetCellInfo();
    const { unitId, subUnitId, row: targetRow, col: targetCol } = targetCellInfo;

    // Build cell matrix with absolute positions
    const cellMatrix = new ObjectMatrix<Nullable<ICellData>>();
    new ObjectMatrix(outputCellMatrix).forValue((row, col, value) => {
        cellMatrix.setValue(targetRow + row, targetCol + col, value);
    });

    const outputRange = pivotTable.getOutputRange();
    const absoluteRange = {
        startRow: outputRange.startRow + targetRow,
        endRow: outputRange.endRow + targetRow,
        startColumn: outputRange.startColumn + targetCol,
        endColumn: outputRange.endColumn + targetCol,
    };

    return {
        runtimeCellData: {
            [unitId]: {
                [subUnitId]: cellMatrix,
            },
        },
        dirtyRanges: {
            [unitId]: {
                [subUnitId]: [absoluteRange],
            },
        },
    };
}
```

### 3. Formula Engine Data Access

The formula engine accesses pivot data through `getRuntimeFeatureCellValue()`:

```typescript
// In BaseReferenceObject.getCellData()
getCellData(row: number, column: number) {
    return (
        activeRuntimeData?.getValue(row, column) ||
        activeRuntimeArrayFormulaCellData?.getValue(row, column) ||
        this.getRuntimeFeatureCellValue(row, column) ||  // ← Pivot data here
        activeArrayFormulaCellData?.getValue(row, column) ||
        activeSheetData?.cellData.getValue(row, column)
    );
}

// In get-runtime-feature-cell.ts
export function getRuntimeFeatureCell(row, column, sheetId, unitId, runtimeFeatureCellData) {
    for (const featureId of Object.keys(runtimeFeatureCellData)) {
        const data = runtimeFeatureCellData[featureId]?.[unitId]?.[sheetId];
        const value = data?.getValue(row, column);
        if (value != null) {
            return value;
        }
    }
}
```

### 4. Lifecycle Management

| Event | Action |
|-------|--------|
| Pivot table created | Register feature with `SetFeatureCalculationMutation` |
| Pivot data changes | Feature's `getDirtyData` returns updated data; formula engine marks dependents dirty |
| Pivot table deleted | Unregister feature with `RemoveFeatureCalculationMutation` |
| Pivot target moves | Update feature's `dependencyRanges` |

### 5. Dependency Graph

```
                     ┌─────────────────┐
                     │ Source Range    │
                     │ (A1:D100)       │
                     └────────┬────────┘
                              │ depends on
                              ▼
                     ┌─────────────────┐
                     │ Pivot Table     │
                     │ Feature Node    │
                     │ (featureId)     │
                     └────────┬────────┘
                              │ produces dirty range
                              ▼
                     ┌─────────────────┐
                     │ Formulas        │
                     │ referencing     │
                     │ pivot output    │
                     │ (=C3+C3)        │
                     └─────────────────┘
```

## Risks and Mitigations

### Risk 1: Circular Dependency

**Scenario**: Formula in source range references pivot output.

**Mitigation**: Formula engine's dependency graph handles this naturally. The pivot feature node depends only on source range, not output. If a formula in source references output, it creates a valid dependency chain, not a cycle.

### Risk 2: Performance with Large Pivot Tables

**Scenario**: Pivot table with 10,000+ output cells.

**Mitigation**:
- `getDirtyData` returns pre-calculated data from `getOutputCellMatrix()` (already computed)
- `ObjectMatrix` provides O(1) cell lookup
- Runtime data is cleared after calculation completes

### Risk 3: Stale Data

**Scenario**: Pivot data changes but formulas show old values.

**Mitigation**:
- `getDirtyData` callback is invoked every calculation cycle
- Feature's `dirtyRanges` marks all dependent formulas for recalculation

## Implementation Sequence

1. **Create `PivotTableFormulaController`** - Manages feature registration lifecycle
2. **Register controller in plugin** - Wire up DI
3. **Test formula integration** - Verify `=C3+C3` works correctly
4. **Handle pivot lifecycle events** - Create, update, delete
5. **Verify performance** - Measure impact on formula calculation

## Decision Record

| Decision | Rationale |
|----------|-----------|
| Use Feature Calculation over SetRangeValuesMutation | Avoids model pollution, no undo/redo conflicts |
| Create separate controller for formula integration | Separation of concerns from permission controller |
| Return full output matrix in getDirtyData | Simple implementation; performance acceptable for typical pivot sizes |
| Continue using CELL_CONTENT interceptor for rendering | Interceptor already handles display values correctly |

