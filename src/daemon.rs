use std::collections::HashMap;

use derive_more::{Deref, DerefMut};
use iced::{
    Color, Element, Event, Subscription, Task,
    advanced::graphics::futures::MaybeSend,
    event, exit,
    keyboard::{self, Key, key::Named},
    mouse,
    task::Handle,
    theme::Style,
    widget::{container, space},
    window::Id,
};
use iced_layershell::{
    reexport::KeyboardInteractivity,
    settings::{LayerShellSettings, StartMode},
    to_layer_message,
};
use tracing::{error as log_err, info};

use crate::{
    AppCommand, Cli,
    delora::{self, DeloraMain},
    feature::{Comp, FeatWindow, Feature, Service},
    launcher,
    niri::{self, monitors::MonitorsServ},
    socket,
    theme::{self as mytheme},
    tray::{TrayLayout, TrayMenuItemId, menu_comp as tray_menu},
};

#[derive(Clone)]
enum Hosts {
    Delora,
    Rena,
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
            quit_keybinds: cli.command.is_some_and(|x| match x {
                AppCommand::Daemon { quit_keybindings } => quit_keybindings,
                _ => false,
            }),
            host: Hosts::Delora,
        }
    }
}

enum Feat {
    Delora(FeatWindow<DeloraMain>),
    TrayMenu(FeatWindow<tray_menu::MenuComp>),
    Launcher(FeatWindow<launcher::Launcher>),
}

#[derive(Deref, DerefMut)]
struct Features(HashMap<Id, Feat>);

#[to_layer_message(multi)]
#[derive(Debug, Clone)]
pub enum Message {
    NiriMon(niri::monitors::Message),

    Delora(Id, delora::Message),
    TrayMenu(Id, tray_menu::Message),
    TrayMenuItemClicked(
        /// sni item name
        String,
        /// menu item id
        TrayMenuItemId,
    ),

    FeatUnfocused(Id),
    FeatFocused(Id),

    OpenLauncher,
    Launcher(Id, launcher::Message),

    Socket(socket::Request),

    Quit,
}

struct Daemon {
    features: Features,
    mon_serv: niri::monitors::MonitorsServ,
    quit_keybinds: bool,
    tray_focused: bool,
    tray_close_handle: Option<Handle>,
}

impl Daemon {
    fn new(init: Init) -> (Self, Task<Message>) {
        let (mon_serv, mon_serv_task) = MonitorsServ::new((), Message::NiriMon);
        (
            Self {
                quit_keybinds: init.quit_keybinds,

                features: Features(HashMap::new()),
                mon_serv,
                tray_focused: false,
                tray_close_handle: None,
            },
            mon_serv_task,
        )
    }

    fn subscription(&self) -> Subscription<Message> {
        let focus_subs = event::listen_with(|event, _status, id| match event {
            Event::Mouse(mouse::Event::CursorEntered) => Some(Message::FeatFocused(id)),
            Event::Mouse(mouse::Event::CursorLeft) => Some(Message::FeatUnfocused(id)),
            _ => None,
        });
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

        let socket_sub = Subscription::run(|| socket::listen().0).filter_map(|res| match res {
            Ok(request) => Some(Message::Socket(request)),
            Err(err) => {
                log_err!("Error starting socket listener: {err:?}");
                None
            }
        });

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
                    Feat::Launcher(launcher) => launcher
                        .subscription()
                        .with(win_id)
                        .map(|(win_id, m)| Message::Launcher(win_id, m)),
                }
            })
            .collect();

        let mut subs = vec![quit_binds, niri_mon, focus_subs, socket_sub];
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
                        delora::Message::OpenTrayMenu(name, layout) => {
                            self.open_tray_menu(name, layout)
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
                        .map_feat(win_id, Message::TrayMenu);

                    let outer_task =
                        if let tray_menu::Message::ItemSelected(name, menu_item_id) = message {
                            Task::done(Message::TrayMenuItemClicked(name, menu_item_id))
                        } else {
                            Task::none()
                        };

                    inner_task.chain(outer_task)
                } else {
                    Task::none()
                }
            }
            Message::Launcher(win_id, message) => {
                if let Some(Feat::Launcher(launcher)) = self.features.get_mut(&win_id) {
                    let inner_task = launcher
                        .update(message.clone())
                        .map_feat(win_id, Message::Launcher);

                    let out_task = if matches!(message, launcher::Message::Close) {
                        Task::done(Message::RemoveWindow(win_id))
                    } else {
                        Task::none()
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
            Message::FeatFocused(id) => match self.features.get(&id) {
                Some(Feat::TrayMenu(_)) => self.focus_tray(),
                _ => Task::none(),
            },
            Message::FeatUnfocused(id) => match self.features.get(&id) {
                Some(Feat::TrayMenu(_)) => self.unfocus_tray(id),
                Some(Feat::Launcher(_)) => self.on_unfocus_launcher(id),
                _ => Task::none(),
            },
            Message::TrayMenuItemClicked(name, menu_item_id) => match self
                .features
                .iter_mut()
                .find(|(_, feat)| matches!(feat, Feat::Delora(_)))
                .map(|(win_id, feat)| (*win_id, feat))
            {
                Some((win_id, Feat::Delora(delora))) => delora
                    .tray_menu_item_clicked(name, menu_item_id)
                    .map_feat(win_id, Message::Delora),
                _ => Task::none(),
            },
            Message::Quit => exit(),

            Message::Socket(req) => match req {
                socket::Request::Launcher => self.open_launcher(),
                // _ => {
                //     info!("request: {req:?}");
                //     Task::none()
                // }
            },
            _ => Task::none(),
        }
    }

    fn view(&self, win_id: Id) -> Element<'_, Message> {
        match self.features.get(&win_id) {
            Some(Feat::Delora(delora)) => delora.view().map_feat(win_id, Message::Delora),
            Some(Feat::TrayMenu(menu_feat)) => menu_feat.view().map_feat(win_id, Message::TrayMenu),
            Some(Feat::Launcher(launcher)) => launcher.view().map_feat(win_id, Message::Launcher),
            None => container(space()).into(),
        }
    }
}

