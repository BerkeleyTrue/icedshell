use iced::{
    Length, Task, padding,
    widget::{
        Button,
        button::{self, Status},
        container, row,
    },
};

use crate::{
    feature::Comp,
    theme::CAT_THEME,
    widget::divider::{self, Angled},
};

pub struct PowerButton {}

#[derive(Debug, Clone)]
pub enum Message {
    OnClick,
}

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
        let main_clr = theme.surface2();

        let cap = Angled::new(
            main_clr,
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
            .padding(padding::horizontal(spacing.sm()));

        let icon = Button::new(icon)
            .style(move |_, status| button::Style {
                background: match status {
                    Status::Hovered | Status::Pressed => Some(theme.overlay2().into()),
                    _ => Some(main_clr.into()),
                },
                ..Default::default()
            })
            .on_press(Message::OnClick);

        container(row![cap, icon]).into()
    }
}
