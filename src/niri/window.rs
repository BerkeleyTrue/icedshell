use iced::{
    Color, Task, padding,
    widget::{container, text},
};

use crate::{config::MonitorId, feature::CompWithProps, niri::state, theme::app_theme};

#[derive(Debug, Clone)]
pub enum Message {}

pub struct Init {
    pub monitor_id: MonitorId,
}

pub struct NiriWin {
    mon: MonitorId,
}

pub struct Props<'a> {
    pub color: Color,
    pub state: &'a state::State,
}

impl CompWithProps for NiriWin {
    type Props<'a> = Props<'a>;
    type Init = Init;
    type Message = Message;
    fn new(input: Self::Init) -> Self {
        Self {
            mon: input.monitor_id,
        }
    }
    fn update(&mut self, _message: Self::Message) -> iced::Task<Self::Message> {
        Task::none()
    }
    fn view<'a>(
        &self,
        Props { state, color }: Self::Props<'a>,
    ) -> iced::Element<'_, Self::Message> {
        let theme = app_theme();
        let mut title = state
            .iter_ws()
            .find(move |ws| ws.monitor_id == Some(self.mon.clone()) && ws.is_active)
            .and_then(|ws| ws.active_win_id.as_ref())
            .and_then(move |win_id| state.get_win(win_id))
            .and_then(|win| win.title.clone())
            .unwrap_or("()".to_string());

        if title.len() >= 9 {
            title = format!("{}...", title.chars().take(9).collect::<String>());
        }

        container(text!("{title}").color(color))
            .padding(padding::horizontal(theme.spacing().sm()))
            .into()
    }
}
