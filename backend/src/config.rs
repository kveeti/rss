use anyhow::Context;
use dotenv::dotenv;
use serde::Deserialize;
use tracing::warn;

use crate::api::ApiConfig;

#[derive(Deserialize)]
pub struct Config {
    pub database_url: String,
    pub host: String,
    #[serde(default)]
    pub frontend_dir: Option<String>,
}

impl Config {
    pub fn new() -> Result<Self, anyhow::Error> {
        let _ = dotenv().map_err(|err| warn!("error loading .env: {:?}", err));
        envy::from_env::<Config>().context("invalid environment variables")
    }
}

impl From<Config> for ApiConfig {
    fn from(config: Config) -> Self {
        ApiConfig {
            host: config.host,
            frontend_dir: config.frontend_dir,
        }
    }
}
