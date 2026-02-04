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

pub trait Comp: Sized {
    type InnerMessage;

    fn subscriptions(&self) -> Subscription<Self::InnerMessage> {
        Subscription::none()
    }

    fn update(&mut self, message: Self::InnerMessage) -> Task<Self::InnerMessage>;
    fn view(&self) -> impl Into<Element<'_, Self::InnerMessage>>;
}

pub trait Feature: Sized {
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
