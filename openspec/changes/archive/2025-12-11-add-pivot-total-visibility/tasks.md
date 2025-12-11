## 1. Implementation
- [ ] 1.1 Align pivot config so first row/column field `showSubTotals` controls grand totals; later fields remain subtotals
- [ ] 1.2 Update PivotEngineV2 to generate grand totals based on first-field `showSubTotals` and add per-value grand total columns
- [ ] 1.3 Render model/controller updates to expose totals state (first-field subtotals as totals) to UI
- [ ] 1.4 Add PivotTableEditor checkboxes that map to first row/column field `showSubTotals` and sync state
- [ ] 1.5 Tests/validation for grand total semantics and per-value total columns in engine and UI behaviors

## 2. Validation
- [ ] 2.1 openspec validate add-pivot-total-visibility --strict
