use crate::cmd::util::{CmdContext, CmdContextExt};
use crate::util::Error;
use poise::CreateReply;
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::{Colour, CreateEmbed, Mentionable};
use serde_json::json;

#[poise::command(
  slash_command,
  prefix_command,
  rename = "news",
  aliases("novedades")
)]
pub async fn news(ctx: CmdContext<'_>) -> Result<(), Error> {
  let l = ctx.loc().await;

  let payload = json!({
    "flags": 1 << 15,
    "components": [
      {
        "type": 17,
        "accent_color": 0x22e9a9,
        "components": [
          {
            "type": 12,
            "items": [
              {
                "media": {
                  "url": l.t("commands.news.components.header-img")
                }
              }
            ]
          },
          {
            "type": 10,
            "content": l.fmt(
              "commands.news.components.header",
              &[(
                "channel_bugs_mention",
                &ctx.data().config.channel_bugs_id.mention()
              )]
            )
          },
          {
            "type": 14,
            "divider": true,
            "spacing": 1,
          },
          {
            "type": 10,
            "content": l.t("commands.news.components.new"),
          },
          {
            "type": 10,
            "content": l.t("commands.news.components.changes"),
          },
          {
            "type": 10,
            "content": l.t("commands.news.components.fixes"),
          }
        ]
      }
    ]
  });

  ctx
    .data()
    .client
    .post(format!(
      "https://discord.com/api/v10/channels/{}/messages",
      ctx.channel_id()
    ))
    .header(
      reqwest::header::AUTHORIZATION,
      format!("Bot {}", ctx.data().config.bot_token),
    )
    .json(&payload)
    .send()
    .await?
    .error_for_status()?;

  ctx.ephemeral(":heart:").await?;

  Ok(())
}

pub async fn news2(ctx: CmdContext<'_>) -> Result<(), Error> {
  let l = ctx.loc().await;

  let embed = CreateEmbed::new()
    .title("Novedades")
    .description(format!(
      r#"\
      ## Versión v2-BETA-1
      ¡Yoni ha sido reescrito! Su funcionamiento se mantiene prácticamente
      igual, pero iba necesitando un lavado de cara. Como no tengo tiempo
      suficiente para testear bien el bot, estoy seguro de que encontraréis
      nuevos y apasionantes bugs que podréis reportar en {}.

      ### ✨ Nuevo
      - Ahora se pueden usar los comandos ordenándoselo a Yoni directamente\
        (e.g. escribiendo "yoni gif" en lugar de usar `/gif`)
      - Añadido el comando `/juegos` (que sólo tiene un juego).
      - Añadido el comando `/ránkings` (que sólo tiene un ránking).
      - Añadidos moderadores para hilos - utiliza `/hilo mod` e `/hilo quitarmod`\
        para gestionarlos.

      ### 🛠️ Cambios
      - `/sendmessage` ahora es `/decir`.
      - `/gif` ahora envía un único GIF siempre.
      - Eliminado `/comandos` y `/preséntate`.
      - Cambios menores en el comportamiento de Yoni.
      - Algunos comandos tienen nombres ligeramente diferentes.

      ### 🐛 Fixes
      - Yoni ya no te borrará comandos y te insultará por ello aleatoriamente.
    "#,
      { ctx.data().config.channel_bugs_id.mention() }
    ))
    .color(Colour::new(0x22e9a9));

  ctx.send(CreateReply::default().embed(embed)).await?;

  Ok(())
}
