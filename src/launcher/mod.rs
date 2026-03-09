mod app_serv;
mod modi;

use derive_more::Display;
use iced::{
    Border, Element, Event, Length, Task,
    advanced::graphics::futures::MaybeSend,
    alignment::Vertical,
    border, event,
    keyboard::{self, Key, key::Named},
    padding,
    widget::{Column, Space, column, container, operation::focus, row, text, text_input},
};
use iced_layershell::reexport::{
    Anchor, KeyboardInteractivity, Layer, NewLayerShellSettings, OutputOption,
};
use tracing::info;

use crate::{
    config::MonitorId,
    feature::{Comp, Feature, Service, align_center},
    launcher::{
        app_serv::AppServ,
        modi::{Modi, Query, Res},
    },
    theme::CAT_THEME,
};

const NUM_OF_ITEMS: usize = 10;
#[derive(Clone, Debug, Display)]
enum PromptType {
    Run,
}

#[derive(Clone, Debug, Display)]
enum Mode {
    Normal,
    Insert,
}

#[derive(Clone, Debug)]
pub enum Message {
    EscapePressed(
        /// captured
        bool,
    ),
    SearchUpdated(String),
    OnSubmit(String),
    ExecSuccess,
    AppServ(app_serv::Message),
    LeftPressed(
        /// captured
        bool,
    ),
    RightPressed(
        /// captured
        bool,
    ),
    UpPressed(
        /// captured
        bool,
    ),
    DownPressed(
        /// captured
        bool,
    ),
    // insert
    IKeyPressed(
        /// captured
        bool,
    ),
    // vim movements
    HKeyPressed(
        /// captured
        bool,
    ),
    JKeyPressed(
        /// captured
        bool,
    ),
    KKeyPressed(
        /// captured
        bool,
    ),
    LKeyPressed(
        /// captured
        bool,
    ),
}

pub struct Init {
    pub output: Option<MonitorId>,
}

pub struct Launcher {
    search: String,
    prompt_type: PromptType,
    app_serv: AppServ,
    page: usize,
    mode: Mode,
    selected: usize,
    monitor: Option<MonitorId>,
}

impl Comp for Launcher {
    type Message = Message;
    type Init = Init;

