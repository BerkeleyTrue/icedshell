use iced::{Task, Theme, advanced::graphics::core::Element, theme::Style};
use iced_layershell::{reexport::{Anchor, KeyboardInteractivity, Layer}, settings::LayerShellSettings, to_layer_message};

use crate::theme::PALETTE;

const SIZE: (u32, u32) = (623, 390);
const REM: u32 = 14;

#[to_layer_message]
enum Message {}

struct Layershell {

}

impl Layershell {
    pub fn new() -> Self {
        Self {}
    }


    pub fn update(&self, message: Message) -> Task<Message> {
        match message {
            _ => Task::none()
        }
    }

    pub fn view() -> Element<'_, Message> {

    }
}

pub fn start() -> iced_layershell::Result {
    iced_layershell::daemon(
        move || Layershell::new(),
        || "Icedshell".to_string(),
        Layershell::update,
        Layershell::view,
    )
    .theme(Theme::custom("catppuccin", PALETTE))
    // .subscription(Layershell::subscription)
    .style(|_layer, theme| Style {
        background_color: theme.palette().background,
        text_color: theme.palette().text,
    })
    .layer_settings(LayerShellSettings {
        layer: Layer::Top,
        size: Some(SIZE).map(|(w, h)| (w + REM as u32, h + REM as u32)),
        anchor: Anchor::empty(),
        keyboard_interactivity: KeyboardInteractivity::OnDemand,
        ..Default::default()
    })
    .run()
}
