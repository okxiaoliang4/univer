use crate::server::services::{DocumentActorManager, DocumentService, OTService, SnapshotService};
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
}

impl ServerState {
    pub fn new(db: DatabaseConnection, snapshot_interval: u64) -> Self {
        let db_arc = Arc::new(db);
        let document_service = DocumentService::new((*db_arc).clone());
        let snapshot_service = SnapshotService::new((*db_arc).clone(), snapshot_interval);
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
        }
    }
}
