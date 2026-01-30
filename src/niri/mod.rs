use std::collections::{BTreeMap, HashMap};

use crate::{niri::state::MonitorId, theme};
use iced::{
    Element, Padding, Subscription, Task, border, padding,
    widget::{Button, Container, button, container, row, text},
};
use stream::{NiriEvent, NiriStreamError};
use tracing::debug;

mod state;
mod stream;

#[derive(Debug, Clone)]
pub enum Message {
    Event(NiriEvent),
    Stream(NiriStreamError),
}

type WSMap<'a> = BTreeMap<u8, &'a state::Workspace>;
type MonitorMap<'a> = BTreeMap<String, WSMap<'a>>;

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
            debug!("niri event {event:?}");
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
        let monitor_map = self
            .state
            .iter_ws()
            .fold(MonitorMap::new(), |mut mon_map, ws| {
                let monitor_id = ws
                    .monitor_id
                    .as_ref()
                    .map(|mon_id| mon_id.get())
                    .or(Some("NA".to_string()))
                    .unwrap();

                let ws_map = mon_map.entry(monitor_id).or_default();

                ws_map.insert(ws.idx.get(), ws);

                mon_map
            });
        let niri_content = monitor_map.iter().map(|(mon_id, mon)| {
            let mon_content = mon.iter().map(|(idx, _ws)| {
                container(text!("*"))
                    .id(format!("ws-{idx}"))
                    .padding(padding::horizontal(4))
                    .into()
            });
            container(row(mon_content))
                .id(format!("mon-{mon_id}"))
                .padding(padding::horizontal(10))
                .style(|_| container::Style {
                    background: Some(theme::BLUE.into()),
                    border: border::rounded(90),
                    ..Default::default()
                })
                .into()
        });
        container(row(niri_content).padding(padding::vertical(3)).spacing(4))
            .style(|_| container::Style {
                background: Some(theme::BASE.into()),
                border: border::rounded(border::left(180)),
                ..Default::default()
            })
            .padding(Padding::default().left(20))
            .into()
    }
}
