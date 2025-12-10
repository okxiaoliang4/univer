## Why
Pivot totals must match Google Sheets/Excel: row/column totals are controlled on rowFields/columnFields (first field = table grand total, deeper fields = in-group subtotals), labels like `<member> 总计` must appear, and column totals must emit one total column per value field. Current behavior diverges.

## What Changes
- Totals toggles live on rowFields/columnFields: first field controls the table grand total; deeper fields control subtotals inside their parent group (e.g., Region total, Quarter subtotals under Region, Channel subtotals under Quarter).
- Subtotal/grand-total labels use `<member> 总计`, and table-level totals use `总计`.
- Column totals expand per value field (n × valueFields) appended after each column group and at table level when the first column field total is on.
- UI exposes the same toggles inside row/column panels; no separate total flags.
- Develop via TDD with fixtures mirroring Google Sheets/Excel matrices and labels.

## Impact
- Affected specs: `sheets-pivot-table`, `sheets-pivot-table-ui`
- Affected code: `packages/sheets-pivot-table/src/models/pivot-engine-v2.ts`, render model/controller, pivot table editor UI, related tests
- Related changes: reconcile with `add-pivot-total-visibility` to avoid diverging grand-total semantics

