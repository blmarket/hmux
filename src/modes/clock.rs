use std::ffi::CStr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::ffi::{localtime, strftime, strlcat, time};
use crate::fmt_args;
use crate::grid::grid_default_cell;
use crate::options::options_get_number;
use crate::reactor::Timer;
use crate::screen::{screen_free, screen_grid_ptr, screen_init, screen_resize};
use crate::screen::{
    screen_write_clearscreen, screen_write_cursormove, screen_write_putc, screen_write_puts,
    screen_write_start, screen_write_stop,
};
pub use crate::types::*;
use crate::window::window_pane_reset_mode;
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
#[repr(C)]
pub struct window_clock_mode_data {
    pub screen: screen,
    pub tim: time_t,
    pub timer: TimerHandle,
}
pub const MODE_CURSOR: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const GRID_FLAG_NOPALETTE: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const PANE_REDRAW: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
/// One 5x5 glyph, read off the picture its rows draw: a `#` lights the cell
/// and anything else leaves it blank.
const fn drawn(rows: [&str; 5]) -> [[bool; 5]; 5] {
    let mut cells = [[false; 5]; 5];
    let mut j = 0;
    while j < 5 {
        let row = rows[j].as_bytes();
        let mut i = 0;
        while i < 5 {
            cells[j][i] = row[i] == b'#';
            i += 1;
        }
        j += 1;
    }
    cells
}

