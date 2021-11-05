#[macro_use]
extern crate async_trait;

extern crate deadpool_redis;
extern crate dotenv;
extern crate tracing;
use crate::core::prelude::*;
use std::sync::Arc;
use twilight_model::gateway::Intents;

mod context;
mod core;
mod db;
mod model;
mod plugins;
mod worker;

pub use context::Context;

#[tokio::main]
async fn main() -> Result<()> {
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

    let plugins: Vec<Box<dyn core::Plugin>> = vec![Box::new(plugins::MessageCounting())];
    let plugins: Arc<Vec<_>> = Arc::new(plugins.into_iter().map(|m| Arc::new(m)).collect());

    let mut worker = worker::Worker::new(config, plugins, intents).await;

    worker.start().await;
    Ok(())
}
