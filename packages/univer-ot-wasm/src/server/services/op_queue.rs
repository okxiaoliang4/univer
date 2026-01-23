use anyhow::Result;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct OpQueueService {
    redis_client: redis::Client,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PendingDoc {
    pub doc_id: String,
    pub timestamp: i64,
}

impl OpQueueService {
    pub fn new(redis_client: redis::Client) -> Self {
        Self { redis_client }
    }

    pub async fn enqueue_doc(&self, doc_id: &str) -> Result<()> {
        let mut conn = self.redis_client.get_multiplexed_async_connection().await?;
        let lock_key = format!("snapshot:lock:{}", doc_id);
        let dedupe_set_key = "snapshot:doc-queue:set";
        let lock_exists: bool = conn.exists(&lock_key).await?;
        if lock_exists {
            let _: i64 = conn.sadd(dedupe_set_key, doc_id).await?;
        } else {
            let added: i64 = conn.sadd(dedupe_set_key, doc_id).await?;
            if added == 0 {
                return Ok(());
            }
        }
        let payload = serde_json::to_string(&PendingDoc {
            doc_id: doc_id.to_string(),
            timestamp: chrono::Utc::now().timestamp(),
        })?;
        let _: () = conn.lpush("snapshot:doc-queue", payload).await?;
        Ok(())
    }

    pub async fn drain_doc_queue(&self, limit: usize) -> Result<Vec<PendingDoc>> {
        let mut conn = self.redis_client.get_multiplexed_async_connection().await?;
        let dedupe_set_key = "snapshot:doc-queue:set";
        let mut result = Vec::new();
        for _ in 0..limit {
            let payload: Option<String> = conn.lpop("snapshot:doc-queue", None).await?;
            if let Some(payload) = payload {
                if let Ok(doc) = serde_json::from_str::<PendingDoc>(&payload) {
                    let _: i64 = conn.srem(dedupe_set_key, &doc.doc_id).await?;
                    result.push(doc);
                }
            } else {
                break;
            }
        }
        Ok(result)
    }

}
