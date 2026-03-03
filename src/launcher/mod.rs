use iced::{
    Length, border,
    widget::{Space, container, text},
};
use iced_layershell::reexport::{
    Anchor, KeyboardInteractivity, Layer, NewLayerShellSettings, OutputOption,
};

use crate::{
    feature::{Comp, Feature},
    theme::CAT_THEME,
};

#[derive(Clone, Debug)]
pub enum Message {}

pub struct Launcher {}

impl Comp for Launcher {
    type Message = Message;
    type Init = ();

    fn new(_input: Self::Init) -> Self {
        Self {}
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        let theme = &CAT_THEME;
        container(text!("foo"))
            .style(|_| container::Style {
                background: Some(theme.background().into()),
                text_color: Some(theme.text_color()),
                border: border::color(theme.overlay0())
                    .width(theme.spacing().xxs())
                    .rounded(theme.radius().md()),
                ..Default::default()
            })
            .padding(theme.spacing().xs())
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
