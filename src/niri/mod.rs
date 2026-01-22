use iced::Subscription;
use stream::NiriEvent;
use tracing::info;


mod stream;
mod types;

pub enum Message {
    Noop(NiriEvent),
}

pub struct NiriWS {}


impl NiriWS {
    fn new() -> Self {
        Self {}
    }
    fn update(&mut self) {}
    fn subscription(&self) -> Subscription<Message> {
        Subscription::run(stream::listen).filter_map(|event| {
            info!("niri event {event:?}");
            None
        })
    }
    fn view() {}
}
