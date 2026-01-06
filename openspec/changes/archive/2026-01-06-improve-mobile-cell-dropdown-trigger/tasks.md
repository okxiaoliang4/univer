## 1. Implementation

- [x] 1.1 Create mobile dropdown trigger controller `packages/sheets-data-validation-ui/src/controllers/mobile/mobile-dropdown-trigger.controller.ts`
  - [x] 1.1.1 Inject `HoverManagerService` to listen to cell click events
  - [x] 1.1.2 Inject `DataValidationDropdownManagerService` to show dropdown
  - [x] 1.1.3 Inject `SheetDataValidationModel` to check if cell has dropdown
  - [x] 1.1.4 Subscribe to `currentClickedCell$` observable
  - [x] 1.1.5 When cell is clicked, check if it has data validation dropdown
  - [x] 1.1.6 If dropdown exists, automatically show it via `showDataValidationDropdown` command

- [x] 1.2 Register mobile controller in mobile plugin
  - [x] 1.2.1 Add controller to `UniverSheetsDataValidationMobileUIPlugin.onStarting()`
  - [x] 1.2.2 Initialize controller in `onRendered()` lifecycle

- [x] 1.3 Ensure dropdown widget doesn't interfere with whole-cell click
  - [x] 1.3.1 Verify dropdown icon click still works on mobile
  - [x] 1.3.2 Ensure whole-cell click doesn't conflict with existing dropdown widget behavior
  - [x] 1.3.3 Handle edge cases (e.g., clicking dropdown icon vs cell area)

## 2. Testing

- [x] 2.1 Unit tests for mobile dropdown trigger controller
  - [x] 2.1.1 Test dropdown opens when clicking cell with data validation
  - [x] 2.1.2 Test no action when clicking cell without data validation
  - [x] 2.1.3 Test dropdown doesn't open if already open for same cell
  - [x] 2.1.4 Test dropdown closes when clicking different cell

- [x] 2.2 Integration tests
  - [x] 2.2.1 Test mobile plugin registration and initialization
  - [x] 2.2.2 Test that desktop behavior is unaffected
  - [x] 2.2.3 Test dropdown icon click still works on mobile

- [x] 2.3 Manual testing on mobile devices/browsers
  - [x] 2.3.1 Test tapping cell with dropdown opens dropdown menu
  - [x] 2.3.2 Test tapping dropdown icon still works
  - [x] 2.3.3 Test tapping cell without dropdown doesn't show dropdown
  - [x] 2.3.4 Test dropdown closes when tapping outside or selecting option
  - [x] 2.3.5 Test works with different dropdown types (list, date, color, cascader)

## 3. Documentation

- [x] 3.1 Update plugin README if needed
- [x] 3.2 Document mobile dropdown behavior in examples
