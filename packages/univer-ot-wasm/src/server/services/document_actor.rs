use super::ot::{Changeset, ChangesetApplied};
use super::OTService;
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot, Mutex};
use tracing::{debug, warn};
use uuid::Uuid;

const DOC_ACTOR_QUEUE_CAPACITY: usize = 128;
// Cleanup interval for closed actors (in seconds)
const ACTOR_CLEANUP_INTERVAL_SECS: u64 = 300; // 5 minutes

struct ApplyChangesetTask {
    changeset: Changeset,
    respond_to: oneshot::Sender<Result<ChangesetApplied>>,
}

#[derive(Clone)]
pub struct DocumentActorManager {
    ot_service: OTService,
    actors: Arc<Mutex<HashMap<Uuid, mpsc::Sender<ApplyChangesetTask>>>>,
}

impl DocumentActorManager {
    pub fn new(ot_service: OTService) -> Self {
        let manager = Self {
            ot_service,
            actors: Arc::new(Mutex::new(HashMap::new())),
        };

        // Spawn background task to periodically clean up closed actors
        let cleanup_manager = manager.clone();
        tokio::spawn(async move {
            cleanup_manager.start_cleanup_task().await;
        });

        manager
    }

    /// Background task that periodically cleans up closed actor senders.
    async fn start_cleanup_task(&self) {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(ACTOR_CLEANUP_INTERVAL_SECS));
        loop {
            interval.tick().await;
            let count = self.cleanup_closed_actors().await;
            if count > 0 {
                debug!("Actor cleanup task: removed {} closed actors", count);
            }
        }
    }

    pub async fn apply_changeset(
        &self,
        doc_id: Uuid,
        changeset: Changeset,
    ) -> Result<ChangesetApplied> {
        let (respond_to, response_rx) = oneshot::channel();
        let task = ApplyChangesetTask {
            changeset,
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
            // Remove the closed sender to prevent memory leak
            actors.remove(&doc_id);
            debug!("Removed closed actor sender: doc_id={}", doc_id);
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

    /// Remove a specific actor from the manager.
    /// This should be called when a document is deleted.
    pub async fn remove_actor(&self, doc_id: Uuid) {
        let mut actors = self.actors.lock().await;
        if actors.remove(&doc_id).is_some() {
            debug!("Removed actor for document: doc_id={}", doc_id);
        }
    }

    /// Clean up all closed actor senders.
    /// Returns the number of actors removed.
    pub async fn cleanup_closed_actors(&self) -> usize {
        let mut actors = self.actors.lock().await;
        let before = actors.len();
        actors.retain(|doc_id, sender| {
            let keep = !sender.is_closed();
            if !keep {
                debug!("Cleaning up closed actor: doc_id={}", doc_id);
            }
            keep
        });
        let removed = before - actors.len();
        if removed > 0 {
            debug!("Cleaned up {} closed actors", removed);
        }
        removed
    }

    /// Get the current number of actors being managed.
    pub async fn actor_count(&self) -> usize {
        self.actors.lock().await.len()
    }

    fn spawn_actor(&self, doc_id: Uuid) -> mpsc::Sender<ApplyChangesetTask> {
        let ot_service = self.ot_service.clone();
        let (sender, mut receiver) = mpsc::channel::<ApplyChangesetTask>(DOC_ACTOR_QUEUE_CAPACITY);

        tokio::spawn(async move {
            debug!("Document actor started: doc_id={}", doc_id);
            while let Some(task) = receiver.recv().await {
                let result = ot_service.apply_changeset(doc_id, task.changeset).await;
                let _ = task.respond_to.send(result);
            }
            debug!("Document actor stopped: doc_id={}", doc_id);
        });

        sender
    }
}
