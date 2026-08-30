use crate::compat::strtonum;
use crate::ffi::sscanf;
use crate::fmt_args;
use crate::log::log_debug;
use crate::types::{client_theme, u_char, u_int};
/// A pane's colour palette: the two default colours and, when they have been
/// set, the 256 entries of the palette itself and of the defaults the
/// `pane-colours` option gave.
#[derive(Clone, Default)]
#[repr(C)]
pub struct colour_palette {
    pub fg: ::core::ffi::c_int,
    pub bg: ::core::ffi::c_int,
    pub palette: Option<Box<[::core::ffi::c_int; 256]>>,
    pub default_palette: Option<Box<[::core::ffi::c_int; 256]>>,
}

pub const THEME_DARK: client_theme = 2;
pub const THEME_LIGHT: client_theme = 1;
pub const THEME_UNKNOWN: client_theme = 0;
pub const COLOUR_FLAG_256: ::core::ffi::c_int = 0x1000000;
pub const COLOUR_FLAG_RGB: ::core::ffi::c_int = 0x2000000;

/// The eight-bit value of each step along one axis of the 6x6x6 colour cube.
const CUBE_STEPS: [::core::ffi::c_int; 6] = [0, 0x5f, 0x87, 0xaf, 0xd7, 0xff];

/// The RGB value of each of the 256 indexed colours.
const RGB_OF_256: [::core::ffi::c_int; 256] = [
    0, 0x800000, 0x8000, 0x808000, 0x80, 0x800080, 0x8080, 0xc0c0c0, 0x808080, 0xff0000, 0xff00,
    0xffff00, 0xff, 0xff00ff, 0xffff, 0xffffff, 0, 0x5f, 0x87, 0xaf, 0xd7, 0xff, 0x5f00, 0x5f5f,
    0x5f87, 0x5faf, 0x5fd7, 0x5fff, 0x8700, 0x875f, 0x8787, 0x87af, 0x87d7, 0x87ff, 0xaf00, 0xaf5f,
    0xaf87, 0xafaf, 0xafd7, 0xafff, 0xd700, 0xd75f, 0xd787, 0xd7af, 0xd7d7, 0xd7ff, 0xff00, 0xff5f,
    0xff87, 0xffaf, 0xffd7, 0xffff, 0x5f0000, 0x5f005f, 0x5f0087, 0x5f00af, 0x5f00d7, 0x5f00ff,
    0x5f5f00, 0x5f5f5f, 0x5f5f87, 0x5f5faf, 0x5f5fd7, 0x5f5fff, 0x5f8700, 0x5f875f, 0x5f8787,
    0x5f87af, 0x5f87d7, 0x5f87ff, 0x5faf00, 0x5faf5f, 0x5faf87, 0x5fafaf, 0x5fafd7, 0x5fafff,
    0x5fd700, 0x5fd75f, 0x5fd787, 0x5fd7af, 0x5fd7d7, 0x5fd7ff, 0x5fff00, 0x5fff5f, 0x5fff87,
    0x5fffaf, 0x5fffd7, 0x5fffff, 0x870000, 0x87005f, 0x870087, 0x8700af, 0x8700d7, 0x8700ff,
    0x875f00, 0x875f5f, 0x875f87, 0x875faf, 0x875fd7, 0x875fff, 0x878700, 0x87875f, 0x878787,
    0x8787af, 0x8787d7, 0x8787ff, 0x87af00, 0x87af5f, 0x87af87, 0x87afaf, 0x87afd7, 0x87afff,
    0x87d700, 0x87d75f, 0x87d787, 0x87d7af, 0x87d7d7, 0x87d7ff, 0x87ff00, 0x87ff5f, 0x87ff87,
    0x87ffaf, 0x87ffd7, 0x87ffff, 0xaf0000, 0xaf005f, 0xaf0087, 0xaf00af, 0xaf00d7, 0xaf00ff,
    0xaf5f00, 0xaf5f5f, 0xaf5f87, 0xaf5faf, 0xaf5fd7, 0xaf5fff, 0xaf8700, 0xaf875f, 0xaf8787,
    0xaf87af, 0xaf87d7, 0xaf87ff, 0xafaf00, 0xafaf5f, 0xafaf87, 0xafafaf, 0xafafd7, 0xafafff,
    0xafd700, 0xafd75f, 0xafd787, 0xafd7af, 0xafd7d7, 0xafd7ff, 0xafff00, 0xafff5f, 0xafff87,
    0xafffaf, 0xafffd7, 0xafffff, 0xd70000, 0xd7005f, 0xd70087, 0xd700af, 0xd700d7, 0xd700ff,
    0xd75f00, 0xd75f5f, 0xd75f87, 0xd75faf, 0xd75fd7, 0xd75fff, 0xd78700, 0xd7875f, 0xd78787,
    0xd787af, 0xd787d7, 0xd787ff, 0xd7af00, 0xd7af5f, 0xd7af87, 0xd7afaf, 0xd7afd7, 0xd7afff,
    0xd7d700, 0xd7d75f, 0xd7d787, 0xd7d7af, 0xd7d7d7, 0xd7d7ff, 0xd7ff00, 0xd7ff5f, 0xd7ff87,
    0xd7ffaf, 0xd7ffd7, 0xd7ffff, 0xff0000, 0xff005f, 0xff0087, 0xff00af, 0xff00d7, 0xff00ff,
    0xff5f00, 0xff5f5f, 0xff5f87, 0xff5faf, 0xff5fd7, 0xff5fff, 0xff8700, 0xff875f, 0xff8787,
    0xff87af, 0xff87d7, 0xff87ff, 0xffaf00, 0xffaf5f, 0xffaf87, 0xffafaf, 0xffafd7, 0xffafff,
    0xffd700, 0xffd75f, 0xffd787, 0xffd7af, 0xffd7d7, 0xffd7ff, 0xffff00, 0xffff5f, 0xffff87,
    0xffffaf, 0xffffd7, 0xffffff, 0x80808, 0x121212, 0x1c1c1c, 0x262626, 0x303030, 0x3a3a3a,
    0x444444, 0x4e4e4e, 0x585858, 0x626262, 0x6c6c6c, 0x767676, 0x808080, 0x8a8a8a, 0x949494,
    0x9e9e9e, 0xa8a8a8, 0xb2b2b2, 0xbcbcbc, 0xc6c6c6, 0xd0d0d0, 0xdadada, 0xe4e4e4, 0xeeeeee,
];

