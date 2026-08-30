//! `display-panes`: puts each pane's number over the panes of a client's
//! current window and runs a command against whichever one is chosen.
//!
//! Exec prepares the command template — `select-pane -t "%%%"` unless the
//! command line gave another — parks it with the item that is waiting, if any,
//! in a [`cmd_display_panes_data`] owned by the overlay, and hands that to the
//! client as an overlay with a delay, a draw callback, a free callback and,
//! unless `-N` was given, a key callback. `-b` gives up the wait, which is what
//! decides whether the chosen pane's command is spliced in behind the asking
//! item or appended to the client's own queue.
//!
//! The drawing puts one pane's number in the middle of that pane, in
//! `display-panes-colour` or, for the active pane, `display-panes-active-colour`.
//! A pane with room for it gets the number as clock digits six columns apart,
//! its size in the top right corner and, past the ninth pane, the letter that
//! selects it under the digits; a pane too small for the digits gets the number
//! written out on one line instead. Every cell goes through
//! [`cmd_display_panes_put`], which asks `screen_redraw` which parts of the run
//! are not hidden behind a floating pane and writes only those.
//!
//! Quirks kept:
//!
//! * The one-line fallback writes the number with the number's length, the
//!   separating space and the letter's length added together, but it writes
//!   them out of the number's own buffer — so the bytes past the number go to
//!   the terminal too. The buffer starts zeroed here, where the C reads
//!   uninitialised stack.
//! * Only two of the four clipping arms clip. A pane that starts inside the
//!   redraw context and runs off its right or its bottom keeps its own width
//!   and height, so the drawing is centred on the whole pane rather than on the
//!   visible part of it.
//! * The pane's number is measured before the colours are read, so a pane with
//!   fewer columns than its number has digits is dropped without even putting
//!   the cursor back.
//! * A visible range that starts later than the run it came from is still
//!   written from the front of the buffer, so a run partly hidden behind a
//!   floating pane repeats its first bytes at the later position.
//! * The key callback tests the whole key against `0` to `9` before it masks
//!   the modifiers off, so a modified digit falls through to the letter test
//!   and is refused, while a modified letter is refused outright.
//!
//! One branch the C spells out is gone. The digit loop skips anything in the
//! number that is not `0` to `9`, and the number is what `xsnprintf` wrote for
//! a `%u`, so every byte in it is a decimal digit and the skip never runs.
//!
//! Coverage exemptions: `fatalx` for a pane that is not in its own window's
//! pane list, and the `fatalx` in [`cmd_display_panes_fill`] for a string too long for the
//! sixteen-byte buffer the C gives it.

