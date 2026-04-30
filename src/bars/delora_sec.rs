use iced::{
    Length, Subscription, Task, padding,
    widget::{container, row, text},
};
use iced_layershell::reexport::{
    Anchor, KeyboardInteractivity, Layer, NewLayerShellSettings, OutputOption,
};

use crate::{
    cmd,
    datetime::{clock_comp, date_comp},
    feature::{Comp, CompWithProps, Feature, Service},
    niri::{state_serv, win_comp},
    theme::CAT_THEME,
    types::MonitorId,
    widget::{
        IntoIteratorExt, bar_widgets,
        container_ext::ContainExt,
        divider::{Angled, Direction, Heading, Semi},
        text_ext::TextExt,
    },
};

pub struct DeloraSec {
    win: win_comp::NiriWinComp,
    output_name: String,
    position: Position,
    niri_serv: state_serv::NiriStateServ,
    clock: clock_comp::Clock,
    date: date_comp::Date,
    eth: cmd::CmdComp,
    btc: cmd::CmdComp,
}

#[derive(Debug, Clone)]
pub enum Message {
    Win(win_comp::Message),
    NiriService(state_serv::Message),
    Clock(clock_comp::Message),
    Date(date_comp::Message),
    Eth(cmd::Message),
    Btc(cmd::Message),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Position {
    Top,
    Bottom,
}

pub struct Init {
    pub output_name: String,
    pub position: Position,
}

impl Comp for DeloraSec {
    type Message = Message;
    type Init = Init;

    fn new<O: iced::advanced::graphics::futures::MaybeSend + 'static>(
        input: Self::Init,
        f: impl Fn(Self::Message) -> O + iced::advanced::graphics::futures::MaybeSend + 'static,
    ) -> (Self, iced::Task<O>) {
        let monitor_id = MonitorId::from(&input.output_name);

        let (win, win_comp_task) =
            win_comp::NiriWinComp::new(win_comp::Init { monitor_id }, Message::Win);
        let (niri_serv, niri_serv_task) = state_serv::NiriStateServ::new((), Message::NiriService);

        let (clock, clock_task) = clock_comp::Clock::new((), Message::Clock);
        let (date, date_task) = date_comp::Date::new((), Message::Date);
        let (eth, eth_task) = cmd::CmdComp::new(
            cmd::Init {
                cmd: "crypto-egg-go".to_owned(),
                args: vec!["price", "eth"].into_owned_vec(),
                interval: 1,
            },
            Message::Eth,
        );
        let (btc, btc_task) = cmd::CmdComp::new(
            cmd::Init {
                cmd: "crypto-egg-go".to_owned(),
                args: vec!["price", "btc"].into_owned_vec(),
                interval: 1,
            },
            Message::Btc,
        );

        let inner_tasks = Task::batch([
            win_comp_task,
            niri_serv_task,
            clock_task,
            date_task,
            eth_task,
            btc_task,
        ]);

        (
            Self {
                output_name: input.output_name,
                position: input.position,
                win,
                niri_serv,
                clock,
                date,
                eth,
                btc,
            },
            inner_tasks.map(f),
        )
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        let clock = self.clock.subscription().map(Message::Clock);
        let date = self.date.subscription().map(Message::Date);
        let niri_win = self.win.subscription().map(Message::Win);
        let niri_serv = self.niri_serv.subscription().map(Message::NiriService);
        let eth = self.eth.subscription().map(Message::Eth);
        let btc = self.btc.subscription().map(Message::Btc);

        Subscription::batch([niri_win, niri_serv, clock, date, eth, btc])
    }

