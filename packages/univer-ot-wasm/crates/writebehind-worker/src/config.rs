//! Configuration for writebehind worker

use std::env;

/// Worker configuration
#[derive(Debug, Clone)]
pub struct Config {
    // Database
    pub database_url: String,

    // Redis
    pub redis_url: String,

    // S3
    pub s3_endpoint: String,
    pub s3_region: String,
    pub s3_bucket: String,
    pub s3_access_key: String,
    pub s3_secret_key: String,

    // Server
    pub server_env: String,
    pub health_port: u16,

    // Worker settings
    pub worker_count: usize,
    pub batch_size: usize,
    pub flush_interval_ms: u64,
    pub lock_ttl_ms: u64,
    pub max_retry_count: u32,
    pub retry_base_delay_ms: u64,
    pub params_inline_threshold_bytes: usize,
    pub cache_ttl_seconds: u64,

    // Stream settings
    pub instance_id: String,
    pub claim_interval_secs: u64,
    pub claim_min_idle_ms: u64,
    pub stream_batch_size: usize,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        let instance_id = env::var("INSTANCE_ID").unwrap_or_else(|_| {
            let short_uuid = &uuid::Uuid::new_v4().to_string()[..8];
            format!("inst-{}", short_uuid)
        });

        Self {
            database_url: env::var("DATABASE_URL")
                .expect("DATABASE_URL must be set"),
            redis_url: env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),
                s3_endpoint: env::var("S3_ENDPOINT")
                .expect("S3_ENDPOINT environment variable must be set"),
            s3_region: env::var("S3_REGION").expect("S3_REGION environment variable must be set"),
            s3_bucket: env::var("S3_BUCKET").expect("S3_BUCKET environment variable must be set"),
            s3_access_key: env::var("S3_ACCESS_KEY_ID")
                .expect("S3_ACCESS_KEY_ID environment variable must be set"),
            s3_secret_key: env::var("S3_SECRET_ACCESS_KEY")
                .expect("S3_SECRET_ACCESS_KEY environment variable must be set"),
            server_env: env::var("SERVER_ENV")
                .unwrap_or_else(|_| "development".to_string()),
            health_port: env::var("HEALTH_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .unwrap_or(8080),
            worker_count: env::var("WORKER_COUNT")
                .unwrap_or_else(|_| "8".to_string())
                .parse()
                .unwrap_or(8),
            batch_size: env::var("BATCH_SIZE")
                .unwrap_or_else(|_| "200".to_string())
                .parse()
                .unwrap_or(200),
            flush_interval_ms: env::var("FLUSH_INTERVAL_MS")
                .unwrap_or_else(|_| "200".to_string())
                .parse()
                .unwrap_or(200),
            lock_ttl_ms: env::var("LOCK_TTL_MS")
                .unwrap_or_else(|_| "5000".to_string())
                .parse()
                .unwrap_or(5000),
            max_retry_count: env::var("MAX_RETRY_COUNT")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .unwrap_or(5),
            retry_base_delay_ms: env::var("RETRY_BASE_DELAY_MS")
                .unwrap_or_else(|_| "50".to_string())
                .parse()
                .unwrap_or(50),
            params_inline_threshold_bytes: env::var("PARAMS_INLINE_THRESHOLD_BYTES")
                .unwrap_or_else(|_| "2048".to_string())
                .parse()
                .unwrap_or(2048),
            cache_ttl_seconds: env::var("CACHE_TTL_SECONDS")
                .unwrap_or_else(|_| "3600".to_string())
                .parse()
                .unwrap_or(3600),
            instance_id,
            claim_interval_secs: env::var("CLAIM_INTERVAL_SECS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .unwrap_or(30),
            claim_min_idle_ms: env::var("CLAIM_MIN_IDLE_MS")
                .unwrap_or_else(|_| "60000".to_string())
                .parse()
                .unwrap_or(60000),
            stream_batch_size: env::var("STREAM_BATCH_SIZE")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .unwrap_or(10),
        }
    }
}

/// Write-behind worker configuration
#[derive(Debug, Clone)]
pub struct WriteBehindConfig {
    pub batch_size: usize,
    pub flush_interval_ms: u64,
    pub worker_count: usize,
    pub lock_ttl_ms: u64,
    pub max_retry_count: u32,
    pub retry_base_delay_ms: u64,
    pub params_inline_threshold_bytes: usize,
    pub instance_id: String,
    pub claim_interval_secs: u64,
    pub claim_min_idle_ms: u64,
    pub stream_batch_size: usize,
}

impl From<&Config> for WriteBehindConfig {
    fn from(config: &Config) -> Self {
        Self {
            batch_size: config.batch_size,
            flush_interval_ms: config.flush_interval_ms,
            worker_count: config.worker_count,
            lock_ttl_ms: config.lock_ttl_ms,
            max_retry_count: config.max_retry_count,
            retry_base_delay_ms: config.retry_base_delay_ms,
            params_inline_threshold_bytes: config.params_inline_threshold_bytes,
            instance_id: config.instance_id.clone(),
            claim_interval_secs: config.claim_interval_secs,
            claim_min_idle_ms: config.claim_min_idle_ms,
            stream_batch_size: config.stream_batch_size,
        }
    }
}
