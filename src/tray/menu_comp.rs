use iced::{
    Element, Event, Length, Point, Subscription, Task, event, mouse,
    widget::{Column, Row, button, container, text},
};
use iced_layershell::{
    actions::{IcedNewMenuSettings, MenuDirection},
    reexport::{Anchor, KeyboardInteractivity, Layer, NewLayerShellSettings, OutputOption},
};
use tracing::info;

use crate::{
    feature::{Comp, Feature},
    theme::{AppTheme, BASE, OVERLAY0, OVERLAY1, SURFACE0, Shade, TEXT, app_theme},
    tray::{TrayLayoutProps, dbus::TrayLayout},
};

#[derive(Debug, Clone)]
pub enum Message {
    Focused,
    Unfocused,
    ToggleMenu(i32),
    ItemSelected(String, i32),
    CloseMenu,
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
    focused: bool,
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
            position: input.starting_position,
            layout: input.layout,
            theme,
            focused: true,
        }
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        event::listen().filter_map(|event| match event {
            Event::Mouse(mouse::Event::CursorEntered) => Some(Message::Focused),
            Event::Mouse(mouse::Event::CursorLeft) => Some(Message::Unfocused),
            _ => None,
        })
    }

    fn update(&mut self, message: Self::Message) -> iced::Task<Self::Message> {
        match message {
            Message::Focused => {
                info!("Focus");
                self.focused = true;
                Task::none()
            }
            Message::Unfocused => {
                info!("unfocus");
                if self.focused {
                    self.focused = false;
                    return Task::done(Message::CloseMenu);
                }
                Task::none()
            }
            Message::ToggleMenu(_id) => Task::none(),
            Message::ItemSelected(_name, _id) => Task::none(),
            Message::CloseMenu => Task::none(),
        }
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        let theme = &self.theme;
        // top level layout is always submenu
        let top_menu = self
            .layout
            .children
            .iter()
            .map(|menu| self.view_menu(&self.name, menu))
            .fold(Column::new(), |col, item_elem| col.push(item_elem))
            .spacing(theme.spacing().xs());

        container(top_menu)
            .height(Length::Fill)
            .width(Length::Fill)
            .style(container::rounded_box)
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
                container(text!("---")).center_x(Length::Fill).into()
            }
            // regular button
            TrayLayoutProps {
                label: Some(label), ..
            } => {
                let label = label.clone();
                button(text(label.replace("_", "")))
                    .style(|_, status| {
                        let base = button::Style {
                            background: Some(BASE.into()),
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
                    .height(theme.spacing().xl())
                    .width(Length::Fill)
                    .on_press(Message::ToggleMenu(layout.id))
                    .padding(theme.spacing().xs())
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
        // let Point { x, y } = self.position;
        let height = theme.spacing().md() + self.layout.children.len() as f32 * item_height;

        IcedNewMenuSettings {
            // layer: Layer::Overlay,
            // x, y
            size: (200, height as u32),
            direction: MenuDirection::Up,
            // anchor: Anchor::Bottom | Anchor::Left,
            // keyboard_interactivity: KeyboardInteractivity::OnDemand,
            // exclusive_zone: Some(-1),
            // output_option: OutputOption::LastOutput,
            // events_transparent: false,
            // namespace: Some("TrayMenu".into()),
            // // top/right/bottom/left
            // margin: Some((0, 0, y as i32 + 8, x as i32)),
        }
    }
}
