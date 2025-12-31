## ADDED Requirements

### Requirement: Mobile Keyboard FAB Display

The system SHALL display a floating action button (FAB) with a keyboard icon when a user focuses on a cell in the mobile view.

#### Scenario: FAB appears on cell focus
- **GIVEN** the user is viewing a spreadsheet on a mobile device
- **WHEN** the user taps to focus on a cell
- **THEN** a FAB with a keyboard icon appears in the bottom-right corner of the screen
- **AND** the native keyboard does NOT automatically appear

#### Scenario: FAB position and appearance
- **GIVEN** the FAB is displayed
- **THEN** the FAB is positioned in the bottom-right corner
- **AND** the FAB is circular with a keyboard icon
- **AND** the FAB does not overlap with the spreadsheet content area
- **AND** the FAB has a shadow/elevation to indicate it is interactive

#### Scenario: FAB disappears when keyboard dismissed
- **GIVEN** the mobile keyboard is visible
- **WHEN** the user dismisses the keyboard (swipe down or back button)
- **THEN** the FAB remains visible
- **AND** the cell remains focused

### Requirement: Mobile Keyboard Activation and Dismissal

The system SHALL display a custom mobile keyboard interface when the FAB is tapped and shall provide a mechanism to dismiss it.

#### Scenario: Activate keyboard via FAB
- **GIVEN** a cell is focused and the FAB is visible
- **WHEN** the user taps the FAB
- **THEN** the custom mobile keyboard interface slides up from the bottom
- **AND** the FAB becomes hidden or fades out
- **AND** the keyboard's editing toolbar shows the current cell value in the FormulaBar

#### Scenario: Dismiss keyboard with swipe gesture
- **GIVEN** the mobile keyboard is visible
- **WHEN** the user swipes down on the keyboard
- **THEN** the keyboard slides down and becomes hidden
- **AND** the FAB reappears
- **AND** the cell remains focused

#### Scenario: Dismiss keyboard and save changes
- **GIVEN** the mobile keyboard is visible and the user has made edits
- **WHEN** the user taps the Confirm (✓) button
- **THEN** the keyboard is dismissed
- **AND** the cell value is saved
- **AND** the FAB reappears
- **AND** the cell remains focused

#### Scenario: Dismiss keyboard without saving
- **GIVEN** the mobile keyboard is visible and the user has made edits
- **WHEN** the user taps the Close (×) button or Back button
- **THEN** the keyboard is dismissed without saving changes
- **AND** the cell reverts to its original value
- **AND** the FAB reappears

### Requirement: Editing Toolbar Operations

The system SHALL provide an editing toolbar at the top of the mobile keyboard with common spreadsheet operations.

#### Scenario: Toolbar button layout
- **GIVEN** the mobile keyboard is visible
- **THEN** the editing toolbar is displayed at the top
- **AND** the toolbar contains six buttons from left to right: Undo, Redo, Copy, Paste, Cut, Clear
- **AND** the buttons are evenly spaced
- **AND** the buttons use standard icons (↶, ↷, 📋, 📋, ✂, 🗑)

#### Scenario: Undo operation
- **GIVEN** the mobile keyboard is visible
- **WHEN** the user taps the Undo button
- **THEN** the most recent edit operation is undone
- **AND** the FormulaBar and cell display reflect the undone state

#### Scenario: Redo operation
- **GIVEN** the mobile keyboard is visible and an operation has been undone
- **WHEN** the user taps the Redo button
- **THEN** the undone operation is reapplied
- **AND** the FormulaBar and cell display reflect the redone state

#### Scenario: Copy operation
- **GIVEN** the mobile keyboard is visible and a cell has content
- **WHEN** the user taps the Copy button
- **THEN** the current cell's content is copied to the clipboard
- **AND** a brief "Copied" confirmation is shown

#### Scenario: Paste operation
- **GIVEN** the mobile keyboard is visible and clipboard has content
- **WHEN** the user taps the Paste button
- **THEN** the clipboard content is pasted into the FormulaBar
- **AND** the user can edit the pasted content before confirming