use crate::arguments::{args_has, args_make_commands, args_make_commands_prepare, args_strtonum};
use crate::cmd::cmd_get_args;
use crate::cmd::queue::{
    CmdqItemWeak, cmdq_append, cmdq_continue, cmdq_error, cmdq_get_command, cmdq_get_error,
    cmdq_get_state_ref, cmdq_get_target_client, cmdq_insert_after, cmdq_item_weak_from_ptr,
};
use crate::fmt_args;
use crate::grid::grid_default_cell;
use crate::log::{fatalx, log_debug};
use crate::modes::window_clock_table;
use crate::options::options_get_number;
use crate::screen::screen_redraw_get_visible_ranges;
use crate::server::server_client_set_overlay;
use crate::session::{session_get_curw, session_options};
use crate::tty::{tty_attributes, tty_cursor, tty_putn};
pub use crate::types::*;
use crate::window::window_get_active;
use crate::window::{
    window_pane_at_index, window_pane_index, window_pane_visible, window_panes_first,
    window_panes_next, window_unzoom,
};
use crate::xmalloc::xasprintf;
use ::core::ffi::{c_char, c_int, c_longlong, c_ulonglong};
use ::core::ptr::null_mut;
use ::std::ffi::CString;
pub const MSG_READ_CANCEL: msgtype = 307;
pub const MSG_WRITE_CLOSE: msgtype = 306;
pub const MSG_WRITE_READY: msgtype = 305;
pub const MSG_WRITE: msgtype = 304;
pub const MSG_WRITE_OPEN: msgtype = 303;
pub const MSG_READ_DONE: msgtype = 302;
pub const MSG_READ: msgtype = 301;
pub const MSG_READ_OPEN: msgtype = 300;
pub const MSG_FLAGS: msgtype = 218;
pub const MSG_EXEC: msgtype = 217;
pub const MSG_WAKEUP: msgtype = 216;
pub const MSG_UNLOCK: msgtype = 215;
pub const MSG_SUSPEND: msgtype = 214;
pub const MSG_OLDSTDOUT: msgtype = 213;
pub const MSG_OLDSTDIN: msgtype = 212;
pub const MSG_OLDSTDERR: msgtype = 211;
pub const MSG_SHUTDOWN: msgtype = 210;
pub const MSG_SHELL: msgtype = 209;
pub const MSG_RESIZE: msgtype = 208;
pub const MSG_READY: msgtype = 207;
pub const MSG_LOCK: msgtype = 206;
pub const MSG_EXITING: msgtype = 205;
pub const MSG_EXITED: msgtype = 204;
pub const MSG_EXIT: msgtype = 203;
pub const MSG_DETACHKILL: msgtype = 202;
pub const MSG_DETACH: msgtype = 201;
pub const MSG_COMMAND: msgtype = 200;
pub const MSG_IDENTIFY_TERMINFO: msgtype = 112;
pub const MSG_IDENTIFY_LONGFLAGS: msgtype = 111;
pub const MSG_IDENTIFY_STDOUT: msgtype = 110;
pub const MSG_IDENTIFY_FEATURES: msgtype = 109;
pub const MSG_IDENTIFY_CWD: msgtype = 108;
pub const MSG_IDENTIFY_CLIENTPID: msgtype = 107;
pub const MSG_IDENTIFY_DONE: msgtype = 106;
pub const MSG_IDENTIFY_ENVIRON: msgtype = 105;
pub const MSG_IDENTIFY_STDIN: msgtype = 104;
pub const MSG_IDENTIFY_OLDCWD: msgtype = 103;
pub const MSG_IDENTIFY_TTYNAME: msgtype = 102;
pub const MSG_IDENTIFY_TERM: msgtype = 101;
pub const MSG_IDENTIFY_FLAGS: msgtype = 100;
pub const MSG_VERSION: msgtype = 12;
pub const PANE_LINES_SPACES: pane_lines = 5;
pub const PANE_LINES_NUMBER: pane_lines = 4;
pub const PANE_LINES_SIMPLE: pane_lines = 3;
pub const PANE_LINES_HEAVY: pane_lines = 2;
pub const PANE_LINES_DOUBLE: pane_lines = 1;
pub const PANE_LINES_SINGLE: pane_lines = 0;
pub const PROGRESS_BAR_PAUSED: progress_bar_state = 4;
pub const PROGRESS_BAR_INDETERMINATE: progress_bar_state = 3;
pub const PROGRESS_BAR_ERROR: progress_bar_state = 2;
pub const PROGRESS_BAR_NORMAL: progress_bar_state = 1;
pub const PROGRESS_BAR_HIDDEN: progress_bar_state = 0;
pub const SCREEN_CURSOR_BAR: screen_cursor_style = 3;
pub const SCREEN_CURSOR_UNDERLINE: screen_cursor_style = 2;
pub const SCREEN_CURSOR_BLOCK: screen_cursor_style = 1;
pub const SCREEN_CURSOR_DEFAULT: screen_cursor_style = 0;
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
pub const THEME_DARK: client_theme = 2;
pub const THEME_LIGHT: client_theme = 1;
pub const THEME_UNKNOWN: client_theme = 0;
pub const LAYOUT_WINDOWPANE: layout_type = 2;
pub const LAYOUT_TOPBOTTOM: layout_type = 1;
pub const LAYOUT_LEFTRIGHT: layout_type = 0;
pub const PROMPT_TYPE_INVALID: prompt_type = 255;
pub const PROMPT_TYPE_WINDOW_TARGET: prompt_type = 3;
pub const PROMPT_TYPE_TARGET: prompt_type = 2;
pub const PROMPT_TYPE_SEARCH: prompt_type = 1;
pub const PROMPT_TYPE_COMMAND: prompt_type = 0;
pub const PROMPT_COMMAND: client_prompt_mode = 1;
pub const PROMPT_ENTRY: client_prompt_mode = 0;
pub const CLIENT_EXIT_DETACH: client_exit_type = 2;
pub const CLIENT_EXIT_SHUTDOWN: client_exit_type = 1;
pub const CLIENT_EXIT_RETURN: client_exit_type = 0;
pub const ARGS_PARSE_COMMANDS: args_parse_type = 3;
pub const ARGS_PARSE_COMMANDS_OR_STRING: args_parse_type = 2;
pub const ARGS_PARSE_STRING: args_parse_type = 1;
pub const ARGS_PARSE_INVALID: args_parse_type = 0;
pub const CMD_FIND_SESSION: cmd_find_type = 2;
pub const CMD_FIND_WINDOW: cmd_find_type = 1;
pub const CMD_FIND_PANE: cmd_find_type = 0;
pub const CMD_RETURN_STOP: cmd_retval = 2;
pub const CMD_RETURN_WAIT: cmd_retval = 1;
pub const CMD_RETURN_NORMAL: cmd_retval = 0;
pub const CMD_RETURN_ERROR: cmd_retval = -1;

