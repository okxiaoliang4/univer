# Pivot Table MVP Implementation

## Why

Currently, the `sheets-pivot-table` and `sheets-pivot-table-ui` packages contain only skeleton implementations with basic data models but no functional capabilities. Users cannot create, configure, or interact with pivot tables in Univer spreadsheets. This MVP implementation will provide essential pivot table functionality following the established plugin architecture pattern (similar to sheets-filter/sheets-filter-ui), enabling users to:
- Create pivot tables from worksheet data ranges
- Configure basic field arrangements (row fields, column fields, value fields)
- Apply fundamental aggregation functions (SUM, COUNT, AVERAGE, MIN, MAX)
- View calculated pivot table results in the worksheet

## What Changes

### sheets-pivot-table (Core Plugin)
- Implement core services for pivot table lifecycle management
- Create controllers for coordinating pivot table operations
- Implement commands/mutations for pivot table CRUD operations
- Add snapshot support for serialization/deserialization
- Implement calculation engine integration with existing aggregation models
- Add basic configuration schema

### sheets-pivot-table-ui (UI Plugin)
- Implement desktop UI controllers for pivot table interactions
- Create configuration panel for field management (drag-drop fields to areas)
- Add UI components for:
  - Pivot table creation wizard/dialog
  - Field configuration panel (row fields, column fields, value fields, filter fields)
  - Aggregation function selector
  - Basic settings (name, data source range, target location)
- Implement menu items, toolbar buttons, and context menu integration
- Add localization support (en-US, zh-CN as baseline)
- Implement render modules for pivot table visualization in sheets
- Add shortcut support for common operations

## Impact

- **Affected specs**: `sheets-pivot-table`, `sheets-pivot-table-ui` (new capabilities being added)
- **Affected code**:
  - `packages/sheets-pivot-table/src/`: Add services, controllers, commands, models
  - `packages/sheets-pivot-table-ui/src/`: Add controllers, views, components, localization
  - Both packages need proper plugin lifecycle implementation
- **Dependencies**: Builds on existing models in `packages/sheets-pivot-table/src/model/`
- **Pattern reference**: Following `sheets-filter` / `sheets-filter-ui` architecture
- **Breaking changes**: None (additive changes only)