#### Scenario: Cut operation
- **GIVEN** the mobile keyboard is visible and a cell has content
- **WHEN** the user taps the Cut button
- **THEN** the current cell's content is copied to the clipboard and removed from the cell
- **AND** the cell becomes empty

#### Scenario: Clear operation
- **GIVEN** the mobile keyboard is visible and a cell has content
- **WHEN** the user taps the Clear button
- **THEN** the cell content is deleted
- **AND** the FormulaBar becomes empty

#### Scenario: Button state based on availability
- **GIVEN** the mobile keyboard is visible
- **THEN** Undo and Redo buttons are disabled if no undo/redo history is available
- **AND** Paste button is disabled if clipboard is empty
- **AND** Copy, Cut, and Clear buttons are disabled if the cell is empty

### Requirement: Mobile FormulaBar Implementation

The system SHALL provide a mobile-specific FormulaBar component based on the existing FormulaBar with simplified layout.

#### Scenario: Mobile FormulaBar layout structure
- **GIVEN** the mobile keyboard is visible
- **THEN** the MobileFormulaBar is displayed between the editing toolbar and the mode switcher
- **AND** the MobileFormulaBar is a simplified version of the desktop FormulaBar
- **AND** it does NOT include: DefinedName dropdown, CloseIcon button, FxIcon button, expand/collapse DropdownIcon
- **AND** it includes: FormulaEditor input area with a Confirm (✓) button on the right edge

#### Scenario: Confirm button position and behavior
- **GIVEN** the MobileFormulaBar is displayed
- **THEN** the CheckMarkIcon (✓) button is positioned on the right side of the FormulaEditor input
- **AND** the Confirm button is always visible (not conditional on edit state)
- **AND** tapping the Confirm button saves the cell value and dismisses the keyboard

#### Scenario: FormulaBar features preserved
- **GIVEN** the MobileFormulaBar is based on FormulaBar.tsx
- **THEN** all formula editor features work identically to desktop:
  - Formula syntax highlighting
  - Formula validation
  - Cell reference selection
  - Auto-complete for function names
  - Integration with EditorBridgeService

#### Scenario: Edit cell value in MobileFormulaBar
- **GIVEN** the mobile keyboard is visible
- **WHEN** the user taps in the MobileFormulaBar and types text
- **THEN** the text appears in the FormulaEditor
- **AND** the cell preview updates in real-time
- **AND** the existing FormulaEditor handlers and behaviors are preserved

#### Scenario: Confirm and dismiss via MobileFormulaBar
- **GIVEN** the mobile keyboard is visible and the MobileFormulaBar has been edited
- **WHEN** the user taps the Confirm (✓) button on the right side
- **THEN** the edited value is saved to the cell via existing FormulaBar save logic
- **AND** the mobile keyboard is dismissed
- **AND** the FAB reappears

### Requirement: Keyboard Mode Switcher

The system SHALL provide a mode switcher bar with buttons to toggle between Formula, Number, and Text input modes.

#### Scenario: Mode switcher layout
- **GIVEN** the mobile keyboard is visible
- **THEN** the mode switcher is displayed below the FormulaBar
- **AND** the switcher contains five buttons from left to right: Tab, f(x), 123, ABC, Enter (↵)
- **AND** the currently active mode button is highlighted
- **AND** the buttons are evenly spaced

#### Scenario: Tab navigation
- **GIVEN** the mobile keyboard is visible and the user is editing a cell
- **WHEN** the user taps the Tab button
- **THEN** the current cell value is saved
- **AND** the selection moves to the cell immediately to the right
- **AND** the mobile keyboard remains open
- **AND** the FormulaBar shows the new cell's value

#### Scenario: Activate Formula mode
- **GIVEN** the mobile keyboard is visible in any mode
- **WHEN** the user taps the f(x) button
- **THEN** the keyboard switches to Formula mode
- **AND** the formula keyboard layout is displayed
- **AND** the f(x) button is highlighted as active

