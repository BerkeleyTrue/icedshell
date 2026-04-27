use iced::{
    Length, Task, padding,
    widget::{container, row},
};

use crate::{
    feature::Comp,
    theme::CAT_THEME,
    widget::{
        container_ext::ContainExt,
        divider::{self, Angled},
    },
};

pub struct PowerButton {}

#[derive(Debug, Clone)]
pub enum Message {}

impl Comp for PowerButton {
    type Message = Message;
    type Init = ();

    fn new<O: iced::advanced::graphics::futures::MaybeSend + 'static>(
        _input: Self::Init,
        _f: impl Fn(Self::Message) -> O + iced::advanced::graphics::futures::MaybeSend + 'static,
    ) -> (Self, iced::Task<O>) {
        (Self {}, Task::none())
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        let theme = &CAT_THEME;
        let spacing = theme.spacing();

        let cap = Angled::new(
            theme.overlay1(),
            theme.trans(),
            divider::Direction::Left,
            divider::Heading::South,
            theme.spacing().xl(),
        );

        let icon = lucide_icons::Icon::Power
            .widget()
            .color(theme.red())
            .center()
            .size(spacing.lg());

        let icon = container(icon)
            .center_y(Length::Fill)
            .padding(padding::horizontal(spacing.md()))
            .background(theme.overlay1());

        row![cap, icon].into()
    }
}
