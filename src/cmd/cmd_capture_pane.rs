//! `capture-pane`: copies what a pane holds into a paste buffer or onto a
//! client's stdout, and `clear-history`, which throws the pane's scrollback
//! away. Both are the same exec routine, told apart by the entry the command
//! carries.
//!
//! A capture is a range of grid lines — `-S` and `-E` name its ends, counting
//! zero from the top of the screen and downwards into history — read out one
//! line at a time and joined into one buffer. Which grid is read depends on
//! `-a` (the saved alternate screen), `-M` (the screen the pane's mode hands
//! over) and otherwise the pane's own; `-P` reads the input parser's pending
//! bytes instead of the grid at all, and `-H` reads the hyperlink URIs the
//! lines carry rather than their text. The flags that shape a line —
//! escape sequences, escaped control bytes, empty cells, trimmed trailing
//! spaces — are handed to `grid_string_cells`, which is what turns cells into
//! text.
//!
//! The answer is built in a `Vec<u8>` which `paste_set` takes over, or which
//! `-p` writes out after appending the terminator the printing path wants.
//!
//! Quirks kept: the width every line is read out to comes from the pane's own
//! base grid even when `-a` or `-M` picked a different one; a `-J` capture
//! whose last line is wrapped ends without a newline; a line whose hyperlink
//! collection came out empty contributes nothing at all, newline included;
//! and `-p` reads the client's flags without checking that there is a client.
//!
//! Coverage exemptions: none.

use crate::arguments::{args_get, args_has, args_strtonum_and_expand};
use crate::cmd::queue::{cmdq_error, cmdq_get_client, cmdq_get_target};
use crate::cmd::{cmd_get_args, cmd_get_entry};
use crate::control::control_write;
use crate::file::{file_can_print, file_print, file_print_buffer};
use crate::fmt_args;
use crate::grid::hyperlinks_get;
use crate::grid::{
    grid_clear_history, grid_default_cell, grid_get_cell, grid_peek_line, grid_string_cells,
};
use crate::input::{ictx_mut, input_pending};
use crate::paste::paste_set;
use crate::screen::{
    screen_grid_mut, screen_grid_ptr, screen_reset_hyperlinks, screen_saved_grid_ptr,
};
pub use crate::types::*;
use crate::window::window_pane_current_mode;
use crate::window::window_pane_reset_mode_all;
use ::core::ffi::{CStr, c_char, c_int, c_longlong};
use ::core::ptr::null;
pub const SHRT_MAX: ::core::ffi::c_int = __SHRT_MAX__;
pub const INT_MIN: ::core::ffi::c_int = -__INT_MAX__ - 1 as ::core::ffi::c_int;
pub const GRID_LINE_WRAPPED: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const GRID_LINE_EXTENDED: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const GRID_LINE_DEAD: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const GRID_LINE_START_PROMPT: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const GRID_LINE_START_OUTPUT: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const GRID_LINE_HYPERLINK: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const GRID_STRING_WITH_SEQUENCES: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const GRID_STRING_ESCAPE_SEQUENCES: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const GRID_STRING_TRIM_SPACES: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const GRID_STRING_EMPTY_CELLS: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const CMD_FIND_PANE: cmd_find_type = 0;
pub const CMD_RETURN_NORMAL: cmd_retval = 0;
pub const CMD_RETURN_ERROR: cmd_retval = -1;
pub const CMD_AFTERHOOK: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const CLIENT_CONTROL: ::core::ffi::c_int = 0x2000 as ::core::ffi::c_int;
pub(crate) static cmd_capture_pane_entry: cmd_entry = cmd_entry {
    name: c"capture-pane",
    alias: Some(c"capturep"),
    args: args_parse_t {
        template: c"ab:CeE:FHJLMNpPqS:Tt:",
        lower: 0,
        upper: 0,
        cb: None,
    },
    usage: c"[-aCeFHJLMNpPqT] [-b buffer-name] [-E end-line] [-S start-line] [-t target-pane]",
    source: cmd_entry_flag {
        flag: 0,
        type_0: CMD_FIND_PANE,
        flags: 0,
    },
    target: cmd_entry_flag {
        flag: b't' as ::core::ffi::c_char,
        type_0: CMD_FIND_PANE,
        flags: 0,
    },
    flags: CMD_AFTERHOOK,
    exec: cmd_capture_pane_exec,
};
pub(crate) static cmd_clear_history_entry: cmd_entry = cmd_entry {
    name: c"clear-history",
    alias: Some(c"clearhist"),
    args: args_parse_t {
        template: c"Ht:",
        lower: 0,
        upper: 0,
        cb: None,
    },
    usage: c"[-H] [-t target-pane]",
    source: cmd_entry_flag {
        flag: 0,
        type_0: CMD_FIND_PANE,
        flags: 0,
    },
    target: cmd_entry_flag {
        flag: b't' as ::core::ffi::c_char,
        type_0: CMD_FIND_PANE,
        flags: 0,
    },
    flags: CMD_AFTERHOOK,
    exec: cmd_capture_pane_exec,
};

