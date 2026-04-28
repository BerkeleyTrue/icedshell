use std::collections::HashMap;

use derive_more::{Deref, DerefMut};
use iced::{
    Color, Element, Event, Subscription, Task,
    advanced::graphics::futures::MaybeSend,
    event, mouse,
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
#[allow(unused_imports)]
use tracing::{debug, error as log_err, info};

use crate::{
    Cli,
    bars::{delora_main, delora_sec},
    feature::{Comp, FeatWindow, Feature, Service},
    launcher,
    niri::{self, monitors::MonitorsServ},
    osd, powermenu, socket,
    theme::{self as mytheme},
    tray::{TrayLayout, TrayMenuItemId, menu_comp as tray_menu},
};

#[derive(Debug, Clone)]
enum Host {
    Delora,
    Rena,
}

#[derive(Clone)]
pub struct Init {
    host: Host,
}

impl Init {
    pub fn host(&mut self, host: &str) {
        self.host = match host {
            "delora" => Host::Delora,
            "rena" => Host::Rena,
            _ => Host::Delora,
        };
    }
}

impl From<Cli> for Init {
    fn from(_cli: Cli) -> Self {
        Self { host: Host::Delora }
    }
}

enum Feat {
    Delora(FeatWindow<delora_main::DeloraMain>),
    DeloraSec(FeatWindow<delora_sec::DeloraSec>),
    TrayMenu(FeatWindow<tray_menu::MenuComp>),
    Launcher(FeatWindow<launcher::Launcher>),
    Osd(FeatWindow<osd::Osd>),
    PowerMenu(FeatWindow<powermenu::PowerMenu>),
}

#[derive(Deref, DerefMut)]
struct Features(HashMap<Id, Feat>);

#[to_layer_message(multi)]
#[derive(Debug, Clone)]
pub enum Message {
    NiriMon(niri::monitors::Message),

    Delora(Id, delora_main::Message),
    DeloraSec(Id, delora_sec::Message),
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

    Osd(Id, osd::Message),

    PowerMenu(Id, powermenu::Message),

    Socket(socket::Request),
}

struct Daemon {
    features: Features,
    mon_serv: niri::monitors::MonitorsServ,
    tray_focused: bool,
    tray_close_handle: Option<Handle>,
    host: Host,
}

impl Daemon {
    fn new(init: Init) -> (Self, Task<Message>) {
        let (mon_serv, mon_serv_task) = MonitorsServ::new((), Message::NiriMon);
        (
            Self {
                host: init.host,
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
                    Feat::DeloraSec(delora) => delora
                        .subscription()
                        .with(win_id)
                        .map(|(win_id, m)| Message::DeloraSec(win_id, m)),
                    Feat::TrayMenu(menu) => menu
                        .subscription()
                        .with(win_id)
                        .map(|(win_id, m)| Message::TrayMenu(win_id, m)),
                    Feat::Launcher(launcher) => launcher
                        .subscription()
                        .with(win_id)
                        .map(|(win_id, m)| Message::Launcher(win_id, m)),
                    Feat::Osd(osd) => osd
                        .subscription()
                        .with(win_id)
                        .map(|(win_id, m)| Message::Osd(win_id, m)),
                    Feat::PowerMenu(powermenu) => powermenu
                        .subscription()
                        .with(win_id)
                        .map(|(win_id, m)| Message::PowerMenu(win_id, m)),
                }
            })
            .collect();

        let mut subs = vec![niri_mon, focus_subs, socket_sub];
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
                        delora_main::Message::OpenTrayMenu(name, layout) => {
                            self.open_tray_menu(name, layout)
                        }
                        delora_main::Message::PowerButtonOnClicked => self.open_powermenu(),
                        _ => Task::none(),
                    };
                    task.chain(open_task)
                } else {
                    Task::none()
                }
            }
            Message::DeloraSec(win_id, message) => {
                if let Some(Feat::DeloraSec(delora)) = self.features.get_mut(&win_id) {
                    delora
                        .update(message.clone())
                        .map(move |m| Message::DeloraSec(win_id, m))
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

                    let out_task = match message {
                        launcher::Message::EscapePressed(false)
                        | launcher::Message::ExecSuccess => {
                            Task::done(Message::RemoveWindow(win_id))
                        }
                        _ => Task::none(),
                    };

                    inner_task.chain(out_task)
                } else {
                    Task::none()
                }
            }
            Message::Osd(win_id, message) => {
                if let Some(Feat::Osd(osd)) = self.features.get_mut(&win_id) {
                    let inner = osd.update(message.clone()).map_feat(win_id, Message::Osd);
                    let outer = match message {
                        osd::Message::Timeout => Task::done(Message::RemoveWindow(win_id)),
                        // _ => Task::none(),
                    };
                    inner.chain(outer)
                } else {
                    Task::none()
                }
            }
            Message::PowerMenu(win_id, message) => {
                if let Some(Feat::PowerMenu(powermenu)) = self.features.get_mut(&win_id) {
                    let inner = powermenu
                        .update(message.clone())
                        .map_feat(win_id, Message::PowerMenu);

                    let outer = match message {
                        powermenu::Message::QuitApp => Task::done(Message::RemoveWindow(win_id)),
                        _ => Task::none(),
                    };

                    inner.chain(outer)
                } else {
                    Task::none()
                }
            }
            Message::NiriMon(message) => {
                let inner_task = self.mon_serv.update(message).map(Message::NiriMon);
                let num_mon = self.mon_serv.len();
                let mon_names: Vec<String> = self
                    .mon_serv
                    .iter()
                    .map(|mon| mon.inner().to_owned())
                    .collect();

                let mut tasks: Vec<_> = mon_names
                    .iter()
                    .map({
                        let host = self.host.clone();
                        move |mon| match (&host, num_mon, mon.as_str()) {
                            (Host::Delora, 2, "HDMI-A-1") => {
                                self.open_delora_main("HDMI-A-1".into())
                            }
                            (Host::Delora, 2, "DP-3") => self.open_delora_sec("DP-3".into()),
                            (Host::Delora, 1, "DP-3") => self.open_delora_main("DP-3".into()),
                            (Host::Rena, _, "eDP-1") => Task::none(),
                            (_, _, _) => Task::none(),
                        }
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

            Message::Socket(req) => match req {
                socket::Request::Launcher => self.open_launcher(),
                socket::Request::Osd(args) => self.open_osd(args),
                socket::Request::PowerMenu => self.open_powermenu(),
            },

            _ => Task::none(),
        }
    }

    fn view(&self, win_id: Id) -> Element<'_, Message> {
        match self.features.get(&win_id) {
            Some(Feat::Delora(delora)) => delora.view().map_feat(win_id, Message::Delora),
            Some(Feat::DeloraSec(delora)) => delora.view().map_feat(win_id, Message::DeloraSec),
            Some(Feat::TrayMenu(menu_feat)) => menu_feat.view().map_feat(win_id, Message::TrayMenu),
            Some(Feat::Launcher(launcher)) => launcher.view().map_feat(win_id, Message::Launcher),
            Some(Feat::Osd(osd)) => osd.view().map_feat(win_id, Message::Osd),
            Some(Feat::PowerMenu(powermenu)) => {
                powermenu.view().map_feat(win_id, Message::PowerMenu)
            }
            None => container(space()).into(),
        }
    }
}

