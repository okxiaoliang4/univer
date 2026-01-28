use std::env;
use tracing::info;

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
    pub etcd_endpoints: Vec<String>,
    pub etcd_lease_ttl_seconds: u64,
    pub etcd_registration_ip: Option<String>,
    pub grpc_server_port: u16,
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
            s3_access_key: env::var("S3_ACCESS_KEY_ID")
                .expect("S3_ACCESS_KEY_ID environment variable must be set"),
            s3_secret_key: env::var("S3_SECRET_ACCESS_KEY")
                .expect("S3_SECRET_ACCESS_KEY environment variable must be set"),
            redis_url: env::var("REDIS_URL").expect("REDIS_URL environment variable must be set"),
            awareness_redis_enabled: env::var("AWARENESS_REDIS_ENABLED")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .expect("AWARENESS_REDIS_ENABLED must be a valid bool"),
            awareness_ttl_seconds: env::var("AWARENESS_TTL_SECONDS")
                .unwrap_or_else(|_| "120".to_string())
                .parse()
                .expect("AWARENESS_TTL_SECONDS must be a valid u64"),
            etcd_endpoints: env::var("ETCD_ENDPOINTS")
                .unwrap_or_else(|_| "http://127.0.0.1:2379".to_string())
                .split(',')
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
                .collect(),
            etcd_lease_ttl_seconds: env::var("ETCD_LEASE_TTL_SECONDS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .expect("ETCD_LEASE_TTL_SECONDS must be a valid u64"),
            etcd_registration_ip: env::var("ETCD_REGISTRATION_IP").ok(),
            grpc_server_port: env::var("GRPC_SERVER_PORT")
                .unwrap_or_else(|_| "50051".to_string())
                .parse()
                .expect("GRPC_SERVER_PORT must be a valid u16"),
        }
    }

    pub fn log_summary(&self) {
        info!(
            "Config loaded: database_url={}, server_port={}, grpc_server_port={}, ws_path={}, snapshot_interval={}, s3_endpoint={}, s3_region={}, s3_bucket={}, s3_access_key={}, s3_secret_key={}, redis_url={}, awareness_redis_enabled={}, awareness_ttl_seconds={}, etcd_endpoints={:?}, etcd_lease_ttl_seconds={}, etcd_registration_ip={:?}",
            redact_url(&self.database_url),
            self.server_port,
            self.grpc_server_port,
            self.ws_path,
            self.snapshot_interval,
            self.s3_endpoint,
            self.s3_region,
            self.s3_bucket,
            redact_secret(&self.s3_access_key),
            redact_secret(&self.s3_secret_key),
            redact_url(&self.redis_url),
            self.awareness_redis_enabled,
            self.awareness_ttl_seconds,
            self.etcd_endpoints,
            self.etcd_lease_ttl_seconds,
            self.etcd_registration_ip
        );
    }
}

fn redact_secret(value: &str) -> String {
    if value.is_empty() {
        return "[empty]".to_string();
    }
    let suffix_len = 4.min(value.len());
    let suffix = &value[value.len() - suffix_len..];
    format!("***{}", suffix)
}

fn redact_url(value: &str) -> String {
    if let Some(protocol_pos) = value.find("://") {
        let rest = &value[protocol_pos + 3..];
        if let Some(at_pos) = rest.find('@') {
            let prefix = &value[..protocol_pos + 3];
            let suffix = &rest[at_pos + 1..];
            return format!("{}***:***@{}", prefix, suffix);
        }
    }
    value.to_string()
}
