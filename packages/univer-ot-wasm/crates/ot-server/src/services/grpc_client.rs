use crate::services::EtcdService;
use anyhow::{anyhow, Result};
use rand::seq::SliceRandom;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tonic::transport::Channel;
use tracing::{debug, info, warn};

// Include generated proto code for user service
pub mod user_proto {
    tonic::include_proto!("user");
}

// Include generated proto code for document service
pub mod document_proto {
    tonic::include_proto!("document");
}

pub use document_proto::document_client::DocumentClient;
pub use user_proto::user_client::UserClient;

/// Channel refresh interval - refresh channels periodically to handle endpoint changes
const CHANNEL_REFRESH_INTERVAL: Duration = Duration::from_secs(300); // 5 minutes

/// Cached channel with creation timestamp for TTL-based refresh
struct CachedChannel {
    channel: Channel,
    created_at: Instant,
    endpoint: String,
}

impl CachedChannel {
    fn new(channel: Channel, endpoint: String) -> Self {
        Self {
            channel,
            created_at: Instant::now(),
            endpoint,
        }
    }

    fn is_expired(&self) -> bool {
        self.created_at.elapsed() > CHANNEL_REFRESH_INTERVAL
    }
}

/// Inner state for GrpcClientService
struct GrpcClientServiceInner {
    etcd_service: Arc<EtcdService>,
    user_rpc_prefix: String,
    document_rpc_prefix: String,
    /// Cached user service channel (lazily initialized, refreshed on TTL expiry)
    user_channel: RwLock<Option<CachedChannel>>,
    /// Cached document service channel (lazily initialized, refreshed on TTL expiry)
    document_channel: RwLock<Option<CachedChannel>>,
}

/// gRPC client service for managing connections to external services
/// Uses etcd for dynamic service discovery
///
/// This service uses connection pooling to reuse HTTP/2 channels:
/// - Channels are lazily created on first use
/// - Channels are refreshed after CHANNEL_REFRESH_INTERVAL (5 minutes)
/// - HTTP/2 multiplexing allows multiple concurrent requests on a single channel
///
/// This service is wrapped in Arc so cloning is cheap (reference count only)
#[derive(Clone)]
pub struct GrpcClientService {
    inner: Arc<GrpcClientServiceInner>,
}

impl GrpcClientService {
    /// Create a new GrpcClientService
    ///
    /// # Arguments
    /// * `etcd_service` - EtcdService for dynamic service discovery
    /// * `user_rpc_prefix` - Etcd prefix for user service endpoints
    /// * `document_rpc_prefix` - Etcd prefix for document service endpoints
    pub fn new(
        etcd_service: Arc<EtcdService>,
        user_rpc_prefix: String,
        document_rpc_prefix: String,
    ) -> Self {
        info!(
            "Initializing GrpcClientService with connection pooling: user_rpc_prefix={}, document_rpc_prefix={}, channel_refresh_interval={:?}",
            user_rpc_prefix, document_rpc_prefix, CHANNEL_REFRESH_INTERVAL
        );

        Self {
            inner: Arc::new(GrpcClientServiceInner {
                etcd_service,
                user_rpc_prefix,
                document_rpc_prefix,
                user_channel: RwLock::new(None),
                document_channel: RwLock::new(None),
            }),
        }
    }

    /// Get a random endpoint from etcd for the given service prefix
    async fn get_service_endpoint(&self, prefix: &str) -> Result<String> {
        let endpoints = self
            .inner
            .etcd_service
            .get_service_endpoints(prefix)
            .await
            .map_err(|e| anyhow!("Failed to get {} endpoints from etcd: {}", prefix, e))?;

        if endpoints.is_empty() {
            return Err(anyhow!("No available endpoints for service: {}", prefix));
        }

        // Randomly select one endpoint for load balancing
        let endpoint = endpoints
            .choose(&mut rand::thread_rng())
            .ok_or_else(|| anyhow!("Failed to select endpoint"))?;

        // Format as gRPC URL (etcd stores IP:PORT, we need http://IP:PORT)
        let url = if endpoint.starts_with("http://") || endpoint.starts_with("https://") {
            endpoint.clone()
        } else {
            format!("http://{}", endpoint)
        };

        Ok(url)
    }

    /// Get or create a cached channel for the user service.
    /// Channels are reused across multiple requests (HTTP/2 multiplexing).
    /// Channels are refreshed after CHANNEL_REFRESH_INTERVAL to handle endpoint changes.
    async fn get_user_channel(&self) -> Result<Channel> {
        // Fast path: check if we have a valid cached channel
        {
            let guard = self.inner.user_channel.read().await;
            if let Some(cached) = guard.as_ref() {
                if !cached.is_expired() {
                    return Ok(cached.channel.clone());
                }
                debug!(
                    "User service channel expired (age={:?}), will refresh",
                    cached.created_at.elapsed()
                );
            }
        }

        // Slow path: create or refresh the channel
        let mut guard = self.inner.user_channel.write().await;

        // Double-check after acquiring write lock (another thread may have refreshed)
        if let Some(cached) = guard.as_ref() {
            if !cached.is_expired() {
                return Ok(cached.channel.clone());
            }
        }

        let url = self
            .get_service_endpoint(&self.inner.user_rpc_prefix)
            .await?;
        let channel = Channel::from_shared(url.clone())?
            .connect()
            .await
            .map_err(|e| anyhow!("Failed to connect to user service at {}: {}", url, e))?;

        info!("Created new user service channel: endpoint={}", url);
        *guard = Some(CachedChannel::new(channel.clone(), url));
        Ok(channel)
    }

