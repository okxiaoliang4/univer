## ADDED Requirements
### Requirement: Sheets Transform Coverage
The OT WASM layer SHALL provide transforms for all Sheets mutation identifiers listed below.

#### Scenario: Sheets transform coverage list
- **WHEN** defining Sheets transform coverage
- **THEN** the following identifiers are included:
  - sheet.mutation.add-range-protection
  - sheet.mutation.add-range-theme
  - sheet.mutation.add-worksheet-merge
  - sheet.mutation.add-worksheet-protection
  - sheet.mutation.copy-worksheet-end
  - sheet.mutation.delete-range-protection
  - sheet.mutation.delete-worksheet-protection
  - sheet.mutation.empty
  - sheet.mutation.insert-col
  - sheet.mutation.insert-row
  - sheet.mutation.insert-sheet
  - sheet.mutation.move-columns
  - sheet.mutation.move-range
  - sheet.mutation.move-rows
  - sheet.mutation.register-worksheet-range-theme-style
  - sheet.mutation.remove-col
  - sheet.mutation.remove-range-theme
  - sheet.mutation.remove-rows
  - sheet.mutation.remove-sheet
  - sheet.mutation.remove-worksheet-merge
  - sheet.mutation.remove-worksheet-range-theme-style
  - sheet.mutation.reorder-range
  - sheet.mutation.set-col-data
  - sheet.mutation.set-col-hidden
  - sheet.mutation.set-col-visible
  - sheet.mutation.set-frozen
  - sheet.mutation.set-gridlines-color
  - sheet.mutation.set-range-protection
  - sheet.mutation.set-range-theme
  - sheet.mutation.set-range-values
  - sheet.mutation.set-row-data
  - sheet.mutation.set-row-hidden
  - sheet.mutation.set-row-visible
  - sheet.mutation.set-tab-color
  - sheet.mutation.set-workbook-name
  - sheet.mutation.set-worksheet-col-width
  - sheet.mutation.set-worksheet-column-count
  - sheet.mutation.set-worksheet-default-style
  - sheet.mutation.set-worksheet-hidden
  - sheet.mutation.set-worksheet-name
  - sheet.mutation.set-worksheet-order
  - sheet.mutation.set-worksheet-permission-points
  - sheet.mutation.set-worksheet-protection
  - sheet.mutation.set-worksheet-range-theme-style
  - sheet.mutation.set-worksheet-right-to-left
  - sheet.mutation.set-worksheet-row-auto-height
  - sheet.mutation.set-worksheet-row-count
  - sheet.mutation.set-worksheet-row-height
  - sheet.mutation.set-worksheet-row-is-auto-height
  - sheet.mutation.toggle-gridlines
  - sheet.mutation.unregister-worksheet-range-theme-style
  - sheet.mutation.set.numfmt
  - sheet.mutation.remove.numfmt

### Requirement: Sheets Transform Coverage Check
The OT WASM change set SHALL flag Sheets mutation identifiers that are already covered by existing transform implementations.

#### Scenario: Detect Sheets transform coverage
- **WHEN** the Sheets mutation inventory is compared with existing OT WASM transforms
- **THEN** the overlap set includes:
  - sheet.mutation.insert-col
  - sheet.mutation.insert-row
  - sheet.mutation.remove-col
  - sheet.mutation.remove-rows
  - sheet.mutation.set-range-values

### Requirement: Sheets Transform Conflict Handling
The OT WASM transform plan SHALL define how conflicts are merged when new Sheets transforms overlap with existing implementations.

#### Scenario: Merge new transform with existing implementation
- **WHEN** a new Sheets transform is introduced for an identifier already implemented
- **THEN** the change uses a single shared implementation and removes duplicate dispatch registration

#### Scenario: No conflict between set-frozen and set-range-values
- **WHEN** sheet.mutation.set-frozen is evaluated against sheet.mutation.set-range-values
- **THEN** both mutations are preserved without additional transform logic

### Requirement: Sheets Mutation Conflict Matrix
The OT WASM change set SHALL document conflicts between Sheets mutations and the existing OT WASM transforms.

