use super::ot::{Changeset, ChangesetApplied};
use super::OTService;
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot, Mutex};
use tracing::{debug, warn};
use uuid::Uuid;

const DOC_ACTOR_QUEUE_CAPACITY: usize = 128;

struct ApplyChangesetTask {
    changeset: Changeset,
    client_msg_id: String,
    respond_to: oneshot::Sender<Result<ChangesetApplied>>,
}

#[derive(Clone)]
pub struct DocumentActorManager {
    ot_service: OTService,
    actors: Arc<Mutex<HashMap<Uuid, mpsc::Sender<ApplyChangesetTask>>>>,
}

impl DocumentActorManager {
    pub fn new(ot_service: OTService) -> Self {
        Self {
            ot_service,
            actors: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn apply_changeset(
        &self,
        doc_id: Uuid,
        changeset: Changeset,
        client_msg_id: String,
    ) -> Result<ChangesetApplied> {
        let (respond_to, response_rx) = oneshot::channel();
        let task = ApplyChangesetTask {
            changeset,
            client_msg_id,
            respond_to,
        };

        let sender = self.get_or_spawn_sender(doc_id).await;
        let task = match sender.send(task).await {
            Ok(()) => None,
            Err(err) => {
                warn!(
                    "Document actor channel closed, respawning: doc_id={}",
                    doc_id
                );
                Some(err.0)
            }
        };

        if let Some(task) = task {
            let sender = self.replace_sender(doc_id).await;
            sender
                .send(task)
                .await
                .map_err(|_| anyhow!("Document actor stopped: doc_id={}", doc_id))?;
        }

        response_rx
            .await
            .map_err(|_| anyhow!("Document actor dropped response: doc_id={}", doc_id))?
    }

    async fn get_or_spawn_sender(&self, doc_id: Uuid) -> mpsc::Sender<ApplyChangesetTask> {
        let mut actors = self.actors.lock().await;
        if let Some(sender) = actors.get(&doc_id) {
            if !sender.is_closed() {
                return sender.clone();
            }
        }

        let sender = self.spawn_actor(doc_id);
        actors.insert(doc_id, sender.clone());
        sender
    }

    async fn replace_sender(&self, doc_id: Uuid) -> mpsc::Sender<ApplyChangesetTask> {
        let mut actors = self.actors.lock().await;
        let sender = self.spawn_actor(doc_id);
        actors.insert(doc_id, sender.clone());
        sender
    }

    fn spawn_actor(&self, doc_id: Uuid) -> mpsc::Sender<ApplyChangesetTask> {
        let ot_service = self.ot_service.clone();
        let (sender, mut receiver) = mpsc::channel::<ApplyChangesetTask>(DOC_ACTOR_QUEUE_CAPACITY);

        tokio::spawn(async move {
            debug!("Document actor started: doc_id={}", doc_id);
            while let Some(task) = receiver.recv().await {
                let result = ot_service
                    .apply_changeset(doc_id, task.changeset, task.client_msg_id)
                    .await;
                let _ = task.respond_to.send(result);
            }
            debug!("Document actor stopped: doc_id={}", doc_id);
        });

        sender
    }
}
