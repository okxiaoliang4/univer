# Sheets Pivot Table Core Plugin Specification

## Purpose

The Sheets Pivot Table plugin provides functionality for creating and managing pivot tables in spreadsheet documents. It supports Cross-Tabulation data analysis with multi-level row and column grouping, subtotals, grand totals, and value field aggregation.
## Requirements
### Requirement: Field Configuration

The system SHALL support configuring pivot table fields in four areas: row fields, column fields, value fields, and filter fields. Each field MAY specify whether to show subtotals.

#### Scenario: Add field to row area with subtotal configuration
- **WHEN** a user adds a source column to the row fields area
- **AND** sets `showSubTotals: true` on the field
- **THEN** the pivot table stores the field ID in the rowFields array
- **AND** stores the `showSubTotals` configuration
- **AND** marks the pivot table as dirty for recalculation
- **AND** `PivotEngineV2` generates subtotal rows for each unique value of that field

#### Scenario: Add field to column area with subtotal configuration
- **WHEN** a user adds a source column to the column fields area
- **AND** sets `showSubTotals: true` on the field
- **THEN** the pivot table stores the field ID in the columnFields array
- **AND** stores the `showSubTotals` configuration
- **AND** marks the pivot table as dirty for recalculation
- **AND** `PivotEngineV2` generates subtotal columns for each unique value of that field

### Requirement: PivotEngineV2 Cross-Tabulation Calculation Engine

The system SHALL provide a `PivotEngineV2` class that calculates pivot table results in a structured Cross-Tabulation format optimized for UI rendering.

#### Scenario: Calculate Cross-Tabulation with structured output
- **WHEN** `PivotEngineV2` is instantiated with `IPivotTableCrossTabConfig` containing row fields, column fields, value fields, and source data
- **AND** `getCalculatedData()` is called
- **THEN** the engine returns `IPivotTableCrossTabData` with structured output including:
  - `isEmpty` flag indicating if data is empty
  - `dimensions` object with row/column counts
  - `structure.rowHeaders` array with multi-level row values
  - `structure.columnHeaders` array with multi-level column values
  - `structure.values` 3D array `[rowIndex][columnIndex][valueFieldIndex]`
  - `structure.rowTypes` and `structure.columnTypes` arrays
  - `structure.subtotalRows` and `structure.subtotalColumns` metadata
  - `structure.rowGroups` and `structure.columnGroups` for collapse/expand
  - `structure.rowLevelMap` and `structure.columnLevelMap` for quick lookup

#### Scenario: Field-level subtotal configuration
- **WHEN** a row field or column field has `showSubTotals: true`
- **THEN** the engine generates subtotal rows/columns for each unique value of that field
- **AND** the subtotal value equals the aggregation of all data rows in that group
- **AND** the first field's subtotal (level=0, fieldIndex=0) is treated as the grand total

#### Scenario: Multi-level row and column fields
- **WHEN** multiple row fields or column fields are configured
- **THEN** the engine creates hierarchical grouping
- **AND** generates proper multi-level headers in `rowHeaders` and `columnHeaders`
- **AND** creates parent-child relationships in `rowGroups` and `columnGroups`
- **AND** populates `rowLevelMap` and `columnLevelMap` with all group IDs for each row/column

#### Scenario: Multi-value field support
- **WHEN** multiple value fields are configured
- **THEN** the engine generates a 3D values array `[rowIndex][columnIndex][valueFieldIndex]`
- **AND** includes `valueFieldHeaders` array with field names
- **AND** each cell contains values for all configured value fields

#### Scenario: Empty data detection
- **WHEN** source data has no value fields configured
- **OR** all source data is filtered out
- **OR** all calculated values are null/empty
- **THEN** the engine sets `isEmpty: true` in the output
- **AND** still returns valid structure with empty arrays

#### Scenario: Dimension calculation
- **WHEN** calculation completes
- **THEN** the engine populates `dimensions` object with:
  - `totalRows`: total number of rows including subtotals and grand total
  - `totalColumns`: total number of columns including subtotals and grand total
  - `dataRowCount`: number of data rows (excluding subtotals)
  - `dataColumnCount`: number of data columns (excluding subtotals)
  - `valueFieldCount`: number of configured value fields

#### Scenario: Individual setter methods
- **WHEN** `setRowFields()`, `setColumnFields()`, `setValueFields()`, `setFilterFields()`, or `setSourceData()` is called
- **THEN** the engine updates the corresponding configuration field
- **AND** marks the engine as dirty (`_isDirty = true`)
- **AND** invalidates cached calculation result