/// The nearest of the sixteen basic colours to each of the 256 indexed ones.
const BASIC_OF_256: [::core::ffi::c_int; 256] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 0, 4, 4, 4, 12, 12, 2, 6, 4, 4, 12, 12,
    2, 2, 6, 4, 12, 12, 2, 2, 2, 6, 12, 12, 10, 10, 10, 10, 14, 12, 10, 10, 10, 10, 10, 14, 1, 5,
    4, 4, 12, 12, 3, 8, 4, 4, 12, 12, 2, 2, 6, 4, 12, 12, 2, 2, 2, 6, 12, 12, 10, 10, 10, 10, 14,
    12, 10, 10, 10, 10, 10, 14, 1, 1, 5, 4, 12, 12, 1, 1, 5, 4, 12, 12, 3, 3, 8, 4, 12, 12, 2, 2,
    2, 6, 12, 12, 10, 10, 10, 10, 14, 12, 10, 10, 10, 10, 10, 14, 1, 1, 1, 5, 12, 12, 1, 1, 1, 5,
    12, 12, 1, 1, 1, 5, 12, 12, 3, 3, 3, 7, 12, 12, 10, 10, 10, 10, 14, 12, 10, 10, 10, 10, 10, 14,
    9, 9, 9, 9, 13, 12, 9, 9, 9, 9, 13, 12, 9, 9, 9, 9, 13, 12, 9, 9, 9, 9, 13, 12, 11, 11, 11, 11,
    7, 12, 10, 10, 10, 10, 10, 14, 9, 9, 9, 9, 9, 13, 9, 9, 9, 9, 9, 13, 9, 9, 9, 9, 9, 13, 9, 9,
    9, 9, 9, 13, 9, 9, 9, 9, 9, 13, 11, 11, 11, 11, 11, 15, 0, 0, 0, 0, 0, 0, 8, 8, 8, 8, 8, 8, 7,
    7, 7, 7, 7, 7, 15, 15, 15, 15, 15, 15,
];

