use crate::model::PluginConfig;
use deadpool_redis::Pool as RedisPool;
use lapin::Connection;
use std::collections::HashMap;
use std::sync::Arc;
#[cfg(feature = "tagscript")]
use tagscript::Interpreter;
use tokio::sync::RwLock;
use twilight_cache_inmemory::InMemoryCache;
use twilight_http::Client as HttpClient;
use twilight_model::{
    id::{marker::UserMarker, Id},
    user::{CurrentUser, User},
};

#[derive(Clone)]
pub struct Context {
    pub cache: Arc<InMemoryCache>,
    pub rabbit_conn: Arc<Connection>,
    #[cfg(feature = "mongo")]
    pub mongo_client: Arc<mongodb::Client>,
    #[cfg(feature = "mongo")]
    pub db: mongodb::Database,
    pub redis_pool: Arc<RedisPool>,
    pub http: Arc<HttpClient>,
    pub user: CurrentUser,
    pub owners: HashMap<Id<UserMarker>, Arc<User>>,
    pub plugin_config: Arc<RwLock<PluginConfig>>,
    #[cfg(feature = "tagscript")]
    pub interpreter: Arc<Interpreter>,
}
