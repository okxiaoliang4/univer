use crate::state::AppState;
use ot_core::MutationInfoWithOpId;
use serde_json::Value as JsonValue;
use tonic::{Request, Response, Status};
use tracing::{error, info, warn};
use uuid::Uuid;

pub mod ot_rpc {
    tonic::include_proto!("ot_rpc");
}

const OT_RPC_DESCRIPTOR_SET: &[u8] =
    tonic::include_file_descriptor_set!("ot_rpc_descriptor");

use ot_rpc::editable_server::{Editable, EditableServer};
use ot_rpc::ot_rpc_service_server::{OtRpcService, OtRpcServiceServer};
use ot_rpc::{
    BroadcastOpRequest, BroadcastOpResponse, CloneDocumentRequest, CloneDocumentResponse,
    DeleteDocumentRequest, DeleteDocumentResponse, DocSnapshot, GetDocIdFromStorageIdRequest,
    GetDocIdFromStorageIdResponse, GetDocLatestsnapshotsRequest, GetDocLatestsnapshotsResponse,
    GetDocSnapshotListRequest, GetDocSnapshotListResponse, GetDocSnapshotRequest,
    NewDocumentRequest, NewDocumentResponse,
    RestoreDocumentRequest, RestoreDocumentResponse, SignObjectUrlRequest, SignObjectUrlResponse,
    UpdateDocSnapshotNameRequest, UpdateDocSnapshotNameResponse,
};

#[derive(Clone)]
pub struct OtGrpcService {
    state: AppState,
}

impl OtGrpcService {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }
}

#[tonic::async_trait]
impl OtRpcService for OtGrpcService {
    async fn broadcast_op(
        &self,
        request: Request<BroadcastOpRequest>,
    ) -> Result<Response<BroadcastOpResponse>, Status> {
        let req = request.into_inner();
        info!("broadcast_op request: doc_id={}, base_rev={}, user_id={}, mutations_count={}",
            req.doc_id, req.base_rev, req.user_id, req.mutations.len());

        let doc_id = match Uuid::parse_str(&req.doc_id) {
            Ok(id) => id,
            Err(e) => {
                error!("Invalid doc_id in broadcast_op: {} - {}", req.doc_id, e);
                return Err(Status::invalid_argument(format!("Invalid doc_id: {}", req.doc_id)));
            }
        };

        let mutations = req
            .mutations
            .into_iter()
            .enumerate()
            .map(|(idx, mutation)| {
                let params = match serde_json::from_str::<JsonValue>(&mutation.params) {
                    Ok(p) => p,
                    Err(e) => {
                        error!("Invalid mutation params at index {}: {}", idx, e);
                        return Err(Status::invalid_argument(format!("Invalid mutation params at index {}: {}", idx, e)));
                    }
                };
                Ok(MutationInfoWithOpId {
                    id: mutation.id,
                    params,
                    op_id: mutation.op_id,
                })
            })
            .collect::<Result<Vec<_>, Status>>()?;

        let changeset = crate::services::ot::Changeset {
            base_rev: req.base_rev,
            user_id: req.user_id.clone(),
            mutations: mutations.clone(),
            client_id: req.client_id.clone(),
        };

        let applied = match self
            .state
            .document_actor_manager
            .apply_changeset(doc_id, changeset)
            .await
        {
            Ok(result) => {
                info!("broadcast_op applied successfully: doc_id={}, server_rev={}",
                    doc_id, result.server_rev);
                result
            }
            Err(err) => {
                error!("Failed to apply changeset for doc_id={}: {}", doc_id, err);
                return Err(Status::internal(format!("Failed to apply changeset: {}", err)));
            }
        };

        // Note: Socket.IO broadcast is not performed here because:
        // 1. The changeset is already broadcasted via the socket handler when clients submit changes
        // 2. Redis adapter automatically syncs messages across all server instances
        // 3. This gRPC endpoint is primarily for applying changes from external services,
        //    and those changes will be picked up by clients via fetch_ops or changeset_pushed events
        //
        // If cross-server broadcast is needed, consider using Redis pub/sub directly
        // or passing SocketIo reference via a different mechanism.

        info!("broadcast_op completed successfully: doc_id={}, server_rev={}",
            doc_id, applied.server_rev);
        Ok(Response::new(BroadcastOpResponse {
            success: true,
            server_rev: applied.server_rev,
            error: String::new(),
        }))
    }
}

