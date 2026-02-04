use derive_more::From;

#[derive(Debug, Clone, Eq, Hash, PartialEq, From)]
#[from(&String)]
pub struct MonitorId(pub String);

impl MonitorId {
    pub fn get(&self) -> String {
        self.0.clone()
    }
}

#[derive(Debug, Clone)]
pub struct Bar {
    pub output: MonitorId
}

#[derive(Debug, Clone)]
pub struct Config {
    pub bars: Vec<Bar>
}
