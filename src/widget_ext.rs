use iced::widget::{Container, container};

pub trait ContainExt<'a, Message: 'a> {
}

impl<'a, Message: 'a> ContainExt<'a, Message> for Container<'a, Message> {
}
