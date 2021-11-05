pub use tracing::{event, Level};
use std::result::Result as StdResult;
pub use serde::{Deserialize, Serialize};
pub use twilight_model::id::{ChannelId, GuildId, RoleId, UserId};
pub use twilight_gateway::Event;
#[allow(dead_code)]
pub mod colors {
    pub const MAIN: u32 = 0x5da9ff;
    pub const BLUE: u32 = 0x6969ff;
    pub const RED: u32 = 0xff4040;
    pub const GREEN: u32 = 0x00ff7f;
}

pub use crate::core::error::Error;
pub type Result<T> = StdResult<T, Error>;
pub use crate::Context;
pub use crate::core::Plugin;
pub use std::sync::{Arc};
pub use tokio::sync::RwLock;