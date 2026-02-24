use iced::{
    Element, Length, Point,
    widget::{Row, button, container, text},
};
use iced_layershell::reexport::{
    Anchor, KeyboardInteractivity, Layer, NewLayerShellSettings, OutputOption,
};
use tracing::info;

use crate::{
    feature::{Comp, Feature, align_center},
    theme::{AppTheme, app_theme},
    tray::{TrayLayoutProps, dbus::TrayLayout},
};

#[derive(Debug, Clone)]
pub enum Message {
    ItemSelected(String, i32),
    ToggleMenu(i32),
}

pub struct Init {
    pub name: String,
    pub starting_position: Point,
    pub layout: TrayLayout,
}

pub struct MenuComp {
    name: String,
    position: Point,
    layout: TrayLayout,
    theme: AppTheme,
}

impl MenuComp {
    fn view_menu<'a>(&self, name: &'a str, layout: &'a TrayLayout) -> Element<'a, Message> {
        let theme = &self.theme;
        match &layout.props {
            TrayLayoutProps {
                label: Some(label), ..
            } => align_center!(
                button(text(label.replace("_", "")))
                    .width(Length::Fill)
                    .on_press(Message::ToggleMenu(layout.id))
            )
            .padding(theme.spacing().xl())
            .into(),
            _ => Row::new().into(),
        }
    }
}

impl Comp for MenuComp {
    type Message = Message;
    type Init = Init;

    fn new(input: Self::Init) -> Self {
        let theme = app_theme();
        Self {
            name: input.name,
            position: input.starting_position,
            layout: input.layout,
            theme,
        }
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        container(text!("foo"))
            // .align_bottom(20)
            .height(Length::Fill)
            .width(Length::Fill)
            .style(container::rounded_box)
            .into()
    }
}

impl Feature for MenuComp {
    fn layer(&self) -> iced_layershell::reexport::NewLayerShellSettings {
        let theme = &self.theme;
        let item_height = theme.spacing().lg();
        let Point { x, y } = self.position;
        let height = self
            .layout
            .children
            .iter()
            .fold(theme.spacing().lg(), |height, menu| {
                height + (menu.children.len() as f32 * item_height)
            });
        info!("height: {height}");
        NewLayerShellSettings {
            layer: Layer::Overlay,
            size: Some((height as u32, 100)),
            anchor: Anchor::Bottom | Anchor::Left,
            keyboard_interactivity: KeyboardInteractivity::OnDemand,
            exclusive_zone: Some(-1),
            output_option: OutputOption::LastOutput,
            events_transparent: false,
            namespace: Some("TrayMenu".into()),
            margin: Some((0, 0, y as i32, x as i32)),
        }
    }
}
