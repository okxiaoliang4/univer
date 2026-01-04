## Why

The mobile sidebar component (`MobileSidebar.tsx`) currently uses custom rendering implementations. Refactoring it to use the standardized `Drawer` component from `@univerjs/design` will:

- **Improve consistency**: Use a unified drawer component across the mobile UI
- **Reduce maintenance**: Leverage the well-tested vaul-based Drawer component
- **Better UX**: Benefit from vaul's built-in gesture support and animations
- **Code reuse**: Eliminate duplicate drawer-like implementations

## What Changes

- **MobileSidebar.tsx**: Refactor to use `Drawer` component instead of custom `<section>` elements
  - Map `ISidebarMethodOptions` properties to `Drawer` props
  - Preserve scroll container management and scroll event handling
  - Maintain `header`, `children`, `footer`, `bodyStyle`, `width`, `visible`, `onClose`, `onOpen` compatibility

- **Property compatibility**: Map existing properties to Drawer props:
  - `visible` → `open` (for sidebar)
  - `header` → `DrawerHeader` with `DrawerTitle` (for sidebar)
  - `bodyStyle` → applied to `DrawerContent` (for sidebar)
  - `width` → applied via `DrawerContent` style
  - `onClose`/`onOpen` → `onOpenChange` callback

## Impact

- **Affected specs**: `ui-mobile-components` (new capability)
- **Affected code**:
  - `packages/ui/src/views/components/sidebar/MobileSidebar.tsx`
  - `packages/ui/src/services/sidebar/mobile-sidebar.service.ts` (may need updates for container management)

- **Breaking changes**: None expected - all existing APIs remain compatible
- **Dependencies**: Uses existing `@univerjs/design` Drawer component (already in dependencies)

## Compatibility Concerns

The following properties require special attention and may need clarification:

1. **Sidebar scroll container management**: `MobileSidebar` currently manages a scroll container reference via `sidebarService.setContainer()`. This needs to be preserved when using Drawer.

2. **Sidebar scroll events**: `MobileSidebar` emits scroll events via `sidebarService.scrollEvent$`. This needs to be maintained.

3. **Sidebar `onOpen` callback**: Sidebar has `onOpen` callback but Drawer uses `onOpenChange`. Need to map appropriately.
