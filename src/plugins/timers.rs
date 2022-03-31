use std::collections::HashMap;

use crate::core::prelude::*;
use crate::core::Plugin;
use crate::db::models::Timer;
use chrono::{Duration as ChronoDuration, Utc};
use futures::stream::TryStreamExt;
use mongodb::bson::oid::ObjectId;
use tagscript::Adapter;
use tokio::time::sleep;
use tracing::error;
use tracing::info;
use twilight_embed_builder::EmbedBuilder;

#[derive(Clone, Debug)]
pub struct Timers {}

#[async_trait]
impl Plugin for Timers {
    fn name(&self) -> &'static str {
        "timers"
    }

    fn description(&self) -> &'static str {
        "Schedules timers"
    }

    async fn on_event(&self, _event: Event, _ctx: Context) -> Result<()> {
        Ok(())
    }

    async fn sync_db(&self, ctx: &Context) -> Result<()> {
        let timer_coll = ctx.db.collection::<Timer>("timers");
        let timestamp = Utc::now() + ChronoDuration::seconds(60);
        let timestamp: bson::DateTime = timestamp.into();

        let mut timer_cursor = timer_coll
            .find(doc! {"active":true, "end":{"$lte":timestamp}}, None)
            .await
            .expect("Failed to find timers");

        let mut results: Vec<ObjectId> = Vec::new();
        while let Some(timer) = timer_cursor.try_next().await? {
            results.push(timer._id);
            let ctx = ctx.clone();
            // Editing the messages
            tokio::spawn(async move {
                let (http, mut conn) = {
                    let http = ctx.http.clone();
                    let conn = ctx.redis_pool.get().await.unwrap();
                    (http, conn)
                };
                info!("Remaining: {:#?}", timer.get_duration_remaining());
                sleep(timer.get_duration_remaining()).await;
                info!("Ending...");
                let embed = EmbedBuilder::new()
                    .title("Timer Ended")
                    .description(format!("{}", timer.get_content()))
                    .build()
                    .expect("could not construct embed for timer");

                if let Err(why) = http
                    .update_message(timer.get_channel_id(), timer.get_message_id())
                    .embeds(Some(&[embed]))
                    .expect("Could not construct update embed for timer")
                    .components(Some(&[]))
                    .expect("Could not construct update components for timer")
                    .exec()
                    .await
                {
                    event!(Level::ERROR, "Failed to update timer message: {}", why);
                } else {
                    info!("Successfully updated timer message");
                    let mut seed_variables: HashMap<String, Adapter> = HashMap::new();
                    seed_variables.insert("title".into(), Adapter::String(timer.title.clone()));
                    seed_variables.insert(
                        "host".into(),
                        Adapter::String(format!("<@{}>", timer.get_host_id().get())),
                    );
                    seed_variables.insert(
                        "channel".into(),
                        Adapter::String(format!("<#{}>", timer.get_channel_id().get())),
                    );
                    seed_variables.insert(
                        "link".into(),
                        Adapter::String(format!(
                            "<https://discord.com/channels/{}/{}/{}>",
                            timer.get_guild_id().get(),
                            timer.get_channel_id().get(),
                            timer.get_message_id().get()
                        )),
                    );
                    dbg!(&timer.end_message);
                    dbg!(&seed_variables);
                    let end_message = ctx
                        .interpreter
                        .process(timer.end_message.clone(), Some(seed_variables), Some(2000))
                        .expect("Tagscript processing failed");
                    http.create_message(timer.get_channel_id())
                        .content(&end_message.body.unwrap())
                        .expect("Could not create content for end message")
                        .components(&[Component::ActionRow(ActionRow {
                            components: vec![Component::Button(Button {
                                style: ButtonStyle::Link,
                                url: Some(format!(
                                    "https://discord.com/channels/{}/{}/{}",
                                    timer.get_guild_id(),
                                    timer.get_channel_id(),
                                    timer.get_message_id()
                                )),
                                label: Some("Jump".to_string()),
                                custom_id: None,
                                disabled: false,
                                emoji: None,
                            })],
                        })])
                        .expect("Could not construct components for end message")
                        .allowed_mentions(Some(&AllowedMentions::builder().build()))
                        .exec()
                        .await
                        .ok();
                }

                let users: Vec<String> = cmd("smembers")
                    .arg(&[timer.get_store_key()])
                    .query_async(&mut conn)
                    .await
                    .unwrap_or_else(|_| {
                        error!("Failed to get users for timer {}", timer._id);
                        Vec::new()
                    });

                if !users.is_empty() {
                    let mut messages = Vec::new();
                    for chunk in users.chunks(86) {
                        // this is some random number that works
                        let content = chunk
                            .into_iter()
                            .map(|u| format!("<@{}>", u))
                            .collect::<Vec<String>>()
                            .join("");

                        match http
                            .create_message(timer.get_channel_id())
                            .content(&content)
                            .expect("Could not create message")
                            .exec()
                            .await
                        {
                            Err(why) => {
                                error!("Failed to send message: {}", why);
                                break;
                            }
                            Ok(raw_msg) => {
                                let msg = raw_msg.model().await.unwrap();
                                messages.push(msg.id);
                            }
                        }
                    }
                    if !messages.is_empty() {
                        if messages.len() == 1 {
                            http.delete_message(timer.get_channel_id(), messages[0])
                                .exec()
                                .await
                                .ok();
                        } else {
                            for chunk in messages.chunks(100) {
                                if let Err(why) = http
                                    .delete_messages(timer.get_channel_id(), chunk)
                                    .exec()
                                    .await
                                {
                                    error!("Failed to delete messages: {}", why);
                                    break;
                                }
                            }
                        }
                    }
                    let _: () = cmd("del")
                        .arg(&[timer.get_store_key()])
                        .query_async(&mut conn)
                        .await
                        .expect("Redis cmd failed"); // clearly the command worked
                }
            });
        }
        if !results.is_empty() {
            timer_coll
                .update_many(
                    doc! {"_id":{ "$in":results }},
                    doc! {"$set":doc! {"active":false}},
                    None,
                )
                .await
                .expect("Failed to update timers");
        }
        Ok(())
    }
}

impl Default for Timers {
    fn default() -> Self {
        Timers {}
    }
}
