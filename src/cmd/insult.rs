use crate::cmd::util::{CmdContext, CmdContextExt};
use crate::util::Error;
use poise::serenity_prelude::{Member, Mentionable};

#[poise::command(
  slash_command,
  prefix_command,
  rename = "insult",
  aliases("insultar")
)]
pub async fn insult(ctx: CmdContext<'_>, person: Member) -> Result<(), Error> {
  let l = ctx.loc().await;

  ctx
    .say(l.fmt(
      "commands.insult.replies.default",
      &[("target_mention", &person.mention())],
    ))
    .await?;

  Ok(())
}
