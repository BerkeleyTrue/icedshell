use iced::{
    Element, Length, Subscription, Task, border, padding, widget::{container}
};
// use tracing::info;

use crate::{theme as my_theme, niri};

#[derive(Debug)]
pub enum Message {
    Niri(niri::Message),
}

pub struct App {
    niri: niri::NiriWS,
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
            Message::Niri(message) => self.niri.update(message).map(Message::Niri),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        container(self.niri.view().map(Message::Niri))
            .style(|theme| {
                container::Style {
                    background: Some(my_theme::MAUVE.into()),
                    ..Default::default()
                }
            })
            .padding(padding::left(20.0))
            .center_y(Length::Fill)
            .into()
    }
}