#### Scenario: Caching mechanism
- **WHEN** `getCalculatedData()` is called multiple times without configuration changes
- **THEN** the engine returns the cached result
- **AND** does not recalculate
- **WHEN** any setter method is called
- **THEN** the engine marks itself as dirty
- **AND** recalculates on next `getCalculatedData()` call

### Requirement: PivotTableRenderModel for UI Rendering

The system SHALL provide a `PivotTableRenderModel` class that provides business logic for rendering Cross-Tabulation pivot tables.

#### Scenario: Visible row/column calculation with collapse support
- **WHEN** `PivotTableRenderModel` is instantiated with `IPivotTableCrossTabData`
- **AND** some row groups or column groups are collapsed
- **THEN** `getVisibleRowIndices()` returns only visible row indices
- **AND** `getVisibleColumnIndices()` returns only visible column indices
- **AND** collapsed group rows/columns are excluded from visible indices

#### Scenario: Cell value retrieval
- **WHEN** `getCellValue(rowIndex, columnIndex, valueFieldIndex)` is called
- **THEN** the model returns the value from the 3D values array
- **AND** handles out-of-bounds indices gracefully

#### Scenario: Row/column information retrieval
- **WHEN** `getRowInfo(rowIndex)` or `getColumnInfo(columnIndex)` is called
- **THEN** the model returns an object containing:
  - Headers array
  - Type ('data' | 'subtotal')
  - Visibility status
  - Group information

#### Scenario: Group collapse/expand operations
- **WHEN** `toggleRowGroup(groupId)` or `toggleColumnGroup(groupId)` is called
- **THEN** the model updates the collapse state
- **AND** subsequent `getVisibleRowIndices()` or `getVisibleColumnIndices()` calls reflect the new state
- **WHEN** `expandAllRows()` or `collapseAllRows()` is called
- **THEN** the model updates all row group collapse states accordingly

#### Scenario: Group information for UI controls
- **WHEN** `getRowGroupInfo(rowIndex)` or `getColumnGroupInfo(columnIndex)` is called
- **THEN** the model returns information needed for collapse/expand buttons:
  - Group ID
  - Level
  - Whether it has children
  - Current expanded state
  - Whether it can be collapsed

### Requirement: Pivot Table Output Protection

The system SHALL automatically protect pivot table output ranges from manual editing to maintain data integrity and prevent confusion between calculated and manually-entered values.

Protection is implemented using a layered architecture:
- **Range Service Layer** (`PivotTableRangeService`): Provides O(log n) range queries using RTree spatial index
- **Render Layer** (`PivotTableRenderController`): Handles cell content and style injection via interceptor
- **Business Logic Layer** (`PivotTablePermissionController`): Handles core edit command blocking
- **UI Layer** (`@sheets-pivot-table-ui`): `PivotTablePermissionUIController` handles UI interactions

#### Scenario: Block manual cell value editing in pivot output range
- **WHEN** a user attempts to edit a cell that is within a pivot table output range
- **AND** executes a `SetRangeValuesCommand` or similar edit command
- **THEN** the `PivotTablePermissionController` uses `PivotTableRangeService.isPivotOutputCell()` for O(log n) lookup
- **AND** throws a `CustomCommandExecutionError` with message indicating the cell is protected
- **AND** displays a permission error dialog to the user
- **AND** the edit command does not execute

#### Scenario: Inject read-only metadata and styles into pivot output cells
- **WHEN** the rendering system requests cell data for a cell within a pivot table output range
- **AND** the `CELL_CONTENT` interceptor is triggered
- **THEN** the `PivotTableRenderController` uses `PivotTableRangeService` for O(log n) range detection
- **AND** injects pivot output values from the relative-positioned output matrix
- **AND** injects styles from `PivotTableStyleService` based on cell type and level
- **AND** the cell data includes `{ isPivotOutput: true, selectionProtection: [{ [UnitAction.Edit]: false, [UnitAction.View]: true }] }`
- **AND** the original cell data in the worksheet model is not modified

#### Scenario: Performance optimization with RTree spatial indexing
- **WHEN** the controllers need to check if a cell is in a pivot output range
- **THEN** they use `PivotTableRangeService.isPivotOutputCell()` for O(log n) lookup
- **AND** the service uses RTree spatial index instead of iterating through all pivot tables
- **AND** interceptor performance does not degrade user experience even with many pivot tables
- **AND** rendering remains smooth during scrolling

#### Scenario: Automatic protection on pivot table creation
- **WHEN** a new pivot table is created via `SheetsPivotTableService.createPivotTable()`
- **THEN** `PivotTableRangeService` registers the output range in RTree
- **AND** `PivotTableRenderController` and `PivotTablePermissionController` are automatically active
- **AND** protection is immediately active for all cells in the pivot output range
- **AND** the output cells are protected from both UI interactions and programmatic edits

