use serde::Deserialize;
use std::net::IpAddr;
use std::sync::Arc;

use crate::core::Result;

#[derive(Debug, Deserialize, Clone)]
pub struct WorkerConfig {
    pub discord_token: String,
    pub application_id: u64,
    pub database_url: String,
    pub database_port: u16,
    //pub redis: deadpool_redis::Config,
}

impl WorkerConfig {
    pub fn from_env() -> Result<Self> {
        if let Err(e) = dotenv::dotenv() {
            tracing::warn!(
                "Failed to read .env file ({}), checking if environment variables already exist",
                e
            );
        }

        let mut cfg = config::Config::new();

        cfg.set_default("database_url", "localhost")?
            .set_default("database_port", "27017")?;

        cfg.merge(config::Environment::new())?;

        cfg.try_into().map_err(Into::into)
    }
}
