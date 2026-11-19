use serde::{Deserialize, Deserializer};

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
  #[serde(deserialize_with = "deserialize_string_or_vec")]
  pub response: Vec<String>,
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