pub fn grpc_server(state: AppState) -> OtRpcServiceServer<OtGrpcService> {
    OtRpcServiceServer::new(OtGrpcService::new(state))
}

#[derive(Clone)]
pub struct EditableGrpcService {
    state: AppState,
}

impl EditableGrpcService {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }
}

#[tonic::async_trait]
impl Editable for EditableGrpcService {
    async fn new_document(
        &self,
        request: Request<NewDocumentRequest>,
    ) -> Result<Response<NewDocumentResponse>, Status> {
        let req = request.into_inner();
        info!("new_document request: doc_id={}, name={}, doc_type={}, creator_id={}",
            req.doc_id, req.name, req.doc_type, req.creator_id);

        let doc_id = match Uuid::parse_str(&req.doc_id) {
            Ok(id) => id,
            Err(e) => {
                error!("Invalid doc_id in new_document: {} - {}", req.doc_id, e);
                return Err(Status::invalid_argument(format!("Invalid doc_id: {}", req.doc_id)));
            }
        };
        if let Some(bytes) = req.updates {
            if bytes.is_empty() {
                warn!("Empty updates payload for doc_id={}, falling back to url", doc_id);
            } else {
                info!(
                    "Creating document from updates payload: doc_id={}, size={}",
                    doc_id,
                    bytes.len()
                );
                match serde_json::from_slice::<JsonValue>(&bytes) {
                    Ok(content) => {
                        match self.state
                            .document_service
                            .create_document(
                                doc_id,
                                req.creator_id,
                                req.name,
                                req.doc_type as i16,
                                req.create_type as i16,
                                content,
                            )
                            .await
                        {
                            Ok(_) => {
                                info!("Document created successfully from updates: doc_id={}", doc_id);
                                return Ok(Response::new(NewDocumentResponse {}));
                            }
                            Err(err) => {
                                error!(
                                    "Failed to create document from updates: doc_id={}, error={}",
                                    doc_id, err
                                );
                                return Err(Status::internal(format!(
                                    "Failed to create document: {}",
                                    err
                                )));
                            }
                        }
                    }
                    Err(e) => {
                        warn!(
                            "Invalid updates payload for doc_id={}, will fallback to url: {}",
                            doc_id, e
                        );
                    }
                }
            }
        }

        if let Some(url) = req.url {
            info!("Creating document from URL: doc_id={}, url={}", doc_id, url);
            return match self.state
                .document_service
                .create_document_from_url(
                    doc_id,
                    req.creator_id,
                    req.name,
                    req.doc_type as i16,
                    req.create_type as i16,
                    url,
                )
                .await
            {
                Ok(_) => {
                    info!("Document created successfully from URL: doc_id={}", doc_id);
                    Ok(Response::new(NewDocumentResponse {}))
                }
                Err(err) => {
                    error!("Failed to create document from URL: doc_id={}, error={}", doc_id, err);
                    Err(Status::internal(format!("Failed to create document: {}", err)))
                }
            };
        }

        // Both bytes and url are missing or invalid
        error!(
            "Neither valid updates payload nor url provided for doc_id={}",
            doc_id
        );
        Err(Status::invalid_argument(
            "Either updates payload or url must be provided"
        ))
    }

