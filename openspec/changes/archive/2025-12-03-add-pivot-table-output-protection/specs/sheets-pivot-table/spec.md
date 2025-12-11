# Sheets Pivot Table Spec Delta

## ADDED Requirements

### Requirement: Pivot Table Output Protection

The system SHALL automatically protect pivot table output ranges from manual editing to maintain data integrity and prevent confusion between calculated and manually-entered values.

#### Scenario: Block manual cell value editing in pivot output range
- **WHEN** a user attempts to edit a cell that is within a pivot table output range
- **AND** executes a `SetRangeValuesCommand` or similar edit command
- **THEN** the `PivotTablePermissionController` intercepts the command before execution
- **AND** throws a `CustomCommandExecutionError` with message indicating the cell is protected
- **AND** displays a permission error dialog to the user
- **AND** the edit command does not execute

#### Scenario: Inject read-only metadata into pivot output cells
- **WHEN** the rendering system requests cell data for a cell within a pivot table output range
- **AND** the `CELL_CONTENT` interceptor is triggered
- **THEN** the `PivotTablePermissionController` injects read-only metadata into the cell data
- **AND** the cell data includes `{ isPivotOutput: true, isReadOnly: true }`
- **AND** the rendering system displays appropriate read-only visual indicators
- **AND** the original cell data in the worksheet model is not modified

#### Scenario: Automatic protection on pivot table creation
- **WHEN** a new pivot table is created via `SheetsPivotTableService.createPivotTable()`
- **THEN** the `PivotTablePermissionController` subscribes to the pivot table's lifecycle
- **AND** builds cache entries for all cells in the pivot output range
- **AND** the output cells are immediately protected without requiring manual configuration

#### Scenario: Update protection when pivot output range changes
- **WHEN** a pivot table's target range is updated via `updateTargetCell()`
- **OR** the pivot output dimensions change due to field configuration changes
- **THEN** the controller clears cache entries for the old output range
- **AND** rebuilds cache entries for the new output range
- **AND** protection automatically applies to the new range

#### Scenario: Remove protection when pivot table is deleted
- **WHEN** a pivot table is deleted via `deletePivotTable()`
- **THEN** the controller clears all cache entries for that pivot table's output range
- **AND** cells in the former output range are no longer protected
- **AND** users can edit those cells normally

#### Scenario: Allow editing cells outside pivot output ranges
- **WHEN** a user attempts to edit a cell that is NOT within any pivot table output range
- **THEN** the `PivotTablePermissionController` does not intercept the command
- **AND** normal editing proceeds as expected
- **AND** no permission errors are shown

#### Scenario: Performance optimization with cache
- **WHEN** the controller needs to check if a cell is in a pivot output range
- **THEN** it uses an O(1) cached lookup `_pivotOutputCache.get(unitId, subUnitId, "row-col")`
- **AND** the cache is automatically updated when pivot tables change
- **AND** the cache is cleared when worksheets are unloaded
- **AND** interceptor performance does not degrade user experience

#### Scenario: Block paste operations into pivot output range
- **WHEN** a user attempts to paste data into a range that overlaps with pivot output
- **AND** executes `SheetPasteShortKeyCommand` or similar paste command
- **THEN** the controller blocks the paste operation for cells within the pivot output
- **AND** shows an appropriate error message
- **AND** allows paste to proceed for cells outside the pivot output (partial paste)

#### Scenario: Block clear operations on pivot output range
- **WHEN** a user attempts to clear content in cells within pivot output range
- **AND** executes `ClearSelectionContentCommand`
- **THEN** the controller blocks the clear operation
- **AND** displays a permission error dialog
- **AND** the pivot table output remains unchanged

