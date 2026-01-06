## 1. Extract Service Interface

- [x] 1.1 Create interface `ISheetsThreadCommentPopupService` in `packages/sheets-thread-comment-ui/src/services/sheets-thread-comment-popup.service.ts`
  - [x] 1.1.1 Extract public API: `activePopup$`, `activePopup` getter, `showPopup()`, `hidePopup()`, `persistPopup()`
  - [x] 1.1.2 Define `IThreadCommentPopup` interface (already exists, keep it)
  - [x] 1.1.3 Export interface from service file

- [x] 1.2 Create Redi token `ISheetsThreadCommentPopupService` using `createIdentifier`
  - [x] 1.2.1 Use token name: `'sheets-thread-comment-ui.sheets-thread-comment-popup.service'`
  - [x] 1.2.2 Export token from service file

## 2. Refactor Desktop Service Implementation

- [x] 2.1 Rename `SheetsThreadCommentPopupService` to `SheetsThreadCommentDesktopPopupService`
  - [x] 2.1.1 Update class name
  - [x] 2.1.2 Implement `ISheetsThreadCommentPopupService` interface
  - [x] 2.1.3 Keep all existing desktop canvas popup logic unchanged
  - [x] 2.1.4 Update error messages to reference desktop service name

- [x] 2.2 Update `plugin.ts` to register desktop service
  - [x] 2.2.1 Change registration from `[SheetsThreadCommentPopupService]` to `[ISheetsThreadCommentPopupService, { useClass: SheetsThreadCommentDesktopPopupService }]`
  - [x] 2.2.2 Import `ISheetsThreadCommentPopupService` token
  - [x] 2.2.3 Import `SheetsThreadCommentDesktopPopupService` class

## 3. Create Mobile Service Implementation

- [x] 3.1 Create `packages/sheets-thread-comment-ui/src/services/sheets-thread-comment-mobile-popup.service.ts`
  - [x] 3.1.1 Implement `ISheetsThreadCommentPopupService` interface
  - [x] 3.1.2 Inject `ISidebarService` instead of canvas popup services
  - [x] 3.1.3 Implement `activePopup$` observable using `BehaviorSubject`
  - [x] 3.1.4 Implement `showPopup()` to open sidebar with `SHEETS_THREAD_COMMENT_MODAL` component
  - [x] 3.1.5 Implement `hidePopup()` to close sidebar
  - [x] 3.1.6 Implement `persistPopup()` to update popup state (no-op for sidebar, but maintain state)
  - [x] 3.1.7 Handle sidebar close callback to sync with popup state
  - [x] 3.1.8 Use sidebar ID: `'sheets-thread-comment-popup'`
  - [x] 3.1.9 Generate sidebar title from cell reference (e.g., "Comment A1")

- [x] 3.2 Update `mobile-plugin.ts` to register mobile service
  - [x] 3.2.1 Change registration from `[SheetsThreadCommentPopupService]` to `[ISheetsThreadCommentPopupService, { useClass: SheetsThreadCommentMobilePopupService }]`
  - [x] 3.2.2 Import `ISheetsThreadCommentPopupService` token
  - [x] 3.2.3 Import `SheetsThreadCommentMobilePopupService` class
  - [x] 3.2.4 Remove mobile controller registrations (no longer needed)

## 4. Update Controllers to Use Interface

- [x] 4.1 Update `sheets-thread-comment-popup.controller.ts`
  - [x] 4.1.1 Change injection from `@Inject(SheetsThreadCommentPopupService)` to `@Inject(ISheetsThreadCommentPopupService)`
  - [x] 4.1.2 Update type annotation to use interface
  - [x] 4.1.3 Verify all method calls match interface API

- [x] 4.2 Update `sheets-thread-comment-hover.controller.ts`
  - [x] 4.2.1 Change injection from `@Inject(SheetsThreadCommentPopupService)` to `@Inject(ISheetsThreadCommentPopupService)`
  - [x] 4.2.2 Update type annotation to use interface
  - [x] 4.2.3 Verify all method calls match interface API

