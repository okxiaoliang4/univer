use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub server_port: u16,
    pub ws_path: String,
    pub snapshot_interval: u64,
    pub s3_endpoint: String,
    pub s3_region: String,
    pub s3_bucket: String,
    pub s3_access_key: String,
    pub s3_secret_key: String,
    pub redis_url: String,
    pub awareness_redis_enabled: bool,
    pub awareness_ttl_seconds: u64,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            database_url: env::var("DATABASE_URL")
                .expect("DATABASE_URL environment variable must be set"),
            server_port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .expect("SERVER_PORT must be a valid u16"),
            ws_path: env::var("WS_PATH").unwrap_or_else(|_| "/ws".to_string()),
            snapshot_interval: env::var("SNAPSHOT_INTERVAL")
                .unwrap_or_else(|_| "50".to_string())
                .parse()
                .expect("SNAPSHOT_INTERVAL must be a valid u64"),
            s3_endpoint: env::var("S3_ENDPOINT")
                .expect("S3_ENDPOINT environment variable must be set"),
            s3_region: env::var("S3_REGION").expect("S3_REGION environment variable must be set"),
            s3_bucket: env::var("S3_BUCKET").expect("S3_BUCKET environment variable must be set"),
            s3_access_key: env::var("S3_ACCESS_KEY")
                .expect("S3_ACCESS_KEY environment variable must be set"),
            s3_secret_key: env::var("S3_SECRET_KEY")
                .expect("S3_SECRET_KEY environment variable must be set"),
            redis_url: env::var("REDIS_URL").expect("REDIS_URL environment variable must be set"),
            awareness_redis_enabled: env::var("AWARENESS_REDIS_ENABLED")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .expect("AWARENESS_REDIS_ENABLED must be a valid bool"),
            awareness_ttl_seconds: env::var("AWARENESS_TTL_SECONDS")
                .unwrap_or_else(|_| "120".to_string())
                .parse()
                .expect("AWARENESS_TTL_SECONDS must be a valid u64"),
        }
    }
}
