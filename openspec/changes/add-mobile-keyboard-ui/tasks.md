## 1. Foundation - Service and State Management

- [x] 1.1 Create `IMobileKeyboardService` interface in `packages/sheets-ui/src/services/mobile/mobile-keyboard.service.ts`
  - Define mobile-specific observables: `isKeyboardVisible$` (for keyboard UI, not editor), `keyboardMode$`
  - Note: Edit state is managed by existing `EditorBridgeService` (`currentEditCellState$`, `visible$`)
  - Define methods: `showKeyboard()`, `hideKeyboard()`, `setMode()`
  - Add DI token: `createIdentifier<IMobileKeyboardService>('sheets-ui.mobile-keyboard.service')`
  - Note: No `beginEdit()` or `endEdit()` - components call `SetCellEditVisibleOperation` directly
- [x] 1.2 Implement `MobileKeyboardService` class with lightweight state management
  - Track keyboard UI visibility state (separate from editor visibility)
  - Track active mode ('formula' | 'number' | 'text')
  - Inject `IEditorBridgeService`
  - Subscribe to `editorBridgeService.visible$` to sync keyboard UI visibility with editor state
  - Implement `showKeyboard()` and `hideKeyboard()` to manage keyboard UI visibility only
- [ ] 1.3 Write unit tests for `MobileKeyboardService`
  - Test keyboard UI visibility state transitions (separate from editor state)
  - Test mode changes and observable emissions
  - Test subscription to `editorBridgeService.visible$` for synchronization
  - Note: No tests for beginEdit/endEdit since they don't exist in this service
- [x] 1.4 Create `MobileKeyboardController` in `packages/sheets-ui/src/controllers/mobile/mobile-keyboard.controller.ts`
  - Inject `IMobileKeyboardService`, `IEditorBridgeService`, `ICommandService`
  - Subscribe to cell focus events to trigger FAB display logic
  - Handle keyboard show/hide coordination with FAB
  - Integrate with existing `EditorBridgeService` for cell editing state (not duplicate it)

## 2. Common Keyboard Components

- [x] 2.1 Create `KeyboardContainer.tsx` in `views/mobile/keyboard/common/`
  - Main wrapper component for the keyboard interface
  - Subscribe to `isKeyboardVisible$` to control visibility
  - Implement slide-up animation from bottom
  - Handle swipe-down gesture to dismiss keyboard
  - Container should occupy bottom 40-50% of screen height
  - Layout order from top to bottom: OperationToolbar → MobileFormulaBar → ModeSwitcher → Mode-specific keyboard
- [x] 2.2 Create `OperationToolbar.tsx` in `views/mobile/keyboard/common/`
  - Render six operation buttons: Undo, Redo, Copy, Paste, Cut, Clear
  - Use icons from `@univerjs/icons` (or standard Unicode symbols if unavailable)
  - Implement button click handlers using existing command services
  - Disable buttons based on availability (undo/redo history, clipboard state, cell empty state)
  - Even horizontal spacing, consistent button sizing
- [x] 2.3 Create `ModeSwitcher.tsx` in `views/mobile/keyboard/common/`
  - Render five mode buttons: Tab, f(x), 123, ABC, Enter (↵)
  - Highlight active mode button
  - Implement mode switch handlers calling `mobileKeyboardService.setMode()`
  - Tab button: save cell, move selection right, keep keyboard open
  - Enter button: save cell, move selection down, keep keyboard open
  - Even horizontal spacing, consistent button sizing
- [x] 2.4 Integrate MobileFormulaBar into KeyboardContainer
  - Import and render MobileFormulaBar between OperationToolbar and ModeSwitcher
  - Ensure it receives necessary props (className, etc.)
  - MobileFormulaBar will handle its own editor state and confirm button
- [ ] 2.5 Write component tests for common keyboard components
  - Test `OperationToolbar` button clicks and disabled states
  - Test `ModeSwitcher` mode transitions
  - Test `KeyboardContainer` visibility and gesture handling
  - Test `MobileFormulaBar` integration and functionality

## 3. Mobile FormulaBar Component

- [x] 3.1 Create `MobileFormulaBar.tsx` in `views/mobile/keyboard/common/`
  - Copy the entire `FormulaBar.tsx` file as the starting point
  - Remove `DefinedName` component and its container (lines 287-289 from original)
  - Remove left button area with CloseIcon, CheckMarkIcon, and FxIcon (lines 291-336 from original)
  - Remove DropdownIcon and its container (lines 387-400 from original)
  - Keep only: FormulaEditor area + necessary state and handlers
