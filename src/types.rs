use derive_more::{Constructor, Deref, DerefMut, Display, From};

#[derive(Debug, Clone, Eq, Hash, PartialEq, From, Display, Deref, DerefMut, Constructor)]
#[from(&String, String)]
pub struct MonitorId(String);

impl MonitorId {
    pub fn inner(&self) -> &str {
        &self.0
    }
}
