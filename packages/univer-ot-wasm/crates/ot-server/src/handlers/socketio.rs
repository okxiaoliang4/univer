use crate::metrics;
use crate::services::auth::AuthUserInfo;
use crate::services::ot::Changeset;
use crate::state::AppState;
use crate::types::{
    AwarenessInitAck, AwarenessStateItem, ChangesetAck, ChangesetPushed, ChangesetRequest,
    FetchOpsAck, FetchOpsRequest, JoinDocAck, JoinDocRequest, LeaveDocRequest, OperationInfo,
    PresenceUpdateRequest,
};
use anyhow::Result;
use dashmap::DashMap;
use ot_core::MutationInfoWithOpId;
use socketioxide::adapter::Adapter;
use socketioxide::extract::AckSender;
use socketioxide::extract::{Data, SocketRef, State};
use std::collections::HashSet;
use std::sync::OnceLock;
use std::time::Instant;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Socket authentication data stored in socket extensions
#[derive(Debug, Clone)]
pub struct SocketAuthData {
    pub user_info: AuthUserInfo,
    pub access_token: String,
}

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
    let mut entry = doc_sockets
        .entry(doc_id.to_string())
        .or_insert_with(HashSet::new);
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

/// Auth request structure for Socket.IO handshake
#[derive(Debug, Clone, serde::Deserialize, Default)]
pub struct AuthRequest {
    #[serde(default)]
    pub token: String,
}

/// Extract auth token from socket handshake (fallback method)
fn extract_auth_token_from_query<A: Adapter>(socket: &SocketRef<A>) -> Option<String> {
    let handshake = socket.req_parts();

    // Try to parse from query string
    if let Some(query) = handshake.uri.query() {
        for pair in query.split('&') {
            if let Some((key, value)) = pair.split_once('=') {
                if key == "token" {
                    return Some(value.to_string());
                }
            }
        }
    }

    None
}

/// Handle socket connection
pub async fn on_connect<A: Adapter>(
    socket: SocketRef<A>,
    Data(auth): Data<AuthRequest>,
    state: State<AppState>,
) {
    info!(
        ">>> [CONNECT] Socket connected to /ws namespace: socket_id={:?}",
        socket.id
    );
    info!(
        ">>> [CONNECT] Auth token present: {}",
        !auth.token.is_empty()
    );

    // Extract auth token from handshake auth data or fallback to query string
    let token = if !auth.token.is_empty() {
        auth.token
    } else if let Some(t) = extract_auth_token_from_query(&socket) {
        t
    } else {
        warn!(
            "Socket {} connected without auth token, disconnecting",
            socket.id
        );
        socket.disconnect().ok();
        return;
    };

    // Verify token with user service
    let user_info = match state.auth_service.verify_token(&token).await {
        Ok(info) => {
            info!(
                "Socket {} authenticated as user {} ({})",
                socket.id, info.uid, info.email
            );
            info
        }
        Err(e) => {
            warn!("Socket {} auth failed: {}, disconnecting", socket.id, e);
            socket.disconnect().ok();
            return;
        }
    };

    // Store auth data in socket extensions for use in event handlers
    socket.extensions.insert(SocketAuthData {
        user_info,
        access_token: token,
    });

    // Increment online users counter
    metrics::increment_online_users();

    // Increment WebSocket connections counter
    metrics::increment_websocket_connections();

    // Register event handlers with named functions
    socket.on("join_doc", on_join_doc);
    socket.on("awareness_init", on_awareness_init);
    socket.on("leave_doc", on_leave_doc);
    socket.on("changeset", on_changeset);
    socket.on("fetch_ops", on_fetch_ops);
    socket.on("presence_update", on_presence_update);
    socket.on_disconnect(on_disconnect);
}

