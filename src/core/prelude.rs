pub use twilight_model::id::{ChannelId, GuildId, RoleId, UserId};
pub use serde::{Deserialize, Serialize};

pub const WEEK: usize = 60*60*24*7;
pub const DAY:  usize = 60*60*24;
pub const HOUR: usize = 60*60;
pub const MIN:  usize = 60;

pub const MESSAGE_CACHE: usize = 100;
pub const SLICE_SIZE: usize = 65535;
pub const USER_SLICE_SIZE: usize = 65535/5;


pub mod colors {
    pub const MAIN: u32 = 0x5da9ff;
    pub const BLUE: u32 = 0x6969ff;
    pub const RED: u32 = 0xff4040;
    pub const GREEN: u32 = 0x00ff7f;
}

