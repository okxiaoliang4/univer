## ADDED Requirements
### Requirement: Field-level subtotal semantics
The first row/column field `showSubTotals` flag SHALL represent the table’s grand total toggles, while subsequent fields’ `showSubTotals` flags continue to represent their own subtotals (no separate grand total flags).

#### Scenario: First row field controls row grand total
- **WHEN** the first row field has `showSubTotals: true`
- **THEN** `PivotEngineV2` generates row grand total rows
- **AND** setting that flag to `false` removes row grand totals but keeps lower-level subtotals according to later row fields

#### Scenario: First column field controls column grand total
- **WHEN** the first column field has `showSubTotals: true`
- **THEN** `PivotEngineV2` generates column grand total columns
- **AND** setting that flag to `false` removes column grand totals but keeps lower-level subtotals according to later column fields

#### Scenario: Subsequent fields keep subtotal behavior
- **WHEN** row or column fields after the first have `showSubTotals: true`
- **THEN** `PivotEngineV2` generates subtotals for those fields without affecting grand totals

#### Scenario: Multi-level row subtotals mapping example
- **GIVEN** three row fields `[Region, Category, Quarter]`
- **WHEN** `Region.showSubTotals = true`
- **THEN** the engine outputs grand total rows for the whole table
- **WHEN** `Category.showSubTotals = true`
- **THEN** the engine outputs subtotals for each Category within its Region
- **WHEN** `Quarter.showSubTotals = true`
- **THEN** the engine outputs subtotals for each Quarter within its Region + Category grouping (does not create another grand total)

### Requirement: Column grand total per value field
Column grand totals SHALL output one grand total column per value field (mirroring Google Sheets behavior) instead of sharing a single grand total column across all value fields.

#### Scenario: Append grand total columns per value field
- **WHEN** column grand totals are enabled and multiple value fields exist
- **THEN** the structure includes one additional column per value field at the end of the column axis
- **AND** column headers/metadata align each grand total column with its corresponding value field
- **AND** dimensions and value arrays reflect the expanded column count
