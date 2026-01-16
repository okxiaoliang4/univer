## Context
SnapshotService currently clones base snapshot content without applying `operation_log` mutations, leaving server-side snapshots inconsistent with applied edits. We need to apply a subset of sheet mutations directly to the JSON `IWorkbookData` snapshot to keep state accurate for collaborative sessions.

## Goals / Non-Goals
- Goals:
  - Apply `set-range-values`, `insert-row`, `insert-col`, `remove-rows`, and `remove-col` mutations to `IWorkbookData`.
  - Mirror TypeScript mutation handler semantics for cell merge, row/column insert/remove, and style/type updates.
  - Preserve unknown fields in snapshot JSON.
- Non-Goals:
  - Implement additional mutation types beyond the five listed.
  - Recompute formulas or trigger UI-side effects.
  - Introduce a full in-memory workbook model in Rust.

## Decisions
- Data representation: Use serde structs for workbook/worksheet fields that are required for mutation apply, with a `serde_json::Map` to preserve unknown fields.
- Mutation dispatch: Map mutation `id` to the corresponding apply handler; apply in the order stored in `operation_log`.
- SetRangeValues behavior:
  - Iterate `cellValue` as an object matrix (row/col keys as strings).
  - Null cell values delete entries; defined values merge into existing cell data.
  - Port `mergeCellData`, `updateCellProperty`, `getCellType`, and `getCellValue` semantics from TypeScript.
- Style handling: Implement a `Styles` helper in Rust to resolve style IDs, merge style objects, and deduplicate styles using deep equality and `generateRandomId(6)`-style IDs.
- Row/column operations:
  - Insert/remove entries in `rowData`/`columnData` using object-array insert/splice helpers.
  - Shift `cellData` rows/columns using object-matrix insert/remove helpers.
  - Update `rowCount`/`columnCount` using the mutation range length.
- Error handling: Invalid ranges or missing workbook/worksheet references return a structured error and abort snapshot application.

## Risks / Trade-offs
- Full parity with TypeScript style and type logic is complex; discrepancies could cause snapshot drift.
- Strict erroring on unsupported mutations may block snapshot updates if new mutation types are introduced without Rust support.

## Migration Plan
No data migration. Snapshots are recomputed using the new apply path; invalid mutations surface as errors rather than silently mutating content.

## Assumptions / Open Questions
- Assume workbook `id` matches mutation `unitId` and worksheets are keyed by `subUnitId`.
- Assume style ID generation can use 6-character alphanumeric IDs compatible with `generateRandomId(6)`.
- Default behavior is to error on unsupported mutation IDs during snapshot apply.