    async fn clone_document(
        &self,
        request: Request<CloneDocumentRequest>,
    ) -> Result<Response<CloneDocumentResponse>, Status> {
        let req = request.into_inner();
        let snapshot_id_str = req.snapshot_id.clone();
        info!("clone_document request: doc_id={}, creator_id={}, snapshot_id={:?}",
            req.doc_id, req.creator_id, snapshot_id_str);

        let doc_id = match Uuid::parse_str(&req.doc_id) {
            Ok(id) => id,
            Err(e) => {
                error!("Invalid doc_id in clone_document: {} - {}", req.doc_id, e);
                return Err(Status::invalid_argument(format!("Invalid doc_id: {}", req.doc_id)));
            }
        };
        let snapshot_id = match req
            .snapshot_id
            .map(|value| Uuid::parse_str(&value))
            .transpose()
        {
            Ok(id) => id,
            Err(e) => {
                error!("Invalid snapshot_id in clone_document: {:?} - {}", snapshot_id_str, e);
                return Err(Status::invalid_argument(format!("Invalid snapshot_id")));
            }
        };
        let doc_type = req.doc_type.map(|value| value as i16);

        let (new_doc_id, size) = match self
            .state
            .document_service
            .clone_document(doc_id, req.creator_id, snapshot_id, doc_type)
            .await
        {
            Ok(result) => {
                info!("Document cloned successfully: source_doc_id={}, new_doc_id={}, size={}",
                    doc_id, result.0, result.1);
                result
            }
            Err(err) => {
                error!("Failed to clone document: doc_id={}, error={}", doc_id, err);
                return Err(Status::internal(format!("Failed to clone document: {}", err)));
            }
        };

        Ok(Response::new(CloneDocumentResponse {
            doc_id: new_doc_id.to_string(),
            size,
        }))
    }

    async fn delete_document(
        &self,
        request: Request<DeleteDocumentRequest>,
    ) -> Result<Response<DeleteDocumentResponse>, Status> {
        let req = request.into_inner();
        let is_soft = req.is_soft.unwrap_or(false);
        info!("delete_document request: doc_id={}, is_soft={}", req.doc_id, is_soft);

        let doc_id = match Uuid::parse_str(&req.doc_id) {
            Ok(id) => id,
            Err(e) => {
                error!("Invalid doc_id in delete_document: {} - {}", req.doc_id, e);
                return Err(Status::invalid_argument(format!("Invalid doc_id: {}", req.doc_id)));
            }
        };

        match self.state
            .document_service
            .delete_document(doc_id, is_soft)
            .await
        {
            Ok(_) => {
                info!("Document deleted successfully: doc_id={}, is_soft={}", doc_id, is_soft);
                Ok(Response::new(DeleteDocumentResponse {}))
            }
            Err(err) => {
                error!("Failed to delete document: doc_id={}, error={}", doc_id, err);
                Err(Status::internal(format!("Failed to delete document: {}", err)))
            }
        }
    }

    async fn restore_document(
        &self,
        request: Request<RestoreDocumentRequest>,
    ) -> Result<Response<RestoreDocumentResponse>, Status> {
        let req = request.into_inner();
        info!("restore_document request: doc_id={}, snapshot_id={}", req.doc_id, req.snapshot_id);

        let doc_id = match Uuid::parse_str(&req.doc_id) {
            Ok(id) => id,
            Err(e) => {
                error!("Invalid doc_id in restore_document: {} - {}", req.doc_id, e);
                return Err(Status::invalid_argument(format!("Invalid doc_id: {}", req.doc_id)));
            }
        };
        let snapshot_id = match Uuid::parse_str(&req.snapshot_id) {
            Ok(id) => id,
            Err(e) => {
                error!("Invalid snapshot_id in restore_document: {} - {}", req.snapshot_id, e);
                return Err(Status::invalid_argument(format!("Invalid snapshot_id: {}", req.snapshot_id)));
            }
        };

        match self.state
            .document_service
            .restore_document(doc_id, snapshot_id)
            .await
        {
            Ok(_) => {
                info!("Document restored successfully: doc_id={}, snapshot_id={}", doc_id, snapshot_id);
                Ok(Response::new(RestoreDocumentResponse {}))
            }
            Err(err) => {
                error!("Failed to restore document: doc_id={}, snapshot_id={}, error={}",
                    doc_id, snapshot_id, err);
                Err(Status::internal(format!("Failed to restore document: {}", err)))
            }
        }
    }

