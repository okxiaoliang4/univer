## 1. Add SubMenu Position Support

- [x] 1.1 Add `SUBMENU = 'ribbon.subMenu'` enum value to `RibbonPosition` in `packages/ui/src/services/menu/types.ts`
- [x] 1.2 Update `MenuManagerService` to initialize subMenu position (`RibbonPosition.SUBMENU`) in menu schema structure under `MenuManagerPosition.RIBBON`
- [x] 1.3 Update `MobileRibbon.tsx` to fetch menu items using `menuManagerService.getMenuByPositionKey(RibbonPosition.SUBMENU)`
- [x] 1.4 Render subMenu items using `ToolbarItem` component (not `MobileToolbarItem`) in the subMenu area
- [x] 1.5 Handle hidden state observables for subMenu items similar to main ribbon items

## 2. Create Mobile Keyboard Menu Schema

- [x] 2.1 Create `packages/mobile-keyboard-ui/src/controllers/menu.schema.ts` with keyboard toggle menu item
- [x] 2.2 Define menu schema using `KeyboardToggleKeyboardOperation.id` as the command
- [x] 2.3 Configure menu item with icon (KeyboardIcon) only - no title, tooltip, or custom label
- [x] 2.4 Set appropriate order value for menu item positioning

## 3. Register Menu Schema

- [x] 3.1 Update `MobileKeyboardController` to inject `IMenuManagerService`
- [x] 3.2 Call `menuManagerService.mergeMenu(menuSchema)` in controller initialization
- [x] 3.3 Ensure proper disposal of menu registration

## 5. Testing

- [ ] 5.1 Verify keyboard toggle menu item appears in MobileRibbon subMenu area
- [ ] 5.2 Verify clicking the menu item toggles keyboard visibility
- [ ] 5.3 Verify menu item reflects keyboard visibility state (activated/inactive)
- [ ] 5.4 Verify menu item respects keyboard enabled state (disabled when keyboard disabled)
- [ ] 5.5 Verify menu item works correctly with existing OperationToolbar keyboard button
- [ ] 5.6 Test on mobile viewport sizes
