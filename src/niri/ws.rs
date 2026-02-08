use std::collections::BTreeMap;

use super::stream::{NiriEvent, NiriStreamError};
use crate::{
    config::MonitorId,
    feature::Comp,
    niri::{state, stream},
    theme::{self, AppTheme, Shade},
};
use iced::{
    Element, Length, Subscription, Task, border, padding,
    widget::{container, row},
};
use lucide_icons::Icon;
use tracing::{debug, info};

#[derive(Debug, Clone)]
pub enum Message {
    Event(NiriEvent),
    Stream(NiriStreamError),
}

type WSMap<'a> = BTreeMap<u8, &'a state::Workspace>;
type MonitorMap<'a> = BTreeMap<(u8, String), WSMap<'a>>;

pub struct Init {
    pub main_mon: MonitorId,
}

pub struct NiriWS {
    state: state::State,
    theme: AppTheme,
    main_mon: MonitorId,
}

impl Comp for NiriWS {
    type Message = Message;
    type Init = Init;

    fn new(init: Self::Init) -> Self {
        Self {
            state: state::State::new(),
            theme: theme::app_theme(),
            main_mon: init.main_mon,
        }
    }
    fn subscription(&self) -> Subscription<Message> {
        Subscription::run(stream::listen).map(|event| {
            debug!("niri event {event:?}");
            match event {
                Ok(ev) => Message::Event(ev),
                Err(err) => Message::Stream(err),
            }
        })
    }
    fn update(&mut self, message: Message) -> Task<Message> {
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
    fn view(&self) -> Element<'_, Self::Message> {
        let theme = &self.theme;
        let monitor_map = self
            .state
            .iter_ws()
            .fold(MonitorMap::new(), |mut mon_map, ws| {
                let monitor_id = ws
                    .monitor_id
                    .as_ref()
                    .map(|mon_id| mon_id.get())
                    .unwrap_or("NA".to_string());

                let priority = if monitor_id == self.main_mon.0 { 0 } else { 1 };
                let ws_map = mon_map.entry((priority, monitor_id)).or_default();

                ws_map.insert(ws.idx.get(), ws);

                mon_map
            });

        let niri_content = monitor_map.iter().map(|((priority, mon_id), mon)| {
            let pri = *priority;
            let mon_content = mon.iter().map(|(idx, ws)| {
                let mut icon = if ws.is_active {
                    Icon::CircleDot.widget()
                } else {
                    Icon::CircleDashed.widget()
                };

                if ws.is_focused {
                    icon = icon.color(theme.destructive(Shade::S500))
                }

                // ws
                container(icon)
                    .id(format!("ws-{idx}"))
                    .padding(padding::horizontal(theme.spacing().xs()))
                    .into()
            });

            // monitor
            container(row(mon_content))
                .id(format!("mon-{mon_id}"))
                .padding(padding::horizontal(theme.spacing().sm()))
                .style(move |_| container::Style {
                    background: Some(
                        theme
                            .neutral(if pri == 0 { Shade::S700 } else { Shade::S800 })
                            .into(),
                    ),
                    border: border::rounded(theme.radius().xs()),
                    ..Default::default()
                })
                .into()
        });
        let niri_row = if niri_content.len() > 0 {
            row(niri_content).spacing(theme.spacing().xs())
        } else {
            row([container(lucide_icons::Icon::CircleSlash2.widget()).into()])
        };

        container(niri_row)
            .padding(padding::horizontal(theme.spacing().sm()))
            .into()
    }
}
