use iced::{
    Color, Element, Subscription, Task,
    time::{every, milliseconds},
    widget::text,
};
use time::{OffsetDateTime, format_description::BorrowedFormatItem};
use time_macros::format_description;

const FORMAT: &[BorrowedFormatItem] = format_description!("[hour]:[minute]:[second]");

fn gen_time() -> String {
    OffsetDateTime::now_local()
        .ok()
        .and_then(|time| time.format(FORMAT).ok())
        .unwrap_or(String::from("00:00:00"))
}

#[derive(Debug, Clone)]
pub enum Message {
    Tick,
}

pub struct Clock {
    time: String,
}

impl Clock {
    pub fn new() -> Self {
        Self { time: gen_time() }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        every(milliseconds(500)).map(|_| Message::Tick)
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {
                self.time = gen_time();
                Task::none()
            }
        }
    }

    pub fn view(&self, color: impl Into<Color>) -> Element<'_, Message> {
        let time = self.time.clone();
        text(time).color(color).into()
    }
}
