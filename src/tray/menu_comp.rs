use iced::{
    Point,
    widget::{container, text},
};
use iced_layershell::reexport::{
    Anchor, KeyboardInteractivity, Layer, NewLayerShellSettings, OutputOption,
};

use crate::feature::{Comp, Feature};

#[derive(Debug, Clone)]
pub enum Message {}

pub struct Init {
    pub starting_position: Point,
}

pub struct MenuComp {
    position: Point,
}

impl Comp for MenuComp {
    type Message = Message;
    type Init = Init;

    fn new(input: Self::Init) -> Self {
        Self {
            position: input.starting_position,
        }
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        container(text!("Menu"))
            .align_bottom(20)
            .style(container::rounded_box)
            .into()
    }
}

impl Feature for MenuComp {
    fn layer(&self) -> iced_layershell::reexport::NewLayerShellSettings {
        let Point { x, .. } = self.position;
        NewLayerShellSettings {
            layer: Layer::Overlay,
            size: Some((100, 100)),
            anchor: Anchor::Bottom | Anchor::Left,
            keyboard_interactivity: KeyboardInteractivity::OnDemand,
            exclusive_zone: Some(-1),
            output_option: OutputOption::LastOutput,
            events_transparent: false,
            namespace: Some("TrayMenu".into()),
            margin: Some((0, 0, 0, x as i32)),
        }
    }
}
