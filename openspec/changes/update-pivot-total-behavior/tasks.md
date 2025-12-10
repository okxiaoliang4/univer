## 1. Implementation
- [ ] 1.1 Confirm Google Sheets/Excel behaviors for row/column totals with multi-level fields and multiple value fields; capture fixtures for tests.
- [x] 1.2 Add TDD coverage in `packages/sheets-pivot-table/src/models/__tests__/pivot-engine.spec.ts` for nested row/column subtotals, labels (`<member> 总计`), and per-value-field column totals.
- [x] 1.3 Update `PivotEngineV2` subtotal/grand-total generation to honor per-field `showSubTotals` (first field = grand total), insert per-group subtotal rows/columns, and expand column totals per value field.
- [ ] 1.4 Ensure render model/controller expose subtotal/grand-total metadata needed for UI layout and grouping.
- [ ] 1.5 Update pivot table UI/editor to surface row/column total toggles inside rowFields/columnFields and reflect new totals structure; add UI-level tests if present.
- [ ] 1.6 Run regression/unit tests and `openspec validate update-pivot-total-behavior --strict`.

## 2. Validation
- [ ] 2.1 openspec validate update-pivot-total-behavior --strict

