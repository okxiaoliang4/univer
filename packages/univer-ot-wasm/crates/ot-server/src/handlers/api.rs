use crate::services::auth::AuthUserInfo;
use crate::state::AppState;
use axum::{
    extract::{Path, Query, State},
    http::{StatusCode, request::Parts},
    response::Json,
};
use ot_core::MutationInfoWithOpId;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use tracing::{error, info, warn};
use uuid::Uuid;

// ─── Auth Extractor ─────────────────────────────────────────

/// Authenticated user extracted from `Authorization: Bearer <token>` header.
///
/// Implements `FromRequestParts` so it can be used as an Axum extractor.
/// When present, it verifies the token and extracts user info.
pub struct AuthUser {
    pub user_info: AuthUserInfo,
    pub token: String,
}

impl axum::extract::FromRequestParts<AppState> for AuthUser {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        let token = if let Some(token) = auth_header.strip_prefix("Bearer ") {
            token.to_string()
        } else {
            warn!("Missing or invalid Authorization header");
            return Err(StatusCode::UNAUTHORIZED);
        };

        let user_info = state
            .auth_service
            .verify_token(&token)
            .await
            .map_err(|e| {
                warn!("Token verification failed: {}", e);
                StatusCode::UNAUTHORIZED
            })?;

        Ok(AuthUser { user_info, token })
    }
}

