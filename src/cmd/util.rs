use crate::config::Config;
use crate::i18n::{I18n, Locale};
use crate::store::Store;
use std::sync::Arc;

pub type Error = Box<dyn std::error::Error + Send + Sync>;

pub struct Data {
  pub config: Config,
  pub i18n: Arc<I18n>,
  pub store: Store,
}

pub type CmdContext<'a> = poise::Context<'a, Data, Error>;

pub trait CmdContextExt {
  async fn loc(&self) -> Locale;
}

impl<'a> CmdContextExt for CmdContext<'a> {
  /// Returns a Locale in this context's language.
  async fn loc(&self) -> Locale {
    self.data().i18n.locale(&self).await
  }
}

/// If it's a slash command, returns the locale of the guild (e.g. server) in
/// which the command was called. Otherwise, returns the default locale.
pub fn interaction_locale<'a>(ctx: &CmdContext<'a>) -> &'a str {
  match ctx {
    poise::Context::Application(app_ctx) => app_ctx
      .interaction
      .guild_locale
      .as_deref()
      .unwrap_or(ctx.data().i18n.default_lang()),
    poise::Context::Prefix(_) => ctx.data().i18n.default_lang(),
  }
}

pub async fn ephemeral_error(ctx: &CmdContext<'_>, message: impl Into<String>) {
  let _ = ctx
    .send(poise::CreateReply::default().content(message).ephemeral(true))
    .await;
}

pub async fn internal_error(ctx: &CmdContext<'_>, locale: &str) {
  ephemeral_error(ctx, ctx.data().i18n.t(locale, "error.internal")).await;
}

/// Checks if the user is in a server. If they aren't, sends a user-only error
/// indicating the command can only be used in a server.
pub async fn check_guild_only(ctx: CmdContext<'_>) -> Result<bool, Error> {
  if ctx.guild_id().is_some() {
    return Ok(true);
  }

  let locale = interaction_locale(&ctx);
  ephemeral_error(&ctx, ctx.data().i18n.t(locale, "command.server-only")).await;
  Ok(false)
}

pub async fn notify_err_not_yet_implemented(
  ctx: &CmdContext<'_>,
) -> Result<(), Error> {
  let locale = interaction_locale(&ctx);
  let i18n = &ctx.data().i18n;

  ephemeral_error(&ctx, i18n.t(locale, "error.not-yet-implemented")).await;

  Ok(())
}

pub async fn notify_err_no_lawyer_role(
  ctx: &CmdContext<'_>,
) -> Result<(), Error> {
  let locale = interaction_locale(&ctx);
  let i18n = &ctx.data().i18n;

  ephemeral_error(&ctx, i18n.t(locale, "error.no-lawyer-role")).await;

  Ok(())
}
