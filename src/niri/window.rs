use iced::{
    Task,
    widget::{container, text},
};

use crate::feature::Comp;

#[derive(Debug, Clone)]
pub enum Message {}

pub struct Init {}

pub struct NiriWin {}

impl Comp for NiriWin {
    type Message = Message;
    type Init = Init;
    fn new(_input: Self::Init) -> Self {
        Self {}
    }
    // fn subscription(&self) -> iced::Subscription<Self::Message> {
    //
    // }
    fn update(&mut self, _message: Self::Message) -> iced::Task<Self::Message> {
        Task::none()
    }
    fn view(&self) -> iced::Element<'_, Self::Message> {
        container(text!("win")).into()
    }
}
