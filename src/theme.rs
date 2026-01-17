use iced::{Color, color, theme::Palette};

const ROSEWATER: Color = color!(0xf2d5cf);// #f2d5cf
const FLAMINGO: Color = color!(0xeebebe); // #eebebe
const PINK: Color = color!(0xf4b8e4);     // #f4b8e4
const MAUVE: Color = color!(0xca9ee6);    // #ca9ee6
const RED: Color = color!(0xe78284);      // #e78284
const MAROON: Color = color!(0xea999c);   // #ea999c
const PEACH: Color = color!(0xef9f76);    // #ef9f76
const YELLOW: Color = color!(0xe5c890);   // #e5c890
const GREEN: Color = color!(0xa6d189);    // #a6d189
const TEAL: Color = color!(0x81c8be);     // #81c8be
const SKY: Color = color!(0x99d1db);      // #99d1db
const SAPPHIRE: Color = color!(0x85c1dc); // #85c1dc
const BLUE: Color = color!(0x8caaee);     // #8caaee
const LAVENDER: Color = color!(0xbabbf1); // #babbf1
const TEXT: Color = color!(0xc6d0f5);     // #c6d0f5
const SUBTEXT1: Color = color!(0xb5bfe2); // #b5bfe2
const SUBTEXT0: Color = color!(0xa5adce); // #a5adce
const OVERLAY2: Color = color!(0x949cbb); // #949cbb
const OVERLAY1: Color = color!(0x838ba7); // #838ba7
const OVERLAY0: Color = color!(0x737994); // #737994
const SURFACE2: Color = color!(0x626880); // #626880
const SURFACE1: Color = color!(0x51576d); // #51576d
const SURFACE0: Color = color!(0x414559); // #414559
const BASE: Color = color!(0x303446);     // #303446
const MANTLE: Color = color!(0x292c3c);   // #292c3c
const CRUST: Color = color!(0x232634);    // #232634

pub const PALETTE: Palette = Palette {
    background: BASE,
    text: TEXT,
    primary: ROSEWATER,
    success: TEAL,
    warning: YELLOW,
    danger: MAROON,
};