/// The colours `colour_byname` accepts, in the order it searches them.
const X11_COLOURS: [(&str, ::core::ffi::c_int); 578] = [
    ("AliceBlue", 0xf0f8ff),
    ("AntiqueWhite", 0xfaebd7),
    ("AntiqueWhite1", 0xffefdb),
    ("AntiqueWhite2", 0xeedfcc),
    ("AntiqueWhite3", 0xcdc0b0),
    ("AntiqueWhite4", 0x8b8378),
    ("BlanchedAlmond", 0xffebcd),
    ("BlueViolet", 0x8a2be2),
    ("CadetBlue", 0x5f9ea0),
    ("CadetBlue1", 0x98f5ff),
    ("CadetBlue2", 0x8ee5ee),
    ("CadetBlue3", 0x7ac5cd),
    ("CadetBlue4", 0x53868b),
    ("CornflowerBlue", 0x6495ed),
    ("DarkBlue", 0x8b),
    ("DarkCyan", 0x8b8b),
    ("DarkGoldenrod", 0xb8860b),
    ("DarkGoldenrod1", 0xffb90f),
    ("DarkGoldenrod2", 0xeead0e),
    ("DarkGoldenrod3", 0xcd950c),
    ("DarkGoldenrod4", 0x8b6508),
    ("DarkGray", 0xa9a9a9),
    ("DarkGreen", 0x6400),
    ("DarkGrey", 0xa9a9a9),
    ("DarkKhaki", 0xbdb76b),
    ("DarkMagenta", 0x8b008b),
    ("DarkOliveGreen", 0x556b2f),
    ("DarkOliveGreen1", 0xcaff70),
    ("DarkOliveGreen2", 0xbcee68),
    ("DarkOliveGreen3", 0xa2cd5a),
    ("DarkOliveGreen4", 0x6e8b3d),
    ("DarkOrange", 0xff8c00),
    ("DarkOrange1", 0xff7f00),
    ("DarkOrange2", 0xee7600),
    ("DarkOrange3", 0xcd6600),
    ("DarkOrange4", 0x8b4500),
    ("DarkOrchid", 0x9932cc),
    ("DarkOrchid1", 0xbf3eff),
    ("DarkOrchid2", 0xb23aee),
    ("DarkOrchid3", 0x9a32cd),
    ("DarkOrchid4", 0x68228b),
    ("DarkRed", 0x8b0000),
    ("DarkSalmon", 0xe9967a),
    ("DarkSeaGreen", 0x8fbc8f),
    ("DarkSeaGreen1", 0xc1ffc1),
    ("DarkSeaGreen2", 0xb4eeb4),
    ("DarkSeaGreen3", 0x9bcd9b),
    ("DarkSeaGreen4", 0x698b69),
    ("DarkSlateBlue", 0x483d8b),
    ("DarkSlateGray", 0x2f4f4f),
    ("DarkSlateGray1", 0x97ffff),
    ("DarkSlateGray2", 0x8deeee),
    ("DarkSlateGray3", 0x79cdcd),
    ("DarkSlateGray4", 0x528b8b),
    ("DarkSlateGrey", 0x2f4f4f),
    ("DarkTurquoise", 0xced1),
    ("DarkViolet", 0x9400d3),
    ("DeepPink", 0xff1493),
    ("DeepPink1", 0xff1493),
    ("DeepPink2", 0xee1289),
    ("DeepPink3", 0xcd1076),
    ("DeepPink4", 0x8b0a50),
    ("DeepSkyBlue", 0xbfff),
    ("DeepSkyBlue1", 0xbfff),
    ("DeepSkyBlue2", 0xb2ee),
    ("DeepSkyBlue3", 0x9acd),
    ("DeepSkyBlue4", 0x688b),
    ("DimGray", 0x696969),
    ("DimGrey", 0x696969),
    ("DodgerBlue", 0x1e90ff),
    ("DodgerBlue1", 0x1e90ff),
    ("DodgerBlue2", 0x1c86ee),
    ("DodgerBlue3", 0x1874cd),
    ("DodgerBlue4", 0x104e8b),
    ("FloralWhite", 0xfffaf0),
    ("ForestGreen", 0x228b22),
    ("GhostWhite", 0xf8f8ff),
    ("GreenYellow", 0xadff2f),
    ("HotPink", 0xff69b4),
    ("HotPink1", 0xff6eb4),
    ("HotPink2", 0xee6aa7),
    ("HotPink3", 0xcd6090),
    ("HotPink4", 0x8b3a62),
    ("IndianRed", 0xcd5c5c),
    ("IndianRed1", 0xff6a6a),
    ("IndianRed2", 0xee6363),
    ("IndianRed3", 0xcd5555),
    ("IndianRed4", 0x8b3a3a),
    ("LavenderBlush", 0xfff0f5),
    ("LavenderBlush1", 0xfff0f5),
    ("LavenderBlush2", 0xeee0e5),
    ("LavenderBlush3", 0xcdc1c5),
    ("LavenderBlush4", 0x8b8386),
    ("LawnGreen", 0x7cfc00),
    ("LemonChiffon", 0xfffacd),
    ("LemonChiffon1", 0xfffacd),
    ("LemonChiffon2", 0xeee9bf),
    ("LemonChiffon3", 0xcdc9a5),
    ("LemonChiffon4", 0x8b8970),
    ("LightBlue", 0xadd8e6),
    ("LightBlue1", 0xbfefff),
    ("LightBlue2", 0xb2dfee),
    ("LightBlue3", 0x9ac0cd),
    ("LightBlue4", 0x68838b),
    ("LightCoral", 0xf08080),
    ("LightCyan", 0xe0ffff),
    ("LightCyan1", 0xe0ffff),
    ("LightCyan2", 0xd1eeee),
    ("LightCyan3", 0xb4cdcd),
    ("LightCyan4", 0x7a8b8b),
    ("LightGoldenrod", 0xeedd82),
    ("LightGoldenrod1", 0xffec8b),
    ("LightGoldenrod2", 0xeedc82),
    ("LightGoldenrod3", 0xcdbe70),
    ("LightGoldenrod4", 0x8b814c),
    ("LightGoldenrodYellow", 0xfafad2),
    ("LightGray", 0xd3d3d3),
    ("LightGreen", 0x90ee90),
    ("LightGrey", 0xd3d3d3),
    ("LightPink", 0xffb6c1),
    ("LightPink1", 0xffaeb9),
    ("LightPink2", 0xeea2ad),
    ("LightPink3", 0xcd8c95),
    ("LightPink4", 0x8b5f65),
    ("LightSalmon", 0xffa07a),
    ("LightSalmon1", 0xffa07a),
    ("LightSalmon2", 0xee9572),
    ("LightSalmon3", 0xcd8162),
    ("LightSalmon4", 0x8b5742),
    ("LightSeaGreen", 0x20b2aa),
    ("LightSkyBlue", 0x87cefa),
    ("LightSkyBlue1", 0xb0e2ff),
    ("LightSkyBlue2", 0xa4d3ee),
    ("LightSkyBlue3", 0x8db6cd),
    ("LightSkyBlue4", 0x607b8b),
    ("LightSlateBlue", 0x8470ff),
    ("LightSlateGray", 0x778899),
    ("LightSlateGrey", 0x778899),
    ("LightSteelBlue", 0xb0c4de),
    ("LightSteelBlue1", 0xcae1ff),
    ("LightSteelBlue2", 0xbcd2ee),
    ("LightSteelBlue3", 0xa2b5cd),
    ("LightSteelBlue4", 0x6e7b8b),
    ("LightYellow", 0xffffe0),
    ("LightYellow1", 0xffffe0),
    ("LightYellow2", 0xeeeed1),
    ("LightYellow3", 0xcdcdb4),
    ("LightYellow4", 0x8b8b7a),
    ("LimeGreen", 0x32cd32),
    ("MediumAquamarine", 0x66cdaa),
    ("MediumBlue", 0xcd),
    ("MediumOrchid", 0xba55d3),
    ("MediumOrchid1", 0xe066ff),
    ("MediumOrchid2", 0xd15fee),
    ("MediumOrchid3", 0xb452cd),
    ("MediumOrchid4", 0x7a378b),
    ("MediumPurple", 0x9370db),
    ("MediumPurple1", 0xab82ff),
    ("MediumPurple2", 0x9f79ee),
    ("MediumPurple3", 0x8968cd),
    ("MediumPurple4", 0x5d478b),
    ("MediumSeaGreen", 0x3cb371),
    ("MediumSlateBlue", 0x7b68ee),
    ("MediumSpringGreen", 0xfa9a),
    ("MediumTurquoise", 0x48d1cc),
    ("MediumVioletRed", 0xc71585),
    ("MidnightBlue", 0x191970),
    ("MintCream", 0xf5fffa),
    ("MistyRose", 0xffe4e1),
    ("MistyRose1", 0xffe4e1),
    ("MistyRose2", 0xeed5d2),
    ("MistyRose3", 0xcdb7b5),
    ("MistyRose4", 0x8b7d7b),
    ("NavajoWhite", 0xffdead),
    ("NavajoWhite1", 0xffdead),
    ("NavajoWhite2", 0xeecfa1),
    ("NavajoWhite3", 0xcdb38b),
    ("NavajoWhite4", 0x8b795e),
    ("NavyBlue", 0x80),
    ("OldLace", 0xfdf5e6),
    ("OliveDrab", 0x6b8e23),
    ("OliveDrab1", 0xc0ff3e),
    ("OliveDrab2", 0xb3ee3a),
    ("OliveDrab3", 0x9acd32),
    ("OliveDrab4", 0x698b22),
    ("OrangeRed", 0xff4500),
    ("OrangeRed1", 0xff4500),
    ("OrangeRed2", 0xee4000),
    ("OrangeRed3", 0xcd3700),
    ("OrangeRed4", 0x8b2500),
    ("PaleGoldenrod", 0xeee8aa),
    ("PaleGreen", 0x98fb98),
    ("PaleGreen1", 0x9aff9a),
    ("PaleGreen2", 0x90ee90),
    ("PaleGreen3", 0x7ccd7c),
    ("PaleGreen4", 0x548b54),
    ("PaleTurquoise", 0xafeeee),
    ("PaleTurquoise1", 0xbbffff),
    ("PaleTurquoise2", 0xaeeeee),
    ("PaleTurquoise3", 0x96cdcd),
    ("PaleTurquoise4", 0x668b8b),
    ("PaleVioletRed", 0xdb7093),
    ("PaleVioletRed1", 0xff82ab),
    ("PaleVioletRed2", 0xee799f),
    ("PaleVioletRed3", 0xcd6889),
    ("PaleVioletRed4", 0x8b475d),
    ("PapayaWhip", 0xffefd5),
    ("PeachPuff", 0xffdab9),
    ("PeachPuff1", 0xffdab9),
    ("PeachPuff2", 0xeecbad),
    ("PeachPuff3", 0xcdaf95),
    ("PeachPuff4", 0x8b7765),
    ("PowderBlue", 0xb0e0e6),
    ("RebeccaPurple", 0x663399),
    ("RosyBrown", 0xbc8f8f),
    ("RosyBrown1", 0xffc1c1),
    ("RosyBrown2", 0xeeb4b4),
    ("RosyBrown3", 0xcd9b9b),
    ("RosyBrown4", 0x8b6969),
    ("RoyalBlue", 0x4169e1),
    ("RoyalBlue1", 0x4876ff),
    ("RoyalBlue2", 0x436eee),
    ("RoyalBlue3", 0x3a5fcd),
    ("RoyalBlue4", 0x27408b),
    ("SaddleBrown", 0x8b4513),
    ("SandyBrown", 0xf4a460),
    ("SeaGreen", 0x2e8b57),
    ("SeaGreen1", 0x54ff9f),
    ("SeaGreen2", 0x4eee94),
    ("SeaGreen3", 0x43cd80),
    ("SeaGreen4", 0x2e8b57),
    ("SkyBlue", 0x87ceeb),
    ("SkyBlue1", 0x87ceff),
    ("SkyBlue2", 0x7ec0ee),
    ("SkyBlue3", 0x6ca6cd),
    ("SkyBlue4", 0x4a708b),
    ("SlateBlue", 0x6a5acd),
    ("SlateBlue1", 0x836fff),
    ("SlateBlue2", 0x7a67ee),
    ("SlateBlue3", 0x6959cd),
    ("SlateBlue4", 0x473c8b),
    ("SlateGray", 0x708090),
    ("SlateGray1", 0xc6e2ff),
    ("SlateGray2", 0xb9d3ee),
    ("SlateGray3", 0x9fb6cd),
    ("SlateGray4", 0x6c7b8b),
    ("SlateGrey", 0x708090),
    ("SpringGreen", 0xff7f),
    ("SpringGreen1", 0xff7f),
    ("SpringGreen2", 0xee76),
    ("SpringGreen3", 0xcd66),
    ("SpringGreen4", 0x8b45),
    ("SteelBlue", 0x4682b4),
    ("SteelBlue1", 0x63b8ff),
    ("SteelBlue2", 0x5cacee),
    ("SteelBlue3", 0x4f94cd),
    ("SteelBlue4", 0x36648b),
    ("VioletRed", 0xd02090),
    ("VioletRed1", 0xff3e96),
    ("VioletRed2", 0xee3a8c),
    ("VioletRed3", 0xcd3278),
    ("VioletRed4", 0x8b2252),
    ("WebGray", 0x808080),
    ("WebGreen", 0x8000),
    ("WebGrey", 0x808080),
    ("WebMaroon", 0x800000),
    ("WebPurple", 0x800080),
    ("WhiteSmoke", 0xf5f5f5),
    ("X11Gray", 0xbebebe),
    ("X11Green", 0xff00),
    ("X11Grey", 0xbebebe),
    ("X11Maroon", 0xb03060),
    ("X11Purple", 0xa020f0),
    ("YellowGreen", 0x9acd32),
    ("alice blue", 0xf0f8ff),
    ("antique white", 0xfaebd7),
    ("aqua", 0xffff),
    ("aquamarine", 0x7fffd4),
    ("aquamarine1", 0x7fffd4),
    ("aquamarine2", 0x76eec6),
    ("aquamarine3", 0x66cdaa),
    ("aquamarine4", 0x458b74),
    ("azure", 0xf0ffff),
    ("azure1", 0xf0ffff),
    ("azure2", 0xe0eeee),
    ("azure3", 0xc1cdcd),
    ("azure4", 0x838b8b),
    ("beige", 0xf5f5dc),
    ("bisque", 0xffe4c4),
    ("bisque1", 0xffe4c4),
    ("bisque2", 0xeed5b7),
    ("bisque3", 0xcdb79e),
    ("bisque4", 0x8b7d6b),
    ("black", 0),
    ("blanched almond", 0xffebcd),
    ("blue violet", 0x8a2be2),
    ("blue", 0xff),
    ("blue1", 0xff),
    ("blue2", 0xee),
    ("blue3", 0xcd),
    ("blue4", 0x8b),
    ("brown", 0xa52a2a),
    ("brown1", 0xff4040),
    ("brown2", 0xee3b3b),
    ("brown3", 0xcd3333),
    ("brown4", 0x8b2323),
    ("burlywood", 0xdeb887),
    ("burlywood1", 0xffd39b),
    ("burlywood2", 0xeec591),
    ("burlywood3", 0xcdaa7d),
    ("burlywood4", 0x8b7355),
    ("cadet blue", 0x5f9ea0),
    ("chartreuse", 0x7fff00),
    ("chartreuse1", 0x7fff00),
    ("chartreuse2", 0x76ee00),
    ("chartreuse3", 0x66cd00),
    ("chartreuse4", 0x458b00),
    ("chocolate", 0xd2691e),
    ("chocolate1", 0xff7f24),
    ("chocolate2", 0xee7621),
    ("chocolate3", 0xcd661d),
    ("chocolate4", 0x8b4513),
    ("coral", 0xff7f50),
    ("coral1", 0xff7256),
    ("coral2", 0xee6a50),
    ("coral3", 0xcd5b45),
    ("coral4", 0x8b3e2f),
    ("cornflower blue", 0x6495ed),
    ("cornsilk", 0xfff8dc),
    ("cornsilk1", 0xfff8dc),
    ("cornsilk2", 0xeee8cd),
    ("cornsilk3", 0xcdc8b1),
    ("cornsilk4", 0x8b8878),
    ("crimson", 0xdc143c),
    ("cyan", 0xffff),
    ("cyan1", 0xffff),
    ("cyan2", 0xeeee),
    ("cyan3", 0xcdcd),
    ("cyan4", 0x8b8b),
    ("dark blue", 0x8b),
    ("dark cyan", 0x8b8b),
    ("dark goldenrod", 0xb8860b),
    ("dark gray", 0xa9a9a9),
    ("dark green", 0x6400),
    ("dark grey", 0xa9a9a9),
    ("dark khaki", 0xbdb76b),
    ("dark magenta", 0x8b008b),
    ("dark olive green", 0x556b2f),
    ("dark orange", 0xff8c00),
    ("dark orchid", 0x9932cc),
    ("dark red", 0x8b0000),
    ("dark salmon", 0xe9967a),
    ("dark sea green", 0x8fbc8f),
    ("dark slate blue", 0x483d8b),
    ("dark slate gray", 0x2f4f4f),
    ("dark slate grey", 0x2f4f4f),
    ("dark turquoise", 0xced1),
    ("dark violet", 0x9400d3),
    ("deep pink", 0xff1493),
    ("deep sky blue", 0xbfff),
    ("dim gray", 0x696969),
    ("dim grey", 0x696969),
    ("dodger blue", 0x1e90ff),
    ("firebrick", 0xb22222),
    ("firebrick1", 0xff3030),
    ("firebrick2", 0xee2c2c),
    ("firebrick3", 0xcd2626),
    ("firebrick4", 0x8b1a1a),
    ("floral white", 0xfffaf0),
    ("forest green", 0x228b22),
    ("fuchsia", 0xff00ff),
    ("gainsboro", 0xdcdcdc),
    ("ghost white", 0xf8f8ff),
    ("gold", 0xffd700),
    ("gold1", 0xffd700),
    ("gold2", 0xeec900),
    ("gold3", 0xcdad00),
    ("gold4", 0x8b7500),
    ("goldenrod", 0xdaa520),
    ("goldenrod1", 0xffc125),
    ("goldenrod2", 0xeeb422),
    ("goldenrod3", 0xcd9b1d),
    ("goldenrod4", 0x8b6914),
    ("green yellow", 0xadff2f),
    ("green", 0xff00),
    ("green1", 0xff00),
    ("green2", 0xee00),
    ("green3", 0xcd00),
    ("green4", 0x8b00),
    ("honeydew", 0xf0fff0),
    ("honeydew1", 0xf0fff0),
    ("honeydew2", 0xe0eee0),
    ("honeydew3", 0xc1cdc1),
    ("honeydew4", 0x838b83),
    ("hot pink", 0xff69b4),
    ("indian red", 0xcd5c5c),
    ("indigo", 0x4b0082),
    ("ivory", 0xfffff0),
    ("ivory1", 0xfffff0),
    ("ivory2", 0xeeeee0),
    ("ivory3", 0xcdcdc1),
    ("ivory4", 0x8b8b83),
    ("khaki", 0xf0e68c),
    ("khaki1", 0xfff68f),
    ("khaki2", 0xeee685),
    ("khaki3", 0xcdc673),
    ("khaki4", 0x8b864e),
    ("lavender blush", 0xfff0f5),
    ("lavender", 0xe6e6fa),
    ("lawn green", 0x7cfc00),
    ("lemon chiffon", 0xfffacd),
    ("light blue", 0xadd8e6),
    ("light coral", 0xf08080),
    ("light cyan", 0xe0ffff),
    ("light goldenrod yellow", 0xfafad2),
    ("light goldenrod", 0xeedd82),
    ("light gray", 0xd3d3d3),
    ("light green", 0x90ee90),
    ("light grey", 0xd3d3d3),
    ("light pink", 0xffb6c1),
    ("light salmon", 0xffa07a),
    ("light sea green", 0x20b2aa),
    ("light sky blue", 0x87cefa),
    ("light slate blue", 0x8470ff),
    ("light slate gray", 0x778899),
    ("light slate grey", 0x778899),
    ("light steel blue", 0xb0c4de),
    ("light yellow", 0xffffe0),
    ("lime green", 0x32cd32),
    ("lime", 0xff00),
    ("linen", 0xfaf0e6),
    ("magenta", 0xff00ff),
    ("magenta1", 0xff00ff),
    ("magenta2", 0xee00ee),
    ("magenta3", 0xcd00cd),
    ("magenta4", 0x8b008b),
    ("maroon", 0xb03060),
    ("maroon1", 0xff34b3),
    ("maroon2", 0xee30a7),
    ("maroon3", 0xcd2990),
    ("maroon4", 0x8b1c62),
    ("medium aquamarine", 0x66cdaa),
    ("medium blue", 0xcd),
    ("medium orchid", 0xba55d3),
    ("medium purple", 0x9370db),
    ("medium sea green", 0x3cb371),
    ("medium slate blue", 0x7b68ee),
    ("medium spring green", 0xfa9a),
    ("medium turquoise", 0x48d1cc),
    ("medium violet red", 0xc71585),
    ("midnight blue", 0x191970),
    ("mint cream", 0xf5fffa),
    ("misty rose", 0xffe4e1),
    ("moccasin", 0xffe4b5),
    ("navajo white", 0xffdead),
    ("navy blue", 0x80),
    ("navy", 0x80),
    ("old lace", 0xfdf5e6),
    ("olive drab", 0x6b8e23),
    ("olive", 0x808000),
    ("orange red", 0xff4500),
    ("orange", 0xffa500),
    ("orange1", 0xffa500),
    ("orange2", 0xee9a00),
    ("orange3", 0xcd8500),
    ("orange4", 0x8b5a00),
    ("orchid", 0xda70d6),
    ("orchid1", 0xff83fa),
    ("orchid2", 0xee7ae9),
    ("orchid3", 0xcd69c9),
    ("orchid4", 0x8b4789),
    ("pale goldenrod", 0xeee8aa),
    ("pale green", 0x98fb98),
    ("pale turquoise", 0xafeeee),
    ("pale violet red", 0xdb7093),
    ("papaya whip", 0xffefd5),
    ("peach puff", 0xffdab9),
    ("peru", 0xcd853f),
    ("pink", 0xffc0cb),
    ("pink1", 0xffb5c5),
    ("pink2", 0xeea9b8),
    ("pink3", 0xcd919e),
    ("pink4", 0x8b636c),
    ("plum", 0xdda0dd),
    ("plum1", 0xffbbff),
    ("plum2", 0xeeaeee),
    ("plum3", 0xcd96cd),
    ("plum4", 0x8b668b),
    ("powder blue", 0xb0e0e6),
    ("purple", 0xa020f0),
    ("purple1", 0x9b30ff),
    ("purple2", 0x912cee),
    ("purple3", 0x7d26cd),
    ("purple4", 0x551a8b),
    ("rebecca purple", 0x663399),
    ("red", 0xff0000),
    ("red1", 0xff0000),
    ("red2", 0xee0000),
    ("red3", 0xcd0000),
    ("red4", 0x8b0000),
    ("rosy brown", 0xbc8f8f),
    ("royal blue", 0x4169e1),
    ("saddle brown", 0x8b4513),
    ("salmon", 0xfa8072),
    ("salmon1", 0xff8c69),
    ("salmon2", 0xee8262),
    ("salmon3", 0xcd7054),
    ("salmon4", 0x8b4c39),
    ("sandy brown", 0xf4a460),
    ("sea green", 0x2e8b57),
    ("seashell", 0xfff5ee),
    ("seashell1", 0xfff5ee),
    ("seashell2", 0xeee5de),
    ("seashell3", 0xcdc5bf),
    ("seashell4", 0x8b8682),
    ("sienna", 0xa0522d),
    ("sienna1", 0xff8247),
    ("sienna2", 0xee7942),
    ("sienna3", 0xcd6839),
    ("sienna4", 0x8b4726),
    ("silver", 0xc0c0c0),
    ("sky blue", 0x87ceeb),
    ("slate blue", 0x6a5acd),
    ("slate gray", 0x708090),
    ("slate grey", 0x708090),
    ("snow", 0xfffafa),
    ("snow1", 0xfffafa),
    ("snow2", 0xeee9e9),
    ("snow3", 0xcdc9c9),
    ("snow4", 0x8b8989),
    ("spring green", 0xff7f),
    ("steel blue", 0x4682b4),
    ("tan", 0xd2b48c),
    ("tan1", 0xffa54f),
    ("tan2", 0xee9a49),
    ("tan3", 0xcd853f),
    ("tan4", 0x8b5a2b),
    ("teal", 0x8080),
    ("thistle", 0xd8bfd8),
    ("thistle1", 0xffe1ff),
    ("thistle2", 0xeed2ee),
    ("thistle3", 0xcdb5cd),
    ("thistle4", 0x8b7b8b),
    ("tomato", 0xff6347),
    ("tomato1", 0xff6347),
    ("tomato2", 0xee5c42),
    ("tomato3", 0xcd4f39),
    ("tomato4", 0x8b3626),
    ("turquoise", 0x40e0d0),
    ("turquoise1", 0xf5ff),
    ("turquoise2", 0xe5ee),
    ("turquoise3", 0xc5cd),
    ("turquoise4", 0x868b),
    ("violet red", 0xd02090),
    ("violet", 0xee82ee),
    ("web gray", 0x808080),
    ("web green", 0x8000),
    ("web grey", 0x808080),
    ("web maroon", 0x800000),
    ("web purple", 0x800080),
    ("wheat", 0xf5deb3),
    ("wheat1", 0xffe7ba),
    ("wheat2", 0xeed8ae),
    ("wheat3", 0xcdba96),
    ("wheat4", 0x8b7e66),
    ("white smoke", 0xf5f5f5),
    ("white", 0xffffff),
    ("x11 gray", 0xbebebe),
    ("x11 green", 0xff00),
    ("x11 grey", 0xbebebe),
    ("x11 maroon", 0xb03060),
    ("x11 purple", 0xa020f0),
    ("yellow green", 0x9acd32),
    ("yellow", 0xffff00),
    ("yellow1", 0xffff00),
    ("yellow2", 0xeeee00),
    ("yellow3", 0xcdcd00),
    ("yellow4", 0x8b8b00),
];

