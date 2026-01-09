## Context

Univer already has basic mobile support through `UniverSheetsMobileUIPlugin` with mobile-specific render controllers and selection services. However, the current input experience on mobile devices is poor because:

1. Native keyboard immediately obscures the spreadsheet view
2. No quick access to common spreadsheet operations (undo/redo, clipboard)
3. Difficult to enter formulas and functions on mobile
4. No specialized input modes for different data types (numbers, formulas, text)
5. Touch interactions are not optimized for mobile ergonomics

**Stakeholders**: Mobile users who need to edit spreadsheets on touch devices

**Constraints**:
- Must integrate with existing FormulaBar and formula engine services
- Must maintain isomorphic design (logic separate from UI)
- Must support both Android and iOS native keyboards
- Must not break existing desktop UI behavior

## Goals / Non-Goals

**Goals**:
- Provide a touch-optimized input interface for mobile spreadsheet editing
- Offer multiple specialized keyboards (formula, number, text) for different data entry scenarios
- Enable quick access to common operations without leaving the editing context
- Maintain visibility of the spreadsheet while editing
- Reuse existing FormulaBar and formula engine components

**Non-Goals**:
- Redesigning the desktop FormulaBar interface
- Adding new formula engine capabilities (reuse existing functions)
- Implementing collaborative editing features
- Adding data validation or conditional formatting UI

## Decisions

### 1. FAB (Floating Action Button) Pattern

**Decision**: Use a FAB that appears when a cell is focused, similar to the debugger FAB component

**Rationale**:
- Familiar mobile pattern (Google Material Design)
- Minimizes screen clutter when not editing
- Clear visual affordance for "start editing"
- Can be positioned to avoid overlapping with spreadsheet content

**Alternatives considered**:
- **Always-visible toolbar**: Would consume valuable screen space constantly
- **Double-tap to edit**: Less discoverable, harder to use
- **Context menu option**: Too many taps to start editing

### 2. Three-Mode Keyboard Architecture

**Decision**: Implement three distinct keyboard modes (Formula, Number, Text) with mode switcher buttons

**Rationale**:
- Each mode optimized for specific data type (reduces cognitive load)
- Formula keyboard: Quick access to operators, functions, cell references
- Number keyboard: Calculator-style layout familiar to users
- Text keyboard: Leverages native autocorrect and prediction

**Alternatives considered**:
- **Single universal keyboard**: Too complex, cluttered, hard to use on small screens
- **Only formula keyboard**: Insufficient for text-heavy data entry

### 3. FormulaBar Integration

**Decision**: Directly reuse the existing `FormulaBar.tsx` component with simplified mobile-specific layout

**Rationale**:
- **Zero duplication**: Copy the existing FormulaBar component instead of creating a wrapper
- **All features included**: Formula syntax highlighting, validation, editor bridge integration already working
- **Simple modifications**: Only need to remove unnecessary UI elements for mobile context
- **Proven code**: FormulaBar is already battle-tested in desktop environment

**Implementation approach**:

Create `MobileFormulaBar.tsx` that is a copy of `FormulaBar.tsx` with these modifications:

1. **Remove these UI elements** (not needed in mobile keyboard context):
   - `DefinedName` component (line 287-289) - mobile keyboard has its own cell reference display
   - `CloseIcon` button (line 310) - mobile keyboard has its own dismiss mechanism
   - `FxIcon` button (line 333) - mobile keyboard has f(x) mode button
   - `DropdownIcon` (line 395) - expand/collapse not needed in mobile keyboard

2. **Relocate CheckMarkIcon**:
   - Move from left button area (line 322) to the right side of the FormulaEditor input
   - Place it after the editor div (after line 386)
   - This provides a clear "Confirm" action at the right edge of the input field

3. **Layout adjustments**:
   - Remove the left button area container (line 291-336)
   - Remove the DefinedName container (line 287-289)
   - Simplify to just: FormulaEditor + CheckMarkIcon on the right

