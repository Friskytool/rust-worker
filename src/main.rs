#[macro_use]
extern crate async_trait;

extern crate deadpool_redis;
extern crate dotenv;
extern crate tracing;

use futures::stream::StreamExt;
use std::{env, error::Error, sync::Arc};
use twilight_cache_inmemory::{InMemoryCache, ResourceType};
use twilight_gateway::{
    cluster::{Cluster, ShardScheme},
    Event,
};
use twilight_http::Client as HttpClient;
use twilight_model::gateway::Intents;

mod context;
mod core;
mod db;
mod model;
mod worker;

pub use context::Context;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let subscriber = tracing_subscriber::fmt().with_target(false).finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("Unable to set global default subscriber");

    let config = model::WorkerConfig::from_env()?;

    // This is the default scheme. It will automatically create as many
    // shards as is suggested by Discord.
    //let scheme = ShardScheme::Auto;

    let intents = Intents::all()
        ^ Intents::GUILD_PRESENCES
        ^ Intents::GUILD_MESSAGE_TYPING
        ^ Intents::DIRECT_MESSAGE_TYPING;

    // Use intents to only receive guild message events.

    let mut worker = worker::Worker::new(config, intents).await;

    worker.start().await;
    Ok(())
}
