use crate::core::prelude::*;
use crate::Context;
use futures::stream::StreamExt;
use lapin::options::BasicAckOptions;
use lapin::Consumer;

use serde::de::DeserializeSeed;
use tracing::error;
use twilight_model::gateway::event::{GatewayEvent, GatewayEventDeserializer};
use twilight_model::gateway::payload::incoming::*;
use twilight_model::gateway::OpCode;
pub struct EventHandler {
    consumer: Consumer,
    ctx: Context,
}

impl EventHandler {
    pub fn new(ctx: Context, consumer: Consumer) -> Self {
        Self { consumer, ctx }
    }

    pub async fn start(&mut self) {
        while let Some(delivery) = self.consumer.next().await {
            let mut delivery = delivery.expect("error in consumer");

            let (op, seq, event_type) = {
                let json = std::str::from_utf8_mut(&mut delivery.data).expect("invalid utf8");
                if let Some(deserializer) = GatewayEventDeserializer::from_json(json) {
                    let (op, seq, event_type) = deserializer.into_parts();

                    // Unfortunately lifetimes and mutability requirements
                    // conflict here if we return an immutable reference to the
                    // event type, so we're going to have to take ownership of
                    // this if we don't want to do anything too dangerous. It
                    // should be a good trade-off either way.
                    // Via twilight-rs/gateway
                    (op, seq, event_type.map(ToOwned::to_owned))
                } else {
                    error!(json = json, "received payload without opcode",);
                    continue; // Shouldn't be doing this but gateway should be already verifying data
                }
            };

            let gateway_event = if op == OpCode::HeartbeatAck as u8 {
                GatewayEvent::HeartbeatAck
            } else if op == OpCode::Reconnect as u8 {
                GatewayEvent::Reconnect
            } else {
                // Json gateway deserializer from twilight

                let gateway_deserializer =
                    GatewayEventDeserializer::new(op, seq, event_type.as_deref());

                let mut json_deserializer =
                    serde_json::Deserializer::from_slice(&mut delivery.data);

                gateway_deserializer
                    .deserialize(&mut json_deserializer)
                    .expect("Could not deserialize ws data to object")
            };

            let event = Event::from(gateway_event);

            self.ctx.cache.update(&event);
            delivery.ack(BasicAckOptions::default()).await.expect("ack"); // We've got the event the rest is up to sentry to monitor
            tokio::spawn(handle_event(event, self.ctx.clone()));
        }
    }
}

async fn handle_event(event: Event, ctx: Context) -> Result<()> {
    let plugin_config = {
        let c = ctx.clone();
        c.plugin_config
    };

    match &event {
        Event::MessageCreate(message) => {
            if let Some(guild_id) = message.guild_id {
                let plugins: Vec<_> = {
                    let r1 = plugin_config.read().await;

                    r1.get_plugins(&ctx, guild_id).await
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
                            event!(
                                Level::ERROR,
                                "error in plugin ({}): {:#?}",
                                e,
                                plugin.name()
                            );
                        }
                    };
                }
            }
        }

        Event::MessageUpdate(message) => {
            if let Some(guild_id) = message.guild_id {
                let plugins: Vec<_> = {
                    let r1 = plugin_config.read().await;

                    r1.get_plugins(&ctx, guild_id).await
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
                            event!(
                                Level::ERROR,
                                "error in plugin ({}): {:#?}",
                                e,
                                plugin.name()
                            );
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

                r1.get_plugins(&ctx, guild_id).await
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
                        event!(
                            Level::ERROR,
                            "error in plugin ({}): {:#?}",
                            e,
                            plugin.name()
                        );
                    }
                };
            }
        }
        Event::GuildCreate(create_event) => {
            let GuildCreate(e) = *create_event.clone();
            let guild_id = e.id;
            let plugins: Vec<_> = {
                let r1 = plugin_config.read().await;

                r1.get_plugins(&ctx, guild_id).await
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
                        event!(
                            Level::ERROR,
                            "error in plugin ({}): {:#?}",
                            e,
                            plugin.name()
                        );
                    }
                };
            }
        }
        Event::GuildDelete(delete_event) => {
            let GuildDelete { id, .. } = delete_event.clone();
            let plugins: Vec<_> = {
                let r1 = plugin_config.read().await;

                r1.get_plugins(&ctx, id).await
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
                        event!(
                            Level::ERROR,
                            "error in plugin ({}): {:#?}",
                            e,
                            plugin.name()
                        );
                    }
                };
            }
        }
        Event::MemberAdd(add_event) => {
            let MemberAdd(e) = *add_event.clone();
            let guild_id = e.guild_id;
            let plugins: Vec<_> = {
                let r1 = plugin_config.read().await;

                r1.get_plugins(&ctx, guild_id).await
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
                        event!(
                            Level::ERROR,
                            "error in plugin ({}): {:#?}",
                            e,
                            plugin.name()
                        );
                    }
                };
            }
        }
        Event::MemberRemove(remove_event) => {
            let MemberRemove { guild_id, .. } = remove_event.clone();
            let plugins: Vec<_> = {
                let r1 = plugin_config.read().await;

                r1.get_plugins(&ctx, guild_id).await
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
                        event!(
                            Level::ERROR,
                            "error in plugin ({}): {:#?}",
                            e,
                            plugin.name()
                        );
                    }
                };
            }
        }
        Event::InviteCreate(create_event) => {
            let guild_id = create_event.guild_id;
            let plugins: Vec<_> = {
                let r1 = plugin_config.read().await;

                r1.get_plugins(&ctx, guild_id).await
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
                        event!(
                            Level::ERROR,
                            "error in plugin ({}): {:#?}",
                            e,
                            plugin.name()
                        );
                    }
                };
            }
        }
        Event::InviteDelete(delete_event) => {
            let guild_id = delete_event.guild_id;
            let plugins: Vec<_> = {
                let r1 = plugin_config.read().await;

                r1.get_plugins(&ctx, guild_id).await
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
                        event!(
                            Level::ERROR,
                            "error in plugin ({}): {:#?}",
                            e,
                            plugin.name()
                        );
                    }
                };
            }
        }

        Event::ReactionAdd(e) => {
            if let Some(guild_id) = e.guild_id {
                let plugins: Vec<_> = {
                    let r1 = plugin_config.read().await;

                    r1.get_plugins(&ctx, guild_id).await
                };
                event!(
                    Level::DEBUG,
                    "Got Plugins (reaction add): ({:#?}) in {}",
                    plugins,
                    guild_id
                );

                for plugin in plugins.iter() {
                    match plugin.on_event(event.clone(), ctx.clone()).await {
                        Ok(()) => {}
                        Err(e) => {
                            event!(
                                Level::ERROR,
                                "error in plugin ({}): {:#?}",
                                e,
                                plugin.name()
                            );
                        }
                    };
                }
            }
        }
        Event::ShardConnected(_) => {
            event!(Level::DEBUG, "Connected? (this shouldn't be printing)");
        }

        Event::MemberUpdate(_) => {}
        n => {
            event!(Level::DEBUG, "Unknown event: {:#?}", n);
        }
    }

    Ok(())
}
