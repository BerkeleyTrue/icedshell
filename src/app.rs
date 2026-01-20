use iced::{
    Element, Length, Subscription, Task, widget::{container, text}
};

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

    pub fn subscription() -> Subscription<Message> {
        niri::EventStream::new().listen().map(Message::noop)
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
