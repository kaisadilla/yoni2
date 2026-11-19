pub mod bless;
pub mod checks;
pub mod delete;
pub mod fuck;
pub mod games;
pub mod gif;
pub mod gifs;
pub mod help;
pub mod insult;
pub mod leaderboard;
pub mod news;
pub mod record;
pub mod say;
pub mod speak;
pub mod thread;
pub mod util;

use crate::util::{Data, Error};
use poise::Command;

pub fn build_command_array() -> Vec<Command<Data, Error>> {
  vec![
    bless::bless(),
    delete::delete(),
    delete::ctx_delete(),
    fuck::fuck(),
    games::games(),
    gif::gif(),
    gifs::gifs(),
    gifs::ctx_remove(),
    gifs::ctx_forbid(),
    help::help(),
    insult::insult(),
    news::news(),
    leaderboard::leaderboard(),
    record::record(),
    say::say(),
    speak::speak(),
    thread::thread(),
  ]
}
