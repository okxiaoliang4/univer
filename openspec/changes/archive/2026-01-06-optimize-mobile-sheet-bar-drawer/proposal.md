## Why

The current mobile sheet bar implementation lacks an intuitive way to access sheet context menu options. Users need to access operations like rename, delete, copy, change color, and protection settings for sheets on mobile devices. Currently, the mobile sheet bar only allows switching between sheets but doesn't provide access to the context menu that's available on desktop via right-click.

## What Changes

- Add a dropdown arrow icon (`MoreDownIcon`) to the right side of the active sheet item's label
- When the active sheet item is clicked, open a `Drawer` component from the bottom
- The Drawer title displays the current sheet name
- The Drawer content displays the same `UIMenu` with `ContextMenuPosition.FOOTER_TABS` that's used in the desktop `SheetBarTabs` component
- The Drawer closes when a menu option is selected or when the user dismisses it

## Impact

- Affected specs: `sheets-ui-mobile` (modification)
- Affected code:
  - `packages/sheets-ui/src/views/mobile/sheet-bar/MobileSheetBar.tsx` - Main component modification
  - `packages/design/src/components/drawer/Drawer.tsx` - Already exists, will be used
  - `packages/sheets-ui/src/controllers/menu.schema.ts` - Menu schema already exists for `FOOTER_TABS`
- Dependencies: Uses existing `Drawer` component and `UIMenu` with `ContextMenuPosition.FOOTER_TABS`
