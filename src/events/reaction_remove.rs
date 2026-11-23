use crate::util::Data;
use poise::serenity_prelude::{Context, Reaction};

pub async fn reaction_remove(data: &Data, ctx: &Context, reaction: &Reaction) {
  discount_tomato(data, ctx, reaction).await;
}

async fn discount_tomato(data: &Data, ctx: &Context, reaction: &Reaction) {
  if reaction.emoji.unicode_eq("🍅") == false {
    return;
  }
  let Some(guild_id) = reaction.guild_id else {
    return;
  };

  let target_usr = match reaction.message(ctx).await {
    Ok(msg) => msg.author.id,
    Err(_) => return,
  };

  data.store.remove_tomato(guild_id, target_usr).await.ok();
}
