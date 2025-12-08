# sheets-pivot-table-ui Specification

## Purpose
TBD - created by archiving change optimize-pivot-table-field-configuration-ui. Update Purpose after archive.
## Requirements
### Requirement: Source Field Checkbox Controls

The system SHALL provide checkbox controls on source fields that allow users to quickly add fields to pivot table configuration areas (rows, columns, values, filters) with intelligent placement.

#### Scenario: Checkbox click adds field with intelligent placement
- **WHEN** a user clicks a checkbox on a source field
- **THEN** the system determines the target area using intelligent placement logic
- **AND** adds the field to the determined area (rowFields, columnFields, or valueFields)
- **AND** the checkbox appears visually checked or highlighted to indicate the field has been added
- **AND** if the field is already in an exclusive area (rowFields/columnFields), it is moved to the new area

#### Scenario: Checkbox visual state reflects field status
- **WHEN** a source field is already added to any configuration area
- **THEN** the checkbox displays a checked or highlighted state
- **AND** clicking the checkbox again removes the field from all areas and returns it to source only

### Requirement: Intelligent Field Placement Logic

The system SHALL automatically determine the appropriate placement area for fields based on data type detection, fuzzy matching rules, and user preferences stored in browser storage.

#### Scenario: Numeric field placed in valueFields
- **WHEN** a source field contains numeric data (all values are numbers or numeric strings)
- **AND** no fuzzy matching rule applies
- **AND** no user preference exists for this field
- **THEN** the field is placed in valueFields area
- **AND** a default aggregation method (e.g., SUM) is assigned

#### Scenario: Text field placed in rowFields
- **WHEN** a source field contains text data (non-numeric values)
- **AND** no fuzzy matching rule applies
- **AND** no user preference exists for this field
- **THEN** the field is placed in rowFields area

#### Scenario: Fuzzy matching rule overrides data type
- **WHEN** a source field name matches a fuzzy matching rule (case-insensitive)
- **AND** the rule specifies a target type (e.g., `{match: 'userId', type: 'row'}`)
- **THEN** the field is placed in the specified area (rowFields or columnFields)
- **AND** data type detection is ignored for this field
- **AND** the rule takes precedence over default data type logic

#### Scenario: User preference overrides fuzzy matching and data type
- **WHEN** a user has previously moved a field from one area to another (e.g., userId from rowFields to columnFields)
- **AND** this preference is stored in browser storage
- **THEN** the stored preference takes precedence over fuzzy matching rules and data type detection
- **AND** the field is placed according to the user's previous choice

#### Scenario: Preference priority order
- **WHEN** determining field placement
- **THEN** the system checks in this order: (1) user preference from storage, (2) fuzzy matching rules, (3) data type detection
- **AND** the first matching rule determines placement

### Requirement: Fuzzy Matching Configuration System

The system SHALL support configurable fuzzy matching rules that allow field name-based placement overrides, with a default configuration set and the ability to store user-defined rules.

#### Scenario: Default fuzzy matching configuration
- **WHEN** the system initializes
- **THEN** a default set of fuzzy matching rules is available
- **AND** rules are stored in a configuration object format: `[{match: 'userId', type: 'row'}, ...]`
- **AND** matching is case-insensitive
- **AND** rules can specify 'row' or 'column' as target types

#### Scenario: Case-insensitive field name matching
- **WHEN** a source field name is "UserId" or "USERID" or "userId"
- **AND** a fuzzy rule exists with `match: 'userId'`
- **THEN** the rule matches regardless of case
- **AND** the field is placed according to the rule's type

#### Scenario: User-defined fuzzy matching rules
- **WHEN** a user moves a field between areas (e.g., from rowFields to columnFields)
- **THEN** a fuzzy matching rule is automatically created or updated in browser storage
- **AND** the rule uses the field name (case-insensitive) as the match pattern
- **AND** the rule specifies the target type based on the user's action

### Requirement: Browser Storage for User Preferences

The system SHALL persist user field placement preferences in browser storage (localStorage) to remember user choices across sessions.

