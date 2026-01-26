use derive_more::From;
use niri_ipc::Event;
use std::{
    collections::{BTreeMap, HashMap},
    hash::Hash,
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
    fn reduce(self, ev: Event) -> Self {
        match ev {
            Event::WorkspacesChanged { workspaces } => {
                let state = Self {
                    ws_map: WsMap::default(),
                    ..self
                };
                workspaces.iter().fold(state, |mut state, niri_ws| {
                    let ws = Workspace::from(niri_ws);

                    state.ws_map.0.insert(ws.id.clone(), ws);

                    state
                })
            }
            Event::WorkspaceUrgencyChanged { id, urgent } => {
                let ws_id = WorkspaceId(id);
                let mut ws_map = self.ws_map;
                ws_map.0.get_mut(&ws_id).map(move |ws| {
                    ws.is_urgent = urgent;
                    ws
                });
                Self { ws_map, ..self }
            }
            Event::WorkspaceActivated { id, focused } => {
                let id = WorkspaceId(id);
                let mut ws_map = self.ws_map;
                ws_map.0.iter_mut().for_each(|(_, ws)| {
                    ws.is_active = false;
                    if focused {
                        ws.is_focused = false;
                    }
                });
                ws_map.0.get_mut(&id).map(|ws| {
                    ws.is_focused = focused;
                    ws.is_active = true;
                });
                Self { ws_map, ..self }
            }
            Event::WorkspaceActiveWindowChanged {
                workspace_id,
                active_window_id,
            } => {
                let ws_id = WorkspaceId(workspace_id);
                let active_win_id = active_window_id.map(WinId);
                let mut ws_map = self.ws_map;
                ws_map.0.get_mut(&ws_id).map(move |ws| {
                    ws.active_win_id = active_win_id;
                    ws
                });
                Self { ws_map, ..self }
            }
            Event::WindowsChanged { windows } => {
                let state = Self {
                    win_map: WinMap::default(),
                    ..self
                };
                windows.iter().fold(state, |mut state, niri_win| {
                    let win = Window::from(niri_win);
                    state.win_map.0.insert(win.id.clone(), win);
                    state
                })
            }
            Event::WindowLayoutsChanged { changes } => {
                let win_map = changes
                    .iter()
                    .fold(self.win_map, |mut win_map, (win_id, change)| {
                        let id = WinId(*win_id);
                        win_map.0.get_mut(&id).map(|win| {
                            if let Some((idx, _)) = change.pos_in_scrolling_layout {
                                win.col_idx = Some(idx.into());
                            }
                        });
                        win_map
                    });
                Self { win_map, ..self }
            }
            Event::WindowUrgencyChanged { id, urgent } => {
                let id = WinId(id);
                let mut win_map = self.win_map;
                if let Some(win) = win_map.0.get_mut(&id) {
                    win.is_urgent = urgent;
                }
                Self { win_map, ..self }
            }
            Event::WindowFocusChanged { id } => {
                if let Some(id) = id.map(WinId) {
                    let mut win_map = self.win_map;
                    win_map.0.iter_mut().for_each(|(win_id, win)| {
                        win.is_focus = win_id == &id;
                    });
                    Self { win_map, ..self }
                } else {
                    self
                }
            }
            Event::WindowOpenedOrChanged { window } => {
                let mut win_map = self.win_map;

                let id = WinId(window.id);
                if window.is_focused {
                    win_map
                        .0
                        .iter_mut()
                        .for_each(|(_, win)| win.is_focus = false);
                }
                win_map.0.insert(id, Window::from(&window));
                Self { win_map, ..self }
            }
            Event::WindowClosed { id } => {
                let id = WinId::from(id);
                let mut win_map = self.win_map;
                win_map.0.remove(&id);
                Self { win_map, ..self }
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
            | Event::ScreenshotCaptured { path: _ } => self,
            // _ => self,
        }
    }
}
