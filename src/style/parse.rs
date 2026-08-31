use super::attributes::{attributes_fromstring, attributes_tostring};
use super::colour::{colour_fromstring, colour_tostring};
use crate::compat::strtonum;
use crate::fmt_args;
use crate::format::format_create;
use crate::grid::grid_default_cell;
use crate::log::{fatalx, log_debug};
use crate::options::options_string_to_style;
use crate::text::utf8_set;
pub use crate::types::*;
use ::core::ffi::{CStr, c_char, c_int, c_longlong};
use ::core::ptr::null_mut;
use ::std::ffi::CString;
pub const UINT_MAX: ::core::ffi::c_uint = u32::MAX;
pub const GRID_ATTR_NOATTR: c_int = 0x4000 as c_int;
pub const STYLE_WIDTH_DEFAULT: c_int = -(1 as c_int);
pub const STYLE_PAD_DEFAULT: c_int = -(1 as c_int);
pub const PANE_SCROLLBARS_DEFAULT_PADDING: c_int = 0 as c_int;
pub const PANE_SCROLLBARS_DEFAULT_WIDTH: c_int = 1 as c_int;
pub const PANE_SCROLLBARS_CHARACTER: c_int = ' ' as i32;
pub const FORMAT_NOJOBS: c_int = 0x4 as c_int;
pub const STYLE_DEFAULT_SET: style_default_type = 3;
pub const STYLE_DEFAULT_POP: style_default_type = 2;
pub const STYLE_DEFAULT_PUSH: style_default_type = 1;
pub const STYLE_DEFAULT_BASE: style_default_type = 0;
pub const STYLE_RANGE_CONTROL: style_range_type = 7;
pub const STYLE_RANGE_USER: style_range_type = 6;
pub const STYLE_RANGE_SESSION: style_range_type = 5;
pub const STYLE_RANGE_WINDOW: style_range_type = 4;
pub const STYLE_RANGE_PANE: style_range_type = 3;
pub const STYLE_RANGE_RIGHT: style_range_type = 2;
pub const STYLE_RANGE_LEFT: style_range_type = 1;
pub const STYLE_RANGE_NONE: style_range_type = 0;
pub const STYLE_LIST_RIGHT_MARKER: style_list = 4;
pub const STYLE_LIST_LEFT_MARKER: style_list = 3;
pub const STYLE_LIST_FOCUS: style_list = 2;
pub const STYLE_LIST_ON: style_list = 1;
pub const STYLE_LIST_OFF: style_list = 0;
pub const STYLE_ALIGN_ABSOLUTE_CENTRE: style_align = 4;
pub const STYLE_ALIGN_RIGHT: style_align = 3;
pub const STYLE_ALIGN_CENTRE: style_align = 2;
pub const STYLE_ALIGN_LEFT: style_align = 1;
pub const STYLE_ALIGN_DEFAULT: style_align = 0;

/// The colour a style names when it wants whatever the base cell has.
const COLOUR_DEFAULT: c_int = 8 as c_int;

/// The bytes that separate one word of a style from the next.
const DELIMITERS: [u8; 3] = *b" ,\n";

/// How much room a style keeps for the argument of `range=user`, terminator
/// included.
const RANGE_STRING_SIZE: usize = 16;

/// The longest word the parser will look at, terminator included: upstream
/// copies each word into a buffer this big before reading it.
const WORD_SIZE: usize = 256;

/// How much room `style_tostring` has to print into, terminator included.
const TOSTRING_SIZE: usize = 256;

/// The style every parse starts from: a plain space cell, no fill, no
/// alignment and no range.
pub static style_default: style = style {
    gc: grid_cell {
        data: utf8_data {
            data: {
                let mut data = [0 as u_char; 32];
                data[0] = ' ' as i32 as u_char;
                data
            },
            have: 0 as u_char,
            size: 1 as u_char,
            width: 1 as u_char,
        },
        attr: 0 as u_short,
        flags: 0 as u_char,
        fg: COLOUR_DEFAULT,
        bg: COLOUR_DEFAULT,
        us: 0 as c_int,
        link: 0 as u_int,
    },
    ignore: 0 as c_int,
    fill: COLOUR_DEFAULT,
    align: STYLE_ALIGN_DEFAULT,
    list: STYLE_LIST_OFF,
    range_type: STYLE_RANGE_NONE,
    range_argument: 0 as u_int,
    range_string: [0 as c_char; RANGE_STRING_SIZE],
    width: STYLE_WIDTH_DEFAULT,
    width_percentage: 0 as c_int,
    pad: STYLE_PAD_DEFAULT,
    default_type: STYLE_DEFAULT_BASE,
};

