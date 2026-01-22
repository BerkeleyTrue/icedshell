mod app;
mod layershell;
mod theme;
mod niri;

use clap::Parser;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;
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

    // initialize tracing
    let log_level = match args.verbose {
        0 => Level::ERROR,
        1 => Level::INFO,
        2 => Level::DEBUG,
        _ => Level::TRACE,
    };
    let subscriber = FmtSubscriber::builder().with_max_level(log_level).finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to setup tracing");

    start(args.into())
}