#### Scenario: Update protection when pivot output range changes
- **WHEN** a pivot table's target range is updated via `updateTargetCell()`
- **OR** the pivot output dimensions change due to field configuration changes
- **THEN** `PivotTableRangeService` updates the RTree index with new range
- **AND** clears cells that are no longer in the output range
- **AND** protection automatically applies to the new range
- **AND** old range cells are no longer protected

#### Scenario: Remove protection when pivot table is deleted
- **WHEN** a pivot table is deleted via `deletePivotTable()`
- **THEN** `PivotTableRangeService` removes the range from RTree
- **AND** clears all cells in the former output range
- **AND** cells in the former output range are no longer protected
- **AND** users can edit those cells normally

### Requirement: Pivot Table Formula Integration

The system SHALL enable formulas to reference pivot table output cells seamlessly, allowing users to create calculations based on pivot table results without writing data to the worksheet model.

Formula integration is implemented using the feature calculation mechanism:
- **Formula Engine Integration**: `PivotTableFormulaController` registers pivot tables as features in `IFeatureCalculationManagerService`
- **Runtime Data Provision**: Pivot table output is provided to the formula engine as runtime cell data without persisting to the model
- **Automatic Lifecycle Management**: Features are registered/unregistered when pivot tables are created/deleted
- **Dynamic Range Updates**: Feature dependency ranges are updated when pivot table configuration changes

#### Scenario: Simple formula referencing pivot output cell
- **WHEN** a user creates a formula `=C3` where `C3` is a pivot table output cell
- **THEN** the formula engine calls the pivot table feature's `getDirtyData()` callback
- **AND** receives the pivot table value for that cell position
- **AND** the formula calculates correctly using the pivot table data

#### Scenario: Formula with operations on pivot output cells
- **WHEN** a user creates a formula `=C3+D3` where both `C3` and `D3` are pivot table output cells
- **THEN** the formula engine retrieves values from both cells via the feature callback
- **AND** performs the addition operation correctly
- **AND** returns the calculated result

#### Scenario: Formula referencing pivot table in different worksheet
- **WHEN** a user creates a formula in `Sheet2!A1` that references `Sheet1!C3` where `C3` is pivot output
- **THEN** the formula engine correctly resolves the cross-sheet reference
- **AND** retrieves pivot data from the appropriate unit and sheet
- **AND** calculates the formula using the pivot table value

#### Scenario: Automatic formula recalculation when pivot data changes
- **WHEN** pivot table source data is modified (e.g., values in source range change)
- **AND** the pivot table recalculates its output
- **THEN** any formulas referencing the pivot output automatically recalculate
- **AND** the new formula results reflect the updated pivot table values

#### Scenario: Feature registration on pivot table creation
- **WHEN** a new pivot table is created via `SheetsPivotTableService.createPivotTable()`
- **THEN** `PivotTableFormulaController` automatically registers the pivot table as a feature
- **AND** executes `SetFeatureCalculationMutation` with appropriate parameters
- **AND** the feature is available for formula calculations immediately

#### Scenario: Feature unregistration on pivot table deletion
- **WHEN** a pivot table is deleted via `SheetsPivotTableService.deletePivotTable()`
- **THEN** `PivotTableFormulaController` automatically unregisters the feature
- **AND** executes `RemoveFeatureCalculationMutation` to clean up the feature
- **AND** formulas referencing the deleted pivot table return errors

#### Scenario: Feature range updates when pivot table moves
- **WHEN** a pivot table's target cell is changed via `updateTargetCell()`
- **THEN** `PivotTableFormulaController` detects the range change event
- **AND** unregisters the old feature and registers a new feature with updated ranges
- **AND** formulas continue to work with the moved pivot table

#### Scenario: Feature range updates when pivot output size changes
- **WHEN** pivot table field configuration changes cause the output dimensions to change
- **THEN** `PivotTableFormulaController` detects the range change event
- **AND** updates the feature dependency ranges to match the new output size
- **AND** formulas referencing cells outside the new range return errors
- **AND** formulas referencing cells within the new range continue to work

#### Scenario: Runtime data isolation from worksheet model
- **WHEN** formulas reference pivot table output cells
- **THEN** the formula engine receives data from the feature callback
- **AND** the worksheet model cells remain empty (no data persistence)
- **AND** pivot table values are not saved to the document
- **AND** rendering continues to use `CELL_CONTENT` interceptor for display

#### Scenario: Performance optimization with O(1) data access
- **WHEN** the formula engine calls `getDirtyData()` for pivot table features
- **THEN** the callback performs O(1) lookup in the pre-calculated pivot output matrix
- **AND** transforms relative positions to absolute worksheet positions
- **AND** returns data in the expected `IRuntimeUnitDataType` format
- **AND** does not trigger pivot table recalculation during formula evaluation

