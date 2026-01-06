## 1. Implementation

- [ ] 1.1 Remove flattening logic from MobileMenu component
  - Remove the `flatMenuItems` useMemo that recursively flattens menu items
  - Keep the original menu structure with hierarchy intact

- [ ] 1.2 Add Drawer component integration
  - Import Drawer components from `@univerjs/design`
  - Create state management for Drawer open/close state
  - Track which menu item's Drawer is currently open

- [ ] 1.3 Implement hierarchical menu item rendering
  - Detect menu items with children (check `IMenuSchema.children` array)
  - Detect SUBITEMS type menu items (check `menuItem.type === MenuItemType.SUBITEMS`)
  - Render menu items with children as clickable items that trigger Drawer

- [ ] 1.4 Create Drawer content component
  - Create component to render menu items inside Drawer
  - Support recursive rendering for nested hierarchies
  - Handle all menu item types (BUTTON, SELECTOR, SUBITEMS)
  - Reference desktop Menu.tsx patterns for type handling

- [ ] 1.5 Implement SUBITEMS type support
  - Fetch submenu items via `menuManagerService.getMenuByPositionKey(menuItem.id)`
  - Render fetched submenu items in Drawer
  - Handle empty submenu cases

- [ ] 1.6 Implement SELECTOR type support
  - Handle SELECTOR menu items with selections
  - Render selections as menu options in Drawer
  - Support observable selections similar to desktop implementation

- [ ] 1.7 Add Drawer header and navigation
  - Display parent menu item title/label in Drawer header
  - Add back button or close button in Drawer header
  - Handle Drawer close events properly

- [ ] 1.8 Support nested Drawers
  - Allow opening nested Drawers when children also have children
  - Manage Drawer state for multiple levels
  - Ensure proper cleanup when navigating between levels

## 2. Testing

- [ ] 2.1 Test flat menus (no hierarchy)
  - Verify menus without children still render correctly
  - Ensure no Drawer appears for flat menus

- [ ] 2.2 Test single-level hierarchy
  - Verify Drawer opens when clicking menu item with children
  - Verify children render correctly in Drawer
  - Verify menu item click handlers work correctly

- [ ] 2.3 Test multi-level hierarchy
  - Verify nested Drawers work correctly
  - Verify navigation between levels
  - Verify proper cleanup when closing Drawers

- [ ] 2.4 Test SUBITEMS type menu items
  - Verify SUBITEMS type triggers Drawer with fetched submenu
  - Verify submenu items render correctly
  - Verify empty submenu handling

- [ ] 2.5 Test SELECTOR type menu items
  - Verify SELECTOR type renders selections in Drawer
  - Verify selection click handlers work correctly
  - Test with observable selections

- [ ] 2.6 Test menu item states
  - Verify disabled menu items don't open Drawer
  - Verify hidden menu items are not rendered
  - Verify activated state display

## 3. Validation

- [ ] 3.1 Visual regression testing
  - Update Playwright snapshots if needed
  - Verify mobile menu appearance matches design

- [ ] 3.2 Cross-device testing
  - Test on various mobile screen sizes
  - Verify Drawer behavior on different devices

- [ ] 3.3 Performance testing
  - Verify no performance regression with Drawer implementation
  - Test with large menu hierarchies
