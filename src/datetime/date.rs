use iced::{
    Color, Element, Subscription, Task, padding,
    time::{every, hours},
    widget::{container, row, text},
};
use time::{OffsetDateTime, format_description::BorrowedFormatItem};
use time_macros::format_description;

use crate::{
    divider::{Direction, Semi},
    feature::{CompWithProps, align_center},
    fira_fonts::TextExt,
    theme::{LAVENDER, app_theme},
    widget_ext::ContainExt,
};

const FORMAT_DATE: &[BorrowedFormatItem] =
    format_description!("[weekday repr:short] [month repr:short] [day]");
const FORMAT_WEEK: &[BorrowedFormatItem] = format_description!("Week [week_number]");

fn gen_date() -> String {
    OffsetDateTime::now_local()
        .ok()
        .and_then(|time| time.format(FORMAT_DATE).ok())
        .unwrap_or(String::from("Xxx Xxx Xx"))
}

fn gen_week() -> String {
    OffsetDateTime::now_local()
        .ok()
        .and_then(|time| time.format(FORMAT_WEEK).ok())
        .unwrap_or(String::from("XX"))
}

#[derive(Debug, Clone)]
pub enum Message {
    Tick,
}

pub struct Date {
    date: String,
    week: String,
}

impl CompWithProps for Date {
    type Message = Message;
    type Init = ();
    type Props<'a> = Color;

    fn new(_init: Self::Init) -> Self {
        Self {
            date: gen_date(),
            week: gen_week(),
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        every(hours(12)).map(|_| Message::Tick)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {
                self.date = gen_date();
                self.week = gen_week();
                Task::none()
            }
        }
    }

    fn view(&self, color: Color) -> Element<'_, Message> {
        let theme = app_theme();

        let time = align_center!(text(&self.date).color(color).bold())
            .padding(padding::right(theme.spacing().sm()));

        let div = Semi::new(LAVENDER, Direction::Left, theme.spacing().xl());

        let date = align_center!(text(&self.week).color(color))
            .background(LAVENDER)
            .padding(padding::horizontal(theme.spacing().sm()));

        align_center!(row![time, div, date]).into()
    }
}
