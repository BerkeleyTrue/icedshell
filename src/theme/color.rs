use iced::{Color, theme::Palette};

#[derive(Debug, Clone)]
pub enum Shade {
    S50 = 0,
    S100 = 1,
    S200 = 2,
    S300 = 3,
    S400 = 4,
    S500 = 5,
    S600 = 6,
    S700 = 7,
    S800 = 8,
    S900 = 9,
}

impl Shade {
    pub const fn value(self) -> u16 {
        match self {
            Self::S50 => 50,
            Self::S100 => 100,
            Self::S200 => 200,
            Self::S300 => 300,
            Self::S400 => 400,
            Self::S500 => 500,
            Self::S600 => 600,
            Self::S700 => 700,
            Self::S800 => 800,
            Self::S900 => 900,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ColorScale {
    /// Shade 50 - lightest
    pub s50: Color,
    pub s100: Color,
    pub s200: Color,
    pub s300: Color,
    pub s400: Color,
    pub s500: Color,
    pub s600: Color,
    pub s700: Color,
    pub s800: Color,
    /// Shade 900 - darkest
    pub s900: Color,
}

impl ColorScale {
    pub const fn new(
        s50: Color,
        s100: Color,
        s200: Color,
        s300: Color,
        s400: Color,
        s500: Color,
        s600: Color,
        s700: Color,
        s800: Color,
        s900: Color,
    ) -> Self {
        Self {
            s50,
            s100,
            s200,
            s300,
            s400,
            s500,
            s600,
            s700,
            s800,
            s900,
        }
    }

    pub const fn get(&self, shade: Shade) -> Color {
        match shade {
            Shade::S50 => self.s50,
            Shade::S100 => self.s100,
            Shade::S200 => self.s200,
            Shade::S300 => self.s300,
            Shade::S400 => self.s400,
            Shade::S500 => self.s500,
            Shade::S600 => self.s600,
            Shade::S700 => self.s700,
            Shade::S800 => self.s800,
            Shade::S900 => self.s900,
        }
    }
    /// Get the base color (shade 500).
    pub const fn base(&self) -> Color {
        self.s500
    }

    /// Get a light variant (shade 100).
    pub const fn light(&self) -> Color {
        self.s100
    }

    /// Get a dark variant (shade 700).
    pub const fn dark(&self) -> Color {
        self.s700
    }
}

#[derive(Debug, Clone)]
pub struct ColorPalette {
    pub primary: ColorScale,
    pub secondary: ColorScale,
    pub neutral: ColorScale,
    pub info: ColorScale,
    pub success: ColorScale,
    pub warning: ColorScale,
    pub destructive: ColorScale,
}

impl From<ColorPalette> for Palette {
    fn from(value: ColorPalette) -> Self {
        Palette {
            background: value.neutral.s900,
            text: value.neutral.s50,
            primary: value.primary.s500,
            success: value.success.s500,
            warning: value.warning.s500,
            danger: value.destructive.s500,
        }
    }
}
