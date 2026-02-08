use iced::{
    Color, Length, Subscription, border, padding,
    widget::{container, row},
};
use iced_layershell::reexport::{
    Anchor, KeyboardInteractivity, Layer, NewLayerShellSettings, OutputOption,
};

use crate::{
    clock,
    config::MonitorId,
    divider::{Direction, Divider, Heading},
    feature::{
        Comp, CompWithProps, Feature, bar_widgets, center_widgets, left_widgets, right_widgets,
        wrap_comp,
    },
    niri::{window, ws},
    theme::{AppTheme, ROSEWATER, Shade, app_theme},
};

#[derive(Debug)]
pub enum Message {
    Clock(clock::Message),
    Ws(ws::Message),
    Win(window::Message),
}

pub struct Init {
    pub output_name: String,
}

pub struct DeloraMain {
    ws: ws::NiriWS,
    win: window::NiriWin,
    clock: clock::Clock,
    theme: AppTheme,
    output_name: String,
    height: f32,
    padding: f32,
}

impl Comp for DeloraMain {
    type Message = Message;
    type Init = Init;

    fn new(input: Self::Init) -> Self {
        let theme = app_theme();
        let height = theme.spacing().xl();
        let padding = theme.spacing().xs();
        Self {
            ws: ws::NiriWS::new(ws::Init {
                main_mon: MonitorId(input.output_name.clone()),
            }),
            win: window::NiriWin::new(window::Init {}),
            clock: clock::Clock::new(()),
            output_name: input.output_name,
            theme,
            height,
            padding,
        }
    }

    fn update(&mut self, message: Self::Message) -> iced::Task<Self::Message> {
        match message {
            Message::Ws(message) => self.ws.update(message).map(Message::Ws),
            Message::Clock(message) => self.clock.update(message).map(Message::Clock),
        }
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        let clock = self.clock.subscription().map(Message::Clock);
        let niri_ws = self.ws.subscription().map(Message::Ws);
        Subscription::batch(vec![clock, niri_ws])
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        let theme = &self.theme;

        let clock_view = wrap_comp(self.clock.view(theme.background()).map(Message::Clock))
            .padding(padding::right(theme.spacing().sm()));

        let niri_ws_view = wrap_comp(self.ws.view().map(self::Message::Ws))
            .padding(padding::left(theme.spacing().xs()))
            .style(|_| container::Style {
                background: Some(theme.background().into()),
                border: border::rounded(border::left(theme.radius().xl())),
                ..Default::default()
            });

        let div = Divider::new(
            theme.background(),
            Direction::Right,
            Heading::North,
            theme.spacing().xl(),
        );

        let win =
            wrap_comp(self.win.view(theme.neutral(Shade::S800)).map(Message::Win)).style(|_| {
                container::Style {
                    background: Some(ROSEWATER.into()),
                    border: border::rounded(border::left(theme.radius().xl())),
                    ..Default::default()
                }
            });

        // main bar
        bar_widgets!(
            left: clock_view, niri_ws_view, div;
            center: win;
            right:
        )
        .style(|_| container::Style {
            background: Some(Color::TRANSPARENT.into()),
            // debug
            // border: border::color(theme.destructive(Shade::S500)).width(1),
            ..Default::default()
        })
        .padding(padding::left(theme.spacing().md()).bottom(self.padding))
        .center_y(Length::Fill)
        .into()
    }
}

impl Feature for DeloraMain {
    fn layer(&self) -> iced_layershell::reexport::NewLayerShellSettings {
        let output_name = self.output_name.clone();
        NewLayerShellSettings {
            layer: Layer::Top,
            size: Some((0, (self.height + self.padding) as u32)),
            anchor: Anchor::Left | Anchor::Bottom | Anchor::Right,
            keyboard_interactivity: KeyboardInteractivity::None,
            exclusive_zone: Some((self.height + self.padding) as i32),
            output_option: OutputOption::OutputName(output_name),
            events_transparent: false,
            namespace: Some("DeloraMainBar".into()),
            margin: None,
        }
    }
}
