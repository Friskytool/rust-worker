use twilight_model::id::GuildId;
use std::collections::HashMap;
use mongodb::Client;
use std::sync::Arc;
use crate::core::prelude::*;
use crate::db::models::*;
use futures::stream::{TryStreamExt};

#[derive(Clone)]
pub struct PluginConfig {
    plugins: HashMap<GuildId, Vec<String>>,
    mongo_client: Arc<Client>
}

impl PluginConfig {
    pub fn new(mongo_client: Arc<Client>) -> Self {
        PluginConfig {
            plugins: HashMap::new(),
            mongo_client
        }
    }
    
    pub async fn load_plugins(&mut self) -> Result<()> {
        let db = self.mongo_client.database("main");
        let collection = db.collection::<GuildPluginConfig>("Plugins");

        let mut cursor = collection.find(None, None).await?;

        while let Some(plugin_config) = cursor.try_next().await? {
            let guild_id = plugin_config.id;
            let plugins = plugin_config.plugins;
            self.plugins.insert(guild_id, plugins);
            event!(Level::INFO, "Loading plugin..");
        }
        Ok(())
    }
}