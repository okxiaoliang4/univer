# collaboration-awareness Specification

## Purpose
TBD - created by archiving change add-collaboration-awareness. Update Purpose after archive.
## Requirements
### Requirement: Server Awareness State
The server SHALL maintain per-document awareness state keyed by client ID and expose it through Socket.IO events for cursor/selection collaboration.

#### Scenario: Join returns current awareness
- **WHEN** a client joins a document via Socket.IO
- **THEN** the server responds with the current awareness state for that document

#### Scenario: Update broadcasts awareness changes
- **WHEN** a client publishes a cursor/selection update
- **THEN** the server stores the new awareness state and broadcasts the update to other clients in the document room

### Requirement: Optional Redis-backed Presence
The server SHALL support optional Redis storage for awareness state and online presence with TTL to survive process restarts when configured.

#### Scenario: Redis configured
- **WHEN** Redis is enabled in server configuration
- **THEN** awareness state is persisted in Redis and restored for late joiners

#### Scenario: Redis not configured
- **WHEN** Redis is disabled
- **THEN** awareness state is maintained in memory only

### Requirement: Client Compatibility
The server SHALL use awareness payloads compatible with the existing client service (`packages/collaboration/src/services/awareness.service.ts`) to minimize client changes.

#### Scenario: Yjs-aligned payload
- **WHEN** the server sends awareness updates
- **THEN** payload fields align with Yjs awareness expectations (clientID, user info, selection parameters)

