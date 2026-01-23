## Context
Rapid local edits can trigger overlapping changeset flushes; the current ack handling clears pending mutations globally, which can drop operations. Adding per-mutation opIds enables idempotent deduplication and safe retries, while serializing flushes prevents concurrent clears.

## Goals / Non-Goals
- Goals:
  - Assign a unique opId to each mutation and carry it through client, server, and persistence.
  - Server-side idempotency using (clientId, opId) as the dedup key.
  - Serialize changeset flushing per document while preserving batch/compose.
- Non-Goals:
  - Changing OT transform logic or mutation semantics beyond opId metadata.
  - Introducing new transport protocols or replacing Socket.IO.

## Decisions
- Add `opId` to mutation structures (client/server) and store in operation log entries.
- Extend server apply pipeline to detect duplicates by `(clientId, opId)` and return the prior result.
- Add per-document in-flight gating on the client to ensure serial flush.

## Risks / Trade-offs
- Additional storage and index cost for opId and clientId.
- Client must ensure opId uniqueness; collisions are possible but extremely unlikely.

## Migration Plan
1. Add opId field to mutation types and wire generation on the client.
2. Add DB columns and unique index for (doc_id, client_id, op_id) or (client_id, op_id) scoped per doc.
3. Update server apply path to dedup and return prior result.
4. Implement serial flush per document on client.
5. Update tests and run validation.

## Open Questions
- Should dedup scope include `doc_id` (recommended) or rely only on `clientId`?
