use crate::server::services::ot::Changeset;
use crate::server::services::{DocumentService, OTService};
use crate::server::state::AppState;
use crate::server::types::{
    ChangesetAck, ChangesetPushed, ChangesetRequest, FetchOpsAck, FetchOpsRequest, JoinDocAck,
    JoinDocRequest, LeaveDocRequest, OperationInfo, PresenceUpdateRequest,
};
use anyhow::Result;
use socketioxide::{
    extract::{AckSender, Data, SocketRef},
    SocketIo,
};
use tracing::{error, info, warn};
use uuid::Uuid;

/// Setup Socket.IO event handlers
pub fn setup_socketio(io: &SocketIo, state: AppState) {
    // Setup handlers for /ws namespace
    let state_clone = state.clone();
    io.ns("/ws", move |socket: SocketRef| async move {
        info!("Socket connected to /ws namespace: {:?}", socket.id);

        let state = state_clone.clone();

        // Handle join_doc event
        socket.on(
            "join_doc",
            move |socket: SocketRef, Data::<JoinDocRequest>(req), ack: AckSender| {
                let state = state.clone();
                async move {
                    match handle_join_doc(&socket, &state, req).await {
                        Ok(ack_data) => {
                            if let Ok(json) = serde_json::to_value(&ack_data) {
                                let _ = ack.send(&json);
                            }
                        }
                        Err(e) => {
                            error!("Error joining doc: {}", e);
                            let error_ack = JoinDocAck {
                                status: "error".to_string(),
                                version: None,
                                content: None,
                                message: Some(e.to_string()),
                            };
                            if let Ok(json) = serde_json::to_value(&error_ack) {
                                let _ = ack.send(&json);
                            }
                        }
                    }
                }
            },
        );

        // Handle leave_doc event
        socket.on(
            "leave_doc",
            |socket: SocketRef, Data::<LeaveDocRequest>(req)| async move {
                handle_leave_doc(&socket, req).await;
            },
        );

        let state = state_clone.clone();
        // Handle changeset event
        socket.on(
            "changeset",
            move |socket: SocketRef, Data::<ChangesetRequest>(req), ack: AckSender| {
                let state = state.clone();
                async move {
                    info!(
                        "·: socket_id={:?}, doc_id={}, base_rev={}",
                        socket.id, req.doc_id, req.base_rev
                    );
                    match handle_changeset(&socket, &state, req).await {
                        Ok(ack_data) => {
                            if let Ok(json) = serde_json::to_value(&ack_data) {
                                let _ = ack.send(&json);
                            }
                        }
                        Err(e) => {
                            error!("Error applying changeset: {}", e);
                            let error_ack = ChangesetAck {
                                status: "error".to_string(),
                                server_rev: None,
                                mutations: None,
                                message: Some(e.to_string()),
                            };
                            if let Ok(json) = serde_json::to_value(&error_ack) {
                                let _ = ack.send(&json);
                            }
                        }
                    }
                }
            },
        );

        let state = state_clone.clone();
        // Handle fetch_ops event
        socket.on(
            "fetch_ops",
            move |socket: SocketRef, Data::<FetchOpsRequest>(req), ack: AckSender| {
                let state = state.clone();
                async move {
                    match handle_fetch_ops(&socket, &state, req).await {
                        Ok(ack_data) => {
                            if let Ok(json) = serde_json::to_value(&ack_data) {
                                let _ = ack.send(&json);
                            }
                        }
                        Err(e) => {
                            error!("Error fetching ops: {}", e);
                            let error_ack = FetchOpsAck {
                                status: "error".to_string(),
                                operations: None,
                                message: Some(e.to_string()),
                            };
                            if let Ok(json) = serde_json::to_value(&error_ack) {
                                let _ = ack.send(&json);
                            }
                        }
                    }
                }
            },
        );

        // Handle presence_update event
        socket.on(
            "presence_update",
            |socket: SocketRef, Data::<PresenceUpdateRequest>(req)| async move {
                handle_presence_update(&socket, req).await;
            },
        );
    });
}

