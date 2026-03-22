use iced::{
    Background, Color, Gradient, Radians, color,
    gradient::{ColorStop, Linear},
    theme::Palette,
};

pub const DARK: Color = color!(0x170f2b); // #170f2b
pub const LIGHT: Color = color!(0xffe3de); // #ffe3de
pub const LIGHTLESS: Color = color!(0xffcfc6); // #ffcfc6
pub const PINK: Color = color!(0xff8bbb); //  #ff8bbb
pub const PURPLE: Color = color!(0xa955e8); // #a955e8

pub const PALETTE: Palette = Palette {
    background: DARK,
    text: LIGHT,
    primary: PINK,
    success: PURPLE,
    warning: PINK,
    danger: PURPLE,
};

pub const LINEAR_BACKGROUND: Background = Background::Gradient(Gradient::Linear(Linear {
    angle: Radians::PI,
    stops: [
        Some(ColorStop {
            offset: 0.0,
            color: LIGHT,
        }),
        Some(ColorStop {
            offset: 0.8,
            color: LIGHT,
        }),
        Some(ColorStop {
            offset: 0.81,
            color: PINK,
        }),
        Some(ColorStop {
            offset: 0.87,
            color: PINK,
        }),
        Some(ColorStop {
            offset: 0.88,
            color: PURPLE,
        }),
        None,
        None,
        None,
    ],
}));

pub const LINEAR_BACKGROUND_FOCUS: Background = Background::Gradient(Gradient::Linear(Linear {
    angle: Radians::PI,
    stops: [
        Some(ColorStop {
            offset: 0.00,
            color: LIGHTLESS,
        }),
        Some(ColorStop {
            offset: 0.8,
            color: LIGHTLESS,
        }),
        Some(ColorStop {
            offset: 0.81,
            color: PINK,
        }),
        Some(ColorStop {
            offset: 0.87,
            color: PINK,
        }),
        Some(ColorStop {
            offset: 0.88,
            color: PURPLE,
        }),
        None,
        None,
        None,
    ],
}));
