use anyhow::Context;
use dotenv::dotenv;
use serde::Deserialize;
use tracing::warn;

use crate::api::ApiConfig;

#[derive(Deserialize)]
pub struct Config {
    pub database_url: String,
    pub front_base_url: String,
    pub host: String,
}

impl Config {
    pub fn new() -> Result<Self, anyhow::Error> {
        let _ = dotenv().map_err(|err| warn!("error loading .env: {:?}", err));

        return Ok(envy::from_env::<Config>().context("invalid environment variables")?);
    }
}

impl From<Config> for ApiConfig {
    fn from(config: Config) -> Self {
        ApiConfig {
            front_base_url: config.front_base_url,
            host: config.host,
        }
    }
}