#### Scenario: Sheets mutation conflicts are explicit
- **WHEN** reviewing Sheets mutation coverage
- **THEN** the conflicts are recorded as follows:
  - sheet.mutation.add-range-protection: conflicts with insert-row, insert-col, remove-rows, remove-col
  - sheet.mutation.add-range-theme: conflicts with insert-row, insert-col, remove-rows, remove-col
  - sheet.mutation.add-worksheet-merge: conflicts with insert-row, insert-col, remove-rows, remove-col
  - sheet.mutation.add-worksheet-protection: no conflict with existing transforms
  - sheet.mutation.copy-worksheet-end: no conflict with existing transforms
  - sheet.mutation.delete-range-protection: conflicts with insert-row, insert-col, remove-rows, remove-col
  - sheet.mutation.delete-worksheet-protection: no conflict with existing transforms
  - sheet.mutation.empty: conflicts with set-range-values
  - sheet.mutation.insert-col: existing transform; conflicts with set-range-values
  - sheet.mutation.insert-row: existing transform; conflicts with set-range-values
  - sheet.mutation.insert-sheet: no conflict with existing transforms
  - sheet.mutation.move-columns: conflicts with insert-col, remove-col, set-range-values
  - sheet.mutation.move-range: conflicts with insert-row, insert-col, remove-rows, remove-col, set-range-values
  - sheet.mutation.move-rows: conflicts with insert-row, remove-rows, set-range-values
  - sheet.mutation.register-worksheet-range-theme-style: conflicts with insert-row, insert-col, remove-rows, remove-col
  - sheet.mutation.remove-col: existing transform; conflicts with set-range-values
  - sheet.mutation.remove-range-theme: conflicts with insert-row, insert-col, remove-rows, remove-col
  - sheet.mutation.remove-rows: existing transform; conflicts with set-range-values
  - sheet.mutation.remove-sheet: no conflict with existing transforms
  - sheet.mutation.remove-worksheet-merge: conflicts with insert-row, insert-col, remove-rows, remove-col
  - sheet.mutation.remove-worksheet-range-theme-style: conflicts with insert-row, insert-col, remove-rows, remove-col
  - sheet.mutation.reorder-range: conflicts with insert-row, insert-col, remove-rows, remove-col, set-range-values
  - sheet.mutation.set-col-data: conflicts with insert-col, remove-col
  - sheet.mutation.set-col-hidden: conflicts with insert-col, remove-col
  - sheet.mutation.set-col-visible: conflicts with insert-col, remove-col
  - sheet.mutation.set-frozen: conflicts with insert-row, insert-col, remove-rows, remove-col
  - sheet.mutation.set-gridlines-color: no conflict with existing transforms
  - sheet.mutation.set-range-protection: conflicts with insert-row, insert-col, remove-rows, remove-col
  - sheet.mutation.set-range-theme: conflicts with insert-row, insert-col, remove-rows, remove-col
  - sheet.mutation.set-range-values: existing transform; conflicts with insert-row, insert-col, remove-rows, remove-col
  - sheet.mutation.set-row-data: conflicts with insert-row, remove-rows
  - sheet.mutation.set-row-hidden: conflicts with insert-row, remove-rows
  - sheet.mutation.set-row-visible: conflicts with insert-row, remove-rows
  - sheet.mutation.set-tab-color: no conflict with existing transforms
  - sheet.mutation.set-workbook-name: no conflict with existing transforms
  - sheet.mutation.set-worksheet-col-width: conflicts with insert-col, remove-col
  - sheet.mutation.set-worksheet-column-count: conflicts with insert-col, remove-col
  - sheet.mutation.set-worksheet-default-style: no conflict with existing transforms
  - sheet.mutation.set-worksheet-hidden: no conflict with existing transforms
  - sheet.mutation.set-worksheet-name: no conflict with existing transforms
  - sheet.mutation.set-worksheet-order: no conflict with existing transforms
  - sheet.mutation.set-worksheet-permission-points: conflicts with insert-row, insert-col, remove-rows, remove-col
  - sheet.mutation.set-worksheet-protection: no conflict with existing transforms
  - sheet.mutation.set-worksheet-range-theme-style: conflicts with insert-row, insert-col, remove-rows, remove-col
  - sheet.mutation.set-worksheet-right-to-left: no conflict with existing transforms
  - sheet.mutation.set-worksheet-row-auto-height: conflicts with insert-row, remove-rows
  - sheet.mutation.set-worksheet-row-count: conflicts with insert-row, remove-rows
  - sheet.mutation.set-worksheet-row-height: conflicts with insert-row, remove-rows
  - sheet.mutation.set-worksheet-row-is-auto-height: conflicts with insert-row, remove-rows
  - sheet.mutation.toggle-gridlines: no conflict with existing transforms
  - sheet.mutation.unregister-worksheet-range-theme-style: conflicts with insert-row, insert-col, remove-rows, remove-col
  - sheet.mutation.set.numfmt: conflicts with insert-row, insert-col, remove-rows, remove-col, set-range-values
  - sheet.mutation.remove.numfmt: conflicts with insert-row, insert-col, remove-rows, remove-col, set-range-values