/// The names `colour_fromstring` accepts for the basic colours, each with the
/// number that is also accepted for it.
const BASIC_NAMES: [(&::core::ffi::CStr, Option<&str>, ::core::ffi::c_int); 18] = [
    (c"default", None, 8),
    (c"terminal", None, 9),
    (c"black", Some("0"), 0),
    (c"red", Some("1"), 1),
    (c"green", Some("2"), 2),
    (c"yellow", Some("3"), 3),
    (c"blue", Some("4"), 4),
    (c"magenta", Some("5"), 5),
    (c"cyan", Some("6"), 6),
    (c"white", Some("7"), 7),
    (c"brightblack", Some("90"), 90),
    (c"brightred", Some("91"), 91),
    (c"brightgreen", Some("92"), 92),
    (c"brightyellow", Some("93"), 93),
    (c"brightblue", Some("94"), 94),
    (c"brightmagenta", Some("95"), 95),
    (c"brightcyan", Some("96"), 96),
    (c"brightwhite", Some("97"), 97),
];

/// Squared distance between two RGB triples.
fn colour_dist_sq(
    R: ::core::ffi::c_int,
    G: ::core::ffi::c_int,
    B: ::core::ffi::c_int,
    r: ::core::ffi::c_int,
    g: ::core::ffi::c_int,
    b: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    (R - r) * (R - r) + (G - g) * (G - g) + (B - b) * (B - b)
}

