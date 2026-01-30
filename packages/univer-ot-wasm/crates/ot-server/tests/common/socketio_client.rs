use anyhow::{Context, Result};
use futures_util::future::FutureExt;
use ot_core::MutationInfoWithOpId;
use rust_socketio::{
    asynchronous::{Client, ClientBuilder},
    Payload,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Request structure for changeset event (same as server)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangesetRequest {
    #[serde(rename = "docId")]
    pub doc_id: String,
    #[serde(rename = "baseRev")]
    pub base_rev: i64,
    #[serde(rename = "clientId")]
    pub client_id: Option<String>,
    pub mutations: Vec<MutationInfoWithOpId>,
}

/// Ack response for changeset event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangesetAck {
    pub status: String,
    #[serde(rename = "serverRev")]
    pub server_rev: Option<i64>,
    pub mutations: Option<Vec<serde_json::Value>>,
    pub message: Option<String>,
}

/// Request structure for join_doc event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinDocRequest {
    #[serde(rename = "docId")]
    pub doc_id: String,
}

/// Ack response for join_doc event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinDocAck {
    pub status: String,
    pub version: Option<i64>,
    pub content: Option<serde_json::Value>,
    pub message: Option<String>,
}

/// Socket.IO test client for e2e testing
pub struct SocketIOTestClient {
    client: Client,
}

impl SocketIOTestClient {
    /// Connect to the Socket.IO server
    pub async fn connect(url: &str) -> Result<Self> {
        println!("🔌 Connecting to Socket.IO server: {}", url);

        // Build the client with event handlers
        let client = ClientBuilder::new(url)
            .namespace("/ws")
            .on("connect", |_, _| {
                async {
                    println!("✅ Connected to Socket.IO server");
                }
                .boxed()
            })
            .on("disconnect", |_, _| {
                async {
                    println!("❌ Disconnected from Socket.IO server");
                }
                .boxed()
            })
            .on("error", |err, _| {
                async move {
                    println!("❌ Socket.IO error: {:?}", err);
                }
                .boxed()
            })
            .on("changeset_pushed", |payload, _| {
                async move {
                    println!("📨 Received changeset_pushed: {:?}", payload);
                }
                .boxed()
            })
            .connect()
            .await
            .context("Failed to connect to Socket.IO server")?;

        println!("🎉 Socket.IO client connected successfully");

        // Wait a bit for connection to stabilize
        tokio::time::sleep(Duration::from_millis(200)).await;

        Ok(Self { client })
    }

    /// Join a document room
    pub async fn join_doc(&self, doc_id: &str) -> Result<JoinDocAck> {
        println!("📥 Joining document: {}", doc_id);

        let request = JoinDocRequest {
            doc_id: doc_id.to_string(),
        };

        let payload_json = serde_json::to_value(&request)
            .context("Failed to serialize join_doc request")?;

        // Store ack in Arc<Mutex>
        let ack_result: Arc<Mutex<Option<JoinDocAck>>> = Arc::new(Mutex::new(None));
        let ack_clone = ack_result.clone();

        // Emit with callback
        self.client
            .emit_with_ack(
                "join_doc",
                Payload::from(payload_json.to_string()),
                Duration::from_secs(10),
                move |payload: Payload, _: Client| {
                    let ack_clone = ack_clone.clone();
                    async move {
                        let parsed_ack: Option<JoinDocAck> = match payload {
                            Payload::Text(text) => {
                                // text[0] is an Array containing the actual ack object
                                if let Some(first) = text.first() {
                                    if let Some(arr) = first.as_array() {
                                        // Array [Object {...}] - get the first element
                                        if let Some(obj) = arr.first() {
                                            serde_json::from_value::<JoinDocAck>(obj.clone()).ok()
                                        } else {
                                            None
                                        }
                                    } else {
                                        // Try direct parse
                                        serde_json::from_value::<JoinDocAck>(first.clone()).ok()
                                    }
                                } else {
                                    None
                                }
                            }
                            Payload::Binary(bytes) => {
                                serde_json::from_slice(&bytes).ok()
                            }
                            _ => None,
                        };

                        if let Some(ack) = parsed_ack {
                            *ack_clone.lock().unwrap() = Some(ack);
                        }
                    }
                    .boxed()
                },
            )
            .await
            .context("Failed to emit join_doc event")?;

        // Wait for ack
        for _ in 0..50 {
            tokio::time::sleep(Duration::from_millis(100)).await;
            if ack_result.lock().unwrap().is_some() {
                break;
            }
        }

        let ack = ack_result
            .lock()
            .unwrap()
            .clone()
            .context("No ack received for join_doc")?;

        if ack.status == "ok" {
            println!("✅ Successfully joined document: {}", doc_id);
            if let Some(version) = ack.version {
                println!("   Document version: {}", version);
            }
        } else {
            println!("❌ Failed to join document: {:?}", ack.message);
        }

        Ok(ack)
    }

