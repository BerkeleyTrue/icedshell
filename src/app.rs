use iced::{
    Color, Element, Length, Subscription, Task, padding,
    widget::{container, row},
};

use crate::{
    clock,
    config::Config,
    feature::{Comp, CompWithProps},
    niri,
    theme::{self as my_theme},
};

#[derive(Debug)]
pub enum Message {
    Niri(niri::Message),
    Clock(clock::Message),
}

pub struct App {
    niri: niri::NiriWS,
    clock: clock::Clock,
    config: Config,
}

impl Comp for App {
    type Message = Message;
    type Init = ();

    fn new(_init: Self::Init) -> Self {
        Self {
            niri: niri::NiriWS::new(()),
            clock: clock::Clock::new(()),
            config: Config::default(),
        }
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        let clock = self.clock.subscription().map(Message::Clock);
        let niri_ws = self.niri.subscription().map(Message::Niri);
        Subscription::batch(vec![clock, niri_ws])
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Niri(message) => self.niri.update(message).map(Message::Niri),
            Message::Clock(message) => self.clock.update(message).map(Message::Clock),
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let theme = my_theme::app_theme();
        let clock_view = container(self.clock.view(theme.background()).map(Message::Clock))
            .padding(padding::right(theme.spacing().sm()));

        let niri_ws_view = self.niri.view().map(Message::Niri);

        let left_widgets = row![clock_view, niri_ws_view];

        container(left_widgets)
            .style(|_| container::Style {
                background: Some(Color::TRANSPARENT.into()),
                ..Default::default()
            })
            .padding(padding::left(theme.spacing().md()).bottom(theme.spacing().xxs()))
            .center_y(Length::Fill)
            .into()
    }
}
