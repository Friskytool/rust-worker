use crate::core::prelude::*;
use crate::Context;
use futures::stream::StreamExt;
use std::sync::Arc;
use twilight_gateway::cluster::Events;
use twilight_gateway::Event;
pub struct EventHandler {
    events: Events,
    ctx: Context,
}

impl EventHandler {
    pub fn new(ctx: Context, events: Events) -> Self {
        Self { events, ctx }
    }

    pub async fn start(&mut self) {
        while let Some((shard_id, event)) = self.events.next().await {
            // If standby is used we would put it here
            self.ctx.cache.update(&event);

            tokio::spawn(handle_event(shard_id, event, self.ctx.clone()));
        }
    }
}

async fn handle_event(shard_id: u64, event: Event, ctx: Context) -> Result<()> {
    let plugin_config = {
        let c = ctx.clone();
        c.plugin_config
    };

    match &event {
        Event::MessageCreate(message) => {
            if let Some(guild_id) = message.guild_id {
                let plugins: Vec<Arc<Box<dyn Plugin>>> = {
                    let r1 = plugin_config.read().await;

                    r1.get_plugins(guild_id).await
                };
                event!(Level::INFO, "Got Plugins: ({:#?}) in {}", plugins, guild_id);

                for plugin in plugins.iter() {
                    plugin.on_event(event.clone(), ctx.clone()).await;
                }
            }
        }

        Event::ShardConnected(_) => {
            event!(Level::INFO, "Connected on shard {}", shard_id);
        }

        _ => {}
    }

    Ok(())
}
