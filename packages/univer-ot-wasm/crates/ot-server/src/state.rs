use crate::services::{
    AuthService, AwarenessService, DocumentActorManager, DocumentService, EtcdService,
    GrpcClientService, OTService, OpQueueService, StorageService,
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
    ) -> Self {
        let db_arc = Arc::new(db);
        let storage_service = Arc::new(StorageService::new(
            (*db_arc).clone(),
            s3_endpoint,
            s3_region,
            s3_bucket,
            s3_access_key,
            s3_secret_key,
            server_env,
            redis_url.clone(),
        ).unwrap());
        let document_service = Arc::new(DocumentService::new(db_arc.clone(), storage_service.clone()));
        let awareness_service = Arc::new(AwarenessService::new(
            redis_url.clone(),
            awareness_redis_enabled,
            awareness_ttl_seconds,
        )
        .unwrap());

        let op_queue_service = Arc::new(OpQueueService::new(
            redis::Client::open(redis_url.clone()).expect("Failed to initialize redis client"),
        ));

        let ot_service = Arc::new(OTService::new(
            db_arc.clone(),
            document_service.clone(),
            op_queue_service.clone(),
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
        }
    }
}
