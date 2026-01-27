use iced::{Subscription, Task};
use stream::{NiriEvent, NiriStreamError};
use tracing::info;

mod state;
mod stream;

#[derive(Debug)]
pub enum Message {
    Event(NiriEvent),
    Stream(NiriStreamError),
}

pub struct NiriWS {
    state: state::State,
}

impl NiriWS {
    pub fn new() -> Self {
        Self {
            state: state::State::new(),
        }
    }
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Event(ev) => {
                self.state.apply(ev);
                Task::none()
            }
            _ => Task::none(),
        }
    }
    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::run(stream::listen).map(|event| {
            info!("niri event {event:?}");
            match event {
                Ok(ev) => Message::Event(ev),
                Err(err) => Message::Stream(err),
            }
        })
    }
    pub fn view() {}
}
