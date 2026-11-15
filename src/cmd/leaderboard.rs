use crate::cmd::util::{
  CmdContext, Error, check_guild_only, interaction_locale,
};
use crate::util::get_guild_emoji;
use poise::CreateReply;
use poise::serenity_prelude::{Colour, CreateEmbed, Mentionable};

#[poise::command(slash_command, subcommands("tomato"))]
pub async fn leaderboard(_ctx: CmdContext<'_>) -> Result<(), Error> {
  Ok(())
}

#[poise::command(slash_command, rename = "tomato", check = "check_guild_only")]
async fn tomato(ctx: CmdContext<'_>) -> Result<(), Error> {
  let guild = ctx.guild_id().unwrap();

  let locale = interaction_locale(&ctx);
  let i18n = &ctx.data().i18n;

  let entries = ctx.data().store.top_tomatoes(guild, 10).await?;
  let monaka = get_guild_emoji(ctx.serenity_context(), guild, "monaka").await;

  let desc = if entries.is_empty() {
    i18n.t(locale, "commands.leaderboard.tomato.embed.bodies.empty")
  } else {
    &entries
      .iter()
      .enumerate()
      .map(|(idx, (usr, count))| {
        let rank = match idx {
          0 => "## 🥇".to_owned(),
          1 => "### 🥈".to_owned(),
          2 => "### 🥉".to_owned(),
          _ => format!("**\\#{}**", idx + 1),
        };

        let emoji = if ctx.data().config.user_monaka_id.as_ref() == Some(usr) {
          monaka.as_deref().unwrap_or(":tomato:")
        } else {
          ":tomato:"
        };

        format!("{rank} {} {emoji} {count}", usr.mention())
      })
      .collect::<Vec<_>>()
      .join("\n\n")
  };

  let embed = CreateEmbed::new()
    .title(i18n.t(locale, "commands.leaderboard.tomato.embed.title"))
    .description(desc)
    .color(Colour::from_rgb(220, 65, 65));

  ctx.send(CreateReply::default().embed(embed)).await?;

  Ok(())
}
