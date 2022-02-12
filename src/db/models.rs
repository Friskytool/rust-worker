use crate::core::prelude::*;
use bson::oid::ObjectId;
use bson::DateTime;
use serde::{Deserialize, Serialize};
use tokio::time::Duration as TokioDuration;
use twilight_model::datetime::Timestamp;
use twilight_model::id::*;
use twilight_model::invite::{Invite, InviteChannel, InviteGuild, InviteStageInstance, TargetType};
use twilight_model::user::User;

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct ItemTrade {
    pub date: DateTime,
    pub amount: i64,
    pub price: i64,
    pub item_id: String,
    pub value: f64,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct TransferStorage {
    pub sender_id: String,
    pub receiver_id: String,
    pub channel_id: String,
    pub guild_id: String,
    pub amount: u64,
    pub timestamp: Timestamp,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct Timer {
    pub _id: ObjectId,
    host_id: String,
    guild_id: String,
    message_id: String,
    channel_id: String,

    pub start: DateTime,
    pub end: DateTime,
    pub active: bool,
    // Data about the timer itself
    pub title: String,
    pub icon_url: String,
}

#[allow(dead_code)]
impl Timer {
    pub fn get_channel_id(&self) -> ChannelId {
        ChannelId(
            self.channel_id
                .parse::<std::num::NonZeroU64>()
                .expect("Nonzero number"),
        )
    }

    pub fn get_message_id(&self) -> MessageId {
        MessageId(
            self.message_id
                .parse::<std::num::NonZeroU64>()
                .expect("Nonzero number"),
        )
    }

    pub fn get_content(&self) -> String {
        format!(
            "**{}**\n\n<t:{}>",
            self.title,
            self.end.to_chrono().timestamp()
        )
    }

    pub fn get_duration_remaining(&self) -> TokioDuration {
        TokioDuration::from_secs(std::cmp::max(
            self.end.to_chrono().timestamp() - Utc::now().timestamp(),
            0,
        ) as u64)
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, Eq, PartialEq)]
pub struct GuildPluginConfig {
    pub id: String,
    pub plugins: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct UserInviteStorage {
    pub doctype: String,
    pub user_id: String,
    pub guild_id: String,

    pub regular: u32,
    pub fake: u32,
    pub bonus: u32,

    pub regular_data: Vec<MongoInvite>,
    pub leaves_data: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MongoInvite {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approximate_member_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approximate_presence_count: Option<u64>,
    pub channel: Option<InviteChannel>,
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<Timestamp>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<Timestamp>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guild: Option<InviteGuild>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inviter: Option<User>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_age: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_uses: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stage_instance: Option<InviteStageInstance>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_type: Option<TargetType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_user: Option<User>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temporary: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uses: Option<u64>,
}

impl From<Invite> for MongoInvite {
    fn from(invite: Invite) -> Self {
        Self {
            approximate_member_count: invite.approximate_member_count,
            approximate_presence_count: invite.approximate_presence_count,
            channel: invite.channel,
            code: invite.code,
            created_at: invite.created_at,
            expires_at: invite.expires_at,
            guild: invite.guild,
            inviter: invite.inviter,
            max_age: invite.max_age,
            max_uses: invite.max_uses,
            stage_instance: invite.stage_instance,
            target_type: invite.target_type,
            target_user: invite.target_user,
            temporary: invite.temporary,
            uses: invite.uses,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GuildInviteStorage {
    pub doctype: String,
    pub invites: Vec<MongoInvite>,
    pub guild_id: String,
}

impl From<Vec<Invite>> for GuildInviteStorage {
    fn from(invites: Vec<Invite>) -> Self {
        let guild_id = invites
            .clone()
            .into_iter()
            .nth(0)
            .expect("Expected invites for guild")
            .guild
            .expect("Expected guild attached to invite")
            .id
            .0
            .to_string();
        Self {
            doctype: "invite_storage".to_string(),
            invites: invites.into_iter().map(Into::into).collect(),
            guild_id,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MessageCountingUserStorage {
    pub guild_id: String,
    pub user_id: String,
    pub count: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct JoinStorage {
    pub doctype: String,
    pub guild_id: String,
    pub user_id: String,
    pub inviter_id: Option<String>,
    pub timestamp: DateTime,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LeaveStorage {
    pub doctype: String,
    pub guild_id: String,
    pub user_id: String,
    pub timestamp: DateTime,
}
