## ADDED Requirements
### Requirement: Pivot output cell expand/collapse control

The system SHALL render an expand/collapse control inside pivot output cells that represent collapsible row or column groups, using the graphics renderer, and update pivot output when toggled.

#### Scenario: Render control for collapsible row group cell
- **WHEN** a pivot output cell belongs to a row group that has child rows
- **AND** the group is currently expanded or collapsed
- **THEN** the UI overlays an expand/collapse button inside that cell via the graphics renderer registered on the sheet
- **AND** the button is positioned so it does not obstruct the cell value
- **AND** no control is rendered for leaf rows, subtotal-only rows, or grand total rows

#### Scenario: Render control for collapsible column group cell
- **WHEN** a pivot output cell belongs to a column group that has child columns
- **AND** the group is currently expanded or collapsed
- **THEN** the UI overlays an expand/collapse button inside that cell via the graphics renderer registered on the sheet
- **AND** the button is positioned so it does not obstruct the cell value
- **AND** no control is rendered for leaf columns, subtotal-only columns, or grand total columns

#### Scenario: Click toggles group and updates pivot output
- **WHEN** a user clicks the expand/collapse button in a pivot output cell
- **THEN** the system toggles the corresponding row or column group state
- **AND** recalculates or refreshes pivot output and visible indices to reflect the new state
- **AND** re-renders the pivot output so the button state matches the updated group visibility

#### Scenario: Graphics renderer registration for pivot UI
- **WHEN** the pivot table UI initializes rendering
- **THEN** it registers the expand/collapse graphics renderer once with the sheet graphics extension
- **AND** the renderer works across normal view and printing contexts without breaking selection or cell hit-testing

