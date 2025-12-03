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
- **Business Logic Layer** (`@sheets-pivot-table`): `PivotTablePermissionController` handles cell content interception and core edit commands
- **UI Layer** (`@sheets-pivot-table-ui`): `PivotTablePermissionUIController` handles UI interactions (edit mode, formula bar input)

#### Scenario: Block manual cell value editing in pivot output range
- **WHEN** a user attempts to edit a cell that is within a pivot table output range
- **AND** executes a `SetRangeValuesCommand` or similar edit command
- **THEN** the `PivotTablePermissionController` (business logic layer) intercepts the command before execution
- **AND** throws a `CustomCommandExecutionError` with message indicating the cell is protected
- **AND** displays a permission error dialog to the user
- **AND** the edit command does not execute

#### Scenario: Inject read-only metadata into pivot output cells
- **WHEN** the rendering system requests cell data for a cell within a pivot table output range
- **AND** the `CELL_CONTENT` interceptor is triggered
- **THEN** the `PivotTablePermissionController` (business logic layer) injects read-only metadata into the cell data
- **AND** the cell data includes `{ isPivotOutput: true, selectionProtection: [{ [UnitAction.Edit]: false, [UnitAction.View]: true }] }`
- **AND** the rendering system displays appropriate read-only visual indicators
- **AND** the original cell data in the worksheet model is not modified

#### Scenario: Block entering edit mode for pivot output cells
- **WHEN** a user attempts to enter edit mode (double-click or F2) on a cell within a pivot table output range
- **AND** executes `SetCellEditVisibleOperation` or `SetCellEditVisibleWithF2Operation`
- **THEN** the `PivotTablePermissionUIController` (UI layer) intercepts the command before execution
- **AND** throws a `CustomCommandExecutionError` with message indicating the cell is protected
- **AND** the edit mode does not activate
- **AND** the user cannot type in the cell

#### Scenario: Block formula bar input for pivot output cells
- **WHEN** a user attempts to type in the formula bar while a pivot output cell is selected
- **AND** executes `InsertCommand` or `IMEInputCommand`
- **THEN** the `PivotTablePermissionUIController` (UI layer) intercepts the command before execution
- **AND** throws a `CustomCommandExecutionError` with message indicating the cell is protected
- **AND** the input is blocked
- **AND** the formula bar does not update

#### Scenario: Automatic protection on pivot table creation
- **WHEN** a new pivot table is created via `SheetsPivotTableService.createPivotTable()`
- **THEN** both `PivotTablePermissionController` (business logic) and `PivotTablePermissionUIController` (UI) are initialized
- **AND** protection is immediately active for all cells in the pivot output range
- **AND** the output cells are protected from both UI interactions and programmatic edits

#### Scenario: Update protection when pivot output range changes
- **WHEN** a pivot table's target range is updated via `updateTargetCell()`
- **OR** the pivot output dimensions change due to field configuration changes
- **THEN** both controllers automatically detect the change through dynamic cell checking
- **AND** protection automatically applies to the new range
- **AND** old range cells are no longer protected

#### Scenario: Remove protection when pivot table is deleted
- **WHEN** a pivot table is deleted via `deletePivotTable()`
- **THEN** both controllers automatically detect the deletion through dynamic cell checking
- **AND** cells in the former output range are no longer protected
- **AND** users can edit those cells normally

#### Scenario: Allow editing cells outside pivot output ranges
- **WHEN** a user attempts to edit a cell that is NOT within any pivot table output range
- **THEN** neither controller intercepts the command
- **AND** normal editing proceeds as expected
- **AND** no permission errors are shown

#### Scenario: Performance optimization with dynamic checking
- **WHEN** the controllers need to check if a cell is in a pivot output range
- **THEN** they use `ISheetsPivotTableService.isPivotOutputCell()` for O(1) lookup
- **AND** the service method queries pivot tables dynamically without maintaining a separate cache
- **AND** protection automatically updates when pivot tables change
- **AND** interceptor performance does not degrade user experience

#### Scenario: Block paste operations into pivot output range
- **WHEN** a user attempts to paste data into a range that overlaps with pivot output
- **AND** executes `SheetPasteShortKeyCommand` or similar paste command
- **THEN** the `PivotTablePermissionController` (business logic layer) blocks the paste operation for cells within the pivot output
- **AND** shows an appropriate error message
- **AND** allows paste to proceed for cells outside the pivot output (partial paste)

#### Scenario: Block clear operations on pivot output range
- **WHEN** a user attempts to clear content in cells within pivot output range
- **AND** executes `ClearSelectionContentCommand`
- **THEN** the `PivotTablePermissionController` (business logic layer) blocks the clear operation
- **AND** displays a permission error dialog
- **AND** the pivot table output remains unchanged

