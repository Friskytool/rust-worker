use crate::core::prelude::*;
use crate::db::models::*;
use futures::stream::TryStreamExt;
use std::collections::HashMap;
use std::sync::Arc;
use twilight_model::id::GuildId;

pub struct PluginConfig {
    pub plugins: Arc<Vec<Arc<Box<dyn Plugin>>>>,
    plugin_cache: HashMap<GuildId, Vec<String>>,
}

impl PluginConfig {
    pub fn new(plugins: Arc<Vec<Arc<Box<dyn Plugin>>>>) -> Self {
        PluginConfig {
            plugins,
            plugin_cache: HashMap::new(),
        }
    }

    pub async fn load_cache(&mut self, ctx: Context) -> Result<()> {
        let collection = ctx.db.collection::<GuildPluginConfig>("plugins");

        let mut cursor = collection.find(None, None).await?;
        event!(Level::INFO, "Loading plugins");
        while let Some(plugin_config) = cursor.try_next().await? {
            let guild_id = GuildId(plugin_config.id.parse::<std::num::NonZeroU64>()?);

            event!(
                Level::INFO,
                "Recieved plugins guild {}: {}",
                guild_id,
                plugin_config.plugins.join(", ")
            );
            self.plugin_cache.insert(guild_id, plugin_config.plugins);
        }
        Ok(())
    }

    pub async fn get_plugins(&self, guild_id: GuildId) -> Vec<Arc<Box<dyn Plugin>>> {
        let mut result: Vec<_> = Vec::new();
        if let Some(plugins) = self.plugin_cache.get(&guild_id) {
            for plugin_name in plugins {
                if let Some(plugin) = self.plugins.iter().find(|p| p.name() == *plugin_name) {
                    result.push(plugin.clone());
                }
            }
        }
        result
    }
}