#### Scenario: Activate Number mode
- **GIVEN** the mobile keyboard is visible in any mode
- **WHEN** the user taps the 123 button
- **THEN** the keyboard switches to Number mode
- **AND** the number keyboard layout is displayed
- **AND** the 123 button is highlighted as active

#### Scenario: Activate Text mode
- **GIVEN** the mobile keyboard is visible in any mode
- **WHEN** the user taps the ABC button
- **THEN** the keyboard switches to Text mode
- **AND** the native system keyboard is displayed
- **AND** the ABC button is highlighted as active

#### Scenario: Confirm and move down
- **GIVEN** the mobile keyboard is visible and the user is editing a cell
- **WHEN** the user taps the Enter (↵) button
- **THEN** the current cell value is saved
- **AND** the selection moves to the cell immediately below
- **AND** the mobile keyboard remains open
- **AND** the FormulaBar shows the new cell's value

### Requirement: Formula Keyboard Layout

The system SHALL provide a formula keyboard with operator keys, function shortcuts, and symbol keys optimized for formula entry.

#### Scenario: Formula keyboard rows layout
- **GIVEN** Formula mode is active
- **THEN** the keyboard displays four rows of keys
- **AND** Row 1: Number keys 1-0 (10 keys)
- **AND** Row 2: Operators +, -, ×, ÷, blank keys, f(x), del (9 keys with spacing)
- **AND** Row 3: Operator +, parentheses ( and ), comma ,, blank keys, Σ, tab (9 keys with spacing)
- **AND** Row 4: Comparison operators <, >, colon :, period ., blank keys, quote "", ↵ (9 keys with spacing)
- **AND** Row 5: Currency symbols $, %, &, ^, blank keys, space, A1 (9 keys with spacing)

#### Scenario: Arithmetic operator keys
- **GIVEN** Formula mode is active
- **THEN** the keys +, -, ×, and ÷ are larger than number keys
- **AND** these keys are visually grouped and separated from function keys

#### Scenario: Insert SUM function via Σ key
- **GIVEN** Formula mode is active and the user is editing
- **WHEN** the user taps the Σ key
- **THEN** "=SUM()" is inserted into the FormulaBar
- **AND** the cursor is positioned between the parentheses

#### Scenario: Insert function via f(x) key
- **GIVEN** Formula mode is active and the user is editing
- **WHEN** the user taps the f(x) key
- **THEN** the function browser panel slides up from the bottom
- **AND** the keyboard remains visible behind the panel

#### Scenario: Insert quotes via "" key
- **GIVEN** Formula mode is active and the user is editing
- **WHEN** the user taps the "" key
- **THEN** two double quotes ("") are inserted into the FormulaBar
- **AND** the cursor is positioned between the quotes

#### Scenario: Move to next cell via ↵ key
- **GIVEN** Formula mode is active and the user is editing
- **WHEN** the user taps the ↵ key in the keyboard (not the mode switcher)
- **THEN** the current cell value is saved
- **AND** the selection moves to the cell in the next row
- **AND** the mobile keyboard remains open

#### Scenario: Insert space
- **GIVEN** Formula mode is active and the user is editing
- **WHEN** the user taps the space key
- **THEN** a space character is inserted at the cursor position in the FormulaBar

#### Scenario: Switch to English keyboard via A1 key
- **GIVEN** Formula mode is active
- **WHEN** the user taps the A1 key
- **THEN** the keyboard layout switches to English QWERTY layout
- **AND** the English layout displays:
  - Row 1: q w e r t y u i o p
  - Row 2: a s d f g h j k l
  - Row 3: ⇧ z x c v b n m del
  - Row 4: $ : , ! space ↵ back
- **AND** a "back" button is shown to return to formula keyboard

#### Scenario: Delete character via del key
- **GIVEN** Formula mode is active and the user is editing
- **WHEN** the user taps the del key
- **THEN** the character before the cursor is deleted from the FormulaBar