/// What the pane's input parser has read but not yet acted on, which is what
/// `-P` captures. `-C` writes anything that is not a printable byte, and the
/// backslash itself, as a three-digit octal escape.
///
/// The byte test is over a signed `char`, as the C's was: a byte over 0x7f is
/// negative there, so it is never printable and always escapes.
unsafe fn cmd_capture_pane_pending(args: &args, wp: *mut window_pane) -> Vec<u8> {
    unsafe {
        let pending = input_pending(ictx_mut(&mut (*wp).ictx));
        if pending.is_null() {
            return Vec::new();
        }
        let line = (*pending).as_slice();
        let linelen = line.len();
        if linelen == 0 {
            return Vec::new();
        }
        if args_has(args, b'C') == 0 {
            return line.to_vec();
        }
        let mut buf = Vec::with_capacity(linelen);
        for &byte in line {
            if byte as c_char >= b' ' as c_char && byte != b'\\' {
                buf.push(byte);
            } else {
                buf.push(b'\\');
                buf.push(b'0' + (byte >> 6));
                buf.push(b'0' + ((byte >> 3) & 7));
                buf.push(b'0' + (byte & 7));
            }
        }
        buf
    }
}

/// The hyperlink URIs line `py` carries that `links` has not seen yet, joined
/// with spaces, which is what `-H` captures in place of the line's text. Each
/// URI is listed once over the whole capture, so `links` carries the link ids
/// already written from line to line and grows to at most one screen width of
/// them; the line that would take it past that stops there.
unsafe fn cmd_capture_pane_hyperlinks(
    gd: *mut grid,
    s: *mut screen,
    py: u_int,
    links: &mut Vec<u_int>,
) -> Vec<u8> {
    unsafe {
        let mut line = Vec::new();
        let hyperlinks = (*s).hyperlinks_ptr();
        let Some(gl) = grid_peek_line(&*gd, py) else {
            return line;
        };
        if hyperlinks.is_null() || gl.flags & GRID_LINE_HYPERLINK == 0 {
            return line;
        }
        let cellused = gl.cellused;
        let mut gc = grid_cell::default();
        for i in 0..cellused {
            gc = grid_get_cell(&*gd, i, py);
            if gc.link == 0 || links.contains(&gc.link) {
                continue;
            }
            let Some((uri, _, _)) = hyperlinks_get(&*hyperlinks, gc.link) else {
                continue;
            };
            if links.len() as u_int == (*gd).sx {
                break;
            }
            links.push(gc.link);
            if !line.is_empty() {
                line.push(b' ');
            }
            line.extend_from_slice(uri.to_bytes());
        }
        line
    }
}

