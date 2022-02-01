use crate::core::prelude::*;
use deadpool_redis::redis::cmd;
use std::sync::Arc;
use tracing::error;
use twilight_model::id::GuildId;

pub struct PluginConfig {
    pub plugins: Arc<Vec<Arc<Box<dyn Plugin>>>>,
}

impl PluginConfig {
    pub fn new(plugins: Arc<Vec<Arc<Box<dyn Plugin>>>>) -> Self {
        PluginConfig { plugins }
    }

    pub async fn get_plugins(&self, ctx: &Context, guild_id: GuildId) -> Vec<Arc<Box<dyn Plugin>>> {
        let mut result: Vec<_> = Vec::new();
        let mut conn = ctx.redis_pool.get().await.unwrap();
        let plugins: Vec<String> = cmd("SMEMBERS")
            .arg(&[format!("plugins:{}", guild_id)])
            .query_async(&mut conn)
            .await
            .unwrap_or_else(|_| {
                error!("Failed to get plugins for guild {}", guild_id);
                Vec::new()
            });

        {
            for plugin_name in plugins {
                if let Some(plugin) = self
                    .plugins
                    .iter()
                    .find(|p| p.name().to_string() == plugin_name)
                {
                    result.push(plugin.clone());
                }
            }
        }
        for plugin in self.plugins.iter() {
            result.push(plugin.clone());
        }
        result
    }
}
