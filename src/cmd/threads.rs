use crate::cmd::util::{Context, Error, check_guild_only, interaction_locale};
use crate::util::{is_mod, is_thread, tomato_bad_command};
use poise::serenity_prelude::Member;

#[poise::command(
  slash_command,
  prefix_command,
  rename = "threads",
  aliases("hilos"),
  subcommands("block")
)]
pub async fn threads(_ctx: Context<'_>) -> Result<(), Error> {
  Ok(())
}

#[poise::command(
  slash_command,
  prefix_command,
  rename = "block",
  aliases("bloquear"),
  check = "check_guild_only"
)]
async fn block(ctx: Context<'_>, person: Member) -> Result<(), Error> {
  let locale = interaction_locale(&ctx);
  let cfg = &ctx.data().config;
  let i18n = &ctx.data().i18n;

  let author_id = ctx.author().id;

  if is_mod(&cfg, &person) {
    ctx
      .reply(i18n.t(locale, "commands_threads.block.replies.target-is-mod"))
      .await?;
    tomato_bad_command(ctx).await?;

    return Ok(());
  }

  if person.user.id == cfg.user_mike_id {
    ctx
      .reply(i18n.f(
        locale,
        "commands_threads.block.replies.target-is-mike",
        &[("author_id", &author_id)],
      ))
      .await?;
    tomato_bad_command(ctx).await?;

    return Ok(());
  }

  if person.user.id == cfg.user_yoni_id {
    ctx
      .reply(i18n.t(locale, "commands_threads.block.replies.target-is-yoni"))
      .await?;
    tomato_bad_command(ctx).await?;

    return Ok(());
  }

  let guild = ctx.guild_id().unwrap();
  let chan = ctx.channel_id().to_channel(&ctx).await?;

  let guild_chan = match chan.guild() {
    Some(gc) => gc,
    None => {
      ctx.reply("This command can only be used in a server channel").await?; // TODO: Change.
      return Ok(());
    },
  };

  if is_thread(&guild_chan) {
    ctx
      .reply(i18n.t(locale, "commands.threads.block.replies.not-a-thread"))
      .await?;
    tomato_bad_command(ctx).await?;

    return Ok(());
  }

  let author_is_owner = guild_chan.owner_id == Some(author_id);

  if author_is_owner == false
    && ctx.data().store.is_thread_mod(guild, guild_chan.id, author_id).await?
      == false
  {
    ctx
      .reply(i18n.t(locale, "commands.threads.block.replies.not-their-thread"))
      .await?;
    tomato_bad_command(ctx).await?;

    return Ok(());
  }

  let target_is_owner = guild_chan.owner_id == Some(person.user.id);
  let target_id = person.user.id;

  if target_is_owner == false
    && ctx.data().store.is_thread_mod(guild, guild_chan.id, target_id).await?
  {
    ctx
      .reply(i18n.t(locale, "commands.threads.block.replies.civil-war"))
      .await?;

    return Ok(());
  }

  ctx.data().store.thread_block(guild, guild_chan.id, target_id).await?;

  ctx
    .reply(i18n.f(
      locale,
      "commands.thread.block.replies.default",
      &[("author_id", &author_id), ("target_id", &target_id)],
    ))
    .await?;

  Ok(())
}
