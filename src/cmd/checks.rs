use crate::cmd::util::CmdContext;
use crate::util::{Error, MemberExt};

// TODO: Add error messages insulting the user.

/// Requires the user to be in a server and have a mod or booster role in it.
pub async fn check_mod_or_booster(ctx: CmdContext<'_>) -> Result<bool, Error> {
  let Some(member) = ctx.author_member().await else {
    return Ok(false);
  };

  let cfg = &ctx.data().config;

  Ok(member.is_mod(&ctx, cfg).await || member.is_booster(cfg))
}

/// Requires the user to be in a server and have a mod role in it.
pub async fn check_mod(ctx: CmdContext<'_>) -> Result<bool, Error> {
  let Some(member) = ctx.author_member().await else {
    return Ok(false);
  };

  let cfg = &ctx.data().config;

  Ok(member.is_mod(&ctx, cfg).await)
}
