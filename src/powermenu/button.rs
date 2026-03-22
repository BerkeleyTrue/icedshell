use iced::{
    Border, Element,
    Length::Fill,
    Theme,
    advanced::widget::text,
    widget::{
        Button,
        button::{self, Status},
        container,
    },
};
use iced_font_awesome::fa_icon_solid;

use super::palette::{LINEAR_BACKGROUND, LINEAR_BACKGROUND_FOCUS};

pub enum Icon {
    Lock,
    Sleep,
    Shutdown,
    Reboot,
    Logout,
}

pub struct PowerButton<T> {
    pub icon: Icon,
    pub message: T,
}

impl<T: Clone + 'static> PowerButton<T> {
    pub fn view(&self, is_focused: bool) -> Element<'static, T> {
        let icon = (match self.icon {
            Icon::Lock => fa_icon_solid("lock"),
            Icon::Sleep => fa_icon_solid("moon"),
            Icon::Reboot => fa_icon_solid("rotate"),
            Icon::Shutdown => fa_icon_solid("power-off"),
            Icon::Logout => fa_icon_solid("arrow-right-to-bracket"),
        })
        .style(|theme: &Theme| {
            let palette = theme.palette();
            text::Style {
                color: Some(palette.background),
            }
        })
        .size(15.0);

        let button = Button::new(container(icon).center(Fill))
            .style(move |theme: &Theme, status| {
                let palette = theme.palette();

                button::Style {
                    background: match (status, is_focused) {
                        (Status::Hovered | Status::Pressed, _) => Some(LINEAR_BACKGROUND_FOCUS),
                        (_, true) => Some(LINEAR_BACKGROUND_FOCUS),
                        _ => Some(LINEAR_BACKGROUND),
                    },
                    border: Border::default()
                        .rounded(1.0)
                        .color(palette.background)
                        .width(2.0),
                    ..Default::default()
                }
            })
            .padding(10)
            .width(Fill)
            .height(40)
            .on_press(self.message.clone());

        let button_wrapper = container(button)
            .style(|theme: &Theme| {
                let palette = theme.palette();

                container::Style {
                    border: Border::default()
                        .rounded(1.0)
                        .color(palette.text)
                        .width(3.0),
                    ..Default::default()
                }
            })
            .padding(1);

        container(button_wrapper).center_x(Fill).into()
    }
}
