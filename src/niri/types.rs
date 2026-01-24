use niri_ipc::Event;
use std::{collections::{BTreeMap, HashMap}, hash::Hash};

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct WorkspaceId(u64);

#[derive(Debug, Clone, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct WorkspaceIdx(u8);

impl From<&niri_ipc::Workspace> for WorkspaceIdx {
    fn from(ws: &niri_ipc::Workspace) -> Self {
        Self(ws.idx)
    }
}


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
            id: WorkspaceId(ws.id),
            monitor_id: ws.output.as_ref().map(|output| MonitorId(output.clone())),
            is_active: ws.is_active,
            is_urgent: ws.is_urgent,
            is_focused: ws.is_focused,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct WsMap(BTreeMap<WorkspaceIdx, Workspace>);

#[derive(Debug, Clone, PartialEq)]
struct WsIdxMap(HashMap<WorkspaceId, WorkspaceIdx>);

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
}

impl State {
    fn reduce(self, ev: Event) -> Self {
        match ev {
            Event::WorkspacesChanged { workspaces } => {
                let state = Self {
                    ws_map: WsMap(BTreeMap::new()),
                    ws_idx_map: WsIdxMap(HashMap::new()),
                };
                workspaces.iter().fold(state, |mut state, ws| {
                    let my_ws = Workspace::from(ws);
                    let idx = WorkspaceIdx::from(ws);

                    state.ws_idx_map.0.insert(my_ws.id.clone(), idx.clone());
                    state.ws_map.0.insert(idx.clone(), my_ws);

                    state
                })
            },
            _ => self,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Events {
    FocusedMonitor(MonitorId),
    FocusedWorkspace(WorkspaceId),
}
