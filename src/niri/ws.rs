use std::collections::BTreeMap;

use crate::{
    config::MonitorId,
    feature::CompWithProps,
    niri::state,
    theme::{AppTheme, Shade, app_theme},
};
use iced::{
    Element, Task, border, padding,
    widget::{container, row},
};
use lucide_icons::Icon;

#[derive(Debug, Clone)]
pub enum Message {}

type WSMap<'a> = BTreeMap<u8, &'a state::Workspace>;
type MonitorMap<'a> = BTreeMap<(u8, String), WSMap<'a>>;

pub struct Init {
    pub main_mon: MonitorId,
}

pub struct Props<'a> {
    pub state: &'a state::State,
}

pub struct NiriWS {
    main_mon: MonitorId,
    theme: AppTheme,
}

impl CompWithProps for NiriWS {
    type Message = Message;
    type Init = Init;
    type Props<'a> = Props<'a>;

    fn new(init: Self::Init) -> Self {
        Self {
            theme: app_theme(),
            main_mon: init.main_mon,
        }
    }
    fn update(&mut self, _message: Message) -> Task<Message> {
        Task::none()
    }
    fn view<'a>(&self, Props { state }: Self::Props<'a>) -> Element<'_, Self::Message> {
        let theme = &self.theme;
        let monitor_map = state.iter_ws().fold(MonitorMap::new(), |mut mon_map, ws| {
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
                    Icon::Circle.widget()
                } else {
                    Icon::CircleDashed.widget()
                };

                if ws.is_focused {
                    icon = Icon::CircleDot
                        .widget()
                        .color(theme.destructive(Shade::S500))
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
                            .clone()
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