- [x] 4.3 Create mobile hover controller
  - [x] 4.3.1 Create `packages/sheets-thread-comment-ui/src/controllers/mobile/sheets-thread-comment-hover.controller.ts`
  - [x] 4.3.2 Use `currentClickedCell$` instead of `currentCell$` for mobile touch interactions
  - [x] 4.3.3 Inject `ISheetsThreadCommentPopupService` interface
  - [x] 4.3.4 Register in `mobile-plugin.ts` instead of desktop hover controller

## 5. Update Views and Components

- [x] 5.1 Update `sheets-thread-comment-cell/index.tsx`
  - [x] 5.1.1 Change `useDependency(SheetsThreadCommentPopupService)` to `useDependency(ISheetsThreadCommentPopupService)`
  - [x] 5.1.2 Import interface token

- [x] 5.2 Update `sheets-thread-comment-panel/index.tsx`
  - [x] 5.2.1 Change `useDependency(SheetsThreadCommentPopupService)` to `useDependency(ISheetsThreadCommentPopupService)`
  - [x] 5.2.2 Import interface token

## 6. Update Commands and Operations

- [x] 6.1 Update `commands/operations/comment.operation.ts`
  - [x] 6.1.1 Change `accessor.get(SheetsThreadCommentPopupService)` to `accessor.get(ISheetsThreadCommentPopupService)`
  - [x] 6.1.2 Import interface token

## 7. Update Exports

- [x] 7.1 Update `packages/sheets-thread-comment-ui/src/index.ts`
  - [x] 7.1.1 Export `ISheetsThreadCommentPopupService` interface and token
  - [x] 7.1.2 Export `SheetsThreadCommentDesktopPopupService` (for testing/debugging)
  - [x] 7.1.3 Export `SheetsThreadCommentMobilePopupService` (for testing/debugging)
  - [x] 7.1.4 Remove export of old `SheetsThreadCommentPopupService` class name

## 8. Testing

- [ ] 8.1 Unit tests for interface extraction
  - [ ] 8.1.1 Test that interface defines all required methods
  - [ ] 8.1.2 Test that both implementations satisfy interface contract

- [ ] 8.2 Unit tests for desktop service
  - [ ] 8.2.1 Test canvas popup behavior unchanged
  - [ ] 8.2.2 Test service implements interface correctly

- [ ] 8.3 Unit tests for mobile service
  - [ ] 8.3.1 Test sidebar opens when `showPopup()` called
  - [ ] 8.3.2 Test sidebar closes when `hidePopup()` called
  - [ ] 8.3.3 Test `activePopup$` observable emits correct values
  - [ ] 8.3.4 Test sidebar close callback syncs with popup state
  - [ ] 8.3.5 Test `persistPopup()` updates state correctly

- [ ] 8.4 Integration tests
  - [ ] 8.4.1 Test desktop plugin registers desktop service correctly
  - [ ] 8.4.2 Test mobile plugin registers mobile service correctly
  - [ ] 8.4.3 Test controllers work with both implementations via interface
  - [ ] 8.4.4 Test desktop behavior unchanged (canvas popups still work)
  - [ ] 8.4.5 Test mobile behavior uses sidebar (no canvas popups)

- [ ] 8.5 Manual testing
  - [ ] 8.5.1 Test desktop: hover/click shows canvas popup
  - [ ] 8.5.2 Test mobile: click shows sidebar
  - [ ] 8.5.3 Test mobile: sidebar can be closed
  - [ ] 8.5.4 Test mobile: comment interactions work in sidebar
  - [ ] 8.5.5 Verify no regressions in comment functionality

## 9. Documentation

- [x] 9.1 Update code comments
  - [x] 9.1.1 Document interface purpose and usage
  - [x] 9.1.2 Document platform-specific implementations
  - [x] 9.1.3 Update JSDoc comments for interface methods

- [x] 9.2 Update plugin documentation if needed
  - [x] 9.2.1 Document service interface pattern (via code comments)
  - [x] 9.2.2 Document mobile vs desktop service registration (via code comments)