    /// Send a changeset to the server
    pub async fn send_changeset(&self, request: ChangesetRequest) -> Result<ChangesetAck> {
        let doc_id = request.doc_id.clone();
        let mutation_count = request.mutations.len();
        let base_rev = request.base_rev;

        println!(
            "📤 Sending changeset: doc={}, baseRev={}, mutations={}",
            doc_id, base_rev, mutation_count
        );

        let payload_json = serde_json::to_value(&request)
            .context("Failed to serialize changeset request")?;

        // Store ack in Arc<Mutex>
        let ack_result: Arc<Mutex<Option<ChangesetAck>>> = Arc::new(Mutex::new(None));
        let ack_clone = ack_result.clone();

        self.client
            .emit_with_ack(
                "changeset",
                Payload::from(payload_json.to_string()),
                Duration::from_secs(30),
                move |payload: Payload, _: Client| {
                    let ack_clone = ack_clone.clone();
                    async move {
                        let parsed_ack: Option<ChangesetAck> = match payload {
                            Payload::Text(text) => {
                                // text[0] is an Array containing the actual ack object
                                if let Some(first) = text.first() {
                                    if let Some(arr) = first.as_array() {
                                        // Array [Object {...}] - get the first element
                                        if let Some(obj) = arr.first() {
                                            serde_json::from_value::<ChangesetAck>(obj.clone()).ok()
                                        } else {
                                            None
                                        }
                                    } else {
                                        // Try direct parse
                                        serde_json::from_value::<ChangesetAck>(first.clone()).ok()
                                    }
                                } else {
                                    None
                                }
                            }
                            Payload::Binary(bytes) => {
                                serde_json::from_slice(&bytes).ok()
                            }
                            _ => None,
                        };

                        if let Some(ack) = parsed_ack {
                            *ack_clone.lock().unwrap() = Some(ack);
                        }
                    }
                    .boxed()
                },
            )
            .await
            .context("Failed to emit changeset event")?;

        // Wait for ack (longer timeout for changesets)
        for _ in 0..300 {
            tokio::time::sleep(Duration::from_millis(100)).await;
            if ack_result.lock().unwrap().is_some() {
                break;
            }
        }

        let ack = ack_result
            .lock()
            .unwrap()
            .clone()
            .context("No ack received for changeset")?;

        if ack.status == "ok" {
            println!(
                "✅ Changeset acknowledged: serverRev={}",
                ack.server_rev.unwrap_or(-1)
            );
        } else {
            println!("❌ Changeset failed: {:?}", ack.message);
        }

        Ok(ack)
    }

    /// Send multiple changesets in sequence
    pub async fn send_changesets_batch(
        &self,
        requests: Vec<ChangesetRequest>,
    ) -> Result<Vec<ChangesetAck>> {
        let mut acks = Vec::new();

        for (i, request) in requests.into_iter().enumerate() {
            println!("\n📦 Sending changeset batch {}/{}", i + 1, acks.len() + 1);
            let ack = self.send_changeset(request).await?;
            acks.push(ack);
        }

        Ok(acks)
    }

    /// Disconnect from the server
    pub async fn disconnect(&self) -> Result<()> {
        println!("🔌 Disconnecting from Socket.IO server");
        self.client
            .disconnect()
            .await
            .context("Failed to disconnect")?;
        println!("✅ Disconnected successfully");
        Ok(())
    }
}

/// Helper to create a changeset request
pub fn create_changeset_request(
    doc_id: &str,
    base_rev: i64,
    client_id: &str,
    mutations: Vec<MutationInfoWithOpId>,
) -> ChangesetRequest {
    ChangesetRequest {
        doc_id: doc_id.to_string(),
        base_rev,
        client_id: Some(client_id.to_string()),
        mutations,
    }
}
