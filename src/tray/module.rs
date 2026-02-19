use iced::{
    Task,
    widget::{container, text},
};

use crate::{
    feature::{CompWithProps, align_center},
    tray::service::TrayService,
};

pub struct TrayMod {}

#[derive(Debug, Clone)]
pub enum Message {}

pub struct Props<'a> {
    pub serv: &'a TrayService,
}

impl CompWithProps for TrayMod {
    type Message = Message;
    type Init = ();
    type Props<'a> = Props<'a>;

    fn new(_input: Self::Init) -> Self {
        Self {}
    }

    fn update(&mut self, _message: Self::Message) -> iced::Task<Self::Message> {
        Task::none()
    }

    fn view<'a>(&self, props: Self::Props<'a>) -> iced::Element<'_, Self::Message> {
        let count = props.serv.items.len();
        container(align_center!(text!("tray: {count}"))).into()
    }
}
