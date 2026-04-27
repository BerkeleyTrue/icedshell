mod button;
pub mod button_comp;
mod dead_internet;
mod palette;

use crate::{
    feature::{Comp, Feature},
    powermenu::{
        button::{Icon, PowerButton},
        palette::{LINEAR_BACKGROUND, PALETTE},
    },
    types::MonitorId,
};

use iced::{
    Border, Color, Element, Event,
    Length::{self, Fill},
    Padding, Shadow, Subscription, Task, Vector,
    advanced::graphics::futures::MaybeSend,
    event,
    keyboard::{self, Key, key::Named},
    padding,
    widget::{column, container, row, space, stack, text},
};
use iced_layershell::reexport::{self as layer, OutputOption};
use tokio::process::Command;
use tracing::{debug, info};

const NUM_OF_BTNS: usize = 5;

#[derive(Debug, Clone)]
pub struct Init {
    pub no_focus: bool,
    pub dryrun: bool,
    pub monitor: Option<MonitorId>,
}

struct FocusButton(usize);
impl FocusButton {
    fn new() -> Self {
        Self(0)
    }
    fn next(&mut self) {
        self.0 = if self.0 == (NUM_OF_BTNS - 1) {
            self.0
        } else {
            self.0 + 1
        };
    }
    fn prev(&mut self) {
        self.0 = if self.0 == 0 { self.0 } else { self.0 - 1 }
    }
    fn is_focused(&self, idx: usize) -> bool {
        self.0 == idx
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    User(String, Option<String>),
    DeadInternet(dead_internet::Message),

    FocusNext,
    FocusPrev,
    Select,

    Lock,
    Sleep,
    Shutdown,
    Reboot,
    Logout,

    QuitApp,
}

pub struct PowerMenu {
    monitor: Option<MonitorId>,
    dryrun: bool,
    user: Option<String>,
    dead_internet: dead_internet::DeadInternet,
    no_focus: bool,
    buttons: [PowerButton<Message>; NUM_OF_BTNS],
    focused_btn: FocusButton,
}

impl Comp for PowerMenu {
    type Init = Init;
    type Message = Message;

