use std::collections::HashMap;
use std::sync::Arc;
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
    //pub db: DatabaseConnection,
    pub http: Arc<HttpClient>,
    pub user: CurrentUser,
    pub owners: HashMap<UserId, Arc<User>>,
}
