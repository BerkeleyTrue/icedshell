use iced::{Subscription, Task};
use itertools::Itertools;

use crate::{config::MonitorId, feature::Service, niri::stream};

#[derive(Debug, Clone)]
pub enum Message {
    UpdateMonitors(Vec<MonitorId>),
}
#[derive(Debug, Clone)]
pub struct MonitorsServ {
    monitors: Vec<MonitorId>,
}

impl MonitorsServ {
    pub fn len(&self) -> usize {
        self.monitors.len()
    }
    pub fn iter(&self) -> std::slice::Iter<'_, MonitorId> {
        self.monitors.iter()
    }
}

impl Service for MonitorsServ {
    type Message = Message;
    type Init = ();
    fn new(_input: Self::Init) -> (Self, Task<Self::Message>) {
        Self { monitors: vec![] }.to_tuple()
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        Subscription::run(stream::listen)
            .filter_map(|res| res.ok())
            .filter_map(|event| match event {
                stream::NiriEvent::WorkspacesChanged { workspaces } => {
                    let outputs: Vec<MonitorId> = workspaces
                        .iter()
                        .filter_map(|ws| ws.output.clone())
                        .map(MonitorId::from)
                        .unique()
                        .collect();

                    Some(Message::UpdateMonitors(outputs))
                }
                _ => None,
            })
    }

    fn update(&mut self, message: Self::Message) -> iced::Task<Self::Message> {
        match message {
            Message::UpdateMonitors(monitors) => {
                self.monitors = monitors;
                Task::none()
            }
        }
    }
}
