use include_dir::Dir;
use rand::seq::IteratorRandom;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::Display;

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

  /// Retrieves the string for the key and locale given if it exists. If the
  /// value associated to the given key is an array of strings, returns one of
  /// these at random.
  pub fn get(&self, locale: &str, key: &str) -> Option<&str> {
    let mut path = key.split('.');
    let mut val = self.languages.get(locale)?.get(path.next()?)?;

    for name in path {
      val = val.as_table()?.get(name)?;
    }

    match val {
      toml::Value::String(val) => Some(val.as_str()),

      toml::Value::Array(vals) => {
        // If the value array doesn't contain any string, we return None.
        if vals.iter().any(|v| matches!(v, toml::Value::String(_)) == false) {
          return None;
        }

        // Return a string inside the array at random.
        vals.iter().filter_map(toml::Value::as_str).choose(&mut rand::rng())
      },

      _ => None,
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
