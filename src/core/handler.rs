use crate::core::prelude::*;
use crate::Context;
use futures::stream::{StreamExt};
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

async fn handle_event(
    shard_id: u64,
    event: Event,
    ctx: Context,
) -> Result<()> {
    let (_cache, http, _plugin_config) = {
        let c = ctx.clone();
        (c.cache, c.http, c.plugin_config)
    };

    match event {
        Event::MessageCreate(message) => {
            let channel_id = message.channel_id;
            let message_id = message.id;
            if message.content == "s.ping" {
                http
                    .create_message(channel_id)
                    .reply(message_id)
                    .content("pong")?
                    .exec()
                    .await?;
            }
        }

        Event::ShardConnected(_) => {
            println!("Connected on shard {}", shard_id);
        }

        _ => {}
    }

    Ok(())
}
