use crate::cmd::util::Data;
use ab_glyph::FontArc;
use image::{ImageFormat, Rgba};
use imageproc::drawing::{draw_text_mut, text_size};
use poise::serenity_prelude::{
  Context, CreateAttachment, CreateEmbed, CreateMessage, Message,
};
use regex::Regex;
use std::sync::OnceLock;
use tracing::{error, info};

pub async fn message(data: &Data, ctx: &Context, message: &Message) {
  if message.author.id == data.config.user_mike_id {
    check_mike_mierda(data, ctx, message).await;
  }
}

async fn check_mike_mierda(data: &Data, ctx: &Context, msg: &Message) {
  info!("Checking mike mierda.");
  const KW: [&str; 7] =
    ["cag", "mierd", "diarre", "defeca", "hece", "hez", "caca"];

  let Some(guild_id) = msg.guild_id else {
    return;
  };
  let content = msg.content.to_lowercase();

  // Check whether the user is talking about mierda.
  let has_kw = KW
    .iter()
    .any(|&kw| content.contains(kw) || regex_culo().is_match(&content));

  if has_kw == false {
    return;
  }

  let locale = data.i18n.default_lang();

  let Ok(Some((time_since_last, is_record))) =
    data.store.update_mierda(guild_id, msg.author.id).await
  else {
    return;
  };

  // If it's been more than 24 h since the last mierda, we send a funny picture.
  if time_since_last >= 1 {
    let key = if is_record {
      "events.last-mierda.replies.record"
    } else {
      "events.last-mierda.replies.default"
    };

    let Ok(Ok(img)) =
      tokio::task::spawn_blocking(move || generate_last_img(15)).await
    else {
      error!("Failed to generate image for last-mierda.");
      return;
    };

    let att = CreateAttachment::bytes(img, "last-mierda.png");
    msg
      .channel_id
      .send_message(
        ctx,
        CreateMessage::new()
          .reference_message(msg)
          .embed(
            CreateEmbed::new()
              .title(data.i18n.t(locale, "events.last-mierda.title"))
              .description(data.i18n.f(
                locale,
                key,
                &[("days", &time_since_last)],
              ))
              .image("attachment://last-mierda.png"),
          )
          .add_file(att),
      )
      .await
      .ok();
  } else {
    // Else we just complain about Mike's inability not to last a day without
    // talking about it.
    msg
      .reply(
        ctx,
        data.i18n.t(locale, "events.last-mierda.replies.not-even-a-day"),
      )
      .await
      .ok();
  }
}

fn regex_culo() -> &'static Regex {
  static REGEX: OnceLock<Regex> = OnceLock::new();

  REGEX.get_or_init(|| Regex::new(r"(?i)\bculo\b").unwrap())
}

fn generate_last_img(days: i64) -> anyhow::Result<Vec<u8>> {
  const TEMPLATE: &[u8] = include_bytes!("../../assets/img/last_mierda.png");
  const FONT: &[u8] = include_bytes!("../../assets/font/arial.ttf");

  const TEXT_CENTER_X: i32 = 107;
  const TEXT_CENTER_Y: i32 = 127;
  const FONT_SIZE: f32 = 100.0;

  let mut img = image::load_from_memory(TEMPLATE)?.to_rgba8();
  let font = FontArc::try_from_slice(FONT)?;
  let text = days.to_string();

  let (w, h) = text_size(FONT_SIZE, &font, &text);

  draw_text_mut(
    &mut img,
    Rgba([0, 0, 0, 255]),
    TEXT_CENTER_X - (w as i32 / 2),
    TEXT_CENTER_Y - (h as i32 / 2),
    FONT_SIZE,
    &font,
    &text,
  );

  let mut out = std::io::Cursor::new(Vec::new());

  image::DynamicImage::ImageRgba8(img).write_to(&mut out, ImageFormat::Png)?;

  Ok(out.into_inner())
}