/// What the command leaves on the client while the numbers are up: the item
/// waiting for a pane to be chosen, if any, and the prepared template the
/// chosen pane's id is substituted into.
#[repr(C)]
pub struct cmd_display_panes_data {
    pub(crate) item: Option<CmdqItemWeak>,
    pub state: Option<Box<args_command_state>>,
}
pub const UINT_MAX: ::core::ffi::c_uint = (__INT_MAX__ as ::core::ffi::c_uint)
    .wrapping_mul(2 as ::core::ffi::c_uint)
    .wrapping_add(1 as ::core::ffi::c_uint);
pub const KEYC_MASK_MODIFIERS: ::core::ffi::c_ulonglong =
    0xff0000000000 as ::core::ffi::c_ulonglong;
pub const KEYC_MASK_KEY: ::core::ffi::c_ulonglong = 0xffffffffff as ::core::ffi::c_ulonglong;
pub const CMD_AFTERHOOK: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const CMD_CLIENT_TFLAG: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;

pub(crate) static cmd_display_panes_entry: cmd_entry = cmd_entry {
    name: c"display-panes",
    alias: Some(c"displayp"),
    args: args_parse_t {
        template: c"bd:Nt:",
        lower: 0,
        upper: 1,
        cb: Some(cmd_display_panes_args_parse),
    },
    usage: c"[-bN] [-d duration] [-t target-client] [template]",
    source: cmd_entry_flag {
        flag: 0,
        type_0: CMD_FIND_PANE,
        flags: 0,
    },
    target: cmd_entry_flag {
        flag: 0,
        type_0: CMD_FIND_PANE,
        flags: 0,
    },
    flags: CMD_AFTERHOOK | CMD_CLIENT_TFLAG,
    exec: cmd_display_panes_exec,
};

/// How the parser is told to read the optional template: as a command list if
/// it parses as one, and as a plain string otherwise.
unsafe fn cmd_display_panes_args_parse(
    _args: &args,
    _idx: u_int,
    _cause: &mut Option<CString>,
) -> args_parse_type {
    ARGS_PARSE_COMMANDS_OR_STRING
}

