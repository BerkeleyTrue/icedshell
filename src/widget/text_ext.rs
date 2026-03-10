use crate::fira_fonts::FIRA_CODE_BOLD;
use iced::widget::Text;

pub trait TextExt {
    fn bold(self) -> Self;
}

impl<'a> TextExt for Text<'a> {
    fn bold(self) -> Self {
        self.font(FIRA_CODE_BOLD)
    }
}
