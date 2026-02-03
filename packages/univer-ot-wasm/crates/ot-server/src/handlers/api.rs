use crate::state::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use tracing::error;
use uuid::Uuid;

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
    pub to_rev: Option<i64>,
    /// Maximum number of operations to return.
    /// Defaults to DEFAULT_OPERATIONS_LIMIT if not specified.
    pub limit: Option<u64>,
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
    pub client_id: String,
    pub op_id: String,
    pub created_at: String,
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
pub async fn get_operations(
    State(state): State<AppState>,
    Path(doc_id): Path<String>,
    Query(query): Query<GetOperationsQuery>,
) -> Result<Json<OperationsResponse>, StatusCode> {
    let doc_uuid = Uuid::parse_str(&doc_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    let from_rev = query.from_rev.unwrap_or(0);
    // Use provided limit or default to DEFAULT_OPERATIONS_LIMIT for API calls
    let limit = query.limit.or(Some(crate::services::document::DocumentService::DEFAULT_OPERATIONS_LIMIT));
    let operations = state
        .document_service
        .get_operations(doc_uuid, from_rev, query.to_rev, limit)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let operation_responses: Vec<OperationResponse> = operations
        .into_iter()
        .map(|op| OperationResponse {
            rev: op.rev,
            user_id: op.user_id,
            mutation_id: op.mutation_id,
            params: op.params,
            client_id: op.client_id,
            op_id: op.op_id,
            created_at: op.created_at.to_rfc3339(),
        })
        .collect();

    Ok(Json(OperationsResponse {
        operations: operation_responses,
    }))
}

/// GET /health - Health check
pub async fn health_check() -> &'static str {
    "OK"
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