- [x] 3.2 Add CheckMarkIcon to the right side of FormulaEditor
  - Add a new container div after the FormulaEditor div (after line 386 from original)
  - Position CheckMarkIcon on the right edge of the input field
  - Use existing `handleConfirmBtnClick` handler
  - Style with appropriate padding and cursor pointer
- [ ] 3.3 Test MobileFormulaBar functionality
  - Verify FormulaBar displays current cell value correctly
  - Verify editing in FormulaBar updates cell preview
  - Verify CheckMarkIcon saves changes and dismisses keyboard
  - Verify all formula editor features work (syntax highlighting, validation, etc.)

## 4. FAB (Floating Action Button)

- [x] 4.1 Create `KeyboardFab.tsx` in `views/mobile/fab/`
  - Circular button with keyboard icon (use icon from `@univerjs/icons` or emoji ⌨️)
  - Fixed position in bottom-right corner
  - Add shadow/elevation for visual prominence
  - Implement click handler to call `commandService.executeCommand(SetCellEditVisibleOperation.id, { visible: true, ... })`
  - Note: Direct command call, no `mobileKeyboardService.beginEdit()` wrapper
- [x] 4.2 Implement FAB visibility logic
  - Show FAB when cell is focused on mobile device
  - Hide FAB when mobile keyboard is visible (subscribe to `mobileKeyboardService.isKeyboardVisible$`)
  - Re-show FAB when keyboard is dismissed
- [x] 4.3 Register FAB component in `mobile-plugin.ts`
  - Register with `ComponentManager`
  - Ensure it only renders on mobile (when `UniverMobileUIPlugin` is active)
- [ ] 4.4 Test FAB behavior
  - Test FAB appears on cell focus
  - Test FAB tap triggers `SetCellEditVisibleOperation` and opens keyboard
  - Test FAB hides when keyboard is visible
  - Test FAB reappears when keyboard is dismissed

## 5. Formula Keyboard Mode

- [x] 5.1 Create `FormulaKeyboard.tsx` in `views/mobile/keyboard/formula-keyboard/`
  - Layout five rows of keys as specified in design
  - Row 1: Number keys 1-0 (10 keys)
  - Row 2: +, -, ×, ÷, [spacers], f(x), del (9 keys)
  - Row 3: +, (, ), ,, [spacers], Σ, tab (9 keys)
  - Row 4: <, >, :, ., [spacers], "", ↵ (9 keys)
  - Row 5: $, %, &, ^, [spacers], space, A1 (9 keys)
  - Make +, -, ×, ÷ keys larger than number keys
  - Add visual grouping/separation for operator keys
- [ ] 5.2 Create `OperatorKeys.tsx` in `views/mobile/keyboard/formula-keyboard/`
  - Extract operator key logic for reusability
  - Implement key handlers to insert operators into FormulaBar
  - Special handling for × (multiply) and ÷ (divide) symbols
- [x] 5.3 Implement special key handlers
  - Σ key: Insert "=SUM()" with cursor between parentheses
  - f(x) key: Trigger function browser panel (see task 6)
  - "" key: Insert two double quotes with cursor between them
  - space key: Insert space character
  - del key: Delete character before cursor
  - A1 key: Switch to English QWERTY layout (see task 5.6)
  - ↵ key: Save cell and move to next row
- [x] 5.4 Create English keyboard layout variant
  - Add state to track if English layout is active
  - Render alternative layout when English mode is active:
    - Row 1: q w e r t y u i o p
    - Row 2: a s d f g h j k l
    - Row 3: ⇧ z x c v b n m del
    - Row 4: $ : , ! space ↵ back
  - Implement "back" button to return to formula layout
  - Handle uppercase/lowercase with ⇧ key
- [ ] 5.5 Write tests for formula keyboard
  - Test each key inserts correct character/operator
  - Test special key behaviors (Σ, f(x), "", A1)
  - Test English layout toggle
  - Test keyboard layout and spacing

## 6. Function Browser Panel

- [x] 6.1 Create `FunctionBrowser.tsx` in `views/mobile/keyboard/formula-keyboard/`
  - Bottom sheet panel that slides up from bottom (70-80% screen height)
  - Dimmed backdrop behind panel
  - Dismissible by tapping backdrop or swiping down
