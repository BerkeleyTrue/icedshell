use iced::{
    Element, Task,
    widget::{container, row},
};
use lucide_icons::Icon;

use crate::{
    feature::{CompWithProps, align_center},
    theme::app_theme,
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
        let theme = app_theme();
        // let count = props.serv.items.len();
        let items: Vec<Element<'_, Self::Message>> = props
            .serv
            .items
            .iter()
            .map(|sn_item| {
                sn_item
                    .icon
                    .as_ref()
                    .map(|icon| icon.elem(theme.spacing().xl()))
                    .unwrap_or(Icon::Dot.widget().into())
            })
            .collect();
        container(align_center!(row(items))).into()
    }
}