**Code structure**:
```tsx
// MobileFormulaBar.tsx - simplified version of FormulaBar.tsx
return (
    <div className="univer-flex univer-h-full univer-w-full">
        {/* FormulaEditor input area */}
        <div className="univer-flex univer-w-full univer-flex-1 univer-overflow-hidden univer-pl-3">
            <div ref={ref} className="univer-relative univer-flex-1">
                <FormulaEditor {...editorProps} />
            </div>
            {/* CheckMarkIcon moved to right side */}
            <div className="univer-flex univer-h-full univer-w-auto univer-cursor-pointer univer-items-center univer-justify-center">
                <span onClick={handleConfirmBtnClick}>
                    <CheckMarkIcon />
                </span>
            </div>
        </div>
    </div>
);
```

### 4. Function Browser Panel

**Decision**: Create a modal panel for browsing and selecting functions, triggered by f(x) button

**Rationale**:
- 200+ functions too many for direct keyboard access
- Categories help users discover relevant functions
- Search capability for power users
- Reuses existing FUNCTION_LIST data from formula engine

**Implementation details**:
- Panel slides up from bottom (bottom sheet pattern)
- Shows function name, category, and brief description
- Categories match existing function list organization (Financial, Date, Math, etc.)
- Inserting function places it in FormulaBar with cursor inside parentheses

### 5. Mobile Keyboard State Management

**Decision**: Reuse existing `EditorBridgeService` and `SetCellEditVisibleOperation` for edit state; create lightweight `IKeyboardService` only for mobile-specific state

**Rationale**:
- `EditorBridgeService` already manages cell editing state (`currentEditCellState$`, `visible$`)
- `SetCellEditVisibleOperation` is the established command for controlling editor visibility (used by FormulaBar, EditorContainer, etc.)
- Reusing existing patterns ensures consistency with desktop editing behavior
- Avoids duplicating edit state logic and prevents synchronization issues
- Mobile-specific service only manages keyboard UI state (mode, keyboard visibility), not cell editing state

**Integration with existing services**:
```typescript
// Use existing EditorBridgeService for edit state
editorBridgeService.currentEditCellState$  // Subscribe to know which cell is being edited
editorBridgeService.visible$               // Subscribe to know if editor is visible

// Edit operations use existing commands directly
commandService.executeCommand(SetCellEditVisibleOperation.id, {
    visible: true,
    eventType: DeviceInputEventType.PointerDown,
    unitId: workbook.getUnitId()
})

// Lightweight service for mobile-specific UI state only
interface IKeyboardService {
    // Mobile keyboard UI visibility (NOT the same as editor visibility)
    isKeyboardVisible$: Observable<boolean>
    showKeyboard(): void
    hideKeyboard(): void

    // Active keyboard mode
    keyboardMode$: Observable<'formula' | 'number' | 'text'>
    setMode(mode: 'formula' | 'number' | 'text'): void
}
```

**Implementation details**:
- Components call `commandService.executeCommand(SetCellEditVisibleOperation.id, ...)` directly to start/end editing
- No `beginEdit()` or `endEdit()` wrapper methods - unnecessary indirection
- Subscribe to `editorBridgeService.visible$` to show/hide mobile keyboard UI in sync with editor state
- Subscribe to `editorBridgeService.currentEditCellState$` to get the currently edited cell's value

### 6. Component Organization

**Decision**: Follow the established mobile/desktop separation pattern

**Structure**:
```
packages/sheets-ui/src/
├── views/mobile/keyboard/
│   ├── common/           # Shared keyboard components
│   │   ├── KeyboardContainer.tsx
│   │   ├── MobileFormulaBar.tsx    # Simplified copy of FormulaBar.tsx
│   │   ├── OperationToolbar.tsx    # Undo/Redo/Copy/Paste/Cut/Clear
│   │   └── ModeSwitcher.tsx        # Tab/f(x)/123/ABC/Confirm
│   ├── formula-keyboard/         # Formula-specific keys
│   │   ├── FormulaKeyboard.tsx
│   │   ├── OperatorKeys.tsx
│   │   └── FunctionBrowser.tsx
│   ├── number-keyboard/          # Number-specific keys
│   │   └── NumberKeyboard.tsx
│   └── text-keyboard/            # Text mode wrapper
│       └── TextKeyboard.tsx
├── views/mobile/fab/
│   └── KeyboardFab.tsx           # Floating action button
├── controllers/mobile/
│   └── keyboard.controller.ts  # Coordinates keyboard logic
└── services/
    └── keyboard.service.ts      # State management
```

