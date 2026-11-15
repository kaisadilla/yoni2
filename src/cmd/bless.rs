use crate::cmd::util::{CmdContext, CmdContextExt, Error, interaction_locale};
use poise::serenity_prelude::Member;

#[poise::command(
  slash_command,
  prefix_command,
  rename = "bless",
  aliases("bendice")
)]
pub async fn bless(ctx: CmdContext<'_>, person: Member) -> Result<(), Error> {
  let l = ctx.loc().await;

  let reply = if person.user.id == ctx.author().id {
    l.fmt(
      "commands.bless.replies.self-bless",
      &[("target_id", &person.user.id.to_string())],
    )
  } else {
    l.fmt(
      "commands.bless.replies.default",
      &[("target_id", &person.user.id.to_string())],
    )
  };

  ctx.say(reply).await?;

  Ok(())
}