    async fn get_doc_snapshot_list(
        &self,
        request: Request<GetDocSnapshotListRequest>,
    ) -> Result<Response<GetDocSnapshotListResponse>, Status> {
        let req = request.into_inner();
        let limit = req.limit.unwrap_or(10);
        let desc = req.desc.unwrap_or(true);
        let cursor_str = req.cursor.clone();
        info!("get_doc_snapshot_list request: doc_id={}, limit={}, cursor={:?}, desc={}, desc_raw={:?}",
            req.doc_id, limit, cursor_str, desc, req.desc);

        let doc_id = match Uuid::parse_str(&req.doc_id) {
            Ok(id) => id,
            Err(e) => {
                error!("Invalid doc_id in get_doc_snapshot_list: {} - {}", req.doc_id, e);
                return Err(Status::invalid_argument(format!("Invalid doc_id: {}", req.doc_id)));
            }
        };
        // Filter out empty strings and parse cursor as i64
        // Empty string is treated as None (first page request)
        let cursor = match req
            .cursor
            .filter(|s| !s.is_empty())
            .map(|value| value.parse::<i64>())
            .transpose()
        {
            Ok(c) => c,
            Err(e) => {
                error!("Invalid cursor in get_doc_snapshot_list: {:?} - {}", cursor_str, e);
                return Err(Status::invalid_argument(format!("Invalid cursor")));
            }
        };

        let (snapshots, next_cursor) = match self.state
            .document_service
            .list_snapshots(doc_id, limit, cursor, desc)
            .await
        {
            Ok(result) => {
                info!("Retrieved snapshot list: doc_id={}, count={}, has_next={}",
                    doc_id, result.0.len(), result.1.is_some());
                result
            }
            Err(err) => {
                error!("Failed to list snapshots: doc_id={}, error={}", doc_id, err);
                return Err(Status::internal(format!("Failed to list snapshots: {}", err)));
            }
        };

        let snapshots = snapshots
            .into_iter()
            .map(|snapshot| to_doc_snapshot(snapshot))
            .collect();

        Ok(Response::new(GetDocSnapshotListResponse {
            doc_id: req.doc_id,
            snapshots,
            next_cursor: next_cursor.map(|value| value.to_string()),
        }))
    }

    async fn update_doc_snapshot_name(
        &self,
        request: Request<UpdateDocSnapshotNameRequest>,
    ) -> Result<Response<UpdateDocSnapshotNameResponse>, Status> {
        let req = request.into_inner();
        info!("update_doc_snapshot_name request: doc_id={}, snapshot_id={}, name={}",
            req.doc_id, req.snapshot_id, req.name);

        let doc_id = match Uuid::parse_str(&req.doc_id) {
            Ok(id) => id,
            Err(e) => {
                error!("Invalid doc_id in update_doc_snapshot_name: {} - {}", req.doc_id, e);
                return Err(Status::invalid_argument(format!("Invalid doc_id: {}", req.doc_id)));
            }
        };
        let snapshot_id = match Uuid::parse_str(&req.snapshot_id) {
            Ok(id) => id,
            Err(e) => {
                error!("Invalid snapshot_id in update_doc_snapshot_name: {} - {}", req.snapshot_id, e);
                return Err(Status::invalid_argument(format!("Invalid snapshot_id: {}", req.snapshot_id)));
            }
        };

        match self.state
            .document_service
            .update_snapshot_name(doc_id, snapshot_id, req.name.clone())
            .await
        {
            Ok(_) => {
                info!("Snapshot name updated successfully: doc_id={}, snapshot_id={}, name={}",
                    doc_id, snapshot_id, req.name);
                Ok(Response::new(UpdateDocSnapshotNameResponse {}))
            }
            Err(err) => {
                error!("Failed to update snapshot name: doc_id={}, snapshot_id={}, error={}",
                    doc_id, snapshot_id, err);
                Err(Status::internal(format!("Failed to update snapshot name: {}", err)))
            }
        }
    }

