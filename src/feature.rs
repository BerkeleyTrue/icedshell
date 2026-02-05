use std::ops::{Deref, DerefMut};

use iced::{Element, Subscription, Task, window};
use iced_layershell::reexport::NewLayerShellSettings;
use tracing::debug;

use crate::layershell;

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
    type InnerMessage;
    type Init;
    type Props;

    fn new(input: Self::Init) -> Self;

    fn subscription(&self) -> Subscription<Self::InnerMessage> {
        Subscription::none()
    }

    fn update(&mut self, message: Self::InnerMessage) -> Task<Self::InnerMessage>;
    fn view(&self, props: Self::Props) -> Element<'_, Self::InnerMessage>;
}

pub trait Feature: Sized + Comp {
    fn layer(&self) -> NewLayerShellSettings;

    fn is_animating(&self) -> bool {
        false
    }

    /// open window, consuming self
    fn open(self) -> (Window<Self>, Task<layershell::Message>) {
        let id = window::Id::unique();
        debug!("{id:}");
        let settings = self.layer();

        (
            Window { id, view: self },
            Task::done(layershell::Message::NewLayerShell { settings, id }),
        )
    }
}
