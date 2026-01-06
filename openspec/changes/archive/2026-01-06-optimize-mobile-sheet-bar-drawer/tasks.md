## 1. Implementation

- [x] 1.1 Import required components and icons in `MobileSheetBar.tsx`
  - Import `Drawer`, `DrawerContent`, `DrawerHeader`, `DrawerTitle`, `DrawerClose` from `@univerjs/design`
  - Import `MoreDownIcon` from `@univerjs/icons`
  - Import `UIMenu`, `ContextMenuPosition` from `@univerjs/ui`

- [x] 1.2 Add state management for Drawer visibility
  - Add `drawerOpen` state using `useState<boolean>(false)`
  - Add handler `onDrawerOpenChange` to manage drawer state

- [x] 1.3 Modify active sheet item rendering to include dropdown icon
  - Conditionally render `MoreDownIcon` when `sheet.sheetId === activeKey`
  - Position icon to the right of the sheet label
  - Style icon appropriately (size, color, spacing)

- [x] 1.4 Update click handler for active sheet item
  - Modify `onTabClick` to check if clicked sheet is already active
  - If active, open drawer instead of switching sheets
  - If not active, switch to the clicked sheet (existing behavior)

- [x] 1.5 Implement Drawer component structure
  - Wrap Drawer with `open` prop bound to `drawerOpen` state
  - Set `direction="bottom"` for bottom-up animation
  - Add `onOpenChange` handler

- [x] 1.6 Implement DrawerHeader with sheet name
  - Use `DrawerHeader` component
  - Use `DrawerTitle` to display current active sheet name
  - Add `DrawerClose` button in header

- [x] 1.7 Implement DrawerContent with UIMenu
  - Use `DrawerContent` component
  - Render `UIMenu` with `menuType={ContextMenuPosition.FOOTER_TABS}`
  - Implement `onOptionSelect` handler to execute commands and close drawer
  - Pass `activeKey` (current sheet ID) to menu commands

- [x] 1.8 Handle menu option selection
  - In `onOptionSelect`, extract `commandId`, `value`, and `label`
  - Execute command via `commandService.executeCommand`
  - Pass `subUnitId: activeKey` to command params
  - Close drawer after command execution

## 2. Testing

- [x] 2.1 Test dropdown icon visibility
  - Verify icon only appears on active sheet item
  - Verify icon disappears when another sheet becomes active

- [x] 2.2 Test Drawer opening behavior
  - Verify drawer opens when clicking active sheet item
  - Verify drawer does not open when clicking inactive sheet items

- [x] 2.3 Test Drawer content
  - Verify sheet name appears in DrawerTitle
  - Verify all menu options from `FOOTER_TABS` context menu are displayed
  - Verify menu options are properly styled and clickable

- [x] 2.4 Test menu command execution
  - Test each menu option (rename, delete, copy, change color, hide, protection, etc.)
  - Verify commands execute correctly with correct `subUnitId`
  - Verify drawer closes after command execution

- [x] 2.5 Test Drawer dismissal
  - Verify drawer closes when clicking backdrop
  - Verify drawer closes when clicking close button
  - Verify drawer closes when swiping down

- [x] 2.6 Test state synchronization
  - Verify sheet list updates correctly after menu operations
  - Verify active sheet updates correctly after switching sheets
  - Verify drawer state resets when sheet changes

## 3. Validation

- [x] 3.1 Run linting and type checking
  - Ensure no ESLint errors
  - Ensure no TypeScript errors

- [x] 3.2 Verify mobile-specific behavior
  - Test on mobile viewport sizes
  - Verify touch interactions work correctly
  - Verify drawer animation is smooth

- [x] 3.3 Verify desktop behavior unchanged
  - Ensure desktop sheet bar still works as before
  - Ensure no regressions in desktop functionality
