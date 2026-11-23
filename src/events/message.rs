use crate::gif::{Gif, GifPHash, GifStatus};
use crate::store::Store;
use crate::util::{
  Data, GuildChannelExt, Hash, MessageExt, StrExt, VecExt, strip_diacritics,
};
use ab_glyph::FontArc;
use image::{ImageFormat, Rgba};
use imageproc::drawing::{draw_text_mut, text_size};
use poise::serenity_prelude::{
  Context, CreateAttachment, CreateEmbed, CreateMessage, GuildChannel, GuildId,
  Message,
};
use rand::RngExt;
use rand::seq::IteratorRandom;
use regex::{Regex, regex};
use std::sync::{LazyLock, OnceLock};
use tracing::{debug, error, info, warn};

pub async fn message(data: &Data, ctx: &Context, message: &Message) {
  check_mike_mierda(data, ctx, message).await;
  check_thread_msg(data, ctx, message).await;
  check_gif(data, ctx, message).await;
  check_yoni_mention(data, ctx, message).await;
  check_hola(data, ctx, message).await;
  check_yoni_channel(data, ctx, message).await;
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
  let has_kw = KW.iter().any(|&kw| {
    content.contains(kw) || regex!(r"(?i)\bculo\b").is_match(&content)
  });

  if has_kw == false {
    return;
  }

  let l = data.i18n.default_locale();

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
              .title(l.t("events.last-mierda.title"))
              .description(l.fmt(key, &[("days", &time_since_last)]))
              .image("attachment://last-mierda.png"),
          )
          .add_file(att),
      )
      .await
      .ok();
  } else {
    // Else we just complain about Mike's inability not to last a day without
    // talking about it.
    msg.reply(ctx, l.t("events.last-mierda.replies.not-even-a-day")).await.ok();
  }
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
    "Deleting message from blocked user..."
  );

  match msg.delete(&ctx).await {
    Ok(_) => true,
    Err(error) => {
      error!(%error, "Failed to delete message.");
      false
    },
  }
}

