use crate::cmd::util::{Context, Error, interaction_locale};
use poise::CreateReply;
use poise::serenity_prelude::{Colour, CreateEmbed};

#[poise::command(slash_command, rename = "help")]
pub async fn help(ctx: Context<'_>) -> Result<(), Error> {
  let locale = interaction_locale(&ctx);
  let config = &ctx.data().config;
  let i18n = &ctx.data().i18n;

  ctx
    .send(
      CreateReply::default().embed(
        CreateEmbed::new()
          .title(i18n.t(locale, "commands.help.reply.title"))
          .description(i18n.f(
            locale,
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
