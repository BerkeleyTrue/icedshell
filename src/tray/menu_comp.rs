use iced::{
    Color, Element, Length, Task,
    alignment::Vertical,
    border, padding,
    widget::{Column, Row, button, container, row, text, toggler},
};
use iced_layershell::actions::{IcedNewMenuSettings, MenuDirection};
use lucide_icons::Icon;
use tracing::info;

use crate::{
    feature::{Comp, Feature},
    theme::CAT_THEME,
    tray::{
        TrayLayoutProps,
        dbus::{TrayLayout, TrayMenuItemId},
    },
};

#[derive(Debug, Clone)]
pub enum Message {
    OpenSubMenu(TrayMenuItemId),
    CloseSubMenu,
    ItemSelected(String, TrayMenuItemId),
}

pub struct Init {
    pub name: String,
    pub layout: TrayLayout,
}

pub struct MenuComp {
    name: String,
    layout: TrayLayout,
    menu_stack: Vec<TrayMenuItemId>,
}

impl MenuComp {
    fn get_current_menu<'a>(
        stack: &'a [TrayMenuItemId],
        layout: &'a TrayLayout,
    ) -> &'a [TrayLayout] {
        if let Some((item_id, rest)) = stack.split_first()
            && let Some(next_children) = layout.children.iter().find(|id| id.id == *item_id)
        {
            Self::get_current_menu(rest, next_children)
        } else {
            &layout.children
        }
    }

    fn view_menu<'a>(&'a self, name: &'a str, layout: &'a TrayLayout) -> Element<'a, Message> {
        let theme = &CAT_THEME;
        match &layout.props {
            // Divider
            TrayLayoutProps { type_: Some(t), .. } if t == "separator" => {
                // info!("sep");
                container(text!("---")).into()
            }
            // toggle state
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
            // sub menu
            TrayLayoutProps {
                children_display: Some(display),
                label: Some(label),
                ..
            } if display == "submenu" => {
                let icon = Icon::DiamondPlus
                    .widget()
                    .size(theme.spacing().md())
                    .align_y(Vertical::Center)
                    .width(theme.spacing().lg());

                let label = text(label.clone().replace("_", ""))
                    .align_y(Vertical::Center)
                    .size(theme.spacing().md());

                let button_content = row![icon, label]
                    .spacing(theme.spacing().xs())
                    .align_y(Vertical::Center);

                button(button_content)
                    .style(|_, status| {
                        let base = button::Style {
                            background: Some(Color::TRANSPARENT.into()),
                            text_color: theme.text_color(),
                            ..Default::default()
                        };
                        match status {
                            button::Status::Hovered => button::Style {
                                background: Some(theme.surface0().into()),
                                ..base
                            },
                            _ => base,
                        }
                    })
                    .padding(padding::vertical(theme.spacing().xxs()).left(theme.spacing().xxs()))
                    .width(Length::Fill)
                    .on_press(Message::OpenSubMenu(layout.id))
                    .into()
            }
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
                            text_color: theme.text_color(),
                            ..Default::default()
                        };
                        match status {
                            button::Status::Hovered => button::Style {
                                background: Some(theme.surface0().into()),
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
                info!("empty");
                Row::new().into()
            }
        }
    }
}

impl Comp for MenuComp {
    type Message = Message;
    type Init = Init;

    fn new(input: Self::Init) -> (Self, Task<Self::Message>) {
        let layout = &input.layout;
        info!("layout: {layout:?}");
        Self {
            name: input.name,
            layout: input.layout,
            menu_stack: Vec::new(),
        }
        .to_tuple()
    }

    fn update(&mut self, message: Self::Message) -> iced::Task<Self::Message> {
        match message {
            Message::OpenSubMenu(id) => {
                self.menu_stack.push(id);
                Task::none()
            }
            Message::CloseSubMenu => {
                self.menu_stack.pop();
                Task::none()
            }
            Message::ItemSelected(_name, _id) => Task::none(),
        }
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        let theme = &CAT_THEME;
        let menu_items = Self::get_current_menu(&self.menu_stack, &self.layout);

        let mut top_menu = menu_items
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

        if !self.menu_stack.is_empty() {
            top_menu = top_menu.push({
                let label = text("Back")
                    .align_y(Vertical::Center)
                    .size(theme.spacing().md());

                Element::from(
                    button(label)
                        .style(|_, status| {
                            let base = button::Style {
                                background: Some(Color::TRANSPARENT.into()),
                                text_color: theme.text_color(),
                                ..Default::default()
                            };
                            match status {
                                button::Status::Hovered => button::Style {
                                    background: Some(theme.surface0().into()),
                                    ..base
                                },
                                _ => base,
                            }
                        })
                        .padding(
                            padding::left(theme.spacing().xl()).vertical(theme.spacing().xxs()),
                        )
                        .width(Length::Fill)
                        .on_press(Message::CloseSubMenu),
                )
            })
        }

        container(top_menu)
            .height(Length::Fill)
            .width(Length::Fill)
            .padding(theme.spacing().xs())
            .style({
                move |_| container::Style {
                    border: border::rounded(theme.radius().lg())
                        .color(theme.mauve())
                        .width(theme.spacing().xxs()),
                    background: Some(theme.base().into()),
                    ..Default::default()
                }
            })
            .into()
    }
}

impl Feature for MenuComp {
    type Settings = IcedNewMenuSettings;
    fn layer(&self) -> IcedNewMenuSettings {
        let theme = &CAT_THEME;
        let item_height = theme.spacing().lg();
        let height = self.layout.children.len() as f32 * item_height + theme.spacing().xs();

        IcedNewMenuSettings {
            size: (220, height as u32),
            direction: MenuDirection::Up,
        }
    }
}
