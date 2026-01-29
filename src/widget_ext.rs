use iced::{
    Border, Color,
    widget::{Container, container},
};

pub trait ContainExt<'a, Message: 'a> {
    fn background(self, color: Color) -> Container<'a, Message>;
    fn border(self, border: Border) -> Container<'a, Message>;
}

impl<'a, Message: 'a> ContainExt<'a, Message> for Container<'a, Message> {
    fn background(self, color: Color) -> Container<'a, Message> {
        self.style(move |_theme| container::Style {
            background: Some(color.into()),
            ..Default::default()
        })
    }

    fn border(self, border: Border) -> Container<'a, Message> {
        self.style(move |_theme| container::Style {
            border,
            ..Default::default()
        })
    }
}
