use crate::async_ret;
use crate::i18n::Locale;
use crate::util::{Data, Error};
use poise::serenity_prelude::ReactionType;
use poise::{CreateReply, ReplyHandle};
use rand::RngExt;

pub type CmdContext<'a> = poise::Context<'a, Data, Error>;

pub trait CmdContextExt {
  /// Returns a Locale in this context's language.
  fn loc(&self) -> async_ret!(Locale);
  /// If the context given belongs to a prefix command, has a 1/3 chance to
  /// tomato the command message.
  fn tomato_bad_command(&self) -> async_ret!(anyhow::Result<()>);
  /// Sends the message given to the user as an ephemereal message.
  fn ephemeral(
    &self,
    message: impl Into<String>,
  ) -> async_ret!(Result<ReplyHandle<'_>, Error>);
  /// Sends the message given to the user as an ephemereal error.
  fn ephemeral_error(&self, message: impl Into<String>) -> async_ret!(());
  /// Sends a generic "internal error" message to the user.
  fn internal_error(&self) -> async_ret!(());
  /// Sends a generic "not yet implemented" message to the user.
  fn not_yet_implemented(&self) -> async_ret!(());
}

impl<'a> CmdContextExt for CmdContext<'a> {
  fn loc(&self) -> async_ret!(Locale) {
    self.data().i18n.locale(&self)
  }

  fn tomato_bad_command(&self) -> async_ret!(anyhow::Result<()>) {
    async move {
      if let poise::Context::Prefix(pctx) = self {
        if rand::rng().random_bool(0.3334) {
          pctx.msg.react(self, ReactionType::Unicode("🍅".to_owned())).await?;
        }
      }

      Ok(())
    }
  }

  fn ephemeral(
    &self,
    message: impl Into<String>,
  ) -> async_ret!(Result<ReplyHandle<'_>, Error>) {
    let message = message.into();
    async move {
      self
        .send(CreateReply::default().content(message).ephemeral(true))
        .await
        .map_err(Into::into)
    }
  }

  fn ephemeral_error(&self, message: impl Into<String>) -> async_ret!(()) {
    let message = message.into();
    async move {
      let _ = self
        .send(CreateReply::default().content(message).ephemeral(true))
        .await;
    }
  }

  fn internal_error(&self) -> async_ret!(()) {
    async move {
      let _ = self.ephemeral_error(self.loc().await.t("error.internal")).await;
    }
  }

  fn not_yet_implemented(&self) -> async_ret!(()) {
    async move {
      let _ = self
        .ephemeral_error(self.loc().await.t("error.not-yet-implemented"))
        .await;
    }
  }
}
