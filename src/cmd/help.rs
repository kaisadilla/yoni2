use crate::cmd::util::{CmdContext, CmdContextExt, Error, interaction_locale};
use poise::CreateReply;
use poise::serenity_prelude::{Colour, CreateEmbed};

#[poise::command(slash_command, rename = "help")]
pub async fn help(ctx: CmdContext<'_>) -> Result<(), Error> {
  let config = &ctx.data().config;
  let l = ctx.loc().await;

  ctx
    .send(
      CreateReply::default().embed(
        CreateEmbed::new()
          .title(l.t("commands.help.reply.title"))
          .description(l.fmt(
            "commands.help.reply.body",
            &[
              ("channel_bugs_id", &config.channel_bugs_id),
              ("channel_comandos_id", &config.channel_comandos_id),
              ("channel_hilos_id", &config.channel_hilos_id),
              ("channel_oratorio_id", &config.channel_oratorio_id),
              ("channel_yoni_id", &config.channel_yoni_id),
            ],
          ))
          .color(Colour::BLUE),
      ),
    )
    .await?;

  Ok(())
}
