## Why

The current pivot table output rendering implementation has the following issues:
1. **Performance Issue**: `isPivotOutputCell` iterates through all pivot tables and calculates ranges, causing noticeable lag in interceptors during rendering
2. **Data Residue Issue**: When output range shrinks, old areas still display intercepted values, causing data confusion
3. **Missing Styles**: Pivot table output has no styling, making it difficult to visually distinguish multi-level headers, data areas, and total areas

## What Changes

### 1. Performance Optimization - RTree Spatial Index
- Add `PivotTableRangeService` using RTree to manage pivot table output ranges
- Optimize range detection from O(n*m) to O(log n) spatial queries
- Reference RTree usage patterns from `sheets-table` package

### 2. Output Range Management - Clearing Mechanism
- Before each pivot table output, clear the target area using `SetRangeValuesMutation`
- When output range changes, clear cells in old range that are no longer in the new range
- Ensure formula references return empty values when range shrinks (expected behavior)

### 3. Style Injection - Green Theme
- Inject styles via interceptors (non-persistent)
- Green color scheme:
  - Header area: Dark green background (#2E7D32)
  - Data area: Alternating light green (#E8F5E9 / #C8E6C9)
  - Subtotal/Total area: Medium green (#81C784)
- Decrease color depth by level

### 4. Code Refactoring
- **BREAKING**: Split `PivotTablePermissionController` into multiple single-responsibility files
- Add `PivotTableRangeService` - Range management and spatial indexing
- Add `PivotTableStyleService` - Style calculation
- Add `PivotTableRenderController` - Render interceptor
- Keep `PivotTablePermissionController` - Only handles permission checks

### 5. Architecture Layering Rules
- **@sheets-pivot-table package**: Contains only model layer and business logic
  - `PivotTableRangeService` (spatial indexing)
  - `PivotTablePermissionController` (permission checks)
- **@sheets-pivot-table-ui package**: Contains rendering and style-related code
  - `PivotTableStyleService` (style calculation)
  - `PivotTableRenderController` (render interceptor)

## Impact

- Affected specs: `specs/sheets-pivot-table/spec.md`
- Affected code:
  - **@sheets-pivot-table package**:
    - `src/controllers/pivot-table-permission.controller.ts` (refactor and split)
    - `src/services/pivot-table.service.ts` (remove isPivotOutputCell)
    - `src/controllers/pivot-table-formula.controller.ts` (adapt to new range management)
    - `src/models/pivot-table.ts` (fix getOutputCellMatrix offset issue)
    - `src/services/pivot-table-range.service.ts` (new - spatial indexing)
  - **@sheets-pivot-table-ui package**:
    - `src/services/pivot-table-style.service.ts` (new - style calculation)
    - `src/controllers/pivot-table-render.controller.ts` (new - render interceptor)