/// Puts `s` in the style's room for a `range=user` argument, cut to fit the
/// way `strlcpy` cut it.
fn style_set_range_string(sy: &mut style, s: &[u8]) {
    let room = &mut sy.range_string;
    let kept = s.len().min(RANGE_STRING_SIZE - 1);
    for (cell, byte) in room.iter_mut().zip(&s[..kept]) {
        *cell = *byte as c_char;
    }
    room[kept] = 0;
}

/// A number between `lower` and `upper`, or nothing when the text is not one.
/// The syntax and the bounds are `strtonum`'s, so what a style takes stays
/// exactly what C took.
fn style_number(s: &[u8], lower: c_longlong, upper: c_longlong) -> Option<u_int> {
    let text = CString::new(s).ok()?;
    unsafe { strtonum(text.as_ptr(), lower, upper) }
        .ok()
        .map(|n| n as u_int)
}

/// The whole number a style may name, which is every one a `u_int` holds.
const ANY: c_longlong = UINT_MAX as c_longlong;

/// Whether the word is exactly `name`, ignoring case as `strcasecmp` did.
fn style_is(w: &[u8], name: &[u8]) -> bool {
    w.eq_ignore_ascii_case(name)
}

/// The word with `prefix` taken off it, when it starts with it and carries
/// something after it. Upstream spells this as a length test beside a
/// `strncasecmp`, and a word that is no longer than its prefix falls through
/// to the arms below.
fn style_after<'a>(w: &'a [u8], prefix: &[u8]) -> Option<&'a [u8]> {
    if w.len() > prefix.len() && w[..prefix.len()].eq_ignore_ascii_case(prefix) {
        Some(&w[prefix.len()..])
    } else {
        None
    }
}

/// The colour `s` names, or an error when it names none.
fn style_colour(s: &[u8]) -> Result<c_int, ()> {
    unsafe {
        let text = CString::new(s).map_err(|_| ())?;
        match colour_fromstring(text.as_ptr()) {
            -1 => Err(()),
            value => Ok(value),
        }
    }
}

/// The attribute bits `s` names, or an error when it names none.
fn style_attributes(s: &[u8]) -> Result<c_int, ()> {
    let text = CString::new(s).map_err(|_| ())?;
    match attributes_fromstring(&text) {
        -1 => Err(()),
        value => Ok(value),
    }
}

/// Reads what follows `range=`: a kind, then a `|` and an argument for the
/// kinds that want one.
fn style_parse_range(sy: &mut style, rest: &[u8]) -> Result<(), ()> {
    let (kind, argument) = match rest.iter().position(|&b| b == b'|') {
        Some(bar) if bar + 1 == rest.len() => return Err(()),
        Some(bar) => (&rest[..bar], Some(&rest[bar + 1..])),
        None => (rest, None),
    };
    let (range_type, range_argument, range_string) = if style_is(kind, b"left") {
        (STYLE_RANGE_LEFT, 0 as u_int, &b""[..])
    } else if style_is(kind, b"right") {
        (STYLE_RANGE_RIGHT, 0 as u_int, &b""[..])
    } else if style_is(kind, b"control") {
        let n = style_number(argument.ok_or(())?, 0, 9).ok_or(())?;
        (STYLE_RANGE_CONTROL, n, &b""[..])
    } else if style_is(kind, b"pane") {
        let id = argument.ok_or(())?.strip_prefix(b"%").ok_or(())?;
        (
            STYLE_RANGE_PANE,
            style_number(id, 0, ANY).ok_or(())?,
            &b""[..],
        )
    } else if style_is(kind, b"window") {
        let n = style_number(argument.ok_or(())?, 0, ANY).ok_or(())?;
        (STYLE_RANGE_WINDOW, n, &b""[..])
    } else if style_is(kind, b"session") {
        let id = argument.ok_or(())?.strip_prefix(b"$").ok_or(())?;
        (
            STYLE_RANGE_SESSION,
            style_number(id, 0, ANY).ok_or(())?,
            &b""[..],
        )
    } else if style_is(kind, b"user") {
        (STYLE_RANGE_USER, 0 as u_int, argument.ok_or(())?)
    } else {
        // No arm of the C reads an unknown kind, and none reports it either:
        // the word is taken and dropped.
        return Ok(());
    };
    if (range_type == STYLE_RANGE_LEFT || range_type == STYLE_RANGE_RIGHT) && argument.is_some() {
        return Err(());
    }
    sy.range_type = range_type;
    sy.range_argument = range_argument;
    style_set_range_string(sy, range_string);
    Ok(())
}

