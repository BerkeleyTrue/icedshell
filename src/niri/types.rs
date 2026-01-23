use niri_ipc::Event;
use std::{collections::HashMap, hash::Hash};

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct WorkspaceId(u64);

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
struct WorkspaceMap(HashMap<WorkspaceId, Workspace>);

impl<'a> FromIterator<&'a niri_ipc::Workspace> for WorkspaceMap {
    fn from_iter<T: IntoIterator<Item = &'a niri_ipc::Workspace>>(iter: T) -> Self {
        let map: WorkspaceMap = WorkspaceMap(HashMap::new());
        return iter.into_iter().fold(map, |mut map, workspace| {
            let ws = Workspace::from(workspace);
            map.0.insert(ws.id.clone(), ws);
            map
        });
    }
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct MonitorId(String);
#[derive(Debug, Clone, PartialEq)]
pub struct Monitor {
    pub id: MonitorId,
}

#[derive(Debug, Clone, PartialEq)]
pub struct State {
    // pub outputs: HashMap<MonitorId, Monitor>,
    pub workspaces: WorkspaceMap,
}

impl State {
    fn reduce(self, ev: Event) -> Self {
        match ev {
            Event::WorkspacesChanged { workspaces } => {
                let workspaces: WorkspaceMap = workspaces.iter().collect();
                Self {
                    workspaces: workspaces,
                }
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
