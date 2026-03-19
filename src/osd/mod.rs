use iced::{
    Length, Task, border,
    widget::{container, text},
};
use iced_layershell::reexport::{self as layer, OutputOption};

use crate::{
    feature::{Comp, Feature},
    theme::CAT_THEME,
    types::MonitorId,
};

#[derive(Debug, Clone)]
pub struct Init {
    pub monitor: Option<MonitorId>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Timeout,
}

pub struct Osd {
    monitor: Option<MonitorId>,
}

impl Comp for Osd {
    type Message = Message;
    type Init = Init;

    fn new<O: iced::advanced::graphics::futures::MaybeSend + 'static>(
        input: Self::Init,
        f: impl Fn(Self::Message) -> O + iced::advanced::graphics::futures::MaybeSend + 'static,
    ) -> (Self, iced::Task<O>) {
        let timeout = Task::perform(
            tokio::time::sleep(tokio::time::Duration::from_millis(650)),
            |_| Message::Timeout,
        )
        .map(f);
        (
            Self {
                monitor: input.monitor,
            },
            timeout,
        )
    }

    // fn subscription(&self) -> iced::Subscription<Self::Message> {
    //     Subscription::none()
    // }

    fn update(&mut self, message: Self::Message) -> iced::Task<Self::Message> {
        match message {
            Message::Timeout => Task::none(),
        }
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        let theme = &CAT_THEME;
        let spacing = theme.spacing();
        container(text!("Hello World").color(theme.text_color()))
            .style(move |_| container::Style {
                background: Some(theme.background().into()),
                border: border::color(theme.teal())
                    .width(spacing.xxs())
                    .rounded(theme.radius().md()),
                ..Default::default()
            })
            .width(Length::Fill)
            .into()
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
            output_option: self
                .monitor
                .as_ref()
                .map(|monitor| OutputOption::OutputName(monitor.inner().to_owned()))
                .unwrap_or(OutputOption::None),
            exclusive_zone: None,
            events_transparent: false,
            namespace: Some("IcedOsd".to_owned()),
        }
    }
}