async fn handle_join_doc(
    socket: &SocketRef,
    state: &AppState,
    req: JoinDocRequest,
) -> Result<JoinDocAck> {
    let doc_id =
        Uuid::parse_str(&req.doc_id).map_err(|e| anyhow::anyhow!("Invalid doc_id: {}", e))?;

    // Verify document exists and get current version from documents table
    let version = state
        .document_service
        .get_current_version(doc_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Document not found: {}", req.doc_id))?;

    // Join the room
    let room = format!("doc:{}", req.doc_id);
    socket.join(room.clone());
    info!("Socket {} joined room {}", socket.id, room);

    Ok(JoinDocAck {
        status: "ok".to_string(),
        version: Some(version),
        content: None, // Don't return content - client should load from API or use existing local state
        message: None,
    })
}

async fn handle_leave_doc(socket: &SocketRef, req: LeaveDocRequest) {
    let room = format!("doc:{}", req.doc_id);
    socket.leave(room.clone());
    info!("Socket {} left room {}", socket.id, room);
}

async fn handle_changeset(
    socket: &SocketRef,
    state: &AppState,
    req: ChangesetRequest,
) -> Result<ChangesetAck> {
    info!(
        "Handling changeset: doc_id={}, base_rev={}, mutations_count={}",
        req.doc_id,
        req.base_rev,
        req.mutations.len()
    );
    let doc_id =
        Uuid::parse_str(&req.doc_id).map_err(|e| anyhow::anyhow!("Invalid doc_id: {}", e))?;

    // Convert ChangesetRequest to Changeset for OTService
    let changeset = Changeset {
        base_rev: req.base_rev,
        user_id: socket.id.to_string(), // TODO: 后面换成auth token中的userId
        mutations: req.mutations.clone(),
        client_id: format!("socket:{}", socket.id), // Use socket ID as client ID
    };

    // Apply changeset via OTService
    let result = state
        .document_actor_manager
        .apply_changeset(doc_id, changeset, req.client_msg_id)
        .await?;

    info!(
        "Changeset applied successfully: doc_id={}, server_rev={}, mutations_count={}",
        req.doc_id,
        result.server_rev,
        result.mutations.len()
    );

    // Broadcast to room (excluding sender)
    let room = format!("doc:{}", req.doc_id);
    let pushed = ChangesetPushed {
        doc_id: req.doc_id.clone(),
        server_rev: result.server_rev,
        user_id: result.user_id.clone(),
        mutations: result.mutations.clone(),
    };

    match serde_json::to_value(&pushed) {
        Ok(json) => {
            match socket
                .to(room.clone())
                .emit("changeset_pushed", &json)
                .await
            {
                Ok(_) => {
                    info!(
                        "Broadcasted changeset_pushed to room {}: server_rev={}",
                        room, result.server_rev
                    );
                }
                Err(e) => {
                    error!(
                        "Failed to broadcast changeset_pushed to room {}: {}",
                        room, e
                    );
                }
            }
        }
        Err(e) => {
            error!(
                "Failed to serialize changeset_pushed for room {}: {}",
                room, e
            );
        }
    }

    Ok(ChangesetAck {
        status: "ok".to_string(),
        server_rev: Some(result.server_rev),
        // Return the transformed mutations so client can verify/sync with server state
        mutations: Some(result.mutations.clone()),
        message: None,
    })
}

async fn handle_fetch_ops(
    _socket: &SocketRef,
    state: &AppState,
    req: FetchOpsRequest,
) -> Result<FetchOpsAck> {
    let doc_id =
        Uuid::parse_str(&req.doc_id).map_err(|e| anyhow::anyhow!("Invalid doc_id: {}", e))?;

    // Get operations since start_rev
    let ops = state
        .document_service
        .get_operations_since(doc_id, req.start_rev)
        .await?;

    // Group operations by revision
    use std::collections::HashMap;
    let mut ops_by_rev: HashMap<i64, Vec<_>> = HashMap::new();
    for op in ops {
        ops_by_rev.entry(op.rev).or_insert_with(Vec::new).push(op);
    }

    // Convert to OperationInfo format (grouped by rev)
    let mut operations: Vec<OperationInfo> = ops_by_rev
        .into_iter()
        .map(|(rev, ops_for_rev)| {
            let user_id = ops_for_rev[0].user_id.clone();
            let mutations: Vec<_> = ops_for_rev
                .into_iter()
                .map(|op| crate::types::MutationInfoInternal {
                    id: op.mutation_id,
                    params: op.params,
                })
                .collect();
            OperationInfo {
                rev,
                user_id,
                mutations,
            }
        })
        .collect();
    operations.sort_by_key(|op| op.rev);

    Ok(FetchOpsAck {
        status: "ok".to_string(),
        operations: Some(operations),
        message: None,
    })
}

async fn handle_presence_update(socket: &SocketRef, req: PresenceUpdateRequest) {
    let room = format!("doc:{}", req.doc_id);
    // Broadcast presence update to room (excluding sender)
    if let Ok(json) = serde_json::to_value(&req) {
        if let Err(e) = socket.to(room).emit("presence_update", &json).await {
            warn!("Error broadcasting presence_update: {}", e);
        }
    }
}