/// Writes `text` into one of the drawing's sixteen-byte buffers the way
/// `xsnprintf` fills one, answering how many bytes it wrote. The bytes past the
/// text keep the zeroes the buffer started with — the one-line drawing reads
/// them back out — and a string that does not fit aborts, as `xsnprintf` does.
fn cmd_display_panes_fill(buf: &mut [u8; 16], text: &str) -> size_t {
    if text.len() >= buf.len() {
        unsafe { fatalx(c"xsnprintf: overflow".as_ptr(), fmt_args![]) };
    }
    buf[..text.len()].copy_from_slice(text.as_bytes());
    text.len() as size_t
}

/// Where one of a pane's two dimensions sits inside the redraw context: the
/// offset from the context's own origin and how much of the pane to draw.
///
/// Only the arms for a pane that starts before the context clip anything. A
/// pane that starts inside it and runs off the far edge keeps its own size, so
/// what the drawing is centred on is the whole pane rather than the part of it
/// that can be seen.
fn cmd_display_panes_clip(
    off: c_int,
    size: u_int,
    ctx_off: c_int,
    ctx_size: u_int,
) -> (u_int, u_int) {
    if off >= ctx_off
        && (off as u_int).wrapping_add(size) <= (ctx_off as u_int).wrapping_add(ctx_size)
    {
        return (off.wrapping_sub(ctx_off) as u_int, size);
    }
    if off < ctx_off
        && (off as u_int).wrapping_add(size) > (ctx_off as u_int).wrapping_add(ctx_size)
    {
        return (0, ctx_size);
    }
    if off < ctx_off {
        return (0, size.wrapping_sub(ctx_off.wrapping_sub(off) as u_int));
    }
    let off = off.wrapping_sub(ctx_off) as u_int;
    (off, size.wrapping_sub(off))
}

/// Writes `buf` across the terminal starting at `cx`, `cy` in the redraw
/// context, one cell at a time and only where the pane is not hidden behind a
/// floating pane. Each visible part of the run is written from the front of
/// `buf` rather than from the byte that lines up with it, which is what the C
/// does.
unsafe fn cmd_display_panes_put(
    ctx: &mut screen_redraw_ctx,
    wp: *mut window_pane,
    cx: u_int,
    cy: u_int,
    buf: &[u8],
) {
    unsafe {
        let c = ctx.c;
        let tty = &raw mut (*c).tty;
        let mut ranges = visible_ranges::default();
        screen_redraw_get_visible_ranges(
            wp,
            (ctx.ox as u_int).wrapping_add(cx) as c_int,
            (ctx.oy as u_int).wrapping_add(cy) as c_int,
            buf.len() as u_int,
            &mut ranges,
        );
        for i in 0..ranges.used {
            let ri = ranges.ranges[i as usize];
            let mut j = ri.px;
            while j < ri.px.wrapping_add(ri.nx) {
                tty_cursor(tty, j.wrapping_sub(ctx.ox as u_int), cy);
                tty_putn(
                    tty,
                    buf.as_ptr().add(j.wrapping_sub(ri.px) as usize) as *const c_char,
                    1,
                    1,
                );
                j = j.wrapping_add(1);
            }
        }
    }
}

