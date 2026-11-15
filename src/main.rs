use crate::cmd::build_command_array;
use crate::cmd::util::{Data, Error, interaction_locale, internal_error};
use crate::config::Config;
use crate::events::event_handler;
use crate::i18n::I18n;
use crate::store::Store;
use poise::builtins::register_globally;
use poise::serenity_prelude::{Client, GatewayIntents};
use poise::{
  Command, Framework, FrameworkError, FrameworkOptions, PrefixFrameworkOptions,
};
use std::borrow::Cow;
use std::sync::Arc;
use tracing::{debug, error, info, trace, warn};
use tracing_subscriber::EnvFilter;

pub mod cmd;
pub mod config;
pub mod events;
pub mod i18n;
pub mod store;
pub mod util;

#[tokio::main]
async fn main() -> Result<(), Error> {
  dotenvy::dotenv().ok();

  let filter = EnvFilter::try_from_default_env()
    .unwrap_or_else(|_| EnvFilter::new("info"));

  tracing_subscriber::fmt().with_env_filter(filter).init();

  trace!("Tracing level: TRACE");
  debug!("Tracing level: DEBUG");
  info!("Tracing level: INFO");

  let config = Config::load()?;

  let tok = config.bot_token.clone();
  let store = Store::load(&config.database_url).await?;

  let intents =
    GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
  let i18n = Arc::new(I18n::load(config.default_lang.clone()));

  let mut commands = build_command_array();

  for cmd in &mut commands {
    localize_cmd(&i18n, cmd, String::from("commands"));
  }

  let framework = Framework::builder()
    .options(FrameworkOptions {
      commands,
      prefix_options: PrefixFrameworkOptions {
        prefix: Some(Cow::Owned(config.prefix.clone())),
        ..Default::default()
      },
      event_handler: |framework, evt| Box::pin(event_handler(framework, evt)),
      on_error: |err| Box::pin(on_error(err)),
      ..Default::default()
    })
    .setup(|ctx, ready, framework| {
      Box::pin(async move {
        info!(user = ready.user.name, "Logged in.");

        register_globally(ctx, &framework.options().commands).await?;

        Ok(Data { config, i18n, store })
      })
    })
    .build();

  let mut client = Client::builder(tok, intents).framework(framework).await?;

  client.start().await?;

  Ok(())
}

fn localize_cmd(i18n: &I18n, command: &mut Command<Data, Error>, path: String) {
  let cmd_key = if command.name == "name" { "_name" } else { &command.name };

  println!("Cmd: {}, Key: {}, Path: {}", command.name, cmd_key, path);

  command.name_localizations =
    i18n.localizations(&format!("{path}.{cmd_key}.name"));
  command.description_localizations =
    i18n.localizations(&format!("{path}.{cmd_key}.desc"));

  for param in &mut command.parameters {
    let param_key = if param.name == "name" { "_name" } else { &param.name };
    let param_path = format!("{path}.{cmd_key}.{param_key}");

    param.name_localizations =
      i18n.localizations(&(param_path.clone() + ".name"));
    param.description_localizations =
      i18n.localizations(&(param_path.clone() + ".desc"));

    for choice in param.choices.to_mut() {
      let choice_key =
        if choice.name == "name" { "_name" } else { &choice.name };

      choice.localizations =
        i18n.localizations(&format!("{param_path}.{choice_key}.name"));
    }
  }

  for subcommand in &mut command.subcommands {
    localize_cmd(i18n, subcommand, format!("{}.{}", path, command.name));
  }
}

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
  match error {
    FrameworkError::Command { error, ctx, .. } => {
      error!("Error in command '{}': {:?}", ctx.command().name, error);

      let locale = interaction_locale(&ctx);
      internal_error(&ctx, locale).await;
    },
    other => {
      if let Err(err) = poise::builtins::on_error(other).await {
        error!("Error while handling error: {:?}", err);
      }
    },
  }
}
