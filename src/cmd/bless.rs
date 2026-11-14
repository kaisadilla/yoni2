use crate::cmd::util::{Context, Error, interaction_locale};
use poise::serenity_prelude::Member;

#[poise::command(
  slash_command,
  prefix_command,
  rename = "bless",
  aliases("bendice")
)]
pub async fn bless(ctx: Context<'_>, person: Member) -> Result<(), Error> {
  let locale = interaction_locale(&ctx);

  let reply = if person.user.id == ctx.author().id {
    ctx.data().i18n.f(
      locale,
      "commands.bless.replies.self-bless",
      &[("target_id", &person.user.id.to_string())],
    )
  } else {
    ctx.data().i18n.f(
      locale,
      "commands.bless.replies.default",
      &[("target_id", &person.user.id.to_string())],
    )
  };

  ctx.say(reply).await?;

  Ok(())
}
