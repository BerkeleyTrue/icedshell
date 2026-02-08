use iced::{
    Color, Task, padding,
    widget::{container, text},
};

use crate::{feature::CompWithProps, theme::app_theme};

#[derive(Debug, Clone)]
pub enum Message {}

pub struct Init {}

pub struct NiriWin {}

impl CompWithProps for NiriWin {
    type Props = Color;
    type Init = Init;
    type Message = Message;
    fn new(_input: Self::Init) -> Self {
        Self {}
    }
    // fn subscription(&self) -> iced::Subscription<Self::Message> {
    //
    // }
    fn update(&mut self, _message: Self::Message) -> iced::Task<Self::Message> {
        Task::none()
    }
    fn view(&self, color: Self::Props) -> iced::Element<'_, Self::Message> {
        let theme = app_theme();
        container(text!("win").color(color))
            .padding(padding::horizontal(theme.spacing().sm()))
            .into()
    }
}
