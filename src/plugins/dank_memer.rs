use crate::{core::prelude::*, db::models::*};
use dashmap::DashMap;
use deadpool_redis::redis::cmd;
use mongodb::options::InsertOneOptions;
use regex::Regex;
use tracing::{debug, info};
use twilight_model::channel::Message;

async fn get_id_from_name(
    dank: &DankMemer,
    receiver_name: &String,
    ctx: &Context,
    message: &Message,
) -> Result<Option<UserId>> {
    Ok(
        if let Some(receiver_id_val) = dank.cache.get(receiver_name) {
            Some(receiver_id_val.value().clone())
        } else if let Some(r) = ctx
            .cache
            .iter()
            .users()
            .filter(|u| u.name.eq(receiver_name))
            .next()
        {
            dank.cache.insert(receiver_name.to_string(), *r.key());
            Some(*r.key())
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
            .find(|m| m.name.eq(receiver_name))
            .map(|m| m.id)
        {
            dank.cache.insert(receiver_name.to_string(), id);
            Some(id)
        } else {
            info!("Could not find user {}", receiver_name);
            None
        },
    )
}

#[derive(Clone, Debug)]
pub struct DankMemer {
    pub amount_expr: Regex,
    pub item_expr: Regex,
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
            if message.author.id.get() != 270904126974590976
                || !message.content.is_empty()
                || message.embeds.is_empty()
                || message.reference.is_none()
                || message.embeds[0].fields.is_empty()
                || message.embeds[0].fields[0].name != "Shared"
                || message.embeds[0].fields[1].name != "Your Pocket"
            {
                if !message.embeds.is_empty()
                    && message.embeds[0].title == Some("Successful Trade!".to_string())
                {
                    let fields = message.embeds[0].fields.clone();
                    if fields
                        .clone()
                        .into_iter()
                        .filter(|f| f.value.contains("⏣"))
                        .count()
                        != 1
                    {
                        return Ok(());
                    }

                    // store the field value which has ⏣ as amount and the other as item
                    let (amount_field, item_field) = if fields[0].value.contains("⏣") {
                        (fields[0].clone(), fields[1].clone())
                    } else {
                        (fields[1].clone(), fields[0].clone())
                    };

                    let price = self.amount_expr.captures(&amount_field.value).unwrap()[1]
                        .to_string()
                        .replace(",", "");

                    let item_caps = dbg!(self.item_expr.captures(&item_field.value).unwrap());
                    let amount = item_caps[1].to_string().replace(",", "");
                    let item_id = item_caps[2].to_string();
                    let item_name = item_caps[3].to_string();

                    let coll = ctx.db.collection::<ItemTrade>("dank_memer_prices");

                    let amount = amount.parse()?;
                    let price = price.parse()?;
                    debug!("Got trade data: {amount} {item_id} {price}");

                    let mut conn = ctx.redis_pool.get().await.unwrap();

                    cmd("SADD")
                        .arg("dank:items")
                        .arg(item_id.clone())
                        .query_async::<_, ()>(&mut conn)
                        .await?;

                    let _: () = cmd("SET")
                        .arg(format!("dank:item:{}:name", &item_id))
                        .arg(item_name)
                        .query_async::<_, ()>(&mut conn)
                        .await?;

                    coll.insert_one(
                        ItemTrade {
                            amount,
                            price,
                            item_id,
                            value: amount as f64 / price as f64,
                            date: bson::DateTime::now(),
                        },
                        None,
                    )
                    .await?;
                }
                return Ok(());
            }

            let embed = message.embeds.first().unwrap();

            if let Some(sender_id) = ctx
                .cache
                .iter()
                .messages()
                .find(|msg| msg.id() == message.reference.as_ref().unwrap().message_id.unwrap())
                .map(|msg| msg.author())
            {
                let receiver_text = embed.fields[2].name.to_string();
                let receiver_name = &receiver_text[0..receiver_text.len() - 9].to_string();
                let receiver_id: UserId;
                if let Ok(Some(result)) =
                    get_id_from_name(&self, &receiver_name, &ctx, &message).await
                {
                    receiver_id = result;
                } else {
                    return Ok(());
                }
                dbg!(&receiver_id);
                let amount_row = embed.fields[0].value.to_string();
                let amount = self.amount_expr.captures(dbg!(&amount_row)).unwrap()[1]
                    .to_string()
                    .replace(",", "");
                let coll = ctx.db.collection::<TransferStorage>("dank_memer");

                coll.insert_one(
                    dbg!(TransferStorage {
                        sender_id: sender_id.to_string(),
                        reciever_id: receiver_id.to_string(),
                        amount: amount.parse()?,
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
            amount_expr: Regex::new(r"^`⏣ ([\d,]+)` \(\+ ⏣ ([\d,]+) tax\)$").unwrap(),
            item_expr: Regex::new(
                r"^<:Reply:870665583593660476>\*\*([\d,]*)x\*\* <:(\w*):(?:\d*)> \*\*([\w ]*)\*\*$",
            )
            .unwrap(),
            cache: DashMap::new(),
        }
    }
}
