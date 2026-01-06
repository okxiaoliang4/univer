## Why

Currently, on mobile devices, users must click a small dropdown icon to open data validation dropdown menus. This is difficult on touch screens due to the small target size. Mobile users would benefit from being able to tap anywhere on a cell with a dropdown to open it, providing a better touch interaction experience.

## What Changes

- **MODIFIED**: Mobile cell click behavior to detect cells with data validation dropdowns
- **ADDED**: Mobile-specific controller that listens to cell click events and automatically opens dropdown if cell has data validation
- **MODIFIED**: Dropdown widget interaction to work with whole-cell clicks on mobile
- The desktop implementation remains unchanged and continues using the dropdown icon

## Impact

- **Affected specs**: Modified capability `sheets-data-validation-mobile-ui`
- **Affected code**: 
  - `packages/sheets-data-validation-ui/src/controllers/mobile/` (new directory)
  - `packages/sheets-data-validation-ui/src/controllers/mobile/mobile-dropdown-trigger.controller.ts` (new)
  - `packages/sheets-data-validation-ui/src/mobile-plugin.ts` (register new controller)
  - `packages/sheets-data-validation-ui/src/views/widgets/dropdown-widget.ts` (may need mobile detection)
- **Breaking changes**: None - this is a mobile-only enhancement
- **Dependencies**: Requires `HoverManagerService.currentClickedCell$` or similar cell click detection mechanism
