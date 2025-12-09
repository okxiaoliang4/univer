## Why
Pivot table users cannot toggle row/column groups directly from rendered cells. We need an in-cell expand/collapse control that leverages the graphics renderer so interactions stay in sync with pivot recalculation and rendering.

## What Changes
- Register a graphics renderer in the pivot table UI to draw an expand/collapse button inside eligible pivot output cells.
- Handle pointer interaction to toggle pivot group state, trigger recalculation, and re-render the updated pivot output.
- Define the integration points between pivot UI renderers and the graphics extension, including printing/selection considerations.

## Impact
- Affected specs: sheets-pivot-table-ui
- Affected code: pivot table UI render controller/model, graphics renderer registration (`packages/sheets-graphics`, `packages/engine-render`), pivot group toggle flow
