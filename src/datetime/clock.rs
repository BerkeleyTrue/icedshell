use iced::{
    Color, Element, Subscription, Task,
    time::{every, milliseconds},
    widget::text,
};
use time::{OffsetDateTime, format_description::BorrowedFormatItem};
use time_macros::format_description;

use crate::feature::CompWithProps;

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

impl CompWithProps for Clock {
    type Message = Message;
    type Init = ();
    type Props<'a> = Color;

    fn new(_init: Self::Init) -> Self {
        Self { time: gen_time() }
    }

    fn subscription(&self) -> Subscription<Message> {
        every(milliseconds(500)).map(|_| Message::Tick)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {
                self.time = gen_time();
                Task::none()
            }
        }
    }

    fn view(&self, color: Color) -> Element<'_, Message> {
        let time = self.time.clone();
        text(time).color(color).into()
    }
}
