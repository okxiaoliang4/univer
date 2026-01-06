## 1. Implementation

- [x] 1.1 Create mobile plugin file `packages/sheets-hyper-link-ui/src/mobile-plugin.ts`
  - [x] 1.1.1 Extend `Plugin` class with proper dependencies (`UniverSheetsHyperLinkUIPlugin`, `UniverMobileUIPlugin`)
  - [x] 1.1.2 Register mobile-specific controllers in `onStarting()` lifecycle
  - [x] 1.1.3 Follow naming convention: `UniverSheetsHyperLinkMobileUIPlugin`

- [x] 1.2 Create mobile popup controller `packages/sheets-hyper-link-ui/src/controllers/mobile/mobile-popup.controller.ts`
  - [x] 1.2.1 Inject `ISidebarService` instead of canvas popup services
  - [x] 1.2.2 Subscribe to `SheetsHyperLinkPopupService.currentEditing$` observable
  - [x] 1.2.3 When editing state changes, open/close sidebar with `CellLinkEdit` component
  - [x] 1.2.4 Map popup service calls to sidebar service calls for mobile platform
  - [x] 1.2.5 Handle sidebar close events to sync with popup service state

- [x] 1.3 Update `CellLinkEdit` component for mobile sidebar context
  - [x] 1.3.1 Ensure component works correctly when rendered inside sidebar
  - [x] 1.3.2 Adjust styling if needed for mobile sidebar layout (remove fixed width constraints)
  - [x] 1.3.3 Verify all form interactions work properly in sidebar context

- [x] 1.4 Register mobile plugin in main plugin exports
  - [x] 1.4.1 Export mobile plugin from `packages/sheets-hyper-link-ui/src/index.ts` (if needed)
  - [x] 1.4.2 Ensure mobile plugin can be imported and used in mobile examples

## 2. Testing

- [x] 2.1 Unit tests for mobile popup controller
  - [x] 2.1.1 Test sidebar opens when editing state changes
  - [x] 2.1.2 Test sidebar closes when editing ends
  - [x] 2.1.3 Test component key is correctly passed to sidebar service

- [x] 2.2 Integration tests
  - [x] 2.2.1 Test mobile plugin registration and initialization
  - [x] 2.2.2 Test that desktop behavior is unaffected

- [x] 2.3 Manual testing on mobile devices/browsers
  - [x] 2.3.1 Test hyperlink editing opens in sidebar
  - [x] 2.3.2 Test sidebar can be closed via close button
  - [x] 2.3.3 Test form submission works correctly
  - [x] 2.3.4 Test all hyperlink types (URL, Range, Sheet, Define Name) work in sidebar

## 3. Documentation

- [x] 3.1 Update plugin README if needed
- [x] 3.2 Document mobile plugin usage in examples
