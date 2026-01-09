## Why

Mobile users need quick access to the keyboard toggle functionality directly from the ribbon interface. Currently, the keyboard toggle button is only available in the OperationToolbar component, which may not always be visible or accessible. Adding a keyboard toggle menu item to the MobileRibbon's subMenu area provides consistent access to this critical mobile feature.

## What Changes

- Add a new menu position key for MobileRibbon subMenu area
- Create `menu.schema.ts` in `packages/keyboard-ui/src/controllers/` to register the keyboard toggle menu item
- Update `MobileRibbon.tsx` to render menu items from the subMenu position using `ToolbarItem` component (not `MobileToolbarItem`)
- Register the menu schema in `KeyboardController` to merge it with the menu system
- Configure the menu item with KeyboardIcon icon only (no custom label, title, or tooltip)

## Impact

- Affected specs: `ui-mobile-components` (MODIFIED - adding subMenu rendering capability)
- Affected code:
  - `packages/ui/src/views/components/ribbon/MobileRibbon.tsx` - Add subMenu rendering logic
  - `packages/ui/src/services/menu/types.ts` - Add new RibbonSubMenu position enum (if needed)
  - `packages/keyboard-ui/src/controllers/` - Add menu.schema.ts and menu controller registration
  - `packages/keyboard-ui/src/controllers/keyboard.controller.ts` - Register menu schema
