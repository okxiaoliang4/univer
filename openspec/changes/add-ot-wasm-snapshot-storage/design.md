## Context
Document snapshots are currently stored inline in PostgreSQL as JSON. We want to move snapshot content to S3 using S3 object versioning while keeping metadata and revision data in PostgreSQL.

## Goals / Non-Goals
- Goals: introduce a reusable storage table for S3 metadata; link snapshots to storage records; use S3 version IDs for snapshot versioning; return signed URLs for snapshot content and cache them in Redis.
- Non-Goals: change snapshot formats in this change.

## Decisions
- Decision: create a generic `storages` table to hold S3 object metadata (endpoint, region, bucket, path, size, hash, version_id, etc.) so the table can be reused by other features.
- Decision: update `document_snapshots` to reference `storages` with a 1:1 relationship and store only metadata + rev in snapshot rows.
- Decision: API responses for snapshot content will return a signed URL (presigned 3 hours) and clients download content directly.
- Decision: cache signed URLs in Redis with a TTL shorter than the signed URL (default 1 minute less).

## Risks / Trade-offs
- Snapshot reads require an extra join to resolve storage data.
- Data migration needed for existing inline content.

## Migration Plan
1. Add `storages` table and `storage_id` column on `document_snapshots`.
2. Backfill existing snapshot content into S3 and create storage records.
3. Update snapshot persistence to write to S3 and store metadata only.
4. Update snapshot APIs to return signed URLs and add Redis caching.

## Open Questions
- Whether to store S3 credentials at record level or only in configuration.
- Confirm cache key format for signed URLs and Redis retention settings.