/// Handle join_doc event
async fn on_join_doc<A: Adapter>(
    socket: SocketRef<A>,
    Data(req): Data<JoinDocRequest>,
    state: State<AppState>,
    ack: AckSender<A>,
) {
    info!(
        ">>> [join_doc] Received request: socket_id={}, doc_id={}",
        socket.id, req.doc_id
    );
    match handle_join_doc(&socket, &state, req.clone()).await {
        Ok(ack_data) => {
            info!(
                ">>> [join_doc] Success: doc_id={}, version={:?}",
                req.doc_id, ack_data.version
            );
            match serde_json::to_value(&ack_data) {
                Ok(json) => {
                    info!(">>> [join_doc] Sending ACK: {:?}", json);
                    if let Err(e) = ack.send(&json) {
                        error!(">>> [join_doc] Failed to send ACK: {:?}", e);
                    } else {
                        info!(">>> [join_doc] ACK sent successfully");
                    }
                }
                Err(e) => {
                    error!(">>> [join_doc] Failed to serialize ACK: {}", e);
                }
            }
        }
        Err(e) => {
            error!(">>> [join_doc] Error: doc_id={}, error={}", req.doc_id, e);
            let error_ack = JoinDocAck {
                status: "error".to_string(),
                version: None,
                content: None,
                message: Some(e.to_string()),
            };
            if let Ok(json) = serde_json::to_value(&error_ack) {
                if let Err(e) = ack.send(&json) {
                    error!(">>> [join_doc] Failed to send error ACK: {:?}", e);
                }
            }
        }
    }
}

/// Handle awareness_init event
async fn on_awareness_init<A: Adapter>(
    socket: SocketRef<A>,
    Data(req): Data<JoinDocRequest>,
    ack: AckSender<A>,
    state: State<AppState>,
) {
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

/// Handle leave_doc event
async fn on_leave_doc<A: Adapter>(socket: SocketRef<A>, Data(req): Data<LeaveDocRequest>) {
    handle_leave_doc(&socket, req).await;
}

/// Handle changeset event
async fn on_changeset<A: Adapter>(
    socket: SocketRef<A>,
    Data(req): Data<ChangesetRequest>,
    ack: AckSender<A>,
    state: State<AppState>,
) {
    debug!(
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
                op_ids: None,
                message: Some(e.to_string()),
            };
            if let Ok(json) = serde_json::to_value(&error_ack) {
                let _ = ack.send(&json);
            }
        }
    }
}

/// Handle fetch_ops event
async fn on_fetch_ops<A: Adapter>(
    socket: SocketRef<A>,
    Data(req): Data<FetchOpsRequest>,
    ack: AckSender<A>,
    state: State<AppState>,
) {
    info!(
        ">>> [fetch_ops] Received request: socket_id={}, doc_id={}, start_rev={}",
        socket.id, req.doc_id, req.start_rev
    );
    match handle_fetch_ops(&socket, &state, req.clone()).await {
        Ok(ack_data) => {
            let ops_count = ack_data.operations.as_ref().map(|o| o.len()).unwrap_or(0);
            info!(
                ">>> [fetch_ops] Success: doc_id={}, ops_count={}",
                req.doc_id, ops_count
            );
            match serde_json::to_value(&ack_data) {
                Ok(json) => {
                    info!(">>> [fetch_ops] Sending ACK with {} operations", ops_count);
                    if let Err(e) = ack.send(&json) {
                        error!(">>> [fetch_ops] Failed to send ACK: {:?}", e);
                    } else {
                        info!(">>> [fetch_ops] ACK sent successfully");
                    }
                }
                Err(e) => {
                    error!(">>> [fetch_ops] Failed to serialize ACK: {}", e);
                }
            }
        }
        Err(e) => {
            error!(">>> [fetch_ops] Error: doc_id={}, error={}", req.doc_id, e);
            let error_ack = FetchOpsAck {
                status: "error".to_string(),
                operations: None,
                message: Some(e.to_string()),
            };
            if let Ok(json) = serde_json::to_value(&error_ack) {
                if let Err(e) = ack.send(&json) {
                    error!(">>> [fetch_ops] Failed to send error ACK: {:?}", e);
                }
            }
        }
    }
}

