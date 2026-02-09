use iced::{
    Color, Subscription, Task, padding,
    widget::{container, text},
};
use tracing::info;

use super::{
    state,
    stream::{NiriEvent, NiriStreamError, listen},
};
use crate::{config::MonitorId, feature::CompWithProps, theme::app_theme};

#[derive(Debug, Clone)]
pub enum Message {
    Event(NiriEvent),
    Stream(NiriStreamError),
}

pub struct Init {
    pub monitor_id: MonitorId,
}

pub struct NiriWin {
    state: state::State,
    mon: MonitorId,
}

impl CompWithProps for NiriWin {
    type Props = Color;
    type Init = Init;
    type Message = Message;
    fn new(input: Self::Init) -> Self {
        Self {
            mon: input.monitor_id,
            state: state::State::new(),
        }
    }
    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::run(listen).map(|event| match event {
            Ok(ev) => Message::Event(ev),
            Err(err) => Message::Stream(err),
        })
    }
    fn update(&mut self, message: Self::Message) -> iced::Task<Self::Message> {
        match message {
            Message::Event(ev) => {
                self.state.apply(ev);
                Task::none()
            }
            Message::Stream(err) => {
                info!("Stream err: {err:}");
                Task::none()
            }
        }
    }
    fn view(&self, color: Self::Props) -> iced::Element<'_, Self::Message> {
        let theme = app_theme();
        let mut title = self
            .state
            .iter_ws()
            .find(move |ws| ws.monitor_id == Some(self.mon.clone()) && ws.is_active)
            .and_then(|ws| ws.active_win_id.as_ref())
            .and_then(|win_id| self.state.get_win(win_id))
            .and_then(|win| win.title.clone())
            .unwrap_or("()".to_string());

        if title.len() >= 9 {
            title = format!("{}...", title.chars().take(9).collect::<String>());
        }

        container(text!("{title}").color(color))
            .padding(padding::horizontal(theme.spacing().sm()))
            .into()
    }
}
