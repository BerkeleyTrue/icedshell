use std::collections::HashMap;

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct MonitorId(i32);
#[derive(Debug, Clone, PartialEq)]
pub struct Monitor {
    pub id: MonitorId,
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct WorkspaceId(i32);
#[derive(Debug, Clone, PartialEq)]
pub struct Workspace {
    pub id: WorkspaceId,
}

#[derive(Debug, Clone, PartialEq)]
pub struct State {
    pub monitors: HashMap<MonitorId, Monitor>,
    pub workspaces: HashMap<WorkspaceId, Workspace>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Events {
    FocusedMonitor(MonitorId),
    FocusedWorkspace(WorkspaceId),

}
