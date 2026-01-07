## 1. Analysis and Planning

- [x] 1.1 Audit all `onMouse*` event handlers in React components
  - Use grep to find all instances: `grep -r "onMouse" packages/`
  - Document each file and specific event handler that needs updating
  - Identify any edge cases or special handling required
- [x] 1.2 Review existing pointer event usage in codebase
  - Check `packages/engine-render/src/base-object.ts` for pointer event patterns
  - Review `packages/sheets-ui/src/services/selection/mobile-selection-render.service.ts` for reference implementation
  - Understand how `IPointerEvent` and `IMouseEvent` are used together
- [x] 1.3 Identify event handler dependencies
  - Check if any handlers rely on mouse-specific properties (e.g., `button`, `which`)
  - Document any handlers that need special touch handling
  - Note any handlers that should remain as mouse-only (if any)

## 2. Core Component Updates

- [x] 2.1 Update `packages/docs-ui/src/views/rich-text-editor/index.tsx`
  - Replace `onMouseUp` with `onPointerUp`
  - Update event type from `MouseEvent` to `PointerEvent` if needed
  - Test focus behavior on both desktop and mobile
- [x] 2.2 Update `packages/thread-comment-ui/src/views/thread-comment-editor/index.tsx`
  - Verify `onPointerDown` is correctly implemented (already partially done)
  - Ensure consistent pointer event usage throughout component
- [x] 2.3 Update UI component library files
  - `packages/ui/src/views/components/ribbon/ToolbarButton.tsx`
  - `packages/design/src/components/dialog/Dialog.tsx`
  - `packages/design/src/components/tooltip/Tooltip.tsx`
  - `packages/ui/src/components/slider/Slider.tsx`
- [x] 2.4 Update sheets-related components
  - `packages/sheets-conditional-formatting-ui/src/components/panel/rule-list/index.tsx`
  - `packages/sheets-data-validation-ui/src/views/components/item/index.tsx`
  - `packages/sheets-ui/src/views/clipboard/ClipboardPopupMenu.tsx`
  - `packages/sheets-formula-ui/src/views/formula-editor/search-function/SearchFunction.tsx`
  - `packages/sheets-formula-ui/src/views/more-functions/select-function/SelectFunction.tsx`
  - `packages/sheets-formula-ui/src/views/formula-editor/index.tsx`
  - `packages/sheets-ui/src/views/permission/panel-list/index.tsx`
  - `packages/sheets-sort-ui/src/views/CustomSortPanel.tsx`
  - `packages/sheets-ui/src/views/auto-fill-popup-menu/AutoFillPopupMenu.tsx`
  - `packages/sheets-table-ui/src/views/components/SheetTableThemePanel.tsx`
- [x] 2.5 Update thread comment components
  - `packages/thread-comment-ui/src/views/thread-comment-panel/index.tsx`
- [x] 2.6 Update docs-related components
  - `packages/docs-ui/src/components/paragraph-menu/index.tsx`
  - `packages/docs-quick-insert-ui/src/views/QuickInsertPopup.tsx`
  - `packages/docs-mention-ui/src/views/mention-list/index.tsx`
- [x] 2.7 Update slides and drawing components
  - `packages/slides-ui/src/components/image-popup-menu/ImagePopupMenu.tsx`
  - `packages/drawing-ui/src/views/image-popup-menu/ImagePopupMenu.tsx`

## 3. Event Handler Logic Updates

- [x] 3.1 Update event property access
  - Replace `event.button` with `event.pointerType` checks where needed
  - Update any `event.which` or `event.buttons` usage to pointer equivalents
  - Ensure `event.preventDefault()` and `event.stopPropagation()` work correctly
- [x] 3.2 Handle pointer type detection
  - Add logic to distinguish between mouse, touch, and pen if needed
  - Ensure touch events don't trigger unwanted behaviors
  - Maintain desktop mouse behavior compatibility
- [x] 3.3 Update event listener registrations
  - Check for any `addEventListener('mouse*')` calls that need updating
  - Update to `addEventListener('pointer*')` where appropriate
  - Ensure proper cleanup in `removeEventListener` calls

## 4. TypeScript Type Updates

- [x] 4.1 Update event type imports
  - Replace `MouseEvent` imports with `PointerEvent` where applicable
  - Update type annotations in function signatures
  - Ensure compatibility with React's `PointerEvent` types
- [x] 4.2 Update custom event type definitions
  - Review `packages/engine-render/src/basics/i-events.ts` for event type definitions
  - Ensure `IPointerEvent` and `IMouseEvent` types are properly used
  - Update any union types that include `MouseEvent`

## 5. Testing

- [x] 5.1 Test desktop mouse interactions
  - Verify all mouse-based interactions still work correctly
  - Test click, drag, hover behaviors
  - Ensure no regressions in desktop functionality
- [x] 5.2 Test mobile touch interactions
  - Test touch events on mobile devices or emulators
  - Verify tap, swipe, long-press behaviors
  - Ensure focus and selection work correctly on mobile
- [x] 5.3 Test edge cases
  - Test rapid clicking/tapping
  - Test simultaneous mouse and touch (if applicable)
  - Test pen/stylus input (if available)
- [x] 5.4 Write unit tests for updated components
  - Add tests for pointer event handlers
  - Test both mouse and touch event scenarios
  - Ensure event propagation works correctly

## 6. Documentation

- [x] 6.1 Update component documentation
  - Add JSDoc comments explaining pointer event usage
  - Document any special handling for touch vs mouse
  - Update prop type documentation
- [x] 6.2 Update development guidelines
  - Add pointer event best practices to `CLAUDE.md` or `AGENTS.md`
  - Document when to use pointer vs mouse events
  - Provide examples of correct pointer event usage

## 7. Validation

- [x] 7.1 Run TypeScript type checking
  - Execute `pnpm typecheck` to ensure no type errors
  - Fix any type mismatches
- [x] 7.2 Run linting
  - Execute `pnpm lint` to ensure code style compliance
  - Fix any linting errors
- [x] 7.3 Run tests
  - Execute `pnpm test` to ensure all tests pass
  - Update tests that depend on mouse events
- [x] 7.4 Manual testing
  - Test on desktop browsers (Chrome, Firefox, Safari, Edge)
  - Test on mobile devices (iOS Safari, Android Chrome)
  - Verify thread comment editor focus issue is resolved
  - Test other critical user interactions
