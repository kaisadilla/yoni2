use crate::cmd::checks::check_mod;
use crate::cmd::util::{CmdContext, CmdContextExt};
use crate::gif::GifStatus;
use crate::util::{Error, MessageExt};
use poise::serenity_prelude::Message;

#[poise::command(
  slash_command,
  prefix_command,
  rename = "gifs",
  aliases("gifs"),
  subcommands("remove", "forbid", "restore")
)]
pub async fn gifs(_ctx: CmdContext<'_>) -> Result<(), Error> {
  Ok(())
}

#[poise::command(
  slash_command,
  prefix_command,
  rename = "remove",
  aliases("quitar"),
  guild_only,
  check = "check_mod"
)]
async fn remove(ctx: CmdContext<'_>, url: String) -> Result<(), Error> {
  set_gif_status(ctx, url, GifStatus::Disabled).await
}

#[poise::command(
  context_menu_command = "Quitar GIF de Yoni",
  guild_only,
  check = "check_mod"
)]
pub async fn ctx_remove(
  ctx: CmdContext<'_>,
  target: Message,
) -> Result<(), Error> {
  set_gif_status_from_msg(ctx, target, GifStatus::Disabled).await
}

#[poise::command(
  slash_command,
  prefix_command,
  rename = "forbid",
  aliases("prohibir"),
  guild_only,
  check = "check_mod"
)]
async fn forbid(ctx: CmdContext<'_>, url: String) -> Result<(), Error> {
  set_gif_status(ctx, url, GifStatus::Banned).await
}

#[poise::command(
  context_menu_command = "Prohibir GIF",
  guild_only,
  check = "check_mod"
)]
pub async fn ctx_forbid(
  ctx: CmdContext<'_>,
  target: Message,
) -> Result<(), Error> {
  set_gif_status_from_msg(ctx, target, GifStatus::Banned).await
}

#[poise::command(
  slash_command,
  prefix_command,
  rename = "restore",
  aliases("restaurar"),
  guild_only,
  check = "check_mod"
)]
async fn restore(ctx: CmdContext<'_>, url: String) -> Result<(), Error> {
  set_gif_status(ctx, url, GifStatus::Allowed).await
}

/// Sets the GIF given to the status given in the store.
async fn set_gif_status(
  ctx: CmdContext<'_>,
  url: String,
  status: GifStatus,
) -> Result<(), Error> {
  let l = ctx.loc().await;

  let guild = ctx.guild_id().unwrap();

  ctx.data().store.set_gif_status(guild, url, status).await?;

  ctx.ephemeral(l.t("commands.gifs.remove.success")).await?;

  Ok(())
}

/// Sets all GIFs in the message given to the stastus given in the store.
async fn set_gif_status_from_msg(
  ctx: CmdContext<'_>,
  target: Message,
  status: GifStatus,
) -> Result<(), Error> {
  let l = ctx.loc().await;

  let guild = ctx.guild_id().unwrap();
  let urls = target.gif_urls();

  for url in urls {
    ctx.data().store.set_gif_status(guild, url, status).await?;
  }

  ctx.ephemeral(l.t("commands.gifs.remove.success")).await?;

  Ok(())
}
