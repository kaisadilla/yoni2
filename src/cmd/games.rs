use crate::cmd::util::{CmdContext, CmdContextExt};
use crate::games::pairs::{Pairs, TurnResult};
use crate::i18n::Locale;
use crate::util::{Error, InteractionExt};
use futures_util::StreamExt;
use poise::CreateReply;
use poise::serenity_prelude::{
  ComponentInteraction, Context, CreateInteractionResponse,
  CreateInteractionResponseMessage, EditMessage, Member, Mentionable, Message,
  ReactionType, User,
};
use rand::seq::IndexedRandom;
use std::cmp::Ordering;
use std::time::Duration;

const GAME_TIMEOUT: Duration = Duration::from_secs(15 * 60);
const MISMATCH_DELAY: Duration = Duration::from_millis(1_200);

const CURSORS: &[&str] = &[
  ":poop: ",
  ":face_in_clouds: ",
  ":ghost: ",
  ":ogre: ",
  ":clown: ",
  ":jack_o_lantern: ",
  ":brain: ",
  ":farmer: ",
  ":woman_with_veil: ",
  ":dolphin: ",
  ":cricket: ",
  ":lobster: ",
  ":shrimp: ",
  ":cat2: ",
  ":sunflower: ",
];

#[poise::command(
  slash_command,
  prefix_command,
  rename = "games",
  aliases("jugar"),
  subcommands("pairs")
)]
pub async fn games(_ctx: CmdContext<'_>) -> Result<(), Error> {
  Ok(())
}

