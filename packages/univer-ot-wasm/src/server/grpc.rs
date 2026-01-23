use crate::server::state::AppState;
use crate::types::MutationInfoInternal;
use crate::types::MutationInfoWithOpId;
use serde_json::Value as JsonValue;
use tonic::{Request, Response, Status};
use uuid::Uuid;

pub mod ot_rpc {
    tonic::include_proto!("ot_rpc");
}

use ot_rpc::ot_rpc_service_server::{OtRpcService, OtRpcServiceServer};
use ot_rpc::{BroadcastOpRequest, BroadcastOpResponse};

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
