use twilight_embed_builder::EmbedError;
use std::error::Error as StdError;
use mongodb::error::Error as MongoError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Environment variable '{0}' not found.")]
    EnvironmentVariableNotFound(String),

    #[error("Header '{0}' not found.")]
    HeaderNotFound(String),

    #[error("Failed to do signal stuff")]
    SignalError(#[from] std::io::Error),

    #[error("Failed to deserialize from or serialize to JSON.")]
    JsonFailed(#[from] serde_json::Error),

    #[error("Invalid payload provided: {0}.")]
    InvalidPayload(String),

    #[error("Embed failed to build.")]
    EmbedFailed(EmbedError),

    #[error("Failed to load config")]
    ConfigError(#[from] config::ConfigError),

    #[error("Twilight raised an error")]
    TwilightError(#[from] Box<dyn StdError + Send + Sync>),

    #[error("MongoDB raised an error")]
    MongoError(#[from] MongoError),

    #[error("TwilightHttp raised an error while generating an api request.")]
    TwilightHttpError(#[from] twilight_http::Error),

    #[error("TwilightHttp raised an error while creating a message.")]
    TwilightMessageCreateFailed(#[from] twilight_http::request::prelude::create_message::CreateMessageError ),
}
