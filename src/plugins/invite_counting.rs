use crate::core::prelude::*;
use crate::core::Plugin;
pub use crate::db::models::{
    GuildInviteStorage, JoinStorage, LeaveStorage, MongoInvite, UserInviteStorage,
};
use mongodb::options::{FindOneAndUpdateOptions, ReturnDocument};
use twilight_model::gateway::payload::incoming::*;

#[derive(Debug, Clone)]
pub struct InviteCounting();

#[async_trait]
impl Plugin for InviteCounting {
    fn name(&self) -> &'static str {
        "invite_counting"
    }

    fn description(&self) -> &'static str {
        "Tracks the invites for a server"
    }
    async fn on_event(&self, event: Event, ctx: Context) -> Result<()> {
        let http = {
            let c = ctx.clone();
            c.http
        };
        let coll = ctx.db.collection::<GuildInviteStorage>("invites");
        match event {
            Event::InviteCreate(e) => {
                event!(Level::INFO, "Invite created");
                coll.find_one_and_update(
                    doc! { "doctype":"invite_storage", "guild_id": e.guild_id.get().to_string() },
                    doc! { "$addToSet": { "invites": bson::to_bson(&e).unwrap() } },
                    Some(FindOneAndUpdateOptions::builder().upsert(true).build()),
                )
                .await?;
            }
            Event::InviteDelete(event) => {
                event!(Level::INFO, "Invite deleted");
                coll.find_one_and_update(
                    doc! { "doctype":"invite_storage", "guild_id": event.guild_id.get().to_string() },
                    doc! { "$pull": { "invites": { "code": event.code } } },
                    None,
                )
                .await?;
            }
            Event::MemberAdd(e) => {
                let MemberAdd(member) = *e;
                if member.user.bot {
                    return Ok(());
                }
                let invites = http
                    .guild_invites(member.guild_id)
                    .exec()
                    .await?
                    .model()
                    .await?
                    .into_iter()
                    .map(Into::into)
                    .collect::<Vec<MongoInvite>>();

                let coll = ctx.db.collection::<GuildInviteStorage>("invites");

                let storage = coll
                        .find_one(doc! {"guild_id":member.guild_id.0.to_string(), "doctype":"invite_storage" }, None)
                        .await?.unwrap_or_else(|| GuildInviteStorage {
                            doctype: "invite_storage".to_string(),
                            guild_id: member.guild_id.0.to_string(),
                            invites: Vec::<MongoInvite>::new(),
                        });

                let cache: Vec<MongoInvite> = storage.invites.into_iter().map(Into::into).collect();

                if !invites.is_empty() {
                    event!(
                        Level::INFO,
                        "Updating invites for guild {}",
                        member.guild_id.0
                    );
                    coll.find_one_and_update(
                            doc! { "doctype":"invite_storage", "guild_id":member.guild_id.0.to_string() },
                            doc! { "$set": { "invites": bson::to_bson(&invites).unwrap() } },
                            Some(FindOneAndUpdateOptions::builder().upsert(true).build()),
                        ).await?;
                }

                if !cache.is_empty() && !invites.is_empty() {
                    event!(
                        Level::INFO,
                        "Updating cache for guild {}",
                        member.guild_id.0
                    );
                    let mut cache = cache.into_iter();
                    let possible: Vec<MongoInvite> = invites
                        .into_iter()
                        .filter(|c| {
                            c.uses.unwrap_or_default()
                                == cache
                                    .find(|i| i.code == c.code)
                                    .map(|o| {
                                        o.uses.expect("Could not pull uses from invite struct")
                                    })
                                    .unwrap_or_default()
                                    + 1 // tysm twilight
                        })
                        .collect::<_>();

                    if possible.is_empty() || possible.len() != 1 {
                        eprintln!("Cannot parse invite used {:#?}", possible);
                        return Ok(());
                    }

                    let invite = possible[0].clone();
                    let invite = ctx
                        .http
                        .invite(&invite.code.to_string())
                        .exec()
                        .await?
                        .model()
                        .await?;
                    event!(Level::INFO, "Found invite: {:#?}", invite);
                    if let Some(user) = &invite.inviter {
                        let user_coll = ctx.db.collection::<UserInviteStorage>("invites");
                        let storage = user_coll.find_one_and_update(
                                doc! { "doctype":"user_storage", "user_id":user.id.0.to_string(), "guild_id":member.guild_id.0.to_string() },
                                doc! {
                                        "$set": { "user_id": user.id.get().to_string(), "guild_id": member.guild_id.0.to_string()},
                                        "$setOnInsert":{
                                            "regular":0,
                                            "fake":0,
                                            "bonus":0,
                                            "regular_data":bson::to_bson(&Vec::<MongoInvite>::new()).unwrap(),
                                            "leaves_data":bson::to_bson(&Vec::<String>::new()).unwrap(),
                                        }
                                    },
                                Some(FindOneAndUpdateOptions::builder().upsert(true).return_document(ReturnDocument::After).build()),
                            ).await?.unwrap();
                        let regular_ids = storage
                            .regular_data
                            .iter()
                            .map(|i| i.clone().inviter.unwrap().id.get())
                            .collect::<Vec<_>>();

                        if storage
                            .leaves_data
                            .iter()
                            .any(|i| i == &member.user.id.get().to_string())
                        {
                            user_coll.find_one_and_update(
                                    doc! { "user_id":user.id.0.to_string(), "guild_id":member.guild_id.0.to_string() },
                                    doc! { "$pull": { "leaves_data": member.user.id.get().to_string() }, "$addToSet": { "regular_data": bson::to_bson(&invite).unwrap() } },
                                    None,
                                ).await?;
                        } else if regular_ids.iter().any(|i| i == &member.user.id.get()) {
                            user_coll.find_one_and_update(
                                        doc! { "user_id":user.id.0.to_string(), "guild_id":member.guild_id.0.to_string() },
                                        doc! { "$inc": { "regular": 1, "fake":-1 } },
                                        None,
                                    ).await?;
                        } else if user.id.0 == member.user.id.0 {
                            user_coll.find_one_and_update(
                                        doc! { "user_id":user.id.0.to_string(), "guild_id":member.guild_id.0.to_string() },
                                        doc! { "$inc": { "regular": 1, "fake":1 }, "$push": { "regular_data": bson::to_bson(&invite).unwrap() } },
                                        None,
                                    ).await?;
                        } else {
                            user_coll.find_one_and_update(
                                        doc! { "user_id":user.id.0.to_string(), "guild_id":member.guild_id.0.to_string() },
                                        doc! { "$inc": { "regular": 1 }, "$push": { "regular_data": bson::to_bson(&invite).unwrap() } },
                                        None,
                                    ).await?;
                        }

                        let join_coll = ctx.db.collection::<JoinStorage>("traffic");

                        join_coll.find_one_and_update(
                                doc! {"doctype": "join_storage", "guild_id": member.guild_id.get().to_string(), "user_id": member.user.id.get().to_string()},
                                doc! { "$set": { "inviter_id": Some(user.id.get().to_string()) , "timestamp": bson::DateTime::now() } },
                                FindOneAndUpdateOptions::builder().upsert(true).build(),
                            ).await?;
                    } else {
                        panic!("Inviter not present in invite object")
                    }
                }
            }
            Event::MemberRemove(event) => {
                event!(Level::INFO, "Member removed: {:#?}", event);
                let coll = ctx.db.collection::<JoinStorage>("traffic");

                let doc = coll.find_one(doc!{ "user_id":event.user.id.get().to_string(), "guild_id":event.guild_id.get().to_string(), "doctype":"join_storage" }, None).await?;

                if doc.is_some() {
                    let doc = doc.unwrap();
                    event!(Level::INFO, "Found user in join storage: {:#?}", doc);
                    if let Some(inviter_id) = doc.inviter_id {
                        let invite_coll = ctx.db.collection::<UserInviteStorage>("invites");
                        invite_coll.find_one_and_update(
                        doc! { "user_id":inviter_id, "guild_id":doc.guild_id, "doctype":"user_storage" },
                        doc! {"$addToSet": { "leaves_data": event.user.id.get().to_string() } },
                        None,
                    ).await?;
                    }
                }
            }
            Event::GuildCreate(_e) => {}
            Event::GuildDelete(_e) => {}

            _event => {}
        };
        Ok(())
    }
    async fn sync_db(&self, _ctx: &Context) -> Result<()> {
        Ok(())
    }
}

impl Default for InviteCounting {
    fn default() -> Self {
        Self {}
    }
}