### Requirement: Number Keyboard Layout

The system SHALL provide a number keyboard with calculator-style layout optimized for numeric entry.

#### Scenario: Number keyboard rows layout
- **GIVEN** Number mode is active
- **THEN** the keyboard displays four rows of keys
- **AND** Row 1: Five blank keys (for spacing)
- **AND** Row 2: +/-, 7, 8, 9, del (5 keys)
- **AND** Row 3: %, 4, 5, 6, tab (5 keys)
- **AND** Row 4: $, 1, 2, 3, ↵ (5 keys)
- **AND** Row 5: ¥, 00, 0, ., ↵ (5 keys)

#### Scenario: Enter number keys
- **GIVEN** Number mode is active and the user is editing
- **WHEN** the user taps any number key (0-9)
- **THEN** that number is inserted at the cursor position in the FormulaBar

#### Scenario: Toggle positive/negative via +/- key
- **GIVEN** Number mode is active and the user is editing a numeric value
- **WHEN** the user taps the +/- key
- **THEN** a negative sign (-) is inserted or removed at the beginning of the number

#### Scenario: Insert percentage via % key
- **GIVEN** Number mode is active and the user is editing
- **WHEN** the user taps the % key
- **THEN** a percent sign (%) is inserted at the cursor position

#### Scenario: Insert currency symbols
- **GIVEN** Number mode is active and the user is editing
- **WHEN** the user taps the $ key
- **THEN** a dollar sign ($) is inserted at the cursor position
- **WHEN** the user taps the ¥ key
- **THEN** a yen sign (¥) is inserted at the cursor position

#### Scenario: Insert double zero via 00 key
- **GIVEN** Number mode is active and the user is editing
- **WHEN** the user taps the 00 key
- **THEN** two zeros (00) are inserted at the cursor position

#### Scenario: Insert decimal point via . key
- **GIVEN** Number mode is active and the user is editing
- **WHEN** the user taps the . key
- **THEN** a decimal point (.) is inserted at the cursor position

### Requirement: Text Keyboard Native Integration

The system SHALL provide a text mode that uses the native system keyboard for text entry.

#### Scenario: Activate native keyboard in Text mode
- **GIVEN** Text mode is activated via the ABC button
- **THEN** the native system keyboard slides up from the bottom
- **AND** a hidden input field receives focus to trigger the native keyboard
- **AND** text entered in the native keyboard appears in the FormulaBar

#### Scenario: Hide native keyboard components
- **GIVEN** Text mode is active
- **THEN** the custom keyboard UI shows only the editing toolbar, FormulaBar, and mode switcher
- **AND** the area below the mode switcher is transparent or shows the native keyboard
- **AND** no custom keyboard keys are displayed

#### Scenario: Sync native keyboard input with FormulaBar
- **GIVEN** Text mode is active and the native keyboard is visible
- **WHEN** the user types using the native keyboard
- **THEN** the typed text appears in the FormulaBar in real-time
- **AND** the cell preview updates accordingly

### Requirement: Function Browser Panel

The system SHALL provide a function browser panel that displays all available formula functions organized by category.

#### Scenario: Open function browser
- **GIVEN** Formula mode is active
- **WHEN** the user taps the f(x) key in the formula keyboard
- **THEN** the function browser panel slides up as a bottom sheet
- **AND** the panel covers the bottom portion of the screen (70-80% height)
- **AND** the keyboard remains visible behind the panel
- **AND** a dimmed backdrop is shown behind the panel

#### Scenario: Function browser layout
- **GIVEN** the function browser panel is open
- **THEN** the panel displays a search input at the top
- **AND** below the search input, category tabs are shown (e.g., All, Financial, Date, Math, etc.)
- **AND** below the category tabs, a scrollable list of functions is displayed
- **AND** each function item shows the function name and a brief description

#### Scenario: Search for function
- **GIVEN** the function browser panel is open
- **WHEN** the user types in the search input
- **THEN** the function list filters to show only matching functions
- **AND** the search matches function names and descriptions

