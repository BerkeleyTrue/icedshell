use std::ops::{Deref, DerefMut};

use iced::{
    Element, Length, Subscription, Task,
    widget::{Container, container, row},
    window,
};
use iced_layershell::reexport::NewLayerShellSettings;
use tracing::debug;

pub struct Window<T>
where
    T: Feature,
{
    pub id: window::Id,
    pub view: T,
}

impl<T> Deref for Window<T>
where
    T: Feature,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.view
    }
}

impl<T> DerefMut for Window<T>
where
    T: Feature,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.view
    }
}

pub trait Comp {
    type Message;
    type Init;

    fn new(input: Self::Init) -> Self;

    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::none()
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message>;
    fn view(&self) -> Element<'_, Self::Message>;
}

pub trait CompWithProps {
    type Message;
    type Init;
    type Props;

    fn new(input: Self::Init) -> Self;

    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::none()
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message>;
    fn view(&self, props: Self::Props) -> Element<'_, Self::Message>;
}

pub trait Feature: Sized + Comp {
    fn layer(&self) -> NewLayerShellSettings;

    fn is_animating(&self) -> bool {
        false
    }

    /// open window, consuming self
    fn open(self) -> (Window<Self>, NewLayerShellSettings) {
        let id = window::Id::unique();
        debug!("{id:}");
        let settings = self.layer();

        (Window { id, view: self }, settings)
    }
}

pub fn wrap_comp<Message>(element: Element<'_, Message>) -> Container<'_, Message> {
    container(element).center_y(Length::Fill)
}

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
        $crate::feature::wrap_widgets([], $crate::feature::Alignment::Left)
    );
    ($($x:expr),+ $(,)?) => (
        $crate::feature::wrap_widgets([$(iced::Element::from($x)),+], $crate::feature::Alignment::Left)
    );
}

pub(crate) use left_widgets;

macro_rules! center_widgets {
    () => (
        $crate::feature::wrap_widgets([], $crate::feature::Alignment::Center)
    );
    ($($x:expr),+ $(,)?) => (
        $crate::feature::wrap_widgets([$(iced::Element::from($x)),+], $crate::feature::Alignment::Center)
    );
}

pub(crate) use center_widgets;

macro_rules! right_widgets {
    () => (
        $crate::feature::wrap_widgets([], $crate::feature::Alignment::Right)
    );
    ($($x:expr),+ $(,)?) => (
        $crate::feature::wrap_widgets([$(iced::Element::from($x)),+], $crate::feature::Alignment::Right)
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
