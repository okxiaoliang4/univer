## MODIFIED Requirements

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

## ADDED Requirements

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


