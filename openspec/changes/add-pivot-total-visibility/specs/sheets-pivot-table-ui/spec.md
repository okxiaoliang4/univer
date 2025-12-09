## ADDED Requirements
### Requirement: Row/Column Grand Total Checkboxes
The pivot table editor SHALL provide row and column grand total checkboxes in the rowFields and columnFields areas, mapping directly to the first row/column field `showSubTotals` (no separate grand total flags).

#### Scenario: Render grand total checkboxes
- **WHEN** the pivot table editor renders the rowFields and columnFields areas
- **THEN** each area displays a grand total checkbox labeled to indicate row or column totals
- **AND** the checkbox default state reflects the first row/column field `showSubTotals`

#### Scenario: Toggle row grand total
- **WHEN** a user toggles the row grand total checkbox
- **THEN** the editor updates the first row field `showSubTotals`
- **AND** triggers recalculation/render so row grand totals appear/disappear

#### Scenario: Toggle column grand total
- **WHEN** a user toggles the column grand total checkbox
- **THEN** the editor updates the first column field `showSubTotals`
- **AND** triggers recalculation/render so column grand totals appear/disappear

#### Scenario: Sync checkbox state from configuration changes
- **WHEN** the pivot configuration changes externally (e.g., loading saved pivot)
- **THEN** the row and column total checkboxes update to match the first row/column field `showSubTotals`

#### Scenario: Preserve other field subtotals
- **WHEN** the grand total checkbox is toggled
- **THEN** only the first row/column field `showSubTotals` is changed
- **AND** other row/column fields retain their `showSubTotals` values for their own subtotals