    async fn get_documentsnapshot(
        &self,
        request: Request<GetDocSnapshotRequest>,
    ) -> Result<Response<DocSnapshot>, Status> {
        let req = request.into_inner();
        let snapshot_id_str = req.snapshot_id.clone();
        info!("get_documentsnapshot request: doc_id={}, snapshot_id={:?}", req.doc_id, snapshot_id_str);

        let doc_id = match Uuid::parse_str(&req.doc_id) {
            Ok(id) => id,
            Err(e) => {
                error!("Invalid doc_id in get_documentsnapshot: {} - {}", req.doc_id, e);
                return Err(Status::invalid_argument(format!("Invalid doc_id: {}", req.doc_id)));
            }
        };
        let snapshot_id = match req
            .snapshot_id
            .map(|value| Uuid::parse_str(&value))
            .transpose()
        {
            Ok(id) => id,
            Err(e) => {
                error!("Invalid snapshot_id in get_documentsnapshot: {:?} - {}", snapshot_id_str, e);
                return Err(Status::invalid_argument(format!("Invalid snapshot_id")));
            }
        };

        let snapshot = match self.state
            .document_service
            .get_doc_snapshot(doc_id, snapshot_id)
            .await
        {
            Ok(Some(snapshot)) => {
                info!("Retrieved snapshot: doc_id={}, snapshot_id={:?}", doc_id, snapshot_id);
                snapshot
            }
            Ok(None) => {
                warn!("Snapshot not found: doc_id={}, snapshot_id={:?}", doc_id, snapshot_id);
                return Err(Status::not_found("Snapshot not found"));
            }
            Err(err) => {
                error!("Failed to get snapshot: doc_id={}, snapshot_id={:?}, error={}",
                    doc_id, snapshot_id, err);
                return Err(Status::internal(format!("Failed to get snapshot: {}", err)));
            }
        };

        Ok(Response::new(to_doc_snapshot(snapshot)))
    }

    async fn get_doc_latestsnapshots(
        &self,
        request: Request<GetDocLatestsnapshotsRequest>,
    ) -> Result<Response<GetDocLatestsnapshotsResponse>, Status> {
        let req = request.into_inner();
        info!("get_doc_latestsnapshots request: doc_ids_count={}", req.doc_ids.len());

        let doc_ids = match req
            .doc_ids
            .iter()
            .map(|id| Uuid::parse_str(id))
            .collect::<Result<Vec<_>, _>>()
        {
            Ok(ids) => ids,
            Err(e) => {
                error!("Invalid doc_id in get_doc_latestsnapshots: {:?} - {}", req.doc_ids, e);
                return Err(Status::invalid_argument(format!("Invalid doc_id")));
            }
        };

        let snapshots = match self
            .state
            .document_service
            .get_latest_snapshots(doc_ids.clone())
            .await
        {
            Ok(result) => {
                info!("Retrieved latest snapshots: requested_count={}, returned_count={}",
                    doc_ids.len(), result.len());
                result
            }
            Err(err) => {
                error!("Failed to get latest snapshots: doc_ids={:?}, error={}", doc_ids, err);
                return Err(Status::internal(format!("Failed to get latest snapshots: {}", err)));
            }
        };

        let snapshot_map = snapshots
            .into_iter()
            .map(|(doc_id, snapshot)| (doc_id.to_string(), to_doc_snapshot(snapshot)))
            .collect();

        Ok(Response::new(GetDocLatestsnapshotsResponse { snapshots: snapshot_map }))
    }

