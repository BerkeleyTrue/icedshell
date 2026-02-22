use iced::widget::text;
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
        text!("Menu").into()
    }
}

impl Feature for MenuComp {
    fn layer(&self) -> iced_layershell::reexport::NewLayerShellSettings {
        NewLayerShellSettings {
            layer: Layer::Top,
            size: Some((0, 20)),
            anchor: Anchor::Bottom | Anchor::Left,
            keyboard_interactivity: KeyboardInteractivity::OnDemand,
            exclusive_zone: Some(32),
            output_option: OutputOption::LastOutput,
            events_transparent: false,
            namespace: Some("DeloraMainBar".into()),
            margin: Some((0, 0, 1500, 10)),
        }
    }
}
