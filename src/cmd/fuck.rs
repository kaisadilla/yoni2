use crate::cmd::util::{CmdContext, CmdContextExt};
use crate::util::Error;
use poise::serenity_prelude::Member;

#[poise::command(
  slash_command,
  prefix_command,
  rename = "fuck",
  aliases("métesela")
)]
pub async fn fuck(ctx: CmdContext<'_>, person: Member) -> Result<(), Error> {
  let l = ctx.loc().await;
  let config = &ctx.data().config;

  let reply = if person.user.id == config.user_yoni_id {
    l.fmt("commands.fuck.replies.yoni", &[("target_id", &config.user_yoni_id)])
  } else if person.user.id == config.user_mike_id {
    l.fmt("commands.fuck.replies.mike", &[("target_id", &config.user_mike_id)])
  } else {
    l.fmt("commands.fuck.replies.default", &[("target_id", &person.user.id)])
  };

  ctx.say(reply).await?;

  Ok(())
}
