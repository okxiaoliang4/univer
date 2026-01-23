use crate::server::types::{AwarenessStateItem, AwarenessStateSnapshot};
use anyhow::Result;
use redis::AsyncCommands;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct AwarenessService {
    memory: Arc<RwLock<HashMap<String, HashMap<u64, AwarenessStateItem>>>>,
    socket_map: Arc<RwLock<HashMap<String, HashMap<String, u64>>>>,
    redis_client: Option<redis::Client>,
    redis_enabled: bool,
    ttl_seconds: u64,
}

impl AwarenessService {
    pub fn new(redis_url: String, redis_enabled: bool, ttl_seconds: u64) -> Result<Self> {
        let redis_client = if redis_enabled {
            Some(redis::Client::open(redis_url)?)
        } else {
            None
        };

        Ok(Self {
            memory: Arc::new(RwLock::new(HashMap::new())),
            socket_map: Arc::new(RwLock::new(HashMap::new())),
            redis_client,
            redis_enabled,
            ttl_seconds,
        })
    }

    fn redis_doc_key(doc_id: &str) -> String {
        format!("awareness:doc:{}", doc_id)
    }

    pub async fn get_state(&self, doc_id: &str) -> Result<HashMap<u64, AwarenessStateItem>> {
        if self.redis_enabled {
            if let Some(client) = &self.redis_client {
                let mut conn = client.get_multiplexed_async_connection().await?;
                let key = Self::redis_doc_key(doc_id);
                if let Ok(Some(raw)) = conn.get::<_, Option<String>>(&key).await {
                    let snapshot = serde_json::from_str::<AwarenessStateSnapshot>(&raw)?;
                    let state = snapshot
                        .states
                        .into_iter()
                        .map(|user| (user.client_id, user))
                        .collect::<HashMap<_, _>>();
                    let mut guard = self.memory.write().expect("awareness memory lock poisoned");
                    guard.insert(doc_id.to_string(), state.clone());
                    return Ok(state);
                }
            }
        }

        let guard = self.memory.read().expect("awareness memory lock poisoned");
        Ok(guard.get(doc_id).cloned().unwrap_or_default())
    }

    pub async fn upsert_state(
        &self,
        doc_id: &str,
        socket_id: &str,
        user: AwarenessStateItem,
    ) -> Result<()> {
        {
            let mut guard = self.memory.write().expect("awareness memory lock poisoned");
            guard
                .entry(doc_id.to_string())
                .or_default()
                .insert(user.client_id, user.clone());
        }

        {
            let mut guard = self
                .socket_map
                .write()
                .expect("awareness socket lock poisoned");
            guard
                .entry(doc_id.to_string())
                .or_default()
                .insert(socket_id.to_string(), user.client_id);
        }

        self.persist_doc(doc_id).await
    }

    pub async fn remove_client(&self, doc_id: &str, client_id: u64) -> Result<()> {
        {
            let mut guard = self.memory.write().expect("awareness memory lock poisoned");
            if let Some(doc_state) = guard.get_mut(doc_id) {
                doc_state.remove(&client_id);
            }
        }

        self.persist_doc(doc_id).await
    }

    pub async fn remove_by_socket(&self, doc_id: &str, socket_id: &str) -> Result<()> {
        let client_id = {
            let mut guard = self
                .socket_map
                .write()
                .expect("awareness socket lock poisoned");
            guard
                .get_mut(doc_id)
                .and_then(|doc_map| doc_map.remove(socket_id))
        };

        if let Some(client_id) = client_id {
            self.remove_client(doc_id, client_id).await?;
        }

        Ok(())
    }

    async fn persist_doc(&self, doc_id: &str) -> Result<()> {
        if !self.redis_enabled {
            return Ok(());
        }

        let Some(client) = &self.redis_client else {
            return Ok(());
        };

        let state = {
            let guard = self.memory.read().expect("awareness memory lock poisoned");
            guard
                .get(doc_id)
                .cloned()
                .unwrap_or_default()
                .into_values()
                .collect::<Vec<_>>()
        };

        let snapshot = AwarenessStateSnapshot { states: state };
        let payload = serde_json::to_string(&snapshot)?;
        let mut conn = client.get_multiplexed_async_connection().await?;
        let key = Self::redis_doc_key(doc_id);
        let _: () = conn.set_ex(key, payload, self.ttl_seconds).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::AwarenessService;
    use crate::server::types::AwarenessStateItem;
    use serde_json::json;

    #[tokio::test]
    async fn awareness_service_upsert_and_get() {
        let service = AwarenessService::new("redis://127.0.0.1/".to_string(), false, 120)
            .expect("awareness service");
        let user = AwarenessStateItem {
            client_id: 1,
            id: "user-1".to_string(),
            name: "User One".to_string(),
            selection_params: json!({
                "unitId": "u1",
                "subUnitId": "s1",
                "selections": [],
            }),
        };

        service
            .upsert_state("doc-1", "socket-1", user.clone())
            .await
            .expect("upsert");

        let state = service.get_state("doc-1").await.expect("get");
        let stored = state.get(&1).expect("stored user");
        assert_eq!(stored.id, "user-1");
        assert_eq!(stored.name, "User One");
    }

    #[tokio::test]
    async fn awareness_service_remove_by_socket() {
        let service = AwarenessService::new("redis://127.0.0.1/".to_string(), false, 120)
            .expect("awareness service");
        let user = AwarenessStateItem {
            client_id: 2,
            id: "user-2".to_string(),
            name: "User Two".to_string(),
            selection_params: json!({
                "unitId": "u2",
                "subUnitId": "s2",
                "selections": [],
            }),
        };

        service
            .upsert_state("doc-2", "socket-2", user)
            .await
            .expect("upsert");

        service
            .remove_by_socket("doc-2", "socket-2")
            .await
            .expect("remove");

        let state = service.get_state("doc-2").await.expect("get");
        assert!(state.get(&2).is_none());
    }
}
