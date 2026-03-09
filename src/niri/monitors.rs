use std::collections::HashMap;

use iced::{Subscription, Task, advanced::graphics::futures::MaybeSend};
use itertools::Itertools;

use crate::{
    config::MonitorId,
    feature::Service,
    niri::stream::{self, NiriEvent},
};

#[derive(Debug, Clone)]
pub enum Message {
    MonitorsChanged(
        /// current monito
        Option<MonitorId>,
        /// map of ws id to monitor
        HashMap<u64, MonitorId>,
    ),
    MonitorActive(u64),
}
#[derive(Debug, Clone)]
pub struct MonitorsServ {
    cur_monitor: Option<MonitorId>,
    map: HashMap<u64, MonitorId>,
}

impl MonitorsServ {
    pub fn len(&self) -> usize {
        self.map.values().unique().collect::<Vec<_>>().len()
    }
    pub fn iter(&self) -> impl Iterator<Item = &MonitorId> {
        self.map.values().unique()
    }
    pub fn cur_monitor(&self) -> Option<&MonitorId> {
        self.cur_monitor.as_ref()
    }
}

impl Service for MonitorsServ {
    type Message = Message;
    type Init = ();
    fn new<O: MaybeSend + 'static>(
        _input: Self::Init,
        _f: impl Fn(Self::Message) -> O + MaybeSend + 'static,
    ) -> (Self, Task<O>) {
        Self {
            cur_monitor: None,
            map: HashMap::new(),
        }
        .to_tuple()
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        Subscription::run(stream::listen)
            .filter_map(|res| res.ok())
            .filter_map(|event| match event {
                NiriEvent::WorkspacesChanged { mut workspaces } => {
                    let map = workspaces
                        .iter()
                        .filter_map(|ws| {
                            ws.output
                                .clone()
                                .map(MonitorId::from)
                                .map(move |monitor_id| (ws.id, monitor_id))
                        })
                        .fold(HashMap::new(), |mut acc, (id, monitor)| {
                            acc.insert(id, monitor);
                            acc
                        });

                    let cur_monitor = workspaces
                        .iter_mut()
                        .find(|ws| ws.is_focused)
                        .and_then(|ws| ws.output.clone())
                        .map(MonitorId::from);

                    Some(Message::MonitorsChanged(cur_monitor, map))
                }
                NiriEvent::WorkspaceActivated { id, focused: _ } => {
                    Some(Message::MonitorActive(id))
                }
                _ => None,
            })
    }

    fn update(&mut self, message: Self::Message) -> iced::Task<Self::Message> {
        match message {
            Message::MonitorsChanged(cur_mon, map) => {
                self.map = map;
                self.cur_monitor = cur_mon;
                Task::none()
            }
            Message::MonitorActive(id) => {
                self.cur_monitor = self.map.get(&id).cloned();
                Task::none()
            }
        }
    }
}
