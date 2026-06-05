use iced::{
    Length, Subscription, Task,
    alignment::Vertical,
    padding,
    widget::{container, row, text},
};
use iced_layershell::reexport::{
    Anchor, KeyboardInteractivity, Layer, NewLayerShellSettings, OutputOption,
};
use lucide_icons::iced::{self as lucide};

use crate::{
    audio, cmd,
    datetime::{clock_comp, date_comp},
    feature::{Comp, CompWithProps, Feature, Service},
    niri::{state_serv, win_comp},
    system_info::{self, BatteryState},
    theme::CAT_THEME,
    types::MonitorId,
    widget::{
        IntoIteratorExt, align_center, bar_widgets,
        container_ext::ContainExt,
        divider::{Angled, Direction, Heading, Semi},
        text_ext::TextExt,
    },
};

#[derive(Debug, Clone)]
pub enum Message {
    Win(win_comp::Message),
    NiriService(state_serv::Message),
    Clock(clock_comp::Message),
    Date(date_comp::Message),
    Eth(cmd::Message),
    Btc(cmd::Message),
    SysInfo(system_info::Message),
    Conn(cmd::Message),
    Audio(audio::Message),
}

pub struct RenaSec {
    win: win_comp::NiriWinComp,
    output_name: String,
    niri_serv: state_serv::NiriStateServ,
    clock: clock_comp::Clock,
    date: date_comp::Date,
    eth: cmd::CmdComp,
    btc: cmd::CmdComp,
    sys_info: system_info::SysInfoComp,
    conn: cmd::CmdComp,
    audio: audio::PulseAudio,
}

pub struct Init {
    pub output_name: String,
}

impl Comp for RenaSec {
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

        let (sys_info, sys_info_task) = system_info::SysInfoComp::new(
            system_info::Init {
                bat_name: Some("BAT1".to_owned()),
            },
            Message::SysInfo,
        );
        let (conn, conn_task) = cmd::CmdComp::new(
            cmd::Init {
                cmd: "connectivity".to_owned(),
                args: Vec::default(),
                interval: 1,
            },
            Message::Conn,
        );
        let (audio, audio_task) = audio::PulseAudio::new((), Message::Audio);

        let inner_tasks = Task::batch([
            win_comp_task,
            niri_serv_task,
            clock_task,
            date_task,
            eth_task,
            btc_task,
            sys_info_task,
            conn_task,
            audio_task,
        ]);