/// Draws one pane's number, and the size and letter that go with it, leaving
/// the cursor at the top left corner. A pane that does not overlap the redraw
/// context at all, and a pane with fewer columns than its number has digits,
/// are both left alone entirely.
unsafe fn cmd_display_panes_draw_pane(ctx: &mut screen_redraw_ctx, wp: *mut window_pane) {
    unsafe {
        let c = ctx.c;
        let tty = &raw mut (*c).tty;
        let oo = session_options((*c).session);
        let w = (*wp).window;

        if (*wp).xoff.wrapping_add((*wp).sx as c_int) <= ctx.ox
            || (*wp).xoff >= ctx.ox.wrapping_add(ctx.sx as c_int)
            || (*wp).yoff.wrapping_add((*wp).sy as c_int) <= ctx.oy
            || (*wp).yoff >= ctx.oy.wrapping_add(ctx.sy as c_int)
        {
            return;
        }
        let (xoff, sx) = cmd_display_panes_clip((*wp).xoff, (*wp).sx, ctx.ox, ctx.sx);
        let (mut yoff, sy) = cmd_display_panes_clip((*wp).yoff, (*wp).sy, ctx.oy, ctx.sy);
        if ctx.statustop != 0 {
            yoff = yoff.wrapping_add(ctx.statuslines);
        }
        let mut px = sx.wrapping_div(2);
        let mut py = sy.wrapping_div(2);

        let (found, pane) = window_pane_index(wp);
        if found != 0 {
            fatalx(c"index not found".as_ptr(), fmt_args![]);
        }
        let mut buf = [0u8; 16];
        let mut len = cmd_display_panes_fill(&mut buf, &pane.to_string());
        if (sx as size_t) < len {
            return;
        }

        let colour = options_get_number(oo, c"display-panes-colour".as_ptr()) as c_int;
        let active_colour =
            options_get_number(oo, c"display-panes-active-colour".as_ptr()) as c_int;
        let mut fgc = grid_default_cell;
        let mut bgc = grid_default_cell;
        if window_get_active(w) == wp {
            fgc.fg = active_colour;
            bgc.bg = active_colour;
        } else {
            fgc.fg = colour;
            bgc.bg = colour;
        }

        let mut rbuf = [0u8; 16];
        let rlen = cmd_display_panes_fill(&mut rbuf, &format!("{}x{}", (*wp).sx, (*wp).sy));
        let mut lbuf = [0u8; 16];
        let llen = match pane > 9 && pane < 35 {
            true => cmd_display_panes_fill(
                &mut lbuf,
                &(b'a'.wrapping_add(pane.wrapping_sub(10) as u8) as char).to_string(),
            ),
            false => 0,
        };

        if (sx as size_t) < len.wrapping_mul(6) || sy < 5 {
            tty_attributes(
                tty,
                &raw mut fgc,
                &raw const grid_default_cell,
                null_mut::<colour_palette>(),
                null_mut::<hyperlinks>(),
            );
            if sx as size_t >= len.wrapping_add(llen).wrapping_add(1) {
                len = len.wrapping_add(llen.wrapping_add(1));
                let mut cx =
                    (xoff.wrapping_add(px) as size_t).wrapping_sub(len.wrapping_div(2)) as u_int;
                let cy = yoff.wrapping_add(py);
                cmd_display_panes_put(&mut *ctx, wp, cx, cy, &buf[..len as usize]);
                cx = (cx as size_t).wrapping_add(len) as u_int;
                cmd_display_panes_put(&mut *ctx, wp, cx, cy, b" ");
                cx = cx.wrapping_add(1);
                cmd_display_panes_put(&mut *ctx, wp, cx, cy, &lbuf[..llen as usize]);
            } else {
                let cx =
                    (xoff.wrapping_add(px) as size_t).wrapping_sub(len.wrapping_div(2)) as u_int;
                let cy = yoff.wrapping_add(py);
                cmd_display_panes_put(&mut *ctx, wp, cx, cy, &buf[..len as usize]);
            }
            tty_cursor(tty, 0, 0);
            return;
        }

        px = (px as size_t).wrapping_sub(len.wrapping_mul(3)) as u_int;
        py = py.wrapping_sub(2);
        tty_attributes(
            tty,
            &raw mut bgc,
            &raw const grid_default_cell,
            null_mut::<colour_palette>(),
            null_mut::<hyperlinks>(),
        );
        for digit in &buf[..len as usize] {
            let idx = digit.wrapping_sub(b'0') as usize;
            for j in 0..5 {
                let mut i = px;
                while i < px.wrapping_add(5) {
                    if window_clock_table[idx][j as usize][i.wrapping_sub(px) as usize] != 0 {
                        cmd_display_panes_put(
                            &mut *ctx,
                            wp,
                            xoff.wrapping_add(i),
                            yoff.wrapping_add(py).wrapping_add(j),
                            b" ",
                        );
                    }
                    i = i.wrapping_add(1);
                }
            }
            px = px.wrapping_add(6);
        }

        if sy > 6 {
            tty_attributes(
                tty,
                &raw mut fgc,
                &raw const grid_default_cell,
                null_mut::<colour_palette>(),
                null_mut::<hyperlinks>(),
            );
            if rlen != 0 && sx as size_t >= rlen {
                let cx = (xoff.wrapping_add(sx) as size_t).wrapping_sub(rlen) as u_int;
                cmd_display_panes_put(&mut *ctx, wp, cx, yoff, &rbuf[..rlen as usize]);
            }
            if llen != 0 {
                let cx = (xoff.wrapping_add(sx.wrapping_div(2)) as size_t)
                    .wrapping_add(len.wrapping_mul(3))
                    .wrapping_sub(llen)
                    .wrapping_sub(1) as u_int;
                let cy = yoff.wrapping_add(py).wrapping_add(5);
                cmd_display_panes_put(&mut *ctx, wp, cx, cy, &lbuf[..llen as usize]);
            }
        }
        tty_cursor(tty, 0, 0);
    }
}

