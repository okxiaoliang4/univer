## ADDED Requirements

### Requirement: Apply sheet mutations to workbook snapshots
The system SHALL apply supported sheet mutations to `IWorkbookData` snapshot content when computing snapshots.

#### Scenario: Apply set-range-values to cell data
- **WHEN** a `sheet.mutation.set-range-values` mutation is applied for a worksheet
- **THEN** the worksheet `cellData` matrix is updated per cell in `cellValue`
- **AND** null cell values remove the cell entry
- **AND** defined cell values merge into existing cell data using the same merge semantics as the TypeScript mutation handler (value/type normalization and style updates)
- **AND** the workbook `styles` map is updated when style merges require new or reused style IDs

#### Scenario: Insert rows into worksheet
- **WHEN** a `sheet.mutation.insert-row` mutation is applied
- **THEN** the worksheet `rowData`, `cellData`, and `rowCount` reflect inserted rows at `range.startRow`
- **AND** provided `rowInfo` is inserted by relative index
- **AND** missing row info uses the worksheet `defaultRowHeight` and `hd: 0`

#### Scenario: Insert columns into worksheet
- **WHEN** a `sheet.mutation.insert-col` mutation is applied
- **THEN** the worksheet `columnData`, `cellData`, and `columnCount` reflect inserted columns at `range.startColumn`
- **AND** provided `colInfo` is inserted by relative index
- **AND** missing column info uses the worksheet `defaultColumnWidth` and `hd: 0`

#### Scenario: Remove rows from worksheet
- **WHEN** a `sheet.mutation.remove-rows` mutation is applied
- **THEN** the worksheet `rowData` and `cellData` remove the row range
- **AND** `rowCount` is decremented by the removed count

#### Scenario: Remove columns from worksheet
- **WHEN** a `sheet.mutation.remove-col` mutation is applied
- **THEN** the worksheet `columnData` and `cellData` remove the column range
- **AND** `columnCount` is decremented by the removed count

### Requirement: Reject invalid mutation application
The system SHALL fail snapshot application when a mutation targets a missing workbook/worksheet or uses an invalid range.

#### Scenario: Missing worksheet during snapshot apply
- **WHEN** a mutation references a `subUnitId` that does not exist in the workbook snapshot
- **THEN** the snapshot apply returns a structured error
- **AND** the snapshot content is not updated for that mutation

#### Scenario: Invalid range during snapshot apply
- **WHEN** a mutation range is invalid (start greater than end)
- **THEN** the snapshot apply returns a structured error
- **AND** the snapshot content is not updated for that mutation
