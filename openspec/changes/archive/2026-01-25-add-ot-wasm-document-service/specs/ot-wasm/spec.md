## ADDED Requirements
### Requirement: Document CRUD via DocumentService
The OT WASM server SHALL expose document creation, cloning, deletion, and retrieval through a shared DocumentService used by gRPC and REST controllers.

#### Scenario: Create a new document
- **WHEN** a NewDocument request is received with doc_id, creator_id, name, and initial content or URL
- **THEN** the service creates a document record, stores the initial snapshot, and returns success

#### Scenario: Clone an existing document
- **WHEN** a CloneDocument request is received with doc_id and optional snapshot_id
- **THEN** the service copies the target snapshot into a new document and returns the new doc_id and size

#### Scenario: Delete a document
- **WHEN** a DeleteDocument request is received with doc_id
- **THEN** the service deletes or soft-deletes document records and associated snapshots based on request flags

#### Scenario: Get document snapshot
- **WHEN** a GetDocumentsnapshot request is received with doc_id and optional snapshot_id
- **THEN** the service returns snapshot metadata with signed storage URL when needed

### Requirement: Snapshot list and metadata update
The OT WASM server SHALL provide snapshot listing with pagination and allow snapshot name updates.

#### Scenario: List snapshots
- **WHEN** a GetDocSnapshotList request is received with doc_id and pagination parameters
- **THEN** the service returns a paginated list of snapshots and the next cursor

#### Scenario: Update snapshot name
- **WHEN** an UpdateDocSnapshotName request is received
- **THEN** the snapshot name is updated and reflected in subsequent reads

### Requirement: Rollback support
The OT WASM server SHALL support document restoration from a snapshot_id or a revision target.

#### Scenario: Restore by snapshot_id
- **WHEN** a RestoreDocument request is received with snapshot_id
- **THEN** the service restores document state based on that snapshot

#### Scenario: Restore by revision
- **WHEN** a RestoreDocument request is received with a revision target
- **THEN** the service resolves the latest snapshot at or before the revision and restores it

### Requirement: Latest snapshot lookup
The OT WASM server SHALL provide latest snapshot lookups for multiple documents.

#### Scenario: Batch latest snapshots
- **WHEN** a GetDocLatestsnapshots request is received with multiple doc_ids
- **THEN** the service returns a map of doc_id to latest snapshot metadata

### Requirement: Document preparation and storage helpers
The OT WASM server SHALL support document preparation and storage URL helpers via the Editable gRPC service.

#### Scenario: Prepare document
- **WHEN** a PrepareDocument request is received
- **THEN** the service ensures document metadata is initialized for collaborative editing

#### Scenario: Sign object URLs
- **WHEN** a SignObjectURL request is received with storage_ids
- **THEN** the service returns signed URLs for each storage_id

#### Scenario: Resolve doc_id by storage_id
- **WHEN** a GetDocIdFromStorageId request is received
- **THEN** the service returns the owning doc_id

### Requirement: Document and snapshot fields
The OT WASM data model SHALL include document and snapshot metadata aligned to `ot_rpc.proto`.

#### Scenario: Document metadata fields
- **WHEN** a document is created
- **THEN** creator_id, doc_type, create_type, name, and timestamps are stored

#### Scenario: Snapshot metadata fields
- **WHEN** a snapshot is created
- **THEN** name, size, users, storage_id, restore_from_id, and timestamps are stored
