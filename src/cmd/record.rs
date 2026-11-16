use crate::cmd::util::{CmdContext, CmdContextExt};
use crate::util::Error;
use poise::CreateReply;
use poise::serenity_prelude::{Colour, CreateEmbed};

#[poise::command(
  slash_command,
  prefix_command,
  rename = "record",
  aliases("récord"),
  subcommands("mierda")
)]
pub async fn record(_ctx: CmdContext<'_>) -> Result<(), Error> {
  Ok(())
}

#[poise::command(
  slash_command,
  prefix_command,
  rename = "mierda",
  aliases("mierda"),
  guild_only
)]
pub async fn mierda(ctx: CmdContext<'_>) -> Result<(), Error> {
  let l = ctx.loc().await;

  let Some(guild) = ctx.guild_id() else {
    return Ok(());
  };

  let Some(info) =
    ctx.data().store.get_mierda_stats(guild, ctx.author().id).await?
  else {
    ctx.say(l.t("commands.record.mierda.replies.not-yet-available")).await?;
    return Ok(());
  };

  let embed = CreateEmbed::new()
    .title(l.t("commands.record.mierda.embed.title"))
    .description(l.fmt(
      "commands.record.mierda.embed.body",
      &[("record", &info.0), ("streak", &info.1)],
    ))
    .color(Colour::from_rgb(220, 65, 65));

  ctx.send(CreateReply::default().embed(embed)).await?;

  Ok(())
}
