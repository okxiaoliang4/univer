# Add PivotEngineV2 with Cross-Tabulation Support

## Why

The current `PivotEngine` implementation outputs data in a sparse matrix format (`IObjectMatrixPrimitiveType`) which is not ideal for rendering Cross-Tabulation pivot tables. The existing implementation lacks support for:
- Structured output format optimized for Cross-Tabulation display
- Subtotal rows/columns with proper grouping information
- Multi-level row/column hierarchies with collapse/expand support
- Clear separation between input configuration and output results
- Empty data detection and dimension information

This change introduces `PivotEngineV2` that provides a structured Cross-Tabulation data format, making it easier for UI components to render pivot tables with proper subtotals, totals, and hierarchical grouping.

## What Changes

### sheets-pivot-table (Core Plugin)
- **ADDED**: `IPivotField.showSubTotals` field for field-level subtotal configuration
- **ADDED**: `IPivotTableCrossTabConfig` interface for input configuration (separated from output)
- **ADDED**: `IPivotTableCrossTabData` interface for structured Cross-Tabulation output
- **ADDED**: `PivotEngineV2` class implementing Cross-Tabulation calculation engine
- **ADDED**: `PivotTableRenderModel` class providing rendering business logic
- **MODIFIED**: Enhanced field configuration to support subtotal display per field

### Key Features
- Structured output format with row/column headers, values matrix, and grouping information
- Support for multi-level row/column fields with hierarchical grouping
- Subtotal calculation based on field-level `showSubTotals` configuration
- Grand total calculation (first field's subtotal = grand total)
- Collapse/expand support via grouping information
- Multi-value field support (3D array: [row][column][valueField])
- Empty data detection (`isEmpty` flag)
- Dimension information (`dimensions` object with row/column counts)
- Input/output separation (no config in output)
- Individual setter methods (`setRowFields`, `setColumnFields`, etc.) following existing pattern
- Smart duplicate value removal in headers:
  - Parent-level values appear only once across entire row/column
  - Child-level values appear once per parent group (e.g., "线下门店" appears once under Q1, once under Q2)
  - Maintains visual hierarchy while reducing clutter

## Impact

- **Affected specs**: `sheets-pivot-table` (new capability being added)
- **Affected code**:
  - `packages/sheets-pivot-table/src/types/type.ts` - Add new interfaces
  - `packages/sheets-pivot-table/src/models/pivot-engine-v2.ts` - New engine implementation
  - `packages/sheets-pivot-table/src/models/pivot-table-render-model.ts` - New render model
  - `packages/sheets-pivot-table/src/models/__tests__/pivot-engine-v2.spec.ts` - Test file
- **Dependencies**: Builds on existing aggregation models and field types
- **Pattern reference**: Follows `PivotEngine` class pattern with individual setters
- **Breaking changes**: None (additive changes, new class alongside existing)

