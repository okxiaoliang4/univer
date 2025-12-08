## ADDED Requirements

### Requirement: Pivot Table Value Position Support

The system SHALL allow users to control the placement of value fields in a pivot table by switching between row-based and column-based positioning, enabling flexible data analysis layouts.

Value position support is implemented in `PivotEngineV2`:
- **Position Control**: `valuePosition` property controls whether value fields appear as row or column headers
- **Position Modes**:
  - `PivotValuePosition.COLUMN`: Value fields appear as column headers (values fill columns)
  - `PivotValuePosition.ROW`: Value fields appear as row headers (values fill rows)
- **Dynamic Switching**: `setValuePosition()` method allows runtime switching between positions with automatic recalculation
- **Structure Adaptation**: Cross-tabulation output structure automatically adapts to position changes
- **Test Coverage**: Comprehensive test cases validate all position combinations and transitions

#### Scenario: Basic pivot with values in column position
- **WHEN** a pivot table is created with `valuePosition: COLUMN`
- **AND** contains row fields, column fields, and value fields
- **THEN** the output structure has value fields as column headers
- **AND** values are organized in columns in the data matrix
- **AND** column count includes space for value field columns

#### Scenario: Basic pivot with values in row position
- **WHEN** a pivot table is created with `valuePosition: ROW`
- **AND** contains the same row fields, column fields, and value fields as COLUMN position pivot
- **THEN** the output structure has value fields as row headers
- **AND** values are organized in rows in the data matrix
- **AND** row count includes space for value field rows
- **AND** column structure is simplified without value headers

#### Scenario: Switching value position from COLUMN to ROW
- **WHEN** a pivot table with `valuePosition: COLUMN` exists
- **AND** `setValuePosition(PivotValuePosition.ROW)` is called
- **THEN** the engine marks itself as dirty (requires recalculation)
- **AND** the next call to `getCalculatedData()` recalculates with ROW position
- **AND** the output structure changes to have values in rows
- **AND** dimensions correctly reflect the new structure (rows and columns swap)

#### Scenario: Switching value position from ROW to COLUMN
- **WHEN** a pivot table with `valuePosition: ROW` exists
- **AND** `setValuePosition(PivotValuePosition.COLUMN)` is called
- **THEN** the engine marks itself as dirty
- **AND** the output recalculates with COLUMN position
- **AND** values return to column-based organization
- **AND** render model receives correctly structured data

#### Scenario: No change when setting same position
- **WHEN** a pivot table already has `valuePosition: COLUMN`
- **AND** `setValuePosition(PivotValuePosition.COLUMN)` is called
- **THEN** the position property is updated to the same value
- **AND** no unnecessary recalculation is triggered (engine may optimize this)
- **AND** existing calculated data remains valid

#### Scenario: Multiple value fields with ROW position
- **WHEN** a pivot table has multiple value fields (e.g., Sum, Count, Average)
- **AND** `valuePosition: ROW` is set
- **THEN** each value field appears as a distinct row in the output
- **AND** aggregated values for each field are in separate rows
- **AND** row headers show the appropriate value field labels

#### Scenario: Multiple value fields with COLUMN position
- **WHEN** a pivot table has multiple value fields
- **AND** `valuePosition: COLUMN` is set
- **THEN** each value field appears as a distinct column
- **AND** column headers show the appropriate value field labels
- **AND** values are laid out with one column per value field

#### Scenario: Row-only pivot with valuePosition ROW
- **WHEN** a pivot contains only row fields and value fields (no column fields)
- **AND** `valuePosition: ROW` is set
- **THEN** the output still functions correctly
- **AND** values appear in the single available position
- **AND** structure is 1D with row organization

#### Scenario: Row-only pivot with valuePosition COLUMN
- **WHEN** a pivot contains only row fields and value fields (no column fields)
- **AND** `valuePosition: COLUMN` is set
- **THEN** the output still functions correctly
- **AND** values are aggregated into a single column
- **AND** structure is valid despite limited dimensionality

#### Scenario: Column-only pivot with valuePosition ROW
- **WHEN** a pivot contains only column fields and value fields (no row fields)
- **AND** `valuePosition: ROW` is set
- **THEN** the output functions correctly
- **AND** values are aggregated into a single row
- **AND** structure shows all column variations in columns

#### Scenario: Column-only pivot with valuePosition COLUMN
- **WHEN** a pivot contains only column fields and value fields (no row fields)
- **AND** `valuePosition: COLUMN` is set
- **THEN** the output has value fields as column headers
- **AND** each value field creates additional columns
- **AND** structure reflects both column fields and value fields in columns

#### Scenario: Empty pivot with valuePosition changes
- **WHEN** a pivot table with no source data exists
- **AND** `valuePosition` is changed
- **THEN** the empty result is returned consistently
- **AND** no errors occur during position change
- **AND** `isEmpty` flag is true in both positions

#### Scenario: Dimension tracking with position changes
- **WHEN** a pivot's `valuePosition` is changed
- **THEN** `dimensions.totalRows` and `dimensions.totalColumns` are correctly updated
- **AND** `dimensions.dataRowCount` and `dimensions.dataColumnCount` reflect new structure
- **AND** `dimensions.valueFieldCount` remains unchanged
- **AND** dimensions accurately describe the output structure

#### Scenario: Subtotals with different value positions
- **WHEN** a pivot with `showSubTotals: true` has its `valuePosition` changed
- **THEN** subtotal rows/columns are positioned correctly for the new layout
- **AND** subtotal labels appear in the appropriate dimension
- **AND** subtotal values are correctly calculated and placed

#### Scenario: Filtering with value position changes
- **WHEN** a pivot with filter fields changes its `valuePosition`
- **THEN** filters continue to apply correctly
- **AND** filtered data is still aggregated properly
- **AND** output reflects both filtering and position changes
