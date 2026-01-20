## Why
The current OT pipeline relies on `__noop__` sentinels, scattered param types, and limited compose usage, which complicates JS/Rust interop and leads to unnecessary operations during offline reconciliation. We need a clearer contract that removes noop mutations, improves composition, and keeps performance and memory efficiency as first-class constraints.

## What Changes
- **BREAKING** Replace noop mutations with optional mutation results in the transform API (Rust + WASM + JS).
- **BREAKING** Update JS OT pipeline to handle optional mutations and remove noop filtering.
- Use Rust compose/compose_list on the JS side to merge compatible mutations before sync.
- Move mutation parameter types out of `types.rs` into per-mutation modules for clearer separation of shared vs business logic.
- Fix and expand Rust tests to validate the new optional transform contract and compose behavior.
- Define performance constraints for zero-copy boundaries and minimal allocations in the OT pipeline.

## Impact
- Affected specs: ot-wasm, collaboration-ot
- Affected code: packages/univer-ot-wasm/src/types.rs, packages/univer-ot-wasm/src/transform/*, packages/univer-ot-wasm/src/mutations/*, packages/collaboration/src/services/collaboration.service.ts, packages/collaboration/src/services/transform.service.ts
