use crate::cmd::util::{CmdContext, CmdContextExt};
use crate::util::Error;
use poise::CreateReply;
use poise::serenity_prelude::{Colour, CreateEmbed, Mentionable};

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
              ("channel_bugs_mention", &config.channel_bugs_id.mention()),
              (
                "channel_comandos_mention",
                &config.channel_comandos_id.mention(),
              ),
              ("channel_hilos_mention", &config.channel_hilos_id.mention()),
              (
                "channel_oratorio_mention",
                &config.channel_oratorio_id.mention(),
              ),
              ("channel_yoni_mention", &config.channel_yoni_id.mention()),
            ],
          ))
          .color(Colour::BLUE),
      ),
    )
    .await?;

  Ok(())
}