    async fn sign_object_url(
        &self,
        request: Request<SignObjectUrlRequest>,
    ) -> Result<Response<SignObjectUrlResponse>, Status> {
        let req = request.into_inner();
        info!("sign_object_url request: storage_ids_count={}", req.storage_ids.len());

        let storage_ids = match req
            .storage_ids
            .iter()
            .map(|id| Uuid::parse_str(id))
            .collect::<Result<Vec<_>, _>>()
        {
            Ok(ids) => ids,
            Err(e) => {
                error!("Invalid storage_id in sign_object_url: {:?} - {}", req.storage_ids, e);
                return Err(Status::invalid_argument(format!("Invalid storage_id")));
            }
        };

        let urls = match self
            .state
            .document_service
            .sign_object_urls(storage_ids.clone())
            .await
        {
            Ok(result) => {
                info!("Signed object URLs: requested_count={}, returned_count={}",
                    storage_ids.len(), result.len());
                result
            }
            Err(err) => {
                error!("Failed to sign object URLs: storage_ids={:?}, error={}", storage_ids, err);
                return Err(Status::internal(format!("Failed to sign object URLs: {}", err)));
            }
        };
        let url_map = urls
            .into_iter()
            .map(|(id, url)| (id.to_string(), url))
            .collect();
        Ok(Response::new(SignObjectUrlResponse { urls: url_map }))
    }

    async fn get_doc_id_from_storage_id(
        &self,
        request: Request<GetDocIdFromStorageIdRequest>,
    ) -> Result<Response<GetDocIdFromStorageIdResponse>, Status> {
        let req = request.into_inner();
        info!("get_doc_id_from_storage_id request: storage_id={}", req.storage_id);

        let storage_id = match Uuid::parse_str(&req.storage_id) {
            Ok(id) => id,
            Err(e) => {
                error!("Invalid storage_id in get_doc_id_from_storage_id: {} - {}", req.storage_id, e);
                return Err(Status::invalid_argument(format!("Invalid storage_id: {}", req.storage_id)));
            }
        };

        let doc_id = match self
            .state
            .document_service
            .get_doc_id_by_storage_id(storage_id)
            .await
        {
            Ok(Some(id)) => {
                info!("Found doc_id for storage_id: storage_id={}, doc_id={}", storage_id, id);
                id
            }
            Ok(None) => {
                warn!("Document not found for storage_id: {}", storage_id);
                return Err(Status::not_found("Document not found"));
            }
            Err(err) => {
                error!("Failed to get doc_id from storage_id: storage_id={}, error={}", storage_id, err);
                return Err(Status::internal(format!("Failed to get doc_id: {}", err)));
            }
        };

        Ok(Response::new(GetDocIdFromStorageIdResponse {
            doc_id: doc_id.to_string(),
        }))
    }
}

pub fn editable_server(state: AppState) -> EditableServer<EditableGrpcService> {
    EditableServer::new(EditableGrpcService::new(state))
}

pub fn reflection_server(
) -> tonic_reflection::server::ServerReflectionServer<impl tonic_reflection::server::ServerReflection> {
    match tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(OT_RPC_DESCRIPTOR_SET)
        .build()
    {
        Ok(server) => {
            info!("gRPC reflection server built successfully");
            server
        }
        Err(e) => {
            error!("Failed to build gRPC reflection service: {}", e);
            panic!("Failed to build gRPC reflection service: {}", e);
        }
    }
}

fn to_doc_snapshot(snapshot: crate::services::document::DocumentSnapshotInfo) -> DocSnapshot {
    let restore_from = snapshot.restore_from.map(|info| *info).map(to_doc_snapshot);
    DocSnapshot {
        id: snapshot.id.to_string(),
        doc_id: snapshot.doc_id.to_string(),
        users: snapshot.users,
        name: snapshot.name.unwrap_or_default(),
        size: snapshot.size.unwrap_or(0),
        storage_id: snapshot.storage_id.to_string(),
        created_at: snapshot.created_at.timestamp() as i32,
        updated_at: snapshot.updated_at.timestamp() as i32,
        restore_from_id: snapshot.restore_from_id.map(|id| id.to_string()),
        restore_from: restore_from.map(Box::new),
    }
}
