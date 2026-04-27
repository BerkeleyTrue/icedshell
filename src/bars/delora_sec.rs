use iced::{
    Length, Subscription, Task, padding,
    widget::{container, row},
};
use iced_layershell::reexport::{
    Anchor, KeyboardInteractivity, Layer, NewLayerShellSettings, OutputOption,
};

use crate::{
    datetime::{clock_comp, date_comp},
    feature::{Comp, CompWithProps, Feature, Service},
    niri::{state_serv, win_comp},
    theme::CAT_THEME,
    types::MonitorId,
    widget::{
        bar_widgets, center_widgets,
        container_ext::ContainExt,
        divider::{Direction, Semi},
    },
};

pub struct DeloraSec {
    win: win_comp::NiriWinComp,
    output_name: String,
    niri_serv: state_serv::NiriStateServ,
    clock: clock_comp::Clock,
    date: date_comp::Date,
}

#[derive(Debug, Clone)]
pub enum Message {
    Win(win_comp::Message),
    NiriService(state_serv::Message),
    Clock(clock_comp::Message),
    Date(date_comp::Message),
}

pub struct Init {
    pub output_name: String,
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

        let inner_tasks = Task::batch([win_comp_task, niri_serv_task, clock_task, date_task]);

        (
            Self {
                output_name: input.output_name,
                win,
                niri_serv,
                clock,
                date,
            },
            inner_tasks.map(f),
        )
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        let clock = self.clock.subscription().map(Message::Clock);
        let date = self.date.subscription().map(Message::Date);
        let niri_win = self.win.subscription().map(Message::Win);
        let niri_serv = self.niri_serv.subscription().map(Message::NiriService);
        Subscription::batch([niri_win, niri_serv, clock, date])
    }

    fn update(&mut self, message: Self::Message) -> iced::Task<Self::Message> {
        match message {
            Message::Clock(message) => self.clock.update(message).map(Message::Clock),
            Message::Date(message) => self.date.update(message).map(Message::Date),
            Message::Win(message) => self.win.update(message).map(Message::Win),
            Message::NiriService(message) => {
                self.niri_serv.update(message).map(Message::NiriService)
            }
        }
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        let theme = &CAT_THEME;
        let spacing = theme.spacing();

        let date_view =
            container(self.date.view(theme.background()).map(Message::Date)).center_y(Length::Fill);

        let win_div = Semi::new(
            theme.rosewater(),
            theme.lavender(),
            Direction::Left,
            theme.spacing().xl(),
        );

        let win_view = self
            .win
            .view(win_comp::Props {
                color: theme.rosewater(),
                next_color: theme.trans(),
                state: &self.niri_serv,
            })
            .map(Message::Win);

        let clock_view = container(self.clock.view(theme.background()).map(Message::Clock))
            .center_y(Length::Fill)
            .padding(padding::right(theme.spacing().sm()));

        bar_widgets!(
            center: date_view, win_div, win_view, clock_view;
        )
        .background(theme.trans())
        .padding(padding::left(theme.spacing().md()).top(spacing.sm()))
        .center_y(Length::Fill)
        .into()
    }
}

impl DeloraSec {
    pub fn is_on_output(&self, output_name: &str) -> bool {
        self.output_name == output_name
    }
}

impl Feature for DeloraSec {
    type Settings = iced_layershell::reexport::NewLayerShellSettings;
    fn layer(&self) -> iced_layershell::reexport::NewLayerShellSettings {
        let output_name = self.output_name.clone();
        let height = CAT_THEME.spacing().xl() + CAT_THEME.spacing().xs();

        NewLayerShellSettings {
            layer: Layer::Top,
            size: Some((0, height as u32)),
            anchor: Anchor::Left | Anchor::Top | Anchor::Right,
            keyboard_interactivity: KeyboardInteractivity::None,
            exclusive_zone: Some(height as i32),
            output_option: OutputOption::OutputName(output_name),
            events_transparent: false,
            namespace: Some("DeloraSecBar".into()),
            margin: None,
        }
    }
}
