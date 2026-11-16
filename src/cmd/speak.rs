use crate::cmd::util::{CmdContext, CmdContextExt};
use crate::util::Error;

#[poise::command(
  slash_command,
  prefix_command,
  rename = "speak",
  aliases("habla")
)]
pub async fn speak(ctx: CmdContext<'_>) -> Result<(), Error> {
  let l = ctx.loc().await;

  let reply = l.t("commands.speak.reply");

  ctx.say(reply).await?;

  Ok(())
}