/// Handle presence_update event
async fn on_presence_update<A: Adapter>(
    socket: SocketRef<A>,
    Data(req): Data<PresenceUpdateRequest>,
    state: State<AppState>,
) {
    if let Err(e) = handle_presence_update(&socket, &state, req).await {
        warn!("Error handling presence_update: {}", e);
    }
}

/// Handle disconnect event
async fn on_disconnect<A: Adapter>(socket: SocketRef<A>, state: State<AppState>) {
    handle_disconnect(&socket, &state).await;
}

async fn handle_join_doc<A: Adapter>(
    socket: &SocketRef<A>,
    state: &AppState,
    req: JoinDocRequest,
) -> Result<JoinDocAck> {
    let doc_id =
        Uuid::parse_str(&req.doc_id).map_err(|e| anyhow::anyhow!("Invalid doc_id: {}", e))?;

    // Get auth data from socket extensions
    let auth_data = socket
        .extensions
        .get::<SocketAuthData>()
        .ok_or_else(|| anyhow::anyhow!("Socket not authenticated"))?;

    // Check document permissions
    let permissions = state
        .auth_service
        .check_document_permission(
            &socket.id.to_string(),
            &auth_data.user_info.uid,
            &req.doc_id,
            &auth_data.access_token,
        )
        .await
        .map_err(|e| anyhow::anyhow!("Permission check failed: {}", e))?;

    if !permissions.readable {
        return Err(anyhow::anyhow!(
            "Permission denied: user {} cannot read document {}",
            auth_data.user_info.uid,
            req.doc_id
        ));
    }

    info!(
        "User {} has permissions for doc {}: readable={}, writable={}",
        auth_data.user_info.uid, req.doc_id, permissions.readable, permissions.writable
    );

    // Verify document exists and get current version from documents table
    let version = state
        .document_service
        .get_current_version(doc_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Document not found: {}", req.doc_id))?;

    // Join the room
    socket.join(format!("doc:{}", req.doc_id));
    info!(
        "Socket {} (user {}) joined room {}",
        socket.id, auth_data.user_info.uid, req.doc_id
    );

    // Track socket joining document
    let is_new_document = track_socket_join(&req.doc_id, &socket.id.to_string());

    // Increment user joins counter
    metrics::increment_user_joins();

    // If this is the first socket in this document, increment online documents and active sessions
    if is_new_document {
        metrics::increment_online_documents();
        metrics::increment_active_sessions();
        info!("Document {} now online (first user)", req.doc_id);
    }

    // Record users per session (count of users in this document)
    if let Some(sockets) = get_document_sockets().get(&req.doc_id) {
        metrics::record_users_per_session(sockets.len() as f64);
    }

    Ok(JoinDocAck {
        status: "ok".to_string(),
        version: Some(version),
        content: None, // Don't return content - client should load from API or use existing local state
        message: None,
    })
}

async fn handle_leave_doc<A: Adapter>(socket: &SocketRef<A>, req: LeaveDocRequest) {
    let room = format!("doc:{}", req.doc_id);
    socket.leave(room.clone());
    info!("Socket {} left room {}", socket.id, room);

    // Increment user leaves counter
    metrics::increment_user_leaves();

    // Track socket leaving document
    let is_document_empty = track_socket_leave(&req.doc_id, &socket.id.to_string());

    // If this was the last socket in this document, decrement online documents and active sessions
    if is_document_empty {
        metrics::decrement_online_documents();
        metrics::decrement_active_sessions();
        info!("Document {} now offline (last user left)", req.doc_id);
    }
}

async fn handle_awareness_init<A: Adapter>(
    socket: &SocketRef<A>,
    state: &AppState,
    req: JoinDocRequest,
) -> Result<AwarenessInitAck> {
    let doc_id = req.doc_id.clone();
    let room = format!("doc:{}", doc_id);
    socket.join(room.clone());

    // Track socket joining document
    let is_new_document = track_socket_join(&doc_id, &socket.id.to_string());

    // Increment user joins counter
    metrics::increment_user_joins();

    // If this is the first socket in this document, increment online documents and active sessions
    if is_new_document {
        metrics::increment_online_documents();
        metrics::increment_active_sessions();
        info!("Document {} now online (first user via awareness)", doc_id);
    }

    // Record users per session
    if let Some(sockets) = get_document_sockets().get(&doc_id) {
        metrics::record_users_per_session(sockets.len() as f64);
    }

    let snapshot = state.awareness_service.get_state(&doc_id).await?;
    let states = snapshot.into_values().collect::<Vec<_>>();

    Ok(AwarenessInitAck {
        status: "ok".to_string(),
        states,
    })
}

async fn handle_changeset<A: Adapter>(
    socket: &SocketRef<A>,
    state: &AppState,
    req: ChangesetRequest,
) -> Result<ChangesetAck> {
    debug!(
        "Handling changeset: doc_id={}, base_rev={}, mutations_count={}",
        req.doc_id,
        req.base_rev,
        req.mutations.len()
    );

    // Get auth data from socket extensions
    let auth_data = socket
        .extensions
        .get::<SocketAuthData>()
        .ok_or_else(|| anyhow::anyhow!("Socket not authenticated"))?;

    let doc_id =
        Uuid::parse_str(&req.doc_id).map_err(|e| anyhow::anyhow!("Invalid doc_id: {}", e))?;

    // Check document permissions - must have write permission
    let permissions = state
        .auth_service
        .check_document_permission(
            &socket.id.to_string(),
            &auth_data.user_info.uid,
            &req.doc_id,
            &auth_data.access_token,
        )
        .await
        .map_err(|e| anyhow::anyhow!("Permission check failed: {}", e))?;

    if !permissions.writable {
        return Err(anyhow::anyhow!(
            "Permission denied: user {} cannot write to document {}",
            auth_data.user_info.uid,
            req.doc_id
        ));
    }

    // Convert ChangesetRequest to Changeset for OTService
    let mutations: Vec<MutationInfoWithOpId> = req.mutations;

    // Use authenticated user_id from token instead of socket.id
    let changeset = Changeset {
        base_rev: req.base_rev,
        user_id: auth_data.user_info.uid.clone(),
        mutations,
        client_id: req
            .client_id
            .clone()
            .unwrap_or_else(|| format!("socket:{}", socket.id)),
    };

    // Apply changeset via DocumentActorManager
    // Within a single instance: requests are serialized via Actor's mpsc channel (no lock contention)
    // Across instances: OTService's distributed lock ensures cross-instance coordination
    // This reduces lock contention from N:1 (all requests) to M:1 (one per instance)
    let result = state
        .document_actor_manager
        .apply_changeset(doc_id, changeset)
        .await?;

    debug!(
        "Changeset applied successfully: doc_id={}, user_id={}, server_rev={}, mutations_count={}",
        req.doc_id,
        auth_data.user_info.uid,
        result.server_rev,
        result.mutations.len()
    );

    // Prepare ACK response first (before broadcast)
    let ack_response = ChangesetAck {
        status: "ok".to_string(),
        server_rev: Some(result.server_rev),
        op_ids: Some(result.op_ids),
        message: None,
    };

    // Broadcast to room asynchronously (fire-and-forget, doesn't block ACK)
    let room = format!("doc:{}", req.doc_id);
    let pushed = ChangesetPushed {
        doc_id: req.doc_id.clone(),
        server_rev: result.server_rev,
        user_id: result.user_id.clone(),
    };
    let socket_clone = socket.clone();
    let grpc_client = state.grpc_client.clone();
    let user_id = auth_data.user_info.uid.clone();
    let doc_id_for_notify = req.doc_id.clone();
    tokio::spawn(async move {
        let broadcast_start = Instant::now();
        match serde_json::to_value(&pushed) {
            Ok(json) => {
                match socket_clone
                    .to(room.clone())
                    .emit("changeset_pushed", &json)
                    .await
                {
                    Ok(_) => {
                        metrics::record_broadcast_latency(broadcast_start.elapsed().as_secs_f64());
                        debug!(
                            "Broadcasted changeset_pushed to room {}: server_rev={}",
                            room, pushed.server_rev
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

        if let Err(e) = grpc_client
            .notify_modify_document(&user_id, &doc_id_for_notify)
            .await
        {
            warn!(
                "Failed to notify document modification: doc_id={}, user_id={}, error={}",
                doc_id_for_notify, user_id, e
            );
        }
    });

    Ok(ack_response)
}

async fn handle_fetch_ops<A: Adapter>(
    _socket: &SocketRef<A>,
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

async fn handle_presence_update<A: Adapter>(
    socket: &SocketRef<A>,
    state: &AppState,
    req: PresenceUpdateRequest,
) -> Result<()> {
    let doc_id = req.doc_id.clone();

    // Use socket.id as the client_id for consistent identification
    let client_id = socket.id.to_string();

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

async fn handle_disconnect<A: Adapter>(socket: &SocketRef<A>, state: &AppState) {
    let socket_id = socket.id.to_string();

    // Get user info before cleanup for logging
    let (user_id, had_auth_data) = socket
        .extensions
        .get::<SocketAuthData>()
        .map(|auth| (auth.user_info.uid.clone(), true))
        .unwrap_or_else(|| ("unknown".to_string(), false));

    debug!(
        "Socket {} (user {}) disconnecting, had_auth_data={}",
        socket_id, user_id, had_auth_data
    );

    // Decrement online users counter (fast, in-memory)
    metrics::decrement_online_users();

    // Decrement WebSocket connections counter (fast, in-memory)
    metrics::decrement_websocket_connections();

    // Collect room data synchronously while we have access to the socket
    let rooms = socket.rooms().into_iter().collect::<Vec<_>>();
    let room_count = rooms.len();

    // Extract doc_ids for async cleanup
    let doc_ids: Vec<String> = rooms
        .iter()
        .filter_map(|room| room.strip_prefix("doc:").map(|s| s.to_string()))
        .collect();

    // Synchronous cleanup: track socket leaving and update metrics
    // These operations are fast (DashMap is lock-free) and must be accurate
    for doc_id in &doc_ids {
        // Increment user leaves counter
        metrics::increment_user_leaves();

        // Track socket leaving document (fast, lock-free DashMap operation)
        let is_document_empty = track_socket_leave(doc_id, &socket_id);

        // If this was the last socket in this document, decrement online documents and active sessions
        if is_document_empty {
            metrics::decrement_online_documents();
            metrics::decrement_active_sessions();
            info!("Document {} now offline (last user disconnected)", doc_id);
        }
    }

    // Log synchronous cleanup completion
    info!(
        "Disconnect sync cleanup done: socket_id={}, user_id={}, rooms={}",
        socket_id, user_id, room_count
    );

    // Spawn async cleanup tasks (awareness state removal, permission cache invalidation)
    // These involve Redis/network calls and should not block the disconnect handler
    let auth_service = state.auth_service.clone();
    let awareness_service = state.awareness_service.clone();
    let socket_id_for_cleanup = socket_id.clone();
    let user_id_for_cleanup = user_id.clone();

    tokio::spawn(async move {
        // Invalidate permission cache for this socket
        auth_service.invalidate_socket_cache(&socket_id_for_cleanup);

        // Clean up awareness state for all documents (involves Redis calls)
        for doc_id in doc_ids {
            if let Err(e) = awareness_service
                .remove_by_socket(&doc_id, &socket_id_for_cleanup)
                .await
            {
                warn!(
                    "Error removing client awareness: doc_id={}, socket_id={}, error={}",
                    doc_id, socket_id_for_cleanup, e
                );
            }
        }

        // Log async cleanup completion
        info!(
            "Disconnect async cleanup done: socket_id={}, user_id={}",
            socket_id_for_cleanup, user_id_for_cleanup
        );
    });
}
