use iced::{
    Color, Event, Point, Task,
    event::listen,
    mouse, padding,
    widget::{button, row, text, tooltip},
};
use lucide_icons::Icon;
use tracing::info;

use crate::{
    divider::{Angled, Direction, Heading},
    feature::{CompWithProps, align_center},
    theme::{SURFACE1, SURFACE2, app_theme},
    tray::service::TrayService,
    widget_ext::ContainExt,
};

pub struct TrayComp {
    mouse: Option<Point>,
}

#[derive(Debug, Clone)]
pub enum Message {
    MouseMoved(Point),
    MouseLeft,
    SnItemClicked(Point),
    SnItemRightClicked(Point),
}

pub struct Props<'a> {
    pub next_color: Color,
    pub serv: &'a TrayService,
}

impl CompWithProps for TrayComp {
    type Message = Message;
    type Init = ();
    type Props<'a> = Props<'a>;

    fn new(_input: Self::Init) -> Self {
        Self { mouse: None }
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        listen().filter_map(|event| match event {
            Event::Mouse(mouse::Event::CursorMoved { position: mouse }) => {
                Some(Message::MouseMoved(mouse))
            }
            Event::Mouse(mouse::Event::CursorLeft) => Some(Message::MouseLeft),
            _ => None,
        })
    }

    fn update(&mut self, message: Self::Message) -> iced::Task<Self::Message> {
        match message {
            Message::MouseMoved(point) => {
                self.mouse = Some(point);
                Task::none()
            }
            Message::MouseLeft => {
                self.mouse = None;
                Task::none()
            }
            Message::SnItemClicked(point) => {
                info!("Icon clicked: point {point:?}");
                Task::none()
            }
            Message::SnItemRightClicked(point) => {
                info!("Icon left clicked: point {point:?}");
                Task::none()
            }
        }
    }

    fn view<'a>(&self, props: Self::Props<'a>) -> iced::Element<'_, Self::Message> {
        let theme = app_theme();
        let items = props.serv.items.values().map(|item| {
            let height = theme.spacing().xl() - theme.spacing().sm();
            let content = align_center!(
                button(
                    item.icon
                        .as_ref()
                        .map(|icon| icon.elem(height))
                        .unwrap_or(Icon::Dot.widget().into())
                )
                .padding(padding::vertical(theme.spacing().xs()))
                .style(|_, status| match status {
                    button::Status::Hovered => button::Style {
                        background: Some(SURFACE1.into()),
                        ..Default::default()
                    },
                    _ => button::Style {
                        background: Some(SURFACE2.into()),
                        ..Default::default()
                    },
                })
                .on_press(Message::SnItemClicked(self.mouse.unwrap_or_default()))
            );

            if let Some((icon, title, description)) = item.tool_tip.as_ref() {
                let icon = icon
                    .as_ref()
                    .map(|icon| icon.elem(theme.spacing().xl() - theme.spacing().sm()))
                    .unwrap_or(Icon::Dot.widget().into());

                let tooltip_text = if !description.is_empty() {
                    text!("{title}: {description}")
                } else {
                    text!("{title}")
                };

                let tooltip_content =
                    align_center!(row![icon, tooltip_text]).background(theme.background());

                tooltip(content, tooltip_content, tooltip::Position::FollowCursor).into()
            } else {
                let title = &item.title;
                let tooltip_content =
                    align_center!(text!("{title}")).background(theme.background());
                tooltip(content, tooltip_content, tooltip::Position::FollowCursor).into()
            }
        });

        let end_div = align_center!(Angled::new(
            SURFACE2,
            Direction::Right,
            Heading::North,
            theme.spacing().xl(),
        ))
        .background(props.next_color);

        let content = align_center!(row(items).spacing(theme.spacing().xs()))
            .background(SURFACE2)
            .padding(padding::horizontal(theme.spacing().sm()));

        row![content, end_div].into()
    }
}