/// The glyph for each character the clock face can draw, in the order
/// [`window_clock_glyph`] looks them up: the ten digits `0` to `9`, then
/// `:`, `A`, `P` and `M`.
#[rustfmt::skip]
pub static window_clock_table: [[[bool; 5]; 5]; 14] = [
    drawn([
        "#####",
        "#...#",
        "#...#",
        "#...#",
        "#####",
    ]),
    drawn([
        "....#",
        "....#",
        "....#",
        "....#",
        "....#",
    ]),
    drawn([
        "#####",
        "....#",
        "#####",
        "#....",
        "#####",
    ]),
    drawn([
        "#####",
        "....#",
        "#####",
        "....#",
        "#####",
    ]),
    drawn([
        "#...#",
        "#...#",
        "#####",
        "....#",
        "....#",
    ]),
    drawn([
        "#####",
        "#....",
        "#####",
        "....#",
        "#####",
    ]),
    drawn([
        "#####",
        "#....",
        "#####",
        "#...#",
        "#####",
    ]),
    drawn([
        "#####",
        "....#",
        "....#",
        "....#",
        "....#",
    ]),
    drawn([
        "#####",
        "#...#",
        "#####",
        "#...#",
        "#####",
    ]),
    drawn([
        "#####",
        "#...#",
        "#####",
        "....#",
        "#####",
    ]),
    drawn([
        ".....",
        "..#..",
        ".....",
        "..#..",
        ".....",
    ]),
    drawn([
        "#####",
        "#...#",
        "#####",
        "#...#",
        "#...#",
    ]),
    drawn([
        "#####",
        "#...#",
        "#####",
        "#....",
        "#....",
    ]),
    drawn([
        "#...#",
        "##.##",
        "#.#.#",
        "#...#",
        "#...#",
    ]),
];
/// Now, as a span since the epoch; a clock set before the epoch reads as zero.
fn window_clock_now() -> Duration {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
}
unsafe fn window_clock_start_timer(wme: *mut window_mode_entry) {
    unsafe {
        let data = (*wme).state.clock();
        let delay = 1_000_000 - window_clock_now().subsec_micros() as __suseconds_t;
        (*data).timer.arm(timeval {
            tv_sec: delay / 1_000_000,
            tv_usec: delay % 1_000_000,
        });
    }
}
unsafe fn window_clock_timer_callback(wme: *mut window_mode_entry) {
    unsafe {
        let wp = (*wme).wp;
        let data = (*wme).state.clock();
        (*data).timer.disarm();
        let t = window_clock_now().as_secs() as time_t;
        if t % 60 != (*data).tim % 60 {
            (*data).tim = t;
            window_clock_draw_screen(wme);
            (*wp).flags |= PANE_REDRAW;
        }
        window_clock_start_timer(wme);
    }
}
pub(crate) unsafe fn window_clock_init(
    wme: &mut window_mode_entry,
    _fs: *mut cmd_find_state,
    _args: Option<&args>,
) -> *mut screen {
    unsafe {
        let mut wp: *mut window_pane = wme.wp;
        let mut s: *mut screen = ::core::ptr::null_mut::<screen>();
        let mut data = Box::new(window_clock_mode_data {
            screen: screen::default(),
            tim: 0,
            timer: TimerHandle::ZERO,
        });
        let data_ptr: *mut window_clock_mode_data = &mut *data;
        let wme: *mut window_mode_entry = wme;
        (*wme).state = WindowModeState::Clock(data);
        let data = data_ptr;
        (*data).tim = window_clock_now().as_secs() as time_t;
        (*data)
            .timer
            .set_callback(move || window_clock_timer_callback(wme));
        window_clock_start_timer(wme);
        s = &raw mut (*data).screen;
        screen_init(
            &mut *s,
            (*screen_grid_ptr(&mut (*wp).base)).sx,
            (*screen_grid_ptr(&mut (*wp).base)).sy,
            0 as u_int,
        );
        (*s).mode &= !MODE_CURSOR;
        window_clock_draw_screen(wme);
        s
    }
}
pub(crate) unsafe fn window_clock_free(wme: &mut window_mode_entry) {
    unsafe {
        let mut data: *mut window_clock_mode_data = wme.state.clock();
        (*data).timer.disarm();
        screen_free(&mut (*data).screen);
    }
}
pub(crate) unsafe fn window_clock_resize(
    wme: &mut window_mode_entry,
    mut sx: u_int,
    mut sy: u_int,
) {
    unsafe {
        let mut data: *mut window_clock_mode_data = wme.state.clock();
        let mut s: *mut screen = &raw mut (*data).screen;
        screen_resize(&mut *s, sx, sy, 0 as ::core::ffi::c_int);
        window_clock_draw_screen(wme);
    }
}
pub(crate) unsafe fn window_clock_key(
    wme: &mut window_mode_entry,
    _c: *mut client,
    _s: *mut session,
    _wl: *mut winlink,
    _key: key_code,
    _m: *mut mouse_event,
) {
    unsafe {
        window_pane_reset_mode(wme.wp);
    }
}
/// The 5x5 glyph the clock face draws a character with: one per digit, and
/// one each for the separator and the three letters an AM/PM time ends with.
/// Anything else is drawn as a blank column.
fn window_clock_glyph(ch: u8) -> Option<&'static [[bool; 5]; 5]> {
    let idx = match ch {
        b'0'..=b'9' => (ch - b'0') as usize,
        b':' => 10,
        b'A' => 11,
        b'P' => 12,
        b'M' => 13,
        _ => return None,
    };
    Some(&window_clock_table[idx])
}
unsafe fn window_clock_draw_screen(mut wme: *mut window_mode_entry) {
    unsafe {
        let mut wp: *mut window_pane = (*wme).wp;
        let mut data: *mut window_clock_mode_data = (*wme).state.clock();
        let mut ctx = screen_write_ctx::default();
        let mut colour: ::core::ffi::c_int = 0;
        let mut style: ::core::ffi::c_int = 0;
        let mut s: *mut screen = &raw mut (*data).screen;
        let mut gc = grid_default_cell;
        let mut tim: [::core::ffi::c_char; 64] = [0; 64];
        let mut t: time_t = 0;
        let mut tm: *mut tm = ::core::ptr::null_mut::<tm>();
        let mut x: u_int = 0;
        let mut y: u_int = 0;
        colour = options_get_number((*(*wp).window).options_ptr(), c"clock-mode-colour".as_ptr())
            as ::core::ffi::c_int;
        style = options_get_number((*(*wp).window).options_ptr(), c"clock-mode-style".as_ptr())
            as ::core::ffi::c_int;
        screen_write_start(&mut ctx, &mut *s);
        t = time(::core::ptr::null_mut::<time_t>());
        tm = localtime(&raw mut t);
        if style == 0 as ::core::ffi::c_int || style == 2 as ::core::ffi::c_int {
            if style == 2 as ::core::ffi::c_int {
                strftime(
                    &raw mut tim as *mut ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 64]>() as size_t,
                    c"%l:%M:%S ".as_ptr(),
                    localtime(&raw mut t),
                );
            } else {
                strftime(
                    &raw mut tim as *mut ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 64]>() as size_t,
                    c"%l:%M ".as_ptr(),
                    localtime(&raw mut t),
                );
            }
            if (*tm).tm_hour >= 12 as ::core::ffi::c_int {
                strlcat(
                    &raw mut tim as *mut ::core::ffi::c_char,
                    c"PM".as_ptr(),
                    ::core::mem::size_of::<[::core::ffi::c_char; 64]>() as size_t,
                );
            } else {
                strlcat(
                    &raw mut tim as *mut ::core::ffi::c_char,
                    c"AM".as_ptr(),
                    ::core::mem::size_of::<[::core::ffi::c_char; 64]>() as size_t,
                );
            }
        } else if style == 3 as ::core::ffi::c_int {
            strftime(
                &raw mut tim as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 64]>() as size_t,
                c"%H:%M:%S".as_ptr(),
                tm,
            );
        } else {
            strftime(
                &raw mut tim as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 64]>() as size_t,
                c"%H:%M".as_ptr(),
                tm,
            );
        }
        let digits = CStr::from_ptr(&raw const tim as *const ::core::ffi::c_char).to_bytes();
        screen_write_clearscreen(&mut ctx, 8 as u_int);
        if ((*screen_grid_ptr(&mut *s)).sx as size_t) < (6 as size_t).wrapping_mul(digits.len())
            || (*screen_grid_ptr(&mut *s)).sy < 6 as u_int
        {
            if (*screen_grid_ptr(&mut *s)).sx as size_t >= digits.len()
                && (*screen_grid_ptr(&mut *s)).sy != 0 as u_int
            {
                x = ((*screen_grid_ptr(&mut *s)).sx.wrapping_div(2 as u_int) as size_t)
                    .wrapping_sub(digits.len().wrapping_div(2 as size_t))
                    as u_int;
                y = (*screen_grid_ptr(&mut *s)).sy.wrapping_div(2 as u_int);
                screen_write_cursormove(
                    &mut ctx,
                    x as ::core::ffi::c_int,
                    y as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                );
                gc = grid_default_cell;
                gc.flags = (gc.flags as ::core::ffi::c_int | GRID_FLAG_NOPALETTE) as u_char;
                gc.fg = colour;
                screen_write_puts(
                    &mut ctx,
                    &mut gc,
                    c"%s".as_ptr(),
                    fmt_args![digits.as_ptr() as *const ::core::ffi::c_char],
                );
            }
            screen_write_stop(&mut ctx);
            return;
        }
        x = ((*screen_grid_ptr(&mut *s)).sx.wrapping_div(2 as u_int) as size_t)
            .wrapping_sub((3 as size_t).wrapping_mul(digits.len())) as u_int;
        y = (*screen_grid_ptr(&mut *s))
            .sy
            .wrapping_div(2 as u_int)
            .wrapping_sub(3 as u_int);
        gc = grid_default_cell;
        gc.flags = (gc.flags as ::core::ffi::c_int | GRID_FLAG_NOPALETTE) as u_char;
        gc.bg = colour;
        gc.fg = colour;
        for &ch in digits {
            if let Some(glyph) = window_clock_glyph(ch) {
                for j in 0..5 as u_int {
                    for i in 0..5 as u_int {
                        screen_write_cursormove(
                            &mut ctx,
                            x.wrapping_add(i) as ::core::ffi::c_int,
                            y.wrapping_add(j) as ::core::ffi::c_int,
                            0 as ::core::ffi::c_int,
                        );
                        if glyph[j as usize][i as usize] {
                            screen_write_putc(&mut ctx, &gc, '#' as i32 as u_char);
                        }
                    }
                }
            }
            x = x.wrapping_add(6 as u_int);
        }
        screen_write_stop(&mut ctx);
    }
}
