use iced::{
    Color, Length, Subscription, Task, border, padding,
    widget::{container, row},
};
use iced_layershell::reexport::{
    Anchor, KeyboardInteractivity, Layer, NewLayerShellSettings, OutputOption,
};
use tracing::info;

use crate::{
    clock,
    config::MonitorId,
    divider::{Angled, Direction, Heading, Semi},
    feature::{
        Comp, CompWithProps, Feature, bar_widgets, center_widgets, left_widgets, right_widgets,
        wrap_comp,
    },
    niri::{state, stream, window, ws},
    theme::{AppTheme, ROSEWATER, Shade, app_theme},
};

#[derive(Debug)]
pub enum Message {
    Clock(clock::Message),

    Ws(ws::Message),
    Win(window::Message),

    NiriEvent(stream::NiriEvent),
    NiriError(stream::NiriStreamError),
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
    niri_state: state::State,
}

impl Comp for DeloraMain {
    type Message = Message;
    type Init = Init;

    fn new(input: Self::Init) -> Self {
        let theme = app_theme();
        let height = theme.spacing().xl();
        let padding = theme.spacing().xs();
        let monitor_id = MonitorId(input.output_name.clone());
        Self {
            ws: ws::NiriWS::new(ws::Init {
                main_mon: monitor_id.clone(),
            }),
            win: window::NiriWin::new(window::Init { monitor_id }),
            clock: clock::Clock::new(()),
            output_name: input.output_name,
            theme,
            height,
            padding,
            niri_state: state::State::new(),
        }
    }

    fn update(&mut self, message: Self::Message) -> iced::Task<Self::Message> {
        match message {
            Message::Ws(message) => self.ws.update(message).map(Message::Ws),
            Message::Clock(message) => self.clock.update(message).map(Message::Clock),
            Message::Win(message) => self.win.update(message).map(Message::Win),
            Message::NiriEvent(event) => {
                self.niri_state.apply(event);
                Task::none()
            }
            Message::NiriError(err) => {
                info!("Stream err: {err:}");
                Task::none()
            }
        }
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        let niri_state = Subscription::run(stream::listen).map(|event| match event {
            Ok(ev) => Message::NiriEvent(ev),
            Err(err) => Message::NiriError(err),
        });
        let clock = self.clock.subscription().map(Message::Clock);
        let niri_ws = self.ws.subscription().map(Message::Ws);
        let niri_win = self.win.subscription().map(Message::Win);
        Subscription::batch(vec![clock, niri_ws, niri_win, niri_state])
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        let theme = &self.theme;

        let clock_view = wrap_comp(self.clock.view(theme.background()).map(Message::Clock))
            .padding(padding::right(theme.spacing().sm()));

        let niri_ws_view = wrap_comp(
            self.ws
                .view(ws::Props {
                    state: &self.niri_state,
                })
                .map(self::Message::Ws),
        )
        .padding(padding::left(theme.spacing().xs()))
        .style(|_| container::Style {
            background: Some(theme.background().into()),
            border: border::rounded(border::left(theme.radius().xl())),
            ..Default::default()
        });

        let div = Angled::new(
            theme.background(),
            Direction::Right,
            Heading::North,
            theme.spacing().xl(),
        );

        let win_div = Semi::new(ROSEWATER, Direction::Left);
        let win = wrap_comp(
            self.win
                .view(window::Props {
                    color: theme.neutral(Shade::S800),
                    state: &self.niri_state,
                })
                .map(Message::Win),
        )
        .style(|_| container::Style {
            background: Some(ROSEWATER.into()),
            ..Default::default()
        });

        let win_div_2 = Angled::new(
            ROSEWATER,
            Direction::Right,
            Heading::South,
            theme.spacing().xl(),
        );

        // main bar
        bar_widgets!(
            left: clock_view, niri_ws_view, div;
            center: win_div, win, win_div_2;
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
