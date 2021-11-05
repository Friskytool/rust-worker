use serde::Deserialize;

use crate::core::Result;

#[derive(Debug, Deserialize, Clone)]
pub struct WorkerConfig {
    pub discord_token: String,
    pub application_id: u64,
    pub mongo_uri: String,
    pub mongo_db: String,
    #[serde(default)]
    pub redis: deadpool_redis::Config,
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

        cfg.set_default("mongo_uri", "mongodb://localhost:27017")?;
        
        cfg.merge(config::Environment::new())?;

        cfg.try_into().map_err(Into::into)
    }
}
