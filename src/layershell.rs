use iced::{
    Color, Element, Subscription, Task,
    keyboard::{self, Key, key::Named},
    theme::Style,
    widget::{container, space},
    window::Id,
};
use iced_layershell::{
    reexport::{Anchor, KeyboardInteractivity, Layer},
    settings::{LayerShellSettings, StartMode},
    to_layer_message,
};

use crate::{
    Cli,
    delora::{self, DeloraMain},
    feature::{Comp, Feature, Window},
    theme::{self as mytheme},
};

#[derive(Clone)]
enum Hosts {
    Delora,
    Rena,
}

#[to_layer_message(multi)]
#[derive(Debug)]
pub enum Message {
    Delora(delora::Message),
    Quit,
}

#[derive(Clone)]
pub struct Init {
    quit_keybinds: bool,
    host: Hosts,
}

impl Init {
    pub fn host(&mut self, host: &str) {
        self.host = match host {
            "delora" => Hosts::Delora,
            "rena" => Hosts::Rena,
            _ => Hosts::Delora,
        };
    }
}

impl From<Cli> for Init {
    fn from(cli: Cli) -> Self {
        Self {
            quit_keybinds: cli.quit_keybindings,
            host: Hosts::Delora,
        }
    }
}

struct Layershell {
    delora_main: Window<DeloraMain>,
    quit_keybinds: bool,
}

impl Layershell {
    fn new(init: Init) -> (Self, Task<Message>) {
        let (delora_window, layer_settings) = DeloraMain::new(()).open();
        let delora_window_id = delora_window.id.clone();
        (
            Self {
                delora_main: delora_window,
                quit_keybinds: init.quit_keybinds,
            },
            Task::done(Message::NewLayerShell {
                id: delora_window_id,
                settings: layer_settings,
            }),
        )
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
            Message::Delora(message) => self.delora_main.update(message).map(Message::Delora),
            _ => Task::none(),
        }
    }

    fn view(&self, id: Id) -> Element<'_, Message> {
        if self.delora_main.id == id {
            self.delora_main.view().map(Message::Delora)
        } else {
            container(space()).into()
        }
    }
}

pub fn start(init: Init) -> iced_layershell::Result {
    let theme = mytheme::app_theme();
    iced_layershell::daemon(
        move || Layershell::new(init.clone()),
        || "Icedshell".to_string(),
        Layershell::update,
        Layershell::view,
    )
    .theme(theme.theme())
    .subscription(Layershell::subscription)
    .style(|_layer, theme| Style {
        background_color: Color::TRANSPARENT,
        text_color: theme.palette().text,
    })
    .layer_settings(LayerShellSettings {
        keyboard_interactivity: KeyboardInteractivity::None,
        start_mode: StartMode::Background,
        ..Default::default()
    })
    .run()
}
