## Why
Snapshot computation in `univer-ot-wasm` currently replays operations without mutating workbook content, which produces stale or incorrect snapshot data for collaborative sessions.

## What Changes
- Implement mutation apply logic for set-range-values, insert-row, insert-col, remove-rows, and remove-col against `IWorkbookData` JSON content.
- Add workbook snapshot helpers (object-matrix operations, style merging, type normalization) to mirror TypeScript mutation handlers.
- Update snapshot computation to apply operations sequentially and surface errors for invalid mutations.

## Impact
- Affected specs: `apply-workbook-mutations` (new)
- Affected code: `packages/univer-ot-wasm/src/server/services/snapshot.rs`, `packages/univer-ot-wasm/src/mutations/*.rs`, new helpers in `packages/univer-ot-wasm/src`
- Affected tests: new Rust unit tests for mutation apply and snapshot recomputation