    fn new<O: MaybeSend + 'static>(
        input: Self::Init,
        f: impl Fn(Self::Message) -> O + MaybeSend + 'static,
    ) -> (Self, Task<O>) {
        let (app_serv, app_serv_task) = AppServ::new((), Message::AppServ);
        (
            Self {
                app_serv,
                page: 0,
                prompt_type: PromptType::Run,
                search: "".to_string(),
                mode: Mode::Insert,
                selected: 0,
                monitor: input.output,
            },
            {
                let outer_task = Task::future(async {
                    tokio::time::sleep(tokio::time::Duration::from_millis(250)).await;
                })
                .discard()
                .chain(focus::<Message>("search-input"));

                Task::batch([app_serv_task, outer_task]).map(f)
            },
        )
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        event::listen_with(|event, status, _| {
            let captured = matches!(status, event::Status::Captured);
            match event {
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key: Key::Named(named),
                    ..
                }) => match named {
                    Named::Escape => Some(Message::EscapePressed(captured)),
                    Named::ArrowLeft => Some(Message::LeftPressed(captured)),
                    Named::ArrowRight => Some(Message::RightPressed(captured)),
                    Named::ArrowDown => Some(Message::DownPressed(captured)),
                    Named::ArrowUp => Some(Message::UpPressed(captured)),
                    _ => None,
                },
                Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) => match key.as_ref() {
                    Key::Character("i") => Some(Message::IKeyPressed(captured)),
                    Key::Character("h") => Some(Message::HKeyPressed(captured)),
                    Key::Character("j") => Some(Message::JKeyPressed(captured)),
                    Key::Character("k") => Some(Message::KKeyPressed(captured)),
                    Key::Character("l") => Some(Message::LKeyPressed(captured)),
                    _ => None,
                },
                _ => None,
            }
        })
    }

    fn update(&mut self, message: Self::Message) -> iced::Task<Self::Message> {
        match message {
            Message::EscapePressed(captured) => {
                if captured {
                    self.mode = Mode::Normal;
                } else {
                    info!("close launcher");
                }
                Task::none()
            }
            Message::SearchUpdated(search) => {
                self.search = search;
                self.page = 0;
                self.app_serv
                    .query(Query::new(
                        if self.search.is_empty() {
                            None
                        } else {
                            Some(self.search.clone())
                        },
                        self.page,
                        NUM_OF_ITEMS,
                    ))
                    .map(Message::AppServ)
            }
            Message::OnSubmit(app_id) => self
                .app_serv
                .exec(&app_id)
                .inspect_err(|err| {
                    info!("Error exec: {err:?}");
                })
                .map(|_| Task::done(Message::ExecSuccess))
                .unwrap_or(Task::none()),
            Message::ExecSuccess => Task::none(),
            Message::LeftPressed(captured) => {
                if !captured {
                    self.page_back()
                } else {
                    Task::none()
                }
            }
            Message::RightPressed(captured) => {
                if !captured {
                    self.page_forward()
                } else {
                    Task::none()
                }
            }
            Message::UpPressed(_captured) => {
                self.selected = self.selected.saturating_sub(1);
                Task::none()
            }
            Message::DownPressed(_captured) => {
                self.selected = (self.selected + 1).min(self.app_serv.len() - 1);
                Task::none()
            }

            Message::IKeyPressed(captured) => {
                if !captured {
                    self.mode = Mode::Insert;
                    focus("search-input")
                } else {
                    Task::none()
                }
            }
            Message::HKeyPressed(captured) => {
                if !captured {
                    self.page_back()
                } else {
                    Task::none()
                }
            }
            Message::JKeyPressed(captured) => {
                if !captured {
                    self.selected = (self.selected + 1).min(self.app_serv.len() - 1);
                }
                Task::none()
            }
            Message::KKeyPressed(captured) => {
                if !captured {
                    self.selected = self.selected.saturating_sub(1);
                }
                Task::none()
            }
            Message::LKeyPressed(captured) => {
                if !captured {
                    self.page_forward()
                } else {
                    Task::none()
                }
            }
            Message::AppServ(message) => {
                let inner_task = self.app_serv.update(message.clone()).map(Message::AppServ);

                if matches!(message, app_serv::Message::Query(_)) {
                    self.selected = self.selected.min(self.app_serv.len().saturating_sub(1));
                }

                inner_task
            }
        }
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        let theme = &CAT_THEME;
        let spacing = theme.spacing();
        let cur_selected = self
            .app_serv
            .res()
            .get(self.selected)
            .map(|res| res.id.clone())
            .map(Message::OnSubmit);

        let prompt = {
            let size = spacing.lg();
            let input = text_input("", &self.search)
                .id("search-input")
                .width(Length::Fill)
                .padding(padding::horizontal(spacing.xs()))
                .style(|_, _| text_input::Style {
                    background: theme.background().into(),
                    border: Border::default(),
                    icon: theme.text_color(),
                    placeholder: theme.subtext0(),
                    value: theme.text_color(),
                    selection: theme.subtext1(),
                })
                .size(size)
                .on_input(Message::SearchUpdated)
                .on_submit_maybe(cur_selected);

            let prompt_type = self.prompt_type.to_string();
            let mode = self.mode.to_string().to_uppercase();
            let mode_color = match self.mode {
                Mode::Normal => theme.blue(),
                Mode::Insert => theme.green(),
            };
            let prompt = text!("{mode}|{prompt_type} >").size(size).color(mode_color);

            align_center!(row![prompt, input])
                .padding(padding::right(spacing.md()))
                .height(spacing.xl2())
        };

        let results = {
            container(self.view_apps())
                .height(Length::Fill)
                .align_y(Vertical::Top)
        };

        let content = column![prompt, results].height(Length::Fill);

        container(content)
            .align_y(Vertical::Top)
            .style(|_| container::Style {
                background: Some(theme.background().into()),
                text_color: Some(theme.text_color()),
                border: border::color(theme.pink())
                    .width(spacing.xs())
                    .rounded(theme.radius().sm()),
                ..Default::default()
            })
            .padding(spacing.lg())
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

impl Launcher {
    fn view_apps(&self) -> Element<'static, Message> {
        let theme = &CAT_THEME;
        let spacing = theme.spacing();
        let selected = self
            .app_serv
            .res()
            .get(self.selected)
            .as_ref()
            .map(|res| &res.id);

        self.app_serv
            .res()
            .iter()
            .map(move |Res { id, icon, content }| {
                let is_selected = selected.is_some_and(|inner_id| inner_id == id);
                let title = align_center!(text!("{content}").size(spacing.lg()));
                let icon = icon
                    .as_ref()
                    .map(|fdo_icon| fdo_icon.elem(spacing.xl()))
                    .map(|icon| {
                        container(icon)
                            .padding(padding::right(spacing.sm()))
                            .center_y(Length::Fill)
                            .into()
                    })
                    .unwrap_or(Element::from(Space::new()));

                align_center!(row![icon, title])
                    .padding(padding::horizontal(spacing.md()))
                    .style(move |_| container::Style {
                        border: border::width(spacing.xs())
                            .rounded(theme.radius().lg())
                            .color({
                                if is_selected {
                                    theme.teal()
                                } else {
                                    theme.trans()
                                }
                            }),
                        ..Default::default()
                    })
                    .height(spacing.xl3())
                    .width(Length::Fill)
            })
            .fold(Column::new().spacing(spacing.xs()), |col, row| {
                col.push(row)
            })
            .into()
    }

    fn page_forward(&mut self) -> Task<Message> {
        self.page += 1;
        self.app_serv
            .query(Query::new(
                if self.search.is_empty() {
                    None
                } else {
                    Some(self.search.clone())
                },
                self.page,
                NUM_OF_ITEMS,
            ))
            .map(Message::AppServ)
    }

    fn page_back(&mut self) -> Task<Message> {
        self.page = self.page.saturating_sub(1);
        self.app_serv
            .query(Query::new(
                if self.search.is_empty() {
                    None
                } else {
                    Some(self.search.clone())
                },
                self.page,
                NUM_OF_ITEMS,
            ))
            .map(Message::AppServ)
    }
}

// TODO: update layout through delora
impl Feature for Launcher {
    type Settings = NewLayerShellSettings;

    fn layer(&self) -> Self::Settings {
        NewLayerShellSettings {
            layer: Layer::Overlay,
            size: Some((800, 600)),
            anchor: Anchor::empty(),
            keyboard_interactivity: KeyboardInteractivity::OnDemand,
            // NOTE: lastOutput doesn't work to since plugging in/out a monitor
            // means last monitor is lost and fails to open silently
            // and none, will sometimes open on random monitors
            output_option: self
                .monitor
                .as_ref()
                .map(|monitor| OutputOption::OutputName(monitor.inner().to_owned()))
                .unwrap_or(OutputOption::None),
            namespace: Some("AppLauncher".into()),
            events_transparent: false,
            exclusive_zone: None,
            margin: None,
        }
    }
}
