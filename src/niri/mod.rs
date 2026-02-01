use std::collections::BTreeMap;

use crate::theme::{self, AppTheme, Shade};
use iced::{
    Element, Padding, Subscription, Task, border, padding,
    widget::{container, row, text},
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
    theme: AppTheme<'static>,
}

impl NiriWS {
    pub fn new() -> Self {
        Self {
            state: state::State::new(),
            theme: theme::app_theme(),
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
        let theme = &self.theme;
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
                    .padding(padding::horizontal(theme.spacing().sm()))
                    .into()
            });
            container(row(mon_content))
                .id(format!("mon-{mon_id}"))
                .padding(padding::horizontal(theme.spacing().sm()))
                .style(|_| container::Style {
                    background: Some(theme.neutral(Shade::S800).into()),
                    border: border::rounded(theme.radius().xs()),
                    ..Default::default()
                })
                .into()
        });
        container(row(niri_content).padding(padding::vertical(theme.spacing().xxs())).spacing(theme.spacing().xs()))
            .style(|_| container::Style {
                background: Some(theme.background().into()),
                border: border::rounded(border::left(theme.radius().xl())),
                ..Default::default()
            })
            .padding(Padding::default().left(theme.spacing().lg()))
            .into()
    }
}
