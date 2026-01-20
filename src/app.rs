use iced::{
    Element, Length, Task,
    widget::{container, text},
};

#[derive(Debug)]
pub enum Message {
    Noop,
}

pub struct App {}

impl App {
    pub fn new() -> Self {
        Self {}
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
