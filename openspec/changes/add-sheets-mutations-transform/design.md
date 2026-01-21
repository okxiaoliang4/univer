## Context
Sheets mutations are not covered by OT WASM transforms beyond insert/remove row/col and set-range-values. We need a consistent plan for handling conflicts when new transforms overlap existing ones.

## Goals / Non-Goals
- Goals:
  - Provide a concrete conflict matrix between Sheets mutations and existing transforms.
  - Define how overlapping transforms are merged without duplicating dispatch paths.
  - Outline how each Sheets mutation family maps to row/column shift semantics.
- Non-Goals:
  - Full transform implementations for all Sheets mutations.
  - Redesign of the transform dispatch architecture.

## Decisions
- Decision: Treat any existing transform as the single source of truth for an overlapping mutation id.
  - Alternatives considered: keep multiple transform structs per id (rejected due to ambiguous dispatch).
- Decision: Document pairwise conflicts for Sheets mutations that can interact with current transforms.
  - Alternatives considered: defer all conflict planning until transform implementation (rejected because spec requires explicit merge rules).

## Conflict Matrix Notes
- sheet.mutation.set-frozen vs sheet.mutation.set-range-values:
  - set-range-values does not affect freeze coordinates, so no transform is required.
  - Both mutations are preserved with identity behavior.
- sheet.mutation.set-frozen vs insert/remove row/col:
  - Insert/remove row/col shifts the frozen row/column start and split values.
  - When inserts occur before the frozen region, increment startRow/startColumn and keep split constant.
  - When removals overlap the frozen region, clamp split and adjust startRow/startColumn to remain within sheet bounds.

## Risks / Trade-offs
- Large mutation surface may lead to limited per-mutation coverage in the first iteration. Mitigate by prioritizing cross transforms against insert/remove and set-range-values.

## Migration Plan
- Update `tasks.md` to track conflict analysis and merge rules.
- Implement Sheets mutation transforms in phases while keeping each commit functional.

## Open Questions
- Should conflicts be recorded only for known positional transforms (insert/remove/set-range), or for all mutations that can shift ranges?
