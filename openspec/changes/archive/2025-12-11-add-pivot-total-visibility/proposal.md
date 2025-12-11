## Why
Grand totals must be controlled via field configuration (first row/column field `showSubTotals`) instead of separate flags. Users also expect column grand totals to emit one total per value field (Google Sheets style).

## What Changes
- Treat the first row/column field `showSubTotals` as the grand total toggle; later fields keep subtotal behavior.
- Emit column grand totals per value field rather than a single shared total column.
- Provide UI checkboxes that map to the first row/column field `showSubTotals` (no separate total flags).

## Impact
- Affected specs: sheets-pivot-table, sheets-pivot-table-ui
- Affected code: pivot-table model, pivot-engine-v2, PivotTableEditor UI
