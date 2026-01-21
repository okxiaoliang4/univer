## Why
Sheets mutations lack transform coverage in OT WASM, which blocks consistent concurrent editing for the full Sheets mutation surface.

## What Changes
- Implement OT WASM transforms for all Sheets mutations.
- Add cross-transform logic against existing insert/remove row/col and set-range-values transforms.
- Expand tests to cover new Sheets transforms and their conflicts.

## Impact
- Affected specs: ot-wasm
- Affected code: packages/univer-ot-wasm/src/transform/, packages/univer-ot-wasm/src/mutations/
