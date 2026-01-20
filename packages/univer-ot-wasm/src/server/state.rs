use crate::server::services::{
    DocumentActorManager, DocumentService, OTService, SnapshotService, StorageService,
};
use sea_orm::DatabaseConnection;
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
}

impl ServerState {
    pub fn new(
        db: DatabaseConnection,
        snapshot_interval: u64,
        s3_endpoint: String,
        s3_region: String,
        s3_bucket: String,
        s3_access_key: String,
        s3_secret_key: String,
        redis_url: String,
    ) -> Self {
        let db_arc = Arc::new(db);
        let storage_service = StorageService::new(
            (*db_arc).clone(),
            s3_endpoint,
            s3_region,
            s3_bucket,
            s3_access_key,
            s3_secret_key,
            redis_url,
        )
        .expect("Failed to initialize storage service");
        let document_service = DocumentService::new((*db_arc).clone(), storage_service.clone());
        let snapshot_service = SnapshotService::new(
            (*db_arc).clone(),
            snapshot_interval,
            storage_service.clone(),
        );
        let ot_service = OTService::new(
            (*db_arc).clone(),
            document_service.clone(),
            snapshot_service.clone(),
        );
        let document_actor_manager = DocumentActorManager::new(ot_service.clone());

        Self {
            db: db_arc,
            document_service,
            ot_service,
            document_actor_manager,
            snapshot_service,
            storage_service,
        }
    }
}
