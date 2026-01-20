## Context
This change replaces noop sentinels with optional mutation results, adds compose usage to reduce mutation counts, and reorganizes mutation params for clearer ownership. The OT pipeline runs in WASM and JS, so the design must minimize allocations and avoid extra conversions at the boundary.

## Goals / Non-Goals
- Goals: remove noop sentinels, support optional mutation outputs, reuse compose in JS to reduce payloads, and clarify type ownership.
- Non-Goals: changing core OT conflict resolution rules or introducing new mutation types.

## Decisions
- Decision: Represent no-op mutations as `Option<MutationInfo>` (Rust) and `MutationInfo | null` (JS) rather than sentinel IDs.
  - Alternatives considered: keep `__noop__`, or use an empty mutation list. Rejected due to API ambiguity and hidden semantics.
- Decision: Keep shared primitives (`Range`, `ObjectMatrixPrimitiveType`, `CellData`) in `types.rs` while moving mutation params to per-mutation modules.
  - Alternatives considered: leave types centralized. Rejected for modularity and maintainability.
- Decision: Compose pending mutations on JS side using WASM compose/compose_list before sending to server.
  - Alternatives considered: compose on server only. Rejected due to bandwidth and client memory pressure.

## Risks / Trade-offs
- Breaking API change for WASM/JS transform results; requires coordinated updates.
- Optional mutation handling adds branching in hot paths; avoid extra allocations by filtering in place.

## Migration Plan
1. Introduce optional result types and update Rust/JS bindings.
2. Update all transform and compose implementations to emit optional mutations.
3. Update JS pipeline and tests.
4. Remove noop filter usage.

## Open Questions
- Do we need a dedicated capability spec for OT WASM or reuse an existing OT spec once identified?
