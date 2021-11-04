mod plugin;

pub mod error;
pub mod handler;
pub mod prelude;

pub use error::Error;
pub use handler::EventHandler;
pub use plugin::{Plugin};

use std::result::Result as StdResult;
pub type Result<T> = StdResult<T, Error>;
