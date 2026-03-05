use iced::{
    Color, Task,
    advanced::graphics::futures::MaybeSend,
    padding,
    widget::{button, row, text, tooltip},
};
use lucide_icons::Icon;
use tracing::debug;

use crate::{
    divider::{Angled, Direction, Heading},
    feature::{CompWithProps, align_center},
    theme::CAT_THEME,
    tray::{dbus::TrayLayout, service::TrayService},
    widget_ext::ContainExt,
};

#[derive(Debug, Clone)]
pub enum Message {
    SnItemClicked(
        /// name
        String,
        /// menu layout
        TrayLayout,
    ),
}

pub struct Props<'a> {
    pub next_color: Color,
    pub serv: &'a TrayService,
}

pub struct TrayComp {}

impl CompWithProps for TrayComp {
    type Message = Message;
    type Init = ();
    type Props<'a> = Props<'a>;

    fn new<O: MaybeSend + 'static>(
        _input: Self::Init,
        _f: impl Fn(Self::Message) -> O + MaybeSend + 'static,
    ) -> (Self, Task<O>) {
        Self {}.to_tuple()
    }

    fn update(&mut self, message: Self::Message) -> iced::Task<Self::Message> {
        match message {
            Message::SnItemClicked(name, _) => {
                debug!("{name} clicked:");
                Task::none()
            }
        }
    }

    fn view<'a>(&self, props: Self::Props<'a>) -> iced::Element<'_, Self::Message> {
        let theme = &CAT_THEME;
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
                        background: Some(theme.surface1().into()),
                        ..Default::default()
                    },
                    _ => button::Style {
                        background: Some(theme.surface2().into()),
                        ..Default::default()
                    },
                })
                .on_press(Message::SnItemClicked(item.name.clone(), item.menu.clone()))
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
            theme.surface2(),
            Direction::Right,
            Heading::North,
            theme.spacing().xl(),
        ))
        .background(props.next_color);

        let content = align_center!(row(items).spacing(theme.spacing().xs()))
            .background(theme.surface2())
            .padding(padding::horizontal(theme.spacing().sm()));

        row![content, end_div].into()
    }
}