/// The colour cube axis step nearest to one eight-bit component.
fn colour_to_6cube(v: ::core::ffi::c_int) -> ::core::ffi::c_int {
    if v < 48 {
        return 0;
    }
    if v < 114 {
        return 1;
    }
    (v - 35) / 40
}

/// The indexed colour closest to an RGB triple: the cube entry it quantises to,
/// unless a step of the grey ramp is nearer.
fn find_rgb(r: u_char, g: u_char, b: u_char) -> ::core::ffi::c_int {
    let (r, g, b) = (
        r as ::core::ffi::c_int,
        g as ::core::ffi::c_int,
        b as ::core::ffi::c_int,
    );
    let qr = colour_to_6cube(r);
    let qg = colour_to_6cube(g);
    let qb = colour_to_6cube(b);
    let cr = CUBE_STEPS[qr as usize];
    let cg = CUBE_STEPS[qg as usize];
    let cb = CUBE_STEPS[qb as usize];
    let cube = 16 + 36 * qr + 6 * qg + qb;
    if cr == r && cg == g && cb == b {
        return cube | COLOUR_FLAG_256;
    }
    let grey_avg = (r + g + b) / 3;
    let grey_idx = if grey_avg > 238 {
        23
    } else {
        (grey_avg - 3) / 10
    };
    let grey = 8 + 10 * grey_idx;
    let idx = if colour_dist_sq(grey, grey, grey, r, g, b) < colour_dist_sq(cr, cg, cb, r, g, b) {
        232 + grey_idx
    } else {
        cube
    };
    idx | COLOUR_FLAG_256
}

