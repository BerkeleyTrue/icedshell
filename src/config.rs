use derive_more::{Display, From};

#[derive(Debug, Clone, Eq, Hash, PartialEq, From, Display)]
#[from(&String, String)]
pub struct MonitorId(pub String);

impl MonitorId {
    pub fn get(&self) -> String {
        self.0.clone()
    }
}
