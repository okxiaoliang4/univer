## Why

The current mobile experience for Univer Sheets has poor usability - focusing on a cell immediately triggers the native keyboard, which obscures the spreadsheet and provides an awkward input experience. Mobile users need a specialized input interface optimized for touch interactions that provides quick access to common operations (undo/redo, clipboard, editing) and multiple input modes (formulas, numbers, text) without losing context of their spreadsheet.

## What Changes

- Add a floating action button (FAB) with keyboard icon that appears when a cell is focused, replacing the immediate native keyboard popup
- Create a custom mobile keyboard overlay with three input modes:
  1. **Formula keyboard**: Function keys (arithmetic, parentheses, common symbols), function browser panel, SUM shortcut, quote insertion, cell reference input
  2. **Number keyboard**: Calculator-style layout with arithmetic operators, percentage, currency symbols, decimal point, and special editing keys
  3. **Text keyboard**: Native system keyboard for alphanumeric input
- Add an editing toolbar with Undo, Redo, Copy, Paste, Cut, and Clear operations above the FormulaBar
- Implement quick navigation buttons (Tab for right cell move, Enter for down cell move, Confirm to submit)
- Integrate the existing FormulaBar component for editing with a confirm button
- Create a function browser panel displaying all available formulas from the formula engine with search and category filtering

## Impact

- Affected specs: New capability `sheets-ui-mobile`
- Affected code:
  - `packages/sheets-ui/src/controllers/mobile/` - Add mobile keyboard controller
  - `packages/sheets-ui/src/views/mobile/` - Add keyboard components and FAB
  - `packages/sheets-ui/src/services/` - Add mobile keyboard state management service
  - `packages/sheets-ui/src/mobile-plugin.ts` - Register new mobile keyboard components
  - Existing FormulaBar component will be reused in mobile keyboard layout
  - Cell focus behavior in mobile context menu/render controller