#### Scenario: Filter by category
- **GIVEN** the function browser panel is open
- **WHEN** the user taps a category tab
- **THEN** the function list filters to show only functions in that category
- **AND** the selected category tab is highlighted

#### Scenario: Insert function
- **GIVEN** the function browser panel is open
- **WHEN** the user taps on a function item (e.g., SUM)
- **THEN** the function browser panel is dismissed
- **AND** the function is inserted into the FormulaBar (e.g., "=SUM()")
- **AND** the cursor is positioned between the parentheses
- **AND** the focus returns to the FormulaBar for editing

#### Scenario: Close function browser without selection
- **GIVEN** the function browser panel is open
- **WHEN** the user taps the backdrop or swipes down on the panel
- **THEN** the function browser panel is dismissed
- **AND** no function is inserted
- **AND** the FormulaBar retains its previous content

#### Scenario: Function data source
- **GIVEN** the function browser is displaying functions
- **THEN** the functions are loaded from the existing FUNCTION_LIST in the formula engine
- **AND** the categories match the existing function list organization (Financial, Date & Time, Math, Statistical, Lookup, Database, Text, Logical, Information, Engineering, Cube, Compatibility, Web, Array, Univer)

### Requirement: Cell Type-Based Keyboard Mode Selection

The system SHALL automatically select an appropriate keyboard mode based on the type of the focused cell.

#### Scenario: Auto-select Formula mode for formula cells
- **GIVEN** a cell is activated that contains a formula (starts with =)
- **WHEN** the user taps the FAB to open the keyboard
- **THEN** Formula mode is automatically activated

#### Scenario: Auto-select Number mode for numeric cells
- **GIVEN** a cell is activated that contains a number
- **WHEN** the user taps the FAB to open the keyboard
- **THEN** Number mode is automatically activated

#### Scenario: Auto-select Text mode for text cells
- **GIVEN** a cell is activated that contains text
- **WHEN** the user taps the FAB to open the keyboard
- **THEN** Text mode is automatically activated

#### Scenario: Default mode for empty cells
- **GIVEN** an empty cell is activated
- **WHEN** the user taps the FAB to open the keyboard
- **THEN** the last-used keyboard mode is activated
- **OR** if no previous mode, Number mode is activated as the default

### Requirement: Mobile Keyboard State Management

The system SHALL provide a centralized service for managing mobile keyboard UI state, separate from cell editing state which is managed by existing services.

#### Scenario: Track keyboard visibility state
- **GIVEN** the mobile keyboard service is initialized
- **THEN** the service provides an observable `isKeyboardVisible$`
- **AND** when the keyboard UI is shown, the observable emits `true`
- **AND** when the keyboard UI is dismissed, the observable emits `false`
- **AND** this is separate from editor visibility managed by `EditorBridgeService.visible$`

#### Scenario: Track keyboard mode state
- **GIVEN** the mobile keyboard service is initialized
- **THEN** the service provides an observable `keyboardMode$`
- **AND** the observable emits the current mode ('formula' | 'number' | 'text')
- **AND** when the user switches modes, the observable emits the new mode

#### Scenario: Subscribe to existing editor state
- **GIVEN** the mobile keyboard service is initialized
- **THEN** the service subscribes to `EditorBridgeService.visible$` to know when editing is active
- **AND** the service subscribes to `EditorBridgeService.currentEditCellState$` to get the current cell being edited
- **AND** no `isEditing$` observable is provided in `MobileKeyboardService` (use `EditorBridgeService.visible$` instead)

#### Scenario: Show and hide keyboard UI
- **GIVEN** the mobile keyboard is being controlled
- **WHEN** `showKeyboard()` is called
- **THEN** `isKeyboardVisible$` emits `true`
- **AND** when `hideKeyboard()` is called
- **THEN** `isKeyboardVisible$` emits `false`
- **AND** these methods only control keyboard UI visibility, not editing state

