use crate::cmd::util::{
  CmdContext, Error, check_guild_only, interaction_locale,
};
use crate::store::Store;
use crate::util::{is_mod, is_thread, is_thread_mod, tomato_bad_command};
use poise::serenity_prelude::{GuildChannel, Member, UserId};

#[poise::command(
  slash_command,
  prefix_command,
  rename = "thread",
  aliases("hilo"),
  subcommands("block", "unblock")
)]
pub async fn thread(_ctx: CmdContext<'_>) -> Result<(), Error> {
  Ok(())
}

#[poise::command(
  slash_command,
  prefix_command,
  rename = "block",
  aliases("bloquear"),
  check = "check_guild_only"
)]
async fn block(ctx: CmdContext<'_>, person: Member) -> Result<(), Error> {
  let locale = interaction_locale(&ctx);
  let cfg = &ctx.data().config;
  let i18n = &ctx.data().i18n;

  let author_id = ctx.author().id;

  let guild = ctx.guild_id().unwrap();
  let chan = ctx.channel_id().to_channel(&ctx).await?.guild().unwrap();

  if is_thread(&chan) == false {
    ctx
      .reply(i18n.t(locale, "commands.thread.block.replies.not-a-thread"))
      .await?;
    tomato_bad_command(ctx).await?;

    return Ok(());
  }

  if is_mod(&cfg, &person) {
    ctx
      .reply(i18n.t(locale, "commands.thread.block.replies.target-is-mod"))
      .await?;
    tomato_bad_command(ctx).await?;

    return Ok(());
  }

  if person.user.id == cfg.user_mike_id {
    ctx
      .reply(i18n.f(
        locale,
        "commands.thread.block.replies.target-is-mike",
        &[("author_id", &author_id)],
      ))
      .await?;
    tomato_bad_command(ctx).await?;

    return Ok(());
  }

  if person.user.id == cfg.user_yoni_id {
    ctx
      .reply(i18n.t(locale, "commands.thread.block.replies.target-is-yoni"))
      .await?;
    tomato_bad_command(ctx).await?;

    return Ok(());
  }

  if is_thread_mod(&chan, &ctx.data().store, author_id).await == false {
    ctx
      .reply(i18n.t(locale, "commands.thread.block.replies.not-their-thread"))
      .await?;
    tomato_bad_command(ctx).await?;

    return Ok(());
  }

  let target_is_owner = chan.owner_id == Some(person.user.id);
  let target_id = person.user.id;

  if target_is_owner == false
    && ctx.data().store.is_thread_mod(guild, chan.id, target_id).await?
  {
    ctx
      .reply(i18n.t(locale, "commands.thread.block.replies.civil-war"))
      .await?;

    return Ok(());
  }

  if ctx.data().store.is_blocked_in_thread(guild, chan.id, target_id).await? {
    ctx
      .reply(i18n.f(
        locale,
        "commands.thread.block.replies.already-blocked",
        &[("channel_bugs_id", &cfg.channel_bugs_id)],
      ))
      .await?;

    return Ok(());
  }

  ctx.data().store.thread_block(guild, chan.id, target_id).await?;

  ctx
    .reply(i18n.f(
      locale,
      "commands.thread.block.replies.default",
      &[("author_id", &author_id), ("target_id", &target_id)],
    ))
    .await?;

  Ok(())
}

#[poise::command(
  slash_command,
  prefix_command,
  rename = "unblock",
  aliases("desbloquear"),
  check = "check_guild_only"
)]
async fn unblock(ctx: CmdContext<'_>, person: Member) -> Result<(), Error> {
  let locale = interaction_locale(&ctx);
  let i18n = &ctx.data().i18n;

  let author_id = ctx.author().id;

  let guild = ctx.guild_id().unwrap();
  let chan = ctx.channel_id().to_channel(&ctx).await?.guild().unwrap();

  if is_thread(&chan) == false {
    ctx
      .reply(i18n.t(locale, "commands.thread.unblock.replies.not-a-thread"))
      .await?;
    tomato_bad_command(ctx).await?;

    return Ok(());
  }

  if is_thread_mod(&chan, &ctx.data().store, author_id).await == false {
    ctx
      .reply(i18n.t(locale, "commands.thread.unblock.replies.not-their-thread"))
      .await?;
    tomato_bad_command(ctx).await?;

    return Ok(());
  }

  let target_id = person.user.id;

  ctx.data().store.thread_unblock(guild, chan.id, target_id).await?;

  ctx
    .reply(i18n.f(
      locale,
      "commands.thread.unblock.replies.default",
      &[("author_id", &author_id), ("target_id", &target_id)],
    ))
    .await?;

  Ok(())
}
