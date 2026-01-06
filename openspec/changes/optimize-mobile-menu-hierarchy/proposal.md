## Why

The current mobile menu implementation (`MobileMenu.tsx`) flattens all menu items recursively, removing hierarchical structure. This creates a poor user experience on mobile devices when menus have nested items, as users lose the visual organization and context of menu groupings.

By implementing Drawer-based navigation for hierarchical menus, we can:
- **Preserve menu hierarchy**: Users can navigate through menu levels naturally
- **Improve UX**: Drawer provides a native mobile navigation pattern that users expect
- **Maintain consistency**: Aligns with existing mobile components (like `MobileSidebar`) that use Drawer
- **Support all menu types**: Properly handle SUBITEMS, SELECTOR, and menu items with children from IMenuSchema

## What Changes

- **MobileMenu.tsx**: Refactor to support hierarchical menus using Drawer component
  - Remove the recursive flattening logic that removes menu hierarchy
  - Detect when a menu item has children (via `IMenuSchema.children` or `SUBITEMS` type with `menuItem.id`)
  - Render menu items with children as clickable items that open a Drawer
  - Drawer should display the children menu items recursively
  - Support all menu item types (BUTTON, SELECTOR, SUBITEMS) similar to desktop Menu.tsx

- **Drawer integration**:
  - Use `Drawer` component from `@univerjs/design` (already available)
  - Drawer opens from the right side (standard mobile pattern)
  - Drawer header shows the parent menu item's title/label
  - Drawer content contains the children menu items
  - Support nested Drawers for multi-level hierarchies

- **Type support**: Reference desktop Menu.tsx implementation patterns
  - Handle `MenuItemType.SUBITEMS` by fetching submenu via `menuManagerService.getMenuByPositionKey(menuItem.id)`
  - Handle `IMenuSchema.children` array for menu items with nested structure
  - Support SELECTOR type with selections (similar to desktop implementation)

## Impact

- **Affected specs**: `ui-mobile-components` (ADDED requirement for mobile menu hierarchical navigation)
- **Affected code**:
  - `packages/ui/src/components/menu/mobile/MobileMenu.tsx` (major refactor)
  - May need updates to menu item rendering logic

- **Breaking changes**: None expected - the component API (`IBaseMenuProps`) remains the same
- **Dependencies**: Uses existing `@univerjs/design` Drawer component (already in dependencies)

## Compatibility Concerns

1. **Backward compatibility**: The component should still work for menus without hierarchy (flat menus)
2. **Menu item click handling**: Need to ensure `onOptionSelect` callback works correctly with Drawer navigation
3. **Nested Drawer support**: May need to manage multiple Drawer instances for deeply nested menus
4. **Performance**: Consider lazy loading of Drawer content for large menu hierarchies
