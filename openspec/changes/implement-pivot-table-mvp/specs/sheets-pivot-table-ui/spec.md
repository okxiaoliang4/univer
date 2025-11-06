# Sheets Pivot Table UI Plugin Specification

## ADDED Requirements

### Requirement: Pivot Table Creation Dialog

The system SHALL provide a dialog for creating new pivot tables.

#### Scenario: Open creation dialog
- **WHEN** user selects "Insert > Pivot Table" from menu or uses keyboard shortcut
- **THEN** the system displays the pivot table creation dialog
- **AND** pre-populates the data range with the current selection (if any)
- **AND** focuses the data range input field

#### Scenario: Select data source range
- **WHEN** user enters or selects a data range in the creation dialog
- **THEN** the system validates the range format
- **AND** displays validation errors if the range is invalid
- **AND** highlights the range in the worksheet (if valid)

#### Scenario: Specify target location
- **WHEN** user specifies a target cell for the pivot table output
- **THEN** the system validates the target location
- **AND** ensures it does not overlap with existing pivot tables or critical ranges
- **AND** displays an error message if overlap detected

#### Scenario: Create pivot table from dialog
- **WHEN** user confirms creation with valid data range and target location
- **THEN** the system executes CreatePivotTableCommand
- **AND** closes the creation dialog
- **AND** opens the field configuration panel
- **AND** displays a success message

#### Scenario: Cancel creation dialog
- **WHEN** user clicks Cancel or presses Escape in the creation dialog
- **THEN** the system closes the dialog without creating a pivot table
- **AND** clears any temporary selections or highlights

### Requirement: Field Configuration Panel

The system SHALL provide a panel for configuring pivot table fields.

#### Scenario: Display available fields
- **WHEN** the field configuration panel opens for a pivot table
- **THEN** the system displays a list of available fields from the source data headers
- **AND** shows each field with its source column name
- **AND** allows dragging fields to configuration areas

#### Scenario: Drag field to row area
- **WHEN** user drags a field from the available fields list to the row area
- **THEN** the system adds the field to the row fields configuration
- **AND** displays the field in the row area drop zone
- **AND** updates the pivot table configuration
- **AND** triggers recalculation

#### Scenario: Drag field to column area
- **WHEN** user drags a field from the available fields list to the column area
- **THEN** the system adds the field to the column fields configuration
- **AND** displays the field in the column area drop zone
- **AND** updates the pivot table configuration
- **AND** triggers recalculation

#### Scenario: Drag field to value area
- **WHEN** user drags a field from the available fields list to the value area
- **THEN** the system adds the field to the value fields configuration with default SUM aggregation
- **AND** displays the field in the value area with aggregation function label
- **AND** updates the pivot table configuration
- **AND** triggers recalculation

#### Scenario: Drag field to filter area
- **WHEN** user drags a field from the available fields list to the filter area
- **THEN** the system adds the field to the filter fields configuration
- **AND** displays the field in the filter area drop zone
- **AND** updates the pivot table configuration

#### Scenario: Reorder fields within area
- **WHEN** user drags a field to a different position within the same area
- **THEN** the system reorders the field in the configuration
- **AND** updates the pivot table layout accordingly
- **AND** triggers recalculation

#### Scenario: Remove field from area
- **WHEN** user clicks the remove button on a field in a configuration area
- **THEN** the system removes the field from that area
- **AND** returns the field to the available fields list
- **AND** updates the pivot table configuration
- **AND** triggers recalculation

#### Scenario: Change aggregation function
- **WHEN** user clicks the aggregation button on a value field
- **THEN** the system displays a dropdown menu with aggregation options (SUM, COUNT, AVERAGE, MIN, MAX)
- **AND** highlights the currently selected aggregation
- **WHEN** user selects a different aggregation function
- **THEN** the system updates the field's aggregation type
- **AND** triggers recalculation

### Requirement: Menu Integration

The system SHALL integrate pivot table actions into the Univer menu system.

#### Scenario: Insert menu item
- **WHEN** the plugin initializes
- **THEN** the system registers "Pivot Table" menu item under the "Insert" menu
- **AND** the menu item triggers the creation dialog when clicked

