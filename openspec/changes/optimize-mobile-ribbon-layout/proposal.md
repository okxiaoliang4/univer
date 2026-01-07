## Why

The current mobile ribbon implementation (`MobileRibbon.tsx`) uses a horizontal layout with collapsed/visible groups separation, which doesn't provide an optimal mobile experience. The layout needs to be optimized for mobile devices with:

- **Simplified layout**: Remove classic style support that adds unnecessary complexity
- **Vertical list layout**: Better suited for mobile touch interactions
- **Unified display**: Show all toolbar items uniformly without collapsing logic
- **Clear visual hierarchy**: Use list items with icons, titles, and indicators for better UX
- **Drawer-based navigation**: Selector items should open drawers for sub-options, following mobile UI patterns

## What Changes

- **Remove classic style support**: Eliminate `ribbonType === 'classic'` handling from MobileRibbon
- **Refactor toolbar layout**: 
  - Remove `visibleGroups` and `collapsedIds` separation logic
  - Change from horizontal grid layout to vertical list layout
  - Display all toolbar items uniformly in a single vertical list
- **Create MobileToolbarItem component**: 
  - New component `MobileToolbarItem.tsx` based on `ToolbarItem.tsx`
  - Support two rendering modes: `renderButtonType` and `renderSelectorType`
  - List item layout: icon (left), title (center), arrow indicator (right, for selectors only)
- **Group separators**: Add horizontal divider lines between toolbar groups
- **Selector drawer integration**: 
  - Selector items open a Drawer component
  - Drawer displays selector options following existing rendering rules
  - Button items maintain original click behavior from ToolbarItem.tsx

## Impact

- **Affected specs**: `ui-mobile-components` (MODIFIED requirement for mobile ribbon layout)
- **Affected code**:
  - `packages/ui/src/views/components/ribbon/MobileRibbon.tsx` (major refactor)
  - `packages/ui/src/views/components/ribbon/MobileToolbarItem.tsx` (new file)
  - `packages/ui/src/controllers/ui/ui.controller.ts` (remove classic from RibbonType for mobile, or handle separately)
