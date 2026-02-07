use crate::services::GrpcClientService;
use anyhow::{anyhow, Result};
use dashmap::DashMap;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::Deserialize;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::info;

/// JWT claims structure for token payload
#[derive(Debug, Deserialize)]
struct JwtClaims {
    /// User ID (from "id" field in token)
    id: Option<String>,
    /// User email
    email: Option<String>,
    /// Subject (alternative user ID field)
    sub: Option<String>,
}

/// User information extracted from token verification
#[derive(Debug, Clone)]
pub struct AuthUserInfo {
    pub uid: String,
    pub email: String,
}

/// Document permissions
#[derive(Debug, Clone)]
pub struct DocumentPermissions {
    pub readable: bool,
    pub commentable: bool,
    pub writable: bool,
    pub owner: bool,
}

/// Cached permission entry with timestamp
#[derive(Debug, Clone)]
struct CachedPermission {
    permissions: DocumentPermissions,
    cached_at: Instant,
}

/// Authentication service that verifies tokens and checks document permissions.
///
/// Features:
/// - Token verification via GrpcClientService
/// - Document permission checking via GrpcClientService
/// - Per-socket permission caching with TTL to reduce gRPC calls
#[derive(Clone)]
pub struct AuthService {
    grpc_client: Arc<GrpcClientService>,
    /// Per-socket permission cache: socket_id -> (doc_id -> CachedPermission)
    permission_cache: Arc<DashMap<String, DashMap<String, CachedPermission>>>,
    cache_ttl: Duration,
    /// Whether to skip token verification (for development/testing)
    skip_token_verification: bool,
    /// Whether to skip permission check and return full permissions (for local development)
    skip_permission_check: bool,
}

impl AuthService {
    /// Create a new AuthService instance
    ///
    /// # Arguments
    /// * `grpc_client` - GrpcClientService for making gRPC calls
    /// * `cache_ttl_secs` - TTL for permission cache in seconds
    pub fn new(grpc_client: Arc<GrpcClientService>, cache_ttl_secs: u64, skip_permission_check: bool) -> Self {
        // Check if token verification should be skipped (for development/testing)
        let skip_token_verification = std::env::var("SKIP_TOKEN_VERIFICATION")
            .map(|v| v.eq_ignore_ascii_case("true") || v == "1")
            .unwrap_or(false);

        info!(
            "Initializing AuthService with cache_ttl={}s, skip_token_verification={}, skip_permission_check={}",
            cache_ttl_secs, skip_token_verification, skip_permission_check
        );

        Self {
            grpc_client,
            permission_cache: Arc::new(DashMap::new()),
            cache_ttl: Duration::from_secs(cache_ttl_secs),
            skip_token_verification,
            skip_permission_check,
        }
    }

    /// Verify an access token by calling the user service
    ///
    /// Returns user information if the token is valid, or an error if invalid/expired
    pub async fn verify_token(&self, token: &str) -> Result<AuthUserInfo> {
        if token.is_empty() {
            return Err(anyhow!("Token is empty"));
        }

        let user_info = self.decode_token_payload(token)?;

        // Verify token with user service (can be skipped via SKIP_TOKEN_VERIFICATION env var)
        if self.skip_token_verification {
            info!(
                "Skipping token verification for user: {} (SKIP_TOKEN_VERIFICATION=true)",
                user_info.uid
            );
        } else {
            self.grpc_client.verify_token(&user_info.uid, token).await?;
            info!("Token verified successfully for user: {}", user_info.uid);
        }

        Ok(user_info)
    }

    /// Decode JWT token payload to extract user information
    /// This should only be called after the token has been verified by the user service
    fn decode_token_payload(&self, token: &str) -> Result<AuthUserInfo> {
        // Create a validation that skips signature verification
        // (token is already verified by the user service)
        let mut validation = Validation::new(Algorithm::HS256);
        validation.insecure_disable_signature_validation();
        validation.validate_exp = false;
        validation.validate_aud = false;

        // Use an empty key since we're skipping signature validation
        let key = DecodingKey::from_secret(&[]);

        let token_data = decode::<JwtClaims>(token, &key, &validation)
            .map_err(|e| anyhow!("Failed to decode JWT: {}", e))?;

        let claims = token_data.claims;

        // Try "id" field first, then fall back to "sub"
        let uid = claims
            .id
            .or(claims.sub)
            .ok_or_else(|| anyhow!("No user ID found in token"))?;

        let email = claims.email.unwrap_or_default();

        Ok(AuthUserInfo { uid, email })
    }

    /// Check document permissions for a user
    ///
    /// This method checks the cache first, and only calls the document service
    /// if the cache is empty or expired.
    ///
    /// # Arguments
    /// * `socket_id` - Socket ID for cache namespacing
    /// * `user_id` - User ID to check permissions for
    /// * `doc_id` - Document ID to check permissions for
    /// * `access_token` - Access token for authentication with document service
    pub async fn check_document_permission(
        &self,
        socket_id: &str,
        user_id: &str,
        doc_id: &str,
        access_token: &str,
    ) -> Result<DocumentPermissions> {
        // Skip permission check for local development
        if self.skip_permission_check {
            info!(
                "Skipping permission check for user: {}, doc: {} (SKIP_PERMISSION_CHECK=true)",
                user_id, doc_id
            );
            return Ok(DocumentPermissions {
                readable: true,
                commentable: true,
                writable: true,
                owner: true,
            });
        }

        // Check cache first
        if let Some(socket_cache) = self.permission_cache.get(socket_id) {
            if let Some(cached) = socket_cache.get(doc_id) {
                if cached.cached_at.elapsed() < self.cache_ttl {
                    return Ok(cached.permissions.clone());
                }
            }
        }

        // Cache miss or expired - call document service
        let proto_permissions = self
            .grpc_client
            .query_document_permissions(user_id, doc_id, access_token)
            .await?;

        let permissions = DocumentPermissions {
            readable: proto_permissions.readable,
            commentable: proto_permissions.commentable,
            writable: proto_permissions.writable,
            owner: proto_permissions.owner,
        };

        // Update cache
        let socket_cache = self
            .permission_cache
            .entry(socket_id.to_string())
            .or_insert_with(DashMap::new);
        socket_cache.insert(
            doc_id.to_string(),
            CachedPermission {
                permissions: permissions.clone(),
                cached_at: Instant::now(),
            },
        );

        Ok(permissions)
    }

    /// Invalidate all cached permissions for a socket
    ///
    /// Call this when a socket disconnects to clean up memory
    pub fn invalidate_socket_cache(&self, socket_id: &str) {
        self.permission_cache.remove(socket_id);
    }

    /// Invalidate a specific document permission for a socket
    pub fn invalidate_document_cache(&self, socket_id: &str, doc_id: &str) {
        if let Some(socket_cache) = self.permission_cache.get(socket_id) {
            socket_cache.remove(doc_id);
        }
    }

    /// Get cache statistics for monitoring
    pub fn get_cache_stats(&self) -> (usize, usize) {
        let socket_count = self.permission_cache.len();
        let total_entries: usize = self
            .permission_cache
            .iter()
            .map(|entry| entry.value().len())
            .sum();
        (socket_count, total_entries)
    }
}

