use crate::cmd::util::{
  CmdContext, CmdContextExt, Error, check_guild_only, interaction_locale,
};
use crate::util::{is_thread, is_thread_mod, tomato_bad_command};
use poise::serenity_prelude::Message;
use tracing::debug;

#[poise::command(prefix_command, rename = "delete", aliases("borra"))]
pub async fn delete(ctx: CmdContext<'_>) -> Result<(), Error> {
  let CmdContext::Prefix(pctx) = ctx else {
    return Ok(());
  };

  let l = ctx.loc().await;

  let Some(replied_msg) = &pctx.msg.referenced_message else {
    ctx.reply(l.t("commands.thread.delete.replies.no-msg")).await?;
    return Ok(());
  };

  let author = ctx.author().id;
  let chan = ctx.channel_id().to_channel(&ctx).await?.guild().unwrap();

  // Insult the author because this isn't a thread.
  if is_thread(&chan) == false {
    ctx.reply(l.t("commands.thread.delete.replies.not-a-thread")).await?;
    tomato_bad_command(ctx).await?;

    return Ok(());
  }

  // If the author is blocked in the thread, ignore him.
  if ctx
    .data()
    .store
    .is_blocked_in_thread(chan.guild_id, chan.id, author)
    .await?
  {
    return Ok(());
  }

  // Insult the author because they aren't a mod in the thread.
  if is_thread_mod(&chan, &ctx.data().store, author).await == false {
    ctx.reply(l.t("commands.thread.delete.replies.not-a-mod")).await?;
    tomato_bad_command(ctx).await?;

    return Ok(());
  }

  debug!(
    msg_id = %replied_msg.id,
    msg_content = %replied_msg.content,
    "Message deleted as petitioned by thread mod."
  );
  replied_msg.delete(ctx).await?;

  Ok(())
}

#[poise::command(context_menu_command = "Borrar", check = "check_guild_only")]
pub async fn ctx_delete(
  ctx: CmdContext<'_>,
  target: Message,
) -> Result<(), Error> {
  debug!("Context of deletion");

  Ok(())
}
