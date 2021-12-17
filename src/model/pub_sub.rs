use uuid:Uuid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MessageData {

}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub id: String,
    pub channel: String,
    pub payload: MessageData,
}

impl Message {
    pub fn new (payload: MessageData) -> Self {
        Message {
            id: uuid::Uuid::new_v4().to_string(),
            channel: "cmd-handler".to_string(),
            payload,
        }
    }
}

