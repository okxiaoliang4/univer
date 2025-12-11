## Why
Pivot table behavior was refactored to use a PivotModel-based `PivotEngine` (formerly PivotEngineV2), removing value position switching and the legacy render model while fixing subtotal/grand total duplication, styling, and sheet-capacity issues. The existing specs still describe the old engine, valuePosition modes, and render model, so they need to be updated to reflect the current implementation.

## What Changes
- Update the pivot engine requirement to describe the PivotModel output, subtotal/grand-total rules, and column-oriented multi-value layout.
- Remove valuePosition row-mode support and the legacy `PivotTableRenderModel`, replacing them with engine helper requirements for rendering/styling.
- Refresh the style service requirement to capture level-based header graduation and the darker grand-total color.
- Add a requirement for ensuring pivot output ranges expand the worksheet (rows/cols) locally when needed.

## Impact
- Affected specs: `specs/sheets-pivot-table/spec.md`
- Affected code: `packages/sheets-pivot-table` (pivot-engine.ts, pivot-table.ts, pivot-table-range.service.ts), `packages/sheets-pivot-table-ui` (pivot-table-style.service.ts), related render/formula controllers

