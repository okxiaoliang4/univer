## 1. Implementation

- [x] 1.1 Create mobile plugin file `packages/sheets-thread-comment-ui/src/mobile-plugin.ts`
  - [x] 1.1.1 Extend `Plugin` class with proper dependencies (`UniverSheetsThreadCommentUIPlugin`, `UniverMobileUIPlugin`)
  - [x] 1.1.2 Register mobile-specific controllers in `onStarting()` lifecycle
  - [x] 1.1.3 Follow naming convention: `UniverSheetsThreadCommentMobileUIPlugin`

- [x] 1.2 Create mobile popup controller `packages/sheets-thread-comment-ui/src/controllers/mobile/mobile-popup.controller.ts`
  - [x] 1.2.1 Inject `ISidebarService` instead of canvas popup services
  - [x] 1.2.2 Subscribe to `SheetsThreadCommentPopupService.activePopup$` observable
  - [x] 1.2.3 When popup state changes, open/close sidebar with `SheetsThreadCommentCell` component
  - [x] 1.2.4 Map popup service calls to sidebar service calls for mobile platform
  - [x] 1.2.5 Handle sidebar close events to sync with popup service state
  - [x] 1.2.6 Handle `persistPopup()` behavior appropriately in sidebar context

- [x] 1.3 Update `SheetsThreadCommentCell` component for mobile sidebar context
  - [x] 1.3.1 Ensure component works correctly when rendered inside sidebar
  - [x] 1.3.2 Adjust styling if needed for mobile sidebar layout (remove fixed width constraints)
  - [x] 1.3.3 Verify all comment interactions work properly in sidebar context
  - [x] 1.3.4 Ensure `onClose` callback properly closes sidebar on mobile

- [x] 1.4 Register mobile plugin in main plugin exports
  - [x] 1.4.1 Export mobile plugin from `packages/sheets-thread-comment-ui/src/index.ts` (if needed)
  - [x] 1.4.2 Ensure mobile plugin can be imported and used in mobile examples

## 2. Testing

- [x] 2.1 Unit tests for mobile popup controller
  - [x] 2.1.1 Test sidebar opens when popup state changes
  - [x] 2.1.2 Test sidebar closes when popup is hidden
  - [x] 2.1.3 Test component key is correctly passed to sidebar service
  - [x] 2.1.4 Test sidebar close syncs with popup service state

- [x] 2.2 Integration tests
  - [x] 2.2.1 Test mobile plugin registration and initialization
  - [x] 2.2.2 Test that desktop behavior is unaffected
  - [x] 2.2.3 Test popup persistence behavior in sidebar context

- [x] 2.3 Manual testing on mobile devices/browsers
  - [x] 2.3.1 Test thread comment popup opens in sidebar on hover/click
  - [x] 2.3.2 Test sidebar can be closed via close button
  - [x] 2.3.3 Test comment interactions (add, reply, edit, delete) work correctly
  - [x] 2.3.4 Test popup persistence (temp vs persistent) behavior
  - [x] 2.3.5 Test multiple comments display correctly in sidebar

## 3. Documentation

- [x] 3.1 Update plugin README if needed
- [x] 3.2 Document mobile plugin usage in examples
