use crate::context::Context;
use crate::core::EventHandler;
use crate::model::{PluginConfig, WorkerConfig};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};
use tracing::{event, Level};
use twilight_cache_inmemory::InMemoryCache;
use twilight_gateway::cluster::{Cluster, ShardScheme};
use twilight_http::Client as HttpClient;
use twilight_model::gateway::Intents;
// Databases
use crate::core::prelude::*;
use crate::db::{MongoClient, MongoClientOptions};
use deadpool_redis::Runtime;
use mongodb::options::Compressor;

#[non_exhaustive]
pub struct Worker {
    pub handler: EventHandler,
    pub config: WorkerConfig,
    pub ctx: Context,
}

impl Worker {
    pub async fn new(
        config: WorkerConfig,
        plugins: Arc<Vec<Arc<Box<dyn Plugin>>>>,
        intents: Intents,
    ) -> Self {
        let http = Arc::new(HttpClient::new(config.discord_token.clone()));
        let cache = Arc::new(InMemoryCache::new());

        // Setting up MongoDB Connection
        let mut mongo_options = MongoClientOptions::parse(&config.mongo_uri)
            .await
            .expect("Failed to parse mongo uri into connection options");

        mongo_options.compressors = Some(vec![Compressor::Zstd {
            level: Default::default(),
        }]);

        let mongo_client = Arc::new(
            MongoClient::with_options(mongo_options).expect("Failed to create MongoClient"),
        );
        let mongo_db = mongo_client.database(&config.mongo_db);
        // Setting up Redis connection
        let redis_pool = config
            .redis
            .create_pool(Some(Runtime::Tokio1))
            .expect("Failed to create Redis pool");
        let redis_pool = Arc::new(redis_pool);

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

        let (cluster, events) = Cluster::builder(&config.discord_token, intents)
            .shard_scheme(ShardScheme::Auto)
            .http_client(http.clone())
            .build()
            .await
            .unwrap_or_else(|err| panic!("Unabled to setup cluster: {}", err));
        let cluster = Arc::new(cluster);

        let plugin_config = PluginConfig::new(plugins);
        let plugin_config = Arc::new(RwLock::new(plugin_config));

        let ctx = Context {
            cache,
            cluster,
            http,
            user,
            owners,
            mongo_client: mongo_client,
            db: mongo_db,
            redis_pool: redis_pool,
            plugin_config: plugin_config.clone(),
        };
        {
            let mut p_config = plugin_config.write().await;
            p_config
                .load_cache(ctx.clone())
                .await
                .expect("Failed to load plugin config cache");
        }

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

        let ctx = self.ctx.clone();
        let _db_sync_handle = tokio::spawn(async move {
            Worker::db_sync_handler(ctx).await;
        });
        event!(Level::DEBUG, "Starting Event Handler");
        self.start_handler().await;
    }

    async fn start_handler(&mut self) {
        self.handler.start().await
    }

    async fn db_sync_handler(ctx: Context) {
        loop {
            sleep(Duration::from_secs(30)).await;
            for plugin in ctx.plugin_config.read().await.plugins.iter() {
                if let Err(why) = plugin.sync_db(&ctx).await {
                    event!(Level::ERROR, "Failed to sync db: {:?}", why);
                };
            }
        }
    }
}
