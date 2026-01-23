## 1. Implementation
- [ ] 1.1 Add opId to mutation types and client generation (generateRandomId(32))
- [ ] 1.2 Persist opId in pending queue and include in changeset payload
- [ ] 1.3 Add server-side dedup logic for (clientId, opId) with idempotent response
- [ ] 1.4 Add DB migration for opId/clientId storage and unique index
- [ ] 1.5 Implement per-document serial flush (in-flight gate)
- [ ] 1.6 Update unit tests and add coverage for dedup/serial flush
