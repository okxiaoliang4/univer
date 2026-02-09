pub mod auth;
pub mod document;
pub mod document_actor;
pub mod etcd;
pub mod grpc_client;
pub mod local_cache;
pub mod op_queue;
pub mod ot;
pub mod params_codec;

// Re-export shared types from ot-common
pub use ot_common::{CacheConfig, CacheService, CachedOperationInfo, OperationEntry};
pub use ot_common::{StreamInfo, StreamMessage, StreamQueueService};
pub use ot_common::StorageService;
pub use ot_common::compression;
pub use ot_common::copy_writer;

pub use auth::AuthService;
pub use document::DocumentService;
pub use document_actor::DocumentActorManager;
pub use etcd::EtcdService;
pub use grpc_client::GrpcClientService;
pub use op_queue::OpQueueService;
pub use ot::OTService;