fn join_rgb(r: u_char, g: u_char, b: u_char) -> ::core::ffi::c_int {
    (r as ::core::ffi::c_int) << 16
        | (g as ::core::ffi::c_int) << 8
        | b as ::core::ffi::c_int
        | COLOUR_FLAG_RGB
}

fn split_rgb(c: ::core::ffi::c_int) -> (u_char, u_char, u_char) {
    (
        (c >> 16 & 0xff) as u_char,
        (c >> 8 & 0xff) as u_char,
        (c & 0xff) as u_char,
    )
}

fn rgb_of_256(c: ::core::ffi::c_int) -> ::core::ffi::c_int {
    RGB_OF_256[(c & 0xff) as usize] | COLOUR_FLAG_RGB
}

/// The RGB value of a colour that has one, or -1.
fn force_rgb(c: ::core::ffi::c_int) -> ::core::ffi::c_int {
    if c & COLOUR_FLAG_RGB != 0 {
        return c;
    }
    if c & COLOUR_FLAG_256 != 0 || (0..=7).contains(&c) {
        return rgb_of_256(c);
    }
    if (90..=97).contains(&c) {
        return rgb_of_256(8 + c - 90);
    }
    -1
}

/// Whether a colour reads as a light or a dark background.
fn totheme(c: ::core::ffi::c_int) -> client_theme {
    if c == -1 {
        return THEME_UNKNOWN;
    }
    if c & COLOUR_FLAG_RGB != 0 {
        let (r, g, b) = split_rgb(c);
        let brightness =
            r as ::core::ffi::c_int + g as ::core::ffi::c_int + b as ::core::ffi::c_int;
        return if brightness > 382 {
            THEME_LIGHT
        } else {
            THEME_DARK
        };
    }
    if c & COLOUR_FLAG_256 != 0 {
        return totheme(rgb_of_256(c));
    }
    match c {
        0 | 90 => THEME_DARK,
        7 | 97 => THEME_LIGHT,
        0..=7 => totheme(rgb_of_256(c)),
        90..=97 => totheme(rgb_of_256(8 + c - 90)),
        _ => THEME_UNKNOWN,
    }
}

