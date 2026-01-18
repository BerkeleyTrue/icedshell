use iced::{Color, color, theme::Palette};

pub const ROSEWATER: Color = color!(0xf2d5cf);// #f2d5cf
pub const FLAMINGO: Color = color!(0xeebebe); // #eebebe
pub const PINK: Color = color!(0xf4b8e4);     // #f4b8e4
pub const MAUVE: Color = color!(0xca9ee6);    // #ca9ee6
pub const RED: Color = color!(0xe78284);      // #e78284
pub const MAROON: Color = color!(0xea999c);   // #ea999c
pub const PEACH: Color = color!(0xef9f76);    // #ef9f76
pub const YELLOW: Color = color!(0xe5c890);   // #e5c890
pub const GREEN: Color = color!(0xa6d189);    // #a6d189
pub const TEAL: Color = color!(0x81c8be);     // #81c8be
pub const SKY: Color = color!(0x99d1db);      // #99d1db
pub const SAPPHIRE: Color = color!(0x85c1dc); // #85c1dc
pub const BLUE: Color = color!(0x8caaee);     // #8caaee
pub const LAVENDER: Color = color!(0xbabbf1); // #babbf1
pub const TEXT: Color = color!(0xc6d0f5);     // #c6d0f5
pub const SUBTEXT1: Color = color!(0xb5bfe2); // #b5bfe2
pub const SUBTEXT0: Color = color!(0xa5adce); // #a5adce
pub const OVERLAY2: Color = color!(0x949cbb); // #949cbb
pub const OVERLAY1: Color = color!(0x838ba7); // #838ba7
pub const OVERLAY0: Color = color!(0x737994); // #737994
pub const SURFACE2: Color = color!(0x626880); // #626880
pub const SURFACE1: Color = color!(0x51576d); // #51576d
pub const SURFACE0: Color = color!(0x414559); // #414559
pub const BASE: Color = color!(0x303446);     // #303446
pub const MANTLE: Color = color!(0x292c3c);   // #292c3c
pub const CRUST: Color = color!(0x232634);    // #232634

pub const PALETTE: Palette = Palette {
    background: BASE,
    text: TEXT,
    primary: ROSEWATER,
    success: TEAL,
    warning: YELLOW,
    danger: MAROON,
};

pub const REM: u32 = 14;
