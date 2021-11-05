use crate::core::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct GuildPluginConfig {
    pub id: std::num::NonZeroU64,
    pub plugins: Vec<String>,
}
