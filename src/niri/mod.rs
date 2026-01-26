use iced::Subscription;
use stream::NiriEvent;
use tracing::info;


mod stream;
mod state;

#[derive(Debug, Clone)]
pub enum Message {
    Noop(NiriEvent),
}

pub struct NiriWS {}


impl NiriWS {
    pub fn new() -> Self {
        Self {}
    }
    pub fn update(&mut self) {}
    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::run(stream::listen).filter_map(|event| {
            info!("niri event {event:?}");
            None
        })
    }
    pub fn view() {}
}