/// Acts when the message features GIFs.
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
/// WARNING: This method is not currently fully functional and is boxed for now.
async fn _register_gif_smart(
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

/// Acts when Yoni is mentioned in a message.
async fn check_yoni_mention(data: &Data, ctx: &Context, msg: &Message) {
  // Yoni doesn't check mentions in his channels.
  if msg.channel_id == data.config.channel_yoni_id
    || msg.channel_id == data.config.channel_oratorio_id
  {
    return;
  }

  let l = data.i18n.default_locale();
  let content = msg.content.to_lowercase();

  if content.starts_with("yoni ") {
    return;
  }

  if regex!(r"y+o+n+i+").is_match(&content) == false {
    return;
  }

  if Some(msg.author.id) == data.config.user_meimeigpt_id {
    msg.reply(&ctx.http, l.t("replies.yoni-mention.meimeigpt")).await.ok();
  } else {
    let usrname = msg
      .member
      .as_deref()
      .and_then(|m| m.nick.as_deref())
      .or(msg.author.global_name.as_deref())
      .unwrap_or(&msg.author.name)
      .escape_markdown();

    msg
      .reply(
        &ctx.http,
        l.fmt("replies.yoni-mention.default", &[("user_name", &usrname)]),
      )
      .await
      .ok();
  }
}

async fn check_hola(data: &Data, ctx: &Context, msg: &Message) {
  let Some(chance) = data.config.no_digas_hola_chance else {
    return;
  };
  let Some(sticker_id) = data.config.sticker_no_digas_hola_id else {
    warn!("Missing config variable: STICKER_NO_DIGAS_HOLA_ID.");
    return;
  };

  if rand::rng().random_bool(chance) == false {
    return;
  }

  let content = msg.content.to_lowercase();

  if content.contains("hola") == false {
    return;
  }

  msg
    .channel_id
    .send_message(
      ctx,
      CreateMessage::new().reference_message(msg).sticker_id(sticker_id),
    )
    .await
    .ok();
}

async fn check_yoni_channel(data: &Data, ctx: &Context, msg: &Message) {
  // Ignore the message if the channel is not Yoni or Oratorio.
  if msg.channel_id != data.config.channel_yoni_id
    && msg.channel_id != data.config.channel_oratorio_id
  {
    return;
  }

  // Ignore the message if it comes from a bot other than MeiMei GPT.
  if msg.author.bot && Some(msg.author.id) != data.config.user_meimeigpt_id {
    return;
  }

  // This message is a reply to a user that isn't Yoni.
  if msg
    .referenced_message
    .as_deref()
    .is_none_or(|r| r.author.id == data.config.user_yoni_id)
    == false
  {
    // There's a 1 in 50 chance Yoni will tell the author to get the f out.
    if rand::rng().random_bool(0.02)
      && let Some(response) = data.conversation_model.interferences.random()
    {
      msg.reply(ctx, response).await.ok();
    }

    return;
  }

  let content = strip_diacritics(&msg.content.to_lowercase());
  let mut message_sent = false;

  // Short messages.
  for res in &data.conversation_model.exact_responses {
    if res.pattern.iter().any(|p| p.eq(&content)) {
      reply_multiple(ctx, msg, &res.response).await;
      message_sent = true;
    }
  }

  // Regexes
  if regex!(r"\A(?:ja[jas]+|aj[jas]+|pua[jas]+|js[jas]+|xd+|lol+)\z")
    .is_match(&content)
  {
    if let Some(v) = data.conversation_model.regexes.haha.random() {
      msg.reply(ctx, v).await.ok();
      message_sent = true;
    }
  }

  if regex!(r"\An+o+o\z").is_match(&content) {
    if let Some(v) = data.conversation_model.regexes.noo.random() {
      reply_multiple(ctx, msg, &v).await;
      message_sent = true;
    }
  }

  // Reply gifs with gifs.
  if msg.gif_urls().is_empty() == false
    && let Some(guild_id) = msg.guild_id
    && let Ok(Some(gif_id)) = data.store.get_random_yoni_gif(guild_id).await
  {
    msg.reply(ctx, gif_id).await.ok();
    message_sent = true;
  }

  // If we've already sent a message, we won't analyze or sent anything more.
  if message_sent {
    return;
  }

  // Partial responses.
  let mut fragments: Vec<&str> = vec![];
  let mut sentences: Vec<&str> = vec![];

  for res in &data.conversation_model.partial_responses {
    for p in &res.pattern {
      let pattern = if res.exact {
        format!(r"\b{}\b", regex::escape(p))
      } else {
        p.clone()
      };

      let re = match Regex::new(&pattern) {
        Ok(re) => re,
        Err(err) => {
          warn!(%err, %pattern, "Invalid regex.");
          continue;
        },
      };

      if re.is_match(&content) == false {
        continue;
      }

      if let Some(response) = res.response.get(msg.author.id) {
        if res.fragment {
          fragments.push(response);
        } else {
          sentences.push(response);
        }
      }
    }
  }

  // If no message is sent
  if fragments.is_empty() == false || sentences.is_empty() == false {
    let mut final_reply = sentences.join(" ");
    let fragments = join_fragments(&fragments);

    if fragments.is_empty() == false {
      final_reply.push(' ');
      final_reply.push_str(&fragments);
      final_reply.push('.');
    };

    if final_reply.is_empty() == false {
      if let Ok(_) = msg.reply(ctx, final_reply).await {
        message_sent = true;
      }
    }
  }

  if message_sent {
    return;
  }
  if msg.channel_id == data.config.channel_oratorio_id {
    return;
  }

  if let Some(fallback) = data.conversation_model.fallbacks.random() {
    msg.reply(ctx, fallback).await.ok();
  }
}

/// Sends all the messages given - the first as a reply and the rest as normal
/// messages immediately after.
async fn reply_multiple(ctx: &Context, msg: &Message, replies: &[String]) {
  for (i, val) in replies.iter().enumerate() {
    if i == 0 {
      msg.reply(ctx, val).await.ok();
    } else {
      msg
        .channel_id
        .send_message(ctx, CreateMessage::new().content(val))
        .await
        .ok();
    }
  }
}

fn join_fragments(fragments: &[&str]) -> String {
  match fragments.split_last() {
    None => String::new(),
    Some((last, [])) => (*last).to_owned(),
    Some((last, rest)) => format!("{} y {}", rest.join(", "), last),
  }
}
