## Why

Currently, the sheets hyperlink UI plugin (`@univerjs/sheets-hyper-link-ui`) uses canvas popups to display hyperlink editing interfaces. On mobile devices, popups are not ideal for user interaction due to limited screen space and touch interaction patterns. Mobile users would benefit from a sidebar-based interface that provides better accessibility and follows mobile UI conventions.

## What Changes

- **ADDED**: Mobile plugin (`mobile-plugin.ts`) for `@univerjs/sheets-hyper-link-ui` that registers mobile-specific controllers
- **ADDED**: Mobile popup controller that uses `ISidebarService` instead of canvas popup service for displaying hyperlink edit component
- **ADDED**: Mobile-specific service or controller that intercepts popup service calls on mobile and redirects them to sidebar service
- **MODIFIED**: Hyperlink edit component (`CellLinkEdit`) integration to work within sidebar context on mobile
- The desktop implementation remains unchanged and continues using canvas popups

## Impact

- **Affected specs**: New capability `sheets-hyper-link-mobile-ui`
- **Affected code**: 
  - `packages/sheets-hyper-link-ui/src/mobile-plugin.ts` (new)
  - `packages/sheets-hyper-link-ui/src/controllers/mobile/` (new directory)
  - `packages/sheets-hyper-link-ui/src/services/popup.service.ts` (may need mobile detection)
  - `packages/sheets-hyper-link-ui/src/views/CellLinkEdit/index.tsx` (may need mobile-specific styling)
- **Breaking changes**: None - this is an additive change that only affects mobile platforms
- **Dependencies**: Requires `@univerjs/ui` with `ISidebarService` available (already available in mobile UI plugin)
