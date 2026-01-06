## MODIFIED Requirements

### Requirement: Mobile Data Validation Dropdown Trigger

The system SHALL allow users to open data validation dropdown menus by tapping anywhere on a cell on mobile platforms, not just the dropdown icon.

#### Scenario: Tapping cell with dropdown opens menu
- **GIVEN** the user is on a mobile device
- **WHEN** the user taps anywhere on a cell that has a data validation dropdown
- **THEN** the dropdown menu is automatically opened
- **AND** the dropdown displays the available options for that cell

#### Scenario: Tapping cell without dropdown does nothing
- **GIVEN** the user is on a mobile device
- **WHEN** the user taps on a cell that does not have a data validation dropdown
- **THEN** no dropdown menu is shown
- **AND** normal cell selection/editing behavior occurs

#### Scenario: Dropdown icon still works
- **GIVEN** the user is on a mobile device
- **WHEN** the user taps specifically on the dropdown icon
- **THEN** the dropdown menu opens (same as whole-cell tap)
- **AND** the behavior is consistent with whole-cell tap

#### Scenario: Dropdown closes appropriately
- **GIVEN** the dropdown menu is open on mobile
- **WHEN** the user selects an option or taps outside the dropdown
- **THEN** the dropdown menu closes
- **AND** the cell value is updated if an option was selected

#### Scenario: Desktop behavior unchanged
- **GIVEN** the user is on a desktop device
- **WHEN** the user clicks on a cell with data validation dropdown
- **THEN** the dropdown icon must still be clicked to open the dropdown
- **AND** whole-cell click does not open dropdown (desktop behavior preserved)
