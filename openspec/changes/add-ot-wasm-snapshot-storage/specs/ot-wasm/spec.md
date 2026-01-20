## ADDED Requirements
### Requirement: Snapshot Storage Records
The system SHALL store snapshot content in S3 and persist a storage record with S3 object metadata, including version identifiers, for each snapshot.

#### Scenario: Persisting a snapshot
- **WHEN** a snapshot is created
- **THEN** the snapshot content is written to S3 and a storage record is created with the S3 version ID

### Requirement: Snapshot Metadata Storage
The system SHALL store snapshot metadata and revision information in PostgreSQL and reference the associated storage record.

#### Scenario: Reading snapshot metadata
- **WHEN** snapshot metadata is retrieved
- **THEN** the storage record identifier is available to resolve the snapshot content

### Requirement: Signed Snapshot Access
The system SHALL return a time-limited signed URL for snapshot content download from the snapshot API and cache signed URLs in Redis with a TTL shorter than the URL validity period.

#### Scenario: Returning snapshot content URL
- **WHEN** a client requests the latest document snapshot
- **THEN** the response includes a signed URL that the client can use to download content directly

#### Scenario: Reusing cached signed URL
- **WHEN** a signed URL is requested while a Redis cache entry exists
- **THEN** the cached URL is returned without generating a new signature
