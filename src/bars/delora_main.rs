use iced::{
    Color, Length, Subscription, Task, advanced::graphics::futures::MaybeSend, padding, widget::row,
};
use iced_layershell::reexport::{
    Anchor, KeyboardInteractivity, Layer, NewLayerShellSettings, OutputOption,
};

use crate::{
    datetime::{clock_comp, date_comp},
    feature::{Comp, CompWithProps, Feature, Service},
    niri::{state_serv, win_comp, ws_comp},
    powermenu::button_comp,
    system_info as sys_info,
    theme::CAT_THEME,
    tray::{TrayLayout, TrayMenuItemId, service as tray_serv, tray_comp},
    types::MonitorId,
    widget::{
        align_center, bar_widgets,
        container_ext::ContainExt,
        divider::{Angled, Direction, Heading, Semi},
    },
};

#[derive(Debug, Clone)]
pub enum Message {
    Clock(clock_comp::Message),
    Date(date_comp::Message),

    Ws(ws_comp::Message),
    Win(win_comp::Message),

    NiriService(state_serv::Message),
    TrayService(tray_serv::Message),
    Tray(tray_comp::Message),
    OpenTrayMenu(
        /// sn item name
        String,
        /// menu layout
        TrayLayout,
    ),
    PowerButtonOnClicked,

    SysInfo(sys_info::Message),
    PowerBtn(button_comp::Message),
}

pub struct Init {
    pub output_name: String,
}

pub struct DeloraMain {
    height: f32,
    padding: f32,

    ws: ws_comp::NiriWsComp,
    win: win_comp::NiriWinComp,
    clock: clock_comp::Clock,
    date: date_comp::Date,
    output_name: String,
    niri_serv: state_serv::NiriStateServ,
    tray_serv: tray_serv::TrayService,
    tray: tray_comp::TrayComp,
    sys_info: sys_info::SysInfoComp,
    power_btn: button_comp::PowerButton,
}

impl DeloraMain {
    pub fn clone_servs(&mut self, old_bar: &DeloraMain) {
        self.niri_serv = old_bar.niri_serv.clone();
        self.tray_serv = old_bar.tray_serv.clone();
    }

    pub fn is_on_output(&self, output_name: &str) -> bool {
        self.output_name == output_name
    }
}

impl Comp for DeloraMain {
    type Message = Message;
    type Init = Init;

