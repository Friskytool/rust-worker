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
    //     let (cluster, mut events) = Cluster::builder(config.discord_token.to_owned(), intents)
    //         .shard_scheme(scheme)
    //         .build()
    //         .await?;
    //     let cluster = Arc::new(cluster);

    //     // Start up the cluster.
    //     let cluster_spawn = Arc::clone(&cluster);

    //     // Start all shards in the cluster in the background.
    //     tokio::spawn(async move {
    //         cluster_spawn.up().await;
    //     });

    //     // HTTP is separate from the gateway, so create a new client.
    //     let http = Arc::new(HttpClient::new(config.discord_token));

    //     let cache = InMemoryCache::builder().message_cache_size(200).build();

    //     // Process each event as they come in.
    //     while let Some((shard_id, event)) = events.next().await {
    //         // Update the cache with the event.
    //         cache.update(&event);

    //         tokio::spawn(handle_event(shard_id, event, Arc::clone(&http)));
    //     }

    //     Ok(())
    // }

    // async fn handle_event(
    //     shard_id: u64,
    //     event: Event,
    //     http: Arc<HttpClient>,
    // ) -> Result<(), Box<dyn Error + Send + Sync>> {
    //     match event {
    //         Event::MessageCreate(msg) if msg.content == "s.ping" => {
    //             http.create_message(msg.channel_id)
    //                 .content("Pong!")?
    //                 .exec()
    //                 .await?;
    //         }
    //         Event::ShardConnected(_) => {
    //             println!("Connected on shard {}", shard_id);
    //         }
    //         // Other events here...
    //         _ => {}
    //     }

    Ok(())
}
