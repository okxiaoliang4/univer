## 1. Proposal
- [x] 1.1 Review existing ot-wasm and collaboration OT specs (or create if missing)
- [x] 1.2 Confirm change-id scope and breaking changes with stakeholders

## 2. Rust OT API
- [x] 2.1 Replace noop mutation sentinel with optional mutation results in core and WASM APIs
- [x] 2.2 Update transform_list to skip None mutations without extra allocations
- [x] 2.3 Update compose/compose_list types to support optional results

## 3. JS OT Pipeline
- [x] 3.1 Update transform service bindings to handle optional mutations
- [x] 3.2 Remove noop filtering and adjust downstream execution logic
- [x] 3.3 Compose pending mutations before sync using wasm compose/compose_list

## 4. Types and Module Layout
- [x] 4.1 Move mutation params from types.rs into per-mutation modules
- [x] 4.2 Keep shared primitive types in types.rs
- [x] 4.3 Update imports and public exports

## 5. Tests & Validation
- [x] 5.1 Update Rust unit tests for optional transform results
- [x] 5.2 Add coverage for compose_list usage and filtering
- [x] 5.3 Validate no extra allocations in hot paths (documented)
