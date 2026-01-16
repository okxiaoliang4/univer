## 1. Implementation
- [ ] 1.1 Add workbook snapshot data helpers (serde structs + JSON access) in `packages/univer-ot-wasm/src`
- [ ] 1.2 Implement set-range-values apply logic (cell merge, value/type normalization, style merge/dedupe)
- [ ] 1.3 Implement insert-row/insert-col apply logic (row/column data insert, counts, cell matrix shift)
- [ ] 1.4 Implement remove-rows/remove-col apply logic (row/column data remove, counts, cell matrix shift)
- [ ] 1.5 Wire snapshot application to parse `IWorkbookData` JSON and apply mutations by id
- [ ] 1.6 Add Rust unit tests for apply behavior and snapshot recomputation order

## 2. Validation
- [ ] 2.1 Run `pnpm --filter @univerjs/univer-ot-wasm test` (or `pnpm test` in `packages/univer-ot-wasm`)
