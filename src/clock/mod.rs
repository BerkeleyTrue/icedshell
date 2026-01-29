use iced::{
    Color, Element, Subscription, Task,
    time::{every, milliseconds},
    widget::text,
};
use time::OffsetDateTime;
use time_macros::format_description;

#[derive(Debug, Clone)]
pub enum Message {
    Tick,
}

pub struct Clock {
    time: String,
    // format:
}

impl Clock {
    pub fn new() -> Self {
        // let format = format_description!("[hour]:[minute]:[second]");
        Self {
            time: String::default(),
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        every(milliseconds(500)).map(|_| Message::Tick)
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {
                OffsetDateTime::now_local()
                    .ok()
                    .and_then(|time| {
                        let format = format_description!("[hour]:[minute]:[second]");
                        time.format(&format).ok()
                    })
                    .map(move |time| {
                        self.time = time;
                        ()
                    });

                Task::none()
            }
        }
    }

    pub fn view(&self, color: impl Into<Color>) -> Element<'_, Message> {
        let time = self.time.clone();
        text(time).color(color).into()
    }
}
