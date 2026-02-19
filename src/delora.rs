use iced::{Color, Length, Subscription, padding, widget::row};
use iced_layershell::reexport::{
    Anchor, KeyboardInteractivity, Layer, NewLayerShellSettings, OutputOption,
};

use crate::{
    config::MonitorId,
    datetime::{clock, date},
    divider::{Angled, Direction, Heading, Semi},
    feature::{
        Comp, CompWithProps, Feature, Service, align_center, bar_widgets, center_widgets,
        left_widgets, right_widgets,
    },
    niri::{state, window, ws},
    theme::{AppTheme, LAVENDER, ROSEWATER, app_theme},
    tray::{self, module::TrayMod},
    widget_ext::ContainExt,
};

#[derive(Debug)]
pub enum Message {
    Clock(clock::Message),
    Date(date::Message),

    Ws(ws::Message),
    Win(window::Message),

    NiriService(state::Message),
    TrayService(tray::service::Message),
    Tray(tray::module::Message),
}

pub struct Init {
    pub output_name: String,
}

pub struct DeloraMain {
    height: f32,
    padding: f32,

    ws: ws::NiriWS,
    win: window::NiriWin,
    clock: clock::Clock,
    date: date::Date,
    theme: AppTheme,
    output_name: String,
    niri_serv: state::State,
    tray_serv: tray::service::TrayService,
    tray: tray::module::TrayMod,
}

impl DeloraMain {
    pub fn clone_servs(&mut self, old_bar: &DeloraMain) {
        self.niri_serv = old_bar.niri_serv.clone();
        self.tray_serv = old_bar.tray_serv.clone();
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
            height,
            padding,

            ws: ws::NiriWS::new(ws::Init {
                main_mon: monitor_id.clone(),
            }),
            win: window::NiriWin::new(window::Init { monitor_id }),
            clock: clock::Clock::new(()),
            date: date::Date::new(()),
            output_name: input.output_name,
            theme,
            niri_serv: state::State::new(()),
            tray_serv: tray::service::TrayService::new(()),
            tray: tray::module::TrayMod::new(()),
        }
    }

    fn update(&mut self, message: Self::Message) -> iced::Task<Self::Message> {
        match message {
            Message::Ws(message) => self.ws.update(message).map(Message::Ws),
            Message::Clock(message) => self.clock.update(message).map(Message::Clock),
            Message::Date(message) => self.date.update(message).map(Message::Date),
            Message::Win(message) => self.win.update(message).map(Message::Win),
            Message::NiriService(message) => {
                self.niri_serv.update(message).map(Message::NiriService)
            }
            Message::TrayService(message) => {
                self.tray_serv.update(message).map(Message::TrayService)
            }
            Message::Tray(message) => self.tray.update(message).map(Message::Tray),
        }
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        let clock = self.clock.subscription().map(Message::Clock);
        let date = self.date.subscription().map(Message::Date);
        let niri_serv = self.niri_serv.subscription().map(Message::NiriService);
        let niri_ws = self.ws.subscription().map(Message::Ws);
        let niri_win = self.win.subscription().map(Message::Win);
        let tray_serv = self.tray_serv.subscription().map(Message::TrayService);
        Subscription::batch([clock, date, niri_ws, niri_win, niri_serv, tray_serv])
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        let theme = &self.theme;

        let date_view = align_center!(self.date.view(theme.background()).map(Message::Date));

        let niri_ws_view = align_center!(
            self.ws
                .view(ws::Props {
                    state: &self.niri_serv,
                })
                .map(self::Message::Ws),
        );

        let div = align_center!(Angled::new(
            LAVENDER,
            Direction::Right,
            Heading::South,
            theme.spacing().xl(),
        ))
        .background(theme.background());

        let clock_view = align_center!(self.clock.view(theme.background()).map(Message::Clock))
            .padding(padding::right(theme.spacing().sm()));

        let win_div = Semi::new(ROSEWATER, Direction::Left, theme.spacing().xl());

        let win = align_center!(
            self.win
                .view(window::Props {
                    color: ROSEWATER,
                    state: &self.niri_serv,
                })
                .map(Message::Win),
        );

        let tray = self
            .tray
            .view(tray::module::Props {
                serv: &self.tray_serv,
            })
            .map(Message::Tray);

        // main bar
        bar_widgets!(
            left:  date_view, div, niri_ws_view;
            center: clock_view, win_div, win, tray;
            right:
        )
        .background(Color::TRANSPARENT)
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