    fn new<O: MaybeSend + 'static>(
        input: Self::Init,
        f: impl Fn(Self::Message) -> O + MaybeSend + 'static,
    ) -> (Self, Task<O>) {
        let theme = &CAT_THEME;
        let height = theme.spacing().xl();
        let padding = theme.spacing().xs();
        let monitor_id = MonitorId::from(&input.output_name);
        let (ws, ws_task) = ws_comp::NiriWsComp::new(
            ws_comp::Init {
                main_mon: monitor_id.clone(),
            },
            Message::Ws,
        );
        let (win, win_comp_task) =
            win_comp::NiriWinComp::new(win_comp::Init { monitor_id }, Message::Win);
        let (clock, clock_task) = clock_comp::Clock::new((), Message::Clock);
        let (date, date_task) = date_comp::Date::new((), Message::Date);
        let (niri_serv, niri_serv_task) = state_serv::NiriStateServ::new((), Message::NiriService);
        let (tray_serv, tray_serv_task) = tray_serv::TrayService::new((), Message::TrayService);
        let (tray, tray_task) = tray_comp::TrayComp::new((), Message::Tray);
        let (sys_info, sys_info_task) = sys_info::SysInfoComp::new((), Message::SysInfo);
        let (power_btn, power_btn_task) = button_comp::PowerButton::new((), Message::PowerBtn);

        let inner_tasks = Task::batch([
            win_comp_task,
            ws_task,
            clock_task,
            date_task,
            niri_serv_task,
            tray_serv_task,
            tray_task,
            sys_info_task,
            power_btn_task,
        ]);

        (
            Self {
                height,
                padding,
                ws,
                win,
                clock,
                date,
                output_name: input.output_name,
                niri_serv,
                tray_serv,
                tray,
                sys_info,
                power_btn,
            },
            inner_tasks.map(f),
        )
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
            Message::Tray(message) => {
                let inner_task = self.tray.update(message.clone()).map(Message::Tray);
                let out_task = match message {
                    tray_comp::Message::SnItemClicked(name, layout) => {
                        Task::done(Message::OpenTrayMenu(name, layout))
                    } // _ => Task::none(),
                };
                inner_task.chain(out_task)
            }
            Message::OpenTrayMenu(_, _) => Task::none(),
            Message::SysInfo(message) => self.sys_info.update(message).map(Message::SysInfo),
            Message::PowerBtn(message) => {
                let inner_task = self.power_btn.update(message).map(Message::PowerBtn);
                let out_task = match message {
                    button_comp::Message::OnClick => Task::done(Message::PowerButtonOnClicked),
                    // _ => Task::none(),
                };
                inner_task.chain(out_task)
            }
            Message::PowerButtonOnClicked => Task::none(),
        }
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        let clock = self.clock.subscription().map(Message::Clock);
        let date = self.date.subscription().map(Message::Date);
        let niri_serv = self.niri_serv.subscription().map(Message::NiriService);
        let niri_ws = self.ws.subscription().map(Message::Ws);
        let niri_win = self.win.subscription().map(Message::Win);
        let tray_serv = self.tray_serv.subscription().map(Message::TrayService);
        let tray = self.tray.subscription().map(Message::Tray);
        let sys_info = self.sys_info.subscription().map(Message::SysInfo);
        Subscription::batch([
            clock, date, niri_ws, niri_win, niri_serv, tray_serv, tray, sys_info,
        ])
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        let theme = &CAT_THEME;

        let date_view = align_center!(self.date.view(theme.background()).map(Message::Date));

        let niri_ws_view = align_center!(
            self.ws
                .view(ws_comp::Props {
                    state: &self.niri_serv,
                })
                .map(self::Message::Ws),
        );

        let div = Angled::new(
            theme.lavender(),
            theme.background(),
            Direction::Right,
            Heading::South,
            theme.spacing().xl(),
        );

        let clock_view = align_center!(self.clock.view(theme.background()).map(Message::Clock))
            .padding(padding::right(theme.spacing().sm()));

        let win_div = Semi::new(
            theme.rosewater(),
            theme.trans(),
            Direction::Left,
            theme.spacing().xl(),
        );

        let win = align_center!(
            self.win
                .view(win_comp::Props {
                    color: theme.rosewater(),
                    next_color: theme.surface2(),
                    state: &self.niri_serv,
                })
                .map(Message::Win),
        );

        let tray = self
            .tray
            .view(tray_comp::Props {
                serv: &self.tray_serv,
                next_color: Color::TRANSPARENT,
            })
            .map(Message::Tray);

        let sys_view = self.sys_info.view().map(Message::SysInfo);
        let power_btn = self.power_btn.view().map(Message::PowerBtn);

        // main bar
        bar_widgets!(
            left:  date_view, div, niri_ws_view;
            center: clock_view, win_div, win, tray;
            right: power_btn, sys_view
        )
        .background(Color::TRANSPARENT)
        .padding(padding::left(theme.spacing().md()).top(self.padding))
        .center_y(Length::Fill)
        .into()
    }
}

impl Feature for DeloraMain {
    type Settings = iced_layershell::reexport::NewLayerShellSettings;
    fn layer(&self) -> iced_layershell::reexport::NewLayerShellSettings {
        let output_name = self.output_name.clone();
        NewLayerShellSettings {
            layer: Layer::Top,
            size: Some((0, (self.height + self.padding) as u32)),
            anchor: Anchor::Left | Anchor::Top | Anchor::Right,
            keyboard_interactivity: KeyboardInteractivity::None,
            exclusive_zone: Some((self.height + self.padding) as i32),
            output_option: OutputOption::OutputName(output_name),
            events_transparent: false,
            namespace: Some("DeloraMainBar".into()),
            margin: None,
        }
    }
}

impl DeloraMain {
    pub fn tray_menu_item_clicked(&mut self, name: String, id: TrayMenuItemId) -> Task<Message> {
        self.tray_serv
            .menu_item_clicked(name, id)
            .map(Message::TrayService)
    }
}
