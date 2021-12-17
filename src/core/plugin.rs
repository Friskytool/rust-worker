use crate::core::prelude::*;
use crate::Context;
use twilight_gateway::Event;
use twilight_gateway::Intents;

#[async_trait::async_trait]
pub trait Plugin: std::fmt::Debug + Send + Sync {
    #[inline]
    fn intents(&self) -> Intents {
        Intents::empty()
    }

    async fn on_event(&self, event: Event, context: Context) -> Result<()>;

    async fn sync_db(&self, context: &Context) -> Result<()>;

    fn name(&self) -> &'static str;

    fn description(&self) -> &'static str;
}
