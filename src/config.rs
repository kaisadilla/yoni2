use anyhow::Context;
use poise::serenity_prelude::{ChannelId, RoleId, UserId};
use std::str::FromStr;

pub struct Config {
  pub prefix: String,

  pub bot_token: String,
  pub default_lang: String,
  pub database_url: String,

  pub channel_bugs_id: ChannelId,
  pub channel_comandos_id: ChannelId,
  /// The id of the channel used to send logs of the bot's activity.
  pub channel_hilos_id: ChannelId,
  pub channel_logs_id: Option<ChannelId>,
  pub channel_oratorio_id: ChannelId,
  pub channel_yoni_id: ChannelId,

  pub user_yoni_id: UserId,
  pub user_mike_id: UserId,
  pub user_monaka_id: Option<UserId>,

  pub role_admin_id: RoleId,
  pub role_booster_id: Option<RoleId>,
  pub role_mod_id: RoleId,
}

impl Config {
  pub fn load() -> anyhow::Result<Self> {
    Ok(Self {
      prefix: mandatory("PREFIX")?,

      bot_token: mandatory("BOT_TOKEN")?,
      default_lang: mandatory("DEFAULT_LANG")?,
      database_url: mandatory("DATABASE_URL")?,

      channel_bugs_id: mandatory_as("CHANNEL_BUGS_ID")?,
      channel_comandos_id: mandatory_as("CHANNEL_COMANDOS_ID")?,
      channel_hilos_id: mandatory_as("CHANNEL_HILOS_ID")?,
      channel_logs_id: option_as("CHANNEL_LOGS_ID"),
      channel_oratorio_id: mandatory_as("CHANNEL_ORATORIO_ID")?,
      channel_yoni_id: mandatory_as("CHANNEL_YONI_ID")?,

      user_yoni_id: mandatory_as("USER_YONI_ID")?,
      user_mike_id: mandatory_as("USER_MIKE_ID")?,
      user_monaka_id: option_as("USER_MONAKA_ID"),

      role_admin_id: mandatory_as("ROLE_ADMIN_ID")?,
      role_booster_id: option_as("ROLE_BOOSTER_ID"),
      role_mod_id: mandatory_as("ROLE_MOD_ID")?,
    })
  }
}

/// Reads an environment variable that is required to exist in the .env file.
/// If it doesn't, returns an error.
fn mandatory(key: &str) -> anyhow::Result<String> {
  std::env::var(key).context(format!("Missing environment variable '{key}'."))
}

fn mandatory_as<T: FromStr>(key: &str) -> anyhow::Result<T>
where
  T::Err: std::error::Error + Send + Sync + 'static,
{
  mandatory(key)?.parse::<T>().with_context(|| format!("Failed to parse {key}"))
}

/// Reads an environment variable for a value that isn't required to run the
/// bot.
fn option(key: &str) -> Option<String> {
  std::env::var(key).ok()
}

/// Reads an optional variable and parses it into the type given.
fn option_as<T: FromStr>(key: &str) -> Option<T> {
  option(key).and_then(|s| s.parse().ok())
}
