use crate::services::{
    AuthService, AwarenessService, CacheConfig, CacheService, DocumentActorManager,
    DocumentService, EtcdService, GrpcClientService, OTService, OpQueueService, StorageService,
    WriteBehindConfig, WriteBehindWorker,
};
use sea_orm::DatabaseConnection;
use std::sync::Arc;

pub type AppState = Arc<ServerState>;

#[derive(Clone)]
pub struct ServerState {
    pub db: Arc<DatabaseConnection>,
    pub document_service: Arc<DocumentService>,
    pub ot_service: Arc<OTService>,
    pub document_actor_manager: Arc<DocumentActorManager>,
    pub storage_service: Arc<StorageService>,
    pub awareness_service: Arc<AwarenessService>,
    pub etcd_service: Arc<EtcdService>,
    pub grpc_client: Arc<GrpcClientService>,
    pub op_queue_service: Arc<OpQueueService>,
    pub auth_service: Arc<AuthService>,
    pub cache_service: Arc<CacheService>,
    pub writebehind_worker: Arc<WriteBehindWorker>,
}

impl ServerState {
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
        awareness_redis_enabled: bool,
        awareness_ttl_seconds: u64,
        etcd_service: Arc<EtcdService>,
        grpc_client: Arc<GrpcClientService>,
        auth_service: Arc<AuthService>,
        writebehind_batch_size: usize,
        writebehind_flush_interval_ms: u64,
        writebehind_worker_count: usize,
        writebehind_ttl_seconds: u64,
    ) -> Self {
        let db_arc = Arc::new(db);
        let storage_service = Arc::new(
            StorageService::new(
                (*db_arc).clone(),
                s3_endpoint,
                s3_region,
                s3_bucket,
                s3_access_key,
                s3_secret_key,
                server_env,
                redis_url.clone(),
            )
            .unwrap(),
        );

        let redis_client =
            redis::Client::open(redis_url.clone()).expect("Failed to initialize redis client");
        let op_queue_service = Arc::new(OpQueueService::new(redis_client.clone()));

        let awareness_service = Arc::new(
            AwarenessService::new(redis_url.clone(), awareness_redis_enabled, awareness_ttl_seconds)
                .unwrap(),
        );

        // Initialize cache service with ConnectionManager for efficient connection reuse
        let cache_config = CacheConfig {
            ttl_seconds: writebehind_ttl_seconds,
            batch_size: writebehind_batch_size,
        };
        let cache_service = Arc::new(
            CacheService::new(redis_client, cache_config)
                .await
                .expect("Failed to initialize CacheService with ConnectionManager"),
        );

        // Initialize services with cache
        let document_service = Arc::new(DocumentService::new(
            db_arc.clone(),
            storage_service.clone(),
            cache_service.clone(),
        ));

        let ot_service = Arc::new(OTService::new(
            db_arc.clone(),
            document_service.clone(),
            op_queue_service.clone(),
            &redis_url,
            cache_service.clone(),
            storage_service.clone(),
        ));

        // Initialize write-behind worker
        let wb_config = WriteBehindConfig {
            batch_size: writebehind_batch_size,
            flush_interval_ms: writebehind_flush_interval_ms,
            worker_count: writebehind_worker_count,
            ..Default::default()
        };
        let writebehind_worker = Arc::new(WriteBehindWorker::new(
            db_arc.clone(),
            cache_service.clone(),
            storage_service.clone(),
            wb_config,
        ));

        let document_actor_manager = Arc::new(DocumentActorManager::new(ot_service.clone()));

        Self {
            db: db_arc,
            document_service,
            ot_service,
            document_actor_manager,
            storage_service,
            awareness_service,
            etcd_service,
            grpc_client,
            op_queue_service,
            auth_service,
            cache_service,
            writebehind_worker,
        }
    }

    /// Start write-behind workers (call after state is created)
    pub fn start_writebehind_workers(&self) -> Vec<tokio::task::JoinHandle<()>> {
        self.writebehind_worker.start()
    }

    /// Flush all pending write-behind operations (call on shutdown)
    pub async fn flush_writebehind(&self) -> anyhow::Result<usize> {
        self.writebehind_worker.flush_all().await
    }

    /// Shutdown write-behind workers
    pub fn shutdown_writebehind(&self) {
        self.writebehind_worker.shutdown();
    }
}
