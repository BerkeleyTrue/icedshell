#![allow(unused_variables, dead_code)]
use iced::{
    Color, Theme, border,
    widget::{Container, container},
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
}
