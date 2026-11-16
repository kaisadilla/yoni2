use crate::cmd::util::{CmdContext, CmdContextExt};
use crate::util::{Error, GuildChannelExt, MemberExt};
use poise::serenity_prelude::Member;

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
  guild_only
)]
async fn block(ctx: CmdContext<'_>, person: Member) -> Result<(), Error> {
  let l = ctx.loc().await;
  let cfg = &ctx.data().config;

  let author_id = ctx.author().id;

  let guild = ctx.guild_id().unwrap();
  let chan = ctx.channel_id().to_channel(&ctx).await?.guild().unwrap();

  if chan.is_thread() == false {
    ctx.reply(l.t("commands.thread.block.replies.not-a-thread")).await?;
    ctx.tomato_bad_command().await.ok();

    return Ok(());
  }

  if person.is_mod(&ctx, &cfg).await {
    ctx.reply(l.t("commands.thread.block.replies.target-is-mod")).await?;
    ctx.tomato_bad_command().await.ok();

    return Ok(());
  }

  if person.user.id == cfg.user_mike_id {
    ctx
      .reply(l.fmt(
        "commands.thread.block.replies.target-is-mike",
        &[("author_id", &author_id)],
      ))
      .await?;
    ctx.tomato_bad_command().await.ok();

    return Ok(());
  }

  if person.user.id == cfg.user_yoni_id {
    ctx.reply(l.t("commands.thread.block.replies.target-is-yoni")).await?;
    ctx.tomato_bad_command().await.ok();

    return Ok(());
  }

  if chan.is_thread_mod(&ctx.data().store, author_id).await == false {
    ctx.reply(l.t("commands.thread.block.replies.not-their-thread")).await?;
    ctx.tomato_bad_command().await.ok();

    return Ok(());
  }

  let target_is_owner = chan.owner_id == Some(person.user.id);
  let target_id = person.user.id;

  if target_is_owner == false
    && ctx.data().store.is_thread_mod(guild, chan.id, target_id).await?
  {
    ctx.reply(l.t("commands.thread.block.replies.civil-war")).await?;

    return Ok(());
  }

  if ctx.data().store.is_blocked_in_thread(guild, chan.id, target_id).await? {
    ctx
      .reply(l.fmt(
        "commands.thread.block.replies.already-blocked",
        &[("channel_bugs_id", &cfg.channel_bugs_id)],
      ))
      .await?;

    return Ok(());
  }

  ctx.data().store.thread_block(guild, chan.id, target_id).await?;

  ctx
    .reply(l.fmt(
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
  guild_only
)]
async fn unblock(ctx: CmdContext<'_>, person: Member) -> Result<(), Error> {
  let l = ctx.loc().await;

  let author_id = ctx.author().id;
  let guild = ctx.guild_id().unwrap();
  let chan = ctx.channel_id().to_channel(&ctx).await?.guild().unwrap();

  if chan.is_thread() == false {
    ctx.reply(l.t("commands.thread.unblock.replies.not-a-thread")).await?;
    ctx.tomato_bad_command().await.ok();

    return Ok(());
  }

  if chan.is_thread_mod(&ctx.data().store, author_id).await == false {
    ctx.reply(l.t("commands.thread.unblock.replies.not-their-thread")).await?;
    ctx.tomato_bad_command().await.ok();

    return Ok(());
  }

  let target_id = person.user.id;

  ctx.data().store.thread_unblock(guild, chan.id, target_id).await?;

  ctx
    .reply(l.fmt(
      "commands.thread.unblock.replies.default",
      &[("author_id", &author_id), ("target_id", &target_id)],
    ))
    .await?;

  Ok(())
}