/// Reads one word of a style.
fn style_parse_word(sy: &mut style, base: &grid_cell, w: &[u8]) -> Result<(), ()> {
    if style_is(w, b"default") {
        sy.gc.fg = base.fg;
        sy.gc.bg = base.bg;
        sy.gc.us = base.us;
        sy.gc.attr = base.attr;
        sy.gc.flags = base.flags;
    } else if style_is(w, b"ignore") {
        sy.ignore = 1 as c_int;
    } else if style_is(w, b"noignore") {
        sy.ignore = 0 as c_int;
    } else if style_is(w, b"push-default") {
        sy.default_type = STYLE_DEFAULT_PUSH;
    } else if style_is(w, b"pop-default") {
        sy.default_type = STYLE_DEFAULT_POP;
    } else if style_is(w, b"set-default") {
        sy.default_type = STYLE_DEFAULT_SET;
    } else if style_is(w, b"nolist") {
        sy.list = STYLE_LIST_OFF;
    } else if w.len() >= 5 && w[..5].eq_ignore_ascii_case(b"list=") {
        let rest = &w[5..];
        sy.list = if style_is(rest, b"on") {
            STYLE_LIST_ON
        } else if style_is(rest, b"focus") {
            STYLE_LIST_FOCUS
        } else if style_is(rest, b"left-marker") {
            STYLE_LIST_LEFT_MARKER
        } else if style_is(rest, b"right-marker") {
            STYLE_LIST_RIGHT_MARKER
        } else {
            return Err(());
        };
    } else if style_is(w, b"norange") {
        sy.range_type = style_default.range_type;
        // Upstream writes the default range *type* into the argument here.
        // Both are zero, so the answer is the same either way.
        sy.range_argument = style_default.range_type as u_int;
        style_set_range_string(sy, b"");
    } else if let Some(rest) = style_after(w, b"range=") {
        style_parse_range(sy, rest)?;
    } else if style_is(w, b"noalign") {
        sy.align = style_default.align;
    } else if let Some(rest) = style_after(w, b"align=") {
        sy.align = if style_is(rest, b"left") {
            STYLE_ALIGN_LEFT
        } else if style_is(rest, b"centre") {
            STYLE_ALIGN_CENTRE
        } else if style_is(rest, b"right") {
            STYLE_ALIGN_RIGHT
        } else if style_is(rest, b"absolute-centre") {
            STYLE_ALIGN_ABSOLUTE_CENTRE
        } else {
            return Err(());
        };
    } else if let Some(rest) = style_after(w, b"fill=") {
        sy.fill = style_colour(rest)?;
    } else if w.len() > 3 && w[1..3].eq_ignore_ascii_case(b"g=") {
        let value = style_colour(&w[3..])?;
        match w[0] {
            b'f' | b'F' => {
                sy.gc.fg = if value != COLOUR_DEFAULT {
                    value
                } else {
                    base.fg
                };
            }
            b'b' | b'B' => {
                sy.gc.bg = if value != COLOUR_DEFAULT {
                    value
                } else {
                    base.bg
                };
            }
            _ => return Err(()),
        }
    } else if let Some(rest) = style_after(w, b"us=") {
        let value = style_colour(rest)?;
        sy.gc.us = if value != COLOUR_DEFAULT {
            value
        } else {
            base.us
        };
    } else if style_is(w, b"none") {
        sy.gc.attr = 0 as u_short;
    } else if let Some(rest) = style_after(w, b"no") {
        // `noattr` is the one word the parser matches case-sensitively.
        if rest == b"attr" {
            sy.gc.attr = (sy.gc.attr as c_int | GRID_ATTR_NOATTR) as u_short;
        } else {
            sy.gc.attr = (sy.gc.attr as c_int & !style_attributes(rest)?) as u_short;
        }
    } else if let Some(rest) = style_after(w, b"width=") {
        // A percentage wants a digit in front of the sign, which is what
        // asking for more than seven characters in all comes to.
        match rest.strip_suffix(b"%").filter(|_| w.len() > 7) {
            Some(n) => {
                sy.width = style_number(n, 0, 100).ok_or(())? as c_int;
                sy.width_percentage = 1 as c_int;
            }
            None => {
                sy.width = style_number(rest, 0, ANY).ok_or(())? as c_int;
                sy.width_percentage = 0 as c_int;
            }
        }
    } else if let Some(rest) = style_after(w, b"pad=") {
        sy.pad = style_number(rest, 0, ANY).ok_or(())? as c_int;
    } else {
        sy.gc.attr = (sy.gc.attr as c_int | style_attributes(w)?) as u_short;
    }
    Ok(())
}