    fn new<O: MaybeSend + 'static>(
        input: Self::Init,
        f: impl Fn(Self::Message) -> O + MaybeSend + 'static,
    ) -> (Self, Task<O>) {
        let (dead_internet, task) = dead_internet::DeadInternet::new();
        (
            Self {
                monitor: input.monitor,
                dryrun: input.dryrun,
                no_focus: input.no_focus,
                dead_internet,
                user: None,
                focused_btn: FocusButton::new(),
                buttons: [
                    PowerButton {
                        icon: Icon::Lock,
                        message: Message::Lock,
                    },
                    PowerButton {
                        icon: Icon::Sleep,
                        message: Message::Sleep,
                    },
                    PowerButton {
                        icon: Icon::Reboot,
                        message: Message::Reboot,
                    },
                    PowerButton {
                        icon: Icon::Shutdown,
                        message: Message::Shutdown,
                    },
                    PowerButton {
                        icon: Icon::Logout,
                        message: Message::Logout,
                    },
                ],
            },
            Task::batch(vec![
                Task::future(get_user()),
                task.map(Message::DeadInternet),
            ])
            .map(f),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::QuitApp => Task::none(),

            Message::FocusNext => {
                self.focused_btn.next();
                Task::none()
            }
            Message::FocusPrev => {
                self.focused_btn.prev();
                Task::none()
            }
            Message::Select => match self.focused_btn.0 {
                0 => Task::done(Message::Lock),
                1 => Task::done(Message::Sleep),
                2 => Task::done(Message::Reboot),
                3 => Task::done(Message::Shutdown),
                4 => Task::done(Message::Logout),
                _ => Task::none(),
            },

            Message::Lock => {
                info!("Lock Request");
                self.command("loginctl", vec!["lock-session"])
            }
            Message::Sleep => {
                info!("Sleep Request");
                self.command("systemctl", vec!["suspend"])
            }
            Message::Reboot => {
                info!("Reboot Request");
                self.command("systemctl", vec!["reboot"])
            }
            Message::Shutdown => {
                info!("Shutdown Request");
                self.command("systemctl", vec!["poweroff"])
            }
            Message::Logout => {
                info!("Logout request");
                self.command(
                    "systemctl",
                    vec!["--user", "start", "shutdown-graphical.target"],
                )
            }
            Message::User(user, host) => {
                self.user = host.map(|host| format!("{user}@{host}")).or(Some(user));
                Task::none()
            }
            Message::DeadInternet(message) => self
                .dead_internet
                .update(message)
                .map(Message::DeadInternet),
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        let no_focus = self.no_focus;
        let quit_bindings = keyboard::listen()
            .filter_map(|event| match event {
                keyboard::Event::KeyPressed { key, .. } => Some(key),
                _ => None,
            })
            .filter_map(|key| match key.as_ref() {
                Key::Named(Named::Escape) | Key::Character("q") => Some(Message::QuitApp),
                _ => None,
            });

        let app_events = event::listen_with(move |event, _, _| match event {
            Event::Window(iced::window::Event::Unfocused) => Some(Message::QuitApp),
            _ => None,
        })
        .with(no_focus)
        .filter_map(move |(no_focus, e)| if no_focus { None } else { Some(e) });

        let nav_bindings = keyboard::listen()
            .filter_map(|event| match event {
                keyboard::Event::KeyPressed { key, .. } => Some(key),
                _ => None,
            })
            .filter_map(|key| match key.as_ref() {
                Key::Character("l") | Key::Named(Named::ArrowRight) => Some(Message::FocusNext),
                Key::Character("h") | Key::Named(Named::ArrowLeft) => Some(Message::FocusPrev),
                Key::Named(Named::Enter) => Some(Message::Select),
                _ => None,
            });

        let dead_internet_subs = self
            .dead_internet
            .subscriptions()
            .map(Message::DeadInternet);

        Subscription::batch(vec![
            quit_bindings,
            nav_bindings,
            app_events,
            dead_internet_subs,
        ])
    }

    fn view(&self) -> Element<'_, Message> {
        let palette = &PALETTE;
        let dead_internet = self.dead_internet.view().map(Message::DeadInternet);
        let buttons = row(self
            .buttons
            .iter()
            .enumerate()
            .map(|(idx, b)| b.view(self.focused_btn.is_focused(idx)))
            .collect::<Vec<_>>())
        .spacing(10)
        .padding(padding::top(10))
        .width(Fill);

        let user_container = self
            .user
            .as_ref()
            .map(|user| {
                let inner_box =
                    container(text(user.clone()))
                        .padding(10)
                        .style(|_| container::Style {
                            background: Some(LINEAR_BACKGROUND),
                            border: Border::default()
                                .rounded(1.0)
                                .color(palette.background)
                                .width(2.0),
                            ..Default::default()
                        });
                let outer_box = container(inner_box)
                    .style(|_| container::Style {
                        border: Border::default()
                            .rounded(1.0)
                            .color(palette.text)
                            .width(2.0),
                        text_color: Some(palette.background),
                        ..Default::default()
                    })
                    .padding(1);
                container(outer_box).width(Fill)
            })
            .or(Some(container(space()).width(Fill)))
            .map(|cont| row![cont].padding(Padding::from([25, 25])));

        let content = column![buttons, user_container].padding(Padding::default().horizontal(10));

        // main layout
        let main_layout = container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_| container::Style {
                background: None,
                border: Border::default()
                    .color(palette.text)
                    .rounded(2.0)
                    .width(2.0),
                shadow: Shadow {
                    blur_radius: 8.0,
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.75),
                    offset: Vector::new(3.0, 3.0),
                },
                ..Default::default()
            })
            .padding(8);

        stack![dead_internet, main_layout].into()
    }
}

impl PowerMenu {
    fn command(&self, program: &str, args: Vec<&str>) -> Task<Message> {
        let mut cmd = std::process::Command::new(program);
        cmd.args(&args);

        if self.dryrun {
            debug!("{cmd:#?}");
            let args_print = args.join(" ");
            println!("dryrun: {program} {args_print}");
        } else {
            cmd.output()
                .map_err(|err| format!("Error running command: {err:?}"))
                .unwrap();
        }
        Task::done(Message::QuitApp)
    }
}

async fn get_user() -> Message {
    let user = Command::new("whoami")
        .output()
        .await
        .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
        .unwrap_or("Anon".to_string());

    let host = Command::new("uname")
        .arg("-n")
        .output()
        .await
        .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
        .ok();

    debug!("user: {}, host: {:?}", user, host);

    Message::User(user, host)
}

impl Feature for PowerMenu {
    type Settings = layer::NewLayerShellSettings;
    fn layer(&self) -> Self::Settings {
        Self::Settings {
            size: Some((623, 390)),
            layer: layer::Layer::Overlay,
            anchor: layer::Anchor::empty(),
            margin: None,
            keyboard_interactivity: layer::KeyboardInteractivity::OnDemand,
            output_option: self
                .monitor
                .as_ref()
                .map(|monitor| OutputOption::OutputName(monitor.inner().to_owned()))
                .unwrap_or(OutputOption::None),
            exclusive_zone: None,
            events_transparent: false,
            namespace: Some("IcedOsd".to_owned()),
        }
    }
}