- [x] 6.2 Implement function browser layout
  - Search input at top
  - Category tabs below search (All, Financial, Date, Math, etc.)
  - Scrollable function list below categories
  - Each item shows function name and brief description
- [x] 6.3 Load function data from formula engine
  - Inject and use existing function list services
  - Fetch `FUNCTION_LIST` from `packages/sheets-formula/src/services/function-list/function-list.ts`
  - Use function metadata for descriptions
- [x] 6.4 Implement search functionality
  - Filter functions by name and description as user types
  - Update list in real-time
- [x] 6.5 Implement category filtering
  - Filter functions by selected category tab
  - Highlight active category tab
  - "All" category shows all functions
- [x] 6.6 Implement function insertion
  - On function item tap, insert function into FormulaBar
  - Insert with "=FUNCTION_NAME()" format
  - Position cursor between parentheses
  - Dismiss panel and return focus to FormulaBar
- [ ] 6.7 Write tests for function browser
  - Test panel open/close behavior
  - Test search filtering
  - Test category filtering
  - Test function insertion

## 7. Number Keyboard Mode

- [x] 7.1 Create `NumberKeyboard.tsx` in `views/mobile/keyboard/number-keyboard/`
  - Layout four rows of keys as specified in design
  - Row 1: Five blank keys (for spacing/alignment)
  - Row 2: +/-, 7, 8, 9, del (5 keys)
  - Row 3: %, 4, 5, 6, tab (5 keys)
  - Row 4: $, 1, 2, 3, ↵ (5 keys)
  - Row 5: ¥, 00, 0, ., ↵ (5 keys)
- [x] 7.2 Implement number key handlers
  - Number keys 0-9: Insert corresponding digit
  - +/- key: Toggle negative sign at beginning of number
  - % key: Insert percent sign
  - $ and ¥ keys: Insert currency symbols
  - 00 key: Insert two zeros
  - . key: Insert decimal point
  - del key: Delete character before cursor
  - tab and ↵ keys: Save and navigate (reuse logic from mode switcher)
- [ ] 7.3 Write tests for number keyboard
  - Test each key inserts correct value
  - Test special keys (+/-, 00, currency symbols)
  - Test keyboard layout and spacing

## 8. Text Keyboard Mode (Native Integration)

- [x] 8.1 Create `TextKeyboard.tsx` in `views/mobile/keyboard/text-keyboard/`
  - Wrapper component that displays only OperationToolbar, FormulaBar, and ModeSwitcher
  - Leave bottom area transparent/empty for native keyboard
- [x] 8.2 Implement hidden input field for native keyboard trigger
  - Create a hidden textarea element
  - Focus textarea when Text mode is activated
  - Sync textarea value with FormulaBar in real-time
  - Handle native keyboard events (input, backspace, enter)
- [x] 8.3 Implement native keyboard show/hide logic
  - Show native keyboard when Text mode is active
  - Handle back button/dismiss native keyboard scenarios
  - Ensure custom keyboard UI remains visible behind native keyboard
- [ ] 8.4 Write tests for text keyboard
  - Test native keyboard activation
  - Test input sync with FormulaBar
  - Test mode switch away from Text mode dismisses native keyboard

## 9. Mode Switching Logic

- [x] 9.2 Implement mode switching in `KeyboardContainer`
  - Subscribe to `keyboardMode$` observable
  - Conditionally render `FormulaKeyboard`, `NumberKeyboard`, or `TextKeyboard` based on mode
  - Animate mode transitions (fade or slide)
- [x] 9.3 Update `ModeSwitcher` component to call `mobileKeyboardService.setMode()`
  - f(x) button → 'formula' mode
  - 123 button → 'number' mode
  - ABC button → 'text' mode
  - Update active button highlight based on current mode
- [x] 9.4 Implement cell type-based auto-mode selection
  - Detect cell type when `beginEdit()` is called
  - Formula cells (starting with =) → auto-select Formula mode
  - Numeric cells → auto-select Number mode
  - Text cells → auto-select Text mode
  - Empty cells → use last-used mode or default to Number mode
  - Store last-used mode in service for persistence
- [ ] 9.5 Write tests for mode switching
  - Test manual mode switching via buttons
  - Test auto-mode selection based on cell type
  - Test last-used mode persistence
  - Test mode transitions and animations