// delora bar feature logic
impl Daemon {
    fn open_delora_main(&mut self, output_name: String) -> Task<Message> {
        let (mut main_feat, main_layer_settings, inner_task) =
            DeloraMain::open(delora::Init { output_name }, Message::Delora);
        let main_id = main_feat.id;

        let remove = self
            .features
            .iter()
            .find(|(_, feat)| matches!(feat, Feat::Delora(_)))
            .and_then(|(win_id, old_win)| {
                if let Feat::Delora(old_win) = old_win {
                    main_feat.view.clone_servs(old_win);
                    return Some(*win_id);
                }
                None
            })
            .map(|win_id| {
                self.features.remove(&win_id);
                Task::done(Message::RemoveWindow(win_id))
            })
            .unwrap_or(Task::none());

        self.features.insert(main_id, Feat::Delora(main_feat));

        remove
            .chain(Task::done(Message::NewLayerShell {
                settings: main_layer_settings,
                id: main_id,
            }))
            .chain(inner_task)
    }
}

// tray menu feature logic
impl Daemon {
    fn focus_tray(&mut self) -> Task<Message> {
        self.tray_focused = true;
        if let Some(handle) = &self.tray_close_handle {
            handle.abort();
            self.tray_close_handle = None;
        }
        Task::none()
    }
    fn unfocus_tray(&mut self, id: Id) -> Task<Message> {
        self.tray_focused = false;
        let (task, handle) = Task::perform(
            tokio::time::sleep(tokio::time::Duration::from_millis(500)),
            move |_| Message::RemoveWindow(id),
        )
        .abortable();

        self.tray_close_handle = Some(handle);

        task
    }
    fn open_tray_menu(&mut self, name: String, layout: TrayLayout) -> Task<Message> {
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

        let (menu_feat, layer_settings, inner_task) =
            tray_menu::MenuComp::open(tray_menu::Init { name, layout }, Message::TrayMenu);
        let win_id = menu_feat.id;

        self.features.insert(win_id, Feat::TrayMenu(menu_feat));

        info!("opening tray menu window");

        remove
            .chain(Task::done(Message::NewMenu {
                settings: layer_settings,
                id: win_id,
            }))
            .chain(inner_task)
    }
}

// launcher window
impl Daemon {
    fn open_launcher(&mut self) -> Task<Message> {
        let (launcher_feat, layer_settings, inner_task) =
            launcher::Launcher::open((), Message::Launcher);
        let win_id = launcher_feat.id;

        let remove = self
            .features
            .iter()
            .find(|(_, feat)| matches!(feat, Feat::Launcher(_)))
            .and_then(|(win_id, old_win)| {
                if let Feat::Launcher(_) = old_win {
                    return Some(*win_id);
                }
                None
            })
            .map(|win_id| {
                self.features.remove(&win_id);
                Task::done(Message::RemoveWindow(win_id))
            })
            .unwrap_or(Task::none());

        self.features.insert(win_id, Feat::Launcher(launcher_feat));

        remove
            .chain(Task::done(Message::NewLayerShell {
                settings: layer_settings,
                id: win_id,
            }))
            .chain(inner_task)
    }

    fn on_unfocus_launcher(&mut self, id: Id) -> Task<Message> {
        Task::perform(
            async {
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            },
            move |_| Message::RemoveWindow(id),
        )
    }
}

pub fn start(init: Init, settings: iced_layershell::Settings) -> anyhow::Result<()> {
    let theme = &mytheme::CAT_THEME;

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
    .run()?;

    Ok(())
}

trait TaskExt<T> {
    fn map_feat<O>(self, id: Id, f: impl FnMut(Id, T) -> O + MaybeSend + 'static) -> Task<O>
    where
        T: MaybeSend + 'static,
        O: MaybeSend + 'static;
}

impl<T> TaskExt<T> for Task<T> {
    fn map_feat<O>(self, id: Id, mut f: impl FnMut(Id, T) -> O + MaybeSend + 'static) -> Task<O>
    where
        T: MaybeSend + 'static,
        O: MaybeSend + 'static,
    {
        self.map(move |m| f(id, m))
    }
}

trait ElementExt<'a, Message> {
    fn map_feat<B>(self, id: Id, f: impl Fn(Id, Message) -> B + 'a) -> Element<'a, B>
    where
        Message: 'a,
        B: 'a;
}

impl<'a, Message> ElementExt<'a, Message> for Element<'a, Message> {
    fn map_feat<B>(self, id: Id, f: impl Fn(Id, Message) -> B + 'a) -> Element<'a, B>
    where
        Message: 'a,
        B: 'a,
    {
        self.map(move |m| f(id, m))
    }
}
