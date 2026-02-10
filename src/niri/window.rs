use freedesktop_icons::lookup;
use iced::{
    Color, Element, Length,
    advanced::{image, svg},
    padding,
    widget::{Image, Svg, container, row, space, text},
};

use crate::{
    config::MonitorId,
    divider::{Angled, Direction, Heading},
    feature::{CompWithProps, align_center},
    fira_fonts::TextExt,
    niri::state,
    theme::{AppTheme, Shade, app_theme},
};

#[derive(Debug, Clone)]
pub enum Message {}

pub struct Init {
    pub monitor_id: MonitorId,
}

pub struct NiriWin {
    mon: MonitorId,
    theme: AppTheme,
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
            theme: app_theme(),
            mon: input.monitor_id,
        }
    }

    fn view<'a>(
        &self,
        Props { state, color }: Self::Props<'a>,
    ) -> iced::Element<'_, Self::Message> {
        let theme = &self.theme;
        let second_color = theme.info(Shade::S500);
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
            .and_then(|app_id| lookup(&app_id).find())
            .map(|path| {
                if path.extension().is_some_and(|ext| ext == "svg") {
                    Element::from(
                        Svg::new(svg::Handle::from_path(path))
                            .height(theme.spacing().lg())
                            .width(theme.spacing().lg()),
                    )
                } else {
                    Element::from(
                        Image::new(image::Handle::from_path(path)).height(theme.spacing().lg()),
                    )
                }
            })
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
            align_center!(text!("{title}").color(theme.neutral(Shade::S700)).bold())
        ])
        .style(move |_| container::Style {
            background: Some(color.into()),
            ..Default::default()
        })
        .padding(padding::horizontal(theme.spacing().sm()));

        let mid_div = align_center!(Angled::new(
            color,
            Direction::Right,
            Heading::South,
            theme.spacing().xl(),
        ))
        .style(move |_| container::Style {
            background: Some(second_color.into()),
            ..Default::default()
        });

        let count_cont = align_center!(text!("{current_win_idx}/{num_of_win}"))
            .style(move |_| container::Style {
                background: Some(second_color.into()),
                ..Default::default()
            })
            .padding(padding::horizontal(theme.spacing().sm()));

        // TODO: add end color
        let end_div = align_center!(Angled::new(
            second_color,
            Direction::Right,
            Heading::North,
            theme.spacing().xl(),
        ));

        row![title_cont, mid_div, count_cont, end_div].into()
    }
}
