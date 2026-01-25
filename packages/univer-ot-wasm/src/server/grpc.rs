use crate::server::state::AppState;
use crate::types::MutationInfoInternal;
use crate::types::MutationInfoWithOpId;
use serde_json::Value as JsonValue;
use tonic::{Request, Response, Status};
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
        let doc_id =
            Uuid::parse_str(&req.doc_id).map_err(|_| Status::invalid_argument("Invalid doc_id"))?;

        let mutations = req
            .mutations
            .into_iter()
            .map(|mutation| {
                let params = serde_json::from_str::<JsonValue>(&mutation.params)
                    .map_err(|_| Status::invalid_argument("Invalid mutation params"))?;
                Ok(MutationInfoWithOpId {
                    id: mutation.id,
                    params,
                    op_id: mutation.op_id,
                })
            })
            .collect::<Result<Vec<_>, Status>>()?;

        let changeset = crate::server::services::ot::Changeset {
            base_rev: req.base_rev,
            user_id: req.user_id.clone(),
            mutations: mutations.clone(),
            client_id: req.client_id.clone(),
        };

        let applied = self
            .state
            .document_actor_manager
            .apply_changeset(doc_id, changeset)
            .await
            .map_err(|err| Status::internal(err.to_string()))?;

        let room = format!("doc:{}", req.doc_id);
        let pushed = crate::server::types::ChangesetPushed {
            doc_id: req.doc_id.clone(),
            server_rev: applied.server_rev,
            user_id: applied.user_id.clone(),
            mutations: applied.mutations.clone(),
        };

        if let Ok(json) = serde_json::to_value(&pushed) {
            let _ = self
                .state
                .socket_io
                .to(room)
                .emit("changeset_pushed", &json)
                .await;
        }

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
        let doc_id = Uuid::parse_str(&req.doc_id)
            .map_err(|_| Status::invalid_argument("Invalid doc_id"))?;
        if let Some(bytes) = req.updates {
            let content = serde_json::from_slice::<JsonValue>(&bytes)
                .map_err(|_| Status::invalid_argument("Invalid updates payload"))?;
            self.state
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
                .map_err(|err| Status::internal(err.to_string()))?;
        } else if let Some(url) = req.url {
            self.state
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
                .map_err(|err| Status::internal(err.to_string()))?;
        } else {
            self.state
                .document_service
                .create_document(
                    doc_id,
                    req.creator_id,
                    req.name,
                    req.doc_type as i16,
                    req.create_type as i16,
                    JsonValue::Object(Default::default()),
                )
                .await
                .map_err(|err| Status::internal(err.to_string()))?;
        }

        Ok(Response::new(NewDocumentResponse {}))
    }

    async fn clone_document(
        &self,
        request: Request<CloneDocumentRequest>,
    ) -> Result<Response<CloneDocumentResponse>, Status> {
        let req = request.into_inner();
        let doc_id = Uuid::parse_str(&req.doc_id)
            .map_err(|_| Status::invalid_argument("Invalid doc_id"))?;
        let snapshot_id = req
            .snapshot_id
            .map(|value| Uuid::parse_str(&value))
            .transpose()
            .map_err(|_| Status::invalid_argument("Invalid snapshot_id"))?;
        let doc_type = req.doc_type.map(|value| value as i16);

        let (new_doc_id, size) = self
            .state
            .document_service
            .clone_document(doc_id, req.creator_id, snapshot_id, doc_type)
            .await
            .map_err(|err| Status::internal(err.to_string()))?;

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
        let doc_id = Uuid::parse_str(&req.doc_id)
            .map_err(|_| Status::invalid_argument("Invalid doc_id"))?;
        self.state
            .document_service
            .delete_document(doc_id, req.is_soft.unwrap_or(false))
            .await
            .map_err(|err| Status::internal(err.to_string()))?;
        Ok(Response::new(DeleteDocumentResponse {}))
    }

    async fn restore_document(
        &self,
        request: Request<RestoreDocumentRequest>,
    ) -> Result<Response<RestoreDocumentResponse>, Status> {
        let req = request.into_inner();
        let doc_id = Uuid::parse_str(&req.doc_id)
            .map_err(|_| Status::invalid_argument("Invalid doc_id"))?;
        let snapshot_id = Uuid::parse_str(&req.snapshot_id)
            .map_err(|_| Status::invalid_argument("Invalid snapshot_id"))?;

        self.state
            .document_service
            .restore_document(doc_id, snapshot_id)
            .await
            .map_err(|err| Status::internal(err.to_string()))?;

        Ok(Response::new(RestoreDocumentResponse {}))
    }

    async fn get_doc_snapshot_list(
        &self,
        request: Request<GetDocSnapshotListRequest>,
    ) -> Result<Response<GetDocSnapshotListResponse>, Status> {
        let req = request.into_inner();
        let doc_id = Uuid::parse_str(&req.doc_id)
            .map_err(|_| Status::invalid_argument("Invalid doc_id"))?;
        let cursor = req
            .cursor
            .map(|value| value.parse::<i64>())
            .transpose()
            .map_err(|_| Status::invalid_argument("Invalid cursor"))?;

        let (snapshots, next_cursor) = self.state
            .document_service
            .list_snapshots(
                doc_id,
                req.limit.unwrap_or(10),
                cursor,
                req.desc.unwrap_or(true),
            )
            .await
            .map_err(|err| Status::internal(err.to_string()))?;

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
        let doc_id = Uuid::parse_str(&req.doc_id)
            .map_err(|_| Status::invalid_argument("Invalid doc_id"))?;
        let snapshot_id = Uuid::parse_str(&req.snapshot_id)
            .map_err(|_| Status::invalid_argument("Invalid snapshot_id"))?;
        self.state
            .document_service
            .update_snapshot_name(doc_id, snapshot_id, req.name)
            .await
            .map_err(|err| Status::internal(err.to_string()))?;
        Ok(Response::new(UpdateDocSnapshotNameResponse {}))
    }

    async fn get_documentsnapshot(
        &self,
        request: Request<GetDocSnapshotRequest>,
    ) -> Result<Response<DocSnapshot>, Status> {
        let req = request.into_inner();
        let doc_id = Uuid::parse_str(&req.doc_id)
            .map_err(|_| Status::invalid_argument("Invalid doc_id"))?;
        let snapshot_id = req
            .snapshot_id
            .map(|value| Uuid::parse_str(&value))
            .transpose()
            .map_err(|_| Status::invalid_argument("Invalid snapshot_id"))?;

        let snapshot = self.state
            .document_service
            .get_doc_snapshot(doc_id, snapshot_id)
            .await
            .map_err(|err| Status::internal(err.to_string()))?
            .ok_or(Status::not_found("Snapshot not found"))?;

        Ok(Response::new(to_doc_snapshot(snapshot)))
    }

    async fn get_doc_latestsnapshots(
        &self,
        request: Request<GetDocLatestsnapshotsRequest>,
    ) -> Result<Response<GetDocLatestsnapshotsResponse>, Status> {
        let req = request.into_inner();
        let doc_ids = req
            .doc_ids
            .iter()
            .map(|id| Uuid::parse_str(id))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| Status::invalid_argument("Invalid doc_id"))?;
        let snapshots = self
            .state
            .document_service
            .get_latest_snapshots(doc_ids)
            .await
            .map_err(|err| Status::internal(err.to_string()))?;

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
        let storage_ids = req
            .storage_ids
            .iter()
            .map(|id| Uuid::parse_str(id))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| Status::invalid_argument("Invalid storage_id"))?;

        let urls = self
            .state
            .document_service
            .sign_object_urls(storage_ids)
            .await
            .map_err(|err| Status::internal(err.to_string()))?;
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
        let storage_id = Uuid::parse_str(&req.storage_id)
            .map_err(|_| Status::invalid_argument("Invalid storage_id"))?;
        let doc_id = self
            .state
            .document_service
            .get_doc_id_by_storage_id(storage_id)
            .await
            .map_err(|err| Status::internal(err.to_string()))?
            .ok_or(Status::not_found("Document not found"))?;
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
    tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(OT_RPC_DESCRIPTOR_SET)
        .build()
        .expect("Failed to build gRPC reflection service")
}

fn to_doc_snapshot(snapshot: crate::server::services::document::DocumentSnapshotInfo) -> DocSnapshot {
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
