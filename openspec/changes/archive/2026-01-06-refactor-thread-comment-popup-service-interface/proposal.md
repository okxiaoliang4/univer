## Why

Currently, the `SheetsThreadCommentPopupService` is a concrete class that implements desktop-specific canvas popup behavior. The mobile plugin attempts to work around this by creating separate mobile controllers, but this creates architectural issues:

1. Mobile and desktop controllers duplicate logic and create maintenance burden
2. The service implementation is tightly coupled to desktop canvas popup mechanism
3. Mobile plugin cannot properly override the popup behavior without creating separate controllers

By extracting a service interface and creating platform-specific implementations, we can:
- Share the same controllers between mobile and desktop (they inject the interface)
- Cleanly separate desktop (canvas popup) and mobile (sidebar) implementations
- Follow dependency injection best practices with interface-based design
- Eliminate the need for separate mobile controllers

## What Changes

- **ADDED**: `ISheetsThreadCommentPopupService` interface defining the public API for popup/sidebar management
- **ADDED**: Redi dependency injection token `ISheetsThreadCommentPopupService` using `createIdentifier`
- **MODIFIED**: Current `SheetsThreadCommentPopupService` renamed to `SheetsThreadCommentDesktopPopupService` implementing the interface
- **ADDED**: `SheetsThreadCommentMobilePopupService` implementing the interface using `ISidebarService` instead of canvas popup
- **MODIFIED**: `plugin.ts` registers `ISheetsThreadCommentPopupService` token with desktop implementation
- **MODIFIED**: `mobile-plugin.ts` registers `ISheetsThreadCommentPopupService` token with mobile implementation
- **ADDED**: `SheetsThreadCommentMobileHoverController` for mobile-specific hover behavior (uses `currentClickedCell$` instead of `currentCell$` for touch interactions)
- **MODIFIED**: `mobile-plugin.ts` registers mobile hover controller instead of desktop hover controller
- **REMOVED**: Mobile popup controller (`mobile-popup.controller.ts`) - popup controller now works with both platforms via interface injection
- **MODIFIED**: All controllers and components inject `ISheetsThreadCommentPopupService` instead of concrete class

## Impact

- **Affected specs**: Modified capability `sheets-thread-comment-mobile-ui`
- **Affected code**:
  - `packages/sheets-thread-comment-ui/src/services/sheets-thread-comment-popup.service.ts` - Extract interface, rename to desktop implementation
  - `packages/sheets-thread-comment-ui/src/services/sheets-thread-comment-mobile-popup.service.ts` (new) - Mobile implementation using sidebar
  - `packages/sheets-thread-comment-ui/src/plugin.ts` - Register desktop service implementation
  - `packages/sheets-thread-comment-ui/src/mobile-plugin.ts` - Register mobile service implementation
  - `packages/sheets-thread-comment-ui/src/controllers/` - Update all controllers to inject interface
  - `packages/sheets-thread-comment-ui/src/controllers/mobile/sheets-thread-comment-hover.controller.ts` (new) - Mobile hover controller using click events
  - `packages/sheets-thread-comment-ui/src/views/` - Update components to inject interface
  - `packages/sheets-thread-comment-ui/src/commands/` - Update operations to inject interface
- **Breaking changes**: None - interface maintains same public API, only internal implementation changes
- **Dependencies**: Requires `@univerjs/ui` with `ISidebarService` for mobile implementation
