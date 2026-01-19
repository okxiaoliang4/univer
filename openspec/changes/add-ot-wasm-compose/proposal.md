## Why
Add compose support in univer-ot-wasm so clients can merge sequential operations locally before sync, reducing OT payload size.

## What Changes
- Add compose and compose_list APIs in the OT wasm transform service for JS usage.
- Implement compose behavior for each existing mutation type (same type, same sheet).
- Add unit tests to cover compose behavior and non-composable cases.

## Impact
- Affected specs: ot-wasm
- Affected code: packages/univer-ot-wasm/src/transform/*, packages/univer-ot-wasm/src/types.rs
