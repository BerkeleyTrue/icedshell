mod clock;
mod config;
mod daemon;
mod delora;
mod divider;
mod feature;
mod fira_fonts;
mod niri;
mod theme;

use crate::daemon::{Init, start};
use clap::Parser;
use iced_layershell::Settings;
use lucide_icons::LUCIDE_FONT_BYTES;
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
        .unwrap_or(String::from("NA"));

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

    start(
        init,
        Settings {
            id: Some("Icedshell".into()),
            fonts: vec![
                LUCIDE_FONT_BYTES.into(),
                fira_fonts::BOLD_BYTES.into(),
                fira_fonts::REGULAR_BYTES.into(),
            ],
            default_font: fira_fonts::FIRA_CODE,
            ..Default::default()
        },
    )
}
