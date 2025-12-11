## 1. Type Definitions
- [x] 1.1 Update `IPivotField` interface to add `showSubTotals?: boolean` field
- [x] 1.2 Create `IPivotTableCrossTabConfig` interface (input configuration)
- [x] 1.3 Create `IPivotTableCrossTabData` interface (output result with isEmpty, dimensions, structure)
- [x] 1.4 Define all nested interfaces (subtotalRows, subtotalColumns, rowGroups, columnGroups, etc.)

## 2. PivotEngineV2 Core Implementation
- [x] 2.1 Create `PivotEngineV2` class structure with constructor accepting `IPivotTableCrossTabConfig`
- [x] 2.2 Implement individual getter methods (`getRowFields`, `getColumnFields`, etc.)
- [x] 2.3 Implement individual setter methods (`setRowFields`, `setColumnFields`, etc.) with `_markDirty()`
- [x] 2.4 Implement caching mechanism (`_calculatedData`, `_isDirty`, `isDirty()`)
- [x] 2.5 Implement `getCalculatedData()` method with lazy calculation

## 3. Data Extraction and Filtering
- [x] 3.1 Implement `_extractData()` method to extract header and data rows from source data
- [x] 3.2 Implement `_applyFilters()` method to apply filterFields before aggregation
- [x] 3.3 Handle empty source data and filtered-out data cases

## 4. Grouping Logic
- [x] 4.1 Implement `_groupByFields()` method for multi-level row field grouping
- [x] 4.2 Implement `_groupByFields()` method for multi-level column field grouping
- [x] 4.3 Generate unique group keys with proper separator handling
- [x] 4.4 Handle blank values in grouping (use placeholder)

## 5. Subtotal Calculation
- [x] 5.1 Implement subtotal row calculation based on `showSubTotals` configuration
- [x] 5.2 Implement subtotal column calculation based on `showSubTotals` configuration
- [x] 5.3 Generate subtotal labels (e.g., "华北 总计")
- [x] 5.4 Ensure first field's subtotal = grand total row/column
- [x] 5.5 Populate `subtotalRows` and `subtotalColumns` arrays with metadata

## 6. Value Field Aggregation
- [x] 6.1 Implement multi-value field aggregation (3D array: [row][column][valueField])
- [x] 6.2 Support all aggregation types (SUM, COUNT, AVERAGE, MIN, MAX)
- [x] 6.3 Generate `valueFieldHeaders` array when multiple value fields exist
- [x] 6.4 Handle null/empty values in aggregation

## 7. Grouping Information Generation
- [x] 7.1 Generate `rowGroups` array with groupId, firstRowIndex, lastRowIndex, level, etc.
- [x] 7.2 Generate `columnGroups` array with groupId, firstColumnIndex, lastColumnIndex, level, etc.
- [x] 7.3 Build parent-child relationships (`parentGroupId`, `childGroupIds`)
- [x] 7.4 Generate `rowLevelMap` as `Record<number, string[]>` (not Map for serialization)
- [x] 7.5 Generate `columnLevelMap` as `Record<number, string[]>` (not Map for serialization)

## 8. isEmpty and dimensions Calculation
- [x] 8.1 Implement `isEmpty` detection logic (no value fields, no data rows, all null values)
- [x] 8.2 Calculate `dimensions.totalRows` (total row count including subtotals)
- [x] 8.3 Calculate `dimensions.totalColumns` (total column count including subtotals)
- [x] 8.4 Calculate `dimensions.dataRowCount` (data rows only, excluding subtotals)
- [x] 8.5 Calculate `dimensions.dataColumnCount` (data columns only, excluding subtotals)
- [x] 8.6 Calculate `dimensions.valueFieldCount` (number of value fields)

## 9. Row/Column Headers Generation
- [x] 9.1 Generate `rowHeaders` array with proper multi-level values
- [x] 9.2 Generate `columnHeaders` array with proper multi-level values
- [x] 9.3 Handle subtotal rows (empty string for corresponding field position)
- [x] 9.4 Handle grand total row (first field = "总计", others empty)
- [x] 9.5 Generate `rowTypes` and `columnTypes` arrays ('data' | 'subtotal')

## 10. PivotTableRenderModel Implementation
- [x] 10.1 Create `PivotTableRenderModel` class accepting `IPivotTableCrossTabData`
- [x] 10.2 Implement `getVisibleRowIndices()` considering collapse state
- [x] 10.3 Implement `getVisibleColumnIndices()` considering collapse state
- [x] 10.4 Implement `isRowVisible()` and `isColumnVisible()` methods
- [x] 10.5 Implement `getCellValue()` and `getCellInfo()` methods
- [x] 10.6 Implement `getRowInfo()` and `getColumnInfo()` methods
- [x] 10.7 Implement `getRowGroupInfo()` and `getColumnGroupInfo()` for collapse buttons
- [x] 10.8 Implement `toggleRowGroup()` and `toggleColumnGroup()` methods
- [x] 10.9 Implement `expandAllRows()`, `collapseAllRows()`, etc.
- [x] 10.10 Implement `getVisibleDimensions()` and index mapping methods

## 11. Testing (TDD)
- [x] 11.1 Write test for basic calculation (single row field + single value field)
- [x] 11.2 Write test for 2D cross-tabulation (row + column + value fields)
- [x] 11.3 Write test for multi-level rows/columns
- [x] 11.4 Write test for subtotal calculation (different levels)
- [x] 11.5 Write test for grand total (first field's subtotal)
- [x] 11.6 Write test for multi-value fields
- [x] 11.7 Write test for filterFields application
- [x] 11.8 Write test for isEmpty detection
- [x] 11.9 Write test for dimensions calculation
- [x] 11.10 Write test for collapse/expand functionality
- [x] 11.11 Write test for empty source data handling
- [x] 11.12 Write test for caching mechanism

## 12. Validation
- [x] 12.1 Run `openspec validate add-pivot-engine-v2 --strict`
- [x] 12.2 Fix any validation issues
- [x] 12.3 Ensure all tests pass
- [x] 12.4 Verify output format matches Cross-Tabulation requirements

