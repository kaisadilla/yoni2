use crate::util::Error;
use poise::serenity_prelude::{
  ButtonStyle, ComponentInteraction, Context, CreateActionRow, CreateButton,
  CreateInteractionResponse, CreateInteractionResponseMessage, Emoji,
  ReactionType,
};
use rand::seq::SliceRandom;
use std::time::Duration;

const GRID_ROWS: usize = 5;
const GRID_COLS: usize = 5;

const CELL_COUNT: usize = GRID_ROWS * GRID_COLS;
const PAIR_COUNT: usize = CELL_COUNT / 2;
const EMOJI_COUNT: usize = (CELL_COUNT + 1) / 2;

const HIDDEN_EMOJI: &str = "❔";

const GENERIC_EMOJIS: &[&str] = &[
  "😀", "😎", "🤖", "👻", "🐱", "🐶", "🦊", "🐸", "🐵", "🐼", "🦁", "🐯", "🐮",
  "🐷", "🐙", "🦄", "🍎", "🍉", "🍇", "🍓", "🍒", "🍍", "🥑", "🌈", "⭐", "🔥",
  "⚡", "🎈", "🎲", "🎸", "🚀", "🌙",
];

const _: () = assert!(GRID_ROWS > 0 && GRID_COLS > 0);
const _: () = assert!(GRID_ROWS <= 5 && GRID_COLS <= 5);
const _: () = assert!(GENERIC_EMOJIS.len() >= EMOJI_COUNT);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurnResult {
  /// It's not the player's turn.
  NotYourTurn,
  /// The card doesn't exist.
  InvalidCard,
  /// This is the player's first choice in their turn.
  FirstCard,
  /// The player matched its first card.
  Match {
    /// The game has finished.
    finished: bool,
  },
  /// The player's second card didn't match its first.
  Mismatch,
}

pub struct Pairs {
  /// An array of indices in [`Self::emojis`].
  cards: Vec<usize>,
  /// The emojis used in this game of Pairs.
  emojis: Vec<ReactionType>,

  matched: Vec<bool>,
  revealed: Vec<bool>,
  resolved: [Vec<usize>; 2],

  scores: [usize; 2],
  turn: usize,

  first_pick: Option<usize>,
  pending_mismatch: Option<(usize, usize)>,
}

impl Pairs {
  pub fn new(mut server_emojis: Vec<Emoji>) -> Self {
    let emojis = Self::generate_emoji_set(&mut server_emojis);

    let mut cards: Vec<usize> = (0..PAIR_COUNT).flat_map(|i| [i, i]).collect();

    if CELL_COUNT % 2 != 0 {
      cards.push(PAIR_COUNT);
    }

    cards.shuffle(&mut rand::rng());

    Self {
      cards,
      emojis,
      matched: vec![false; CELL_COUNT],
      revealed: vec![false; CELL_COUNT],
      resolved: [Vec::new(), Vec::new()],
      scores: [0, 0],
      turn: 0,
      first_pick: None,
      pending_mismatch: None,
    }
  }

  pub fn play(&mut self, player: usize, idx: usize) -> TurnResult {
    if player != self.turn {
      return TurnResult::NotYourTurn;
    }

    if self.is_finished()
      || self.pending_mismatch.is_some()
      || idx >= CELL_COUNT
      || self.matched[idx]
      || self.revealed[idx]
    {
      return TurnResult::InvalidCard;
    }

    self.revealed[idx] = true;

    let Some(first) = self.first_pick.take() else {
      self.first_pick = Some(idx);
      return TurnResult::FirstCard;
    };

    if self.cards[first] == self.cards[idx] {
      self.matched[first] = true;
      self.matched[idx] = true;

      self.revealed[first] = false;
      self.revealed[idx] = false;

      self.resolved[self.turn].push(self.cards[first]);

      self.scores[self.turn] += 1;

      TurnResult::Match { finished: self.is_finished() }
    } else {
      self.pending_mismatch = Some((first, idx));
      TurnResult::Mismatch
    }
  }

  /// Hides mismatched cards and gives he turn to the other player. Returns
  /// false if there is no pending mismatch.
  pub fn finish_mismatch(&mut self) -> bool {
    let Some((first, second)) = self.pending_mismatch.take() else {
      return false;
    };

    self.revealed[first] = false;
    self.revealed[second] = false;

    self.turn = 1 - self.turn;

    true
  }

  /// Returns the index of the player whose turn it is.
  pub fn active_player_idx(&self) -> usize {
    self.turn
  }

  pub fn scores(&self) -> [usize; 2] {
    self.scores
  }

  pub fn is_finished(&self) -> bool {
    self.scores.iter().sum::<usize>() == PAIR_COUNT
  }

  /// Generates the Discord button grid.
  pub fn build_components(&self, lock_all: bool) -> Vec<CreateActionRow> {
    (0..GRID_ROWS)
      .map(|row| {
        let buttons = (0..GRID_COLS)
          .map(|col| {
            let idx = row * GRID_COLS + col;

            let is_unpaired =
              (CELL_COUNT % 2 != 0) && self.cards[idx] == PAIR_COUNT;

            let visible = self.matched[idx]
              || self.revealed[idx]
              || (self.is_finished() && is_unpaired);

            let emoji = if visible {
              self.emojis[self.cards[idx]].clone()
            } else {
              ReactionType::Unicode(HIDDEN_EMOJI.to_owned())
            };

            let disabled = lock_all
              || self.matched[idx]
              || self.revealed[idx]
              || (self.is_finished() && is_unpaired);

            CreateButton::new(format!("pairs:{idx}"))
              .style(ButtonStyle::Secondary)
              .emoji(emoji)
              .disabled(lock_all || disabled)
          })
          .collect();

        CreateActionRow::Buttons(buttons)
      })
      .collect()
  }

  pub fn resolved_emojis(&self) -> [String; 2] {
    std::array::from_fn(|player| {
      self.resolved[player]
        .iter()
        .map(|&idx| self.emojis[idx].to_string())
        .collect::<Vec<_>>()
        .join(" ")
    })
  }

  /// Given an array of server emoji, generates a set of emoji valid for a game
  /// of Pairs.
  fn generate_emoji_set(server_emojis: &mut Vec<Emoji>) -> Vec<ReactionType> {
    let mut rng = rand::rng();

    // Keep available emojis only.
    server_emojis.retain(|e| e.available && e.roles.is_empty());
    server_emojis.shuffle(&mut rng);

    let mut emojis: Vec<ReactionType> = server_emojis
      .iter()
      .take(EMOJI_COUNT)
      .cloned()
      .map(ReactionType::from)
      .collect();

    let missing = EMOJI_COUNT - emojis.len();

    let mut generic = GENERIC_EMOJIS.to_vec();
    generic.shuffle(&mut rng);

    emojis.extend(
      generic
        .into_iter()
        .take(missing)
        .map(|e| ReactionType::Unicode(e.to_owned())),
    );

    emojis
  }
}
