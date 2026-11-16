use crate::cmd::util::{CmdContext, CmdContextExt};
use crate::i18n::Locale;
use crate::util::{Error, GuildChannelExt};
use poise::serenity_prelude::Message;
use tracing::debug;

#[poise::command(prefix_command, rename = "delete", aliases("borra"))]
pub async fn delete(
  ctx: CmdContext<'_>,
  target: Option<Message>,
) -> Result<(), Error> {
  let mut needs_response = true;

  let target = match &target {
    Some(target) => target,
    None => {
      let CmdContext::Prefix(pctx) = ctx else {
        return Ok(());
      };

      needs_response = false;
      // Delete the message containing this command.
      pctx.msg.delete(ctx.serenity_context()).await?;

      match &pctx.msg.referenced_message {
        Some(msg) => msg,
        None => return Ok(()),
      }
    },
  };

  run_cmd(&ctx, &target, needs_response).await
}

#[poise::command(context_menu_command = "Borrar", guild_only)]
pub async fn ctx_delete(
  ctx: CmdContext<'_>,
  target: Message,
) -> Result<(), Error> {
  run_cmd(&ctx, &target, true).await
}

async fn run_cmd(
  ctx: &CmdContext<'_>,
  replied_msg: &Message,
  needs_response: bool,
) -> Result<(), Error> {
  let l = ctx.loc().await;

  let author = ctx.author().id;
  let chan = ctx.channel_id().to_channel(&ctx).await?.guild().unwrap();

  // Insult the author because this isn't a thread.
  if chan.is_thread() == false {
    ctx.reply(l.t("commands.thread.delete.replies.not-a-thread")).await?;
    ctx.tomato_bad_command().await.ok();

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
  if chan.is_thread_mod(&ctx.data().store, author).await == false {
    ctx.reply(l.t("commands.thread.delete.replies.not-a-mod")).await?;
    ctx.tomato_bad_command().await.ok();

    return Ok(());
  }

  debug!(
    msg_id = %replied_msg.id,
    msg_content = %replied_msg.content,
    "Message deleted as petitioned by thread mod."
  );
  replied_msg.delete(ctx).await?;

  if needs_response == false {
    return Ok(());
  }

  ctx.ephemeral(l.t("commands.thread.delete.success")).await?;

  Ok(())
}
