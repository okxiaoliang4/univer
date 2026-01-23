## ADDED Requirements

### Requirement: etcd Service Registration
The OT server SHALL register itself to etcd on startup with a lease-based key-value pair.

#### Scenario: Service registration on startup
- **WHEN** the OT server starts successfully
- **THEN** it SHALL create an etcd lease and register key `ot-collaboration/{UUID}` with value `{IP}:{PORT}`

#### Scenario: Lease keepalive
- **WHEN** the service is running
- **THEN** it SHALL maintain the etcd lease through periodic keepalive requests

#### Scenario: Service deregistration on shutdown
- **WHEN** the OT server shuts down gracefully
- **THEN** it SHALL revoke the etcd lease, causing automatic key deletion

### Requirement: gRPC Service Discovery
The OT server SHALL discover available services from etcd and create gRPC clients dynamically.

#### Scenario: Get gRPC client by service name
- **WHEN** calling `get_grpc_client(service_name)`
- **THEN** it SHALL query etcd for keys with prefix `{service_name}/` and create a gRPC client to one of the returned endpoints

#### Scenario: Load balancing
- **WHEN** multiple service instances are registered
- **THEN** the client SHALL select one using random or round-robin strategy

### Requirement: gRPC Snapshot Queue Client
The OT server SHALL communicate with Node BullMQ service via gRPC to enqueue snapshot jobs.

#### Scenario: Enqueue snapshot job
- **WHEN** a document has pending operations ready for snapshot
- **THEN** the server SHALL call `EnqueueSnapshotJob` RPC with `doc_id`, `target_rev`, and `timestamp`

#### Scenario: gRPC call failure handling
- **WHEN** the gRPC call to Node BullMQ service fails
- **THEN** the request SHALL be written to an outbox for later retry

### Requirement: Redis Operation Queue
The OT server SHALL maintain pending operations in Redis for asynchronous snapshot processing.

#### Scenario: Write operation to queue
- **WHEN** an operation is applied to a document
- **THEN** the operation metadata SHALL be written to Redis ZSET `ops:{doc_id}` with rev as score

#### Scenario: Read pending operations
- **WHEN** snapshot worker requests pending operations
- **THEN** operations SHALL be returned in rev order from the ZSET

#### Scenario: Remove processed operations
- **WHEN** snapshot is successfully generated up to version N
- **THEN** all operations with rev <= N SHALL be removed from Redis

### Requirement: Snapshot Queue Fault Tolerance
The snapshot queue mechanism SHALL guarantee no operation loss even when services are unavailable.

#### Scenario: Outbox retry mechanism
- **WHEN** outbox contains failed enqueue requests
- **THEN** a background task SHALL retry these requests with exponential backoff

#### Scenario: Compensation scan
- **WHEN** Node BullMQ service recovers
- **THEN** it SHALL scan Redis for documents with pending ops and ensure jobs are enqueued

#### Scenario: Idempotent processing
- **WHEN** a snapshot job is processed
- **THEN** it SHALL check checkpoint version and skip if `target_rev <= checkpoint`

### Requirement: Periodic Snapshot Trigger
The snapshot queue SHALL trigger snapshot generation on a time-based schedule.

#### Scenario: One-minute interval trigger
- **WHEN** one minute has elapsed since last snapshot check
- **THEN** the system SHALL identify documents with new operations and enqueue snapshot jobs

#### Scenario: Distributed lock for snapshot
- **WHEN** processing snapshot for a document
- **THEN** a distributed lock `snapshot:lock:{doc_id}` SHALL be acquired to prevent concurrent processing
