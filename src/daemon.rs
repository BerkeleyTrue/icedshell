use std::collections::HashMap;

use derive_more::{Deref, DerefMut};
use iced::{
    Color, Element, Point, Subscription, Task, exit,
    keyboard::{self, Key, key::Named},
    theme::Style,
    widget::{container, space},
    window::{self, Id},
};
use iced_layershell::{
    reexport::KeyboardInteractivity,
    settings::{LayerShellSettings, StartMode},
    to_layer_message,
};
use tracing::info;

use crate::{
    Cli,
    delora::{self, DeloraMain},
    feature::{Comp, FeatWindow, Feature, Service},
    niri::{self, monitors::MonitorsServ},
    theme::{self as mytheme},
    tray::{TrayLayout, menu_comp as tray_menu},
};

#[derive(Clone)]
enum Hosts {
    Delora,
    Rena,
}

#[to_layer_message(multi)]
#[derive(Debug, Clone)]
pub enum Message {
    NiriMon(niri::monitors::Message),

    Delora(window::Id, delora::Message),
    TrayMenu(window::Id, tray_menu::Message),

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

// TODO: reconsider when menu is more flush
#[allow(clippy::large_enum_variant)]
enum Feat {
    Delora(FeatWindow<DeloraMain>),
    TrayMenu(FeatWindow<tray_menu::MenuComp>),
}

#[derive(Deref, DerefMut)]
struct Features(HashMap<window::Id, Feat>);

struct Daemon {
    features: Features,
    mon_serv: niri::monitors::MonitorsServ,
    quit_keybinds: bool,
}

impl Daemon {
    fn new(init: Init) -> Self {
        Self {
            features: Features(HashMap::new()),
            mon_serv: MonitorsServ::new(()),
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

        let niri_mon = self.mon_serv.subscription().map(Message::NiriMon);
        let mut win_subs: Vec<_> = self
            .features
            .iter()
            .map(|(win_id, feat)| {
                let win_id = *win_id;
                match feat {
                    Feat::Delora(delora) => delora
                        .subscription()
                        .with(win_id)
                        .map(|(win_id, m)| Message::Delora(win_id, m)),
                    Feat::TrayMenu(menu) => menu
                        .subscription()
                        .with(win_id)
                        .map(|(win_id, m)| Message::TrayMenu(win_id, m)),
                }
            })
            .collect();

        let mut subs = vec![quit_binds, niri_mon];
        subs.append(&mut win_subs);
        Subscription::batch(subs)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Delora(win_id, message) => {
                if let Some(Feat::Delora(delora)) = self.features.get_mut(&win_id) {
                    let task = delora
                        .update(message.clone())
                        .map(move |m| Message::Delora(win_id, m));

                    let open_task = match message {
                        delora::Message::OpenTrayMenu(name, point, layout) => {
                            self.open_tray_menu(name, point, layout)
                        }
                        _ => Task::none(),
                    };
                    task.chain(open_task)
                } else {
                    Task::none()
                }
            }
            Message::TrayMenu(win_id, message) => {
                if let Some(Feat::TrayMenu(menu)) = self.features.get_mut(&win_id) {
                    let inner_task = menu
                        .update(message.clone())
                        .map(move |m| Message::TrayMenu(win_id, m));

                    let out_task = match message {
                        tray_menu::Message::CloseMenu => Task::done(Message::RemoveWindow(win_id)),
                        _ => Task::none(),
                    };
                    inner_task.chain(out_task)
                } else {
                    Task::none()
                }
            }
            Message::NiriMon(message) => {
                let inner_task = self.mon_serv.update(message).map(Message::NiriMon);
                let num_mon = self.mon_serv.len();
                let mon_names: Vec<String> =
                    self.mon_serv.iter().map(|mon| mon.0.clone()).collect();

                let mut tasks: Vec<_> = mon_names
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
            Message::Quit => exit(),
            _ => Task::none(),
        }
    }

    fn view(&self, win_id: Id) -> Element<'_, Message> {
        match self.features.get(&win_id) {
            Some(Feat::Delora(delora)) => delora.view().map(move |m| Message::Delora(win_id, m)),
            Some(Feat::TrayMenu(menu_feat)) => {
                menu_feat.view().map(move |m| Message::TrayMenu(win_id, m))
            }
            None => container(space()).into(),
        }
    }
}

impl Daemon {
    fn open_delora_main(&mut self, output_name: String) -> Task<Message> {
        let (mut main_bar, main_layer_settings) =
            DeloraMain::new(delora::Init { output_name }).open();
        let main_id = main_bar.id;

        let remove = self
            .features
            .iter()
            .find(|(_, feat)| matches!(feat, Feat::Delora(_)))
            .and_then(|(win_id, old_win)| {
                if let Feat::Delora(old_win) = old_win {
                    main_bar.view.clone_servs(old_win);
                    return Some(*win_id);
                }
                None
            })
            .map(|win_id| {
                self.features.remove(&win_id);
                Task::done(Message::RemoveWindow(win_id))
            })
            .unwrap_or(Task::none());

        self.features.insert(main_id, Feat::Delora(main_bar));

        remove.chain(Task::done(Message::NewLayerShell {
            settings: main_layer_settings,
            id: main_id,
        }))
    }

    fn open_tray_menu(&mut self, name: String, point: Point, layout: TrayLayout) -> Task<Message> {
        let remove = self
            .features
            .iter()
            .find(|(_, feat)| matches!(feat, Feat::TrayMenu(_)))
            .map(|(win_id, _)| *win_id)
            .map(|win_id| {
                info!("Removing old tray menu windows");
                self.features.remove(&win_id);
                Task::done(Message::RemoveWindow(win_id))
            })
            .unwrap_or(Task::none());

        let (new_feat, settings) = tray_menu::MenuComp::new(tray_menu::Init {
            name,
            starting_position: point,
            layout,
        })
        .open();
        let main_id = new_feat.id;

        self.features.insert(main_id, Feat::TrayMenu(new_feat));

        info!("opening tray menu window");

        remove.chain(Task::done(Message::NewMenu {
            settings,
            id: main_id,
        }))
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
