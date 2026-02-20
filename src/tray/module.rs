use iced::{
    Color, Task, padding,
    widget::{row, text, tooltip},
};
use lucide_icons::Icon;

use crate::{
    divider::{Angled, Direction, Heading},
    feature::{CompWithProps, align_center},
    theme::{SURFACE2, app_theme},
    tray::service::TrayService,
    widget_ext::ContainExt,
};

pub struct TrayMod {}

#[derive(Debug, Clone)]
pub enum Message {}

pub struct Props<'a> {
    pub next_color: Color,
    pub serv: &'a TrayService,
}

impl CompWithProps for TrayMod {
    type Message = Message;
    type Init = ();
    type Props<'a> = Props<'a>;

    fn new(_input: Self::Init) -> Self {
        Self {}
    }

    fn update(&mut self, _message: Self::Message) -> iced::Task<Self::Message> {
        Task::none()
    }

    fn view<'a>(&self, props: Self::Props<'a>) -> iced::Element<'_, Self::Message> {
        let theme = app_theme();
        let items = props.serv.items.values().map(|item| {
            let height = if item.title == "tailscale-systray" {
                theme.spacing().xl() - theme.spacing().sm()
            } else {
                theme.spacing().xl() - theme.spacing().xs()
            };
            let content = align_center!(
                item.icon
                    .as_ref()
                    .map(|icon| icon.elem(height))
                    .unwrap_or(Icon::Dot.widget().into())
            );

            if let Some((icon, title, description)) = item.tool_tip.as_ref() {
                let icon = icon
                    .as_ref()
                    .map(|icon| icon.elem(theme.spacing().xl() - theme.spacing().sm()))
                    .unwrap_or(Icon::Dot.widget().into());

                let tooltip_content = align_center!(row![icon, text!("{title}: {description}"),])
                    .background(theme.background());

                tooltip(content, tooltip_content, tooltip::Position::Top).into()
            } else {
                let title = &item.title;
                let tooltip_content =
                    align_center!(text!("{title}")).background(theme.background());
                tooltip(content, tooltip_content, tooltip::Position::Top).into()
            }
        });

        let end_div = align_center!(Angled::new(
            SURFACE2,
            Direction::Right,
            Heading::North,
            theme.spacing().xl(),
        ))
        .background(props.next_color);

        let content = align_center!(row(items).spacing(theme.spacing().xxs()))
            .background(SURFACE2)
            .padding(padding::horizontal(theme.spacing().sm()));

        row![content, end_div].into()
    }
}
