use crate::core::prelude::*;
use crate::core::Plugin;
use crate::db::models::Giveaway;
use chrono::{Duration as ChronoDuration, Utc};
use futures::stream::TryStreamExt;
use rand::seq::SliceRandom;
use tokio::time::sleep;
use tracing::error;
use tracing::info;
use twilight_embed_builder::EmbedBuilder;

#[derive(Clone, Debug)]
pub struct Giveaways {}

#[async_trait]
impl Plugin for Giveaways {
    fn name(&self) -> &'static str {
        "giveaways"
    }

    fn description(&self) -> &'static str {
        "Schedules giveaways"
    }

    async fn on_event(&self, _event: Event, _ctx: Context) -> Result<()> {
        Ok(())
    }

    async fn sync_db(&self, ctx: &Context) -> Result<()> {
        let giveaway_coll = ctx.db.collection::<Giveaway>("giveaways");
        let timestamp = Utc::now() + ChronoDuration::seconds(60);
        let timestamp: bson::DateTime = timestamp.into();

        let mut giveaway_cursor = giveaway_coll
            .find(doc! {"active":true, "end":{"$lte":timestamp}}, None)
            .await
            .expect("Failed to find giveaways");

        let mut results = Vec::new();
        while let Some(giveaway) = giveaway_cursor.try_next().await? {
            results.push(giveaway._id);
            let ctx = ctx.clone();
            // Editing the messages
            tokio::spawn(async move {
                let (http, mut conn) = {
                    let http = ctx.http.clone();
                    let conn = ctx.redis_pool.get().await.unwrap();
                    (http, conn)
                };
                info!("Remaining: {:#?}", giveaway.get_duration_remaining());
                sleep(giveaway.get_duration_remaining()).await;
                info!("Ending...");

                let users: Vec<Id<UserMarker>> = cmd("smembers")
                    .arg(&[giveaway.get_store_key()])
                    .query_async(&mut conn)
                    .await
                    .unwrap_or_else(|_| {
                        error!("Failed to get users for giveaway {}", giveaway._id);
                        Vec::new()
                    })
                    .into_iter()
                    .map(|user: String| Id::new(user.parse().unwrap()))
                    .collect();

                let winners = if !users.is_empty() {
                    if users.len() > giveaway.winners {
                        users
                            .choose_multiple(&mut rand::thread_rng(), giveaway.winners)
                            .cloned()
                            .collect()
                    } else {
                        users
                    }
                } else {
                    Vec::new()
                };

                let mut description = format!("{}\n\n", giveaway.get_content());

                let winner_str: String;
                if !winners.is_empty() {
                    let _: () = cmd("expire")
                        .arg(&[giveaway.get_store_key(), "604800".to_string()])
                        .query_async(&mut conn)
                        .await
                        .expect("Redis cmd failed"); // clearly the command worked

                    winner_str = winners
                        .iter()
                        .map(|user| format!("<@{}>", user.get()))
                        .reduce(|acc, user| format!("{}, {}", acc, user))
                        .unwrap();
                    description += &format!("Winners: {}", winner_str);
                } else {
                    winner_str = "Nobody".to_string();
                    description += "No one won";
                };

                let embed = EmbedBuilder::new()
                    .title("Giveaway Ended")
                    .description(description)
                    .build()
                    .expect("could not construct embed for giveaway");

                if let Err(why) = http
                    .update_message(giveaway.get_channel_id(), giveaway.get_message_id())
                    .embeds(Some(&[dbg!(embed)]))
                    .expect("Could not construct update embed for giveaway")
                    .components(Some(&[]))
                    .expect("Could not construct update components for giveaway")
                    .exec()
                    .await
                {
                    event!(Level::ERROR, "Failed to update giveaway message: {}", why);
                } else {
                    info!("Successfully updated giveaway message");
                    http.create_message(giveaway.get_channel_id())
                        .content(&format!(
                            "{} has won the giveaway for `{}`",
                            &winner_str, &giveaway.prize
                        ))
                        .expect("Could not create content for end message")
                        .components(&[Component::ActionRow(ActionRow {
                            components: vec![Component::Button(Button {
                                style: ButtonStyle::Link,
                                url: Some(format!(
                                    "https://discord.com/channels/{}/{}/{}",
                                    giveaway.get_guild_id(),
                                    giveaway.get_channel_id(),
                                    giveaway.get_message_id()
                                )),
                                label: Some("Jump".to_string()),
                                custom_id: None,
                                disabled: false,
                                emoji: None,
                            })],
                        })])
                        .expect("Could not construct components for end message")
                        .allowed_mentions(Some(
                            &AllowedMentions::builder().user_ids(winners).build(),
                        ))
                        .exec()
                        .await
                        .ok();
                }
            });
        }
        if !results.is_empty() {
            giveaway_coll
                .update_many(
                    doc! {"_id":{ "$in":results }},
                    doc! {"$set":doc! {"active":false}},
                    None,
                )
                .await
                .expect("Failed to update giveaways");
        }
        Ok(())
    }
}

impl Default for Giveaways {
    fn default() -> Self {
        Giveaways {}
    }
}
