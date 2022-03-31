use crate::core::prelude::*;
use crate::core::Plugin;
use crate::db::models::AfkUser;
use dashmap::DashMap;
use futures::stream::TryStreamExt;
use twilight_embed_builder::EmbedBuilder;

#[derive(Debug, Clone)]
pub struct Utility {
    cache: DashMap<Id<UserMarker>, String>,
}

#[async_trait]
impl Plugin for Utility {
    fn description(&self) -> &'static str {
        "Sends afk messages when a user is AFK"
    }

    fn name(&self) -> &'static str {
        "utility"
    }

    async fn on_event(&self, event: Event, ctx: Context) -> Result<()> {
        if let Event::MessageCreate(message) = event {
            if message.author.bot || message.mentions.is_empty() {
                return Ok(());
            }

            if let Some(mention) = message
                .mentions
                .iter()
                .filter(|m| self.cache.get(&m.id).is_some() && m.id != message.author.id)
                .next()
            {
                let afk_message = &*self.cache.get(&mention.id).unwrap();

                let _ = ctx
                    .http
                    .create_message(message.channel_id)
                    .embeds(&vec![EmbedBuilder::new()
                        .description(afk_message)
                        .build()
                        .unwrap()])?
                    .exec()
                    .await?;
            }
        }
        Ok(())
    }

    async fn sync_db(&self, ctx: &Context) -> Result<()> {
        let coll = ctx.db.collection::<AfkUser>("afk");
        let mut afk_cursor = coll
            .find(doc! {}, None)
            .await
            .expect("Failed to find afk users");

        let mut users = Vec::new();
        while let Some(afk_user) = afk_cursor.try_next().await? {
            let id = afk_user.get_user_id();
            users.push(id.clone());

            self.cache.insert(id, afk_user.message.clone());
        }

        for item in self.cache.iter_mut() {
            if !users.contains(&item.key()) {
                self.cache.remove(&item.key());
            }
        }

        Ok(())
    }
}

impl Default for Utility {
    fn default() -> Self {
        Utility {
            cache: DashMap::new(),
        }
    }
}
