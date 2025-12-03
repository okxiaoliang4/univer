## MODIFIED Requirements

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

