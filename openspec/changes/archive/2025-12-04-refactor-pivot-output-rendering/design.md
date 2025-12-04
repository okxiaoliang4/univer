## Context

Pivot table output rendering needs to solve three core problems: performance, data consistency, and visual styling. The current implementation iterates through all pivot tables to check if cells are within output ranges during each render, resulting in O(n*m) complexity, where n is the number of pivot tables and m is the output range size.

### Constraints
- Styles cannot be persisted to documents (injected dynamically via interceptors)
- Output values are not persisted (provided via interceptors and formula engine features)
- Must be compatible with formula engine Feature Calculation mechanism
- Must support multiple pivot tables on the same worksheet

## Goals / Non-Goals

### Goals
1. Optimize cell range queries to O(log n) complexity
2. Ensure data consistency when output ranges change
3. Provide clear multi-level table visual styling
4. Maintain single-responsibility code that is easy to maintain

### Non-Goals
- User-customizable pivot table themes not supported (consider for future versions)
- Style persistence not supported
- PivotEngineV2 calculation logic will not be modified

## Decisions

### Decision 1: Use RTree Spatial Index to Optimize Range Queries

**What**: Create `PivotTableRangeService` using `@univerjs/core`'s RTree class to manage pivot table output ranges.

**Why**:
- RTree provides O(log n) range query performance
- Mature implementation already exists in `sheets-table` package
- Supports dynamic insert/delete/update

**Implementation**:
```typescript
// PivotTableRangeService
class PivotTableRangeService {
  private _rTree = new Map<string, RTree>(); // unitId -> RTree

  // Register pivot table range
  registerPivotRange(unitId: string, subUnitId: string, pivotTableId: string, range: IRange): void;

  // Update pivot table range (remove old, insert new)
  updatePivotRange(unitId: string, subUnitId: string, pivotTableId: string, oldRange: IRange, newRange: IRange): void;

  // Remove pivot table range
  removePivotRange(unitId: string, subUnitId: string, pivotTableId: string, range: IRange): void;

  // O(log n) query: Check if cell is within any pivot table output range
  isPivotOutputCell(unitId: string, subUnitId: string, row: number, col: number): boolean;

  // O(log n) query: Get pivot table ID containing this cell
  getPivotTableIdByCell(unitId: string, subUnitId: string, row: number, col: number): string | null;
}
```

### Decision 2: Clear Target Area Before Output

**What**: Execute `SetRangeValuesMutation` to clear the target area before pivot table calculation completes and rendering begins.

**Why**:
- Ensures old data does not persist
- Formula references to cells outside the shrunk range return empty values
- Compatible with existing rendering mechanism

**Implementation Flow**:
```
calculatedData$ changes
  ↓
Compare old and new outputRange
  ↓
[New range] Execute SetRangeValuesMutation to clear new range
  ↓
[Range shrinks] Calculate difference, clear cells no longer in output
  ↓
Update RTree index
  ↓
Trigger rendering (interceptor injects values and styles)
```

### Decision 3: Fix getOutputCellMatrix Offset Issue

**What**: `getOutputCellMatrix` should return a relative-position (0-indexed) matrix, not an absolute position after applying targetCell offset.

**Why**:
- Current implementation `_moveMatrix` offsets matrix to absolute position
- But `isPivotOutputCell` and interceptors subtract offset when accessing
- Causes double offset or index errors

**Fix**:
```typescript
// Before (incorrect):
getOutputCellMatrix(): IObjectMatrixPrimitiveType<Nullable<ICellData>> {
  return this._moveMatrix(targetMatrix, this._targetCellInfo).getMatrix();
}

// After (correct):
getOutputCellMatrix(): IObjectMatrixPrimitiveType<Nullable<ICellData>> {
  return targetMatrix; // Return relative position
}

// Convert separately where absolute position is needed
getAbsoluteOutputRange(): IRange {
  const relativeRange = new ObjectMatrix(this.getOutputCellMatrix()).getDataRange();
  return {
    startRow: relativeRange.startRow + this._targetCellInfo.row,
    endRow: relativeRange.endRow + this._targetCellInfo.row,
    startColumn: relativeRange.startColumn + this._targetCellInfo.col,
    endColumn: relativeRange.endColumn + this._targetCellInfo.col,
  };
}
```

