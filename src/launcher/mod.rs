use iced::{
    Length,
    widget::{Space, container, text},
};
use iced_layershell::reexport::{
    Anchor, KeyboardInteractivity, Layer, NewLayerShellSettings, OutputOption,
};

use crate::feature::{Comp, Feature};

#[derive(Clone, Debug)]
pub enum Message {}

pub struct Launcher {}

impl Comp for Launcher {
    type Message = Message;
    type Init = ();

    fn new(input: Self::Init) -> Self {
        Self {}
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        container(text!("foo"))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

impl Feature for Launcher {
    type Settings = NewLayerShellSettings;

    fn layer(&self) -> Self::Settings {
        NewLayerShellSettings {
            layer: Layer::Overlay,
            size: Some((400, 400)),
            anchor: Anchor::empty(),
            keyboard_interactivity: KeyboardInteractivity::OnDemand,
            output_option: OutputOption::LastOutput,
            namespace: Some("AppLauncher".into()),
            events_transparent: false,
            exclusive_zone: None,
            margin: None,
        }
    }
}
