use serde::{Deserialize, Serialize};
use crate::core::prelude::*;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct GuildPluginConfig {
    pub id: GuildId,
    pub plugins: Vec<String>,
}