//! Cache service module
//!
//! Provides Redis-based caching for OT operations with write-behind support.
//! Queue notifications use Redis Streams via `StreamQueueService`.

mod queue;
mod service;
mod types;

pub use queue::{StreamInfo, StreamMessage, StreamQueueService};
pub use service::CacheService;
pub use types::{CacheConfig, CachedOperationInfo, OperationEntry};
