#![allow(unused_variables, dead_code)]
use iced::theme::Palette;

pub use crate::theme::{
    color::{ColorPalette, Shade},
    radius::RadiusScale,
    spacing::SpacingScale,
};

#[derive(Debug, Clone)]
pub struct Tokens {
    spacing: SpacingScale,
    colors: ColorPalette,
    radius: RadiusScale,
}

impl Tokens {
    pub fn new(spacing: SpacingScale, colors: ColorPalette, radius: RadiusScale) -> Self {
        Self {
            spacing,
            colors,
            radius,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AppTheme {
    tokens: Tokens,
}

impl AppTheme {
    pub fn new(tokens: Tokens) -> Self {
        Self { tokens }
    }

    #[must_use]
    pub fn tokens(&self) -> &Tokens {
        &self.tokens
    }

    #[must_use]
    pub fn spacing(&self) -> &SpacingScale {
        &self.tokens.spacing
    }

    #[must_use]
    pub fn colors(&self) -> &ColorPalette {
        &self.tokens.colors
    }

    #[must_use]
    pub fn radius(&self) -> &RadiusScale {
        &self.tokens.radius
    }

    // Color helpers

    #[must_use]
    pub fn primary(&self, shade: Shade) -> iced::Color {
        self.tokens.colors.primary.get(shade)
    }

    #[must_use]
    pub fn secondary(&self, shade: Shade) -> iced::Color {
        self.tokens.colors.secondary.get(shade)
    }

    #[must_use]
    pub fn neutral(&self, shade: Shade) -> iced::Color {
        self.tokens.colors.neutral.get(shade)
    }

    #[must_use]
    pub fn success(&self, shade: Shade) -> iced::Color {
        self.tokens.colors.success.get(shade)
    }

    #[must_use]
    pub fn warning(&self, shade: Shade) -> iced::Color {
        self.tokens.colors.warning.get(shade)
    }

    #[must_use]
    pub fn destructive(&self, shade: Shade) -> iced::Color {
        self.tokens.colors.destructive.get(shade)
    }

    #[must_use]
    pub fn info(&self, shade: Shade) -> iced::Color {
        self.tokens.colors.info.get(shade)
    }

    #[must_use]
    pub fn background(&self) -> iced::Color {
        self.neutral(Shade::S900)
    }

    /// Get the surface color (slightly elevated background).
    #[must_use]
    pub fn surface(&self) -> iced::Color {
        self.neutral(Shade::S800)
    }

    /// Get the text color (high contrast against background).
    #[must_use]
    pub fn text(&self) -> iced::Color {
        self.neutral(Shade::S50)
    }

    /// Get the muted text color (lower contrast).
    #[must_use]
    pub fn text_muted(&self) -> iced::Color {
        self.neutral(Shade::S400)
    }

    /// Get the border color.
    #[must_use]
    pub fn border(&self) -> iced::Color {
        self.neutral(Shade::S700)
    }

    /// iced layershell daemon themefn
    pub fn theme(&self) -> iced::Theme {
        let color_palette = self.colors().clone();
        let palette: Palette = color_palette.into();
        iced::Theme::custom("catppuccin", palette)
    }
}
