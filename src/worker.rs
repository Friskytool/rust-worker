use crate::context::Context;
use crate::core::EventHandler;
use crate::model::{PluginConfig, WorkerConfig};
use lapin::options::{BasicConsumeOptions, QueueDeclareOptions};
use lapin::{types::FieldTable, Connection, ConnectionProperties};
use std::collections::HashMap;
use std::sync::Arc;
use tagscript::{block, Interpreter};
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};
use tracing::{event, Level};
use twilight_cache_inmemory::InMemoryCache;
use twilight_http::Client as HttpClient;
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
    pub async fn new(config: WorkerConfig, plugins: Arc<Vec<Arc<Box<dyn Plugin>>>>) -> Self {
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

        let rabbit_conn = Connection::connect(&config.rabbit_uri, ConnectionProperties::default())
            .await
            .expect("Failed to connect to RabbitMQ");
        let rabbit_conn = Arc::new(rabbit_conn);

        let channel = rabbit_conn
            .create_channel()
            .await
            .expect("Could not create RabbitMQ channel");

        let channel2 = rabbit_conn
            .create_channel()
            .await
            .expect("Could not create RabbitMQ channel");

        channel2
            .queue_declare(
                &config.rabbit_queue,
                QueueDeclareOptions::default(),
                FieldTable::default(),
            )
            .await
            .expect("Could not initialize queue");

        let consumer = channel
            .basic_consume(
                &config.rabbit_queue,
                "gateway-worker",
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await
            .expect("Could not create RabbitMQ consumer");
        // let (cluster, events) = Cluster::builder(config.discord_token.clone(), intents)
        //     .shard_scheme(ShardScheme::Auto)
        //     .http_client(http.clone())
        //     .build()
        //     .await
        //     .unwrap_or_else(|err| panic!("Unabled to setup cluster: {}", err));

        let plugin_config = PluginConfig::new(plugins);
        let plugin_config = Arc::new(RwLock::new(plugin_config));

        let interpreter = Arc::new(Interpreter::new(vec![
            Box::new(block::AssignmentBlock {}),
            Box::new(block::BreakBlock {}),
            Box::new(block::AllBlock {}),
            Box::new(block::AnyBlock {}),
            Box::new(block::IfBlock {}),
            Box::new(block::FiftyFiftyBlock {}),
            Box::new(block::LooseVariableGetterBlock {}),
            Box::new(block::MathBlock {}),
            Box::new(block::RandomBlock {}),
            Box::new(block::RangeBlock {}),
            Box::new(block::ShortCutRedirectBlock {
                redirect_name: "args".into(),
            }),
            Box::new(block::StopBlock {}),
            Box::new(block::SubstringBlock {}),
        ]));
        let ctx = Context {
            cache,
            http,
            user,
            owners,
            interpreter,
            mongo_client,
            db: mongo_db,
            redis_pool,
            rabbit_conn: rabbit_conn.clone(),
            plugin_config: plugin_config.clone(),
        };

        let handler = EventHandler::new(ctx.clone(), consumer);

        Self {
            ctx,
            config,
            handler,
        }
    }

    pub async fn start(&mut self) {
        // let cluster_spawn = self.ctx.cluster.clone();
        // event!(Level::DEBUG, "Starting Cluster");
        // let _cluster_handle = tokio::spawn(async move {
        //     cluster_spawn.up().await;
        // });

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
            sleep(Duration::from_secs(15)).await;
            for plugin in ctx.plugin_config.read().await.plugins.iter() {
                if let Err(why) = plugin.sync_db(&ctx).await {
                    event!(Level::ERROR, "Failed to sync db: {:?}", why);
                };
            }
        }
    }
}
