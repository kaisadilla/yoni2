use crate::cmd::util::{CmdContext, CmdContextExt};
use crate::util::{Error, GuildExt};
use poise::CreateReply;
use poise::serenity_prelude::{Colour, CreateEmbed, Mentionable};

#[poise::command(slash_command, subcommands("tomato"))]
pub async fn leaderboard(_ctx: CmdContext<'_>) -> Result<(), Error> {
  Ok(())
}

#[poise::command(slash_command, rename = "tomato", guild_only)]
async fn tomato(ctx: CmdContext<'_>) -> Result<(), Error> {
  let l = ctx.loc().await;
  let guild = ctx.guild_id().unwrap();

  let entries = ctx.data().store.top_tomatoes(guild, 10).await?;
  let monaka = guild.get_emoji(&ctx, "monaka");

  let desc = if entries.is_empty() {
    l.t("commands.leaderboard.tomato.embed.bodies.empty")
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
    .title(l.t("commands.leaderboard.tomato.embed.title"))
    .description(desc)
    .color(Colour::from_rgb(220, 65, 65));

  ctx.send(CreateReply::default().embed(embed)).await?;

  Ok(())
}
