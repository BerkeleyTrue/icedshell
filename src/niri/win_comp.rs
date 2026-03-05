use iced::{
    Color, Element, Length, Task, padding,
    widget::{container, row, space, text},
};

use crate::{
    config::MonitorId,
    divider::{Angled, Direction, Heading},
    fdo_icons,
    feature::{CompWithProps, align_center},
    fira_fonts::TextExt,
    niri::state_serv,
    theme::CAT_THEME,
    widget_ext::ContainExt,
};

#[derive(Debug, Clone)]
pub enum Message {}

pub struct Init {
    pub monitor_id: MonitorId,
}

pub struct NiriWinComp {
    mon: MonitorId,
}

pub struct Props<'a> {
    pub color: Color,
    pub next_color: Color,
    pub state: &'a state_serv::NiriStateServ,
}

impl CompWithProps for NiriWinComp {
    type Props<'a> = Props<'a>;
    type Init = Init;
    type Message = Message;
    fn new(input: Self::Init) -> (Self, Task<Self::Message>) {
        Self {
            mon: input.monitor_id,
        }
        .to_tuple()
    }

    fn view<'a>(
        &self,
        Props {
            state,
            color,
            next_color,
        }: Self::Props<'a>,
    ) -> iced::Element<'_, Self::Message> {
        let theme = &CAT_THEME;
        let second_color = theme.blue();
        let maybe_ws = state
            .iter_ws()
            .find(move |ws| ws.monitor_id == Some(self.mon.clone()) && ws.is_active);

        let maybe_win = maybe_ws
            .and_then(|ws| ws.active_win_id.as_ref())
            .and_then(move |win_id| state.get_win(win_id));

        let mut title = maybe_win
            .and_then(|win| win.title.clone())
            .unwrap_or("()".to_string());

        let app_icon = maybe_win
            .and_then(|win| win.app_id.clone())
            .and_then(|app_id| fdo_icons::find(&app_id))
            .map(|fdo_icon| fdo_icon.elem(theme.spacing().lg()))
            .map(|icon| {
                container(icon)
                    .padding(padding::right(theme.spacing().xs()))
                    .center_y(Length::Fill)
                    .into()
            })
            .unwrap_or(Element::from(space()));

        if title.len() >= 9 {
            title = format!("{}...", title.chars().take(9).collect::<String>());
        }

        let num_of_win = maybe_ws
            .map(|ws| ws.id.clone())
            .map(move |workspace_id| {
                state
                    .iter_win()
                    .filter(|win| {
                        let workspace_id = &workspace_id.clone();
                        win.ws_id
                            .as_ref()
                            .is_some_and(move |win_ws_id| win_ws_id == workspace_id)
                    })
                    .count()
            })
            .unwrap_or_default();

        let current_win_idx = maybe_win
            .and_then(|win| win.col_idx.clone())
            .unwrap_or_default();

        let title_cont = align_center!(row![
            app_icon,
            align_center!(text!("{title}").color(theme.surface1()).bold())
        ])
        .background(color)
        .padding(padding::horizontal(theme.spacing().sm()));

        let mid_div = align_center!(Angled::new(
            color,
            Direction::Right,
            Heading::South,
            theme.spacing().xl(),
        ))
        .background(second_color);

        let count_cont = align_center!(text!("{current_win_idx}/{num_of_win}"))
            .background(second_color)
            .padding(padding::horizontal(theme.spacing().sm()));

        let end_div = align_center!(Angled::new(
            second_color,
            Direction::Right,
            Heading::North,
            theme.spacing().xl(),
        ))
        .background(next_color);

        row![title_cont, mid_div, count_cont, end_div].into()
    }
}
