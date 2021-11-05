use serde::{Deserialize, Serialize};
use crate::core::prelude::*;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct GuildPluginConfig {
    pub id: std::num::NonZeroU64,
    pub plugins: Vec<String>,
}