pub unsafe fn style_parse(sy: &mut style, base: &grid_cell, input: &[u8]) -> c_int {
    unsafe {
        if input.is_empty() {
            return 0 as c_int;
        }
        let saved: style = *sy;
        log_debug(
            c"%s: %.*s".as_ptr(),
            fmt_args![
                c"style_parse".as_ptr(),
                input.len() as c_int,
                input.as_ptr()
            ],
        );
        for w in input
            .split(|b| DELIMITERS.contains(b))
            .filter(|w| !w.is_empty())
        {
            let refused = w.len() > WORD_SIZE - 1 || {
                let text = CString::new(w).unwrap_or_default();
                log_debug(
                    c"%s: %s".as_ptr(),
                    fmt_args![c"style_parse".as_ptr(), text.as_ptr()],
                );
                style_parse_word(sy, base, w).is_err()
            };
            if refused {
                *sy = saved;
                return -(1 as c_int);
            }
        }
        0 as c_int
    }
}

/// The name a colour goes by.
fn style_colour_name(colour: c_int) -> String {
    colour_tostring(colour).to_string_lossy().into_owned()
}

/// How a style is written out, as the caller's own string.
pub unsafe fn style_tostring(sy: &style) -> ::std::ffi::CString {
    unsafe {
        let gc = &sy.gc;
        let mut parts: Vec<String> = Vec::new();
        // The name each block prints is one variable that lives across all of
        // them, and a block whose value has no arm leaves whatever the block
        // before it put there. That is how a control range comes out under the
        // name of the list beside it.
        let mut tmp = String::new();
        if sy.list != STYLE_LIST_OFF {
            if sy.list == STYLE_LIST_ON {
                tmp = "on".to_owned();
            } else if sy.list == STYLE_LIST_FOCUS {
                tmp = "focus".to_owned();
            } else if sy.list == STYLE_LIST_LEFT_MARKER {
                tmp = "left-marker".to_owned();
            } else if sy.list == STYLE_LIST_RIGHT_MARKER {
                tmp = "right-marker".to_owned();
            }
            parts.push(format!("list={tmp}"));
        }
        if sy.range_type != STYLE_RANGE_NONE {
            let argument = sy.range_argument;
            if sy.range_type == STYLE_RANGE_LEFT {
                tmp = "left".to_owned();
            } else if sy.range_type == STYLE_RANGE_RIGHT {
                tmp = "right".to_owned();
            } else if sy.range_type == STYLE_RANGE_PANE {
                tmp = format!("pane|%{argument}");
            } else if sy.range_type == STYLE_RANGE_WINDOW {
                tmp = format!("window|{argument}");
            } else if sy.range_type == STYLE_RANGE_SESSION {
                tmp = format!("session|${argument}");
            } else if sy.range_type == STYLE_RANGE_USER {
                tmp = format!(
                    "user|{}",
                    CStr::from_ptr(&raw const sy.range_string as *const c_char).to_string_lossy()
                );
            }
            parts.push(format!("range={tmp}"));
        }
        if sy.align != STYLE_ALIGN_DEFAULT {
            if sy.align == STYLE_ALIGN_LEFT {
                tmp = "left".to_owned();
            } else if sy.align == STYLE_ALIGN_CENTRE {
                tmp = "centre".to_owned();
            } else if sy.align == STYLE_ALIGN_RIGHT {
                tmp = "right".to_owned();
            } else if sy.align == STYLE_ALIGN_ABSOLUTE_CENTRE {
                tmp = "absolute-centre".to_owned();
            }
            parts.push(format!("align={tmp}"));
        }
        if sy.default_type != STYLE_DEFAULT_BASE {
            if sy.default_type == STYLE_DEFAULT_PUSH {
                tmp = "push-default".to_owned();
            } else if sy.default_type == STYLE_DEFAULT_POP {
                tmp = "pop-default".to_owned();
            } else if sy.default_type == STYLE_DEFAULT_SET {
                tmp = "set-default".to_owned();
            }
            parts.push(tmp.clone());
        }
        if sy.fill != COLOUR_DEFAULT {
            parts.push(format!("fill={}", style_colour_name(sy.fill)));
        }
        for (label, colour) in [("fg", gc.fg), ("bg", gc.bg), ("us", gc.us)] {
            if colour != COLOUR_DEFAULT {
                parts.push(format!("{label}={}", style_colour_name(colour)));
            }
        }
        if gc.attr as c_int != 0 as c_int {
            let name = attributes_tostring(gc.attr as c_int);
            let name = name.to_string_lossy();
            parts.push(name.into_owned());
        }
        if sy.width >= 0 as c_int {
            let width = sy.width as u_int;
            let percent = if sy.width_percentage != 0 { "%" } else { "" };
            parts.push(format!("width={width}{percent}"));
        }
        if sy.pad >= 0 as c_int {
            parts.push(format!("pad={}", sy.pad as u_int));
        }
        if parts.is_empty() {
            return c"default".to_owned();
        }
        let out = parts.join(",");
        // Upstream prints a piece at a time with `xsnprintf`, which stops the
        // server when a piece will not fit; the whole answer not fitting is the
        // same condition.
        if out.len() > TOSTRING_SIZE - 1 {
            fatalx(c"xsnprintf: overflow".as_ptr(), fmt_args![]);
        }
        ::std::ffi::CString::new(out).expect("a style has no interior NUL")
    }
}

