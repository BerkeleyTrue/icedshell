use derive_more::{Display, From};
use iced::{Subscription, Task};
use niri_ipc::Event;
use std::{
    collections::{BTreeMap, HashMap},
    hash::Hash,
    iter::Iterator,
};
use tracing::info;

use crate::{
    config::MonitorId,
    feature::Service,
    niri::stream::{self, NiriStreamError},
};

#[derive(Debug, Clone, Eq, Hash, PartialEq, From, PartialOrd, Ord)]
pub struct WorkspaceId(u64);

#[derive(Debug, Clone, Eq, Hash, PartialEq, PartialOrd, Ord, From)]
pub struct WorkspaceIdx(u8);

impl WorkspaceIdx {
    pub fn get(&self) -> u8 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Workspace {
    pub id: WorkspaceId,
    pub idx: WorkspaceIdx,

    pub is_urgent: bool,
    pub is_active: bool,
    pub is_focused: bool,

    pub monitor_id: Option<MonitorId>,
    pub active_win_id: Option<WinId>,
}

impl<'a> From<&'a niri_ipc::Workspace> for Workspace {
    fn from(ws: &'a niri_ipc::Workspace) -> Self {
        Self {
            id: ws.id.into(),
            idx: ws.idx.into(),

            is_active: ws.is_active,
            is_urgent: ws.is_urgent,
            is_focused: ws.is_focused,

            monitor_id: ws.output.as_ref().map(MonitorId::from),
            active_win_id: ws.active_window_id.map(WinId),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct WsMap(BTreeMap<WorkspaceId, Workspace>);

#[derive(Debug, Clone, PartialEq, Eq, Hash, From)]
pub struct WinId(u64);

#[derive(Debug, Clone, PartialEq, Eq, Hash, From, PartialOrd, Ord, Default, Display)]
pub struct WinIdx(usize);

#[derive(Debug, Clone, PartialEq)]
pub struct Window {
    pub id: WinId,

    pub title: Option<String>,
    pub app_id: Option<String>,
    pub ws_id: Option<WorkspaceId>,
    pub col_idx: Option<WinIdx>,

    pub is_focus: bool,
    pub is_urgent: bool,
    pub is_floating: bool,
}

impl<'a> From<&'a niri_ipc::Window> for Window {
    fn from(niri_win: &'a niri_ipc::Window) -> Self {
        Self {
            id: niri_win.id.into(),
            title: niri_win.title.clone(),
            app_id: niri_win.app_id.clone(),
            ws_id: niri_win.workspace_id.map(|id| id.into()),
            col_idx: niri_win
                .layout
                .pos_in_scrolling_layout
                .map(|(idx, _)| WinIdx(idx)),

            is_urgent: niri_win.is_urgent,
            is_focus: niri_win.is_focused,
            is_floating: niri_win.is_floating,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct WinMap(HashMap<WinId, Window>);

#[derive(Debug, Clone, PartialEq, Default)]
pub struct State {
    pub ws_map: WsMap,
    pub win_map: WinMap,
}

impl State {
    pub fn iter_ws(&self) -> impl Iterator<Item = &Workspace> {
        self.ws_map.0.values()
    }
    pub fn iter_win(&self) -> impl Iterator<Item = &Window> {
        self.win_map.0.values()
    }

    pub fn get_win(&self, win_id: &WinId) -> Option<&Window> {
        self.win_map.0.get(win_id)
    }
    pub fn apply(&mut self, ev: Event) {
        match ev {
            Event::WorkspacesChanged { workspaces } => {
                self.ws_map = WsMap::default();
                workspaces.iter().for_each(move |niri_ws| {
                    let ws = Workspace::from(niri_ws);

                    self.ws_map.0.insert(ws.id.clone(), ws);
                });
            }
            Event::WorkspaceUrgencyChanged { id, urgent } => {
                let ws_id = WorkspaceId(id);
                if let Some(ws) = self.ws_map.0.get_mut(&ws_id) {
                    ws.is_urgent = urgent;
                };
            }
            Event::WorkspaceActivated { id, focused } => {
                let id = WorkspaceId(id);

                if let Some(ws) = self.ws_map.0.get_mut(&id) {
                    ws.is_focused = focused;
                    ws.is_active = true;
                    let mon_id = ws.monitor_id.clone();

                    self.ws_map.0.iter_mut().for_each(|(_, ws)| {
                        if ws.id == id {
                            return;
                        }

                        if focused {
                            ws.is_focused = false;
                        }

                        if ws.monitor_id == mon_id {
                            ws.is_active = false;
                        }
                    });
                }
            }
            Event::WorkspaceActiveWindowChanged {
                workspace_id,
                active_window_id,
            } => {
                let ws_id = WorkspaceId(workspace_id);
                let active_win_id = active_window_id.map(WinId);
                if let Some(ws) = self.ws_map.0.get_mut(&ws_id) {
                    ws.active_win_id = active_win_id;
                }
            }
            Event::WindowsChanged { windows } => {
                self.win_map = WinMap::default();
                windows.iter().for_each(move |niri_win| {
                    let win = Window::from(niri_win);
                    self.win_map.0.insert(win.id.clone(), win);
                });
            }
            Event::WindowLayoutsChanged { changes } => {
                changes.iter().for_each(move |(win_id, change)| {
                    let id = WinId(*win_id);

                    if let Some(win) = self.win_map.0.get_mut(&id)
                        && let Some((idx, _)) = change.pos_in_scrolling_layout
                    {
                        win.col_idx = Some(idx.into());
                    }
                });
            }
            Event::WindowUrgencyChanged { id, urgent } => {
                let id = WinId(id);
                if let Some(win) = self.win_map.0.get_mut(&id) {
                    win.is_urgent = urgent;
                }
            }
            Event::WindowFocusChanged { id } => {
                if let Some(id) = id.map(WinId) {
                    self.win_map.0.iter_mut().for_each(|(win_id, win)| {
                        win.is_focus = win_id == &id;
                    });
                }
            }
            Event::WindowOpenedOrChanged { window } => {
                let id = WinId(window.id);
                if window.is_focused {
                    self.win_map
                        .0
                        .iter_mut()
                        .for_each(|(_, win)| win.is_focus = false);
                }
                self.win_map.0.insert(id, Window::from(&window));
            }
            Event::WindowClosed { id } => {
                let id = WinId::from(id);
                self.win_map.0.remove(&id);
            }
            Event::WindowFocusTimestampChanged {
                id: _,
                focus_timestamp: _,
            }
            | Event::KeyboardLayoutsChanged {
                keyboard_layouts: _,
            }
            | Event::KeyboardLayoutSwitched { idx: _ }
            | Event::OverviewOpenedOrClosed { is_open: _ }
            | Event::ConfigLoaded { failed: _ }
            | Event::ScreenshotCaptured { path: _ } => (),
            // _ => self,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Event(Event),
    Error(NiriStreamError),
}

impl Service for State {
    type Message = Message;
    type Init = ();

    fn new(_input: Self::Init) -> Self {
        Self::default()
    }

    fn update(&mut self, message: Self::Message) -> iced::Task<Self::Message> {
        match message {
            Message::Event(event) => {
                self.apply(event);
                Task::none()
            }
            Message::Error(err) => {
                info!("Stream err: {err:}");
                Task::none()
            }
        }
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::run(stream::listen).map(|event| match event {
            Ok(ev) => Message::Event(ev),
            Err(err) => Message::Error(err),
        })
    }
}
