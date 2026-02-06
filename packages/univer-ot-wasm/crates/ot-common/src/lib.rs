//! OT Common - Shared library for OT server and writebehind worker
//!
//! This crate provides shared functionality including:
//! - Cache service for Redis operations
//! - Stream queue service for write-behind notifications (Redis Streams)
//! - Storage service for S3 operations
//! - Database entities for SeaORM
//! - Compression utilities
//! - COPY writer for PostgreSQL bulk inserts

pub mod cache;
pub mod compression;
pub mod copy_writer;
pub mod database;
pub mod storage;

// Re-export commonly used types
pub use cache::{
    CacheConfig, CacheService, CachedOperationInfo, OperationEntry, StreamInfo, StreamMessage,
    StreamQueueService,
};
pub use compression::{deserialize_smart, serialize_smart};
pub use copy_writer::{copy_operations, copy_operations_atomic, CopyOperationData, VersionUpdate};
pub use storage::StorageService;
