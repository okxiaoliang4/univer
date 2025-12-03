## Why

The pivot table output protection was initially implemented in a single controller in the core package (`@sheets-pivot-table`), mixing UI-related command interception with business logic. This violates separation of concerns and makes the codebase harder to maintain.

## What Changes

- **Refactor**: Split `PivotTablePermissionController` into two controllers:
  - `PivotTablePermissionController` (core package): Handles business logic (cell content interceptor, core edit commands)
  - `PivotTablePermissionUIController` (UI package): Handles UI interactions (edit mode, formula bar input)
- **Architecture**: Implement layered architecture with clear separation between UI and business logic layers
- **Dependencies**: UI package depends on core package, core package has no UI dependencies

## Impact

- Affected specs: `sheets-pivot-table`
- Affected code:
  - `packages/sheets-pivot-table/src/controllers/pivot-table-permission.controller.ts` - Refactored to remove UI commands
  - `packages/sheets-pivot-table-ui/src/controllers/pivot-table-permission-ui.controller.ts` - New UI controller
  - `packages/sheets-pivot-table-ui/src/plugin.ts` - Register new UI controller

