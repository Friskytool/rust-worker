use crate::core::prelude::*;
use crate::core::{Plugin};

#[derive(Clone, Debug)]
pub struct MessageCounting();

#[async_trait]
impl Plugin for MessageCounting {

    fn name(&self) -> String {
        "MessageCounting".to_string()
    }

    async fn on_event(&self, event: Event, _ctx: Context) {
        match event {
            Event::MessageCreate(message) => {
                if let Some(guild_id) = message.guild_id {
                    event!(Level::INFO, "messagecounting - guild id: {}", guild_id);
                }
            }
            _ => {}
        }
    }
}