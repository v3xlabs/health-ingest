use async_std::sync::Mutex;
use figment::{providers::Env, Figment};
use serde::Deserialize;
use tracing::{debug, error, info};
use url::{ParseError, Url};

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub influxdb_url: String,
    pub influxdb_token: String,
    pub influxdb_org: String,
    pub influxdb_bucket: String,
}

pub struct AppState {
    pub config: AppConfig,
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AppState")
    }
}

impl AppState {
    pub async fn new() -> Self {
        let config: AppConfig = Figment::new()
            .merge(Env::raw().split("_").lowercase(true))
            .extract()
            .unwrap_or_else(|error| {
                tracing::error!("Failed to load configuration: {}", error);
                std::process::exit(1);
            });

        Self { config }
    }
}
