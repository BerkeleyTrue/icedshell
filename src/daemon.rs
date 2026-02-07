use iced::{
    Color, Element, Subscription, Task,
    keyboard::{self, Key, key::Named},
    theme::Style,
    widget::{container, space},
    window::Id,
};
use iced_layershell::{
    reexport::KeyboardInteractivity,
    settings::{LayerShellSettings, StartMode},
    to_layer_message,
};
use itertools::Itertools;
use tracing::debug;

use crate::{
    Cli,
    config::MonitorId,
    delora::{self, DeloraMain},
    feature::{Comp, Feature, Window},
    niri,
    theme::{self as mytheme},
};

#[derive(Clone)]
enum Hosts {
    Delora,
    Rena,
}

#[to_layer_message(multi)]
#[derive(Debug)]
pub enum Message {
    Delora(delora::Message),
    UpdateMonitors(Vec<MonitorId>),
    Quit,
}

#[derive(Clone)]
pub struct Init {
    quit_keybinds: bool,
    host: Hosts,
}

impl Init {
    pub fn host(&mut self, host: &str) {
        self.host = match host {
            "delora" => Hosts::Delora,
            "rena" => Hosts::Rena,
            _ => Hosts::Delora,
        };
    }
}

impl From<Cli> for Init {
    fn from(cli: Cli) -> Self {
        Self {
            quit_keybinds: cli.quit_keybindings,
            host: Hosts::Delora,
        }
    }
}

struct Daemon {
    delora_main: Option<Window<DeloraMain>>,
    quit_keybinds: bool,
}

impl Daemon {
    fn new(init: Init) -> Self {
        Self {
            delora_main: None,
            quit_keybinds: init.quit_keybinds,
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        let quit_binds = keyboard::listen()
            .with(self.quit_keybinds)
            .filter_map(|(quit_keybinds, event)| match (quit_keybinds, event) {
                (false, keyboard::Event::KeyPressed { key, .. }) => Some(key),
                _ => None,
            })
            .filter_map(|key| match key.as_ref() {
                Key::Named(Named::Escape) | Key::Character("q") => Some(Message::Quit),
                _ => None,
            });

        let delora_subs = self
            .delora_main
            .as_ref()
            .map(|delora| delora.subscription().map(Message::Delora))
            .unwrap_or(Subscription::none());

        let niri_sub = Subscription::run(niri::stream::listen)
            .filter_map(|res| res.ok())
            .filter_map(|event| match event {
                niri::stream::NiriEvent::WorkspacesChanged { workspaces } => {
                    let outputs: Vec<MonitorId> = workspaces
                        .iter()
                        .filter_map(|ws| ws.output.clone())
                        .map(MonitorId::from)
                        .unique()
                        .collect();

                    Some(Message::UpdateMonitors(outputs))
                }
                _ => None,
            });

        Subscription::batch(vec![quit_binds, delora_subs, niri_sub])
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Delora(message) => self
                .delora_main
                .as_mut()
                .map(|delora| delora.update(message).map(Message::Delora))
                .unwrap_or(Task::none()),

            Message::UpdateMonitors(monitors) => {
                let num_mon = monitors.len();
                debug!("monitors {0:?}", monitors);
                let tasks: Vec<Task<Message>> = monitors
                    .iter()
                    .map(move |mon| match (num_mon, mon.0.as_str()) {
                        (2, "HDMI-A-1") => self.open_delora_main("HDMI-A-1".into()),
                        (1, "DP-3") => self.open_delora_main("DP-3".into()),
                        (_, _) => Task::none(),
                    })
                    .collect();

                Task::batch(tasks)
            }
            _ => Task::none(),
        }
    }

    fn view(&self, win_id: Id) -> Element<'_, Message> {
        if let Some(delora) = self.delora_main.as_ref()
            && delora.id == win_id
        {
            delora.view().map(Message::Delora)
        } else {
            container(space()).into()
        }
    }
}

impl Daemon {
    fn open_delora_main(&mut self, output_name: String) -> Task<Message> {
        let (main_bar, main_layer_settings) = DeloraMain::new(delora::Init { output_name }).open();
        let main_id = main_bar.id;

        self.delora_main.replace(main_bar);

        Task::done(Message::NewLayerShell {
            settings: main_layer_settings,
            id: main_id,
        })
    }
}

pub fn start(init: Init, settings: iced_layershell::Settings) -> iced_layershell::Result {
    let theme = mytheme::app_theme();
    iced_layershell::daemon(
        move || Daemon::new(init.clone()),
        || "Icedshell".to_string(),
        Daemon::update,
        Daemon::view,
    )
    .theme(theme.theme())
    .subscription(Daemon::subscription)
    .style(|_layer, theme| Style {
        background_color: Color::TRANSPARENT,
        text_color: theme.palette().text,
    })
    .settings(settings)
    .layer_settings(LayerShellSettings {
        keyboard_interactivity: KeyboardInteractivity::None,
        start_mode: StartMode::Background,
        ..Default::default()
    })
    .run()
}
