use crate::core::prelude::*;
use crate::core::Plugin;
use mongodb::options::FindOneAndUpdateOptions;
use twilight_model::gateway::payload::incoming::{GuildCreate, GuildDelete, GuildUpdate};
use twilight_model::guild::{Guild, PartialGuild};

#[derive(Debug, Clone)]
pub struct ServerIndexer();

#[async_trait]
impl Plugin for ServerIndexer {
    fn description(&self) -> &'static str {
        "Syncs database information needed for workers cmd handling"
    }

    fn name(&self) -> &'static str {
        "server_indexer"
    }

    async fn on_event(&self, event: Event, ctx: Context) -> Result<()> {
        match event {
            Event::GuildUpdate(e) => {
                let GuildUpdate(partial_guild) = *e;
                let coll = ctx.db.collection::<PartialGuild>("servers");

                coll.find_one_and_update(
                    doc! {"_id": partial_guild.id.get().to_string()},
                    doc! {"$set": bson::to_bson(&partial_guild).unwrap()},
                    Some(FindOneAndUpdateOptions::builder().upsert(true).build()),
                )
                .await?;
            }
            Event::GuildCreate(e) => {
                let GuildCreate(guild) = *e;
                let coll = ctx.db.collection::<Guild>("servers");

                coll.find_one_and_update(
                    doc! {"_id": guild.id.get().to_string() },
                    doc! {"$set": bson::to_bson(&guild).unwrap()},
                    Some(FindOneAndUpdateOptions::builder().upsert(true).build()),
                )
                .await?;
            }
            Event::GuildDelete(e) => {
                let GuildDelete { id, unavailable } = *e;
                if !unavailable {
                    let coll = ctx.db.collection::<Guild>("servers");
                    coll.find_one_and_delete(doc! {"_id": id.0.get().to_string()}, None)
                        .await?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    async fn sync_db(&self, _ctx: &Context) -> Result<()> {
        Ok(())
    }
}
