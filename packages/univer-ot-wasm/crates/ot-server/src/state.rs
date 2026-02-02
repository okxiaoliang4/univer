use crate::services::{
    AuthService, AwarenessService, DocumentActorManager, DocumentService, EtcdService,
    GrpcClientService, OTService, OpQueueService, SnapshotService, StorageService,
};
use sea_orm::DatabaseConnection;
use socketioxide::SocketIo;
use std::sync::Arc;

pub type AppState = Arc<ServerState>;

#[derive(Clone)]
pub struct ServerState {
    pub db: Arc<DatabaseConnection>,
    pub document_service: DocumentService,
    pub ot_service: OTService,
    pub document_actor_manager: DocumentActorManager,
    pub snapshot_service: SnapshotService,
    pub storage_service: StorageService,
    pub awareness_service: AwarenessService,
    pub etcd_service: EtcdService,
    pub grpc_client: GrpcClientService,
    pub op_queue_service: OpQueueService,
    pub socket_io: SocketIo,
    pub auth_service: AuthService,
}

impl ServerState {
    #[allow(clippy::too_many_arguments)]
    pub async fn new(
        db: DatabaseConnection,
        snapshot_interval: u64,
        s3_endpoint: String,
        s3_region: String,
        s3_bucket: String,
        s3_access_key: String,
        s3_secret_key: String,
        server_env: String,
        redis_url: String,
        awareness_redis_enabled: bool,
        awareness_ttl_seconds: u64,
        etcd_service: EtcdService,
        grpc_client: GrpcClientService,
        socket_io: SocketIo,
        auth_service: AuthService,
    ) -> Self {
        let db_arc = Arc::new(db);
        let storage_service = StorageService::new(
            (*db_arc).clone(),
            s3_endpoint,
            s3_region,
            s3_bucket,
            s3_access_key,
            s3_secret_key,
            server_env,
            redis_url.clone(),
        )
        .expect("Failed to initialize storage service");
        let document_service = DocumentService::new((*db_arc).clone(), storage_service.clone());
        let snapshot_service = SnapshotService::new(
            (*db_arc).clone(),
            snapshot_interval,
            storage_service.clone(),
        );
        let awareness_service = AwarenessService::new(
            redis_url.clone(),
            awareness_redis_enabled,
            awareness_ttl_seconds,
        )
        .expect("Failed to initialize awareness service");

        let op_queue_service = OpQueueService::new(
            redis::Client::open(redis_url.clone()).expect("Failed to initialize redis client"),
        );

        let ot_service = OTService::new(
            (*db_arc).clone(),
            document_service.clone(),
            op_queue_service.clone(),
        );

        let document_actor_manager = DocumentActorManager::new(ot_service.clone());

        Self {
            db: db_arc,
            document_service,
            ot_service,
            document_actor_manager,
            snapshot_service,
            storage_service,
            awareness_service,
            etcd_service,
            grpc_client,
            op_queue_service,
            socket_io,
            auth_service,
        }
    }
}
