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
- **AND** the PivotEngine generates subtotal rows for each unique value of that field in the PivotModel.

#### Scenario: Add field to column area with subtotal configuration
- **WHEN** a user adds a source column to the column fields area
- **AND** sets `showSubTotals: true` on the field
- **THEN** the pivot table stores the field ID in the columnFields array
- **AND** stores the `showSubTotals` configuration
- **AND** marks the pivot table as dirty for recalculation
- **AND** the PivotEngine generates subtotal columns for each unique value of that field in the PivotModel.

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
- **RPC Compatibility**: The plugin supports a `notExecuteFormula` configuration option to disable formula integration in the main thread when using Web Workers, allowing the Worker thread to handle formula integration instead

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
- **AND** the `notExecuteFormula` configuration is `false` (default)
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

#### Scenario: RPC environment with notExecuteFormula enabled
- **WHEN** the pivot table plugin is configured with `notExecuteFormula: true` in the main thread
- **AND** the application uses Web Workers for formula calculation
- **THEN** `PivotTableFormulaController` is NOT initialized in the main thread
- **AND** no `SetFeatureCalculationMutation` is executed in the main thread
- **AND** the Worker thread's `PivotTableFormulaController` handles formula integration
- **AND** no `DataCloneError` occurs during mutation synchronization

#### Scenario: Non-RPC environment with default configuration
- **WHEN** the pivot table plugin is configured with default settings (`notExecuteFormula: false`)
- **AND** the application does NOT use Web Workers
- **THEN** `PivotTableFormulaController` is initialized in the main thread
- **AND** formula integration works correctly in the main thread
- **AND** all existing functionality is preserved

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

### Requirement: Field-level subtotal semantics
The first row/column field `showSubTotals` flag SHALL represent the table’s grand total toggles, while subsequent fields’ `showSubTotals` flags continue to represent their own subtotals (no separate grand total flags).

#### Scenario: First row field controls row grand total
- **WHEN** the first row field has `showSubTotals: true`
- **THEN** `PivotEngineV2` generates row grand total rows
- **AND** setting that flag to `false` removes row grand totals but keeps lower-level subtotals according to later row fields

#### Scenario: First column field controls column grand total
- **WHEN** the first column field has `showSubTotals: true`
- **THEN** `PivotEngineV2` generates column grand total columns
- **AND** setting that flag to `false` removes column grand totals but keeps lower-level subtotals according to later column fields

#### Scenario: Subsequent fields keep subtotal behavior
- **WHEN** row or column fields after the first have `showSubTotals: true`
- **THEN** `PivotEngineV2` generates subtotals for those fields without affecting grand totals

#### Scenario: Multi-level row subtotals mapping example
- **GIVEN** three row fields `[Region, Category, Quarter]`
- **WHEN** `Region.showSubTotals = true`
- **THEN** the engine outputs grand total rows for the whole table
- **WHEN** `Category.showSubTotals = true`
- **THEN** the engine outputs subtotals for each Category within its Region
- **WHEN** `Quarter.showSubTotals = true`
- **THEN** the engine outputs subtotals for each Quarter within its Region + Category grouping (does not create another grand total)

### Requirement: Column grand total per value field
Column grand totals SHALL output one grand total column per value field (mirroring Google Sheets behavior) instead of sharing a single grand total column across all value fields.

#### Scenario: Append grand total columns per value field
- **WHEN** column grand totals are enabled and multiple value fields exist
- **THEN** the structure includes one additional column per value field at the end of the column axis
- **AND** column headers/metadata align each grand total column with its corresponding value field
- **AND** dimensions and value arrays reflect the expanded column count

### Requirement: RPC-Compatible Plugin Configuration

The system SHALL provide a `notExecuteFormula` configuration option that allows users to disable formula integration in the main thread when using Web Workers, following the same pattern as `@univerjs/sheets-formula`.

#### Scenario: Configure plugin for RPC environment
- **WHEN** a user registers the pivot table plugin with `{ notExecuteFormula: true }`
- **THEN** the plugin stores this configuration
- **AND** skips `PivotTableFormulaController` initialization
- **AND** allows the Worker thread to handle formula integration

#### Scenario: Configure plugin for non-RPC environment
- **WHEN** a user registers the pivot table plugin without configuration or with `{ notExecuteFormula: false }`
- **THEN** the plugin uses default configuration
- **AND** initializes `PivotTableFormulaController` normally
- **AND** formula integration works in the main thread

#### Scenario: Default configuration preserves backward compatibility
- **WHEN** existing code registers the pivot table plugin without any configuration changes
- **THEN** the plugin behaves exactly as before the change
- **AND** no breaking changes occur for non-RPC deployments

### Requirement: Calculated Data Sync via Mutation

The system SHALL provide a `SetPivotTableCalculatedDataMutation` that syncs calculated pivot table data from Worker thread to Main thread in RPC environments.

#### Scenario: Worker calculates and syncs data to Main thread
- **WHEN** Worker thread calculates pivot table data
- **AND** the `calculatedData$` observable emits new data
- **THEN** `PivotTableFormulaController` executes `SetPivotTableCalculatedDataMutation`
- **AND** the mutation is synced to Main thread via `DataSyncReplicaController`
- **AND** Main thread's `SheetsPivotDataSourceModel.setCalculatedData()` receives the data
- **AND** the pivot table renders correctly on Main thread

#### Scenario: Main thread skips auto-calculation in RPC environment
- **WHEN** the pivot table plugin is configured with `notExecuteFormula: true`
- **AND** a new pivot table is created via `AddPivotTableMutation`
- **THEN** the PivotTable instance is created with `skipAutoCalculation: true`
- **AND** the `_initCalculatedDataListener` is NOT initialized
- **AND** no automatic calculation occurs on Main thread
- **AND** calculated data is received via `SetPivotTableCalculatedDataMutation` from Worker

#### Scenario: Single-threaded environment works normally
- **WHEN** the pivot table plugin is configured with `notExecuteFormula: false` (default)
- **AND** a new pivot table is created
- **THEN** the PivotTable instance is created with `skipAutoCalculation: false`
- **AND** the `_initCalculatedDataListener` IS initialized
- **AND** calculation occurs automatically when fields or source data change
- **AND** no mutation is needed for data sync (single-threaded)

