use crate::cmd::util::{CmdContext, CmdContextExt};
use crate::util::Error;
use rand::RngExt;

#[poise::command(
  slash_command,
  prefix_command,
  rename = "gif",
  aliases("gif"),
  guild_only
)]
pub async fn gif(ctx: CmdContext<'_>) -> Result<(), Error> {
  let l = ctx.loc().await;

  match ctx.data().store.get_random_yoni_gif(ctx.guild_id().unwrap()).await? {
    Some(gif) => {
      if rand::rng().random_bool(0.01) {
        ctx.say(l.t("commands.gif.replies.disobey")).await?;
      } else {
        ctx.say(gif).await?;
      }
    },
    None => {
      ctx.say(l.t("commands.gif.replies.no-gifs")).await?;
    },
  };

  Ok(())
}
