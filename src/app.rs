use iced::{
    Element, Length, Subscription, Task,
    widget::{container, text},
};
use tracing::info;

use crate::niri;

#[derive(Debug)]
pub enum Message {
    Noop,
    Niri(niri::Message)
}

pub struct App {
    niri: niri::NiriWS
}

impl App {
    pub fn new() -> Self {
        Self {
            niri: niri::NiriWS::new(),
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        self.niri.subscription().map(Message::Niri)
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            _ => Task::none(),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        container(text("Hello World")).center_y(Length::Fill).into()
    }
}
