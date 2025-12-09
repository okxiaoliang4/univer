## 1. Implementation
- [ ] 1.1 Confirm default sort direction and null/empty handling for field/value sorts with stakeholders.
- [ ] 1.2 Extend pivot config to carry row/column sort settings (field-based and value-based) with sensible defaults.
- [ ] 1.3 Update `PivotEngineV2` matrix calculation to sort row/column headers and corresponding value matrices by field content when no explicit sort is provided.
- [ ] 1.4 Add support to sort rows/columns by a chosen value field aggregate (any value field), keeping deterministic tie-breakers.
- [ ] 1.5 Add/adjust TDD coverage in `packages/sheets-pivot-table/src/models/__tests__/pivot-engine.spec.ts` to validate new sorting behaviors (row/column fields and value-field sort cases).
- [ ] 1.6 Run unit tests and ensure downstream render models consume the sorted matrix without regressions.

