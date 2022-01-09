use crate::{core::prelude::*, db::models::*};
use dashmap::DashMap;
use mongodb::options::InsertOneOptions;
use regex::Regex;
use tracing::info;
use twilight_model::channel::Message;

#[derive(Clone, Debug)]
pub struct DankMemer {
    pub share_expr: Regex,
    pub gift_expr: Regex,
    pub cache: DashMap<String, UserId>,
}

#[async_trait]
impl Plugin for DankMemer {
    fn name(&self) -> &'static str {
        "dank_memer"
    }

    fn description(&self) -> &'static str {
        "Tracking dank memer data"
    }

    async fn sync_db(&self, _ctx: &Context) -> Result<()> {
        Ok(())
    }

    async fn on_event(&self, event: Event, ctx: Context) -> Result<()> {
        if let Event::MessageCreate(message) = event {
            // if message.author.id.get() != 270904126974590976 || message.content.is_empty() || !message.embeds.is_empty() {
            //     return Ok(());
            // }

            if self.share_expr.is_match(&message.content) {
                let caps = self.share_expr.captures(&message.content).unwrap();
                let (sender, reciever_name, amount) = (
                    caps.get(2).unwrap().as_str(),
                    caps.get(3).unwrap().as_str(),
                    caps.get(4).unwrap().as_str(),
                );

                let reciever = if let Some(reciever_id) = self.cache.get(reciever_name) {
                    *reciever_id.value()
                } else if let Some(r) = ctx
                    .cache
                    .iter()
                    .users()
                    .filter(|u| u.name.eq(reciever_name))
                    .next()
                {
                    self.cache.insert(reciever_name.to_string(), *r.key());
                    *r.key()
                } else if let Some(id) = ctx
                    .http
                    .channel_messages(message.channel_id)
                    .before(message.id)
                    .limit(10)
                    .unwrap()
                    .exec()
                    .await?
                    .model()
                    .await
                    .unwrap()
                    .into_iter()
                    .flat_map(|m: Message| m.mentions)
                    .find(|m| m.name.eq(reciever_name))
                    .map(|m| m.id)
                {
                    self.cache.insert(reciever_name.to_string(), id);
                    id
                } else {
                    info!("Could not find user {}", reciever_name);
                    return Ok(());
                };

                let coll = ctx.db.collection::<TransferStorage>("dank_memer");

                coll.insert_one(
                    dbg!(TransferStorage {
                        sender_id: sender.to_string(),
                        reciever_id: reciever.to_string(),
                        amount: amount.replace(",", "").parse()?,
                        timestamp: message.timestamp,
                        channel_id: message.channel_id.to_string(),
                        guild_id: message.guild_id.unwrap().to_string(),
                    }),
                    InsertOneOptions::builder().build(),
                )
                .await?;
            }
        }
        Ok(())
    }
}

impl Default for DankMemer {
    fn default() -> Self {
        Self {
            share_expr: Regex::new(r#"^(<@!?([0-9]+)>) You gave (.*?) \*\*⏣ (.*?)\*\* \(and paid ⏣ (.*?) tax\), now you have ⏣ (.*?) and they've got ⏣ (.*?)$"#).unwrap(),
            gift_expr: Regex::new(r#"(.*?) You gave (.*?) \*\*(.*?)\*\* (.*?), now"#).unwrap(),
            cache: DashMap::new(),
        }
    }
}