    /// Get or create a cached channel for the document service.
    /// Channels are reused across multiple requests (HTTP/2 multiplexing).
    /// Channels are refreshed after CHANNEL_REFRESH_INTERVAL to handle endpoint changes.
    async fn get_document_channel(&self) -> Result<Channel> {
        // Fast path: check if we have a valid cached channel
        {
            let guard = self.inner.document_channel.read().await;
            if let Some(cached) = guard.as_ref() {
                if !cached.is_expired() {
                    return Ok(cached.channel.clone());
                }
                debug!(
                    "Document service channel expired (age={:?}), will refresh",
                    cached.created_at.elapsed()
                );
            }
        }

        // Slow path: create or refresh the channel
        let mut guard = self.inner.document_channel.write().await;

        // Double-check after acquiring write lock (another thread may have refreshed)
        if let Some(cached) = guard.as_ref() {
            if !cached.is_expired() {
                return Ok(cached.channel.clone());
            }
        }

        let url = self
            .get_service_endpoint(&self.inner.document_rpc_prefix)
            .await?;
        let channel = Channel::from_shared(url.clone())?
            .connect()
            .await
            .map_err(|e| anyhow!("Failed to connect to document service at {}: {}", url, e))?;

        info!("Created new document service channel: endpoint={}", url);
        *guard = Some(CachedChannel::new(channel.clone(), url));
        Ok(channel)
    }

    /// Create a user service client using the pooled channel.
    /// The underlying HTTP/2 connection is reused across multiple clients.
    pub async fn create_user_client(&self) -> Result<UserClient<Channel>> {
        let channel = self.get_user_channel().await?;
        Ok(UserClient::new(channel))
    }

    /// Create a document service client using the pooled channel.
    /// The underlying HTTP/2 connection is reused across multiple clients.
    pub async fn create_document_client(&self) -> Result<DocumentClient<Channel>> {
        let channel = self.get_document_channel().await?;
        Ok(DocumentClient::new(channel))
    }

    /// Force refresh the user service channel.
    /// Useful when connection errors occur or endpoints change.
    pub async fn refresh_user_channel(&self) {
        let mut guard = self.inner.user_channel.write().await;
        if let Some(cached) = guard.take() {
            info!(
                "Forcing user service channel refresh (previous endpoint={})",
                cached.endpoint
            );
        }
    }

    /// Force refresh the document service channel.
    /// Useful when connection errors occur or endpoints change.
    pub async fn refresh_document_channel(&self) {
        let mut guard = self.inner.document_channel.write().await;
        if let Some(cached) = guard.take() {
            info!(
                "Forcing document service channel refresh (previous endpoint={})",
                cached.endpoint
            );
        }
    }

    /// Verify an access token by calling the user service
    pub async fn verify_token(&self, uid: &str, token: &str) -> Result<()> {
        if token.is_empty() {
            return Err(anyhow!("Token is empty"));
        }

        let request = tonic::Request::new(user_proto::CheckAccessTokenReq {
            uid: uid.to_string(),
            access_token: token.to_string(),
            ep_id: String::new(),
            invoker: "ot-server".to_string(),
        });

        let mut client = self.create_user_client().await?;
        client.check_access_token(request).await.map_err(|e| {
            warn!("Token verification failed: {}", e);
            anyhow!("Token verification failed: {}", e)
        })?;

        Ok(())
    }

    /// Query document permissions from the document service
    pub async fn query_document_permissions(
        &self,
        user_id: &str,
        doc_id: &str,
        access_token: &str,
    ) -> Result<document_proto::Permissions> {
        let request = tonic::Request::new(document_proto::GetDocumentRequest {
            document_id: doc_id.to_string(),
            user_id: Some(user_id.to_string()),
            access_token: Some(access_token.to_string()),
        });

        let mut client = self.create_document_client().await?;
        let response = client.get_document(request).await.map_err(|e| {
            warn!(
                "Failed to query document permissions for doc_id={}, user_id={}: {}",
                doc_id, user_id, e
            );
            anyhow!("Failed to query document permissions: {}", e)
        })?;

        let resp = response.into_inner();
        let doc = resp.doc.ok_or_else(|| anyhow!("No document returned from document service"))?;
        doc.permissions
            .ok_or_else(|| anyhow!("No permissions returned from document service"))
    }

    /// Notify document service that a document has been modified
    pub async fn notify_modify_document(&self, user_id: &str, doc_id: &str) -> Result<()> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        let request = tonic::Request::new(document_proto::NotifyModifyDocumentRequest {
            user_id: user_id.to_string(),
            document_id: doc_id.to_string(),
            timestamp,
        });

        let mut client = self.create_document_client().await?;
        client.notify_modify_document(request).await.map_err(|e| {
            warn!(
                "Failed to notify document modification for doc_id={}, user_id={}: {}",
                doc_id, user_id, e
            );
            anyhow!("Failed to notify document modification: {}", e)
        })?;

        debug!(
            "Notified document modification: doc_id={}, user_id={}",
            doc_id, user_id
        );

        Ok(())
    }
}
