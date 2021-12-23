use crate::core::prelude::*;
use crate::Context;
use futures::stream::StreamExt;
use twilight_gateway::cluster::Events;
use twilight_gateway::Event;
use twilight_model::gateway::payload::incoming::*;
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
                let plugins: Vec<_> = {
                    let r1 = plugin_config.read().await;

                    r1.get_plugins(guild_id).await
                };
                event!(
                    Level::DEBUG,
                    "Got Plugins: ({:#?}) in {}",
                    plugins,
                    guild_id
                );

                for plugin in plugins.iter() {
                    match plugin.on_event(event.clone(), ctx.clone()).await {
                        Ok(()) => {}
                        Err(e) => {
                            event!(Level::ERROR, "Error in plugin: {:#?}", e);
                        }
                    };
                }
            }
        }

        // Guild Based events
        Event::GuildUpdate(update_event) => {
            let GuildUpdate(e) = *update_event.clone();
            let guild_id = e.id;
            let plugins: Vec<_> = {
                let r1 = plugin_config.read().await;

                r1.get_plugins(guild_id).await
            };
            event!(
                Level::DEBUG,
                "Got Plugins (guild update): ({:#?}) in {}",
                plugins,
                guild_id
            );

            for plugin in plugins.iter() {
                match plugin.on_event(event.clone(), ctx.clone()).await {
                    Ok(()) => {}
                    Err(e) => {
                        event!(Level::ERROR, "Error in plugin: {:#?}", e);
                    }
                };
            }
        }
        Event::GuildCreate(create_event) => {
            let GuildCreate(e) = *create_event.clone();
            let guild_id = e.id;
            let plugins: Vec<_> = {
                let r1 = plugin_config.read().await;

                r1.get_plugins(guild_id).await
            };
            event!(
                Level::DEBUG,
                "Got Plugins (guild create): ({:#?}) in {}",
                plugins,
                guild_id
            );

            for plugin in plugins.iter() {
                match plugin.on_event(event.clone(), ctx.clone()).await {
                    Ok(()) => {}
                    Err(e) => {
                        event!(Level::ERROR, "Error in plugin: {:#?}", e);
                    }
                };
            }
        }
        Event::GuildDelete(delete_event) => {
            let GuildDelete { id, unavailable: _ } = *delete_event.clone();
            let plugins: Vec<_> = {
                let r1 = plugin_config.read().await;

                r1.get_plugins(id).await
            };
            event!(
                Level::DEBUG,
                "Got Plugins (guild delete): ({:#?}) in {}",
                plugins,
                id
            );

            for plugin in plugins.iter() {
                match plugin.on_event(event.clone(), ctx.clone()).await {
                    Ok(()) => {}
                    Err(e) => {
                        event!(Level::ERROR, "Error in plugin: {:#?}", e);
                    }
                };
            }
        }
        Event::MemberAdd(add_event) => {
            let MemberAdd(e) = *add_event.clone();
            let guild_id = e.guild_id;
            let plugins: Vec<_> = {
                let r1 = plugin_config.read().await;

                r1.get_plugins(guild_id).await
            };
            event!(
                Level::DEBUG,
                "Got Plugins (member add): ({:#?}) in {}",
                plugins,
                guild_id
            );

            for plugin in plugins.iter() {
                match plugin.on_event(event.clone(), ctx.clone()).await {
                    Ok(()) => {}
                    Err(e) => {
                        event!(Level::ERROR, "Error in plugin: {:#?}", e);
                    }
                };
            }
        }
        Event::MemberRemove(remove_event) => {
            let MemberRemove { guild_id, user: _ } = remove_event.clone();
            let plugins: Vec<_> = {
                let r1 = plugin_config.read().await;

                r1.get_plugins(guild_id).await
            };
            event!(
                Level::DEBUG,
                "Got Plugins (member remove): ({:#?}) in {}",
                plugins,
                guild_id
            );

            for plugin in plugins.iter() {
                match plugin.on_event(event.clone(), ctx.clone()).await {
                    Ok(()) => {}
                    Err(e) => {
                        event!(Level::ERROR, "Error in plugin: {:#?}", e);
                    }
                };
            }
        }
        Event::InviteCreate(create_event) => {
            let guild_id = create_event.guild_id;
            let plugins: Vec<_> = {
                let r1 = plugin_config.read().await;

                r1.get_plugins(guild_id).await
            };
            event!(
                Level::DEBUG,
                "Got Plugins (invite create): ({:#?}) in {}",
                plugins,
                guild_id
            );

            for plugin in plugins.iter() {
                match plugin.on_event(event.clone(), ctx.clone()).await {
                    Ok(()) => {}
                    Err(e) => {
                        event!(Level::ERROR, "Error in plugin: {:#?}", e);
                    }
                };
            }
        }
        Event::InviteDelete(delete_event) => {
            let guild_id = delete_event.guild_id;
            let plugins: Vec<_> = {
                let r1 = plugin_config.read().await;

                r1.get_plugins(guild_id).await
            };
            event!(
                Level::DEBUG,
                "Got Plugins (invite delete): ({:#?}) in {}",
                plugins,
                guild_id
            );

            for plugin in plugins.iter() {
                match plugin.on_event(event.clone(), ctx.clone()).await {
                    Ok(()) => {}
                    Err(e) => {
                        event!(Level::ERROR, "Error in plugin: {:#?}", e);
                    }
                };
            }
        }

        Event::ShardConnected(_) => {
            event!(Level::DEBUG, "Connected on shard {}", shard_id);
        }

        Event::MemberUpdate(_) => {}
        n => {
            event!(Level::DEBUG, "Unknown event: {:#?}", n);
        }
    }

    Ok(())
}
