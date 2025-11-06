# Technical Design

## Context

The pivot table feature requires coordination between data processing (core plugin) and user interaction (UI plugin). The implementation must follow Univer's plugin architecture patterns while providing an intuitive user experience similar to Excel or Google Sheets pivot tables.

**Reference Implementation**: The `sheets-filter` and `sheets-filter-ui` packages provide the architectural blueprint for this implementation, demonstrating the separation of concerns between core logic and UI.

**Existing Foundation**: Basic models exist in `packages/sheets-pivot-table/src/model/` including:
- `PivotTable` class with calculation logic
- `PivotField` class for field configuration
- Aggregation functions (SUM, COUNT, AVERAGE, MIN, MAX)
- Type definitions and enums

## Goals / Non-Goals

### Goals
- Provide functional MVP pivot table capability following Univer's plugin patterns
- Enable users to create and configure basic pivot tables through UI
- Support essential aggregation functions (SUM, COUNT, AVERAGE, MIN, MAX)
- Ensure proper snapshot serialization for save/load
- Follow sheets-filter architecture as reference
- Maintain isomorphic design (browser and Node.js)

### Non-Goals (for MVP)
- Advanced features (calculated fields, custom formulas, slicers)
- Multiple value field aggregations in single pivot table
- Complex filtering UI (defer to future iteration)
- Mobile UI support (desktop only for MVP)
- Cross-workbook pivot tables
- Pivot table formatting/styling customization
- Drill-down/drill-up functionality

## Decisions

### Architecture: Two-Plugin Pattern

**Decision**: Implement as two separate plugins:
- `@univerjs/sheets-pivot-table`: Core logic, data models, calculation engine
- `@univerjs/sheets-pivot-table-ui`: UI components, interactions, rendering

**Rationale**:
- Follows established Univer pattern (sheets-filter, sheets-conditional-formatting, etc.)
- Enables headless usage scenarios (server-side, API-only)
- Separates concerns: data/logic vs presentation
- Supports future mobile UI as separate plugin

**Alternatives Considered**:
- Single combined plugin: Rejected due to violation of isomorphic design principles
- Three plugins (core, desktop UI, mobile UI): Deferred until mobile support needed

### Service Layer Design

**Decision**: Implement two core services in sheets-pivot-table:
1. `PivotTableService`: Manages pivot table lifecycle (CRUD, storage, snapshot)
2. `PivotTableCalculationService`: Handles calculation and result generation

**Rationale**:
- Single Responsibility Principle: Separate lifecycle management from computation
- Calculation service can be optimized independently (e.g., Web Workers in future)
- Service layer provides clean API for UI plugin and commands
- Matches sheets-filter pattern (FilterService + FilterFormulaService)

### Command/Mutation Pattern

**Decision**: Follow CQRS-style command/mutation separation:
- **Commands** (sheets-pivot-table): High-level user actions with business logic
- **Mutations** (sheets-pivot-table): Low-level data changes for undo/redo
- **Operations** (sheets-pivot-table-ui): UI-specific actions (panel visibility, etc.)

**Rationale**:
- Enables proper undo/redo support
- Commands can trigger multiple mutations atomically
- Operations handle UI state separate from data state
- Standard Univer pattern across all plugins

### Calculation Strategy

**Decision**: Incremental calculation with dirty tracking
- Mark pivot tables dirty on source data change
- Recalculate on-demand when pivot table is visible
- Cache results until marked dirty
- Use existing aggregation models

**Rationale**:
- Avoid unnecessary recalculations
- Balance responsiveness with performance
- Existing `PivotTable` class already implements dirty tracking
- Simple for MVP, can optimize later with partial recalculation

**Alternatives Considered**:
- Real-time calculation on every change: Too expensive for large datasets
- Manual refresh only: Poor UX, users expect automatic updates

### Field Configuration Model

**Decision**: Store field configuration as arrays of field IDs in four categories:
- `rowFields: string[]`
- `columnFields: string[]`
- `valueFields: string[]`
- `filterFields: string[]`

Each field ID maps to a `PivotField` object with metadata (source column, aggregation, etc.)

