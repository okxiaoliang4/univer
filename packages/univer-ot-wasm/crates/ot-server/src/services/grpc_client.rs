use crate::services::EtcdService;
use anyhow::{anyhow, Result};
use rand::seq::SliceRandom;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tonic::transport::Channel;
use tracing::{info, warn};

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

/// Inner state for GrpcClientService
struct GrpcClientServiceInner {
    etcd_service: EtcdService,
    user_rpc_prefix: String,
    document_rpc_prefix: String,
}

/// gRPC client service for managing connections to external services
/// Uses etcd for dynamic service discovery
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
        etcd_service: EtcdService,
        user_rpc_prefix: String,
        document_rpc_prefix: String,
    ) -> Self {
        info!(
            "Initializing GrpcClientService: user_rpc_prefix={}, document_rpc_prefix={}",
            user_rpc_prefix, document_rpc_prefix
        );

        Self {
            inner: Arc::new(GrpcClientServiceInner {
                etcd_service,
                user_rpc_prefix,
                document_rpc_prefix,
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

    /// Create a user service client by discovering endpoint from etcd
    pub async fn create_user_client(&self) -> Result<UserClient<Channel>> {
        let url = self.get_service_endpoint(&self.inner.user_rpc_prefix).await?;
        let channel = Channel::from_shared(url.clone())?
            .connect()
            .await
            .map_err(|e| anyhow!("Failed to connect to user service at {}: {}", url, e))?;
        Ok(UserClient::new(channel))
    }

    /// Create a document service client by discovering endpoint from etcd
    pub async fn create_document_client(&self) -> Result<DocumentClient<Channel>> {
        let url = self.get_service_endpoint(&self.inner.document_rpc_prefix).await?;
        let channel = Channel::from_shared(url.clone())?
            .connect()
            .await
            .map_err(|e| anyhow!("Failed to connect to document service at {}: {}", url, e))?;
        Ok(DocumentClient::new(channel))
    }

    /// Verify an access token by calling the user service
    pub async fn verify_token(&self, token: &str) -> Result<()> {
        if token.is_empty() {
            return Err(anyhow!("Token is empty"));
        }

        let request = tonic::Request::new(user_proto::CheckAccessTokenReq {
            uid: String::new(),
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

        info!(
            "Notified document modification: doc_id={}, user_id={}",
            doc_id, user_id
        );

        Ok(())
    }
}