/// A window's panes, front to back, as `TAILQ_FOREACH` walks them.
unsafe fn cmd_display_panes_panes(w: *mut window) -> impl Iterator<Item = *mut window_pane> {
    let mut wp = unsafe { window_panes_first(w) };
    ::core::iter::from_fn(move || {
        let this = wp;
        if this.is_null() {
            return None;
        }
        wp = unsafe { window_panes_next(w, this) };
        Some(this)
    })
}

/// The overlay's draw callback: every visible pane of the client's current
/// window gets its number.
pub(crate) unsafe fn cmd_display_panes_draw(
    c: *mut client,
    _data: *mut cmd_display_panes_data,
    ctx: &mut screen_redraw_ctx,
) {
    unsafe {
        let w = (*session_get_curw((*c).session)).window();
        log_debug(
            c"%s: %s @%u".as_ptr(),
            fmt_args![
                c"cmd_display_panes_draw".as_ptr(),
                cstr_ptr(&(*c).name),
                (*w).id
            ],
        );
        for wp in cmd_display_panes_panes(w) {
            if window_pane_visible(wp) != 0 {
                cmd_display_panes_draw_pane(&mut *ctx, wp);
            }
        }
    }
}

/// Gives the private state back once the numbers are down, however they went,
/// and lets whatever was waiting on them carry on.
pub(crate) unsafe fn cmd_display_panes_free_box(
    _c: *mut client,
    mut data: Box<cmd_display_panes_data>,
) {
    unsafe {
        if let Some(item) = data.item.as_ref().and_then(CmdqItemWeak::upgrade) {
            cmdq_continue(item.as_ptr());
        }
        drop(data.state.take());
    }
}

/// The pane index a key press names, if it names one: a digit is that index,
/// and an unmodified letter continues from ten. The digits are tested against
/// the whole key before the modifiers are masked off, so a modified digit is
/// not one.
fn cmd_display_panes_index(key: key_code) -> Option<u_int> {
    if (b'0' as key_code..=b'9' as key_code).contains(&key) {
        return Some(key.wrapping_sub(b'0' as key_code) as u_int);
    }
    if key as c_ulonglong & KEYC_MASK_MODIFIERS != 0 {
        return None;
    }
    let key = (key as c_ulonglong & KEYC_MASK_KEY) as key_code;
    match (b'a' as key_code..=b'z' as key_code).contains(&key) {
        true => Some((10 as key_code).wrapping_add(key.wrapping_sub(b'a' as key_code)) as u_int),
        false => None,
    }
}

