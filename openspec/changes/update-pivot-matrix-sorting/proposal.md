## Why
Pivot table matrix output currently follows data iteration order, causing row and column headers (and their value matrices) to appear unsorted. This produces unexpected Quarter ordering and makes value-based sorting impossible.

## What Changes
- Add deterministic sorting for row and column outputs that follows field values (e.g., Quarter sorted as Q1, Q2, Q3, Q4) in `PivotEngineV2`.
- Enable optional sorting by a selected value field aggregate so users can order rows/columns by metrics such as Sales; support any value field as the sort key.
- Drive the change with new TDD coverage in `pivot-engine.spec.ts` and update the matrix algorithm to satisfy the tests.

## Impact
- Affected specs: `sheets-pivot-table`
- Affected code: `packages/sheets-pivot-table/src/models/pivot-engine-v2.ts`, `packages/sheets-pivot-table/src/models/__tests__/pivot-engine.spec.ts`, downstream render models depending on matrix ordering

