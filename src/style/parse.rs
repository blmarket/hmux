use super::attributes::{AttributeCodec, RustAttributeCodec};
use super::colour::{ColourEngine, RustColourEngine};
use crate::compat::strtonum;
use crate::fmt_args;
use crate::format::format_create;
use crate::grid::grid_default_cell;
use crate::log::{fatalx, log_debug};

pub use crate::consts::{
    FORMAT_NOJOBS, GRID_ATTR_NOATTR, STYLE_ALIGN_ABSOLUTE_CENTRE, STYLE_ALIGN_CENTRE,
    STYLE_ALIGN_DEFAULT, STYLE_ALIGN_LEFT, STYLE_ALIGN_RIGHT, STYLE_DEFAULT_BASE,
    STYLE_DEFAULT_POP, STYLE_DEFAULT_PUSH, STYLE_DEFAULT_SET, STYLE_LIST_FOCUS,
    STYLE_LIST_LEFT_MARKER, STYLE_LIST_OFF, STYLE_LIST_ON, STYLE_LIST_RIGHT_MARKER,
    STYLE_RANGE_CONTROL, STYLE_RANGE_LEFT, STYLE_RANGE_NONE, STYLE_RANGE_PANE, STYLE_RANGE_RIGHT,
    STYLE_RANGE_SESSION, STYLE_RANGE_USER, STYLE_RANGE_WINDOW, UINT_MAX,
};
use crate::text::utf8_set;
pub use crate::types::*;
use ::core::ffi::{CStr, c_char, c_int, c_longlong};
use ::std::ffi::CString;

pub const STYLE_WIDTH_DEFAULT: c_int = -(1 as c_int);
pub const STYLE_PAD_DEFAULT: c_int = -(1 as c_int);
pub const PANE_SCROLLBARS_DEFAULT_PADDING: c_int = 0 as c_int;
pub const PANE_SCROLLBARS_DEFAULT_WIDTH: c_int = 1 as c_int;
pub const PANE_SCROLLBARS_CHARACTER: c_int = ' ' as i32;

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

/// A style string that could not be parsed without changing its input style.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StyleParseError;

/// Parsing, serialization, and value construction for terminal styles.
pub trait StyleCodec {
    /// The style representation operated on by this implementation.
    type Style: crate::Style + Default;

    /// The terminal-cell representation used as a style's base.
    type Cell: crate::GridCell + Default;

    /// Adds `input` to `style`, restoring it when any word is invalid.
    fn parse(
        &self,
        style: &mut Self::Style,
        base: &Self::Cell,
        input: &[u8],
    ) -> Result<(), StyleParseError>;

    /// Serializes all nondefault parts of `style`.
    fn to_string(&self, style: &Self::Style) -> CString;

    /// Initializes a style from a terminal cell.
    fn set(&self, style: &mut Self::Style, cell: &Self::Cell);

    /// Replaces `destination` with `source`.
    fn copy(&self, destination: &mut Self::Style, source: &Self::Style);
}

/// Style codec implemented by hmux.
#[derive(Clone, Copy, Debug, Default)]
pub struct RustStyleCodec;

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
    unsafe { strtonum(&text, lower, upper) }
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
    let text = CString::new(s).map_err(|_| ())?;
    match RustColourEngine.from_string(&text) {
        -1 => Err(()),
        value => Ok(value),
    }
}

