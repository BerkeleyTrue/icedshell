// use std::collections::HashMap;

use iced::{Element, Subscription, Task, Theme, keyboard::{self, Key, key::Named}, theme::Style, window::Id};
use iced_layershell::{
    reexport::{Anchor, KeyboardInteractivity, Layer},
    settings::LayerShellSettings,
    to_layer_message,
};

use crate::{Cli, app, theme::{self as mytheme, PALETTE, REM}};

#[to_layer_message]
#[derive(Debug)]
enum Message {
    App(app::Message),
    Quit,
}

#[derive(Clone)]
pub struct Init {
    quit_keybinds: bool,
}

impl From<Cli> for Init {
    fn from(cli: Cli) -> Self {
        Self {
            quit_keybinds: cli.quit_keybindings,
        }
    }
}

struct Layershell {
    app: app::App,
    quit_keybinds: bool,
}

impl Layershell {
    fn new(init: Init) -> Self {
        Self {
            app: app::App::new(),
            quit_keybinds: init.quit_keybinds,
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        let quit_binds = keyboard::listen()
            .with(self.quit_keybinds)
            .filter_map(|(quit_keybinds, event)| match (quit_keybinds, event) {
                (false, keyboard::Event::KeyPressed { key, .. }) => Some(key),
                _ => None,
            })
            .filter_map(|key| match key.as_ref() {
                Key::Named(Named::Escape) | Key::Character("q") => Some(Message::Quit),
                _ => None,
            });

        Subscription::batch(vec![quit_binds])
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::App(message) => self.app.update(message).map(Message::App),
            _ => Task::none(),
        }
    }

    fn view(&self, _id: Id) -> Element<'_, Message> {
        self.app.view().map(Message::App)
    }
}

pub fn start(init: Init) -> iced_layershell::Result {
    iced_layershell::daemon(
        move || Layershell::new(init.clone()),
        || "Icedshell".to_string(),
        Layershell::update,
        Layershell::view,
    )
    .theme(Theme::custom("catppuccin", PALETTE))
    .subscription(Layershell::subscription)
    .style(|_layer, _theme| Style {
        background_color: mytheme::BASE,
        text_color: mytheme::TEXT,
    })
    .layer_settings(LayerShellSettings {
        layer: Layer::Top,
        size: Some((0, REM * 2)),
        exclusive_zone: (REM * 2) as i32,
        anchor: Anchor::Left | Anchor::Bottom | Anchor::Right,
        keyboard_interactivity: KeyboardInteractivity::None,
        ..Default::default()
    })
    .run()
}
