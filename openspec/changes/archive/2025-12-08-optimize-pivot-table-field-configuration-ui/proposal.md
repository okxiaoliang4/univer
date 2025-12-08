## Why

The current pivot table field configuration UI requires users to manually drag fields from source to target areas (rows, columns, values, filters), which is cumbersome and time-consuming. Users need a more efficient way to add fields with intelligent placement based on data types and user preferences.

## What Changes

- **ADDED**: Checkbox controls on source fields for quick field addition
- **ADDED**: Intelligent field placement logic based on data type detection (numeric → valueFields, text → rowFields)
- **ADDED**: Fuzzy matching configuration system for field name-based placement rules
- **ADDED**: Browser storage persistence for user field placement preferences
- **ADDED**: "Add" button with dropdown menu in field configuration areas (similar to Google Sheets)
- **ADDED**: Filtered field lists in dropdown menus based on area type (rowFields/columnFields exclude already-added fields, valueFields show all, filterFields exclude already-added)

## Impact

- Affected specs: `sheets-pivot-table-ui` (new capability)
- Affected code:
  - `packages/sheets-pivot-table-ui/src/views/components/PivotTableEditor.tsx`
  - `packages/sheets-pivot-table-ui/src/views/components/PivotTablePanel.tsx`
  - `packages/sheets-pivot-table-ui/src/views/components/FieldRender.tsx`
  - `packages/sheets-pivot-table-ui/src/views/components/FieldItemsContainer.tsx`
  - New service for field type detection and storage management
