## ADDED Requirements

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

## MODIFIED Requirements

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