/// The line number the flag `flag` names, as a row of `gd` counted from its
/// first history line. A lone `-` means `dash`, a value that cannot be read
/// means `fallback`, a negative value counts back from the top of the screen
/// and stops at the first history line, and anything below the screen is
/// pulled back to its last row.
///
/// The negation is a wrapping one: the accepted range reaches down to
/// `INT_MIN`, whose negation is itself, and the C read that as a count larger
/// than any history.
unsafe fn cmd_capture_pane_edge(
    args: &args,
    item: *mut cmdq_item,
    flag: u_char,
    gd: *mut grid,
    dash: u_int,
    fallback: u_int,
) -> u_int {
    unsafe {
        let value = args_get(args, flag);
        if !value.is_null() && CStr::from_ptr(value) == c"-" {
            return dash;
        }
        let mut cause = None;
        let n = args_strtonum_and_expand(
            args,
            flag,
            INT_MIN as c_longlong,
            SHRT_MAX as c_longlong,
            item,
            &mut cause,
        ) as c_int;
        let mut edge = if cause.is_some() {
            fallback
        } else if n < 0 && n.wrapping_neg() as u_int > (*gd).hsize {
            0
        } else {
            (*gd).hsize.wrapping_add(n as u_int)
        };
        let last = (*gd).hsize.wrapping_add((*gd).sy).wrapping_sub(1);
        if edge > last {
            edge = last;
        }
        edge
    }
}

/// The captured text of the range of grid lines the arguments name, or
/// nothing at all when the capture was refused and the reason already
/// reported.
unsafe fn cmd_capture_pane_history(
    args: &args,
    item: *mut cmdq_item,
    wp: *mut window_pane,
) -> Option<Vec<u8>> {
    unsafe {
        let base_grid = screen_grid_ptr(&mut (*wp).base);
        let sx = (*base_grid).sx;
        let (s, gd) = if args_has(args, b'a') != 0 {
            let gd = screen_saved_grid_ptr(&mut (*wp).base);
            if (*wp).base.saved_grid.is_none() {
                if args_has(args, b'q') == 0 {
                    cmdq_error(item, c"no alternate screen".as_ptr(), fmt_args![]);
                    return None;
                }
                return Some(Vec::new());
            }
            (&raw mut (*wp).base, gd)
        } else if args_has(args, b'M') != 0 {
            let wme = window_pane_current_mode(wp);
            if !wme.is_null()
                && let Some(s) = (*wme).mode().get_screen(wme)
            {
                (s, screen_grid_ptr(&mut *s))
            } else {
                (&raw mut (*wp).base, base_grid)
            }
        } else {
            (&raw mut (*wp).base, base_grid)
        };

        let last = (*gd).hsize.wrapping_add((*gd).sy).wrapping_sub(1);
        let mut top = cmd_capture_pane_edge(args, item, b'S', gd, 0, (*gd).hsize);
        let mut bottom = cmd_capture_pane_edge(args, item, b'E', gd, last, last);
        if bottom < top {
            ::core::mem::swap(&mut top, &mut bottom);
        }

        let join_lines = args_has(args, b'J') != 0;
        let mut flags: c_int = 0;
        if args_has(args, b'e') != 0 {
            flags |= GRID_STRING_WITH_SEQUENCES;
        }
        if args_has(args, b'C') != 0 {
            flags |= GRID_STRING_ESCAPE_SEQUENCES;
        }
        if !join_lines && args_has(args, b'T') == 0 {
            flags |= GRID_STRING_EMPTY_CELLS;
        }
        if !join_lines && args_has(args, b'N') == 0 {
            flags |= GRID_STRING_TRIM_SPACES;
        }
        let number_lines = args_has(args, b'L') != 0;
        let show_flags = args_has(args, b'F') != 0;
        let hyperlinks = args_has(args, b'H') != 0;

        let mut links: Vec<u_int> = Vec::new();
        if hyperlinks {
            links.reserve((*gd).sx as usize);
        }
        let mut lastgc: grid_cell = grid_default_cell;
        let mut buf: Vec<u8> = Vec::new();
        for i in top..=bottom {
            let line = if hyperlinks {
                let line = cmd_capture_pane_hyperlinks(gd, s, i, &mut links);
                if line.is_empty() {
                    continue;
                }
                line
            } else {
                grid_string_cells(&*gd, 0, i, sx, Some(&mut lastgc), flags, s).into_bytes()
            };
            if number_lines {
                let n = if i >= (*gd).hsize {
                    i.wrapping_sub((*gd).hsize) as c_int
                } else {
                    i as c_int - (*gd).hsize as c_int
                };
                buf.extend_from_slice(::std::format!("{n} ").as_bytes());
            }
            if show_flags {
                let gl = grid_peek_line(&*gd, i);
                let mut letters: Vec<u8> = Vec::new();
                for (bit, letter) in [
                    (GRID_LINE_DEAD, b'D'),
                    (GRID_LINE_HYPERLINK, b'H'),
                    (GRID_LINE_START_OUTPUT, b'O'),
                    (GRID_LINE_START_PROMPT, b'P'),
                    (GRID_LINE_WRAPPED, b'W'),
                    (GRID_LINE_EXTENDED, b'X'),
                ] {
                    if gl.is_some_and(|gl| gl.flags & bit != 0) {
                        letters.push(letter);
                    }
                }
                if letters.is_empty() {
                    letters.push(b'-');
                }
                letters.push(b' ');
                buf.extend_from_slice(&letters);
            }
            buf.extend_from_slice(&line);
            let gl = grid_peek_line(&*gd, i);
            if !join_lines || !gl.is_some_and(|gl| gl.flags & GRID_LINE_WRAPPED != 0) {
                buf.push(b'\n');
            }
        }
        Some(buf)
    }
}

