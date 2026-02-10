use iced::{
    Color, Length, Subscription, border, padding,
    widget::{container, row},
};
use iced_layershell::reexport::{
    Anchor, KeyboardInteractivity, Layer, NewLayerShellSettings, OutputOption,
};

use crate::{
    config::MonitorId,
    datetime,
    divider::{Angled, Direction, Heading, Semi},
    feature::{
        Comp, CompWithProps, Feature, Service, align_center, bar_widgets, center_widgets,
        left_widgets, right_widgets,
    },
    niri::{state, window, ws},
    theme::{AppTheme, ROSEWATER, app_theme},
};

#[derive(Debug)]
pub enum Message {
    Clock(datetime::Message),

    Ws(ws::Message),
    Win(window::Message),

    NiriService(state::Message),
}

pub struct Init {
    pub output_name: String,
}

pub struct DeloraMain {
    ws: ws::NiriWS,
    win: window::NiriWin,
    clock: datetime::Clock,
    theme: AppTheme,
    output_name: String,
    height: f32,
    padding: f32,
    niri_serv: state::State,
}

impl DeloraMain {
    pub fn clone_niri_serv(&mut self, old_bar: &DeloraMain) {
        self.niri_serv = old_bar.niri_serv.clone()
    }
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
            clock: datetime::Clock::new(()),
            output_name: input.output_name,
            theme,
            height,
            padding,
            niri_serv: state::State::new(()),
        }
    }

    fn update(&mut self, message: Self::Message) -> iced::Task<Self::Message> {
        match message {
            Message::Ws(message) => self.ws.update(message).map(Message::Ws),
            Message::Clock(message) => self.clock.update(message).map(Message::Clock),
            Message::Win(message) => self.win.update(message).map(Message::Win),
            Message::NiriService(message) => {
                self.niri_serv.update(message).map(Message::NiriService)
            }
        }
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        let clock = self.clock.subscription().map(Message::Clock);
        let niri_serv = self.niri_serv.subscription().map(Message::NiriService);
        let niri_ws = self.ws.subscription().map(Message::Ws);
        let niri_win = self.win.subscription().map(Message::Win);
        Subscription::batch(vec![clock, niri_ws, niri_win, niri_serv])
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        let theme = &self.theme;

        let clock_view = align_center!(self.clock.view(theme.background()).map(Message::Clock))
            .padding(padding::right(theme.spacing().sm()));

        let niri_ws_view = align_center!(
            self.ws
                .view(ws::Props {
                    state: &self.niri_serv,
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
        let win = align_center!(
            self.win
                .view(window::Props {
                    color: ROSEWATER,
                    state: &self.niri_serv,
                })
                .map(Message::Win),
        );

        // main bar
        bar_widgets!(
            left:  clock_view, niri_ws_view, div;
            center: win_div, win;
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
