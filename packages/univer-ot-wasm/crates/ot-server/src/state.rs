use crate::services::{
    AuthService, CacheConfig, CacheService, DocumentActorManager,
    DocumentService, EtcdService, GrpcClientService, OTService, OpQueueService, StorageService,
    StreamQueueService,
};
use crate::services::local_cache::LocalCache;
use dashmap::DashMap;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{info, warn};

pub type AppState = Arc<ServerState>;

/// Rate limiter for gRPC document modification notifications.
///
/// Prevents flooding the notification service during high-frequency editing.
/// Uses leading-edge rate limiting: first call passes immediately, subsequent
/// calls within the interval are suppressed.
#[derive(Clone)]
pub struct NotifyDebouncer {
    last_notify: Arc<DashMap<String, Instant>>,
    interval: Duration,
}

impl NotifyDebouncer {
    pub fn new(interval_secs: u64) -> Self {
        Self {
            last_notify: Arc::new(DashMap::new()),
            interval: Duration::from_secs(interval_secs),
        }
    }

    /// Returns true if enough time has passed since last notification for this doc.
    /// Uses DashMap entry API for atomic check-and-update.
    pub fn should_notify(&self, doc_id: &str) -> bool {
        let now = Instant::now();
        let mut entry = self
            .last_notify
            .entry(doc_id.to_string())
            .or_insert(now - self.interval * 2);
        if now.duration_since(*entry) >= self.interval {
            *entry = now;
            true
        } else {
            false
        }
    }
}

/// Redis Pub/Sub publisher for broadcasting changeset notifications to ws-gateway.
///
/// Uses ConnectionManager for efficient connection reuse and automatic reconnection.
#[derive(Clone)]
pub struct RedisBroadcastPublisher {
    conn_manager: redis::aio::ConnectionManager,
}

impl RedisBroadcastPublisher {
    pub async fn new(redis_url: &str) -> Self {
        let client = redis::Client::open(redis_url).expect("Failed to create Redis broadcast client");
        let conn_manager = redis::aio::ConnectionManager::new(client)
            .await
            .expect("Failed to create Redis ConnectionManager for broadcast");
        info!("RedisBroadcastPublisher: ConnectionManager initialized");
        Self { conn_manager }
    }

    /// Publish a message to the `ws:broadcast` channel.
    ///
    /// Uses a cloned ConnectionManager which reuses the underlying connection.
    pub async fn publish(&self, message: &str) {
        let mut conn = self.conn_manager.clone();
        let result: Result<(), _> = redis::cmd("PUBLISH")
            .arg("ws:broadcast")
            .arg(message)
            .query_async(&mut conn)
            .await;
        if let Err(e) = result {
            warn!("Failed to publish to ws:broadcast: {}", e);
        }
    }
}

#[derive(Clone)]
pub struct ServerState {
    pub db: Arc<DatabaseConnection>,
    pub document_service: Arc<DocumentService>,
    pub ot_service: Arc<OTService>,
    pub document_actor_manager: Arc<DocumentActorManager>,
    pub storage_service: Arc<StorageService>,
    pub etcd_service: Arc<EtcdService>,
    pub grpc_client: Arc<GrpcClientService>,
    pub op_queue_service: Arc<OpQueueService>,
    pub auth_service: Arc<AuthService>,
    pub cache_service: Arc<CacheService>,
    pub queue_service: Arc<StreamQueueService>,
    pub local_cache: LocalCache,
    pub notify_debouncer: NotifyDebouncer,
    pub redis_broadcast: RedisBroadcastPublisher,
}

impl ServerState {
    /// Create a new ServerState
    ///
    /// Note: WriteBehind worker has been moved to a separate crate (writebehind-worker)
    /// that runs as an independent process. The StreamQueueService uses Redis Streams
    /// to notify the external worker when documents need to be flushed.
    #[allow(clippy::too_many_arguments)]
    pub async fn new(
        db: DatabaseConnection,
        s3_endpoint: String,
        s3_region: String,
        s3_bucket: String,
        s3_access_key: String,
        s3_secret_key: String,
        server_env: String,
        redis_url: String,
        etcd_service: Arc<EtcdService>,
        grpc_client: Arc<GrpcClientService>,
        auth_service: Arc<AuthService>,
        cache_ttl_seconds: u64,
        cache_batch_size: usize,
    ) -> Self {
        let db_arc = Arc::new(db);

        // Create Redis client first (will be cloned for multiple services)
        let redis_client =
            redis::Client::open(redis_url.clone()).expect("Failed to initialize redis client");

        // Initialize StorageService with Arc<DatabaseConnection> and redis_client clone
        // This avoids unnecessary DatabaseConnection clone + Arc::new in StorageService::new
        let storage_service = Arc::new(
            StorageService::new_with_db(
                db_arc.clone(),
                s3_endpoint,
                s3_region,
                s3_bucket,
                s3_access_key,
                s3_secret_key,
                server_env,
                redis_client.clone(),
            )
            .await
            .expect("Failed to initialize StorageService with ConnectionManager"),
        );

        let op_queue_service = Arc::new(
            OpQueueService::new(redis_client.clone())
                .await
                .expect("Failed to initialize OpQueueService with ConnectionManager"),
        );

        // Initialize cache service with ConnectionManager for efficient connection reuse
        let cache_config = CacheConfig {
            ttl_seconds: cache_ttl_seconds,
            batch_size: cache_batch_size,
        };
        let cache_service = Arc::new(
            CacheService::new(redis_client, cache_config)
                .await
                .expect("Failed to initialize CacheService with ConnectionManager"),
        );

        // Initialize StreamQueueService (reuses CacheService connections)
        let queue_service = Arc::new(StreamQueueService::new(
            cache_service.connection_managers(),
        ));

        // Initialize services with cache
        let document_service = Arc::new(DocumentService::new(
            db_arc.clone(),
            storage_service.clone(),
            cache_service.clone(),
        ));

        // Process-local LRU cache (L1): survives write-behind worker's Redis cleanup
        let local_cache = LocalCache::new(
            10_000, // max 10,000 documents cached
            300,    // TTL 300 seconds (5 minutes)
        );

        let ot_service = Arc::new(OTService::new(
            db_arc.clone(),
            document_service.clone(),
            op_queue_service.clone(),
            &redis_url,
            cache_service.clone(),
            storage_service.clone(),
            queue_service.clone(),
            local_cache.clone(),
        ));

        let document_actor_manager = Arc::new(DocumentActorManager::new(ot_service.clone()));

        // gRPC notification rate limiter: at most one notify per doc per 5 seconds
        let notify_debouncer = NotifyDebouncer::new(5);

        // Redis Pub/Sub publisher for ws-gateway broadcast (uses ConnectionManager for connection reuse)
        let redis_broadcast = RedisBroadcastPublisher::new(&redis_url).await;

        Self {
            db: db_arc,
            document_service,
            ot_service,
            document_actor_manager,
            storage_service,
            etcd_service,
            grpc_client,
            op_queue_service,
            auth_service,
            cache_service,
            queue_service,
            local_cache,
            notify_debouncer,
            redis_broadcast,
        }
    }
}
