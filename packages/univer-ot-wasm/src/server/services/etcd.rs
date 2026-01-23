use anyhow::{Context, Result};
use etcd_client::{Client, GetOptions, LeaseKeepAliveStream, LeaseKeeper};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio::time::interval;
use tracing::{error, info, warn};
use uuid::Uuid;

#[derive(Clone)]
pub struct EtcdService {
    client: Client,
}

pub struct EtcdRegistration {
    client: Client,
    lease_id: i64,
    keepalive_task: JoinHandle<()>,
    response_task: JoinHandle<()>,
}

impl EtcdService {
    pub async fn connect(endpoints: &[String]) -> Result<Self> {
        let client = Client::connect(endpoints, None)
            .await
            .context("Failed to connect to etcd")?;
        Ok(Self { client })
    }

    pub fn client(&self) -> Client {
        self.client.clone()
    }

    pub async fn register_with_lease(
        &self,
        service_name: &str,
        instance_id: Uuid,
        endpoint: String,
        lease_ttl_seconds: u64,
    ) -> Result<EtcdRegistration> {
        let mut client = self.client.clone();
        let lease = client
            .lease_grant(lease_ttl_seconds as i64, None)
            .await
            .context("Failed to grant etcd lease")?;
        let lease_id = lease.id();
        let key = format!("{}/{}", service_name, instance_id);

        client
            .put(key, endpoint, Some(etcd_client::PutOptions::new().with_lease(lease_id)))
            .await
            .context("Failed to register service key in etcd")?;

        let (keeper, stream) = client
            .lease_keep_alive(lease_id)
            .await
            .context("Failed to create lease keepalive")?;

        let keeper = Arc::new(Mutex::new(keeper));
        let keepalive_task = spawn_keepalive_task(keeper.clone(), lease_ttl_seconds);
        let response_task = spawn_keepalive_response_task(stream, lease_id);

        info!(
            "Registered service in etcd: key={}, lease_id={}",
            format!("{}/{}", service_name, instance_id),
            lease_id
        );

        Ok(EtcdRegistration {
            client: self.client.clone(),
            lease_id,
            keepalive_task,
            response_task,
        })
    }

    pub async fn get_service_endpoints(&self, service_name: &str) -> Result<Vec<String>> {
        let options = GetOptions::new().with_prefix();
        let mut client = self.client.clone();
        let response = client
            .get(format!("{}/", service_name), Some(options))
            .await
            .context("Failed to query etcd service endpoints")?;

        let endpoints = response
            .kvs()
            .iter()
            .filter_map(|kv| String::from_utf8(kv.value().to_vec()).ok())
            .collect();
        Ok(endpoints)
    }
}

impl EtcdRegistration {
    pub async fn revoke(mut self) {
        if let Err(err) = self.client.lease_revoke(self.lease_id).await {
            warn!(?err, "Failed to revoke etcd lease");
        }
        self.keepalive_task.abort();
        self.response_task.abort();
    }
}

fn spawn_keepalive_task(
    keeper: Arc<Mutex<LeaseKeeper>>,
    lease_ttl_seconds: u64,
) -> JoinHandle<()> {
    let interval_seconds = (lease_ttl_seconds / 3).max(1);
    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(interval_seconds));
        loop {
            ticker.tick().await;
            let result = {
                let mut guard = keeper.lock().await;
                guard.keep_alive().await
            };
            if let Err(err) = result {
                error!(?err, "Lease keepalive failed");
                break;
            }
        }
    })
}

fn spawn_keepalive_response_task(
    mut stream: LeaseKeepAliveStream,
    lease_id: i64,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        while let Ok(Some(resp)) = stream.message().await {
            if resp.ttl() <= 0 {
                warn!(lease_id, "Lease expired or revoked");
                break;
            }
        }
    })
}
