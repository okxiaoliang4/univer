use crate::metrics;
use crate::services::ot::Changeset;
use crate::state::AppState;
use crate::types::{
    AwarenessInitAck, AwarenessStateItem, ChangesetAck, ChangesetPushed, ChangesetRequest,
    FetchOpsAck, FetchOpsRequest, JoinDocAck, JoinDocRequest, LeaveDocRequest, OperationInfo,
    PresenceUpdateRequest,
};
use ot_core::MutationInfoWithOpId;
use anyhow::Result;
use dashmap::DashMap;
use socketioxide::{
    extract::{AckSender, Data, SocketRef},
    SocketIo,
};
use std::collections::HashSet;
use std::sync::OnceLock;
use tracing::{error, info, warn};
use uuid::Uuid;

/// Track which sockets are in which document rooms
/// Key: doc_id, Value: set of socket IDs
static DOCUMENT_SOCKETS: OnceLock<DashMap<String, HashSet<String>>> = OnceLock::new();

fn get_document_sockets() -> &'static DashMap<String, HashSet<String>> {
    DOCUMENT_SOCKETS.get_or_init(|| DashMap::new())
}

/// Track a socket joining a document
/// Returns true if this is a new document (first socket)
fn track_socket_join(doc_id: &str, socket_id: &str) -> bool {
    let doc_sockets = get_document_sockets();
    let mut entry = doc_sockets.entry(doc_id.to_string()).or_insert_with(HashSet::new);
    let was_empty = entry.is_empty();
    entry.insert(socket_id.to_string());
    was_empty
}

/// Track a socket leaving a document
/// Returns true if this was the last socket (document now empty)
fn track_socket_leave(doc_id: &str, socket_id: &str) -> bool {
    let doc_sockets = get_document_sockets();
    if let Some(mut entry) = doc_sockets.get_mut(doc_id) {
        entry.remove(socket_id);
        let is_now_empty = entry.is_empty();
        drop(entry);

        if is_now_empty {
            doc_sockets.remove(doc_id);
            return true;
        }
    }
    false
}

/// Setup Socket.IO event handlers
pub fn setup_socketio(io: &SocketIo, state: AppState) {
    // Setup handlers for /ws namespace
    let state_clone = state.clone();
    io.ns("/ws", move |socket: SocketRef| async move {
        info!("Socket connected to /ws namespace: {:?}", socket.id);

        // Increment online users counter
        metrics::increment_online_users();

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

        // Handle awareness_init event
        let state = state_clone.clone();
        socket.on(
            "awareness_init",
            move |socket: SocketRef, Data::<JoinDocRequest>(req), ack: AckSender| {
                let state = state.clone();
                async move {
                    match handle_awareness_init(&socket, &state, req).await {
                        Ok(ack_data) => {
                            if let Ok(json) = serde_json::to_value(&ack_data) {
                                let _ = ack.send(&json);
                            }
                        }
                        Err(e) => {
                            error!("Error initializing awareness: {}", e);
                            let error_ack = AwarenessInitAck {
                                status: "error".to_string(),
                                states: vec![],
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
        let state = state_clone.clone();
        socket.on(
            "presence_update",
            move |socket: SocketRef, Data::<PresenceUpdateRequest>(req)| {
                let state = state.clone();
                async move {
                    if let Err(e) = handle_presence_update(&socket, &state, req).await {
                        warn!("Error handling presence_update: {}", e);
                    }
                }
            },
        );

        let state = state_clone.clone();
        socket.on_disconnect(move |socket: SocketRef| {
            let state = state.clone();
            async move {
                handle_disconnect(&socket, &state).await;
            }
        });
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

    // Track socket joining document
    let is_new_document = track_socket_join(&req.doc_id, &socket.id.to_string());

    // If this is the first socket in this document, increment online documents
    if is_new_document {
        metrics::increment_online_documents();
        info!("Document {} now online (first user)", req.doc_id);
    }

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

    // Track socket leaving document
    let is_document_empty = track_socket_leave(&req.doc_id, &socket.id.to_string());

    // If this was the last socket in this document, decrement online documents
    if is_document_empty {
        metrics::decrement_online_documents();
        info!("Document {} now offline (last user left)", req.doc_id);
    }
}

async fn handle_awareness_init(
    socket: &SocketRef,
    state: &AppState,
    req: JoinDocRequest,
) -> Result<AwarenessInitAck> {
    let doc_id = req.doc_id.clone();
    let room = format!("doc:{}", doc_id);
    socket.join(room.clone());

    // Track socket joining document
    let is_new_document = track_socket_join(&doc_id, &socket.id.to_string());

    // If this is the first socket in this document, increment online documents
    if is_new_document {
        metrics::increment_online_documents();
        info!("Document {} now online (first user via awareness)", doc_id);
    }

    let snapshot = state.awareness_service.get_state(&doc_id).await?;
    let states = snapshot.into_values().collect::<Vec<_>>();

    Ok(AwarenessInitAck {
        status: "ok".to_string(),
        states,
    })
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
    let mutations: Vec<MutationInfoWithOpId> = req.mutations;

    let changeset = Changeset {
        base_rev: req.base_rev,
        user_id: socket.id.to_string(), // TODO: 后面换成auth token中的userId
        mutations,
        client_id: req
            .client_id
            .clone()
            .unwrap_or_else(|| format!("socket:{}", socket.id)),
    };

    // Apply changeset via OTService
    let result = state
        .document_actor_manager
        .apply_changeset(doc_id, changeset)
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
                .map(|op| ot_core::MutationInfo {
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

async fn handle_presence_update(
    socket: &SocketRef,
    state: &AppState,
    req: PresenceUpdateRequest,
) -> Result<()> {
    let doc_id = req.doc_id.clone();
    let client_id = req.client_id.unwrap_or_else(|| {
        socket
            .id
            .to_string()
            .chars()
            .fold(0u64, |acc, c| acc + c as u64)
    });

    let user = req
        .user
        .ok_or_else(|| anyhow::anyhow!("Missing user info"))?;
    let selection_params = req.selection_params.unwrap_or_else(|| {
        serde_json::json!({
            "unitId": "",
            "subUnitId": "",
            "selections": [],
        })
    });

    let awareness_user = AwarenessStateItem {
        client_id,
        id: user.id,
        name: user.name,
        selection_params,
    };

    state
        .awareness_service
        .upsert_state(&doc_id, &socket.id.to_string(), awareness_user.clone())
        .await?;

    let room = format!("doc:{}", doc_id);
    let json = serde_json::to_value(&awareness_user)?;
    if let Err(e) = socket.to(room).emit("presence_update", &json).await {
        warn!("Error broadcasting presence_update: {}", e);
    }

    Ok(())
}

async fn handle_disconnect(socket: &SocketRef, state: &AppState) {
    info!("Socket disconnecting: {:?}", socket.id);

    // Decrement online users counter
    metrics::decrement_online_users();

    let rooms = socket.rooms().into_iter().collect::<Vec<_>>();

    for room in rooms {
        if let Some(doc_id) = room.strip_prefix("doc:") {
            // Clean up awareness state
            if let Err(e) = state
                .awareness_service
                .remove_by_socket(doc_id, &socket.id.to_string())
                .await
            {
                warn!("Error removing client awareness: {}", e);
            }

            // Track socket leaving document
            let is_document_empty = track_socket_leave(doc_id, &socket.id.to_string());

            // If this was the last socket in this document, decrement online documents
            if is_document_empty {
                metrics::decrement_online_documents();
                info!("Document {} now offline (last user disconnected)", doc_id);
            }
        }
    }
}
