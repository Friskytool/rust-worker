use redis::{Commands};

pub struct PubSubHandler {
    pub client: Client,
    pub context: Context,
}

impl PubSubHandler {
    pub fn new(ctx: Context, client: Client) -> Self {
        Self { client, context: ctx }
    }

    pub async fn start(&mut self) {
        let mut con = client.
    }
}