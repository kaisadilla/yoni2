use crate::cmd::util::{Context, Error, interaction_locale};
use poise::serenity_prelude::Member;

#[poise::command(
  slash_command,
  prefix_command,
  rename = "fuck",
  aliases("métesela")
)]
pub async fn fuck(ctx: Context<'_>, person: Member) -> Result<(), Error> {
  let locale = interaction_locale(&ctx);
  let config = &ctx.data().config;
  let i18n = &ctx.data().i18n;

  let reply = if person.user.id == config.user_yoni_id {
    i18n.f(
      locale,
      "commands.fuck.replies.yoni",
      &[("target_id", &config.user_yoni_id)],
    )
  } else if person.user.id == config.user_mike_id {
    i18n.f(
      locale,
      "commands.fuck.replies.mike",
      &[("target_id", &config.user_mike_id)],
    )
  } else {
    i18n.f(
      locale,
      "commands.fuck.replies.default",
      &[("target_id", &person.user.id)],
    )
  };

  ctx.say(reply).await?;

  Ok(())
}
