use crate::core::prelude::*;
use crate::core::Plugin;
use crate::db::models::*;
use dashmap::DashMap;
use mongodb::options::FindOneAndUpdateOptions;

#[derive(Clone, Debug)]
pub struct MessageCounting {
    pub cache: DashMap<GuildId, DashMap<UserId, i64>>,
}

#[async_trait]
impl Plugin for MessageCounting {
    fn name(&self) -> &'static str {
        "message_counting"
    }

    fn description(&self) -> &'static str {
        "Counts the number of messages sent by each user"
    }

    async fn on_event(&self, event: Event, _ctx: Context) -> Result<()> {
        match event {
            Event::MessageCreate(message) => {
                if message.author.bot {
                    return Ok(());
                }
                if let Some(guild_id) = message.guild_id {
                    if self.cache.contains_key(&guild_id) {
                        let cache = self.cache.get_mut(&guild_id).unwrap();

                        if cache.contains_key(&message.author.id) {
                            let mut count = cache.get_mut(&message.author.id).unwrap();
                            *count += 1;
                        } else { 
                            cache.insert(message.author.id, 1);
                        }
                    } else {
                        let cache = DashMap::new();

                        cache.insert(message.author.id, 1);

                        self.cache.insert(guild_id, cache);
                    }
                }
            }
            _ => {}
        };
        Ok(())
    }

    async fn sync_db(&self, context: &Context) -> Result<()> {
        let db = {
            let ctx = context.clone();
            ctx.db
        };

        let coll = db.collection::<MessageCountingUserStorage>("message_counting");

        for row in self.cache.iter() {
            let guild_id = row.key().clone();
            let cache = row.value().clone();

            for g_row in cache.iter() {
                let user_id = g_row.key().clone();
                let count = g_row.value().clone();
                event!(
                    Level::INFO,
                    "Saving message count for user {} in guild {}",
                    user_id,
                    guild_id
                );

                coll.find_one_and_update(
                    doc! {
                        "guild_id": guild_id.get().to_string(),
                        "user_id": user_id.get().to_string(),
                    },
                    doc! {
                        "$inc": {
                            "count": count
                        },
                        "$setOnInsert":{
                            "guild_id": guild_id.get().to_string(),
                            "user_id": user_id.get().to_string(),
                        }
                    },
                    Some(FindOneAndUpdateOptions::builder().upsert(true).build()),
                )
                .await?;
            }
        }
        self.cache.clear();

        Ok(())
    }
}

impl Default for MessageCounting {
    fn default() -> Self {
        Self {
            cache: DashMap::new(),
        }
    }
}