unsafe fn cmd_capture_pane_exec(self_0: &cmd, item: *mut cmdq_item) -> cmd_retval {
    unsafe {
        let args = cmd_get_args(self_0);
        let c = cmdq_get_client(&*item);
        let wp = (*cmdq_get_target(item)).pane();
        if ::core::ptr::eq(cmd_get_entry(self_0), &cmd_clear_history_entry) {
            window_pane_reset_mode_all(wp);
            grid_clear_history(screen_grid_mut(&mut (*wp).base));
            if args_has(args, b'H') != 0 {
                screen_reset_hyperlinks((*wp).screen());
            }
            return CMD_RETURN_NORMAL;
        }
        let bytes = if args_has(args, b'P') != 0 && args_has(args, b'H') == 0 {
            cmd_capture_pane_pending(args, wp)
        } else {
            match cmd_capture_pane_history(args, item, wp) {
                Some(bytes) => bytes,
                None => return CMD_RETURN_ERROR,
            }
        };
        if args_has(args, b'p') != 0 {
            let mut len = bytes.len() as size_t;
            if len > 0 && bytes[len.wrapping_sub(1) as usize] == b'\n' {
                len = len.wrapping_sub(1);
            }
            let mut buf = bytes;
            buf.push(b'\0');
            if (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
                control_write(
                    c,
                    c"%.*s".as_ptr(),
                    fmt_args![len as c_int, buf.as_ptr() as *const c_char],
                );
            } else {
                if file_can_print(c) == 0 {
                    cmdq_error(item, c"can't write to client".as_ptr(), fmt_args![]);
                    return CMD_RETURN_ERROR;
                }
                file_print_buffer(c, &buf[..len as usize]);
                file_print(c, c"\n".as_ptr(), fmt_args![]);
            }
        } else {
            let bufname = if args_has(args, b'b') != 0 {
                args_get(args, b'b')
            } else {
                null()
            };
            if let Err(cause) = paste_set(bytes, bufname) {
                cmdq_error(item, c"%s".as_ptr(), fmt_args![cause.as_ptr()]);
                return CMD_RETURN_ERROR;
            }
        }
        CMD_RETURN_NORMAL
    }
}
pub const __SHRT_MAX__: ::core::ffi::c_int = 32767 as ::core::ffi::c_int;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