/// The fixed name of a colour, if it has one.
fn name_of(c: ::core::ffi::c_int) -> Option<&'static ::core::ffi::CStr> {
    BASIC_NAMES
        .iter()
        .find(|(_, _, value)| *value == c)
        .map(|(name, _, _)| *name)
}

/// `strtonum` applied to `s` from byte `skip` on, or `None` when what follows
/// is not a whole number within `[0, max]`.
fn suffix_number(
    s: &::core::ffi::CStr,
    skip: usize,
    max: ::core::ffi::c_longlong,
) -> Option<::core::ffi::c_int> {
    let tail = &s.to_bytes_with_nul()[skip..];
    unsafe { strtonum(tail.as_ptr().cast::<::core::ffi::c_char>(), 0, max) }
        .ok()
        .map(|n| n as ::core::ffi::c_int)
}

fn starts_with_ignore_case(s: &[u8], prefix: &[u8]) -> bool {
    s.len() >= prefix.len() && s[..prefix.len()].eq_ignore_ascii_case(prefix)
}

/// An X11 colour name, a `grey`/`gray` percentage, or -1.
fn byname(name: &::core::ffi::CStr) -> ::core::ffi::c_int {
    let bytes = name.to_bytes();
    if starts_with_ignore_case(bytes, b"grey") || starts_with_ignore_case(bytes, b"gray") {
        if bytes.len() == 4 {
            return 0xbebebe | COLOUR_FLAG_RGB;
        }
        let Some(percent) = suffix_number(name, 4, 100) else {
            return -1;
        };
        let v = (2.55 * percent as ::core::ffi::c_double).round() as u_char;
        return join_rgb(v, v, v);
    }
    match X11_COLOURS
        .iter()
        .find(|(n, _)| bytes.eq_ignore_ascii_case(n.as_bytes()))
    {
        Some((_, c)) => c | COLOUR_FLAG_RGB,
        None => -1,
    }
}

/// A colour written as `#rrggbb`, `colourN`, a basic name or number, or an X11
/// colour name; -1 when none of those fit.
fn fromstring(s: &::core::ffi::CStr) -> ::core::ffi::c_int {
    let bytes = s.to_bytes();
    if bytes.len() == 7 && bytes[0] == b'#' {
        let digits = &bytes[1..];
        if !digits.iter().all(u8::is_ascii_hexdigit) {
            return -1;
        }
        let byte = |i: usize| {
            ((digits[i] as char).to_digit(16).unwrap() * 16
                + (digits[i + 1] as char).to_digit(16).unwrap()) as u_char
        };
        return join_rgb(byte(0), byte(2), byte(4));
    }
    for prefix in [b"colour".as_slice(), b"color".as_slice()] {
        if starts_with_ignore_case(bytes, prefix) {
            return match suffix_number(s, prefix.len(), 255) {
                Some(n) => n | COLOUR_FLAG_256,
                None => -1,
            };
        }
    }
    for (name, number, c) in BASIC_NAMES {
        if bytes.eq_ignore_ascii_case(name.to_bytes())
            || number.is_some_and(|number| bytes == number.as_bytes())
        {
            return c;
        }
    }
    byname(s)
}

pub fn colour_find_rgb(r: u_char, g: u_char, b: u_char) -> ::core::ffi::c_int {
    find_rgb(r, g, b)
}

pub fn colour_join_rgb(r: u_char, g: u_char, b: u_char) -> ::core::ffi::c_int {
    join_rgb(r, g, b)
}

pub unsafe fn colour_split_rgb(
    c: ::core::ffi::c_int,
    r: *mut u_char,
    g: *mut u_char,
    b: *mut u_char,
) {
    unsafe {
        let (cr, cg, cb) = split_rgb(c);
        *r = cr;
        *g = cg;
        *b = cb;
    }
}

pub fn colour_force_rgb(c: ::core::ffi::c_int) -> ::core::ffi::c_int {
    force_rgb(c)
}

/// The name a colour goes by, as the caller's own string.
pub fn colour_tostring(c: ::core::ffi::c_int) -> ::std::ffi::CString {
    if c == -1 {
        return c"none".to_owned();
    }
    if c & COLOUR_FLAG_RGB != 0 {
        let (r, g, b) = split_rgb(c);
        return ::std::ffi::CString::new(format!("#{r:02x}{g:02x}{b:02x}"))
            .expect("a colour name has no interior NUL");
    }
    if c & COLOUR_FLAG_256 != 0 {
        return ::std::ffi::CString::new(format!("colour{}", c & 0xff))
            .expect("a colour name has no interior NUL");
    }
    match name_of(c) {
        Some(name) => name.to_owned(),
        None => c"invalid".to_owned(),
    }
}