**Rationale**:
- Simple data structure for MVP
- Easy to serialize/deserialize
- Matches mental model of pivot table "areas"
- Extensible for future enhancements (e.g., field ordering, grouping)

### UI Component Strategy

**Decision**: Implement React components with drag-and-drop for field configuration:
- Use `@univerjs/design` components for consistency
- Implement custom drag-drop with HTML5 drag-and-drop API
- Panel-based UI (side panel for configuration, dialog for creation)

**Rationale**:
- Drag-and-drop is intuitive and industry standard for pivot tables
- Reuses Univer design system components
- Panel approach fits Univer's UI architecture

**Alternatives Considered**:
- Form-based field configuration: Less intuitive, more clicks required
- Third-party drag-drop library: Adds dependency, HTML5 API sufficient for MVP

### Rendering Approach

**Decision**: Render pivot table results as regular worksheet cells with metadata:
- Calculate pivot table into cell data matrix
- Apply styling (bold headers, background colors for totals)
- Add metadata to cells to track pivot table association
- Render controller manages updates

**Rationale**:
- Reuses existing cell rendering infrastructure
- No custom rendering engine needed
- Copy/paste works naturally
- Print support automatic
- Follows data validation/conditional formatting patterns

**Alternatives Considered**:
- Custom canvas rendering: More complex, loses cell-level interactions
- Read-only overlay: Disconnected from worksheet, harder to implement

### Snapshot Strategy

**Decision**: Store pivot tables in workbook resources:
```typescript
{
  id: 'pivotTable-{id}',
  pivotTables: {
    [pivotTableId]: {
      id, name, sourceRangeInfo, targetCellInfo, fieldsConfig
    }
  }
}
```

**Rationale**:
- Follows established resource pattern in Univer
- Separate from cell data (allows rebuild if needed)
- Supports multiple pivot tables per workbook
- Easy to serialize/deserialize

## Risks / Trade-offs

### Performance with Large Datasets

**Risk**: Pivot table calculation may be slow for large source ranges (>10,000 rows)

**Mitigation**:
- Implement dirty tracking to avoid unnecessary recalculations
- Add progress indicator for long calculations
- Document performance limitations in MVP
- Plan for Web Worker optimization in future iteration
- Consider sampling for very large datasets

### Memory Usage

**Risk**: Multiple large pivot tables could consume significant memory

**Mitigation**:
- Implement cache eviction strategy (clear when pivot table not visible)
- Limit number of concurrent active pivot tables (configuration option)
- Monitor and test with realistic data sizes
- Document memory considerations

### Complex Field Configurations

**Risk**: Supporting all possible field combinations increases complexity

**Mitigation**:
- MVP limits to simple configurations (1-2 row fields, 1 value field)
- Clear error messages for unsupported configurations
- Document MVP limitations
- Incremental enhancement in future versions

### UI State Synchronization

**Risk**: Keeping UI panel in sync with pivot table state

**Mitigation**:
- Single source of truth: PivotTableService
- Panel service subscribes to service events
- Use RxJS observables for reactive updates
- Clear separation of concerns (service vs panel state)

### Undo/Redo Complexity

**Risk**: Pivot table operations may involve multiple mutations, complicating undo/redo

**Mitigation**:
- Use command pattern to encapsulate complex operations
- Commands trigger multiple mutations as atomic unit
- Leverage existing undo/redo infrastructure
- Test undo/redo for all pivot table operations

## Migration Plan

N/A - This is a new feature, not a migration. No existing pivot table data to migrate.

## Open Questions

1. **Refresh Strategy**: Should pivot tables auto-refresh on source data change, or require manual refresh?
   - **Recommendation**: Auto-refresh for MVP, add manual mode in settings later

2. **Field Naming**: Should we auto-detect column headers or require user to specify?
   - **Recommendation**: Auto-detect first row as headers (common pattern)

3. **Target Location**: Should pivot table placement be fixed or allow dynamic positioning?
   - **Recommendation**: User specifies target cell, pivot table grows from there (Excel pattern)

4. **Multiple Value Fields**: Support in MVP or defer?
   - **Recommendation**: Defer to post-MVP, keep single value field for simplicity

5. **Styling**: Use default styles or allow customization?
   - **Recommendation**: Default theme in MVP (defined in const.ts), customization later

