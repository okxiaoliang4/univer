## RENAMED Requirements
- FROM: `### Requirement: PivotEngineV2 Cross-Tabulation Calculation Engine`
- TO: `### Requirement: PivotEngine PivotModel Calculation Engine`

## REMOVED Requirements
### Requirement: PivotTableRenderModel for UI Rendering
Reason: Rendering now uses PivotEngine helper methods (`getOutputMatrix`, `determineCellType`, `getRowInfo`, `getColumnInfo`, `getCellInfo`) without a separate render model.
Migration: Update UI/render callers to consume PivotEngine helpers directly; remove dependencies on `PivotTableRenderModel`.

### Requirement: Pivot Table Value Position Support
Reason: Pivot tables now render value fields in column orientation only; row-based value positioning was removed.
Migration: Remove `valuePosition` configuration and tests; assume column-oriented value fields in UI and engine usage.

## ADDED Requirements
### Requirement: Pivot Engine Rendering Helpers
The system SHALL expose rendering helpers on PivotEngine for UI and styling without relying on a separate render model.

#### Scenario: Row/column info lookup
- **WHEN** `getRowInfo()` or `getColumnInfo()` is called with an index
- **THEN** the engine returns normalized headers for the axis depth, the axis item type (`data` | `subtotal` | `grand`), its level, and fieldIndex based on the PivotModel axis items
- **AND** out-of-bounds indices return an empty headers array with type `data`.

#### Scenario: Cell typing for styling
- **WHEN** `determineCellType(rowIndex, columnIndex)` is called for an output-matrix coordinate
- **THEN** the engine classifies the cell as `header`, `rowHeader`, `columnHeader`, `data`, `subtotal`, or `grandTotal`
- **AND** returns the hierarchy level derived from axis metadata and header depths (including optional value-header rows when multiple value fields exist)
- **AND** subtotal level is derived from the corresponding row/column subtotal items.

### Requirement: Pivot Output Capacity Management
The system SHALL ensure worksheets have sufficient rows and columns for pivot output ranges.

#### Scenario: Insert rows/cols when output exceeds sheet
- **WHEN** a pivot table output range in absolute coordinates exceeds the current worksheet row or column count
- **THEN** `PivotTableRangeService` executes `InsertRowMutation` or `InsertColMutation` with `onlyLocal: true` to expand the sheet before registering the range
- **AND** the updated range is then registered in the spatial index
- **AND** inserted rows/columns keep other users synchronized via the shared mutation stream.

## MODIFIED Requirements
### Requirement: Field Configuration
The system SHALL support configuring pivot table fields in four areas: row fields, column fields, value fields, and filter fields. Each field MAY specify whether to show subtotals.

#### Scenario: Add field to row area with subtotal configuration
- **WHEN** a user adds a source column to the row fields area
- **AND** sets `showSubTotals: true` on the field
- **THEN** the pivot table stores the field ID in the rowFields array
- **AND** stores the `showSubTotals` configuration
- **AND** marks the pivot table as dirty for recalculation
- **AND** the PivotEngine generates subtotal rows for each unique value of that field in the PivotModel.

#### Scenario: Add field to column area with subtotal configuration
- **WHEN** a user adds a source column to the column fields area
- **AND** sets `showSubTotals: true` on the field
- **THEN** the pivot table stores the field ID in the columnFields array
- **AND** stores the `showSubTotals` configuration
- **AND** marks the pivot table as dirty for recalculation
- **AND** the PivotEngine generates subtotal columns for each unique value of that field in the PivotModel.

### Requirement: PivotEngine PivotModel Calculation Engine
The system SHALL provide a PivotEngine that calculates pivot table results as a PivotModel with axis metadata and values for rendering and formulas.

#### Scenario: Calculate PivotModel with axis metadata
- **WHEN** PivotEngine is instantiated with row, column, value, and filter fields plus source data
- **AND** `getPivotModel()` is called
- **THEN** the engine returns a PivotModel containing `isEmpty`, `valueFields` (id, name, agg), `rowAxis` and `colAxis` (items with headers/display/type/level/fieldIndex/valueFieldIndex, headerDepth, levelMap, subtotalMap), `values` matrix indexed by `[row][col][valueIndex]`, and `dimensions` (rowCount, colCount, valueFieldCount).

#### Scenario: Subtotals and grand totals without duplication
- **WHEN** row or column fields have `showSubTotals: true`
- **THEN** the engine creates subtotal items per field level (labelled with “总计”) and a single grand total at level 0 when the first field has `showSubTotals: true`
- **AND** subtotal and grand items reuse the correct combo sets so no duplicate subtotal/grand rows or columns are emitted.

#### Scenario: Multi-value fields in column orientation
- **WHEN** multiple value fields are configured
- **THEN** column axis items carry `valueFieldIndex` to disambiguate per-value columns, and the output matrix renders value headers in column orientation (row-based value positioning is not supported)
- **AND** `getHeaderRowsCount()` and `getOutputMatrix()` include a value-header row when multiple value fields exist.

#### Scenario: Normalized header rows and placeholders
- **WHEN** no column fields are configured
- **THEN** `getOutputMatrix()` emits a single header row containing row field names and value labels
- **AND** when the model is empty it returns a placeholder matrix for rendering instead of null.

#### Scenario: Cached calculation invalidation
- **WHEN** field setters, `setSourceData`, or `setCalculatedData` are called
- **THEN** the engine marks itself dirty and recalculates on the next `getPivotModel()` call
- **AND** repeated calls without configuration changes reuse the cached PivotModel.

### Requirement: Pivot Table Style Service (Green Theme)
The system SHALL provide pivot table cell styling with a green theme.

#### Scenario: Header cell style with level-based graduation
- **WHEN** `getCellStyle()` is called for header, rowHeader, or columnHeader cells
- **THEN** the service uses the dark header green (#2E7D32) at level 0 and progressively lightens the shade by mixing with white (20% per additional level)
- **AND** applies white text, bold font, and thin green borders.

#### Scenario: Data cell zebra striping
- **WHEN** `getCellStyle()` is called for data cells
- **THEN** the service alternates row backgrounds (#E8F5E9 for odd rows, #C8E6C9 for even rows) with green borders.

#### Scenario: Subtotal style
- **WHEN** `getCellStyle()` is called for subtotal cells
- **THEN** the service returns a medium green background (#A5D6A7), bold text, and green borders.

#### Scenario: Grand total emphasis
- **WHEN** `getCellStyle()` is called for grandTotal cells
- **THEN** the service applies the darkest header background (#2E7D32) with white text, bold font, and green borders for maximum emphasis.

