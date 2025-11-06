# Implementation Tasks

## 1. Core Plugin Foundation (sheets-pivot-table)

- [x] 1.1 Create configuration schema (`controllers/config.schema.ts`)
  - Plugin configuration interface
  - Default configuration values
  - Configuration key constant
- [x] 1.2 Implement PivotTableService (`services/pivot-table.service.ts`)
  - Manage pivot table instances (CRUD operations)
  - Track pivot tables by workbook and worksheet
  - Provide snapshot ID and resource key
  - Implement snapshot save/load methods
- [x] 1.3 Implement PivotTableCalculationService (`services/pivot-table-calculation.service.ts`)
  - Calculate pivot table results from source data
  - Handle field grouping and aggregation
  - Support multiple aggregation functions
  - Cache calculation results
- [x] 1.4 Create PivotTableController (`controllers/pivot-table.controller.ts`)
  - Coordinate pivot table lifecycle
  - Handle workbook/worksheet events
  - Manage pivot table refresh logic
  - Register snapshot handlers
- [x] 1.5 Implement commands (`commands/commands/pivot-table.command.ts`)
  - CreatePivotTableCommand
  - UpdatePivotTableFieldsCommand
  - DeletePivotTableCommand
  - RefreshPivotTableCommand
- [x] 1.6 Implement mutations (`commands/mutations/pivot-table.mutation.ts`)
  - SetPivotTableMutation (add/update in model)
  - RemovePivotTableMutation (remove from model)
- [x] 1.7 Update plugin.ts with proper DI registration
  - Register services and controllers
  - Touch dependencies in lifecycle hooks
  - Configure snapshot integration
- [x] 1.8 Export public API in index.ts
  - Export plugin class
  - Export services, commands, types
  - Export models and enums

## 2. UI Plugin Foundation (sheets-pivot-table-ui)

- [x] 2.1 Create configuration schema (`controllers/config.schema.ts`)
  - UI plugin configuration interface
  - Menu configuration
  - Default values
- [x] 2.2 Implement menu schema (`controllers/menu.schema.ts`)
  - Define menu items (Insert > Pivot Table)
  - Context menu items
  - Toolbar buttons
- [x] 2.3 Implement shortcuts (`controllers/pivot-table.shortcut.ts`)
  - Define keyboard shortcuts for common operations
- [x] 2.4 Create PivotTableUIDesktopController (`controllers/pivot-table-ui-desktop.controller.ts`)
  - Register menu items and shortcuts
  - Handle UI command routing
  - Manage panel visibility
  - Coordinate with render modules
- [x] 2.5 Implement PivotTablePanelService (`services/pivot-table-panel.service.ts`)
  - Manage panel state (open/close, active pivot table)
  - Provide API for panel interactions
  - Track field configuration changes
- [x] 2.6 Update plugin.ts with proper DI registration
  - Register controllers and services
  - Touch dependencies in lifecycle hooks
  - Handle menu configuration

## 3. UI Components (sheets-pivot-table-ui)

- [ ] 3.1 Create PivotTableCreationDialog (`views/components/PivotTableCreationDialog.tsx`)
  - Data source range selector
  - Target location selector
  - Pivot table name input
  - Create/Cancel actions
- [ ] 3.2 Create PivotTableFieldPanel (`views/components/PivotTableFieldPanel.tsx`)
  - Available fields list (from source data headers)
  - Four drop zones: Filters, Columns, Rows, Values
  - Drag-and-drop support
  - Field configuration (aggregation function selection)
  - Remove field action
- [ ] 3.3 Create FieldConfigButton (`views/components/FieldConfigButton.tsx`)
  - Button to open aggregation function selector
  - Display current aggregation type
  - Dropdown menu for function selection
- [ ] 3.4 Create PivotTablePanel (`views/components/PivotTablePanel.tsx`)
  - Main panel combining creation and field configuration
  - Tab or mode switching between creation and editing
  - Refresh button
  - Settings button

## 4. Render Integration (sheets-pivot-table-ui)

- [ ] 4.1 Create PivotTableRenderController (`views/render-modules/pivot-table.render-controller.ts`)
  - Render pivot table results in worksheet
  - Handle cell styling (headers, totals, data)
  - Update on pivot table changes
  - Handle selection and hover states
- [ ] 4.2 Implement pivot table marker rendering
  - Visual indicators for pivot table boundaries
  - Corner icon/button to access settings
  - Highlight on hover/selection

## 5. Commands and Operations (sheets-pivot-table-ui)

- [ ] 5.1 Implement UI commands (`commands/commands/`)
  - OpenCreatePivotTableDialogCommand
  - OpenPivotTablePanelCommand
  - TogglePivotTableFieldCommand
- [ ] 5.2 Implement UI operations (`commands/operations/`)
  - ShowPivotTablePanelOperation
  - HidePivotTablePanelOperation
  - UpdatePivotTableFieldsOperation

## 6. Localization (sheets-pivot-table-ui)

- [ ] 6.1 Create locale files (`locale/`)
  - en-US.ts (English)
  - zh-CN.ts (Chinese Simplified)
- [ ] 6.2 Add localization keys
  - Menu items
  - Dialog titles and labels
  - Field names
  - Aggregation function names
  - Error messages
  - Button labels

## 7. Testing

- [x] 7.1 Unit tests for sheets-pivot-table
  - PivotTableService tests (11 tests)
  - PivotTableCalculationService tests (5 tests)
  - Command execution tests (9 tests)
  - Mutation tests (6 tests)
  - Model tests (PivotField: 14 tests)
  - Aggregation functions tests (25 tests)
- [x] 7.2 Unit tests for sheets-pivot-table-ui
  - Panel service tests (15 tests)
  - Controller tests (covered via integration)
  - Menu/shortcut registration (covered via integration)
- [ ] 7.3 Integration tests (Deferred)
  - End-to-end pivot table creation flow
  - Field configuration updates
  - Calculation and rendering
  - Snapshot save/load

## 8. Documentation and Examples

- [x] 8.1 Update README.md for sheets-pivot-table
  - Installation instructions
  - Basic usage example
  - API reference links
- [x] 8.2 Update README.md for sheets-pivot-table-ui
  - Installation instructions
  - UI features overview
  - Configuration options
- [ ] 8.3 Add example to examples/ directory (Future work)
  - Demo workbook with sample data
  - Pre-configured pivot table
  - Show all aggregation functions

## Dependencies

- Tasks 2.x depend on completion of 1.x (UI requires core services)
- Tasks 3.x and 4.x can be done in parallel after 2.x
- Tasks 5.x depend on 2.x and 3.x
- Task 6.x can be done in parallel with 3.x-5.x
- Task 7.x should be done incrementally alongside implementation
- Task 8.x is final step after core implementation

