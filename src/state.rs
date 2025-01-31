use figment::{providers::Env, Figment};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct AppConfig {
    // pub influxdb: InfluxDbConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct InfluxDbConfig {
    // pub url: String,
    // pub token: String,
    // pub org: String,
    // pub bucket: String,
}

pub struct AppState {
    // pub config: AppConfig,
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

        // Self { config }
        Self {}
    }
}
