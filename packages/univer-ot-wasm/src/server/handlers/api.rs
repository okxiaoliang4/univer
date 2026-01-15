use crate::server::state::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateDocumentRequest {
    pub doc_id: String,
    pub content: JsonValue,
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

#[derive(Debug, Serialize)]
pub struct DocumentResponse {
    pub doc_id: String,
    pub content: JsonValue,
    pub version: i64,
}

#[derive(Debug, Deserialize)]
pub struct GetOperationsQuery {
    pub from_rev: Option<i64>,
    pub to_rev: Option<i64>,
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
    pub client_msg_id: String,
    pub created_at: String,
}

/// POST /api/documents - Create a new document
pub async fn create_document(
    State(state): State<AppState>,
    Json(req): Json<CreateDocumentRequest>,
) -> Result<Json<CreateDocumentResponse>, StatusCode> {
    let doc_id = Uuid::parse_str(&req.doc_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    state
        .document_service
        .create_document(doc_id, req.content, None)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(CreateDocumentResponse {
        doc_id: req.doc_id,
        version: 0,
    }))
}

/// POST /api/documents/:doc_id/snapshot - Update document snapshot
pub async fn update_snapshot(
    State(state): State<AppState>,
    Path(doc_id): Path<String>,
    Json(req): Json<UpdateSnapshotRequest>,
) -> Result<StatusCode, StatusCode> {
    let doc_uuid = Uuid::parse_str(&doc_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    state
        .snapshot_service
        .update_snapshot(doc_uuid, req.content, req.version)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}

/// GET /api/documents/:doc_id - Get document
pub async fn get_document(
    State(state): State<AppState>,
    Path(doc_id): Path<String>,
) -> Result<Json<DocumentResponse>, StatusCode> {
    let doc_uuid = Uuid::parse_str(&doc_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let (content, version) = state
        .document_service
        .get_document(doc_uuid)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(DocumentResponse {
        doc_id,
        content,
        version,
    }))
}

/// GET /api/documents/:doc_id/operations - Get operation logs
pub async fn get_operations(
    State(state): State<AppState>,
    Path(doc_id): Path<String>,
    Query(query): Query<GetOperationsQuery>,
) -> Result<Json<OperationsResponse>, StatusCode> {
    let doc_uuid = Uuid::parse_str(&doc_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let from_rev = query.from_rev.unwrap_or(0);
    let operations = state
        .document_service
        .get_operations(doc_uuid, from_rev, query.to_rev)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let operation_responses: Vec<OperationResponse> = operations
        .into_iter()
        .map(|op| OperationResponse {
            rev: op.rev,
            user_id: op.user_id,
            mutation_id: op.mutation_id,
            params: op.params,
            client_msg_id: op.client_msg_id,
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