// delora bar feature logic
impl Daemon {
    /// find old bar,
    /// if so and bar is on output, noop
    /// if so and bar is on wrong output, close and open new one
    /// - clone services
    /// - replace old bar with new bar
    fn open_delora_main(&mut self, output_name: String) -> Task<Message> {
        let old_feat = self
            .features
            .iter()
            .find(|(_, feat)| matches!(feat, Feat::Delora(_)))
            // release features first
            .map(|(win_id, _)| *win_id)
            .and_then(|win_id| self.features.get(&win_id))
            .and_then(|old_feat| match old_feat {
                Feat::Delora(old_feat) => Some(old_feat),
                _ => None,
            });

        if let Some(old_feat) = old_feat
            && old_feat.is_on_output(&output_name)
        {
            return Task::none();
        }

        let (mut main_feat, main_layer_settings, inner_task) =
            delora_main::DeloraMain::open(delora_main::Init { output_name }, Message::Delora);
        let main_id = main_feat.id;

        let remove = old_feat
            .inspect(|old_feat| {
                main_feat.view.clone_servs(old_feat);
            })
            .map(|old_feat| old_feat.id)
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

// delora secondary bar feature logic
impl Daemon {
    fn open_delora_sec(&mut self, output_name: String) -> Task<Message> {
        let old_feat = self
            .features
            .iter()
            .find(|(_, feat)| matches!(feat, Feat::DeloraSec(_)))
            .map(|(win_id, _)| *win_id)
            .and_then(|win_id| self.features.get(&win_id))
            .and_then(|old_feat| match old_feat {
                Feat::DeloraSec(old_feat) => Some(old_feat),
                _ => None,
            });

        if let Some(old_feat) = old_feat
            && old_feat.is_on_output(&output_name)
        {
            return Task::none();
        }

        let (sec_feat, sec_layer_settings, inner_task) =
            delora_sec::DeloraSec::open(delora_sec::Init { output_name }, Message::DeloraSec);
        let sec_id = sec_feat.id;

        let remove = old_feat
            .map(|old_feat| old_feat.id)
            .map(|win_id| {
                self.features.remove(&win_id);
                Task::done(Message::RemoveWindow(win_id))
            })
            .unwrap_or(Task::none());

        self.features.insert(sec_id, Feat::DeloraSec(sec_feat));

        remove
            .chain(Task::done(Message::NewLayerShell {
                settings: sec_layer_settings,
                id: sec_id,
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
                debug!("Removing old tray menu windows");
                self.features.remove(&win_id);
                Task::done(Message::RemoveWindow(win_id))
            })
            .unwrap_or(Task::none());

        let (menu_feat, layer_settings, inner_task) =
            tray_menu::MenuComp::open(tray_menu::Init { name, layout }, Message::TrayMenu);
        let win_id = menu_feat.id;

        self.features.insert(win_id, Feat::TrayMenu(menu_feat));

        debug!("opening tray menu window");

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
        let (launcher_feat, settings, inner_task) = launcher::Launcher::open(
            launcher::Init {
                output: self.mon_serv.cur_monitor().cloned(),
            },
            Message::Launcher,
        );
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
                settings,
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

/// osd logic
impl Daemon {
    fn open_osd(&mut self, args: osd::OsdCommand) -> Task<Message> {
        let (osd_feat, settings, inner_task) = osd::Osd::open(
            osd::Init {
                monitor: self.mon_serv.cur_monitor().cloned(),
                command: args,
            },
            Message::Osd,
        );
        let win_id = osd_feat.id;

        let remove = self
            .features
            .iter()
            .find(|(_, feat)| matches!(feat, Feat::Osd(_)))
            .and_then(|(win_id, old_win)| {
                if let Feat::Osd(_) = old_win {
                    return Some(*win_id);
                }
                None
            })
            .map(|win_id| {
                self.features.remove(&win_id);
                Task::done(Message::RemoveWindow(win_id))
            })
            .unwrap_or(Task::none());

        self.features.insert(win_id, Feat::Osd(osd_feat));

        remove
            .chain(Task::done(Message::NewLayerShell {
                settings,
                id: win_id,
            }))
            .chain(inner_task)
    }
}

/// powermenu logic
impl Daemon {
    fn open_powermenu(&mut self) -> Task<Message> {
        let (powermenu_feat, settings, inner_task) = powermenu::PowerMenu::open(
            powermenu::Init {
                monitor: self.mon_serv.cur_monitor().cloned(),
                dryrun: true,
                no_focus: true,
            },
            Message::PowerMenu,
        );
        let win_id = powermenu_feat.id;

        let remove = self
            .features
            .iter()
            .find(|(_, feat)| matches!(feat, Feat::PowerMenu(_)))
            .and_then(|(win_id, old_win)| {
                if let Feat::PowerMenu(_) = old_win {
                    return Some(*win_id);
                }
                None
            })
            .map(|win_id| {
                self.features.remove(&win_id);
                Task::done(Message::RemoveWindow(win_id))
            })
            .unwrap_or(Task::none());

        self.features
            .insert(win_id, Feat::PowerMenu(powermenu_feat));

        remove
            .chain(Task::done(Message::NewLayerShell {
                settings,
                id: win_id,
            }))
            .chain(inner_task)
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
