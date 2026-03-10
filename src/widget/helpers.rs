use iced::{
    Element, Length,
    widget::{Container, container, row},
};

macro_rules! align_center {
    ($el:expr$(,)?) => {
        iced::widget::container(iced::Element::from($el)).center_y(iced::Length::Fill)
    };
}

pub(crate) use align_center;

pub enum Alignment {
    Left,
    Right,
    Center,
}

pub fn wrap_widgets<'a, Message: 'a>(
    children: impl IntoIterator<Item = Element<'a, Message>>,
    alignment: Alignment,
) -> Container<'a, Message> {
    let content = container(row(children));
    match alignment {
        Alignment::Left => content.align_left(Length::Fill),
        Alignment::Right => content.align_right(Length::Fill),
        Alignment::Center => content.center_x(Length::Fill),
    }
}

macro_rules! left_widgets {
    () => (
        $crate::widget::wrap_widgets([], $crate::widget::Alignment::Left)
    );
    ($($x:expr),+ $(,)?) => (
        $crate::widget::wrap_widgets([$(iced::Element::from($x)),+], $crate::widget::Alignment::Left)
    );
}

pub(crate) use left_widgets;

macro_rules! center_widgets {
    () => (
        $crate::widget::wrap_widgets([], $crate::widget::Alignment::Center)
    );
    ($($x:expr),+ $(,)?) => (
        $crate::widget::wrap_widgets([$(iced::Element::from($x)),+], $crate::widget::Alignment::Center)
    );
}

pub(crate) use center_widgets;

macro_rules! right_widgets {
    () => (
        $crate::widget::wrap_widgets([], $crate::widget::Alignment::Right)
    );
    ($($x:expr),+ $(,)?) => (
        $crate::widget::wrap_widgets([$(iced::Element::from($x)),+], $crate::widget::Alignment::Right)
    );
}

pub(crate) use right_widgets;

macro_rules! bar_widgets {
    (left: $($x:expr),* $(,)?; center: $($y:expr),* $(,)?; right: $($z:expr),* $(,)?) => (
        iced::widget::container(row![
            left_widgets![$($x),*],
            center_widgets![$($y),*],
            right_widgets![$($z),*],
        ])
    );
}

pub(crate) use bar_widgets;
