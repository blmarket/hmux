use crate::screen::Screen as _;
use crate::grid::Grid as _;
use crate::WindowPane;
use crate::args::RustArguments;
use crate::screen::Screen;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::ffi::time;
use crate::fmt_args;
use crate::grid::grid_default_cell;

use crate::reactor::Timer;
use crate::screen::{ScreenWriteCtx, screen_write_ctx_on_screen};
pub use crate::types::*;
#[repr(C)]
pub struct window_clock_mode_data {
    pub screen: RustScreen,
    pub tim: time_t,
    pub timer: TimerHandle,
}
pub use crate::consts::{GRID_FLAG_NOPALETTE, MODE_CURSOR, PANE_REDRAW};

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
fn window_clock_start_timer(data: &mut window_clock_mode_data) {
    let delay = 1_000_000 - window_clock_now().subsec_micros() as __suseconds_t;
    data.timer.arm(timeval {
        tv_sec: delay / 1_000_000,
        tv_usec: delay % 1_000_000,
    });
}
unsafe fn window_clock_timer_callback(mut pane: RustWindowPaneWeak) {
    unsafe {
        let Some(window) = pane.window() else { return };
        let options = window.options();
        let Some(wp) = pane.get_mut() else {
            return;
        };
        let Some(wme) = wp.find_mode_mut(WindowMode::Clock).map(|mode| mode.into_entry())
        else {
            return;
        };
        let Some(data) = wme.state.clock() else {
            return;
        };
        data.timer.disarm();
        let t = window_clock_now().as_secs() as time_t;
        let redraw = t % 60 != data.tim % 60;
        if redraw {
            data.tim = t;
            window_clock_draw_screen(data, &options);
        }
        window_clock_start_timer(data);
        if redraw {
            wp.request_redraw();
        }
    }
}
pub(crate) unsafe fn window_clock_init(
    wme: &mut window_mode_entry,
    pane: crate::window::RustWindowPaneWeak,
    _fs: Option<&cmd_find_state>,
    _args: Option<&RustArguments>,
) {
    unsafe {
        let (sx, sy) = pane
            .get()
            .expect("the initializing pane still exists")
            .base()
            .size();
        let options = pane
            .window()
            .expect("the clock pane has a window")
            .options();
        let mut data = Box::new(window_clock_mode_data {
            screen: RustScreen::new_with_server_options(sx, sy, 0),
            tim: window_clock_now().as_secs() as time_t,
            timer: TimerHandle::ZERO,
        });
        data.timer
            .set_callback(move || window_clock_timer_callback(pane.clone()));
        window_clock_start_timer(&mut data);
        data.screen.set_mode(data.screen.mode() & !MODE_CURSOR);
        window_clock_draw_screen(&mut data, &options);
        wme.state = WindowModeState::Clock(data);
    }
}
pub(crate) fn window_clock_free(wme: &mut window_mode_entry) {
    let data = wme.state.clock().expect("the mode holds its state");
    data.timer.disarm();
    data.screen = RustScreen::default();
}
pub(crate) unsafe fn window_clock_resize(wme: &mut window_mode_entry, sx: u_int, sy: u_int) {
    unsafe {
        let window = wme.pane_ref().and_then(|pane| pane.window());
        let data = wme.state.clock().expect("the mode holds its state");
        (&mut data.screen).resize(sx, sy, 0 as core::ffi::c_int);
        if let Some(window) = window {
            let options = window.options();
            window_clock_draw_screen(data, &options);
        }
    }
}

pub(crate) unsafe fn window_clock_key(mut pane: crate::window::RustWindowPaneWeak) {
    if let Some(pane) = unsafe { pane.get_mut() } {
        unsafe { (pane).reset_mode() };
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
unsafe fn window_clock_draw_screen(data: &mut window_clock_mode_data, options: &RustOptionsRef) {
    unsafe {
        let colour = options.number(c"clock-mode-colour") as core::ffi::c_int;
        let style = options.number(c"clock-mode-style") as core::ffi::c_int;
        let s: &mut RustScreen = &mut data.screen;
        let mut gc;
        let mut tim = [0u8; 64];

        let mut x: u_int;
        let y: u_int;
        let t: time_t = time(core::ptr::null_mut::<time_t>());
        let tm = tm::local(t).unwrap_or_default();
        let format = match style {
            0 => c"%l:%M ",
            2 => c"%l:%M:%S ",
            3 => c"%H:%M:%S",
            _ => c"%H:%M",
        };
        let mut length = tm.format_into(&mut tim, format);
        if style == 0 || style == 2 {
            let suffix = if tm.tm_hour >= 12 { b"PM" } else { b"AM" };
            tim[length..length + suffix.len()].copy_from_slice(suffix);
            length += suffix.len();
        }
        let digits = &tim[..length];
        let sx = RustScreen::grid(&*s).width();
        let sy = RustScreen::grid(&*s).height();
        let mut writer = screen_write_ctx_on_screen(s);
        writer.clearscreen(8 as u_int);
        if (sx as size_t) < (6 as size_t).wrapping_mul(digits.len()) || sy < 6 as u_int {
            if sx as size_t >= digits.len() && sy != 0 as u_int {
                x = (sx.wrapping_div(2 as u_int) as size_t)
                    .wrapping_sub(digits.len().wrapping_div(2 as size_t))
                    as u_int;
                y = sy.wrapping_div(2 as u_int);
                writer.cursormove(
                    x as core::ffi::c_int,
                    y as core::ffi::c_int,
                    0 as core::ffi::c_int,
                );
                gc = grid_default_cell;
                gc.flags = (gc.flags as core::ffi::c_int | GRID_FLAG_NOPALETTE) as u_char;
                gc.fg = colour;
                writer.puts(&gc, c"%s", fmt_args![digits]);
            }
            writer.finish();
            return;
        }
        x = (sx.wrapping_div(2 as u_int) as size_t)
            .wrapping_sub((3 as size_t).wrapping_mul(digits.len())) as u_int;
        y = sy.wrapping_div(2 as u_int).wrapping_sub(3 as u_int);
        gc = grid_default_cell;
        gc.flags = (gc.flags as core::ffi::c_int | GRID_FLAG_NOPALETTE) as u_char;
        gc.bg = colour;
        gc.fg = colour;
        for &ch in digits {
            if let Some(glyph) = window_clock_glyph(ch) {
                for j in 0..5 as u_int {
                    for i in 0..5 as u_int {
                        writer.cursormove(
                            x.wrapping_add(i) as core::ffi::c_int,
                            y.wrapping_add(j) as core::ffi::c_int,
                            0 as core::ffi::c_int,
                        );
                        if glyph[j as usize][i as usize] {
                            writer.putc(&gc, '#' as i32 as u_char);
                        }
                    }
                }
            }
            x = x.wrapping_add(6 as u_int);
        }
        writer.finish();
    }
}
use crate::screen::RustScreen;

#[cfg(test)]
#[path = "../tests/test_modes_clock_focused.rs"]
mod focused_tests;
