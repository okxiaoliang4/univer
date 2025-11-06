# Sheets Pivot Table Core Plugin Specification

## ADDED Requirements

### Requirement: Pivot Table Service Management

The system SHALL provide a PivotTableService to manage pivot table instances across workbooks and worksheets.

#### Scenario: Create new pivot table
- **WHEN** a user creates a new pivot table with valid source range and target location
- **THEN** the service generates a unique identifier for the pivot table
- **AND** stores the pivot table instance in the service registry
- **AND** associates the pivot table with the workbook and worksheet
- **AND** returns the pivot table ID

#### Scenario: Retrieve pivot table by ID
- **WHEN** a component requests a pivot table by its ID
- **THEN** the service returns the corresponding PivotTable instance
- **AND** returns null if the ID does not exist

#### Scenario: List pivot tables for worksheet
- **WHEN** a component requests all pivot tables for a specific worksheet
- **THEN** the service returns an array of all PivotTable instances for that worksheet
- **AND** returns an empty array if no pivot tables exist

#### Scenario: Delete pivot table
- **WHEN** a user deletes a pivot table
- **THEN** the service removes the pivot table from the registry
- **AND** emits a deletion event for subscribers
- **AND** clears associated cached data

### Requirement: Pivot Table Calculation

The system SHALL provide a PivotTableCalculationService to calculate pivot table results from source data.

#### Scenario: Calculate simple pivot table
- **WHEN** calculation is requested for a pivot table with one row field and one value field
- **THEN** the service extracts source data from the specified range
- **AND** groups data by unique values in the row field
- **AND** applies the aggregation function to each group
- **AND** returns a result matrix with row headers and aggregated values

#### Scenario: Handle empty source range
- **WHEN** calculation is requested for a pivot table with an empty source range
- **THEN** the service returns an empty result
- **AND** does not throw an error

#### Scenario: Support multiple aggregation functions
- **WHEN** a value field is configured with an aggregation function (SUM, COUNT, AVERAGE, MIN, MAX)
- **THEN** the service applies the correct aggregation algorithm
- **AND** returns properly aggregated numeric results
- **AND** handles non-numeric values appropriately (ignores for SUM/AVG/MIN/MAX, counts for COUNT)

#### Scenario: Calculate grand totals
- **WHEN** pivot table calculation completes
- **THEN** the service calculates grand total row for value fields
- **AND** includes grand total in the result matrix

### Requirement: Pivot Table Data Source

The system SHALL support worksheet ranges as data sources for pivot tables.

#### Scenario: Valid range as data source
- **WHEN** a pivot table is created with a worksheet range (e.g., Sheet1!A1:D100)
- **THEN** the system extracts data from the specified range
- **AND** treats the first row as column headers
- **AND** processes subsequent rows as data records

#### Scenario: Cross-sheet reference
- **WHEN** a pivot table's source range references a different sheet than the target location
- **THEN** the system correctly retrieves data from the source sheet
- **AND** renders results in the target sheet

#### Scenario: Invalid range handling
- **WHEN** a pivot table is created with an invalid range reference
- **THEN** the system returns an error
- **AND** does not create the pivot table

### Requirement: Field Configuration

The system SHALL support configuring pivot table fields in four areas: row fields, column fields, value fields, and filter fields.

#### Scenario: Add field to row area
- **WHEN** a user adds a source column to the row fields area
- **THEN** the pivot table stores the field ID in the rowFields array
- **AND** marks the pivot table as dirty for recalculation

#### Scenario: Add field to value area with aggregation
- **WHEN** a user adds a source column to the value fields area
- **AND** selects an aggregation function (SUM, COUNT, AVERAGE, MIN, MAX)
- **THEN** the pivot table stores the field ID with the aggregation type
- **AND** uses that aggregation in calculation

#### Scenario: Remove field from area
- **WHEN** a user removes a field from any area (row, column, value, filter)
- **THEN** the pivot table removes the field ID from the corresponding array
- **AND** marks the pivot table as dirty for recalculation

#### Scenario: Reorder fields within area
- **WHEN** a user changes the order of fields within an area
- **THEN** the pivot table updates the field order in the array
- **AND** marks the pivot table as dirty for recalculation

### Requirement: Pivot Table Commands

The system SHALL provide commands to perform pivot table operations.

#### Scenario: CreatePivotTableCommand execution
- **WHEN** CreatePivotTableCommand is executed with source range, target location, and initial configuration
- **THEN** the command validates the input parameters
- **AND** creates a new pivot table instance via PivotTableService
- **AND** triggers calculation
- **AND** executes SetPivotTableMutation for undo/redo support
- **AND** returns success with the new pivot table ID

