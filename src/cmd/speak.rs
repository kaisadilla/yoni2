use crate::cmd::util::{CmdContext, Error, interaction_locale};

#[poise::command(
  slash_command,
  prefix_command,
  rename = "speak",
  aliases("habla")
)]
pub async fn speak(ctx: CmdContext<'_>) -> Result<(), Error> {
  let locale = interaction_locale(&ctx);

  let reply = ctx.data().i18n.t(locale, "commands.speak.reply");

  ctx.say(reply).await?;

  Ok(())
}
