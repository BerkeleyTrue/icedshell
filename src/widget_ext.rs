#![allow(unused_variables, dead_code)]
use iced::{
    Color, Element, Theme, border,
    widget::{Container, Tooltip, container, tooltip},
};

use crate::theme::CAT_THEME;

pub fn debug_border(_: &Theme) -> container::Style {
    let theme = &CAT_THEME;
    container::Style {
        border: border::width(2).color(theme.red()),
        ..Default::default()
    }
}

pub trait ContainExt<'a, Message: 'a> {
    fn debug_border(self) -> Self;
    fn background(self, color: Color) -> Self;
    fn tooltip(
        self,
        position: tooltip::Position,
        content: impl Into<Element<'a, Message>>,
    ) -> Tooltip<'a, Message>;
    fn maybe_tooltip(
        self,
        position: tooltip::Position,
        content: Option<impl Into<Element<'a, Message>>>,
    ) -> Element<'a, Message>;
}

impl<'a, Message: 'a> ContainExt<'a, Message> for Container<'a, Message> {
    fn debug_border(self) -> Self {
        self.style(debug_border)
    }
    fn background(self, color: Color) -> Self {
        self.style(move |_| container::Style {
            background: Some(color.into()),
            ..Default::default()
        })
    }

    fn tooltip(
        self,
        position: tooltip::Position,
        content: impl Into<Element<'a, Message>>,
    ) -> Tooltip<'a, Message> {
        tooltip(self, content, position)
    }

    fn maybe_tooltip(
        self,
        position: tooltip::Position,
        content: Option<impl Into<Element<'a, Message>>>,
    ) -> Element<'a, Message> {
        if let Some(content) = content {
            tooltip(self, content, position).into()
        } else {
            self.into()
        }
    }
}
