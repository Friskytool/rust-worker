use crate::core::prelude::*;
use crate::core::Plugin;
use dashmap::DashMap;
use meval::eval_str;
use regex::Regex;
use twilight_http::request::channel::reaction::RequestReactionType;
use twilight_model::id::MessageId;

#[derive(Debug, Clone)]
pub struct MathSolving {
    pub equation_expr: Regex,
    pub cache: DashMap<MessageId, f64>,
}

#[async_trait]
impl Plugin for MathSolving {
    fn description(&self) -> &'static str {
        "Computes math equations sent in chat without prefixes"
    }

    fn name(&self) -> &'static str {
        "math_solving"
    }

    async fn on_event(&self, event: Event, ctx: Context) -> Result<()> {
        match event {
            Event::MessageCreate(msg) => {
                if msg.author.bot || msg.content.is_empty() {
                    return Ok(());
                }

                if !self.equation_expr.is_match(&msg.content) {
                    return Ok(());
                }

                if let Ok(result) = eval_str(&msg.content) {
                    let emoji = RequestReactionType::Unicode { name: "🌃" };
                    if let Ok(_) = ctx
                        .http
                        .create_reaction(msg.channel_id, msg.id, &emoji)
                        .exec()
                        .await
                    {
                        self.cache.insert(msg.id, result);
                    }
                }
            }

            Event::ReactionAdd(reaction) => {
                if let Some((_, val)) = self.cache.remove(&reaction.message_id) {
                    ctx.http
                        .create_message(reaction.channel_id)
                        .content(&format!("{val}"))?
                        .exec()
                        .await
                        .ok();
                }
            }
            _ => {}
        }

        Ok(())
    }

    async fn sync_db(&self, _ctx: &Context) -> Result<()> {
        Ok(())
    }
}

impl Default for MathSolving {
    fn default() -> Self {
        Self {
            equation_expr: Regex::new(r"([\dekmbh\(\)]*)").unwrap(),
            cache: DashMap::new(),
        }
    }
}
