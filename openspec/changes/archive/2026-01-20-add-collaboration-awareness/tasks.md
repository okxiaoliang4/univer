## 1. Implementation
- [x] 1.1 Add awareness state service (in-memory + Redis optional) in `packages/univer-ot-wasm/src/server/services/`
- [x] 1.2 Extend socket.io handlers to join/leave awareness channels and broadcast presence updates
- [x] 1.3 Define awareness request/response types for cursor/selection payloads in `types.rs`
- [x] 1.4 Wire awareness service into `ServerState` and config
- [x] 1.5 Add unit tests for awareness state storage and eviction behavior
