use crate::model::PluginConfig;
use deadpool_redis::Pool as RedisPool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use twilight_cache_inmemory::InMemoryCache;
use twilight_gateway::Cluster;
use twilight_http::Client as HttpClient;
use twilight_model::{
    id::UserId,
    user::{CurrentUser, User},
};

#[derive(Clone)]
pub struct Context {
    pub cache: Arc<InMemoryCache>,
    pub cluster: Arc<Cluster>,
    pub mongo_client: Arc<mongodb::Client>,
    pub db: mongodb::Database,
    pub redis_pool: Arc<RedisPool>,
    pub http: Arc<HttpClient>,
    pub user: CurrentUser,
    pub owners: HashMap<UserId, Arc<User>>,
    pub plugin_config: Arc<RwLock<PluginConfig>>,
}
