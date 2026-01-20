## Why
Document snapshots are stored inline in PostgreSQL, which limits storage scale and makes versioning difficult. Moving snapshot content to S3 with versioning keeps the database lean while enabling durable history.

## What Changes
- Add a storage table to record S3 object metadata and version identifiers for any stored content.
- Update document snapshot persistence to store only metadata and link to a storage record.
- Use S3 object version IDs as the snapshot content versioning mechanism.
- Update document snapshot API responses to return a signed URL for content download (client fetches via CDN/cache).
- Cache signed URLs in Redis to reuse signatures within a shorter TTL than the URL itself.

## Impact
- Affected specs: ot-wasm
- Affected code: ot snapshot entities, API handlers, Redis caching, and database migrations in `packages/univer-ot-wasm`
