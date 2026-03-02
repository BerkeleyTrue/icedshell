mod config;
mod daemon;
mod datetime;
mod delora;
mod divider;
mod fdo_icons;
mod feature;
mod fira_fonts;
mod launcher;
mod niri;
mod system_info;
mod theme;
mod tray;
mod widget_ext;

use crate::daemon::{Init, start};
use clap::{Parser, Subcommand};
use derive_more::Display;
use iced_layershell::Settings;
use lucide_icons::LUCIDE_FONT_BYTES;
use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Cli {
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
    #[command(subcommand)]
    command: Option<AppCommand>,
}

#[derive(Subcommand, Debug, Display, Clone)]
enum AppCommand {
    Daemon {
        /// add quit keybinds for testing
        #[arg(short, long)]
        quit_keybindings: bool,
    },
    Launcher,
    // PowerMenu,
    // Notify
}

fn main() -> anyhow::Result<()> {
    let cli_args = Cli::parse();
    let host = hostname::get()
        .map(|os| os.to_string_lossy().to_string())
        .ok()
        .unwrap_or(String::from("NA"));

    // initialize tracing
    let log_level = match cli_args.verbose {
        0 => Level::ERROR,
        1 => Level::INFO,
        2 => Level::DEBUG,
        _ => Level::TRACE,
    };
    let subscriber = FmtSubscriber::builder().with_max_level(log_level).finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to setup tracing");

    info!("host: {host:?}");
    if cli_args.command.is_none() {
        return Ok(());
    }

    match cli_args.command.clone().unwrap() {
        AppCommand::Daemon {
            quit_keybindings: _,
        } => {
            let mut init = Init::from(cli_args);
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
            )?;
            Ok(())
        }
        AppCommand::Launcher => {
            info!("App Launcher");
            Ok(())
        }
    }
}