#[poise::command(
  slash_command,
  prefix_command,
  guild_only,
  rename = "pairs",
  aliases("parejas")
)]
async fn pairs(ctx: CmdContext<'_>, opponent: Member) -> Result<(), Error> {
  let l = ctx.loc().await;

  let guild_id = ctx.guild_id().expect("guild_only command.");
  let author = ctx.author();
  let player_id = author.id;
  let opponent_id = opponent.user.id;

  // The author challenged themselves.
  if player_id == opponent_id {
    ctx.say(l.t("commands.games.pairs.replies.challenged-themselves")).await?;
    return Ok(());
  }

  // The author challenged a bot.
  if opponent.user.bot {
    ctx
      .say(l.fmt(
        "commands.games.pairs.replies.challenged-bot",
        &[("user_name", &opponent.display_name())],
      ))
      .await?;
    return Ok(());
  }

  let player_name = ctx
    .author_member()
    .await
    .map(|m| m.display_name().to_owned())
    .unwrap_or_else(|| ctx.author().name.clone());
  let opponent_name = opponent.display_name().to_owned();

  let names = [player_name, opponent_name];

  let reply = ctx
    .send(
      CreateReply::default()
        .content(l.t("commands.games.pairs.replies.info.loading")),
    )
    .await?;

  let server_emojis = guild_id.emojis(&ctx).await?;

  let mut game = Pairs::new(server_emojis);
  let mut cursor = CURSORS.choose(&mut rand::rng()).copied().unwrap();

  let content = message_content(
    &game,
    &l,
    &author,
    &opponent,
    &names,
    cursor,
    &l.fmt(
      "commands.games.pairs.replies.info.status.first-move",
      &[("user_name", &names[0])],
    ),
  );

  reply
    .edit(
      ctx,
      CreateReply::default()
        .content(content)
        .components(game.build_components(false)),
    )
    .await?;

  let mut msg = reply.into_message().await?;

  let mut interactions = msg
    .await_component_interaction(&ctx.serenity_context().shard)
    .timeout(GAME_TIMEOUT)
    .stream();

  let mut finished = false;

  while let Some(interaction) = interactions.next().await {
    let usr_id = interaction.user.id;

    let player = if usr_id == player_id {
      0
    } else if usr_id == opponent_id {
      1
    } else {
      interaction
        .ephemeral(&ctx, l.t("commands.games.pairs.replies.not-your-game"))
        .await?;
      continue;
    };

    let Some(idx) = interaction
      .data
      .custom_id
      .strip_prefix("pairs:")
      .and_then(|v| v.parse::<usize>().ok())
    else {
      interaction
        .ephemeral(&ctx, l.t("commands.games.pairs.replies.invalid-button"))
        .await
        .ok();
      continue;
    };

    let result = game.play(player, idx);

    match result {
      TurnResult::NotYourTurn => {
        interaction
          .ephemeral(&ctx, l.t("commands.games.pairs.replies.not-your-turn"))
          .await
          .ok();
      },

      TurnResult::InvalidCard => {
        interaction
          .ephemeral(&ctx, l.t("commands.games.pairs.replies.invalid-card"))
          .await
          .ok();
      },

      TurnResult::FirstCard => {
        let content = message_content(
          &game,
          &l,
          &author,
          &opponent,
          &names,
          cursor,
          l.t("commands.games.pairs.replies.info.status.second-card"),
        );

        update_board(&game, &interaction, &ctx, &mut msg, content, false)
          .await?;
      },

      TurnResult::Match { finished: false } => {
        let content = message_content(
          &game,
          &l,
          &author,
          &opponent,
          &names,
          cursor,
          &l.fmt(
            "commands.games.pairs.replies.info.status.match-extra-turn",
            &[("user_name", &names[game.active_player_idx()])],
          ),
        );
        update_board(&game, &interaction, &ctx, &mut msg, content, false)
          .await?;
      },

      TurnResult::Match { finished: true } => {
        let scores = game.scores();

        let res = match scores[0].cmp(&scores[1]) {
          Ordering::Greater => l.fmt(
            "commands.games.pairs.replies.info.winner",
            &[("winner_name", &names[0]), (("loser_name", &names[1]))],
          ),
          Ordering::Less => l.fmt(
            "commands.games.pairs.replies.info.winner",
            &[("winner_name", &names[1]), ("loser_name", &names[0])],
          ),
          Ordering::Equal => l.fmt(
            "commands.games.pairs.replies.info.tie",
            &[("player_name", &names[0]), ("opponent_name", &names[1])],
          ),
        };

        let resolved = game.resolved_emojis();

        let content = l.fmt(
          "commands.games.pairs.replies.info.game-over",
          &[
            ("player_mention", &ctx.author().mention()),
            ("opponent_mention", &opponent.mention()),
            ("player_points", &scores[0]),
            ("opponent_points", &scores[1]),
            ("player_emojis", &resolved[0]),
            ("opponent_emojis", &resolved[1]),
            ("result", &res),
          ],
        );

        update_board(&game, &interaction, &ctx, &mut msg, content, true)
          .await?;

        finished = true;
        break;
      },

      TurnResult::Mismatch => {
        let content = message_content(
          &game,
          &l,
          &author,
          &opponent,
          &names,
          cursor,
          &l.fmt(
            "commands.games.pairs.replies.info.status.no-match",
            &[("user_name", &names[game.active_player_idx()])],
          ),
        );

        update_board(&game, &interaction, &ctx, &mut msg, content, true)
          .await?;

        let delay = tokio::time::sleep(MISMATCH_DELAY);
        tokio::pin!(delay);

        loop {
          tokio::select! {
            _ = &mut delay => break,
            next = interactions.next() => {
              let Some(stale_interaction) = next else { break; };

              stale_interaction.ephemeral(
                &ctx,
                l.t("commands.games.pairs.replies.wait")
              ).await?;
            }
          }
        }

        game.finish_mismatch();

        cursor = CURSORS.choose(&mut rand::rng()).copied().unwrap();

        let content = message_content(
          &game,
          &l,
          &author,
          &opponent,
          &names,
          cursor,
          &l.fmt(
            "commands.games.pairs.replies.info.status.first-move",
            &[("user_name", &names[game.active_player_idx()])],
          ),
        );

        msg
          .edit(
            &ctx,
            EditMessage::new()
              .content(content)
              .components(game.build_components(false)),
          )
          .await?;
      },
    }
  }

  if finished == false {
    let scores = game.scores();
    let resolved = game.resolved_emojis();

    let content = l.fmt(
      "commands.games.pairs.replies.info.game-expired",
      &[
        ("player_mention", &ctx.author().mention()),
        ("opponent_mention", &opponent.mention()),
        ("player_points", &scores[0]),
        ("opponent_points", &scores[1]),
        ("player_emojis", &resolved[0]),
        ("opponent_emojis", &resolved[1]),
      ],
    );

    msg
      .edit(
        &ctx,
        EditMessage::new()
          .content(content)
          .components(game.build_components(true)),
      )
      .await?;
  }

  Ok(())
}

fn message_content(
  game: &Pairs,
  locale: &Locale,
  player: &User,
  opponent: &Member,
  player_names: &[String; 2],
  cursor: &str,
  status: &str,
) -> String {
  let scores = game.scores();
  let active = game.active_player_idx();
  let resolved = game.resolved_emojis();

  locale.fmt(
    "commands.games.pairs.replies.info.game",
    &[
      ("player_mention", &player.mention()),
      ("opponent_mention", &opponent.mention()),
      ("player_points", &scores[0]),
      ("opponent_points", &scores[1]),
      ("player_emojis", &resolved[0]),
      ("opponent_emojis", &resolved[1]),
      (
        "player_active_icon",
        &(if active == 0 { cursor } else { ":black_small_square: " }),
      ),
      (
        "opponent_active_icon",
        &(if active == 1 { cursor } else { ":black_small_square: " }),
      ),
      ("active_player_name", &player_names[active]),
      ("status", &status),
    ],
  )
}

async fn update_board(
  game: &Pairs,
  interaction: &ComponentInteraction,
  ctx: &CmdContext<'_>,
  msg: &mut Message,
  content: String,
  lock_all: bool,
) -> Result<(), Error> {
  interaction
    .create_response(
      ctx,
      CreateInteractionResponse::UpdateMessage(
        CreateInteractionResponseMessage::new()
          .content(content)
          .components(game.build_components(lock_all)),
      ),
    )
    .await?;

  Ok(())
}
