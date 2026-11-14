use crate::config::Config;
use crate::i18n::I18n;
use crate::store::Store;

pub type Error = Box<dyn std::error::Error + Send + Sync>;

pub struct Data {
  pub config: Config,
  pub i18n: I18n,
  pub store: Store,
}

pub type Context<'a> = poise::Context<'a, Data, Error>;

/// If it's a slash command, returns the locale of the guild (e.g. server) in
/// which the command was called. Otherwise, returns the default locale.
pub fn interaction_locale<'a>(ctx: &Context<'a>) -> &'a str {
  match ctx {
    poise::Context::Application(app_ctx) => app_ctx
      .interaction
      .guild_locale
      .as_deref()
      .unwrap_or(ctx.data().i18n.default_lang()),
    poise::Context::Prefix(_) => ctx.data().i18n.default_lang(),
  }
}

pub async fn ephemeral_error(ctx: &Context<'_>, message: impl Into<String>) {
  let _ = ctx
    .send(poise::CreateReply::default().content(message).ephemeral(true))
    .await;
}

pub async fn internal_error(ctx: &Context<'_>, locale: &str) {
  ephemeral_error(ctx, ctx.data().i18n.t(locale, "error.internal")).await;
}

/// Checks if the user is in a server. If they aren't, sends a user-only error
/// indicating the command can only be used in a server.
pub async fn check_guild_only(ctx: Context<'_>) -> Result<bool, Error> {
  if ctx.guild_id().is_some() {
    return Ok(true);
  }

  let locale = interaction_locale(&ctx);
  ephemeral_error(&ctx, ctx.data().i18n.t(locale, "command.server-only")).await;
  Ok(false)
}

pub async fn notify_err_not_yet_implemented(
  ctx: &Context<'_>,
) -> Result<(), Error> {
  let locale = interaction_locale(&ctx);
  let i18n = &ctx.data().i18n;

  ephemeral_error(&ctx, i18n.t(locale, "error.not-yet-implemented")).await;

  Ok(())
}

pub async fn notify_err_no_lawyer_role(ctx: &Context<'_>) -> Result<(), Error> {
  let locale = interaction_locale(&ctx);
  let i18n = &ctx.data().i18n;

  ephemeral_error(&ctx, i18n.t(locale, "error.no-lawyer-role")).await;

  Ok(())
}
