mod app_theme;
mod color;
mod radius;
mod spacing;

use iced::{Color, color};

use crate::theme::app_theme::Tokens;
pub use crate::theme::{
    app_theme::AppTheme,
    color::{ColorPalette, ColorScale, Shade},
    radius::RadiusScale,
    spacing::SpacingScale,
};

pub const ROSEWATER: Color = color!(0xf2d5cf); // #f2d5cf
pub const FLAMINGO: Color = color!(0xeebebe); // #eebebe
pub const PINK: Color = color!(0xf4b8e4); // #f4b8e4
pub const MAUVE: Color = color!(0xca9ee6); // #ca9ee6
pub const RED: Color = color!(0xe78284); // #e78284
pub const MAROON: Color = color!(0xea999c); // #ea999c
pub const PEACH: Color = color!(0xef9f76); // #ef9f76
pub const YELLOW: Color = color!(0xe5c890); // #e5c890
pub const GREEN: Color = color!(0xa6d189); // #a6d189
pub const TEAL: Color = color!(0x81c8be); // #81c8be
pub const SKY: Color = color!(0x99d1db); // #99d1db
pub const SAPPHIRE: Color = color!(0x85c1dc); // #85c1dc
pub const BLUE: Color = color!(0x8caaee); // #8caaee
pub const LAVENDER: Color = color!(0xbabbf1); // #babbf1
pub const TEXT: Color = color!(0xc6d0f5); // #c6d0f5
pub const SUBTEXT1: Color = color!(0xb5bfe2); // #b5bfe2
pub const SUBTEXT0: Color = color!(0xa5adce); // #a5adce
pub const OVERLAY2: Color = color!(0x949cbb); // #949cbb
pub const OVERLAY1: Color = color!(0x838ba7); // #838ba7
pub const OVERLAY0: Color = color!(0x737994); // #737994
pub const SURFACE2: Color = color!(0x626880); // #626880
pub const SURFACE1: Color = color!(0x51576d); // #51576d
pub const SURFACE0: Color = color!(0x414559); // #414559
pub const BASE: Color = color!(0x303446); // #303446
pub const MANTLE: Color = color!(0x292c3c); // #292c3c
pub const CRUST: Color = color!(0x232634); // #232634

pub fn app_theme() -> AppTheme {
    AppTheme::new(catppuccin_tokens())
}

fn catppuccin_tokens() -> Tokens {
    Tokens::new(
        SpacingScale::DEFAULT,
        catppuccin_colors(),
        RadiusScale::DEFAULT,
    )
}

pub fn catppuccin_colors() -> ColorPalette {
    ColorPalette {
        primary: ColorScale::new(
            color!(0xe6e8f8),
            color!(0xdee0f7),
            color!(0xd5d8f5),
            color!(0xcccef3),
            color!(0xc3c5f2),
            LAVENDER,
            color!(0xa1a4eb),
            color!(0x898ee5),
            color!(0x7178de),
            color!(0x5962d8),
        ),
        secondary: ColorScale::new(
            color!(0xefe2f7),
            color!(0xe6d2f3),
            color!(0xddc2ef),
            color!(0xd4b1eb),
            MAUVE,
            color!(0xbd89e0),
            color!(0xae71d9),
            color!(0xa05ad2),
            color!(0x9242cb),
            color!(0x8334ba),
        ),

        neutral: ColorScale::new(
            TEXT, SUBTEXT1, SUBTEXT0, OVERLAY2, OVERLAY1, OVERLAY0, SURFACE2, SURFACE1, SURFACE0,
            BASE,
        ),
        success: ColorScale::new(
            color!(0xcae6e1),
            color!(0xbce0da),
            color!(0xaedad3),
            color!(0x9fd4cc),
            color!(0x91cec5),
            color!(0x81c8be),
            color!(0x6dbfb2),
            color!(0x58b5a6),
            color!(0x4aa596),
            color!(0x409083),
        ),
        warning: ColorScale::new(
            color!(0xf9e0d6),
            color!(0xf7d3c4),
            color!(0xf5c7b2),
            color!(0xf3bb9f),
            color!(0xf1ae8c),
            color!(0xef9f76),
            color!(0xec8e5f),
            color!(0xe77b44),
            color!(0xe3682a),
            color!(0xd2591c),
        ),
        destructive: ColorScale::new(
            color!(0xfaeeef),
            color!(0xf7dee0),
            color!(0xf3cdd0),
            color!(0xf0bbbf),
            color!(0xedaaae),
            color!(0xea999c),
            color!(0xe48085),
            color!(0xdd686e),
            color!(0xd75059),
            color!(0xd03842),
        ),
        info: ColorScale::new(
            color!(0xd7e3f8),
            color!(0xc5d5f5),
            color!(0xb3c7f3),
            color!(0xa1baf1),
            color!(0x8caaee),
            color!(0x759aea),
            color!(0x5b87e5),
            color!(0x4277e0),
            color!(0x2967da),
            color!(0x225dc3),
        ),
    }
}
