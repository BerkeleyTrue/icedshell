use iced::{
    Length, Task, time,
    widget::{container, text},
};
use iced_layershell::reexport::{self as layer};

use crate::{
    feature::{Comp, Feature},
    theme::CAT_THEME,
};

#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    Timeout,
}

pub struct Osd {}

impl Comp for Osd {
    type Message = Message;
    type Init = ();

    fn new<O: iced::advanced::graphics::futures::MaybeSend + 'static>(
        _input: Self::Init,
        f: impl Fn(Self::Message) -> O + iced::advanced::graphics::futures::MaybeSend + 'static,
    ) -> (Self, iced::Task<O>) {
        let timeout = Task::perform(
            tokio::time::sleep(tokio::time::Duration::from_secs(5)),
            |_| Message::Timeout,
        )
        .map(f);
        (Self {}, timeout)
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        time::every(time::Duration::from_millis(500)).map(|_| Message::Tick)
    }

    fn update(&mut self, message: Self::Message) -> iced::Task<Self::Message> {
        match message {
            Message::Tick => Task::none(),
            Message::Timeout => Task::none(),
        }
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        let _theme = &CAT_THEME;
        container(text!("Hello World")).width(Length::Fill).into()
    }
}

impl Feature for Osd {
    type Settings = layer::NewLayerShellSettings;
    fn layer(&self) -> Self::Settings {
        Self::Settings {
            size: Some((300, 300)),
            layer: layer::Layer::Overlay,
            anchor: layer::Anchor::empty(),
            margin: None,
            keyboard_interactivity: layer::KeyboardInteractivity::None,
            output_option: layer::OutputOption::None,
            exclusive_zone: None,
            events_transparent: false,
            namespace: Some("IcedOsd".to_owned()),
        }
    }
}
