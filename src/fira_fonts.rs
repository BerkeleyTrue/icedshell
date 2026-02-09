use iced::{Font, widget::Text};

pub const REGULAR_BYTES: &[u8] =
    include_bytes!("../resources/fonts/FiraCodeNerdFontMono-Regular.ttf");
pub const BOLD_BYTES: &[u8] = include_bytes!("../resources/fonts/FiraCodeNerdFontMono-Bold.ttf");

pub const FIRA_CODE: Font = Font::with_name("FiraCode Nerd Font Mono");
pub const FIRA_CODE_BOLD: Font = Font {
    weight: iced::font::Weight::Bold,
    ..FIRA_CODE
};

pub trait TextExt {
    fn bold(self) -> Self;
}

impl<'a> TextExt for Text<'a> {
    fn bold(self) -> Self {
        self.font(FIRA_CODE_BOLD)
    }
}