### Decision 4: Green Theme Style Scheme

**What**: Use Material Design green palette for green-themed styles.

**Palette**:
| Area | Color Code | Purpose |
|------|-----------|---------|
| Header background | #2E7D32 (Green 800) | Row/column header area |
| Header font | #FFFFFF | White text |
| Data area odd rows | #E8F5E9 (Green 50) | Zebra stripe light |
| Data area even rows | #C8E6C9 (Green 100) | Zebra stripe dark |
| Subtotal rows | #A5D6A7 (Green 200) | Group subtotals |
| Grand total rows | #81C784 (Green 300) | Final totals |
| Border | #4CAF50 (Green 500) | Table borders |

**Style Service Interface**:
```typescript
interface IPivotCellStyle {
  bg: { rgb: string };      // Background color
  cl?: { rgb: string };     // Font color
  bl?: BooleanNumber;       // Bold
  bd?: IBorderData;         // Border
}

class PivotTableStyleService {
  // Calculate style based on cell position and type
  getCellStyle(
    pivotTableId: string,
    row: number,
    col: number,
    cellType: 'header' | 'rowHeader' | 'columnHeader' | 'data' | 'subtotal' | 'grandTotal',
    level: number
  ): IPivotCellStyle;
}
```

### Decision 5: Package Architecture Separation

**What**: Rendering-related code (`PivotTableStyleService`, `PivotTableRenderController`) goes in `@sheets-pivot-table-ui` package, while model/business logic (`PivotTableRangeService`) stays in `@sheets-pivot-table` package.

**Why**:
- Clear separation of concerns: UI package handles presentation, core package handles data/business logic
- Better dependency management: UI package can depend on core, but not vice versa
- Consistent with existing architecture patterns in the codebase

**Implementation Structure**:
```
packages/sheets-pivot-table/src/          # Core business logic package
├── controllers/
│   └── pivot-table-permission.controller.ts  # Permission checks (refactored)
├── services/
│   ├── pivot-table.service.ts                # Core service (modified)
│   └── pivot-table-range.service.ts          # Range management (new)
└── ...

packages/sheets-pivot-table-ui/src/       # UI rendering package
├── controllers/
│   └── pivot-table-render.controller.ts      # Render interceptor (new)
└── services/
    └── pivot-table-style.service.ts          # Style calculation (new)
```

**Cross-Package Communication**:
- UI package imports `PivotTableRangeService` from core package
- Core package provides events/hooks for UI package to react to
- Dependency injection handles inter-package communication

## Risks / Trade-offs

### Risk 1: RTree Memory Overhead
- **Risk**: Each pivot table needs to store range information in RTree
- **Mitigation**: RTree implementation is optimized, memory overhead is acceptable; and pivot table count is usually limited

### Risk 2: Clear Operations Trigger Unnecessary Recalculation
- **Risk**: `SetRangeValuesMutation` may trigger formula recalculation
- **Mitigation**: Use `{ onlyLocal: true }` option to avoid broadcasting; clear operations complete before interceptor injection

### Risk 3: Style Conflicts with Other Interceptors
- **Risk**: Conditional formatting or other style interceptors may override pivot table styles
- **Mitigation**: Set appropriate priority (900), lower than protection permissions (999) but higher than general styles

## Migration Plan

1. **Phase 1**: Create `PivotTableRangeService` and `PivotTableStyleService`
2. **Phase 2**: Modify `PivotTable.getOutputCellMatrix()` to return relative position
3. **Phase 3**: Refactor `PivotTablePermissionController`, split rendering logic to `PivotTableRenderController`
4. **Phase 4**: Implement clearing mechanism and style injection
5. **Phase 5**: Update unit tests

**Rollback**: Since this is an internal refactoring, rollback only requires restoring files and does not affect data format.

## Open Questions

1. ~~Base color selection~~ → Confirmed to use green theme
2. ~~Style persistence~~ → Confirmed not to persist
3. ~~Formula return value~~ → Confirmed to return empty value outside range
