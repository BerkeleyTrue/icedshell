use derive_more::{Deref, DerefMut};
use iced::{Element, Subscription, Task, advanced::graphics::futures::MaybeSend, window};
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
