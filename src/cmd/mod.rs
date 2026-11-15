pub mod bless;
pub mod delete;
pub mod fuck;
pub mod help;
pub mod leaderboard;
pub mod record;
pub mod speak;
pub mod thread;
pub mod util;

use crate::cmd::util::{Data, Error};
use poise::Command;

pub fn build_command_array() -> Vec<Command<Data, Error>> {
  vec![
    bless::bless(),
    delete::delete(),
    delete::ctx_delete(),
    fuck::fuck(),
    help::help(),
    leaderboard::leaderboard(),
    record::record(),
    speak::speak(),
    thread::thread(),
  ]
}
