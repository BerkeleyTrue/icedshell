use crate::{theme, widget_ext::ContainExt};
use iced::{
    Element, Padding, Subscription, Task, border, widget::{Button, button, container, row, text}
};
use stream::{NiriEvent, NiriStreamError};
use tracing::info;

mod state;
mod stream;

#[derive(Debug, Clone)]
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
    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::run(stream::listen).map(|event| {
            info!("niri event {event:?}");
            match event {
                Ok(ev) => Message::Event(ev),
                Err(err) => Message::Stream(err),
            }
        })
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
    pub fn view(&self) -> Element<'_, Message> {
        let ws = self.state.iter_ws().map(|_ws| {
            Button::new(text!("*"))
                .style(|_, _| button::Style {
                    background: Some(theme::BASE.into()),
                    text_color: theme::TEXT,
                    ..Default::default()
                })
                .into()
        });
        container(row(ws))
            .styl(container::Style {
                background: Some(theme::BASE.into()),
                border: border::rounded(border::left(180)),
                ..Default::default()
            })
            .padding(Padding::default().left(20))
            .into()
    }
}
