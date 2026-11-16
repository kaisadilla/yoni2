use crate::cmd::checks::check_mod_or_booster;
use crate::cmd::util::{CmdContext, CmdContextExt};
use crate::util::Error;
use poise::CreateReply;
use poise::serenity_prelude::{
  Colour, CreateEmbed, CreateMessage, Mentionable,
};

#[poise::command(
  slash_command,
  prefix_command,
  rename = "say",
  aliases("decir"),
  guild_only,
  check = "check_mod_or_booster"
)]
pub async fn say(ctx: CmdContext<'_>, message: String) -> Result<(), Error> {
  let l = ctx.loc().await;

  ctx
    .channel_id()
    .send_message(
      ctx.serenity_context(),
      CreateMessage::new().content(&message),
    )
    .await?;

  let msg_id = ctx
    .send(
      CreateReply::default().content(l.t("commands.say.reply")).ephemeral(true),
    )
    .await?;

  let Some(chan_id_log) = ctx.data().config.channel_logs_id else {
    return Ok(());
  };

  chan_id_log
    .send_message(
      ctx,
      CreateMessage::new().embed(
        CreateEmbed::new()
          .color(Colour::RED)
          .title(l.t("commands.say.log.embed.title"))
          .description(l.fmt(
            "commands.say.log.embed.body",
            &[
              ("user_name", &ctx.author().display_name()),
              ("user_mention", &ctx.author().id.mention()),
              ("content", &message),
              ("message_link", &msg_id.message().await?.link()),
            ],
          ))
          .thumbnail(
            ctx
              .author()
              .avatar_url()
              .unwrap_or(ctx.author().default_avatar_url()),
          ),
      ),
    )
    .await?;

  Ok(())
}