### Requirement: Pivot Table Range Service (RTree Spatial Index)

The system SHALL provide a `PivotTableRangeService` that uses RTree spatial indexing for O(log n) pivot table output range queries to optimize rendering performance.

#### Scenario: Register pivot table range on creation
- **WHEN** a new pivot table is created via `SheetsPivotTableService.createPivotTable()`
- **THEN** `PivotTableRangeService` inserts the output range into the RTree index
- **AND** the range is keyed by `unitId`, `subUnitId`, and `pivotTableId`
- **AND** subsequent `isPivotOutputCell()` queries can find this range in O(log n) time

#### Scenario: Update pivot table range on configuration change
- **WHEN** a pivot table's output range changes due to field configuration or target cell update
- **THEN** `PivotTableRangeService` removes the old range from RTree
- **AND** inserts the new range into RTree
- **AND** the index remains consistent with actual output ranges

#### Scenario: Remove pivot table range on deletion
- **WHEN** a pivot table is deleted via `deletePivotTable()`
- **THEN** `PivotTableRangeService` removes the range from RTree
- **AND** subsequent queries no longer find this pivot table

#### Scenario: O(log n) cell lookup performance
- **WHEN** `isPivotOutputCell(unitId, subUnitId, row, col)` is called
- **THEN** the service performs an RTree point query
- **AND** returns `true` if the cell is within any pivot table output range
- **AND** the query completes in O(log n) time where n is the number of pivot tables

### Requirement: Pivot Table Style Service (Green Theme)

The system SHALL provide a `PivotTableStyleService` that calculates cell styles for pivot table output using a green color theme.

#### Scenario: Calculate header cell style
- **WHEN** `getCellStyle()` is called for a header cell (row header or column header)
- **THEN** the service returns a style with:
  - Background color: #2E7D32 (Green 800)
  - Font color: #FFFFFF (White)
  - Bold: true
- **AND** the style is suitable for rendering without persistence

#### Scenario: Calculate data cell style with zebra striping
- **WHEN** `getCellStyle()` is called for a data cell
- **THEN** the service returns a style with alternating background colors:
  - Odd rows: #E8F5E9 (Green 50)
  - Even rows: #C8E6C9 (Green 100)
- **AND** provides visual distinction between rows

#### Scenario: Calculate subtotal cell style
- **WHEN** `getCellStyle()` is called for a subtotal cell (row or column subtotal)
- **THEN** the service returns a style with:
  - Background color: #A5D6A7 (Green 200)
  - Bold: true
- **AND** visually distinguishes subtotals from data cells

#### Scenario: Calculate grand total cell style
- **WHEN** `getCellStyle()` is called for a grand total cell
- **THEN** the service returns a style with:
  - Background color: #81C784 (Green 300)
  - Bold: true
- **AND** provides the strongest visual emphasis in the table

#### Scenario: Multi-level header color graduation
- **WHEN** pivot table has multiple row or column field levels
- **THEN** `getCellStyle()` returns progressively lighter shades for deeper levels
- **AND** level 0 uses the darkest shade (#2E7D32)
- **AND** each subsequent level uses a 10% lighter shade

### Requirement: Pivot Table Output Range Clearing

The system SHALL automatically clear pivot table output ranges before rendering to ensure data consistency when output dimensions change.

#### Scenario: Clear target range before initial output
- **WHEN** a pivot table calculates its output for the first time
- **THEN** the system executes `SetRangeValuesMutation` to clear the target range
- **AND** uses `{ onlyLocal: true }` option to avoid broadcasting
- **AND** the interceptor then injects calculated values during rendering

#### Scenario: Clear old cells when output range shrinks
- **WHEN** a pivot table's configuration changes cause the output range to shrink
- **AND** the old range was 8x8 and the new range is 4x4
- **THEN** the system calculates the difference (cells in old range but not in new range)
- **AND** executes `SetRangeValuesMutation` with null values for those cells
- **AND** formulas referencing cleared cells return empty values
- **AND** the UI no longer displays stale data in cleared cells

#### Scenario: Clear entire range when pivot table is deleted
- **WHEN** a pivot table is deleted via `deletePivotTable()`
- **THEN** the system clears all cells in the former output range
- **AND** the cells return to their original empty state
- **AND** any formulas referencing the deleted pivot output return empty values

#### Scenario: Maintain formula integration after range change
- **WHEN** a pivot table output range shrinks
- **AND** a formula `=C3` references a cell that was in the old range but not in the new range
- **THEN** the formula engine no longer receives data from the pivot table feature
- **AND** the formula returns an empty value (not an error)
- **AND** formulas referencing cells still in the output range continue to work correctly

