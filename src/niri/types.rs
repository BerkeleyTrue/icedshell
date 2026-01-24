use derive_more::From;
use niri_ipc::Event;
use std::{
    collections::{BTreeMap, HashMap},
    hash::Hash,
};

#[derive(Debug, Clone, Eq, Hash, PartialEq, From)]
pub struct WorkspaceId(u64);

#[derive(Debug, Clone, Eq, Hash, PartialEq, PartialOrd, Ord, From)]
pub struct WorkspaceIdx(u8);

#[derive(Debug, Clone, PartialEq)]
pub struct Workspace {
    pub id: WorkspaceId,
    pub monitor_id: Option<MonitorId>,
    pub is_urgent: bool,
    pub is_active: bool,
    pub is_focused: bool,
}

impl<'a> From<&'a niri_ipc::Workspace> for Workspace {
    fn from(ws: &'a niri_ipc::Workspace) -> Self {
        Self {
            id: ws.id.into(),
            monitor_id: ws.output.as_ref().map(|output| MonitorId(output.clone())),
            is_active: ws.is_active,
            is_urgent: ws.is_urgent,
            is_focused: ws.is_focused,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WsMap(BTreeMap<WorkspaceIdx, Workspace>);

#[derive(Debug, Clone, PartialEq)]
pub struct WsIdxMap(HashMap<WorkspaceId, WorkspaceIdx>);

#[derive(Debug, Clone, PartialEq, Eq, Hash, From)]
pub struct WinId(u64);

#[derive(Debug, Clone, PartialEq)]
pub struct Window {
    pub id: WinId,
    pub title: Option<String>,
    pub app_id: Option<String>,
    pub ws_id: Option<WorkspaceId>,
    pub is_focus: bool,
    pub is_urgent: bool,
    pub is_floating: bool,
}

impl<'a> From<&'a niri_ipc::Window> for Window {
    fn from(value: &'a niri_ipc::Window) -> Self {
        Self {
            id: value.id.into(),
            title: value.title.clone(),
            ws_id: value.workspace_id.map(|id| id.into()),
            app_id: value.app_id.clone(),
            is_urgent: value.is_urgent,
            is_focus: value.is_focused,
            is_floating: value.is_floating,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WinMap(HashMap<WinId, Window>);

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct MonitorId(String);
#[derive(Debug, Clone, PartialEq)]
pub struct Monitor {
    pub id: MonitorId,
}

#[derive(Debug, Clone, PartialEq)]
pub struct State {
    // pub outputs: HashMap<MonitorId, Monitor>,
    pub ws_map: WsMap,
    pub ws_idx_map: WsIdxMap,
    pub win_map: WinMap,
}

impl State {
    fn reduce(self, ev: Event) -> Self {
        match ev {
            Event::WorkspacesChanged { workspaces } => {
                let state = Self {
                    ws_map: WsMap(BTreeMap::new()),
                    ws_idx_map: WsIdxMap(HashMap::new()),
                    win_map: self.win_map,
                };
                workspaces.iter().fold(state, |mut state, ws| {
                    let my_ws = Workspace::from(ws);
                    let idx = WorkspaceIdx::from(ws.idx);

                    state.ws_idx_map.0.insert(my_ws.id.clone(), idx.clone());
                    state.ws_map.0.insert(idx.clone(), my_ws);

                    state
                })
            }
            Event::WindowsChanged { windows } => {
                let state = Self {
                    ws_map: self.ws_map,
                    ws_idx_map: self.ws_idx_map,
                    win_map: WinMap(HashMap::new()),
                };
                windows.iter().fold(state, |mut state, niri_win| {
                    let win = Window::from(niri_win);
                    state.win_map.0.insert(win.id.clone(), win);
                    state
                })
            }
            _ => self,
        }
    }
}
