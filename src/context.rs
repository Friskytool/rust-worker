use twilight_cache_inmemory::InMemoryCache;
use twilight_gateway::Cluster;
use twilight_http::Client as HttpClient;
use twilight_model::{
    id::UserId,
    user::{CurrentUser, User}
};
use std::sync::Arc;
use std::collections::HashMap;

#[derive(Clone)]
pub struct Context {
    pub cache: Arc<InMemoryCache>,
    pub cluster: Arc<Cluster>,
    //pub db: DatabaseConnection,
    pub http: Arc<HttpClient>,
    pub user: Arc<CurrentUser>,
    pub owners: Arc<HashMap<UserId, Arc<User>>>,
}