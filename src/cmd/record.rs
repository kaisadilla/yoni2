use crate::cmd::util::{
  CmdContext, Error, check_guild_only, interaction_locale,
};
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
  check = "check_guild_only"
)]
pub async fn mierda(ctx: CmdContext<'_>) -> Result<(), Error> {
  let locale = interaction_locale(&ctx);

  let i18n = &ctx.data().i18n;

  let Some(guild) = ctx.guild_id() else {
    return Ok(());
  };

  let Some(info) =
    ctx.data().store.get_mierda_stats(guild, ctx.author().id).await?
  else {
    ctx
      .say(i18n.t(locale, "commands.record.mierda.replies.not-yet-available"))
      .await?;
    return Ok(());
  };

  let embed = CreateEmbed::new()
    .title(i18n.t(locale, "commands.record.mierda.embed.title"))
    .description(i18n.f(
      locale,
      "commands.record.mierda.embed.body",
      &[("record", &info.0), ("streak", &info.1)],
    ))
    .color(Colour::from_rgb(220, 65, 65));

  ctx.send(CreateReply::default().embed(embed)).await?;

  Ok(())
}
