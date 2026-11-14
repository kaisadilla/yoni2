mod bless;
mod fuck;
mod help;
mod leaderboard;
mod speak;
pub mod threads;
pub mod util;

use crate::cmd::util::{Data, Error};
use poise::Command;

pub fn build_command_array() -> Vec<Command<Data, Error>> {
  vec![
    bless::bless(),
    fuck::fuck(),
    help::help(),
    leaderboard::leaderboard(),
    speak::speak(),
    threads::threads(),
  ]
}
