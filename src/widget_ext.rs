use iced::widget::{Container, container};

pub trait ContainExt<'a, Message: 'a> {
    fn styl(self, style: container::Style) -> Container<'a, Message>;
}

impl<'a, Message: 'a> ContainExt<'a, Message> for Container<'a, Message> {
    fn styl(self, style: container::Style) -> Container<'a, Message> {
        self.style(move |_theme| style)
    }
}
