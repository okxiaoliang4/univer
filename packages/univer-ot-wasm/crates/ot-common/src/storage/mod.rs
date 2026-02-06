//! Storage service module
//!
//! Provides S3-based storage for documents, snapshots, and operation params.

mod service;

pub use service::{StorageLocation, StorageService, StoredSnapshot, UploadedOperationParams};
