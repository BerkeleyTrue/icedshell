mod app;
mod layershell;
mod theme;

use clap::Parser;
use crate::layershell::start;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Cli {
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// add quit keybinds for testing
    #[arg(short, long)]
    quit_keybindings: bool,
}

fn main() -> iced_layershell::Result {
    let args = Cli::parse();

    start(args.into())
}
