use iced::{
    Color, Element, Length, Task,
    alignment::Vertical,
    border, padding,
    widget::{Column, Row, button, container, text, toggler},
};
use iced_layershell::actions::{IcedNewMenuSettings, MenuDirection};
use tracing::info;

use crate::{
    feature::{Comp, Feature},
    theme::{AppTheme, SURFACE0, Shade, TEXT, app_theme},
    tray::{TrayLayoutProps, dbus::TrayLayout},
};

#[derive(Debug, Clone)]
pub enum Message {
    ToggleMenu(i32),
    ItemSelected(String, i32),
}

pub struct Init {
    pub name: String,
    pub layout: TrayLayout,
}

pub struct MenuComp {
    name: String,
    layout: TrayLayout,
    theme: AppTheme,
}

impl Comp for MenuComp {
    type Message = Message;
    type Init = Init;

    fn new(input: Self::Init) -> Self {
        let theme = app_theme();
        let layout = &input.layout;
        info!("layout: {layout:?}");
        Self {
            name: input.name,
            layout: input.layout,
            theme,
        }
    }

    fn update(&mut self, message: Self::Message) -> iced::Task<Self::Message> {
        match message {
            Message::ToggleMenu(_id) => Task::none(),
            Message::ItemSelected(_name, _id) => Task::none(),
        }
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        let theme = &self.theme;
        // top level layout is always submenu
        let top_menu = self
            .layout
            .children
            .iter()
            .map(|menu| {
                Element::from(
                    container(self.view_menu(&self.name, menu))
                        .padding(padding::left(theme.spacing().xs()))
                        .center_y(theme.spacing().lg())
                        .center_x(Length::Fill),
                )
            })
            .fold(Column::new(), |col, item_elem| col.push(item_elem));

        container(top_menu)
            .height(Length::Fill)
            .width(Length::Fill)
            .padding(theme.spacing().xs())
            .style(|_| container::Style {
                border: border::rounded(theme.radius().lg())
                    .color(theme.secondary(Shade::S500))
                    .width(theme.spacing().xxs()),
                background: Some(theme.neutral(Shade::S900).into()),
                ..Default::default()
            })
            .into()
    }
}

impl MenuComp {
    fn view_menu<'a>(&self, name: &'a str, layout: &'a TrayLayout) -> Element<'a, Message> {
        let theme = &self.theme;
        match &layout.props {
            // Divider
            TrayLayoutProps { type_: Some(t), .. } if t == "seperator" => {
                // info!("sep");
                container(text!("---")).into()
            }
            TrayLayoutProps {
                label: Some(label),
                toggle_type: Some(togg_type),
                toggle_state: Some(togg_state),
                ..
            } if togg_type == "checkmark" => toggler(*togg_state > 0)
                .label(label.replace("_", ""))
                .text_size(theme.spacing().md())
                .width(Length::Fill)
                .size(theme.spacing().sm())
                .on_toggle({
                    let id = layout.id;
                    move |_| Message::ItemSelected(name.to_string(), id)
                })
                .into(),
            // regular button
            TrayLayoutProps {
                label: Some(label), ..
            } => {
                let label = text(label.clone().replace("_", ""))
                    .align_y(Vertical::Center)
                    .size(theme.spacing().md());

                button(label)
                    .style(|_, status| {
                        let base = button::Style {
                            background: Some(Color::TRANSPARENT.into()),
                            text_color: TEXT,
                            ..Default::default()
                        };
                        match status {
                            button::Status::Hovered => button::Style {
                                background: Some(SURFACE0.into()),
                                ..base
                            },
                            _ => base,
                        }
                    })
                    .padding(padding::left(theme.spacing().xl()).vertical(theme.spacing().xxs()))
                    .width(Length::Fill)
                    .on_press(Message::ItemSelected(name.to_string(), layout.id))
                    .into()
            }
            _ => {
                // info!("empty");
                Row::new().into()
            }
        }
    }
}

impl Feature for MenuComp {
    type Settings = IcedNewMenuSettings;
    fn layer(&self) -> IcedNewMenuSettings {
        let theme = &self.theme;
        let item_height = theme.spacing().lg();
        let height = self.layout.children.len() as f32 * item_height + theme.spacing().xs();

        IcedNewMenuSettings {
            size: (220, height as u32),
            direction: MenuDirection::Up,
        }
    }
}
