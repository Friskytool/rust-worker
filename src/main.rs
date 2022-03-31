#[macro_use]
extern crate async_trait;
#[cfg(feature = "dates")]
extern crate date_time_parser;
extern crate deadpool_redis;
extern crate dotenv;
#[cfg(feature = "math-solving")]
extern crate meval;
extern crate tracing;
use crate::core::prelude::*;
use std::sync::Arc;

mod context;
mod core;
mod db;
mod model;
mod plugins;
mod worker;

pub use context::Context;

#[tokio::main]
async fn main() -> Result<()> {
    // let subscriber = tracing_subscriber::fmt().with_target(false).finish();
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .compact()
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Unable to set global default subscriber");

    let config = model::WorkerConfig::from_env()?;

    let _guard = sentry::init((
        config.sentry_dsn_url.clone(),
        sentry::ClientOptions {
            release: sentry::release_name!(),
            ..Default::default()
        },
    ));
    // This is the default scheme. It will automatically create as many
    // shards as is suggested by Discord.
    //let scheme = ShardScheme::Auto;

    // Use intents to only receive guild message events.

    let plugins: Vec<Box<dyn core::Plugin>> = vec![
        #[cfg(feature = "message-counting")]
        Box::new(plugins::message_counting::MessageCounting::default()),
        #[cfg(feature = "invite-counting")]
        Box::new(plugins::invite_counting::InviteCounting::default()),
        #[cfg(feature = "date-transformer")]
        Box::new(plugins::date_transform::DateTransformer::default()),
        #[cfg(feature = "dank-memer")]
        Box::new(plugins::dank_memer::DankMemer::default()),
        #[cfg(feature = "timers")]
        Box::new(plugins::timers::Timers::default()),
        #[cfg(feature = "giveaways")]
        Box::new(plugins::giveaways::Giveaways::default()),
        #[cfg(feature = "server-indexer")]
        Box::new(plugins::server_indexer::ServerIndexer()),
        #[cfg(feature = "math-solving")]
        Box::new(plugins::math_solving::MathSolving::default()),
    ];
    let plugins: Arc<Vec<_>> = Arc::new(plugins.into_iter().map(|m| Arc::new(m)).collect());

    let mut worker = worker::Worker::new(config, plugins).await;

    worker.start().await;
    Ok(())
}