/// The overlay's key callback. A key naming a pane the window has runs the
/// template against that pane's id — spliced in behind the waiting item, or
/// appended to the client's own queue when nothing is waiting — and a template
/// that does not parse is reported instead. Anything else is refused, which is
/// what takes the numbers down.
pub(crate) unsafe fn cmd_display_panes_key(
    c: *mut client,
    data: *mut cmd_display_panes_data,
    event: *mut key_event,
) -> c_int {
    unsafe {
        let cdata = data;
        let item = (*cdata).item.as_ref().and_then(CmdqItemWeak::upgrade);
        let w = (*session_get_curw((*c).session)).window();

        let index = match cmd_display_panes_index((*event).key) {
            Some(index) => index,
            None => return -1,
        };
        let wp = window_pane_at_index(w, index);
        if wp.is_null() {
            return 1;
        }
        window_unzoom(w, 1);

        let expanded = xasprintf(c"%%%u".as_ptr(), fmt_args![(*wp).id]);
        let mut error = None;
        let cmdlist = args_make_commands(
            (*cdata).state.as_deref_mut().unwrap(),
            &[expanded],
            &mut error,
        );
        if let Some(error) = error.as_ref() {
            cmdq_append(c, cmdq_get_error(error.as_ptr()));
        } else if let Some(item) = &item {
            cmdq_insert_after(
                item.as_ptr(),
                cmdq_get_command(
                    cmdlist.as_ref().unwrap(),
                    Some(cmdq_get_state_ref(item.as_ptr())),
                ),
            );
        } else {
            cmdq_append(c, cmdq_get_command(cmdlist.as_ref().unwrap(), None));
        }
        1
    }
}

/// How long the numbers stay up: what `-d` says, or `display-panes-time`. An
/// unusable `-d` is the command's own error.
unsafe fn cmd_display_panes_delay(
    args: &args,
    item: *mut cmdq_item,
    s: *mut session,
) -> Option<u_int> {
    unsafe {
        if args_has(args, b'd') == 0 {
            return Some(
                options_get_number(session_options(s), c"display-panes-time".as_ptr()) as u_int,
            );
        }
        let mut cause = None;
        let delay = args_strtonum(args, b'd', 0, UINT_MAX as c_longlong, &mut cause) as u_int;
        if let Some(cause) = cause.as_ref() {
            cmdq_error(item, c"delay %s".as_ptr(), fmt_args![cause.as_ptr()]);
            return None;
        }
        Some(delay)
    }
}

unsafe fn cmd_display_panes_exec(self_0: &cmd, item: *mut cmdq_item) -> cmd_retval {
    unsafe {
        let args = cmd_get_args(self_0);
        let tc = cmdq_get_target_client(&*item);
        let wait = args_has(args, b'b') == 0;

        if (*tc).overlay().is_some() {
            return CMD_RETURN_NORMAL;
        }
        let delay = match cmd_display_panes_delay(args, item, (*tc).session) {
            Some(delay) => delay,
            None => return CMD_RETURN_ERROR,
        };

        let mut cdata = Box::new(cmd_display_panes_data {
            item: None,
            state: None,
        });
        if wait {
            cdata.item = cmdq_item_weak_from_ptr(item);
        }
        cdata.state = Some(args_make_commands_prepare(
            self_0,
            item,
            0,
            c"select-pane -t \"%%%\"".as_ptr(),
            wait as c_int,
            0,
        ));

        let overlay = Overlay::DisplayPanes {
            keys: args_has(args, b'N') == 0,
        };
        server_client_set_overlay(tc, delay, overlay, OverlayState::DisplayPanes(cdata));

        match wait {
            true => CMD_RETURN_WAIT,
            false => CMD_RETURN_NORMAL,
        }
    }
}

#[cfg(test)]
#[path = "../tests/test_cmd_display_panes.rs"]
mod tests;
