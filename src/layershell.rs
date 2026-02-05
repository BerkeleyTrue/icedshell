use iced::{
    Color, Element, Subscription, Task,
    keyboard::{self, Key, key::Named},
    theme::Style,
    window::Id,
};
use iced_layershell::{
    reexport::{Anchor, KeyboardInteractivity, Layer},
    settings::LayerShellSettings,
    to_layer_message,
};

use crate::{
    Cli, app,
    feature::Comp,
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
    App(app::Message),
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
    app: app::App,
    quit_keybinds: bool,
}

impl Layershell {
    fn new(init: Init) -> Self {
        Self {
            app: app::App::new(()),
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

        let app_sub = self.app.subscription().map(Message::App);
        Subscription::batch(vec![quit_binds, app_sub])
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
        layer: Layer::Top,
        size: Some((0, theme.spacing().xl() as u32)),
        exclusive_zone: (theme.spacing().xl()) as i32,
        anchor: Anchor::Left | Anchor::Bottom | Anchor::Right,
        keyboard_interactivity: KeyboardInteractivity::None,
        ..Default::default()
    })
    .run()
}
