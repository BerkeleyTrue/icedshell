use iced::{Color, theme::Palette};

use crate::theme::{
    BASE, BLUE, CRUST, FLAMINGO, GREEN, LAVENDER, MANTLE, MAROON, MAUVE, OVERLAY0, OVERLAY1,
    OVERLAY2, PEACH, PINK, RED, ROSEWATER, SAPPHIRE, SKY, SUBTEXT0, SUBTEXT1, SURFACE0, SURFACE1,
    SURFACE2, TEAL, TEXT, YELLOW, radius::RadiusScale, spacing::SpacingScale,
};

#[derive(Debug, Clone)]
pub struct AppTheme {
    spacing: SpacingScale,
    radius: RadiusScale,
}

impl AppTheme {
    pub const fn new(spacing: SpacingScale, radius: RadiusScale) -> Self {
        Self { spacing, radius }
    }

    #[must_use]
    pub const fn spacing(&self) -> &SpacingScale {
        &self.spacing
    }

    #[must_use]
    pub const fn radius(&self) -> &RadiusScale {
        &self.radius
    }

    /// Transparent color
    #[must_use]
    pub const fn trans(&self) -> Color {
        Color::TRANSPARENT
    }

    // Catppuccin Frappe color accessors

    /// #f2d5cf
    #[must_use]
    pub const fn rosewater(&self) -> Color {
        ROSEWATER
    }

    /// #eebebe
    #[must_use]
    pub const fn flamingo(&self) -> Color {
        FLAMINGO
    }

    /// #f4b8e4
    #[must_use]
    pub const fn pink(&self) -> Color {
        PINK
    }

    /// #ca9ee6
    #[must_use]
    pub const fn mauve(&self) -> Color {
        MAUVE
    }

    /// #e78284
    #[must_use]
    pub const fn red(&self) -> Color {
        RED
    }

    /// #ea999c
    #[must_use]
    pub const fn maroon(&self) -> Color {
        MAROON
    }

    /// #ef9f76
    #[must_use]
    pub const fn peach(&self) -> Color {
        PEACH
    }

    /// #e5c890
    #[must_use]
    pub const fn yellow(&self) -> Color {
        YELLOW
    }

    /// #a6d189
    #[must_use]
    pub const fn green(&self) -> Color {
        GREEN
    }

    /// #81c8be
    #[must_use]
    pub const fn teal(&self) -> Color {
        TEAL
    }

    /// #99d1db
    #[must_use]
    pub const fn sky(&self) -> Color {
        SKY
    }

    /// #85c1dc
    #[must_use]
    pub const fn sapphire(&self) -> Color {
        SAPPHIRE
    }

    /// #8caaee
    #[must_use]
    pub const fn blue(&self) -> Color {
        BLUE
    }

    /// #babbf1
    #[must_use]
    pub const fn lavender(&self) -> Color {
        LAVENDER
    }

    /// #c6d0f5
    #[must_use]
    pub const fn text_color(&self) -> Color {
        TEXT
    }

    /// #b5bfe2
    #[must_use]
    pub const fn subtext1(&self) -> Color {
        SUBTEXT1
    }

    /// #a5adce
    #[must_use]
    pub const fn subtext0(&self) -> Color {
        SUBTEXT0
    }

    /// #949cbb
    #[must_use]
    pub const fn overlay2(&self) -> Color {
        OVERLAY2
    }

    /// #838ba7
    #[must_use]
    pub const fn overlay1(&self) -> Color {
        OVERLAY1
    }

    /// #737994
    #[must_use]
    pub const fn overlay0(&self) -> Color {
        OVERLAY0
    }

    /// #626880
    #[must_use]
    pub const fn surface2(&self) -> Color {
        SURFACE2
    }

    /// #51576d
    #[must_use]
    pub const fn surface1(&self) -> Color {
        SURFACE1
    }

    /// #414559
    #[must_use]
    pub const fn surface0(&self) -> Color {
        SURFACE0
    }

    /// #303446
    #[must_use]
    pub const fn base(&self) -> Color {
        BASE
    }

    /// #292c3c
    #[must_use]
    pub const fn mantle(&self) -> Color {
        MANTLE
    }

    /// #232634
    #[must_use]
    pub const fn crust(&self) -> Color {
        CRUST
    }

    // Semantic color helpers

    /// Background color (#303446 / BASE).
    #[must_use]
    pub const fn background(&self) -> Color {
        BASE
    }

    /// Surface color, slightly elevated background (#414559 / SURFACE0).
    #[must_use]
    pub const fn surface(&self) -> Color {
        SURFACE0
    }

    /// Muted text color, lower contrast (#838ba7 / OVERLAY1).
    #[must_use]
    pub const fn text_muted(&self) -> Color {
        OVERLAY1
    }

    /// Border color (#51576d / SURFACE1).
    #[must_use]
    pub const fn border(&self) -> Color {
        SURFACE1
    }

    /// iced layershell daemon theme
    pub fn theme(&self) -> iced::Theme {
        let palette = Palette {
            background: BASE,
            text: TEXT,
            primary: LAVENDER,
            success: TEAL,
            warning: PEACH,
            danger: MAROON,
        };
        iced::Theme::custom("catppuccin", palette)
    }
}
