use iced::widget::{container, text};
use iced_layershell::reexport::{
    Anchor, KeyboardInteractivity, Layer, NewLayerShellSettings, OutputOption,
};

use crate::feature::{Comp, Feature};

#[derive(Debug, Clone)]
pub enum Message {}

pub struct MenuComp {}

impl Comp for MenuComp {
    type Message = Message;
    type Init = ();

    fn new(_input: Self::Init) -> Self {
        Self {}
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        container(text!("Menu")).style(container::danger).into()
    }
}

impl Feature for MenuComp {
    fn layer(&self) -> iced_layershell::reexport::NewLayerShellSettings {
        NewLayerShellSettings {
            layer: Layer::Overlay,
            size: Some((100, 100)),
            anchor: Anchor::Bottom | Anchor::Left,
            keyboard_interactivity: KeyboardInteractivity::OnDemand,
            exclusive_zone: None,
            output_option: OutputOption::LastOutput,
            events_transparent: false,
            namespace: Some("TrayMenu".into()),
            margin: Some((0, 0, 10, 10)),
        }
    }
}