    fn update(&mut self, message: Self::Message) -> iced::Task<Self::Message> {
        match message {
            Message::Clock(message) => self.clock.update(message).map(Message::Clock),
            Message::Date(message) => self.date.update(message).map(Message::Date),
            Message::Win(message) => self.win.update(message).map(Message::Win),
            Message::NiriService(message) => {
                self.niri_serv.update(message).map(Message::NiriService)
            }
            Message::Eth(message) => self.eth.update(message).map(Message::Eth),
            Message::Btc(message) => self.btc.update(message).map(Message::Btc),
        }
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        let theme = &CAT_THEME;
        let spacing = theme.spacing();

        let date_view =
            container(self.date.view(theme.background()).map(Message::Date)).center_y(Length::Fill);

        let win = {
            let div = Semi::new(
                theme.rosewater(),
                theme.lavender(),
                Direction::Left,
                theme.spacing().xl(),
            );

            let view = self
                .win
                .view(win_comp::Props {
                    color: theme.rosewater(),
                    next_color: theme.trans(),
                    state: &self.niri_serv,
                })
                .map(Message::Win);
            row![div, view]
        };

        let clock_view = container(self.clock.view(theme.background()).map(Message::Clock))
            .center_y(Length::Fill)
            .padding(padding::right(theme.spacing().sm()));

        let right_cap = Angled::new(
            theme.overlay2(),
            theme.trans(),
            Direction::Left,
            Heading::South,
            spacing.xl(),
        );

        let bitcoin = {
            let txt = self.btc.output();
            let txt = text!("{txt}").color(theme.text_color()).bold();

            let icon = iced_font_awesome::fa_icon_brands("bitcoin")
                .size(spacing.md())
                .color(theme.peach());

            let icon = container(icon)
                .center_y(Length::Fill)
                .padding(padding::right(spacing.xs()));

            let view = container(row![icon, txt])
                .center_y(Length::Fill)
                .padding(padding::horizontal(spacing.sm()))
                .background(theme.overlay2());

            let div = Semi::new(
                theme.overlay2(),
                theme.overlay1(),
                Direction::Right,
                spacing.xl(),
            );
            container(row![view, div])
        };

        let eth = {
            let txt = self.eth.output();
            let txt = text!("{txt}").color(theme.text_color()).bold();

            let icon = iced_font_awesome::fa_icon_brands("ethereum")
                .size(spacing.md())
                .color(theme.sapphire());

            let icon = container(icon)
                .center_y(Length::Fill)
                .padding(padding::right(spacing.xs()));

            let view = container(row![icon, txt])
                .center_y(Length::Fill)
                .padding(padding::left(spacing.sm()))
                .background(theme.overlay1());

            let div = Semi::new(
                theme.overlay1(),
                theme.trans(),
                Direction::Right,
                spacing.xl(),
            );

            container(row![view, div]).padding(padding::right(spacing.sm()))
        };

        let pad = {
            let pad = padding::left(theme.spacing().md());
            match self.position {
                Position::Top => pad.top(spacing.sm()),
                Position::Bottom => pad.bottom(spacing.sm()),
            }
        };

        bar_widgets!(
            center: date_view, win, clock_view;
            right: right_cap, bitcoin, eth,
        )
        .background(theme.trans())
        .padding(pad)
        .center_y(Length::Fill)
        .into()
    }
}

impl DeloraSec {
    pub fn is_on_output(&self, output_name: &str) -> bool {
        self.output_name == output_name
    }

    pub fn is_position(&self, position: Position) -> bool {
        self.position == position
    }
}

impl Feature for DeloraSec {
    type Settings = iced_layershell::reexport::NewLayerShellSettings;
    fn layer(&self) -> iced_layershell::reexport::NewLayerShellSettings {
        let output_name = self.output_name.clone();
        let height = CAT_THEME.spacing().xl() + CAT_THEME.spacing().xs();
        let anchor = match self.position {
            Position::Top => Anchor::Left | Anchor::Top | Anchor::Right,
            Position::Bottom => Anchor::Left | Anchor::Bottom | Anchor::Right,
        };

        NewLayerShellSettings {
            layer: Layer::Top,
            size: Some((0, height as u32)),
            anchor,
            keyboard_interactivity: KeyboardInteractivity::None,
            exclusive_zone: Some(height as i32),
            output_option: OutputOption::OutputName(output_name),
            events_transparent: false,
            namespace: Some("DeloraSecBar".into()),
            margin: None,
        }
    }
}