pub unsafe fn style_add(
    gc: *mut grid_cell,
    oo: *mut options,
    name: *const c_char,
    ft: Option<&mut format_tree>,
) {
    unsafe {
        let mut ft0: Option<Box<format_tree>> = None;
        let ft: &mut format_tree = match ft {
            Some(ft) => ft,
            None => ft0.insert(format_create(
                null_mut::<client>(),
                null_mut::<cmdq_item>(),
                0 as c_int,
                FORMAT_NOJOBS,
            )),
        };
        let sy = options_string_to_style(oo, name, Some(&mut *ft));
        let sy: &style = if sy.is_null() { &style_default } else { &*sy };
        if sy.gc.fg != COLOUR_DEFAULT {
            (*gc).fg = sy.gc.fg;
        }
        if sy.gc.bg != COLOUR_DEFAULT {
            (*gc).bg = sy.gc.bg;
        }
        if sy.gc.us != COLOUR_DEFAULT {
            (*gc).us = sy.gc.us;
        }
        (*gc).attr = ((*gc).attr as c_int | sy.gc.attr as c_int) as u_short;
    }
}

pub unsafe fn style_apply(
    gc: *mut grid_cell,
    oo: *mut options,
    name: *const c_char,
    ft: Option<&mut format_tree>,
) {
    unsafe {
        *gc = grid_default_cell;
        style_add(gc, oo, name, ft);
    }
}

pub fn style_set(sy: &mut style, gc: &grid_cell) {
    *sy = style_default;
    sy.gc = *gc;
}

pub fn style_copy(dst: &mut style, src: &style) {
    *dst = *src;
}

pub unsafe fn style_set_scrollbar_style_from_option(sb_style: &mut style, oo: *mut options) {
    unsafe {
        let sy = options_string_to_style(oo, c"pane-scrollbars-style".as_ptr(), None);
        if sy.is_null() {
            style_set(sb_style, &grid_default_cell);
            sb_style.width = PANE_SCROLLBARS_DEFAULT_WIDTH;
            sb_style.pad = PANE_SCROLLBARS_DEFAULT_PADDING;
        } else {
            style_copy(sb_style, &*sy);
            if sb_style.width < 1 as c_int {
                sb_style.width = PANE_SCROLLBARS_DEFAULT_WIDTH;
            }
            if sb_style.pad < 0 as c_int {
                sb_style.pad = PANE_SCROLLBARS_DEFAULT_PADDING;
            }
        }
        utf8_set(&mut sb_style.gc.data, PANE_SCROLLBARS_CHARACTER as u_char);
    }
}

/// Gives `srs` an empty range list. The caller owns raw memory here — a pane
/// or a client fresh out of `xcalloc` — so this writes over it rather than
/// dropping what was there.
pub unsafe fn style_ranges_init(srs: *mut style_ranges) {
    unsafe {
        ::core::ptr::write(srs, style_ranges::new());
    }
}

/// Lets go of every range on a list that is already live, leaving it empty and
/// ready to be drawn into again. The list keeps no allocation afterwards, so a
/// caller tearing its owner down can stop here.
pub unsafe fn style_ranges_free(srs: *mut style_ranges) {
    unsafe {
        *srs = style_ranges::new();
    }
}

pub unsafe fn style_ranges_get_range(srs: *mut style_ranges, x: u_int) -> *mut style_range {
    unsafe {
        if srs.is_null() {
            return null_mut::<style_range>();
        }
        for sr in (*srs).iter_mut() {
            if x >= sr.start && x < sr.end {
                return sr as *mut style_range;
            }
        }
        null_mut::<style_range>()
    }
}

#[cfg(test)]
#[path = "../tests/test_style.rs"]
mod tests;
