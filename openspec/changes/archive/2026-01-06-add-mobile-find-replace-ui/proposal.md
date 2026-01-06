## Why

Currently, the find-replace feature (`@univerjs/find-replace`) uses a Dialog component for desktop users. On mobile devices, dialogs are not ideal for user interaction due to limited screen space and touch interaction patterns. Mobile users would benefit from a sidebar-based interface that provides better accessibility and follows mobile UI conventions. Additionally, the current implementation mixes business logic with UI components, making it difficult to reuse logic between desktop and mobile implementations.

## What Changes

- **ADDED**: Mobile plugin (`mobile-plugin.ts`) for `@univerjs/find-replace` that registers mobile-specific controllers
- **ADDED**: Mobile controller (`mobile-find-replace.controller.ts`) that uses `ISidebarService` instead of `IDialogService` for displaying find-replace component on mobile, registers commands (but not shortcuts), and follows the same structure as desktop controller
- **ADDED**: Mobile find-replace component (`MobileFindReplace.tsx`) that renders the find-replace UI within a sidebar context
- **ADDED**: Shared hooks (`use-find-replace-logic.ts`, `use-find-replace-options.ts`) to extract business logic from UI components for reuse between desktop and mobile
- **MODIFIED**: `FindReplaceDialog.tsx` to use shared hooks, reducing code duplication
- **MODIFIED**: `FindReplaceController.ts` to conditionally register mobile controller when mobile UI plugin is available
- The desktop implementation remains unchanged in behavior and continues using Dialog

## Impact

- **Affected specs**: New capability `find-replace-mobile-ui`
- **Affected code**: 
  - `packages/find-replace/src/mobile-plugin.ts` (new)
  - `packages/find-replace/src/controllers/mobile/mobile-find-replace.controller.ts` (new)
  - `packages/find-replace/src/views/mobile/MobileFindReplace.tsx` (new)
  - `packages/find-replace/src/views/dialog/hooks/use-find-replace-logic.ts` (new)
  - `packages/find-replace/src/views/dialog/hooks/use-find-replace-options.ts` (new)
  - `packages/find-replace/src/views/dialog/FindReplaceDialog.tsx` (refactor to use hooks)
  - `packages/find-replace/src/controllers/find-replace.controller.ts` (may need mobile detection)
  - `packages/sheets-find-replace/src/mobile-plugin.ts` (new)
  - `packages/sheets-find-replace/src/controllers/mobile/sheet-find-replace.controller.ts` (new)
  - `packages/sheets-find-replace/src/controllers/sheet-find-replace.controller.ts` (export SheetsFindReplaceProvider)
  - `packages/sheets-find-replace/src/index.ts` (export mobile plugin)
- **Breaking changes**: None - this is an additive change that only affects mobile platforms
- **Dependencies**: Requires `@univerjs/ui` with `ISidebarService` available (already available in mobile UI plugin)
