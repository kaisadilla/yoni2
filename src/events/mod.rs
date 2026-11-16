pub mod message;
pub mod reaction_add;
pub mod reaction_remove;

use crate::events::message::message;
use crate::events::reaction_add::reaction_add;
use crate::events::reaction_remove::reaction_remove;
use crate::util::{Data, Error};
use poise::FrameworkContext;
use poise::serenity_prelude::FullEvent;

pub async fn event_handler(
  framework: FrameworkContext<'_, Data, Error>,
  evt: &FullEvent,
) -> Result<(), Error> {
  let data = framework.user_data().await;
  let ctx = framework.serenity_context;

  match evt {
    FullEvent::CommandPermissionsUpdate { .. } => {},
    FullEvent::AutoModRuleCreate { .. } => {},
    FullEvent::AutoModRuleUpdate { .. } => {},
    FullEvent::AutoModRuleDelete { .. } => {},
    FullEvent::AutoModActionExecution { .. } => {},
    FullEvent::CacheReady { .. } => {},
    FullEvent::ShardsReady { .. } => {},
    FullEvent::ChannelCreate { .. } => {},
    FullEvent::CategoryCreate { .. } => {},
    FullEvent::CategoryDelete { .. } => {},
    FullEvent::ChannelDelete { .. } => {},
    FullEvent::ChannelPinsUpdate { .. } => {},
    FullEvent::ChannelUpdate { .. } => {},
    FullEvent::GuildAuditLogEntryCreate { .. } => {},
    FullEvent::GuildBanAddition { .. } => {},
    FullEvent::GuildBanRemoval { .. } => {},
    FullEvent::GuildCreate { .. } => {},
    FullEvent::GuildDelete { .. } => {},
    FullEvent::GuildEmojisUpdate { .. } => {},
    FullEvent::GuildIntegrationsUpdate { .. } => {},
    FullEvent::GuildMemberAddition { .. } => {},
    FullEvent::GuildMemberRemoval { .. } => {},
    FullEvent::GuildMemberUpdate { .. } => {},
    FullEvent::GuildMembersChunk { .. } => {},
    FullEvent::GuildRoleCreate { .. } => {},
    FullEvent::GuildRoleDelete { .. } => {},
    FullEvent::GuildRoleUpdate { .. } => {},
    FullEvent::GuildStickersUpdate { .. } => {},
    FullEvent::GuildUpdate { .. } => {},
    FullEvent::InviteCreate { .. } => {},
    FullEvent::InviteDelete { .. } => {},
    FullEvent::Message { new_message } => message(data, ctx, new_message).await,
    FullEvent::MessageDelete { .. } => {},
    FullEvent::MessageDeleteBulk { .. } => {},
    FullEvent::MessageUpdate { .. } => {},
    FullEvent::ReactionAdd { add_reaction } => {
      reaction_add(data, ctx, add_reaction).await
    },
    FullEvent::ReactionRemove { removed_reaction } => {
      reaction_remove(data, ctx, removed_reaction).await
    },
    FullEvent::ReactionRemoveAll { .. } => {},
    FullEvent::ReactionRemoveEmoji { .. } => {},
    FullEvent::PresenceReplace { .. } => {},
    FullEvent::PresenceUpdate { .. } => {},
    FullEvent::Ready { .. } => {},
    FullEvent::Resume { .. } => {},
    FullEvent::ShardStageUpdate { .. } => {},
    FullEvent::SoundboardSounds { .. } => {},
    FullEvent::SoundboardSoundCreate { .. } => {},
    FullEvent::SoundboardSoundUpdate { .. } => {},
    FullEvent::SoundboardSoundsUpdate { .. } => {},
    FullEvent::SoundboardSoundDelete { .. } => {},
    FullEvent::TypingStart { .. } => {},
    FullEvent::UserUpdate { .. } => {},
    FullEvent::VoiceServerUpdate { .. } => {},
    FullEvent::VoiceStateUpdate { .. } => {},
    FullEvent::VoiceChannelStatusUpdate { .. } => {},
    FullEvent::WebhookUpdate { .. } => {},
    FullEvent::InteractionCreate { .. } => {},
    FullEvent::IntegrationCreate { .. } => {},
    FullEvent::IntegrationUpdate { .. } => {},
    FullEvent::IntegrationDelete { .. } => {},
    FullEvent::StageInstanceCreate { .. } => {},
    FullEvent::StageInstanceUpdate { .. } => {},
    FullEvent::StageInstanceDelete { .. } => {},
    FullEvent::ThreadCreate { .. } => {},
    FullEvent::ThreadUpdate { .. } => {},
    FullEvent::ThreadDelete { .. } => {},
    FullEvent::ThreadListSync { .. } => {},
    FullEvent::ThreadMemberUpdate { .. } => {},
    FullEvent::ThreadMembersUpdate { .. } => {},
    FullEvent::GuildScheduledEventCreate { .. } => {},
    FullEvent::GuildScheduledEventUpdate { .. } => {},
    FullEvent::GuildScheduledEventDelete { .. } => {},
    FullEvent::GuildScheduledEventUserAdd { .. } => {},
    FullEvent::GuildScheduledEventUserRemove { .. } => {},
    FullEvent::EntitlementCreate { .. } => {},
    FullEvent::EntitlementUpdate { .. } => {},
    FullEvent::EntitlementDelete { .. } => {},
    FullEvent::MessagePollVoteAdd { .. } => {},
    FullEvent::MessagePollVoteRemove { .. } => {},
    FullEvent::Ratelimit { .. } => {},
    _ => {},
  }

  Ok(())
}
