use crate::util::Data;
use poise::serenity_prelude::{Context, Reaction};

pub async fn reaction_add(data: &Data, ctx: &Context, reaction: &Reaction) {
  count_tomato(data, reaction).await;
}

async fn count_tomato(data: &Data, reaction: &Reaction) {
  if reaction.emoji.unicode_eq("🍅") == false {
    return;
  }
  let Some(guild_id) = reaction.guild_id else {
    return;
  };
  let Some(target_usr) = reaction.message_author_id else {
    return;
  };

  data.store.add_tomato(guild_id, target_usr).await.ok();
}
