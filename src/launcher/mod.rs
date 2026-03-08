mod app_serv;

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
    feature::{Comp, Feature, Service, align_center},
    launcher::app_serv::{AppDesc, AppServ},
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
    AppServ(app_serv::Message),
    LeftPressed(
        /// captured
        bool,
    ),
    RightPressed(
        /// captured
        bool,
    ),
    IKeyPressed(
        /// captured
        bool,
    ),
}

pub struct Launcher {
    search: String,
    prompt_type: PromptType,
    app_serv: AppServ,
    page: usize,
    mode: Mode,
}

impl Comp for Launcher {
    type Message = Message;
    type Init = ();

    fn new<O: MaybeSend + 'static>(
        _input: Self::Init,
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
                    key: Key::Named(Named::Escape),
                    ..
                }) => Some(Message::EscapePressed(captured)),
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key: Key::Named(Named::ArrowLeft),
                    ..
                }) => Some(Message::LeftPressed(captured)),
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key: Key::Named(Named::ArrowRight),
                    ..
                }) => Some(Message::RightPressed(captured)),
                Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) => match key.as_ref() {
                    Key::Character("i") => Some(Message::IKeyPressed(captured)),
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
                    info!("escape: normal");
                    self.mode = Mode::Normal;
                } else {
                    info!("close window");
                }
                Task::none()
            }
            Message::SearchUpdated(search) => {
                self.search = search;
                self.page = 0;
                Task::done(Message::AppServ(app_serv::Message::Query(
                    app_serv::Query::new(
                        if self.search.is_empty() {
                            None
                        } else {
                            Some(self.search.clone())
                        },
                        self.page,
                        NUM_OF_ITEMS,
                    ),
                )))
            }
            Message::RightPressed(captured) => {
                if !captured {
                    self.page = self.page.saturating_add(1);
                    Task::done(Message::AppServ(app_serv::Message::Query(
                        app_serv::Query::new(
                            if self.search.is_empty() {
                                None
                            } else {
                                Some(self.search.clone())
                            },
                            self.page,
                            NUM_OF_ITEMS,
                        ),
                    )))
                } else {
                    Task::none()
                }
            }
            Message::LeftPressed(captured) => {
                if !captured {
                    self.page = self.page.saturating_sub(1);
                    Task::done(Message::AppServ(app_serv::Message::Query(
                        app_serv::Query::new(
                            if self.search.is_empty() {
                                None
                            } else {
                                Some(self.search.clone())
                            },
                            self.page,
                            NUM_OF_ITEMS,
                        ),
                    )))
                } else {
                    Task::none()
                }
            }
            Message::IKeyPressed(captured) => {
                if !captured {
                    self.mode = Mode::Insert;
                    focus("search-input")
                } else {
                    Task::none()
                }
            }
            Message::AppServ(message) => self.app_serv.update(message).map(Message::AppServ),
        }
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        let theme = &CAT_THEME;
        let prompt = {
            let size = theme.spacing().lg();
            let input = text_input("", &self.search)
                .id("search-input")
                .width(Length::Fill)
                .padding(padding::horizontal(theme.spacing().xs()))
                .style(|_, _| text_input::Style {
                    background: theme.background().into(),
                    border: Border::default(),
                    icon: theme.text_color(),
                    placeholder: theme.subtext0(),
                    value: theme.text_color(),
                    selection: theme.subtext1(),
                })
                .size(size)
                .on_input(Message::SearchUpdated);

            let prompt = self.prompt_type.to_string();
            let prompt = text!("{prompt} >").size(size);

            align_center!(row![prompt, input])
                .style(|_| container::Style {
                    border: border::width(theme.spacing().xxs())
                        .rounded(theme.radius().sm())
                        .color({
                            match self.mode {
                                Mode::Normal => theme.blue(),
                                Mode::Insert => theme.green(),
                            }
                        }),
                    ..Default::default()
                })
                .padding(padding::right(theme.spacing().md()))
                .height(theme.spacing().xl2())
        };

        let results = { container(self.view_apps()).height(Length::Fill) };

        let content = column![prompt, results].height(Length::Fill);

        container(content)
            .align_y(Vertical::Top)
            .style(|_| container::Style {
                background: Some(theme.background().into()),
                text_color: Some(theme.text_color()),
                border: border::color(theme.pink())
                    .width(theme.spacing().xs())
                    .rounded(theme.radius().sm()),
                ..Default::default()
            })
            .padding(theme.spacing().lg())
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

impl Launcher {
    fn view_apps(&self) -> Element<'static, Message> {
        let theme = &CAT_THEME;
        self.app_serv
            .res
            .iter()
            .map(|AppDesc { name, icon, .. }| {
                let title = align_center!(text!("{name}").size(theme.spacing().lg()));
                let icon = icon
                    .as_ref()
                    .map(|fdo_icon| fdo_icon.elem(theme.spacing().xl()))
                    .map(|icon| {
                        container(icon)
                            .padding(padding::right(theme.spacing().sm()))
                            .center_y(Length::Fill)
                            .into()
                    })
                    .unwrap_or(Element::from(Space::new()));
                align_center!(row![icon, title])
                    .padding(padding::horizontal(theme.spacing().md()))
                    .width(Length::Fill)
            })
            .fold(Column::new(), |col, row| col.push(row))
            .into()
    }
}

impl Feature for Launcher {
    type Settings = NewLayerShellSettings;

    fn layer(&self) -> Self::Settings {
        NewLayerShellSettings {
            layer: Layer::Overlay,
            size: Some((800, 600)),
            anchor: Anchor::empty(),
            keyboard_interactivity: KeyboardInteractivity::OnDemand,
            output_option: OutputOption::LastOutput,
            namespace: Some("AppLauncher".into()),
            events_transparent: false,
            exclusive_zone: None,
            margin: None,
        }
    }
}