pub fn colour_totheme(c: ::core::ffi::c_int) -> client_theme {
    totheme(c)
}

pub unsafe fn colour_fromstring(s: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    unsafe { fromstring(::core::ffi::CStr::from_ptr(s)) }
}

pub fn colour_256toRGB(c: ::core::ffi::c_int) -> ::core::ffi::c_int {
    rgb_of_256(c)
}

pub fn colour_256to16(c: ::core::ffi::c_int) -> ::core::ffi::c_int {
    BASIC_OF_256[(c & 0xff) as usize]
}

pub unsafe fn colour_byname(name: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    unsafe { byname(::core::ffi::CStr::from_ptr(name)) }
}

/// Reads the X11 colour spellings tmux accepts. The numeric forms are matched
/// with `sscanf` so the accepted syntax stays exactly C's, including what
/// `%lf` takes for the CMYK components.
pub unsafe fn colour_parseX11(p: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    unsafe {
        let text = ::core::ffi::CStr::from_ptr(p).to_bytes();
        let len = text.len();
        let mut r: u_int = 0;
        let mut g: u_int = 0;
        let mut b: u_int = 0;
        let mut c: ::core::ffi::c_double = 0.;
        let mut m: ::core::ffi::c_double = 0.;
        let mut y: ::core::ffi::c_double = 0.;
        let mut k: ::core::ffi::c_double = 0.;
        let mut logged = p;

        let colour = if len == 12
            && sscanf(
                p,
                c"rgb:%02x/%02x/%02x".as_ptr(),
                &raw mut r,
                &raw mut g,
                &raw mut b,
            ) == 3
            || len == 7
                && sscanf(
                    p,
                    c"#%02x%02x%02x".as_ptr(),
                    &raw mut r,
                    &raw mut g,
                    &raw mut b,
                ) == 3
            || sscanf(p, c"%d,%d,%d".as_ptr(), &raw mut r, &raw mut g, &raw mut b) == 3
        {
            join_rgb(r as u_char, g as u_char, b as u_char)
        } else if len == 18
            && sscanf(
                p,
                c"rgb:%04x/%04x/%04x".as_ptr(),
                &raw mut r,
                &raw mut g,
                &raw mut b,
            ) == 3
            || len == 13
                && sscanf(
                    p,
                    c"#%04x%04x%04x".as_ptr(),
                    &raw mut r,
                    &raw mut g,
                    &raw mut b,
                ) == 3
        {
            join_rgb((r >> 8) as u_char, (g >> 8) as u_char, (b >> 8) as u_char)
        } else if (sscanf(
            p,
            c"cmyk:%lf/%lf/%lf/%lf".as_ptr(),
            &raw mut c,
            &raw mut m,
            &raw mut y,
            &raw mut k,
        ) == 4
            || sscanf(
                p,
                c"cmy:%lf/%lf/%lf".as_ptr(),
                &raw mut c,
                &raw mut m,
                &raw mut y,
            ) == 3)
            && (0.0..=1.0).contains(&c)
            && (0.0..=1.0).contains(&m)
            && (0.0..=1.0).contains(&y)
            && (0.0..=1.0).contains(&k)
        {
            join_rgb(
                ((1. - c) * (1. - k) * 255.) as u_char,
                ((1. - m) * (1. - k) * 255.) as u_char,
                ((1. - y) * (1. - k) * 255.) as u_char,
            )
        } else {
            let start = text.iter().position(|&b| b != b' ').unwrap_or(len);
            let end = text
                .iter()
                .rposition(|&b| b != b' ')
                .map_or(start, |i| i + 1);
            logged = p.add(start);
            let trimmed: Vec<::core::ffi::c_char> = text[start..end]
                .iter()
                .map(|&b| b as ::core::ffi::c_char)
                .chain(::core::iter::once(0))
                .collect();
            byname(::core::ffi::CStr::from_ptr(trimmed.as_ptr()))
        };
        log_debug(
            c"%s: %s = %s".as_ptr(),
            fmt_args![
                c"colour_parseX11".as_ptr(),
                logged,
                colour_tostring(colour).as_c_str()
            ],
        );
        colour
    }
}

pub unsafe fn colour_palette_init(p: *mut colour_palette) {
    unsafe {
        (*p).fg = 8;
        (*p).bg = 8;
        (*p).palette = None;
        (*p).default_palette = None;
    }
}

pub unsafe fn colour_palette_clear(p: *mut colour_palette) {
    unsafe {
        if !p.is_null() {
            (*p).fg = 8;
            (*p).bg = 8;
            (*p).palette = None;
        }
    }
}

pub unsafe fn colour_palette_free(p: *mut colour_palette) {
    unsafe {
        if !p.is_null() {
            (*p).palette = None;
            (*p).default_palette = None;
        }
    }
}

pub unsafe fn colour_palette_get(
    p: *mut colour_palette,
    n: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        if p.is_null() {
            return -1;
        }
        let n = if (90..=97).contains(&n) {
            8 + n - 90
        } else if n & COLOUR_FLAG_256 != 0 {
            n & !COLOUR_FLAG_256
        } else if n >= 8 {
            return -1;
        } else {
            n
        } as usize;
        if n >= 256 {
            return -1;
        }
        if let Some(ref pal) = (*p).palette
            && pal[n] != -1
        {
            return pal[n];
        }
        if let Some(ref def) = (*p).default_palette
            && def[n] != -1
        {
            return def[n];
        }
        -1
    }
}

pub unsafe fn colour_palette_set(
    p: *mut colour_palette,
    n: ::core::ffi::c_int,
    c: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        if p.is_null() || !(0..=255).contains(&n) {
            return 0;
        }
        if c == -1 && (*p).palette.is_none() {
            return 0;
        }
        let pal = (*p).palette.get_or_insert_with(|| Box::new([-1; 256]));
        pal[n as usize] = c;
        1
    }
}

/// Points the palette's default table at `defaults`, or drops the table when
/// the caller has none to give.
pub unsafe fn colour_palette_from_defaults(
    p: *mut colour_palette,
    defaults: Option<&[::core::ffi::c_int; 256]>,
) {
    unsafe {
        if p.is_null() {
            return;
        }
        match defaults {
            None => (*p).default_palette = None,
            Some(defaults) => {
                **(*p)
                    .default_palette
                    .get_or_insert_with(|| Box::new([-1; 256])) = *defaults;
            }
        }
    }
}

#[cfg(test)]
#[path = "../tests/test_colour.rs"]
mod tests;