// ─── Request/Response Types ─────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateDocumentRequest {
    pub doc_id: String,
    pub creator_id: String,
    pub name: String,
    pub doc_type: i16,
    pub create_type: i16,
    pub content: JsonValue,
    pub url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateDocumentResponse {
    pub doc_id: String,
    pub version: i64,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSnapshotRequest {
    pub content: JsonValue,
    pub version: i64,
}

#[derive(Debug, Deserialize)]
pub struct RestoreDocumentRequest {
    pub snapshot_id: String,
}

#[derive(Debug, Deserialize)]
pub struct GetDocSnapshotListQuery {
    pub limit: Option<i32>,
    pub cursor: Option<i64>,
    pub desc: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSnapshotNameRequest {
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct SnapshotListResponse {
    pub doc_id: String,
    pub snapshots: Vec<SnapshotResponse>,
    pub next_cursor: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct SnapshotResponse {
    pub id: String,
    pub doc_id: String,
    pub name: Option<String>,
    pub size: Option<i64>,
    pub storage_id: String,
    pub users: serde_json::Value,
    pub version: i64,
    pub created_at: String,
    pub updated_at: String,
    pub restore_from_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DocumentResponse {
    pub doc_id: String,
    pub signed_url: String,
    pub version: i64,
}

#[derive(Debug, Deserialize)]
pub struct GetOperationsQuery {
    pub from_rev: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct OperationsResponse {
    pub operations: Vec<OperationResponse>,
}

#[derive(Debug, Serialize)]
pub struct OperationResponse {
    pub rev: i64,
    pub user_id: String,
    pub mutation_id: String,
    pub params: JsonValue,
    pub op_id: String,
    pub created_at: String,
}

// === New request/response types for additional endpoints ===

#[derive(Debug, Deserialize)]
pub struct CloneDocumentRequest {
    pub creator_id: String,
    pub snapshot_id: Option<String>,
    pub doc_type: Option<i16>,
}

#[derive(Debug, Serialize)]
pub struct CloneDocumentResponse {
    pub doc_id: String,
    pub size: i64,
}

#[derive(Debug, Deserialize)]
pub struct DeleteDocumentQuery {
    pub is_soft: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct GetLatestSnapshotsRequest {
    pub doc_ids: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct GetLatestSnapshotsResponse {
    pub snapshots: std::collections::HashMap<String, SnapshotResponse>,
}

#[derive(Debug, Deserialize)]
pub struct SignObjectUrlsRequest {
    pub storage_ids: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct SignObjectUrlsResponse {
    pub urls: std::collections::HashMap<String, String>,
}

#[derive(Debug, Serialize)]
pub struct GetDocIdResponse {
    pub doc_id: String,
}

#[derive(Debug, Deserialize)]
pub struct BroadcastOpRequest {
    pub base_rev: i64,
    pub user_id: String,
    pub client_id: String,
    pub mutations: Vec<MutationInput>,
}

#[derive(Debug, Deserialize)]
pub struct MutationInput {
    pub id: String,
    pub params: JsonValue,
    pub op_id: String,
}

#[derive(Debug, Serialize)]
pub struct BroadcastOpResponse {
    pub success: bool,
    pub server_rev: i64,
    pub op_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Redis broadcast message for ws-gateway
#[derive(Debug, Serialize)]
struct ChangesetPushedBroadcast {
    #[serde(rename = "type")]
    msg_type: String,
    #[serde(rename = "docId")]
    doc_id: String,
    #[serde(rename = "serverRev")]
    server_rev: i64,
    #[serde(rename = "userId")]
    user_id: String,
}

/// POST /api/documents - Create a new document
pub async fn create_document(
    State(state): State<AppState>,
    Json(req): Json<CreateDocumentRequest>,
) -> Result<Json<CreateDocumentResponse>, StatusCode> {
    let doc_id = Uuid::parse_str(&req.doc_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    let result = if let Some(url) = req.url {
        state
            .document_service
            .create_document_from_url(
                doc_id,
                req.creator_id,
                req.name,
                req.doc_type,
                req.create_type,
                url,
            )
            .await
    } else {
        state
            .document_service
            .create_document(
                doc_id,
                req.creator_id,
                req.name,
                req.doc_type,
                req.create_type,
                req.content,
            )
            .await
    };

    if let Err(err) = result {
        error!(?err, "Failed to create document");
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    Ok(Json(CreateDocumentResponse {
        doc_id: req.doc_id,
        version: 0,
    }))
}

/// POST /api/documents/:doc_id/restore - Restore document snapshot
pub async fn restore_document(
    State(state): State<AppState>,
    Path(doc_id): Path<String>,
    Json(req): Json<RestoreDocumentRequest>,
) -> Result<Json<SnapshotResponse>, StatusCode> {
    let doc_uuid = Uuid::parse_str(&doc_id).map_err(|_| StatusCode::BAD_REQUEST)?;
    let snapshot_id = Uuid::parse_str(&req.snapshot_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    let snapshot = state
        .document_service
        .restore_document(doc_uuid, snapshot_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(to_snapshot_response(snapshot)))
}

/// GET /api/documents/:doc_id/snapshots - List snapshots
pub async fn get_snapshot_list(
    State(state): State<AppState>,
    Path(doc_id): Path<String>,
    Query(query): Query<GetDocSnapshotListQuery>,
) -> Result<Json<SnapshotListResponse>, StatusCode> {
    let doc_uuid = Uuid::parse_str(&doc_id).map_err(|_| StatusCode::BAD_REQUEST)?;
    let (snapshots, next_cursor) = state
        .document_service
        .list_snapshots(
            doc_uuid,
            query.limit.unwrap_or(10),
            query.cursor,
            query.desc.unwrap_or(true),
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let items = snapshots.into_iter().map(to_snapshot_response).collect();

    Ok(Json(SnapshotListResponse {
        doc_id,
        snapshots: items,
        next_cursor,
    }))
}

/// PATCH /api/documents/:doc_id/snapshots/:snapshot_id - Update snapshot name
pub async fn update_snapshot_name(
    State(state): State<AppState>,
    Path((doc_id, snapshot_id)): Path<(String, String)>,
    Json(req): Json<UpdateSnapshotNameRequest>,
) -> Result<StatusCode, StatusCode> {
    let doc_uuid = Uuid::parse_str(&doc_id).map_err(|_| StatusCode::BAD_REQUEST)?;
    let snapshot_uuid = Uuid::parse_str(&snapshot_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    state
        .document_service
        .update_snapshot_name(doc_uuid, snapshot_uuid, req.name)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}

/// GET /api/documents/:doc_id - Get document
pub async fn get_document(
    State(state): State<AppState>,
    Path(doc_id): Path<String>,
) -> Result<Json<DocumentResponse>, StatusCode> {
    let doc_uuid = Uuid::parse_str(&doc_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    let (storage_id, version) = state
        .document_service
        .get_document(doc_uuid)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    tracing::info!(?storage_id, ?version, "get_document response");

    let storage = state
        .storage_service
        .get_storage_location(storage_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    tracing::info!(?storage, "get_storage_location response");

    let signed_url = state
        .storage_service
        .get_signed_url(&storage)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    tracing::info!(?signed_url, "get_signed_url response");

    Ok(Json(DocumentResponse {
        doc_id,
        signed_url,
        version,
    }))
}

/// GET /api/documents/:doc_id/operations - Get operation logs
///
/// Requires authentication. Checks readable permission.
/// Uses three-tier cache: L1 (LocalCache) → L2/L3 (Redis/DB via DocumentService).
pub async fn get_operations(
    State(state): State<AppState>,
    Path(doc_id): Path<String>,
    auth: AuthUser,
    Query(query): Query<GetOperationsQuery>,
) -> Result<Json<OperationsResponse>, StatusCode> {
    let doc_uuid = Uuid::parse_str(&doc_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    // Check readable permission
    let permissions = state
        .auth_service
        .check_document_permission(
            "http",
            &auth.user_info.uid,
            &doc_id,
            &auth.token,
        )
        .await
        .map_err(|e| {
            warn!("Permission check failed: {}", e);
            StatusCode::FORBIDDEN
        })?;

    if !permissions.readable {
        return Err(StatusCode::FORBIDDEN);
    }

    let from_rev = query.from_rev.unwrap_or(0);

    // === L1: Try LocalCache (same instance used by OTService write-through) ===
    if let Some(local_ops) = state.local_cache.get_ops_since(doc_uuid, from_rev).await {
        // Validate coverage against authoritative version from Redis
        if let Ok(Some(current_version)) = state.cache_service.get_version(doc_uuid).await {
            let first_rev = local_ops.first().map(|o| o.rev).unwrap_or(i64::MAX);
            let last_rev = local_ops.last().map(|o| o.rev).unwrap_or(0);
            if first_rev <= from_rev + 1 && last_rev >= current_version {
                // L1 has complete coverage — decode and return directly
                let mut operation_responses = Vec::with_capacity(local_ops.len());
                for op in local_ops {
                    let params = crate::services::params_codec::decode_params(&op.params)
                        .map_err(|e| {
                            error!("Failed to decode cached params: rev={}, error={}", op.rev, e);
                            StatusCode::INTERNAL_SERVER_ERROR
                        })?;
                    operation_responses.push(OperationResponse {
                        rev: op.rev,
                        user_id: op.user_id,
                        mutation_id: op.mutation_id,
                        params,
                        op_id: op.op_id,
                        created_at: op.created_at.to_rfc3339(),
                    });
                }
                crate::metrics::increment_local_cache_hits();
                return Ok(Json(OperationsResponse {
                    operations: operation_responses,
                }));
            }
        }
        // L1 incomplete or version check failed — fall through
    }

    // === L2/L3: Redis ops cache → DB (via DocumentService) ===
    let operations = state
        .document_service
        .get_operations_since(doc_uuid, from_rev)
        .await
        .map_err(|e| {
            error!("Failed to get operations: doc_id={}, error={}", doc_id, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let operation_responses: Vec<OperationResponse> = operations
        .into_iter()
        .map(|op| OperationResponse {
            rev: op.rev,
            user_id: op.user_id,
            mutation_id: op.mutation_id,
            params: op.params,
            op_id: op.op_id,
            created_at: op.created_at.to_rfc3339(),
        })
        .collect();

    Ok(Json(OperationsResponse {
        operations: operation_responses,
    }))
}

/// POST /api/documents/:doc_id/clone - Clone a document
pub async fn clone_document(
    State(state): State<AppState>,
    Path(doc_id): Path<String>,
    Json(req): Json<CloneDocumentRequest>,
) -> Result<Json<CloneDocumentResponse>, StatusCode> {
    info!(
        "clone_document request: doc_id={}, creator_id={}, snapshot_id={:?}",
        doc_id, req.creator_id, req.snapshot_id
    );

    let doc_uuid = Uuid::parse_str(&doc_id).map_err(|e| {
        error!("Invalid doc_id: {} - {}", doc_id, e);
        StatusCode::BAD_REQUEST
    })?;

    let snapshot_id = req
        .snapshot_id
        .map(|s| Uuid::parse_str(&s))
        .transpose()
        .map_err(|e| {
            error!("Invalid snapshot_id: {}", e);
            StatusCode::BAD_REQUEST
        })?;

    let (new_doc_id, size) = state
        .document_service
        .clone_document(doc_uuid, req.creator_id, snapshot_id, req.doc_type)
        .await
        .map_err(|e| {
            error!("Failed to clone document: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    info!(
        "Document cloned successfully: source={}, new_doc_id={}, size={}",
        doc_id, new_doc_id, size
    );

    Ok(Json(CloneDocumentResponse {
        doc_id: new_doc_id.to_string(),
        size,
    }))
}

/// DELETE /api/documents/:doc_id - Delete a document
pub async fn delete_document(
    State(state): State<AppState>,
    Path(doc_id): Path<String>,
    Query(query): Query<DeleteDocumentQuery>,
) -> Result<StatusCode, StatusCode> {
    let is_soft = query.is_soft.unwrap_or(false);
    info!("delete_document request: doc_id={}, is_soft={}", doc_id, is_soft);

    let doc_uuid = Uuid::parse_str(&doc_id).map_err(|e| {
        error!("Invalid doc_id: {} - {}", doc_id, e);
        StatusCode::BAD_REQUEST
    })?;

    state
        .document_service
        .delete_document(doc_uuid, is_soft)
        .await
        .map_err(|e| {
            error!("Failed to delete document: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    info!("Document deleted successfully: doc_id={}, is_soft={}", doc_id, is_soft);
    Ok(StatusCode::OK)
}

/// GET /api/documents/:doc_id/snapshots/:snapshot_id - Get a specific snapshot
pub async fn get_snapshot(
    State(state): State<AppState>,
    Path((doc_id, snapshot_id)): Path<(String, String)>,
) -> Result<Json<SnapshotResponse>, StatusCode> {
    info!(
        "get_snapshot request: doc_id={}, snapshot_id={}",
        doc_id, snapshot_id
    );

    let doc_uuid = Uuid::parse_str(&doc_id).map_err(|e| {
        error!("Invalid doc_id: {} - {}", doc_id, e);
        StatusCode::BAD_REQUEST
    })?;

    let snapshot_uuid = Uuid::parse_str(&snapshot_id).map_err(|e| {
        error!("Invalid snapshot_id: {} - {}", snapshot_id, e);
        StatusCode::BAD_REQUEST
    })?;

    let snapshot = state
        .document_service
        .get_doc_snapshot(doc_uuid, Some(snapshot_uuid))
        .await
        .map_err(|e| {
            error!("Failed to get snapshot: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or_else(|| {
            warn!("Snapshot not found: doc_id={}, snapshot_id={}", doc_id, snapshot_id);
            StatusCode::NOT_FOUND
        })?;

    Ok(Json(to_snapshot_response(snapshot)))
}

/// POST /api/documents/latest-snapshots - Get latest snapshots for multiple documents
pub async fn get_latest_snapshots(
    State(state): State<AppState>,
    Json(req): Json<GetLatestSnapshotsRequest>,
) -> Result<Json<GetLatestSnapshotsResponse>, StatusCode> {
    info!(
        "get_latest_snapshots request: doc_ids_count={}",
        req.doc_ids.len()
    );

    let doc_ids: Vec<Uuid> = req
        .doc_ids
        .iter()
        .map(|id| Uuid::parse_str(id))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| {
            error!("Invalid doc_id in request: {}", e);
            StatusCode::BAD_REQUEST
        })?;

    let snapshots = state
        .document_service
        .get_latest_snapshots(doc_ids.clone())
        .await
        .map_err(|e| {
            error!("Failed to get latest snapshots: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    info!(
        "Retrieved latest snapshots: requested={}, returned={}",
        doc_ids.len(),
        snapshots.len()
    );

    let snapshot_map = snapshots
        .into_iter()
        .map(|(doc_id, snapshot)| (doc_id.to_string(), to_snapshot_response(snapshot)))
        .collect();

    Ok(Json(GetLatestSnapshotsResponse {
        snapshots: snapshot_map,
    }))
}

/// POST /api/sign-urls - Sign object URLs for storage IDs
pub async fn sign_object_urls(
    State(state): State<AppState>,
    Json(req): Json<SignObjectUrlsRequest>,
) -> Result<Json<SignObjectUrlsResponse>, StatusCode> {
    info!(
        "sign_object_urls request: storage_ids_count={}",
        req.storage_ids.len()
    );

    let storage_ids: Vec<Uuid> = req
        .storage_ids
        .iter()
        .map(|id| Uuid::parse_str(id))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| {
            error!("Invalid storage_id in request: {}", e);
            StatusCode::BAD_REQUEST
        })?;

    let urls = state
        .document_service
        .sign_object_urls(storage_ids.clone())
        .await
        .map_err(|e| {
            error!("Failed to sign object URLs: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    info!(
        "Signed object URLs: requested={}, returned={}",
        storage_ids.len(),
        urls.len()
    );

    let url_map = urls
        .into_iter()
        .map(|(id, url)| (id.to_string(), url))
        .collect();

    Ok(Json(SignObjectUrlsResponse { urls: url_map }))
}

/// GET /api/storage/:storage_id/doc-id - Get document ID from storage ID
pub async fn get_doc_id_from_storage_id(
    State(state): State<AppState>,
    Path(storage_id): Path<String>,
) -> Result<Json<GetDocIdResponse>, StatusCode> {
    info!("get_doc_id_from_storage_id request: storage_id={}", storage_id);

    let storage_uuid = Uuid::parse_str(&storage_id).map_err(|e| {
        error!("Invalid storage_id: {} - {}", storage_id, e);
        StatusCode::BAD_REQUEST
    })?;

    let doc_id = state
        .document_service
        .get_doc_id_by_storage_id(storage_uuid)
        .await
        .map_err(|e| {
            error!("Failed to get doc_id: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or_else(|| {
            warn!("Document not found for storage_id: {}", storage_id);
            StatusCode::NOT_FOUND
        })?;

    info!(
        "Found doc_id for storage_id: storage_id={}, doc_id={}",
        storage_id, doc_id
    );

    Ok(Json(GetDocIdResponse {
        doc_id: doc_id.to_string(),
    }))
}

/// POST /api/documents/:doc_id/changeset - Apply a changeset (broadcast operation)
///
/// Requires authentication. Uses token-extracted user_id (not request body).
/// After successful OT, publishes to Redis Pub/Sub for ws-gateway broadcast
/// and sends rate-limited gRPC notification.
pub async fn broadcast_op(
    State(state): State<AppState>,
    Path(doc_id): Path<String>,
    auth: AuthUser,
    Json(req): Json<BroadcastOpRequest>,
) -> Result<(StatusCode, Json<BroadcastOpResponse>), StatusCode> {
    use std::time::Instant;
    let request_start = Instant::now();

    // Use user_id from verified token, not from request body (prevents spoofing)
    let user_id = auth.user_info.uid.clone();

    info!(
        "changeset START: doc_id={}, base_rev={}, user_id={}, mutations_count={}",
        doc_id, req.base_rev, user_id, req.mutations.len()
    );

    let doc_uuid = Uuid::parse_str(&doc_id).map_err(|e| {
        error!("Invalid doc_id: {} - {}", doc_id, e);
        StatusCode::BAD_REQUEST
    })?;

    // === PHASE 1: Permission Check ===
    let perm_start = Instant::now();
    let permissions = state
        .auth_service
        .check_document_permission(
            "http",
            &user_id,
            &doc_id,
            &auth.token,
        )
        .await
        .map_err(|e| {
            warn!("Permission check failed for broadcast_op: {}", e);
            StatusCode::FORBIDDEN
        })?;

    if !permissions.writable {
        warn!("Permission denied: user {} cannot write to document {}", user_id, doc_id);
        return Err(StatusCode::FORBIDDEN);
    }
    let perm_elapsed = perm_start.elapsed();
    info!("changeset [permission_check] doc_id={}, elapsed={:.1}ms", doc_id, perm_elapsed.as_secs_f64() * 1000.0);

    // === PHASE 2: Data Preparation ===
    let prep_start = Instant::now();
    let mutations: Vec<MutationInfoWithOpId> = req
        .mutations
        .into_iter()
        .map(|m| MutationInfoWithOpId {
            id: m.id,
            params: m.params,
            op_id: m.op_id,
        })
        .collect();

    let changeset = crate::services::ot::Changeset {
        base_rev: req.base_rev,
        user_id: user_id.clone(),
        mutations,
        client_id: req.client_id.clone(),
    };
    let prep_elapsed = prep_start.elapsed();
    info!("changeset [data_preparation] doc_id={}, elapsed={:.1}ms", doc_id, prep_elapsed.as_secs_f64() * 1000.0);

    // === PHASE 3: Apply Changeset (OT Transform + DB Write) ===
    let apply_start = Instant::now();
    let result = state
        .document_actor_manager
        .apply_changeset(doc_uuid, changeset)
        .await;

    match result {
        Ok(result) => {
            let apply_elapsed = apply_start.elapsed();
            info!("changeset [apply_changeset] doc_id={}, server_rev={}, elapsed={:.1}ms",
                  doc_id, result.server_rev, apply_elapsed.as_secs_f64() * 1000.0);

            let total_elapsed = request_start.elapsed();
            info!(
                "changeset DONE: doc_id={}, server_rev={}, total={:.1}ms (perm={:.1}ms, prep={:.1}ms, apply={:.1}ms)",
                doc_id, result.server_rev,
                total_elapsed.as_secs_f64() * 1000.0,
                perm_elapsed.as_secs_f64() * 1000.0,
                prep_elapsed.as_secs_f64() * 1000.0,
                apply_elapsed.as_secs_f64() * 1000.0
            );

            // Publish changeset_pushed to Redis for ws-gateway broadcast (fire-and-forget)
            let broadcast_msg = ChangesetPushedBroadcast {
                msg_type: "changeset_pushed".to_string(),
                doc_id: doc_id.clone(),
                server_rev: result.server_rev,
                user_id: user_id.clone(),
            };
            let redis_broadcast = state.redis_broadcast.clone();
            let doc_id_for_notify = doc_id.clone();
            let user_id_for_notify = user_id.clone();
            let grpc_client = state.grpc_client.clone();
            let debouncer = state.notify_debouncer.clone();

            tokio::spawn(async move {
                // Publish to Redis Pub/Sub
                if let Ok(payload) = serde_json::to_string(&broadcast_msg) {
                    redis_broadcast.publish(&payload).await;
                }

                // Rate-limited gRPC notification (at most once per 5s per doc)
                if debouncer.should_notify(&doc_id_for_notify) {
                    if let Err(e) = grpc_client
                        .notify_modify_document(&user_id_for_notify, &doc_id_for_notify)
                        .await
                    {
                        warn!(
                            "Failed to notify document modification: doc_id={}, error={}",
                            doc_id_for_notify, e
                        );
                    }
                }
            });

            Ok((StatusCode::OK, Json(BroadcastOpResponse {
                success: true,
                server_rev: result.server_rev,
                op_ids: result.op_ids,
                error: None,
            })))
        }
        Err(e) => {
            let error_msg = e.to_string();
            let (status, client_msg) = classify_changeset_error(&error_msg);
            warn!("Changeset failed: doc_id={}, status={}, error={}", doc_id, status.as_u16(), error_msg);
            Ok((status, Json(BroadcastOpResponse {
                success: false,
                server_rev: -1,
                op_ids: vec![],
                error: Some(client_msg),
            })))
        }
    }
}

/// GET /health - Health check
pub async fn health_check() -> &'static str {
    "OK"
}

/// Classify changeset errors into appropriate HTTP status codes.
///
/// Maps internal OT error strings to client-meaningful status codes:
/// - 409 Conflict: version drift / mismatch (client should re-fetch document)
/// - 503 Service Unavailable: lock timeout / Redis failure (client should retry)
/// - 422 Unprocessable Entity: transform error (client should report)
/// - 404 Not Found: document not found
/// - 500 Internal Server Error: unexpected errors
fn classify_changeset_error(error_msg: &str) -> (StatusCode, String) {
    if error_msg.starts_with("VERSION_DRIFT_TOO_LARGE:") {
        (StatusCode::CONFLICT, error_msg.to_string())
    } else if error_msg.contains("Version mismatch") {
        (StatusCode::CONFLICT, error_msg.to_string())
    } else if error_msg.contains("Failed to acquire document lock") {
        (StatusCode::SERVICE_UNAVAILABLE, "Server busy, please retry".to_string())
    } else if error_msg.starts_with("Transform error") {
        (StatusCode::UNPROCESSABLE_ENTITY, error_msg.to_string())
    } else if error_msg.contains("Redis cache write failed") {
        (StatusCode::SERVICE_UNAVAILABLE, "Temporary storage error".to_string())
    } else if error_msg.contains("Document not found") {
        (StatusCode::NOT_FOUND, "Document not found".to_string())
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
    }
}

fn to_snapshot_response(snapshot: crate::services::document::DocumentSnapshotInfo) -> SnapshotResponse {
    SnapshotResponse {
        id: snapshot.id.to_string(),
        doc_id: snapshot.doc_id.to_string(),
        name: snapshot.name,
        size: snapshot.size,
        storage_id: snapshot.storage_id.to_string(),
        users: serde_json::to_value(snapshot.users).unwrap_or_default(),
        version: snapshot.version,
        created_at: snapshot.created_at.to_rfc3339(),
        updated_at: snapshot.updated_at.to_rfc3339(),
        restore_from_id: snapshot.restore_from_id.map(|id| id.to_string()),
    }
}
