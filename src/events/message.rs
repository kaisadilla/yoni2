use crate::gif::{Gif, GifPHash, GifStatus};
use crate::store::Store;
use crate::util::{Data, GuildChannelExt, Hash, MessageExt};
use ab_glyph::FontArc;
use image::{ImageFormat, Rgba};
use imageproc::drawing::{draw_text_mut, text_size};
use poise::serenity_prelude::{
  Context, CreateAttachment, CreateEmbed, CreateMessage, GuildChannel, GuildId,
  Message,
};
use regex::Regex;
use std::sync::OnceLock;
use tracing::{debug, error, info, warn};

pub async fn message(data: &Data, ctx: &Context, message: &Message) {
  check_mike_mierda(data, ctx, message).await;
  check_thread_msg(data, ctx, message).await;
  check_gif(data, ctx, message).await;
}

/// If the message given comes from USER_MIKE_ID and talks about mierda,
/// updates their mierda timestamp and sends them a message congratulating them
/// for their shittiness.
async fn check_mike_mierda(data: &Data, ctx: &Context, msg: &Message) {
  if msg.author.id != data.config.user_mike_id {
    return;
  }

  debug!("Checking potential mierda message.");

  const KW: [&str; 7] =
    ["cag", "mierd", "diarre", "defeca", "hece", "hez", "caca"];

  let Some(guild_id) = msg.guild_id else {
    return;
  };
  let content = msg.content.to_lowercase();

  if msg.content.contains("yoni record mierda")
    || msg.content.contains("yoni récord mierda")
  {
    return;
  }

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
      tokio::task::spawn_blocking(move || generate_last_img(time_since_last))
        .await
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

/// Returns the regex for the word "culo".
fn regex_culo() -> &'static Regex {
  static REGEX: OnceLock<Regex> = OnceLock::new();

  REGEX.get_or_init(|| Regex::new(r"(?i)\bculo\b").unwrap())
}

/// Generates an image of The Simpsons replacing the number in a poster for
/// "days without mierda".
fn generate_last_img(days: i64) -> anyhow::Result<Vec<u8>> {
  const TEMPLATE: &[u8] = include_bytes!("../../assets/img/last_mierda.png");
  const FONT: &[u8] = include_bytes!("../../assets/font/arial.ttf");

  const TEXT_CENTER_X: i32 = 113;
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

async fn check_thread_msg(data: &Data, ctx: &Context, msg: &Message) {
  let Some(guild) = msg.guild_id else {
    return;
  };

  let Some(chan) =
    msg.channel_id.to_channel(&ctx).await.ok().and_then(|c| c.guild())
  else {
    return;
  };

  if chan.is_thread() == false {
    return;
  }

  if check_usr_is_blocked(data, ctx, msg, guild, chan).await {
    return;
  }
}

/// Checks whether the user that wrote the comment is blocked in the thread
/// given, and deletes their comment if so. Returns true if the message was
/// deleted
async fn check_usr_is_blocked(
  data: &Data,
  ctx: &Context,
  msg: &Message,
  guild: GuildId,
  chan: GuildChannel,
) -> bool {
  if data.store.is_blocked_in_thread(guild, chan.id, msg.author.id).await.ok()
    == Some(false)
  {
    return false;
  }

  debug!(
    usr_id = %msg.author.id,
    usr_name = %msg.author.name,
    thread_id = %chan.id,
    thread_name = %chan.name,
    msg_id = %msg.id,
    msg_content = %msg.content,
    "Message from blocked user deleted."
  );

  msg.delete(&ctx).await.ok();

  true
}

async fn check_gif(data: &Data, ctx: &Context, msg: &Message) {
  let Some(guild) = msg.guild_id else {
    return;
  };

  let gifs = msg.gif_urls();
  if gifs.is_empty() {
    return;
  }

  for gif in gifs {
    let status = match register_gif(&data.store, guild, gif).await {
      Ok(status) => status,
      Err(err) => {
        error!(%err, url = %gif, "Error while trying to register GIF.");
        continue;
      },
    };

    if matches!(status, GifStatus::Banned) {
      info!(gif_url = %gif, "Deleted message with banned GIF.");
      msg.delete(&ctx).await.ok();

      break;
    }
  }
}

async fn register_gif(
  store: &Store,
  guild: GuildId,
  url: &str,
) -> anyhow::Result<GifStatus> {
  match store.get_gif_status_by_url(guild, url).await? {
    Some(status) => Ok(status),
    None => {
      store.add_yoni_gif(guild, url, GifStatus::Allowed).await?;
      debug!(%url, "Registered new GIF for Yoni.");

      Ok(GifStatus::Allowed)
    },
  }
}

/// Registers this GIF in the store, if it isn't there already. Returns its
/// status.
async fn _register_gif_complex(
  store: &Store,
  client: &reqwest::Client,
  guild: GuildId,
  url: &str,
) -> anyhow::Result<GifStatus> {
  todo!();
  let mut exact_hash: Option<Hash> = None;
  let mut visual_hash: Option<Hash> = None;
  let mut similarity_hash: Option<GifPHash> = None;

  let status = match store.get_gif_status_by_url(guild, url).await? {
    // The GIF url is in the store.
    Some(status) => Some(status),
    // New GIF, we try to identify it by its static hash.
    None => {
      debug!("Checking GIF by static hash.");

      let gif = Gif::download(client, url).await?;

      exact_hash = Some(gif.exact_hash());
      let status =
        store.get_gif_status_by_exact_hash(guild, exact_hash.unwrap()).await?;

      match status {
        // The GIF has been found in the store by its static hash.
        Some(status) => Some(status),
        // New GIF, we try to identify it by its visual hash.
        None => {
          debug!("Checking GIF by visual hash.");

          visual_hash = Some(gif.visual_hash()?);
          let status = store
            .get_gif_status_by_visual_hash(guild, visual_hash.unwrap())
            .await?;

          match status {
            // The GIF has been found in the store by its visual hash.
            Some(status) => Some(status),
            // New Gif, we try to identify it by its similarity hash.
            None => {
              debug!("Checking GIF by similarity hash.");

              similarity_hash = Some(gif.similarity_hash()?);

              None // TODO: Implement this.
            },
          }
        },
      }
    },
  };

  Ok(match status {
    Some(status) => status,
    None => {
      debug!(%url, "Registering new GIF for Yoni.");

      store
        .add_yoni_gif(
          guild,
          url,
          //exact_hash.unwrap(),
          //visual_hash.unwrap(),
          //similarity_hash.unwrap(),
          GifStatus::Allowed,
        )
        .await?;

      GifStatus::Allowed
    },
  })
}
