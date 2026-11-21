use crate::cmd::util::{CmdContext, CmdContextExt};
use crate::util::Error;
use poise::serenity_prelude::{Member, Mentionable};

#[poise::command(
  slash_command,
  prefix_command,
  rename = "bless",
  aliases("bendecir")
)]
pub async fn bless(ctx: CmdContext<'_>, person: Member) -> Result<(), Error> {
  let l = ctx.loc().await;

  let reply = if person.user.id == ctx.author().id {
    l.fmt(
      "commands.bless.replies.self-bless",
      &[("target_mention", &person.user.mention())],
    )
  } else {
    l.fmt(
      "commands.bless.replies.default",
      &[("target_mention", &person.user.mention())],
    )
  };

  ctx.say(reply).await?;

  Ok(())
}
