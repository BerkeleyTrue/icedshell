use derive_more::From;
use niri_ipc::Event;
use std::{
    collections::{BTreeMap, HashMap},
    hash::Hash,
    iter::{Iterator},
};

#[derive(Debug, Clone, Eq, Hash, PartialEq, From, PartialOrd, Ord)]
pub struct WorkspaceId(u64);

#[derive(Debug, Clone, Eq, Hash, PartialEq, PartialOrd, Ord, From)]
pub struct WorkspaceIdx(u8);

#[derive(Debug, Clone, Eq, Hash, PartialEq, From)]
pub struct MonitorId(String);

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

            monitor_id: ws.output.as_ref().map(|output| MonitorId(output.clone())),
            active_win_id: ws.active_window_id.map(WinId),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct WsMap(BTreeMap<WorkspaceId, Workspace>);

#[derive(Debug, Clone, PartialEq, Eq, Hash, From)]
pub struct WinId(u64);

#[derive(Debug, Clone, PartialEq, Eq, Hash, From, PartialOrd, Ord)]
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
    pub fn new() -> Self {
        Self::default()
    }

    pub fn iter_ws(&self) -> impl Iterator<Item = &Workspace> {
        self.ws_map.0.iter().map(|(_, ws)| ws)
    }
    pub fn iter_win(&self) -> impl Iterator<Item = &Window> {
        self.win_map.0.iter().map(|(_, win)| win)
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
                self.ws_map.0.get_mut(&ws_id).map(move |ws| {
                    ws.is_urgent = urgent;
                    ws
                });
            }
            Event::WorkspaceActivated { id, focused } => {
                let id = WorkspaceId(id);
                self.ws_map.0.iter_mut().for_each(|(_, ws)| {
                    ws.is_active = false;
                    if focused {
                        ws.is_focused = false;
                    }
                });
                self.ws_map.0.get_mut(&id).map(|ws| {
                    ws.is_focused = focused;
                    ws.is_active = true;
                });
            }
            Event::WorkspaceActiveWindowChanged {
                workspace_id,
                active_window_id,
            } => {
                let ws_id = WorkspaceId(workspace_id);
                let active_win_id = active_window_id.map(WinId);
                self.ws_map.0.get_mut(&ws_id).map(move |ws| {
                    ws.active_win_id = active_win_id;
                    ws
                });
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
                    self.win_map.0.get_mut(&id).map(|win| {
                        if let Some((idx, _)) = change.pos_in_scrolling_layout {
                            win.col_idx = Some(idx.into());
                        }
                    });
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
