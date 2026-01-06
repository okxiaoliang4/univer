## Why

Currently, the sheets thread comment UI plugin (`@univerjs/sheets-thread-comment-ui`) uses canvas popups to display thread comment interfaces when users hover over or click on cells with comments. On mobile devices, popups are not ideal for user interaction due to limited screen space and touch interaction patterns. Mobile users would benefit from a sidebar-based interface that provides better accessibility and follows mobile UI conventions.

## What Changes

- **ADDED**: Mobile plugin (`mobile-plugin.ts`) for `@univerjs/sheets-thread-comment-ui` that registers mobile-specific controllers
- **ADDED**: Mobile popup controller that uses `ISidebarService` instead of canvas popup service for displaying thread comment component
- **ADDED**: Mobile-specific controller that intercepts popup service calls on mobile and redirects them to sidebar service
- **MODIFIED**: Thread comment cell component (`SheetsThreadCommentCell`) integration to work within sidebar context on mobile
- The desktop implementation remains unchanged and continues using canvas popups

## Impact

- **Affected specs**: New capability `sheets-thread-comment-mobile-ui`
- **Affected code**: 
  - `packages/sheets-thread-comment-ui/src/mobile-plugin.ts` (new)
  - `packages/sheets-thread-comment-ui/src/controllers/mobile/` (new directory)
  - `packages/sheets-thread-comment-ui/src/controllers/mobile/mobile-popup.controller.ts` (new)
  - `packages/sheets-thread-comment-ui/src/views/sheets-thread-comment-cell/index.tsx` (may need mobile-specific styling)
- **Breaking changes**: None - this is an additive change that only affects mobile platforms
- **Dependencies**: Requires `@univerjs/ui` with `ISidebarService` available (already available in mobile UI plugin)
