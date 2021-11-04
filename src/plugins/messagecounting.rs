use crate::core::prelude::*;
use crate::core::{Plugin};

#[derive(Clone)]
pub struct MessageCounting();

#[async_trait]
impl Plugin for MessageCounting {
    async fn on_event(&self, event: Event, ctx: Context) {
        match event {
            Event::MessageCreate(message) => {
                if let Some(guild_id) = message.guild_id {
                    event!(Level::INFO, "guild id: {}", guild_id);
                }
            }
            _ => {}
        }
    }
}