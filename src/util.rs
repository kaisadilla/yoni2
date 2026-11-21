use crate::config::Config;
use crate::conversation::ConversationModel;
use crate::i18n::I18n;
use crate::store::Store;
use poise::CreateReply;
use poise::serenity_prelude::{
  CacheHttp, ChannelType, ComponentInteraction, Context,
  CreateInteractionResponse, CreateInteractionResponseMessage, GuildChannel,
  GuildId, Interaction, Member, Message, Sticker, UserId,
};
use rand::seq::IteratorRandom;
use std::collections::HashSet;
use std::sync::Arc;
use tracing::debug;
use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char::is_combining_mark;

/// Takes a type as a parameter and represents the return value of a function
/// that returns an async-compatible Future.
#[macro_export]
macro_rules! async_ret {
  ($t:ty) => {
    impl std::future::Future<Output = $t> + Send
  };
}

pub struct Data {
  pub config: Config,
  pub i18n: Arc<I18n>,
  pub store: Store,
  pub client: reqwest::Client,
  pub conversation_model: ConversationModel,
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Hash = [u8; 32];

pub trait MemberExt {
  /// Returns true if this user is the owner of the server.
  fn is_server_owner(&self, cache: impl CacheHttp) -> async_ret!(bool);
  /// Returns true if this user has a mod role (as provided in the config, this
  /// does not check the `Administrator` privilege).
  fn is_mod(&self, cache: impl CacheHttp, cfg: &Config) -> async_ret!(bool);
  /// Returns true if this user is a booster in the server.
  fn is_booster(&self, cfg: &Config) -> bool;
}

impl MemberExt for Member {
  fn is_server_owner(&self, cache: impl CacheHttp) -> async_ret!(bool) {
    async move {
      if let Some(guild) =
        cache.cache().and_then(|c| self.guild_id.to_guild_cached(c))
      {
        return guild.owner_id == self.user.id;
      }

      match self.guild_id.to_partial_guild(&cache).await {
        Ok(guild) => guild.owner_id == self.user.id,
        Err(_) => false,
      }
    }
  }

  fn is_mod(&self, cache: impl CacheHttp, cfg: &Config) -> async_ret!(bool) {
    async move {
      self.roles.contains(&cfg.role_admin_id)
        || self.roles.contains(&cfg.role_mod_id)
        || self.is_server_owner(cache).await
    }
  }

  fn is_booster(&self, cfg: &Config) -> bool {
    match cfg.role_booster_id {
      Some(id) => self.roles.contains(&id),
      None => false,
    }
  }
}

pub trait GuildExt {
  /// Returns the fully qualified name of the emoji given in this guild.
  fn get_emoji(&self, cache: impl CacheHttp, name: &str) -> Option<String>;
}

impl GuildExt for GuildId {
  fn get_emoji(&self, cache: impl CacheHttp, name: &str) -> Option<String> {
    let guild = cache.cache()?.guild(self)?;
    let emoji = guild.emojis.values().find(|e| e.name == name)?;

    Some(emoji.to_string())
  }
}

pub trait GuildChannelExt {
  /// Returns true if the channel given is a post or thread.
  fn is_thread(&self) -> bool;
  /// Returns true if the user given is a mod in the thread (this counts the
  /// thread's OP as a mod).
  fn is_thread_mod(&self, store: &Store, usr: UserId) -> async_ret!(bool);
}

impl GuildChannelExt for GuildChannel {
  fn is_thread(&self) -> bool {
    matches!(
      self.kind,
      ChannelType::PublicThread
        | ChannelType::PrivateThread
        | ChannelType::NewsThread
    )
  }

  fn is_thread_mod(&self, store: &Store, usr: UserId) -> async_ret!(bool) {
    async move {
      if self.owner_id == Some(usr) {
        return true;
      }

      store.is_thread_mod(self.guild_id, self.id, usr).await.unwrap_or(false)
    }
  }
}

pub trait MessageExt {
  /// Returns all GIFs included in this message. If this message has no gif,
  /// the array will be empty.
  fn gif_urls(&self) -> Vec<&str>;
}

impl MessageExt for Message {
  fn gif_urls(&self) -> Vec<&str> {
    let mut urls: HashSet<&str> = HashSet::new();

    for att in &self.attachments {
      let is_gif = att.content_type.as_deref().is_some_and(|mime| {
        mime
          .split(';')
          .next()
          .unwrap_or("")
          .trim()
          .eq_ignore_ascii_case("image/gif")
      }) || att.filename.to_ascii_lowercase().ends_with(".gif");

      if is_gif && urls.contains(&att.url.as_str()) == false {
        urls.insert(&att.url);
      }
    }

    // GIFs embedded in external websites.
    for embed in &self.embeds {
      if embed.kind.as_deref() == Some("gifv") {
        if let Some(video) = &embed.video {
          urls.insert(&video.url);
        }
      } else if let Some(img) = &embed.image {
        if is_gif_url(&img.url) {
          urls.insert(&img.url);
        }
      }
    }

    // GIF urls contained directly in the message.
    for tok in self.content.split_whitespace() {
      let url = tok.trim_matches(|c: char| {
        matches!(
          c,
          '<' | '>' | '(' | ')' | '[' | ']' | ',' | ';' | '!' | '?' | '.'
        )
      });

      if (url.starts_with("https://") || url.starts_with("http://"))
        && is_gif_url(url)
      {
        urls.insert(&url);
      }
    }

    debug!(?urls, "GIFS found");

    urls.into_iter().collect()
  }
}

pub trait InteractionExt {
  fn ephemeral(
    &self,
    cache: impl CacheHttp,
    message: impl Into<String>,
  ) -> async_ret!(Result<(), Error>);
}

impl InteractionExt for ComponentInteraction {
  fn ephemeral(
    &self,
    cache: impl CacheHttp,
    message: impl Into<String>,
  ) -> async_ret!(Result<(), Error>) {
    let message = message.into();

    async move {
      self
        .create_response(
          cache,
          CreateInteractionResponse::Message(
            CreateInteractionResponseMessage::new()
              .content(message)
              .ephemeral(true),
          ),
        )
        .await
        .map_err(Into::into)
    }
  }
}

pub trait VecExt<T> {
  fn random(&self) -> Option<&T>;
}

impl<T> VecExt<T> for Vec<T> {
  fn random(&self) -> Option<&T> {
    let mut rng = rand::rng();

    self.iter().choose(&mut rng)
  }
}

fn is_gif_url(url: &str) -> bool {
  let path = url.split(|c| c == '?' || c == '#').next().unwrap_or(url);

  path.to_ascii_lowercase().ends_with(".gif")
}

/// Strips diacritics in the string given. This will turn Ñ into N.
pub fn strip_diacritics(s: &str) -> String {
  s.nfd().filter(|c| is_combining_mark(*c) == false).collect()
}
