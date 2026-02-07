use iced::{
    Color, Length, Subscription, padding,
    widget::{container, row},
};
use iced_layershell::reexport::{
    Anchor, KeyboardInteractivity, Layer, NewLayerShellSettings, OutputOption,
};

use crate::{
    clock,
    divider::{Direction, Heading, divider},
    feature::{Comp, CompWithProps, Feature},
    niri,
    theme::{self, AppTheme},
};

#[derive(Debug)]
pub enum Message {
    Clock(clock::Message),
    Niri(niri::Message),
}

pub struct DeloraMain {
    niri: niri::NiriWS,
    clock: clock::Clock,
    theme: AppTheme,
}

impl Comp for DeloraMain {
    type Message = Message;
    type Init = ();

    fn new(_input: Self::Init) -> Self {
        Self {
            niri: niri::NiriWS::new(()),
            clock: clock::Clock::new(()),
            theme: theme::app_theme(),
        }
    }

    fn update(&mut self, message: Self::Message) -> iced::Task<Self::Message> {
        match message {
            Message::Niri(message) => self.niri.update(message).map(Message::Niri),
            Message::Clock(message) => self.clock.update(message).map(Message::Clock),
        }
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        let clock = self.clock.subscription().map(Message::Clock);
        let niri_ws = self.niri.subscription().map(Message::Niri);
        Subscription::batch(vec![clock, niri_ws])
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        let theme = &self.theme;
        let clock_view = container(self.clock.view(theme.background()).map(Message::Clock))
            .padding(padding::right(theme.spacing().sm()));

        let niri_ws_view = self.niri.view().map(self::Message::Niri);

        let div = divider::<Self::Message>(
            theme.background(),
            Direction::Right,
            Heading::North,
            theme.spacing().xl(),
        );
        let left_widgets = row![clock_view, niri_ws_view, div];

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

impl Feature for DeloraMain {
    fn layer(&self) -> iced_layershell::reexport::NewLayerShellSettings {
        NewLayerShellSettings {
            layer: Layer::Top,
            size: Some((0, self.theme.spacing().xl() as u32)),
            anchor: Anchor::Left | Anchor::Bottom | Anchor::Right,
            keyboard_interactivity: KeyboardInteractivity::None,
            exclusive_zone: Some(self.theme.spacing().xl() as i32),
            output_option: OutputOption::OutputName("HDMI-A-1".into()),
            events_transparent: false,
            namespace: Some("DeloraMainBar".into()),
            margin: None,
        }
    }
}