/// The attribute bits `s` names, or an error when it names none.
fn style_attributes(s: &[u8]) -> Result<c_int, ()> {
    let text = CString::new(s).map_err(|_| ())?;
    match RustAttributeCodec.from_string(&text) {
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

unsafe fn style_parse_impl(sy: &mut style, base: &grid_cell, input: &[u8]) -> c_int {
    {
        if input.is_empty() {
            return 0 as c_int;
        }
        let saved: style = *sy;
        log_debug(
            c"%s: %.*s",
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
                log_debug(c"%s: %s", fmt_args![c"style_parse".as_ptr(), text.as_ptr()]);
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
fn style_colour_name(colour: c_int) -> CString {
    RustColourEngine.to_string(colour)
}

fn style_part(prefix: &[u8], value: &CStr) -> CString {
    let mut part = Vec::with_capacity(prefix.len() + value.to_bytes().len());
    part.extend_from_slice(prefix);
    part.extend_from_slice(value.to_bytes());
    CString::new(part).expect("a style part has no interior NUL")
}

/// How a style is written out, as the caller's own string.
unsafe fn style_tostring_impl(sy: &style) -> CString {
    unsafe {
        let gc = &sy.gc;
        let mut parts: Vec<CString> = Vec::new();
        // The name each block prints is one variable that lives across all of
        // them, and a block whose value has no arm leaves whatever the block
        // before it put there. That is how a control range comes out under the
        // name of the list beside it.
        let mut tmp = CString::default();
        if sy.list != STYLE_LIST_OFF {
            if sy.list == STYLE_LIST_ON {
                tmp = c"on".to_owned();
            } else if sy.list == STYLE_LIST_FOCUS {
                tmp = c"focus".to_owned();
            } else if sy.list == STYLE_LIST_LEFT_MARKER {
                tmp = c"left-marker".to_owned();
            } else if sy.list == STYLE_LIST_RIGHT_MARKER {
                tmp = c"right-marker".to_owned();
            }
            parts.push(style_part(b"list=", &tmp));
        }
        if sy.range_type != STYLE_RANGE_NONE {
            let argument = sy.range_argument;
            if sy.range_type == STYLE_RANGE_LEFT {
                tmp = c"left".to_owned();
            } else if sy.range_type == STYLE_RANGE_RIGHT {
                tmp = c"right".to_owned();
            } else if sy.range_type == STYLE_RANGE_PANE {
                tmp = CString::new(format!("pane|%{argument}"))
                    .expect("a numeric style range has no interior NUL");
            } else if sy.range_type == STYLE_RANGE_WINDOW {
                tmp = CString::new(format!("window|{argument}"))
                    .expect("a numeric style range has no interior NUL");
            } else if sy.range_type == STYLE_RANGE_SESSION {
                tmp = CString::new(format!("session|${argument}"))
                    .expect("a numeric style range has no interior NUL");
            } else if sy.range_type == STYLE_RANGE_USER {
                tmp = style_part(
                    b"user|",
                    CStr::from_ptr(&raw const sy.range_string as *const c_char),
                );
            }
            parts.push(style_part(b"range=", &tmp));
        }
        if sy.align != STYLE_ALIGN_DEFAULT {
            if sy.align == STYLE_ALIGN_LEFT {
                tmp = c"left".to_owned();
            } else if sy.align == STYLE_ALIGN_CENTRE {
                tmp = c"centre".to_owned();
            } else if sy.align == STYLE_ALIGN_RIGHT {
                tmp = c"right".to_owned();
            } else if sy.align == STYLE_ALIGN_ABSOLUTE_CENTRE {
                tmp = c"absolute-centre".to_owned();
            }
            parts.push(style_part(b"align=", &tmp));
        }
        if sy.default_type != STYLE_DEFAULT_BASE {
            if sy.default_type == STYLE_DEFAULT_PUSH {
                tmp = c"push-default".to_owned();
            } else if sy.default_type == STYLE_DEFAULT_POP {
                tmp = c"pop-default".to_owned();
            } else if sy.default_type == STYLE_DEFAULT_SET {
                tmp = c"set-default".to_owned();
            }
            parts.push(tmp.clone());
        }
        if sy.fill != COLOUR_DEFAULT {
            parts.push(style_part(b"fill=", &style_colour_name(sy.fill)));
        }
        for (label, colour) in [("fg", gc.fg), ("bg", gc.bg), ("us", gc.us)] {
            if colour != COLOUR_DEFAULT {
                parts.push(style_part(
                    format!("{label}=").as_bytes(),
                    &style_colour_name(colour),
                ));
            }
        }
        if gc.attr as c_int != 0 as c_int {
            parts.push(RustAttributeCodec.to_string(gc.attr as c_int));
        }
        if sy.width >= 0 as c_int {
            let width = sy.width as u_int;
            let percent = if sy.width_percentage != 0 { "%" } else { "" };
            parts.push(
                CString::new(format!("width={width}{percent}"))
                    .expect("a numeric style width has no interior NUL"),
            );
        }
        if sy.pad >= 0 as c_int {
            parts.push(
                CString::new(format!("pad={}", sy.pad as u_int))
                    .expect("a numeric style pad has no interior NUL"),
            );
        }
        if parts.is_empty() {
            return c"default".to_owned();
        }
        let mut out = Vec::new();
        for part in parts {
            if !out.is_empty() {
                out.push(b',');
            }
            out.extend_from_slice(part.to_bytes());
        }
        // Upstream prints a piece at a time with `xsnprintf`, which stops the
        // server when a piece will not fit; the whole answer not fitting is the
        // same condition.
        if out.len() > TOSTRING_SIZE - 1 {
            fatalx(c"xsnprintf: overflow", fmt_args![]);
        }
        CString::new(out).expect("a style has no interior NUL")
    }
}

pub unsafe fn style_add(
    gc: &mut grid_cell,
    oo: &RustOptionsRef,
    name: &CStr,
    ft: Option<&mut format_tree>,
) {
    unsafe {
        let mut ft0: Option<Box<format_tree>> = None;
        let ft: &mut format_tree = match ft {
            Some(ft) => ft,
            None => ft0.insert(format_create(None, None, 0 as c_int, FORMAT_NOJOBS)),
        };
        let sy = (oo).style_value(name, Some(ft));
        let sy = sy.as_ref().unwrap_or(&style_default);
        if sy.gc.fg != COLOUR_DEFAULT {
            gc.fg = sy.gc.fg;
        }
        if sy.gc.bg != COLOUR_DEFAULT {
            gc.bg = sy.gc.bg;
        }
        if sy.gc.us != COLOUR_DEFAULT {
            gc.us = sy.gc.us;
        }
        gc.attr = (gc.attr as c_int | sy.gc.attr as c_int) as u_short;
    }
}

pub unsafe fn style_apply(
    gc: &mut grid_cell,
    oo: &RustOptionsRef,
    name: &CStr,
    ft: Option<&mut format_tree>,
) {
    unsafe {
        *gc = grid_default_cell;
        style_add(gc, oo, name, ft);
    }
}

fn style_set_impl(sy: &mut style, gc: &grid_cell) {
    *sy = style_default;
    sy.gc = *gc;
}

fn style_copy_impl(dst: &mut style, src: &style) {
    *dst = *src;
}

impl StyleCodec for RustStyleCodec {
    type Style = style;
    type Cell = grid_cell;

    fn parse(
        &self,
        style: &mut Self::Style,
        base: &Self::Cell,
        input: &[u8],
    ) -> Result<(), StyleParseError> {
        match unsafe { style_parse_impl(style, base, input) } {
            0 => Ok(()),
            _ => Err(StyleParseError),
        }
    }

    fn to_string(&self, style: &Self::Style) -> CString {
        unsafe { style_tostring_impl(style) }
    }

    fn set(&self, style: &mut Self::Style, cell: &Self::Cell) {
        style_set_impl(style, cell);
    }

    fn copy(&self, destination: &mut Self::Style, source: &Self::Style) {
        style_copy_impl(destination, source);
    }
}

pub unsafe fn style_parse(sy: &mut style, base: &grid_cell, input: &[u8]) -> c_int {
    RustStyleCodec.parse(sy, base, input).map_or(-1, |()| 0)
}

pub unsafe fn style_tostring(sy: &style) -> CString {
    RustStyleCodec.to_string(sy)
}

pub fn style_set(sy: &mut style, gc: &grid_cell) {
    RustStyleCodec.set(sy, gc);
}

pub fn style_copy(dst: &mut style, src: &style) {
    RustStyleCodec.copy(dst, src);
}

pub unsafe fn style_set_scrollbar_style_from_option(sb_style: &mut style, oo: &RustOptionsRef) {
    unsafe {
        let sy = (oo).style_value(c"pane-scrollbars-style", None);
        if let Some(sy) = sy {
            style_copy(sb_style, &sy);
            if sb_style.width < 1 as c_int {
                sb_style.width = PANE_SCROLLBARS_DEFAULT_WIDTH;
            }
            if sb_style.pad < 0 as c_int {
                sb_style.pad = PANE_SCROLLBARS_DEFAULT_PADDING;
            }
        } else {
            style_set(sb_style, &grid_default_cell);
            sb_style.width = PANE_SCROLLBARS_DEFAULT_WIDTH;
            sb_style.pad = PANE_SCROLLBARS_DEFAULT_PADDING;
        }
        utf8_set(&mut sb_style.gc.data, PANE_SCROLLBARS_CHARACTER as u_char);
    }
}

pub unsafe fn pane_scrollbar_style_from_option(
    oo: &RustOptionsRef,
) -> crate::pane_scrollbar_style::PaneScrollbarStyle {
    unsafe {
        let mut style = style::default();
        style_set_scrollbar_style_from_option(&mut style, oo);
        crate::pane_scrollbar_style::PaneScrollbarStyle {
            cell: style.gc,
            width: style.width,
            padding: style.pad,
        }
    }
}

/// Lets go of every range on a list that is already live, leaving it empty and
/// ready to be drawn into again. The list keeps no allocation afterwards, so a
/// caller tearing its owner down can stop here.
pub fn style_ranges_free(srs: &mut style_ranges) {
    *srs = style_ranges::new();
}

pub fn style_ranges_get_range(srs: &style_ranges, x: u_int) -> Option<style_range> {
    srs.iter().find(|sr| x >= sr.start && x < sr.end).copied()
}

#[cfg(test)]
#[path = "../tests/test_style.rs"]
mod tests;