#### Scenario: UpdatePivotTableFieldsCommand execution
- **WHEN** UpdatePivotTableFieldsCommand is executed with pivot table ID and new field configuration
- **THEN** the command retrieves the pivot table by ID
- **AND** updates the field configuration
- **AND** marks the pivot table as dirty
- **AND** executes SetPivotTableMutation
- **AND** returns success

#### Scenario: DeletePivotTableCommand execution
- **WHEN** DeletePivotTableCommand is executed with pivot table ID
- **THEN** the command retrieves the pivot table by ID
- **AND** removes the pivot table via PivotTableService
- **AND** executes RemovePivotTableMutation
- **AND** clears target cells
- **AND** returns success

#### Scenario: RefreshPivotTableCommand execution
- **WHEN** RefreshPivotTableCommand is executed with pivot table ID
- **THEN** the command marks the pivot table as dirty
- **AND** triggers recalculation
- **AND** updates target cells with new results
- **AND** returns success

### Requirement: Pivot Table Mutations

The system SHALL provide mutations to modify pivot table state for undo/redo support.

#### Scenario: SetPivotTableMutation execution
- **WHEN** SetPivotTableMutation is executed with pivot table data
- **THEN** the mutation adds or updates the pivot table in the service registry
- **AND** emits a change event

#### Scenario: RemovePivotTableMutation execution
- **WHEN** RemovePivotTableMutation is executed with pivot table ID
- **THEN** the mutation removes the pivot table from the service registry
- **AND** emits a deletion event

#### Scenario: Undo SetPivotTableMutation
- **WHEN** user triggers undo after SetPivotTableMutation
- **THEN** the mutation restores the previous pivot table state
- **AND** recalculates if necessary

### Requirement: Snapshot Serialization

The system SHALL serialize and deserialize pivot tables in workbook snapshots.

#### Scenario: Save pivot table in snapshot
- **WHEN** a workbook is saved
- **THEN** the system serializes all pivot table configurations to the snapshot
- **AND** stores them in the workbook resources under a pivot table resource key
- **AND** includes all field configurations, source ranges, and target locations

#### Scenario: Load pivot table from snapshot
- **WHEN** a workbook is loaded with pivot table data in the snapshot
- **THEN** the system deserializes all pivot table configurations
- **AND** reconstructs PivotTable instances via PivotTableService
- **AND** marks them as dirty for recalculation
- **AND** registers them with the service

#### Scenario: Handle missing snapshot data
- **WHEN** a workbook is loaded without pivot table snapshot data
- **THEN** the system initializes with no pivot tables
- **AND** does not throw an error

### Requirement: Configuration Schema

The system SHALL provide a configuration schema for the pivot table plugin.

#### Scenario: Default configuration
- **WHEN** the plugin is instantiated without custom configuration
- **THEN** the system uses default configuration values
- **AND** all features are enabled

#### Scenario: Custom configuration
- **WHEN** the plugin is instantiated with custom configuration
- **THEN** the system merges custom configuration with defaults
- **AND** applies the merged configuration to the IConfigService

### Requirement: Pivot Table Controller

The system SHALL provide a PivotTableController to coordinate pivot table lifecycle and events.

#### Scenario: Handle workbook change events
- **WHEN** cell data changes in a source range of a pivot table
- **THEN** the controller marks the affected pivot table as dirty
- **AND** schedules recalculation

#### Scenario: Handle worksheet deletion
- **WHEN** a worksheet containing pivot table target cells is deleted
- **THEN** the controller removes the pivot table
- **AND** cleans up associated resources

#### Scenario: Register snapshot handlers
- **WHEN** the controller starts
- **THEN** it registers snapshot save and load handlers with the resource manager
- **AND** ensures proper serialization/deserialization

### Requirement: Aggregation Functions

The system SHALL support five basic aggregation functions: SUM, COUNT, AVERAGE, MIN, and MAX.

#### Scenario: SUM aggregation
- **WHEN** a value field is configured with SUM aggregation
- **THEN** the system sums all numeric values in each group
- **AND** ignores non-numeric values
- **AND** returns null if no numeric values exist

#### Scenario: COUNT aggregation
- **WHEN** a value field is configured with COUNT aggregation
- **THEN** the system counts all non-empty cells in each group
- **AND** includes both numeric and non-numeric values

#### Scenario: AVERAGE aggregation
- **WHEN** a value field is configured with AVERAGE aggregation
- **THEN** the system calculates the mean of all numeric values in each group
- **AND** ignores non-numeric values
- **AND** returns null if no numeric values exist

#### Scenario: MIN aggregation
- **WHEN** a value field is configured with MIN aggregation
- **THEN** the system finds the minimum numeric value in each group
- **AND** ignores non-numeric values
- **AND** returns null if no numeric values exist

#### Scenario: MAX aggregation
- **WHEN** a value field is configured with MAX aggregation
- **THEN** the system finds the maximum numeric value in each group
- **AND** ignores non-numeric values
- **AND** returns null if no numeric values exist

