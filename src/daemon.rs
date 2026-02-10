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

use crate::{
    Cli,
    delora::{self, DeloraMain},
    feature::{Comp, Feature, Service, Window},
    niri::{self, monitors::MonitorsServ},
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
    NiriMon(niri::monitors::Message),
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
    mon_serv: niri::monitors::MonitorsServ,
    delora_main: Option<Window<DeloraMain>>,
    quit_keybinds: bool,
}

impl Daemon {
    fn new(init: Init) -> Self {
        Self {
            mon_serv: MonitorsServ::new(()),
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

        let niri_mon = self.mon_serv.subscription().map(Message::NiriMon);

        Subscription::batch(vec![quit_binds, delora_subs, niri_mon])
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Delora(message) => self
                .delora_main
                .as_mut()
                .map(|delora| delora.update(message).map(Message::Delora))
                .unwrap_or(Task::none()),

            Message::NiriMon(message) => {
                let inner_task = self.mon_serv.update(message).map(Message::NiriMon);
                let num_mon = self.mon_serv.len();
                let mon_names: Vec<String> =
                    self.mon_serv.iter().map(|mon| mon.0.clone()).collect();

                let mut tasks: Vec<Task<Message>> = mon_names
                    .iter()
                    .map(move |mon| match (num_mon, mon.as_ref()) {
                        (2, "HDMI-A-1") => self.open_delora_main("HDMI-A-1".into()),
                        (1, "DP-3") => self.open_delora_main("DP-3".into()),
                        (_, _) => Task::none(),
                    })
                    .collect();

                tasks.push(inner_task);

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
        let (mut main_bar, main_layer_settings) =
            DeloraMain::new(delora::Init { output_name }).open();
        let main_id = main_bar.id;

        let mut remove = Task::none();
        if let Some(old_win) = &self.delora_main {
            main_bar.view.clone_niri_serv(&old_win.view);
            remove = Task::done(Message::RemoveWindow(old_win.id));
        };

        self.delora_main.replace(main_bar);

        Task::batch([
            remove,
            Task::done(Message::NewLayerShell {
                settings: main_layer_settings,
                id: main_id,
            }),
        ])
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
