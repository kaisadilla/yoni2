use crate::util::VecExt;
use poise::serenity_prelude::UserId;
use rand::RngExt;
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct ConversationModel {
  /// These represent exact responses to exact messages. This means that, when
  /// a message matches one of these responses, whatever is defined as the value
  /// of the response is sent as-is in an independent message.
  pub exact_responses: Vec<ConversationResponse>,
  pub regexes: ConversationRegexCollection,
  /// These represent partial responses to messages (which may or may not be
  /// exact). This means that, when a message contains a pattern in one of these
  /// responses, the value of the response is appended to the final, single
  /// message sent as the reply.
  pub partial_responses: Vec<ConversationPartialResponse>,
  /// Messages sent when Yoni doesn't have anything to say in his channel.
  pub fallbacks: Vec<String>,
  /// Messages sent very scarcely when users are replying to other users in
  /// Yoni's channel.
  pub interferences: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct ConversationResponse {
  #[serde(deserialize_with = "deserialize_string_or_vec")]
  pub pattern: Vec<String>,
  #[serde(deserialize_with = "deserialize_string_or_vec")]
  pub response: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct ConversationRegexCollection {
  pub haha: Vec<String>,
  pub noo: Vec<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct ConversationPartialResponse {
  pub exact: bool,
  #[serde(default)]
  pub fragment: bool,
  #[serde(deserialize_with = "deserialize_string_or_vec")]
  pub pattern: Vec<String>,
  #[serde(deserialize_with = "deserialize_response")]
  pub response: ResponseContent,
}

#[derive(Debug, Deserialize)]
pub struct ResponseContent {
  pub default: Vec<String>,
  pub custom: Option<HashMap<UserId, Vec<String>>>,
}

impl ResponseContent {
  /// Returns a message at random from all possible options in this response,
  /// with a small chance of replying with a custom message if the user_id
  /// exists in the array of custom messages.
  pub fn get(&self, user_id: UserId) -> Option<&String> {
    let rng = &mut rand::rng();

    match self.custom {
      Some(ref custom)
        if let Some(set) = custom.get(&user_id)
          && rng.random_bool(0.05) =>
      {
        set.random()
      },
      _ => self.default.random(),
    }
  }
}

fn deserialize_string_or_vec<'de, D>(
  deserializer: D,
) -> Result<Vec<String>, D::Error>
where
  D: Deserializer<'de>,
{
  #[derive(Deserialize)]
  #[serde(untagged)]
  enum StringOrVec {
    One(String),
    Many(Vec<String>),
  }

  Ok(match StringOrVec::deserialize(deserializer)? {
    StringOrVec::One(v) => vec![v],
    StringOrVec::Many(v) => v,
  })
}

fn deserialize_response<'de, D>(
  deserializer: D,
) -> Result<ResponseContent, D::Error>
where
  D: Deserializer<'de>,
{
  #[derive(Deserialize)]
  #[serde(untagged)]
  enum ResponseInput {
    One(String),
    Many(Vec<String>),
    Structured(StructuredResponse),
  }

  #[derive(Deserialize)]
  struct StructuredResponse {
    #[serde(default)]
    default: Vec<String>,
    #[serde(default)]
    custom: Option<Vec<CustomResponse>>,
  }

  #[derive(Deserialize)]
  struct CustomResponse {
    user: u64,
    responses: Vec<String>,
  }

  Ok(match ResponseInput::deserialize(deserializer)? {
    ResponseInput::One(v) => ResponseContent { default: vec![v], custom: None },
    ResponseInput::Many(v) => ResponseContent { default: v, custom: None },
    ResponseInput::Structured(v) => ResponseContent {
      default: v.default,
      custom: v.custom.map(|entries| {
        entries
          .into_iter()
          .map(|entry| (UserId::new(entry.user), entry.responses))
          .collect()
      }),
    },
  })
}
