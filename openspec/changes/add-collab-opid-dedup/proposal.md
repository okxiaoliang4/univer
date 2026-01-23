## Why
Rapid input can trigger concurrent changeset flushes that clear pending mutations out of order, causing lost operations. We need a durable, idempotent identifier per mutation and a serial send path to guarantee delivery and deduplication.

## What Changes
- Add a per-mutation `opId` generated via `generateRandomId(32)` and persist it end-to-end.
- Enforce server-side idempotency using `clientId + opId` as the unique key and return prior results on duplicates.
- Require per-document serial changeset flush while still allowing batch/compose.

## Impact
- Affected specs: collaboration-ot
- Affected code: collaboration client send path, server changeset handling, operation log schema
