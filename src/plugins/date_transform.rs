use crate::core::prelude::*;
use chrono::NaiveDateTime;
use chrono::{DateTime, Utc};
use date_time_parser::{DateParser, TimeParser};
use regex::Regex;

#[derive(Clone, Debug)]
pub struct DateTransformer {
    pub regex_expr: Regex,
}

#[async_trait]
impl Plugin for DateTransformer {
    fn name(&self) -> &'static str {
        "date_transform"
    }

    fn description(&self) -> &'static str {
        "Transforms dates to a more readable format"
    }

    async fn sync_db(&self, _ctx: &Context) -> Result<()> {
        Ok(())
    }

    async fn on_event(&self, event: Event, ctx: Context) -> Result<()> {
        match event {
            Event::MessageCreate(message) => {
                // Get the time from message content
                if message.author.bot
                    || message.content.is_empty()
                    || !self.regex_expr.is_match(&message.content)
                {
                    event!(Level::DEBUG, "Message does not contain a date");
                    return Ok(());
                }
                let mut content = message.content.clone();
                let src = NaiveDateTime::from_timestamp(message.timestamp.as_secs(), 0);
                for caps in self.regex_expr.captures_iter(&message.content) {
                    let mut relative = false;

                    let time: Option<NaiveDateTime> =
                        DateParser::parse(caps.get(1).unwrap().as_str()).map_or_else(
                            || {
                                relative = true;
                                TimeParser::parse(caps.get(1).unwrap().as_str()).map(|time| {
                                    NaiveDateTime::new(
                                        src.date(),
                                        time, /*+ Duration::hours(src.time().hour().into()),*/
                                    )
                                })
                            },
                            |date| Some(NaiveDateTime::new(date, src.time())),
                        );
                    if time.is_some() {
                        let time = time.unwrap();
                        let time = DateTime::<Utc>::from_utc(time, Utc);
                        content = content.replacen(
                            caps.get(0).unwrap().as_str(),
                            &format!(
                                "<t:{}{}>",
                                time.timestamp(),
                                if relative { ":R" } else { "" }
                            ),
                            1,
                        );
                    } else {
                    }
                }
                if content != message.content {
                    ctx.http
                        .create_message(message.channel_id)
                        .content(&content)?
                        .exec()
                        .await?;
                }
            }
            _ => {}
        };
        Ok(())
    }
}

impl Default for DateTransformer {
    fn default() -> Self {
        Self {
            regex_expr: Regex::new(r"\{([^}]*)\}").unwrap(),
        }
    }
}
