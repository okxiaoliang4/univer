## ADDED Requirements
### Requirement: Pivot row/column sorting
`PivotEngineV2` SHALL produce deterministically sorted row and column outputs, aligning headers and value matrices by field content by default and supporting configurable value-field-based sorting.

#### Scenario: Default field-value sorting
- **WHEN** rowFields or columnFields are configured without an explicit sort rule
- **THEN** row headers and column headers are sorted by their field values in stable ascending order (e.g., `Quarter` renders as `Q1`, `Q2`, `Q3`, `Q4`)
- **AND** the corresponding `structure.values` matrix aligns with the sorted headers
- **AND** subtotal/grand total rows or columns remain anchored after their sorted data groups.

#### Scenario: Column field sorting by content
- **WHEN** columnFields include ordinal-like values (e.g., `Quarter`, `Month`, custom labels)
- **THEN** column header levels follow the ascending content order for each column field
- **AND** subtotal columns for a field stay after that field’s sorted members
- **AND** value-position switching (ROW or COLUMN) preserves the sorted header order.

#### Scenario: Sort rows by selected value field aggregate
- **WHEN** the configuration specifies a row sort rule of type `valueField` with a `valueFieldId` drawn from configured valueFields (e.g., Sales)
- **THEN** `PivotEngineV2` orders row groups by the aggregated result of that value field before inserting subtotal or grand total rows
- **AND** ties fall back to the default field-value sort for determinism
- **AND** sort direction defaults to ascending unless an explicit direction (asc|desc) is provided.

#### Scenario: Sort columns by selected value field aggregate
- **WHEN** the configuration specifies a column sort rule of type `valueField` with a `valueFieldId` drawn from configured valueFields
- **THEN** column groups are ordered by the aggregated result for that value field prior to inserting subtotal or grand total columns
- **AND** ties fall back to field-value sorting
- **AND** the chosen direction (default ascending; support desc) is applied consistently across header levels.

#### Scenario: Multiple value field sort options
- **WHEN** multiple value fields exist (e.g., Sales, Count, Average)
- **THEN** the sort configuration accepts any `valueFieldId` from the valueFields list as the sort key for rows or columns
- **AND** when no `valueFieldId` is provided, the engine reverts to default field-value sorting for that axis.

