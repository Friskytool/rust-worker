use crate::context::Context;
use crate::core::EventHandler;
use crate::model::WorkerConfig;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{event, Level};
use twilight_cache_inmemory::{InMemoryCache, ResourceType};
use twilight_gateway::{
    cluster::{Cluster, ShardScheme},
    Event,
};
use twilight_http::Client as HttpClient;
use twilight_model::gateway::Intents;
#[non_exhaustive]
pub struct Worker {
    pub handler: EventHandler,
    pub config: WorkerConfig,
    pub ctx: Context,
}

impl Worker {
    pub async fn new(config: WorkerConfig, intents: Intents) -> Self {
        let http = Arc::new(HttpClient::new(config.discord_token.clone()));
        let cache = Arc::new(InMemoryCache::new());

        let app_info = http
            .current_user_application()
            .exec()
            .await
            .expect("Unable to retrieve application info.")
            .model()
            .await
            .expect("Unable to retrieve application info");
        let user = http
            .current_user()
            .exec()
            .await
            .expect("Unable to retrieve current user.")
            .model()
            .await
            .expect("Unable to retrieve current user");

        let owner = app_info.owner;
        let mut owners = HashMap::new();
        owners.insert(owner.id, Arc::new(owner));

        let (cluster, mut events) = Cluster::builder(&config.discord_token, intents)
            .shard_scheme(ShardScheme::Auto)
            .http_client(http.clone())
            .build()
            .await
            .unwrap_or_else(|err| panic!("Unabled to setup cluster: {}", err));
        let cluster = Arc::new(cluster);
        let ctx = Context {
            cache,
            cluster,
            http,
            user,
            owners,
        };

        let handler = EventHandler::new(ctx.clone(), events);

        Self {
            ctx,
            config,
            handler,
        }
    }

    pub async fn start(&mut self) {
        let cluster_spawn = self.ctx.cluster.clone();

        event!(Level::DEBUG, "Starting Cluster");
        let _cluster_handle = tokio::spawn(async move {
            cluster_spawn.up().await;
        });

        event!(Level::DEBUG, "Starting Event Handler");
        self.start_handler().await;
    }

    async fn start_handler(&mut self) {
        self.handler.start().await
    }
}
