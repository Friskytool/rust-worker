use crate::core::prelude::*;
use futures::stream::{Stream, StreamExt};
use twilight_gateway::Event;
use twilight_cache_inmemory::model::{CachedMember, CachedMessage};
use std::{
    error::Error
};
use crate::Context;
type EventStream = Box<dyn Stream<Item = (u64, Event)> + Send + Sync + Unpin + 'static>;

pub struct EventHandler {
    events: EventStream,
    ctx: Context,
}

impl EventHandler {
    pub fn new(ctx: Context, events: EventStream) -> Self {
        Self {
            events,
            ctx,
        }
    }

    pub async fn start(&mut self) {
        let mut events = self.events.as_mut();
        while let Some((shard_id, event)) = events.next().await {
            

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
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let (cache, http) = {
        let c = ctx.clone();
        (c.cache, c.http)
    };

    match event {
        Event::MessageCreate(message) => {
            let channel_id = message.channel_id;
            let message_id = message.id;

            if message.content == "s!ping" {
                ctx.http.create_message(channel_id).reply(message_id).content("pong")?.exec().await?;
            }
        }

        Event::ShardConnected(_) => {
            println!("Connected on shard {}", shard_id);
        }

        _ => {}
    }

    Ok(())
}