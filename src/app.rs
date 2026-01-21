use iced::{
    Element, Length, Subscription, Task, widget::{container, text}
};
use tracing::info;

use crate::niri;

#[derive(Debug)]
pub enum Message {
    Noop,
}

pub struct App {}

impl App {
    pub fn new() -> Self {
        Self {}
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::run(niri::listen).filter_map(|event| {
            info!("niri event {event:?}");
            None
        })
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            _ => Task::none(),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        container(text("Hello World"))
            .center_y(Length::Fill)
            .into()
    }
}