## 10. Plugin Integration and Registration

- [ ] 10.1 Register `IMobileKeyboardService` in `mobile-plugin.ts`
  - Add to dependency injection in `onStarting()` lifecycle
  - Use `createIdentifier` for DI token
- [ ] 10.2 Register `MobileKeyboardController` in `mobile-plugin.ts`
  - Add to dependency injection
  - Ensure controller is initialized during plugin lifecycle
- [ ] 10.3 Register keyboard components in `mobile-plugin.ts`
  - Register FAB component with `ComponentManager`
  - Register keyboard container component
  - Register function browser component
  - Associate components with appropriate part keys
- [ ] 10.4 Ensure mobile-only activation
  - Verify components only render when `UniverMobileUIPlugin` is active
  - Test that FAB and keyboard do not appear on desktop
  - Add conditional rendering based on mobile detection
- [ ] 10.5 Update `SheetUIMobileController` if needed
  - Integrate with existing mobile UI controller
  - Ensure no conflicts with existing mobile render controllers

## 11. Styling and Responsive Design

- [ ] 11.1 Apply Tailwind CSS utility classes to all components
  - Follow existing design system patterns
  - Use responsive classes for different screen sizes
  - Ensure consistent spacing and sizing
- [ ] 11.2 Handle landscape vs portrait orientations
  - Adjust keyboard height/width for landscape mode
  - Ensure FAB positioning works in both orientations
  - Test on common mobile screen sizes (e.g., 375px, 414px widths)
- [ ] 11.3 Add dark mode support
  - Use existing dark mode classes from `@univerjs/design`
  - Ensure keyboard components are visible in both themes
  - Test keyboard appearance in dark mode
- [ ] 11.4 Add accessibility attributes
  - Add ARIA labels to buttons
  - Ensure keyboard navigation works with external keyboard
  - Test with screen reader if possible

## 12. Integration Testing

- [ ] 12.1 Test end-to-end editing workflow
  - Focus cell → Tap FAB → Open keyboard → Edit → Save → Dismiss
  - Verify cell value is updated correctly
  - Verify undo/redo operations work
- [ ] 12.2 Test mode switching workflows
  - Test switching between Formula, Number, and Text modes
  - Verify mode persists across navigation (Tab/Enter)
  - Verify auto-mode selection based on cell type
- [ ] 12.3 Test clipboard operations
  - Copy from one cell, navigate, paste to another cell
  - Cut and paste operations
  - Clear operation
- [ ] 12.4 Test function browser workflow
  - Open function browser → Search function → Insert function
  - Verify function is inserted with correct syntax
  - Verify cursor positioning
- [ ] 12.5 Test keyboard dismissal scenarios
  - Swipe down to dismiss
  - Back button to dismiss
  - Confirm button to save and dismiss
  - Verify FAB reappears after dismissal

## 13. Documentation and Cleanup

- [ ] 13.1 Add JSDoc comments to all public APIs
  - Document `IMobileKeyboardService` interface
  - Document component props and behaviors
  - Add usage examples in comments
- [ ] 13.2 Create visual examples/storybook stories
  - Add Storybook stories for keyboard components
  - Document different modes and states
  - Include screenshots for reference
- [ ] 13.3 Update CLAUDE.md or create mobile-specific documentation
  - Document mobile keyboard feature
  - Include architectural decisions
  - Add component organization diagram
- [ ] 13.4 Remove debug code and console.logs
  - Clean up any temporary debugging statements
  - Ensure production-ready code quality

## 14. Validation and Verification

- [ ] 14.1 Run `pnpm typecheck` to verify no TypeScript errors
- [ ] 14.2 Run `pnpm lint` to verify code style compliance
- [ ] 14.3 Run `pnpm test` to ensure all tests pass
- [ ] 14.4 Run `pnpm test:coverage` to verify test coverage
- [ ] 14.5 Manual testing on actual mobile devices or browser mobile emulators
  - Test on iOS Safari
  - Test on Android Chrome
  - Verify touch interactions work smoothly
  - Verify performance is acceptable (no lag on typing)
- [ ] 14.6 Verify no regressions in desktop UI
  - Test that desktop FormulaBar and editing still work
  - Test that FAB does not appear on desktop
  - Verify existing mobile features (scroll, selection) still work
