use crate::cmd::util::CmdContext;
use include_dir::Dir;
use rand::seq::IteratorRandom;
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::sync::Arc;
use tracing::error;

pub static LANG_DIR: Dir =
  include_dir::include_dir!("$CARGO_MANIFEST_DIR/lang");

pub struct I18n {
  default_lang: String,
  languages: HashMap<String, toml::Table>,
}

impl I18n {
  pub fn load(default_lang: impl Into<String>) -> Self {
    let mut languages = HashMap::new();

    for file in LANG_DIR.files() {
      let path = file.path().display();

      if file.path().extension().and_then(|ext| ext.to_str()) != Some("toml") {
        continue;
      }

      let locale =
        file.path().file_stem().unwrap().to_str().unwrap().to_owned();

      let table = toml::from_str(file.contents_utf8().unwrap())
        .unwrap_or_else(|err| panic!("Invalid language file {path}: {err}"));

      languages.insert(locale, table);
    }

    Self { default_lang: default_lang.into(), languages }
  }

  /// Returns a locale with the language that should be used for the specific
  /// context given. That is: the server's language if possible, else the bot's
  /// default language.
  pub async fn locale(self: &Arc<Self>, ctx: &CmdContext<'_>) -> Locale {
    Locale {
      i18n: self.clone(),
      lang: match ctx {
        CmdContext::Application(actx) => actx.interaction.guild_locale.clone(),
        CmdContext::Prefix(_) => {
          ctx.partial_guild().await.map(|g| g.preferred_locale)
        },
      }
      .unwrap_or_else(|| self.default_lang.clone()),
    }
  }

  /// Returns a locale with the bot's default language.
  pub fn default_locale(self: &Arc<Self>) -> Locale {
    Locale { i18n: self.clone(), lang: self.default_lang.clone() }
  }

  /// Retrieves the string for the key and locale given if it exists. If the
  /// value associated to the given key is an array of strings, returns one of
  /// these at random.
  pub fn get(&self, locale: &str, key: &str) -> Option<&str> {
    let mut current_key = key.to_owned();
    let mut visited = HashSet::new();

    // A loop to resolve redirects (values in the format of `::key`).
    loop {
      if visited.insert(current_key.clone()) == false {
        error!(origin = key, "Lang keys form a circular dependency.");
      }

      let mut path = current_key.split('.');
      let mut val = self.languages.get(locale)?.get(path.next()?)?;

      for name in path {
        val = val.as_table()?.get(name)?;
      }

      let val = match val {
        toml::Value::String(val) => val.as_str(),

        toml::Value::Array(vals) => {
          // If the value array doesn't contain any string, we return None.
          if vals.iter().any(|v| matches!(v, toml::Value::String(_)) == false) {
            return None;
          }

          // Return a string inside the array at random. If the array is empty,
          // returns None.
          match vals
            .iter()
            .filter_map(toml::Value::as_str)
            .choose(&mut rand::rng())
          {
            Some(v) => v,
            None => return None,
          }
        },

        _ => return None,
      };

      if let Some(alias) = val.strip_prefix("::") {
        current_key = alias.to_owned();
      } else {
        return Some(val);
      }
    }
  }

  pub fn default_lang(&self) -> &str {
    self.default_lang.as_str()
  }

  /// Retrieves the definite string for the key and locale given.
  pub fn t<'a>(&'a self, locale: &'a str, key: &'a str) -> &'a str {
    self
      .get(locale, key)
      .or_else(|| self.get(&self.default_lang, key))
      .unwrap_or(key)
  }

  /// Retrieves the definite string for the key and locale given, applying
  /// the arguments given to the string's template parameters.
  pub fn f(
    &self,
    locale: &str,
    key: &str,
    args: &[(&str, &(dyn Display + Send + Sync))],
  ) -> String {
    let Some(base) =
      self.get(locale, key).or_else(|| self.get(&self.default_lang, key))
    else {
      return String::from(key);
    };

    let mut res = base.to_string();

    for (k, v) in args {
      res = res.replace(&format!("{{{}}}", k), &format!("{v}"));
    }

    res
  }

  /// Retrieves the definite string for a boolean value (true or false).
  pub fn b(&self, locale: &str, val: bool) -> &str {
    let key = if val { "value.bool.true" } else { "value.bool.false" };

    self
      .get(locale, key)
      .or_else(|| self.get(&self.default_lang, key))
      .unwrap_or(key)
  }

  pub fn locales(&self) -> impl Iterator<Item = &str> {
    self.languages.keys().map(String::as_str)
  }

  /// Collects every locale's translation of the key given into a Discord-ready
  /// localization map (langcode -> string). Locales that don't define the key
  /// are simply omitted, so Discord falls back to the command's base name and
  /// description for them.
  pub fn localizations(
    &self,
    key: &str,
  ) -> Cow<'static, [(Cow<'static, str>, Cow<'static, str>)]> {
    Cow::Owned(
      self
        .languages
        .keys()
        .filter_map(|locale| {
          self
            .get(locale, key)
            .map(|val| (Cow::Owned(locale.clone()), Cow::Owned(val.to_owned())))
        })
        .collect(),
    )
  }
}

/// Wraps an I18n instance and returns values from it with the language assigned
/// to this Locale.
pub struct Locale {
  lang: String,
  i18n: Arc<I18n>,
}

impl Locale {
  /// Retrieves the definite string for the key and locale given.
  pub fn t<'a>(&'a self, key: &'a str) -> &'a str {
    self.i18n.t(&self.lang, key)
  }

  /// Retrieves the definite string for the key and locale given, applying
  /// the arguments given to the string's template parameters.
  pub fn fmt(
    &self,
    key: &str,
    args: &[(&str, &(dyn Display + Send + Sync))],
  ) -> String {
    self.i18n.f(&self.lang, key, args)
  }

  /// Retrieves the definite string for a boolean value (true or false).
  pub fn bool(&self, val: bool) -> &str {
    self.i18n.b(&self.lang, val)
  }
}
