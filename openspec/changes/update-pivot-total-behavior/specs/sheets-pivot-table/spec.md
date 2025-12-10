## MODIFIED Requirements
### Requirement: Field Configuration

The system SHALL support configuring pivot table fields in four areas: row fields, column fields, value fields, and filter fields. Each row/column field MAY specify `showSubTotals`, which drives subtotal generation at that field’s grouping level. The first row/column field `showSubTotals` flag controls the table-level grand total for that axis; deeper fields control subtotals inside their parent group without adding additional table-level totals.

#### Scenario: Add field to row area with subtotal configuration
- **WHEN** a user adds a source column to the row fields area
- **AND** sets `showSubTotals: true` on the field
- **THEN** the pivot table stores the field ID in the rowFields array
- **AND** stores the `showSubTotals` configuration
- **AND** marks the pivot table as dirty for recalculation
- **AND** `PivotEngineV2` generates subtotal rows for each unique value of that field, placing the subtotal row immediately after the group with label `<member> 总计`
- **AND** if the field is the first row field, its `showSubTotals` controls the presence of the table-level grand total row.

#### Scenario: Add field to column area with subtotal configuration
- **WHEN** a user adds a source column to the column fields area
- **AND** sets `showSubTotals: true` on the field
- **THEN** the pivot table stores the field ID in the columnFields array
- **AND** stores the `showSubTotals` configuration
- **AND** marks the pivot table as dirty for recalculation
- **AND** `PivotEngineV2` generates subtotal columns for each unique value of that field, placing the subtotal column immediately after the group with label `<member> 总计`
- **AND** if the field is the first column field, its `showSubTotals` controls the presence of the table-level grand total columns.

#### Scenario: Multi-level row subtotal toggles
- **WHEN** row fields are configured in order (e.g., `Region`, `Quarter`, `Channel`)
- **AND** `Region.showSubTotals = true`, `Quarter.showSubTotals = true`, `Channel.showSubTotals = false`
- **THEN** the engine emits quarter-level subtotal rows labeled `<Quarter> 总计` within each Region group
- **AND** emits Region grand total rows (and overall table grand total row) controlled by `Region.showSubTotals`
- **AND** disabling `Region.showSubTotals` removes table/Region grand totals while retaining quarter subtotals when `Quarter.showSubTotals` remains true.

#### Scenario: Multi-level column subtotal toggles
- **WHEN** column fields are configured in order (e.g., `Category`, `Channel`)
- **AND** `Category.showSubTotals = true`, `Channel.showSubTotals = true`
- **THEN** the engine emits Channel-level subtotal columns labeled `<Channel> 总计` within each Category group
- **AND** appends Category-level grand total columns controlled by `Category.showSubTotals`
- **AND** disabling `Category.showSubTotals` removes Category grand total columns while keeping Channel subtotals when `Channel.showSubTotals` remains true.

### Requirement: PivotEngineV2 Cross-Tabulation Calculation Engine

The system SHALL provide a `PivotEngineV2` class that calculates pivot table results in a structured Cross-Tabulation format optimized for UI rendering.

#### Scenario: Calculate Cross-Tabulation with structured output
- **WHEN** `PivotEngineV2` is instantiated with `IPivotTableCrossTabConfig` containing row fields, column fields, value fields, and source data
- **AND** `getCalculatedData()` is called
- **THEN** the engine returns `IPivotTableCrossTabData` with structured output including:
  - `isEmpty` flag indicating if data is empty
  - `dimensions` object with row/column counts
  - `structure.rowHeaders` array with multi-level row values
  - `structure.columnHeaders` array with multi-level column values
  - `structure.values` 3D array `[rowIndex][columnIndex][valueFieldIndex]`
  - `structure.rowTypes` and `structure.columnTypes` arrays
  - `structure.subtotalRows` and `structure.subtotalColumns` metadata
  - `structure.rowGroups` and `structure.columnGroups` for collapse/expand
  - `structure.rowLevelMap` and `structure.columnLevelMap` for quick lookup

#### Scenario: Field-level subtotal configuration
- **WHEN** a row field or column field has `showSubTotals: true`
- **THEN** the engine generates subtotal rows/columns for each unique value of that field at its grouping level
- **AND** subtotal labels follow `<member> 总计` (rows or columns) while table-level totals use `总计`
- **AND** the first row field’s flag controls whether table-level grand total rows are emitted; the first column field’s flag controls table-level grand total columns
- **AND** deeper fields only add subtotals inside their parent groups without adding extra table-level totals.

#### Scenario: Column subtotal expansion per value field
- **WHEN** column fields with `showSubTotals: true` exist
- **AND** multiple value fields are configured
- **THEN** the engine appends subtotal/grand-total columns per value field for that column grouping (including table-level grand totals when the first column field is enabled)
- **AND** each subtotal/grand-total column aligns to the corresponding value field index in `columnHeaders`, `columnTypes`, `structure.values`, and `dimensions.totalColumns`.

#### Scenario: Multi-level row and column fields
- **WHEN** multiple row fields or column fields are configured
- **THEN** the engine creates hierarchical grouping
- **AND** generates proper multi-level headers in `rowHeaders` and `columnHeaders`
- **AND** creates parent-child relationships in `rowGroups` and `columnGroups`
- **AND** populates `rowLevelMap` and `columnLevelMap` with all group IDs for each row/column
- **AND** inserts subtotal rows/columns after each group for any field whose `showSubTotals` is true.

#### Scenario: Multi-value field support
- **WHEN** multiple value fields are configured
- **THEN** the engine generates a 3D values array `[rowIndex][columnIndex][valueFieldIndex]`
- **AND** includes `valueFieldHeaders` array with field names
- **AND** each cell contains values for all configured value fields

#### Scenario: Empty data detection
- **WHEN** source data has no value fields configured
- **OR** all source data is filtered out
- **OR** all calculated values are null/empty
- **THEN** the engine sets `isEmpty: true` in the output
- **AND** still returns valid structure with empty arrays

#### Scenario: Dimension calculation
- **WHEN** calculation completes
- **THEN** the engine populates `dimensions` object with:
  - `totalRows`: total number of rows including subtotals and grand totals (first-row-field controlled)
  - `totalColumns`: total number of columns including subtotal/grand-total columns and one column per value field for each subtotal/grand total on the column axis
  - `dataRowCount`: number of data rows (excluding subtotals)
  - `dataColumnCount`: number of data columns (excluding subtotals)
  - `valueFieldCount`: number of configured value fields

#### Scenario: Individual setter methods
- **WHEN** `setRowFields()`, `setColumnFields()`, `setValueFields()`, `setFilterFields()`, or `setSourceData()` is called
- **THEN** the engine updates the corresponding configuration field
- **AND** marks the engine as dirty (`_isDirty = true`)
- **AND** invalidates cached calculation result

#### Scenario: Caching mechanism
- **WHEN** `getCalculatedData()` is called multiple times without configuration changes
- **THEN** the engine returns the cached result
- **AND** does not recalculate
- **WHEN** any setter method is called
- **THEN** the engine marks itself as dirty
- **AND** recalculates on next `getCalculatedData()` call