#### Scenario: Context menu on selection
- **WHEN** user right-clicks a cell range
- **THEN** the system displays "Create Pivot Table" in the context menu
- **AND** clicking it opens the creation dialog with the selected range pre-filled

#### Scenario: Context menu on pivot table
- **WHEN** user right-clicks a cell within a pivot table
- **THEN** the system displays pivot table-specific context menu items
- **AND** includes "Refresh Pivot Table", "Edit Pivot Table", "Delete Pivot Table"

### Requirement: Keyboard Shortcuts

The system SHALL provide keyboard shortcuts for common pivot table operations.

#### Scenario: Shortcut to create pivot table
- **WHEN** user presses the configured keyboard shortcut for pivot table creation
- **THEN** the system opens the pivot table creation dialog
- **AND** pre-fills the data range with the current selection

#### Scenario: Shortcut to refresh pivot table
- **WHEN** user presses the configured keyboard shortcut while focused on a pivot table cell
- **THEN** the system refreshes the pivot table
- **AND** displays a brief success message

### Requirement: Pivot Table Panel Service

The system SHALL provide a PivotTablePanelService to manage panel state and interactions.

#### Scenario: Open panel for pivot table
- **WHEN** a component requests to open the panel for a specific pivot table ID
- **THEN** the service sets the active pivot table
- **AND** emits an event to display the panel
- **AND** loads the current field configuration

#### Scenario: Close panel
- **WHEN** user closes the pivot table panel
- **THEN** the service clears the active pivot table reference
- **AND** emits an event to hide the panel

#### Scenario: Update field configuration through panel
- **WHEN** user makes changes in the panel (add/remove/reorder fields)
- **THEN** the service updates the field configuration
- **AND** executes UpdatePivotTableFieldsCommand
- **AND** emits a configuration change event

#### Scenario: Track panel visibility state
- **WHEN** a component queries the panel visibility state
- **THEN** the service returns whether the panel is currently open
- **AND** returns the active pivot table ID if any

### Requirement: Desktop UI Controller

The system SHALL provide a PivotTableUIDesktopController to coordinate UI interactions.

#### Scenario: Register UI components on startup
- **WHEN** the controller starts
- **THEN** it registers all menu items, shortcuts, and commands
- **AND** subscribes to panel service events
- **AND** initializes render modules

#### Scenario: Handle create pivot table command
- **WHEN** OpenCreatePivotTableDialogCommand is executed
- **THEN** the controller opens the creation dialog
- **AND** initializes dialog state with current selection

#### Scenario: Handle open panel command
- **WHEN** OpenPivotTablePanelCommand is executed with a pivot table ID
- **THEN** the controller calls the panel service to open the panel
- **AND** ensures the panel is visible

#### Scenario: Coordinate with render modules
- **WHEN** pivot table data changes
- **THEN** the controller notifies render modules
- **AND** triggers re-rendering of affected pivot tables

### Requirement: Pivot Table Rendering

The system SHALL render pivot table results in the worksheet with appropriate styling.

#### Scenario: Render pivot table headers
- **WHEN** a pivot table is calculated and rendered
- **THEN** the system applies header styling to row and column headers
- **AND** uses bold font for headers
- **AND** applies header background color (configurable theme)

#### Scenario: Render pivot table data cells
- **WHEN** pivot table data cells are rendered
- **THEN** the system displays aggregated values with appropriate number formatting
- **AND** applies data cell styling (theme-dependent)

#### Scenario: Render grand total row
- **WHEN** pivot table includes grand totals
- **THEN** the system renders the grand total row below the data
- **AND** applies total styling (bold font, distinct background color)

#### Scenario: Render pivot table boundary markers
- **WHEN** a pivot table is rendered
- **THEN** the system displays a visual indicator (corner icon) at the top-left of the pivot table
- **AND** clicking the icon opens the field configuration panel

#### Scenario: Update rendering on pivot table change
- **WHEN** pivot table configuration or source data changes
- **THEN** the render controller recalculates the pivot table
- **AND** updates only the affected cells
- **AND** maintains styling consistency

