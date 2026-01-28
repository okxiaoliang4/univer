use crate::server::database::entities::storage;
use anyhow::{Context, Result};
use aws_credential_types::Credentials;
use aws_sdk_s3::config::Region;
use aws_sdk_s3::presigning::PresigningConfig;
use aws_sdk_s3::{config::Builder as S3ConfigBuilder, Client as S3Client};
use redis::AsyncCommands;
use sea_orm::{ActiveModelTrait, EntityTrait};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

const SIGNED_URL_TTL_SECONDS: u64 = 60 * 60 * 3;
const SIGNED_URL_CACHE_SAFETY_SECONDS: u64 = 60;

#[derive(Clone)]
pub struct StorageService {
    db: Arc<sea_orm::DatabaseConnection>,
    s3_client: S3Client,
    redis_client: redis::Client,
    endpoint: String,
    region: String,
    bucket: String,
    server_env: String,
}

#[derive(Debug, Clone)]
pub struct StoredSnapshot {
    pub storage_id: Uuid,
    pub size: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StorageLocation {
    pub endpoint: String,
    pub region: String,
    pub bucket: String,
    pub path: String,
    pub version_id: String,
}

impl StorageService {
    pub fn new(
        db: sea_orm::DatabaseConnection,
        endpoint: String,
        region: String,
        bucket: String,
        access_key: String,
        secret_key: String,
        server_env: String,
        redis_url: String,
    ) -> Result<Self> {
        let credentials = Credentials::new(access_key, secret_key, None, None, "static");
        let s3_config = S3ConfigBuilder::new()
            .credentials_provider(credentials)
            .region(Region::new(region.clone()))
            .endpoint_url(endpoint.clone())
            .force_path_style(true)
            .build();
        let s3_client = S3Client::from_conf(s3_config);
        let redis_client = redis::Client::open(redis_url)?;
        Ok(Self {
            db: Arc::new(db),
            s3_client,
            redis_client,
            endpoint,
            region,
            bucket,
            server_env,
        })
    }

    pub async fn get_storage_location(&self, storage_id: Uuid) -> Result<StorageLocation> {
        let record = storage::Entity::find_by_id(storage_id)
            .one(&*self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Storage not found: {}", storage_id))?;

        Ok(StorageLocation {
            endpoint: record.endpoint,
            region: record.region,
            bucket: record.bucket,
            path: record.path,
            version_id: record.version_id,
        })
    }

    pub async fn get_signed_url(&self, storage: &StorageLocation) -> Result<String> {
        let cache_key = format!(
            "storage:signed-url:{}:{}:{}",
            storage.bucket, storage.path, storage.version_id
        );

        let mut conn = self.redis_client.get_multiplexed_async_connection().await?;
        if let Ok(Some(cached)) = conn.get::<_, Option<String>>(&cache_key).await {
            if !cached.is_empty() {
                return Ok(cached);
            }
        }

        let expires_in = Duration::from_secs(SIGNED_URL_TTL_SECONDS);
        let presign_config = PresigningConfig::expires_in(expires_in)?;
        let request = self
            .s3_client
            .get_object()
            .bucket(&storage.bucket)
            .key(&storage.path)
            .version_id(&storage.version_id)
            .presigned(presign_config)
            .await?;
        let signed_url = request.uri().to_string();

        let cache_ttl = SIGNED_URL_TTL_SECONDS.saturating_sub(SIGNED_URL_CACHE_SAFETY_SECONDS);
        let _: () = conn.set_ex(&cache_key, &signed_url, cache_ttl).await?;

        Ok(signed_url)
    }

    pub async fn store_snapshot_content(
        &self,
        doc_id: Uuid,
        version: i64,
        content: &JsonValue,
    ) -> Result<StoredSnapshot> {
        let bytes = serde_json::to_vec(content)?;
        let size = bytes.len() as i64;
        let hash = format!("{:x}", md5::compute(&bytes));
        let path = format!(
            "{}/documents/{}/snapshots/{}.json",
            self.server_env, doc_id, version
        );
        let filename = format!("{}-{}.json", doc_id, version);
        let metadata = json!({
            "doc_id": doc_id.to_string(),
            "version": version,
        });

        tracing::info!(
            "Uploading snapshot to storage: doc_id={}, version={}, bucket={}, endpoint={}, path={}",
            doc_id,
            version,
            self.bucket,
            self.endpoint,
            path
        );
        let output = self
            .s3_client
            .put_object()
            .bucket(&self.bucket)
            .key(&path)
            .content_type("application/json")
            .body(bytes.into())
            .send()
            .await
            .with_context(|| {
                format!(
                    "Failed to upload snapshot to storage: doc_id={}, version={}, bucket={}, endpoint={}, path={}",
                    doc_id, version, self.bucket, self.endpoint, path
                )
            })?;

        let version_id = output
            .version_id()
            .map(|value| value.to_string())
            .unwrap_or_else(|| "null".to_string());

        let now = chrono::Utc::now();
        let storage_id = Uuid::new_v4();
        let record = storage::ActiveModel {
            id: sea_orm::Set(storage_id),
            endpoint: sea_orm::Set(self.endpoint.clone()),
            region: sea_orm::Set(self.region.clone()),
            bucket: sea_orm::Set(self.bucket.clone()),
            path: sea_orm::Set(path.clone()),
            size: sea_orm::Set(size),
            hash: sea_orm::Set(hash),
            hash_algorithm: sea_orm::Set("md5".to_string()),
            filename: sea_orm::Set(filename),
            content_type: sea_orm::Set(Some("application/json".to_string())),
            metadata: sea_orm::Set(Some(metadata.into())),
            version_id: sea_orm::Set(version_id),
            compressed: sea_orm::Set(None),
            created_at: sea_orm::Set(now.into()),
            updated_at: sea_orm::Set(now.into()),
        };

        record
            .insert(&*self.db)
            .await
            .with_context(|| {
                format!(
                    "Failed to insert storage record: doc_id={}, storage_id={}, path={}",
                    doc_id, storage_id, path
                )
            })?;

        Ok(StoredSnapshot { storage_id, size })
    }

    pub async fn fetch_snapshot_content(&self, storage_id: Uuid) -> Result<JsonValue> {
        let storage = self.get_storage_location(storage_id).await?;
        let response = self
            .s3_client
            .get_object()
            .bucket(storage.bucket)
            .key(storage.path)
            .version_id(storage.version_id)
            .send()
            .await?;

        let data = response.body.collect().await?.into_bytes();
        let content = serde_json::from_slice::<JsonValue>(&data)?;
        Ok(content)
    }
}