#### Scenario: Edit operations use existing commands
- **GIVEN** a cell is focused and the FAB is tapped
- **WHEN** edit should begin
- **THEN** the component calls `commandService.executeCommand(SetCellEditVisibleOperation.id, { visible: true, ... })` directly
- **AND** no `beginEdit()` method exists on `MobileKeyboardService`
- **AND** when editing should end, the component calls `SetCellEditVisibleOperation` with `visible: false` directly
- **AND** no `endEdit()` method exists on `MobileKeyboardService`

### Requirement: Mobile Keyboard Component Organization

The system SHALL organize mobile keyboard components following the established mobile/desktop separation pattern.

#### Scenario: Component folder structure
- **GIVEN** the mobile keyboard feature is implemented
- **THEN** components are organized under `packages/sheets-ui/src/views/mobile/keyboard/`
- **AND** shared components are in `common/` subdirectory
- **AND** mode-specific components are in `formula-keyboard/`, `number-keyboard/`, and `text-keyboard/` subdirectories
- **AND** the FAB component is in `views/mobile/fab/` subdirectory

#### Scenario: Common keyboard components
- **GIVEN** the keyboard common directory exists
- **THEN** it contains `KeyboardContainer.tsx` for the main keyboard wrapper
- **AND** it contains `MobileFormulaBar.tsx` for the simplified FormulaBar component
- **AND** it contains `OperationToolbar.tsx` for the undo/redo/copy/paste/cut/clear buttons
- **AND** it contains `ModeSwitcher.tsx` for the Tab/f(x)/123/ABC/↵ buttons

#### Scenario: Formula keyboard components
- **GIVEN** the formula-keyboard directory exists
- **THEN** it contains `FormulaKeyboard.tsx` for the main formula keyboard layout
- **AND** it contains `OperatorKeys.tsx` for the arithmetic and comparison operator keys
- **AND** it contains `FunctionBrowser.tsx` for the function selection panel

#### Scenario: Number keyboard components
- **GIVEN** the number-keyboard directory exists
- **THEN** it contains `NumberKeyboard.tsx` for the calculator-style number keyboard

#### Scenario: Text keyboard components
- **GIVEN** the text-keyboard directory exists
- **THEN** it contains `TextKeyboard.tsx` for the native keyboard integration wrapper

#### Scenario: FAB component
- **GIVEN** the fab directory exists
- **THEN** it contains `KeyboardFab.tsx` for the floating action button

#### Scenario: Controller and service
- **GIVEN** the mobile keyboard feature is implemented
- **THEN** `packages/sheets-ui/src/controllers/mobile/mobile-keyboard.controller.ts` exists for coordinating keyboard logic
- **AND** `packages/sheets-ui/src/services/mobile/mobile-keyboard.service.ts` exists for state management

### Requirement: Integration with Existing Mobile Plugin

The system SHALL integrate the mobile keyboard feature into the existing `UniverSheetsMobileUIPlugin`.

#### Scenario: Register mobile keyboard service
- **GIVEN** the `UniverSheetsMobileUIPlugin` is initialized
- **THEN** the `IMobileKeyboardService` is registered as a dependency
- **AND** the service is available for injection by other components

#### Scenario: Register mobile keyboard controller
- **GIVEN** the `UniverSheetsMobileUIPlugin` is initialized
- **THEN** the `MobileKeyboardController` is registered as a controller
- **AND** the controller is initialized during the plugin lifecycle

#### Scenario: Register keyboard components
- **GIVEN** the `UniverSheetsMobileUIPlugin` is initialized
- **THEN** the FAB component is registered with the ComponentManager
- **AND** the keyboard container component is registered with the ComponentManager
- **AND** the components are available for rendering in the mobile UI

#### Scenario: Only activate on mobile
- **GIVEN** the `UniverSheetsMobileUIPlugin` depends on `UniverMobileUIPlugin`
- **THEN** the mobile keyboard components only render when the mobile UI plugin is active
- **AND** the FAB and keyboard are not displayed on desktop