        (
            Self {
                output_name: input.output_name,
                win,
                niri_serv,
                clock,
                date,
                eth,
                btc,
                sys_info,
                conn,
                audio,
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
        let conn = self.conn.subscription().map(Message::Conn);
        let sys_info = self.sys_info.subscription().map(Message::SysInfo);
        let audio = self.audio.subscription().map(Message::Audio);

        Subscription::batch([
            niri_win, niri_serv, clock, date, eth, btc, conn, sys_info, audio,
        ])
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
            Message::Conn(message) => self.conn.update(message).map(Message::Conn),
            Message::SysInfo(message) => self.sys_info.update(message).map(Message::SysInfo),
            Message::Audio(message) => self.audio.update(message).map(Message::Audio),
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
                spacing.xl(),
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
            .padding(padding::right(spacing.sm()));

        let right_cap = Semi::new(
            theme.overlay2(),
            theme.trans(),
            Direction::Left,
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
                .padding(padding::horizontal(spacing.sm()))
                .background(theme.overlay1());

            let div = Angled::new(
                theme.overlay1(),
                theme.trans(),
                Direction::Right,
                Heading::South,
                spacing.xl(),
            );

            container(row![view, div]).padding(padding::right(spacing.sm()))
        };

        let bat = {
            let bat_state = self.sys_info.bat_stat();

            let color = match bat_state {
                BatteryState::Discharging(cap) => {
                    if cap < &60.0 {
                        theme.yellow()
                    } else {
                        theme.text_color()
                    }
                }
                BatteryState::Charging(_) => theme.green(),
                BatteryState::Low(_) => theme.red(),
                BatteryState::Full | BatteryState::None => theme.text_color(),
            };
            let text = match bat_state {
                BatteryState::None => "N/A".to_owned(),
                BatteryState::Full => "100".to_owned(),
                BatteryState::Charging(cap)
                | BatteryState::Discharging(cap)
                | BatteryState::Low(cap) => format!("{}", cap),
            };
            let text = text!("{text}").color(color);
            let icon = match bat_state {
                BatteryState::None => lucide::icon_battery_warning(),
                BatteryState::Charging(_) => lucide::icon_battery_charging(),
                BatteryState::Full => lucide::icon_battery_full(),
                BatteryState::Discharging(cap) => {
                    if cap > &90.0 {
                        lucide::icon_battery_full()
                    } else {
                        lucide::icon_battery_medium()
                    }
                }
                BatteryState::Low(_) => lucide::icon_battery_low(),
            }
            .size(spacing.md())
            .center()
            .color(color);

            let div = Angled::new(
                theme.surface0(),
                theme.trans(),
                Direction::Left,
                Heading::South,
                spacing.xl(),
            );

            let content = align_center!(
                row![icon, text]
                    .spacing(spacing.xs())
                    .align_y(Vertical::Center)
            )
            .padding(padding::horizontal(spacing.sm()))
            .background(theme.surface0());

            row![div, content].align_y(Vertical::Center)
        };

        let audio = {
            let vol = self.audio.get_vol();
            let muted = self.audio.get_muted();

            let icon = match (muted, vol) {
                (true, _) => lucide::icon_volume_off(),
                (_, val) if val > 60 => lucide::icon_volume_2(),
                (_, val) if val > 20 => lucide::icon_volume_1(),
                (_, _) => lucide::icon_volume(),
            }
            .center()
            .size(spacing.md())
            .color(theme.base())
            .bold();

            let text = align_center!(
                row![icon, text!("{vol}%").color(theme.base()).bold(),]
                    .align_y(Vertical::Center)
                    .spacing(spacing.xxs()),
            )
            .padding(padding::horizontal(spacing.sm()));

            let div = Angled::new(
                theme.surface0(),
                theme.green(),
                Direction::Right,
                Heading::South,
                spacing.xl(),
            );

            align_center!(row![div, text]).background(theme.green())
        };

        let conn = {
            let icon = if !self.conn.is_error() {
                lucide::icon_globe().color(theme.surface2())
            } else {
                lucide::icon_globe_x().color(theme.red())
            }
            .center()
            .size(spacing.md())
            .bold();

            let icon = container(icon).padding(padding::horizontal(spacing.sm()));

            let div = Angled::new(
                theme.green(),
                theme.lavender(),
                Direction::Right,
                Heading::South,
                spacing.xl(),
            );
            container(row![div, icon]).background(theme.lavender())
        };

        let disk_usage = {
            let div = Semi::new(
                theme.lavender(),
                theme.trans(),
                Direction::Right,
                spacing.xl(),
            );

            let icon = lucide::icon_hard_drive()
                .size(spacing.md())
                .center()
                .color(theme.base());

            let text = self.sys_info.disk_usage();
            let text = text!("{text}").color(theme.base()).bold();

            let main = align_center!(
                row![icon, text]
                    .align_y(Vertical::Center)
                    .spacing(spacing.xxs()),
            )
            .background(theme.trans())
            .padding(padding::left(spacing.sm()));

            align_center!(row![div, main])
        };

        bar_widgets!(
            left: right_cap, bitcoin, eth;
            center: date_view, win, clock_view;
            right: bat, audio, conn, disk_usage;
        )
        .background(theme.trans())
        .padding(padding::horizontal(spacing.md()).bottom(spacing.sm()))
        .center_y(Length::Fill)
        .into()
    }
}

impl RenaSec {
    pub fn is_on_output(&self, output_name: &str) -> bool {
        self.output_name == output_name
    }
}

impl Feature for RenaSec {
    type Settings = iced_layershell::reexport::NewLayerShellSettings;
    fn layer(&self) -> iced_layershell::reexport::NewLayerShellSettings {
        let output_name = self.output_name.clone();
        let height = CAT_THEME.spacing().xl() + CAT_THEME.spacing().xs();

        NewLayerShellSettings {
            layer: Layer::Top,
            size: Some((0, height as u32)),
            anchor: Anchor::Left | Anchor::Bottom | Anchor::Right,
            keyboard_interactivity: KeyboardInteractivity::None,
            exclusive_zone: Some(height as i32),
            output_option: OutputOption::OutputName(output_name),
            events_transparent: false,
            namespace: Some("RenaSecBar".into()),
            margin: None,
        }
    }
}
