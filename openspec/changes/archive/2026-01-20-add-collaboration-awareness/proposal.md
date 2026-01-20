## Why
We need cursor/selection collaboration without relying on Yjs awareness so the OT WASM server can provide real-time awareness data to clients with minimal client changes.

## What Changes
- Introduce server-side awareness presence handling over Socket.IO in `@packages/univer-ot-wasm/src/server/`.
- Support in-memory awareness state with optional Redis-backed storage for online presence.
- Align payload shape with existing Yjs awareness client expectations to reduce client-side changes.

## Impact
- Affected specs: collaboration-awareness
- Affected code: packages/univer-ot-wasm/src/server/handlers/socketio.rs, services/, state.rs, types.rs
