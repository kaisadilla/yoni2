use anyhow::Context;
use poise::serenity_prelude::{RoleId, UserId};

pub struct Config {
  pub prefix: String,

  pub bot_token: String,
  pub default_lang: String,
  pub database_url: String,

  pub channel_bugs_id: String,
  pub channel_comandos_id: String,
  pub channel_hilos_id: String,
  pub channel_oratorio_id: String,
  pub channel_yoni_id: String,

  pub user_yoni_id: UserId,
  pub user_mike_id: UserId,
  pub user_monaka_id: Option<UserId>,

  pub role_admin_id: RoleId,
  pub role_mod_id: RoleId,
}

impl Config {
  pub fn load() -> anyhow::Result<Self> {
    Ok(Self {
      prefix: mandatory("PREFIX")?,

      bot_token: mandatory("BOT_TOKEN")?,
      default_lang: mandatory("DEFAULT_LANG")?,
      database_url: mandatory("DATABASE_URL")?,

      channel_bugs_id: mandatory("CHANNEL_BUGS_ID")?,
      channel_comandos_id: mandatory("CHANNEL_COMANDOS_ID")?,
      channel_hilos_id: mandatory("CHANNEL_HILOS_ID")?,
      channel_oratorio_id: mandatory("CHANNEL_ORATORIO_ID")?,
      channel_yoni_id: mandatory("CHANNEL_YONI_ID")?,

      user_yoni_id: mandatory("USER_YONI_ID")?.parse()?,
      user_mike_id: mandatory("USER_MIKE_ID")?.parse()?,
      user_monaka_id: option("USER_MONAKA_ID").and_then(|s| s.parse().ok()),

      role_admin_id: mandatory("ROLE_ADMIN_ID")?.parse()?,
      role_mod_id: mandatory("ROLE_MOD_ID")?.parse()?,
    })
  }
}

/// Reads an environment variable that is required to exist in the .env file.
/// If it doesn't, returns an error.
fn mandatory(key: &str) -> anyhow::Result<String> {
  std::env::var(key).context(format!("Missing environment variable '{key}'."))
}

/// Reads an environment variable for a value that isn't required to run the
/// bot.
fn option(key: &str) -> Option<String> {
  std::env::var(key).ok()
}
