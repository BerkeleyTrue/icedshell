use derive_more::{Deref, DerefMut};
use iced::{
    Element, Length, Subscription, Task,
    advanced::graphics::futures::MaybeSend,
    widget::{Container, container, row},
    window,
};
use tracing::debug;

#[derive(Deref, DerefMut)]
pub struct FeatWindow<T>
where
    T: Feature,
{
    pub id: window::Id,

    #[deref]
    #[deref_mut]
    pub view: Box<T>,
}

pub trait Comp: Sized {
    type Message: MaybeSend + 'static;
    type Init;

    fn new<O: MaybeSend + 'static>(
        input: Self::Init,
        f: impl Fn(Self::Message) -> O + MaybeSend + 'static,
    ) -> (Self, Task<O>);

    /// helper: convert self to tuple with empty task
    fn to_tuple<O>(self) -> (Self, Task<O>) {
        (self, Task::none())
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::none()
    }

    #[allow(unused_variables)]
    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        Task::none()
    }

    fn view(&self) -> Element<'_, Self::Message>;
}

pub trait CompWithProps: Sized {
    type Message: MaybeSend + 'static;
    type Init;
    type Props<'a>;

    fn new<O: MaybeSend + 'static>(
        input: Self::Init,
        f: impl Fn(Self::Message) -> O + MaybeSend + 'static,
    ) -> (Self, Task<O>);

    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::none()
    }

    /// helper: convert self to tuple with empty task
    fn to_tuple<O>(self) -> (Self, Task<O>) {
        (self, Task::none())
    }

    #[allow(unused_variables)]
    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        Task::none()
    }

    fn view<'a>(&self, props: Self::Props<'a>) -> Element<'_, Self::Message>;
}

pub trait Service: Sized {
    type Message: MaybeSend + 'static;
    type Init;

    fn new<O: MaybeSend + 'static>(
        input: Self::Init,
        f: impl Fn(Self::Message) -> O + MaybeSend + 'static,
    ) -> (Self, Task<O>);

    /// helper: convert self to tuple with empty task
    fn to_tuple<O>(self) -> (Self, Task<O>) {
        (self, Task::none())
    }

    fn subscription(&self) -> Subscription<Self::Message>;

    #[allow(unused_variables)]
    fn update(&mut self, message: Self::Message) -> Task<Self::Message>;
}

pub trait Feature: Sized + Comp {
    type Settings;

    fn layer(&self) -> Self::Settings;

    fn open<O: MaybeSend + 'static>(
        input: Self::Init,
        f: impl Fn(window::Id, Self::Message) -> O + MaybeSend + 'static,
    ) -> (FeatWindow<Self>, Self::Settings, Task<O>) {
        let id = window::Id::unique();
        debug!("{id:}");
        let (comp, task) = Self::new(input, move |m| f(id, m));
        let settings = comp.layer();
        (
            FeatWindow {
                id,
                view: Box::new(comp),
            },
            settings,
            task,
        )
    }
}

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
