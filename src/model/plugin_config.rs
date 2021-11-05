use twilight_model::id::GuildId;
use std::collections::HashMap;
use std::sync::Arc;
use crate::core::prelude::*;
use crate::db::models::*;
use futures::stream::{TryStreamExt};
use deadpool_redis::redis::cmd;

pub struct PluginConfig {
    plugins: Arc<Vec<Arc<Box<dyn Plugin>>>>,
    plugin_cache: HashMap<GuildId, Vec<String>>
}

impl PluginConfig {
    pub fn new(plugins: Arc<Vec<Arc<Box<dyn Plugin>>>>) -> Self {
        PluginConfig {
            plugins: plugins,
            plugin_cache: HashMap::new()
        }
    }
    
    pub async fn load_cache(&mut self, ctx: Context) -> Result<()> {
        let collection = ctx.db.collection::<GuildPluginConfig>("Plugins");

        let mut cursor = collection.find(None, None).await?;
        event!(Level::INFO, "Loading plugins");
        while let Some(plugin_config) = cursor.try_next().await? {
            let guild_id = GuildId(plugin_config.id);
        
            event!(Level::INFO, "Recieved plugins guild {}: {}", guild_id, plugin_config.plugins.join(", "));
            self.plugin_cache.insert(guild_id, plugin_config.plugins);
        }
        Ok(())
    }

    pub async fn get_plugins(&self, guild_id: GuildId) -> Vec<Arc<Box<dyn Plugin>>> {
        let mut result : Vec<_> = Vec::new();
        for plugin in self.plugins.iter() {
            if self.plugin_cache.contains_key(&guild_id) {
                if self.plugin_cache.get(&guild_id).unwrap().contains(&plugin.name()) {
                    result.push(plugin.clone());
                }
            }
        };
        result
    }
}