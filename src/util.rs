use crate::config::Config;
use crate::store::Store;
use poise::serenity_prelude::{
  ChannelId, ChannelType, Context, GuildChannel, GuildId, Member, ReactionType,
  UserId,
};
use rand::RngExt;

pub async fn get_guild_emoji(
  ctx: &Context,
  guild: GuildId,
  name: &str,
) -> Option<String> {
  let guild = ctx.cache.guild(guild)?;
  let emoji = guild.emojis.values().find(|e| e.name == name)?;

  Some(emoji.to_string())
}

/// If the context given belongs to a prefix command, has a 1/3 chance to
/// tomato the command message.
pub async fn tomato_bad_command(
  ctx: crate::cmd::util::CmdContext<'_>,
) -> anyhow::Result<()> {
  if let poise::Context::Prefix(pctx) = ctx {
    if rand::rng().random_bool(0.3334) {
      pctx.msg.react(ctx, ReactionType::Unicode("🍅".to_owned())).await?;
    }
  }

  Ok(())
}

/// Returns true if the member given has any mod role.
pub fn is_mod(cfg: &Config, usr: &Member) -> bool {
  usr.roles.contains(&cfg.role_admin_id) || usr.roles.contains(&cfg.role_mod_id)
}

/// Returns true if the channel given is a post or thread.
pub fn is_thread(chan: &GuildChannel) -> bool {
  matches!(
    chan.kind,
    ChannelType::PublicThread
      | ChannelType::PrivateThread
      | ChannelType::NewsThread
  )
}

/// Returns true if the user given is a mod in the thread (this counts the
/// thread's OP as a mod).
pub async fn is_thread_mod(
  chan: &GuildChannel,
  store: &Store,
  usr: UserId,
) -> bool {
  if chan.owner_id == Some(usr) {
    return true;
  }

  store.is_thread_mod(chan.guild_id, chan.id, usr).await.unwrap_or(false)
}
