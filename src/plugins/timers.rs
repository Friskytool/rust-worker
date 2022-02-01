use crate::core::prelude::*;
use crate::core::Plugin;
use crate::db::models::Timer;
use chrono::{Duration as ChronoDuration, Utc};
use futures::stream::TryStreamExt;
use mongodb::bson::oid::ObjectId;
use tokio::time::sleep;
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
            .find(doc! {"active":true, "end": {"$lte":timestamp}}, None)
            .await
            .expect("Failed to find timers");

        let mut results: Vec<ObjectId> = Vec::new();
        while let Some(timer) = timer_cursor.try_next().await? {
            info!("{:#?}", &timer);
            results.push(timer._id);
            let ctx = ctx.clone();
            tokio::spawn(async move {
                let http = {
                    let http = ctx.http.clone();
                    http
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
                    .embeds(&vec![dbg!(embed)])
                    .expect("Could not construct update embed for timer")
                    .exec()
                    .await
                {
                    event!(Level::ERROR, "Failed to update timer message: {}", why);
                };
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
