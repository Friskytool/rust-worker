pub use deadpool_redis::redis::cmd;
pub use serde::{Deserialize, Serialize};
use std::result::Result as StdResult;
pub use tracing::{event, Level};
pub use twilight_gateway::Event;
pub use twilight_model::{
    application::component::{button::ButtonStyle, ActionRow, Button, Component},
    channel::message::AllowedMentions,
    id::{marker::*, Id},
};
#[allow(dead_code)]
pub mod colors {
    pub const MAIN: u32 = 0x5da9ff;
    pub const BLUE: u32 = 0x6969ff;
    pub const RED: u32 = 0xff4040;
    pub const GREEN: u32 = 0x00ff7f;
}

pub use crate::core::error::Error;
pub type Result<T> = StdResult<T, Error>;
pub use crate::core::Plugin;
pub use crate::Context;
#[cfg(feature = "chrono")]
pub use chrono::prelude::*;
#[cfg(feature = "mongo")]
pub use mongodb::{bson, bson::doc};
pub use std::sync::Arc;
pub use tokio::sync::RwLock;
