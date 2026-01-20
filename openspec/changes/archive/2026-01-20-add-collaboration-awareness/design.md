## Context
The current collaboration system uses Yjs awareness on the client side. The OT WASM server only forwards a basic `presence_update` event without tracking awareness state, which makes cursor/selection syncing unreliable when clients reconnect or join late.

## Goals / Non-Goals
- Goals:
  - Provide a server-side awareness state registry keyed by document ID and client ID.
  - Support an in-memory mode with optional Redis persistence for online presence.
  - Keep client-side changes minimal by aligning payload shape with Yjs awareness semantics.
- Non-Goals:
  - Full Yjs document syncing on the server.
  - Long-term persistence of awareness state across server restarts when Redis is disabled.

## Decisions
- Decision: Add a dedicated awareness service to manage presence state and broadcast updates via Socket.IO.
- Alternatives considered: Keep stateless broadcast-only presence (insufficient for late joiners).
- Decision: Redis is optional; when configured, store per-doc awareness state and online users with TTL.

## Risks / Trade-offs
- In-memory mode loses awareness on server restart.
- Redis introduces operational dependency and needs TTL hygiene to avoid stale sessions.

## Migration Plan
1. Add awareness state service and types.
2. Update socket.io handlers to use awareness service for join/leave/update events.
3. Expose minimal client-side configuration to align with existing awareness payloads.

## Open Questions
- What TTL should Redis use for awareness entries?
- How should client identity map to socket IDs vs authenticated user IDs?
