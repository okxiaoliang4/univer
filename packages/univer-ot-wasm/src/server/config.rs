use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub server_port: u16,
    pub ws_path: String,
    pub snapshot_interval: u64,
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
        }
    }
}
