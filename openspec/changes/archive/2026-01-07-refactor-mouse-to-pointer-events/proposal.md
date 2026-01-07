## Why

The current codebase uses `onMouse*` events (e.g., `onMouseUp`, `onMouseDown`, `onMouseMove`) in React components, which are not compatible with touch interactions on mobile devices. This causes focus and interaction issues on mobile, as demonstrated by the thread comment editor focus bug where `onMouseUp` didn't properly handle touch events. The Pointer Events API provides a unified interface for mouse, touch, and pen interactions, ensuring consistent behavior across desktop and mobile platforms.

## What Changes

- Replace all `onMouse*` event handlers in React components with `onPointer*` equivalents:
  - `onMouseUp` → `onPointerUp`
  - `onMouseDown` → `onPointerDown`
  - `onMouseMove` → `onPointerMove`
  - `onMouseEnter` → `onPointerEnter`
  - `onMouseLeave` → `onPointerLeave`
  - `onMouseOver` → `onPointerOver`
  - `onMouseOut` → `onPointerOut`
- Update event type references from `MouseEvent` to `PointerEvent` where applicable in TypeScript
- Ensure all pointer event handlers properly handle both mouse and touch interactions
- Maintain backward compatibility with existing mouse-based interactions on desktop

## Impact

- Affected specs: New capability `ui-events` (event handling standards)
- Affected code:
  - `packages/docs-ui/src/views/rich-text-editor/index.tsx` - Replace `onMouseUp` with `onPointerUp`
  - `packages/ui/src/views/components/ribbon/ToolbarButton.tsx` - Replace mouse events
  - `packages/sheets-conditional-formatting-ui/src/components/panel/rule-list/index.tsx` - Replace mouse events
  - `packages/design/src/components/dialog/Dialog.tsx` - Replace mouse events
  - `packages/sheets-data-validation-ui/src/views/components/item/index.tsx` - Replace mouse events
  - `packages/thread-comment-ui/src/views/thread-comment-panel/index.tsx` - Replace mouse events
  - `packages/sheets-ui/src/views/clipboard/ClipboardPopupMenu.tsx` - Replace mouse events
  - `packages/sheets-formula-ui/src/views/formula-editor/search-function/SearchFunction.tsx` - Replace mouse events
  - `packages/design/src/components/tooltip/Tooltip.tsx` - Replace mouse events
  - `packages/sheets-formula-ui/src/views/more-functions/select-function/SelectFunction.tsx` - Replace mouse events
  - `packages/ui/src/components/slider/Slider.tsx` - Replace mouse events
  - `packages/sheets-formula-ui/src/views/formula-editor/index.tsx` - Replace mouse events
  - `packages/sheets-ui/src/views/permission/panel-list/index.tsx` - Replace mouse events
  - `packages/sheets-sort-ui/src/views/CustomSortPanel.tsx` - Replace mouse events
  - `packages/slides-ui/src/components/image-popup-menu/ImagePopupMenu.tsx` - Replace mouse events
  - `packages/sheets-ui/src/views/auto-fill-popup-menu/AutoFillPopupMenu.tsx` - Replace mouse events
  - `packages/sheets-table-ui/src/views/components/SheetTableThemePanel.tsx` - Replace mouse events
  - `packages/drawing-ui/src/views/image-popup-menu/ImagePopupMenu.tsx` - Replace mouse events
  - `packages/docs-ui/src/components/paragraph-menu/index.tsx` - Replace mouse events
  - `packages/docs-quick-insert-ui/src/views/QuickInsertPopup.tsx` - Replace mouse events
  - `packages/docs-mention-ui/src/views/mention-list/index.tsx` - Replace mouse events
  - Additional files identified via grep search (31 total files)
