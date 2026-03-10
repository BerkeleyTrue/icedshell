use derive_more::Constructor;
use iced::Task;

use crate::fdo_icons::FdIcon;

#[derive(Debug, Clone, Constructor)]
pub struct Res<T> {
    pub id: T,
    pub icon: Option<FdIcon>,
    pub content: String,
    pub tooltip: Option<String>,
}

#[derive(Debug, Clone, Constructor, PartialEq, Eq, Default)]
pub struct Query {
    pub term: Option<String>,
    pub page: usize,
    pub limit: usize,
}

pub trait Modi {
    type Id;
    type Message;

    /// num of results
    #[must_use]
    fn len(&self) -> usize;

    /// the results of the last query
    #[must_use]
    fn res(&self) -> &Vec<Res<Self::Id>>;

    /// perform query and update internal results
    fn query(&mut self, query: Query) -> Task<Self::Message>;

    /// execute on selected results
    /// update internal state such as exec count or history
    fn exec(&mut self, id: &Self::Id) -> anyhow::Result<Task<Self::Message>>;
}