#### Scenario: Handle pivot table selection
- **WHEN** user selects cells within a pivot table
- **THEN** the system highlights the selected cells
- **AND** displays pivot table context menu on right-click

### Requirement: Localization Support

The system SHALL support multiple languages for pivot table UI elements.

#### Scenario: English (en-US) localization
- **WHEN** the system language is set to English
- **THEN** all UI text displays in English
- **AND** includes menu items, dialog labels, button text, and error messages

#### Scenario: Chinese (zh-CN) localization
- **WHEN** the system language is set to Chinese
- **THEN** all UI text displays in Chinese
- **AND** includes menu items, dialog labels, button text, and error messages

#### Scenario: Aggregation function name localization
- **WHEN** aggregation function names are displayed
- **THEN** they appear in the current system language
- **AND** use standard terminology (e.g., "SUM", "求和")

### Requirement: UI Commands

The system SHALL provide UI-specific commands for pivot table interactions.

#### Scenario: OpenCreatePivotTableDialogCommand execution
- **WHEN** OpenCreatePivotTableDialogCommand is executed
- **THEN** the command opens the creation dialog
- **AND** initializes with current selection or empty state

#### Scenario: OpenPivotTablePanelCommand execution
- **WHEN** OpenPivotTablePanelCommand is executed with a pivot table ID
- **THEN** the command opens the field configuration panel for that pivot table
- **AND** loads current configuration

#### Scenario: TogglePivotTableFieldCommand execution
- **WHEN** TogglePivotTableFieldCommand is executed with field ID and area
- **THEN** the command adds or removes the field from the specified area
- **AND** updates the pivot table configuration

### Requirement: UI Operations

The system SHALL provide operations to manage UI state changes.

#### Scenario: ShowPivotTablePanelOperation execution
- **WHEN** ShowPivotTablePanelOperation is executed
- **THEN** the operation updates UI state to show the panel
- **AND** does not modify pivot table data

#### Scenario: HidePivotTablePanelOperation execution
- **WHEN** HidePivotTablePanelOperation is executed
- **THEN** the operation updates UI state to hide the panel
- **AND** does not modify pivot table data

#### Scenario: UpdatePivotTableFieldsOperation execution
- **WHEN** UpdatePivotTableFieldsOperation is executed with new field configuration
- **THEN** the operation updates the panel's displayed field configuration
- **AND** synchronizes with the underlying pivot table state

### Requirement: Error Handling and Validation

The system SHALL provide clear error messages and validation for user inputs.

#### Scenario: Invalid data range error
- **WHEN** user enters an invalid data range in the creation dialog
- **THEN** the system displays an error message "Invalid range format"
- **AND** disables the Create button
- **AND** highlights the invalid input field

#### Scenario: Overlapping pivot table error
- **WHEN** user specifies a target location that overlaps with an existing pivot table
- **THEN** the system displays an error message "Target location overlaps with existing pivot table"
- **AND** suggests an alternative location or prompts to choose a different location

#### Scenario: Empty data range warning
- **WHEN** user attempts to create a pivot table with an empty or single-row range
- **THEN** the system displays a warning message "Data range must contain at least one header row and one data row"
- **AND** prevents creation

#### Scenario: No value fields warning
- **WHEN** user attempts to configure a pivot table without any value fields
- **THEN** the system displays a warning message "At least one value field is required"
- **AND** disables refresh or apply actions

### Requirement: Configuration Schema

The system SHALL provide a configuration schema for the UI plugin with menu customization options.

#### Scenario: Default UI configuration
- **WHEN** the plugin is instantiated without custom UI configuration
- **THEN** the system uses default menu items, shortcuts, and settings
- **AND** all UI features are enabled

#### Scenario: Custom menu configuration
- **WHEN** the plugin is instantiated with custom menu configuration
- **THEN** the system merges custom menu items with defaults
- **AND** applies the merged configuration to the menu system

#### Scenario: Disable menu items
- **WHEN** menu configuration disables specific pivot table menu items
- **THEN** those menu items do not appear in the menu
- **AND** corresponding shortcuts are also disabled