### 7. Platform Detection

**Decision**: Use existing `UniverMobileUIPlugin` dependency to determine mobile context

**Rationale**:
- Already provides mobile-specific render controllers
- Avoids re-implementing device detection
- Consistent with existing mobile/desktop separation

**Implementation**: Mobile keyboard only registers when `UniverMobileUIPlugin` is loaded

### 8. Native Keyboard Integration

**Decision**: For Text mode, use a hidden input that triggers the native keyboard

**Rationale**:
- Leverages OS-level autocorrect, predictive text, and multilingual support
- Familiar experience for users
- Avoids reimplementing complex text input handling

**Implementation**: Create a hidden textarea that receives focus when Text mode is active, syncing its value with the FormulaBar

## Risks / Trade-offs

### Risk: Screen Real Estate on Small Devices

**Risk**: The keyboard interface occupies significant screen space, potentially obscuring spreadsheet content

**Mitigation**:
- Make keyboard semi-transparent or collapsible
- Ensure keyboard can be dismissed with a single gesture
- Optimize layout for smallest common mobile screen (320px width)
- Consider landscape/portrait orientation differences

### Risk: Performance with FormulaBar Re-rendering

**Risk**: Frequent FormulaBar updates during typing could cause performance issues

**Mitigation**:
- Use React.memo and useMemo strategically
- Ensure FormulaBar is already optimized for desktop use
- Test with large formulas and rapid typing

### Risk: Inconsistent Mobile/Desktop Behavior

**Risk**: Users switching between devices might find different interaction patterns confusing

**Mitigation**:
- Document mobile-specific behavior clearly
- Consider adding optional desktop FAB for accessibility
- Maintain core editing semantics (both support same operations)

### Risk: Function Browser Complexity

**Risk**: Function browser could become overwhelming with 200+ functions

**Mitigation**:
- Implement search as primary discovery mechanism
- Show "recently used" or "common" functions first
- Allow filtering by category
- Keep descriptions concise

### Trade-off: Complexity vs. Flexibility

**Decision**: Build all three keyboard modes from the start rather than starting with just one

**Rationale**: The requirements specify all three modes as interdependent (mode switcher is part of the design). Building incrementally would require more refactoring.

**Counter-argument**: Could start with just formula and number keyboards, add text keyboard later

**Rebuttal**: Text mode is critical for alphanumeric data entry; without it, users would have to dismiss the keyboard entirely, breaking the flow. All three modes provide a complete editing experience.

## Migration Plan

**No migration needed** - this is a net-new feature for mobile users that does not change existing desktop behavior or APIs.

**Rollback strategy**: If issues arise, the mobile keyboard controller can be disabled via configuration flag without affecting desktop functionality.

## Open Questions

1. **Cell type detection**: The requirements mention "determine which input mode to show based on cell.t type". Need to clarify:
   - What is `cell.t`? Is this the cell type/mtype property?
   - Should we auto-select keyboard mode based on existing cell value type?
   - Or should we remember the last-used mode for that cell?

2. **FAB positioning**: Where should the FAB appear? Options:
   - Bottom-right corner (like debugger FAB)
   - Bottom-center (easier to reach with thumbs)
   - Dynamic based on selection position

3. **Keyboard persistence**: When should the keyboard stay open vs. dismiss?
   - After submitting a cell (Enter/Confirm), move to next cell and keep keyboard open?
   - Or dismiss after each edit?
   - This affects the navigation flow (Tab vs Enter behavior)

4. **Function browser organization**: How should functions be grouped in the browser?
   - Use existing FUNCTION_LIST categories (Financial, Date, Math, etc.)?
   - Add "Most Used" category?
   - Show full function list or filter by typing only?

5. **Native keyboard behavior**: Should the native keyboard in Text mode:
   - Always be visible when Text mode is active?
   - Only appear when user taps the input area?
   - How do we handle the back button dismissing the keyboard but not the mobile keyboard UI?
