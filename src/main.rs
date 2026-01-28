mod app;
mod layershell;
mod niri;
mod theme;

use crate::layershell::{Init, start};
use clap::Parser;
use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;

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
    let host = hostname::get()
        .map(|os| os.to_string_lossy().to_string())
        .ok()
        .or(Some(String::from("NA")))
        .unwrap();

    // initialize tracing
    let log_level = match args.verbose {
        0 => Level::ERROR,
        1 => Level::INFO,
        2 => Level::DEBUG,
        _ => Level::TRACE,
    };
    let subscriber = FmtSubscriber::builder().with_max_level(log_level).finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to setup tracing");

    info!("host: {host:?}");
    let mut init = Init::from(args);
    init.host(&host);
    start(init)
}