#### Scenario: Store field placement preference
- **WHEN** a user moves a field from one area to another (e.g., userId from rowFields to columnFields)
- **THEN** the system stores a preference entry in localStorage
- **AND** the entry maps the field name (or sourceColumnIndex) to the target area type
- **AND** the preference persists across browser sessions

#### Scenario: Load user preferences on initialization
- **WHEN** the pivot table editor initializes
- **THEN** the system loads user preferences from localStorage
- **AND** applies preferences when determining field placement
- **AND** preferences take precedence over fuzzy matching and data type detection

#### Scenario: Update preference on field move
- **WHEN** a user manually moves a field between areas via drag-and-drop
- **THEN** the system updates the preference in localStorage
- **AND** subsequent checkbox clicks use the updated preference

#### Scenario: Clear preference when field removed
- **WHEN** a user removes a field from all configuration areas (returns to source only)
- **THEN** the system removes the preference entry from localStorage
- **AND** subsequent additions use fuzzy matching or data type detection

### Requirement: Add Button with Dropdown Menu

The system SHALL provide an "Add" button in the top-right corner of each field configuration area (rowFields, columnFields, valueFields, filterFields) that opens a dropdown menu for selecting fields to add.

#### Scenario: Add button appears in field configuration areas
- **WHEN** the pivot table editor renders
- **THEN** an "Add" button appears in the top-right corner of each field configuration area header
- **AND** the button is positioned similar to Google Sheets pivot table interface
- **AND** clicking the button opens a dropdown menu

#### Scenario: Dropdown shows available fields for rowFields
- **WHEN** a user clicks the "Add" button in the rowFields area
- **THEN** a dropdown menu opens showing all source fields
- **AND** fields already added to rowFields or columnFields are excluded from the list
- **AND** selecting a field adds it to rowFields
- **AND** if the field exists in columnFields, it is moved to rowFields (exclusive rule)

#### Scenario: Dropdown shows available fields for columnFields
- **WHEN** a user clicks the "Add" button in the columnFields area
- **THEN** a dropdown menu opens showing all source fields
- **AND** fields already added to rowFields or columnFields are excluded from the list
- **AND** selecting a field adds it to columnFields
- **AND** if the field exists in rowFields, it is moved to columnFields (exclusive rule)

#### Scenario: Dropdown shows all fields for valueFields
- **WHEN** a user clicks the "Add" button in the valueFields area
- **THEN** a dropdown menu opens showing all source fields
- **AND** all fields are shown even if they are already added to valueFields (allowing duplicates)
- **AND** selecting a field adds a new instance to valueFields

#### Scenario: Dropdown shows available fields for filterFields
- **WHEN** a user clicks the "Add" button in the filterFields area
- **THEN** a dropdown menu opens showing all source fields
- **AND** fields already added to filterFields are excluded from the list
- **AND** selecting a field adds it to filterFields

#### Scenario: Dropdown uses Dropdown component
- **WHEN** the "Add" button is clicked
- **THEN** the dropdown menu is rendered using the `@univerjs/design` Dropdown component
- **AND** the dropdown displays field names in a selectable list
- **AND** selecting a field closes the dropdown and adds the field

### Requirement: Field Type Detection

The system SHALL detect whether a source field contains numeric or text data to inform intelligent placement decisions.

#### Scenario: Detect numeric field
- **WHEN** analyzing source field data
- **AND** all non-empty values in the column are numeric (numbers or numeric strings)
- **THEN** the field is classified as numeric
- **AND** empty cells are ignored in the classification

#### Scenario: Detect text field
- **WHEN** analyzing source field data
- **AND** any non-empty value in the column is non-numeric (text, dates, mixed content)
- **THEN** the field is classified as text
- **AND** numeric fields default to valueFields placement

#### Scenario: Handle empty or mixed data
- **WHEN** a source field has no data or mixed numeric/text data
- **THEN** the field defaults to text classification
- **AND** is placed in rowFields by default

