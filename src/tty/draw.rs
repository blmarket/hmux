use super::driver::tty_client;
use super::driver::{
    tty_attributes, tty_check_codeset, tty_cursor, tty_default_attributes, tty_fake_bce,
    tty_margin_off, tty_putc, tty_putcode, tty_putcode_i, tty_putn, tty_region_off,
    tty_repeat_space, tty_update_mode,
};
pub use crate::consts::{
    GRID_ATTR_CHARSET, GRID_FLAG_CLEARED, GRID_FLAG_PADDING, GRID_FLAG_SELECTED, GRID_FLAG_TAB,
    GRID_LINE_WRAPPED, TTY_NOCURSOR, TTYC_ECH, TTYC_EL, TTYC_EL1,
};
use crate::fmt_args;
use crate::grid::grid_cells_look_equal;
use crate::grid::grid_view_get_cell;
use crate::grid::{grid_default_cell, grid_peek_info};
use crate::log::{fatalx, log_debug, log_get_level};
use crate::screen::Screen;
use crate::terminfo::{TerminalCapabilities, tty_term_of};
pub use crate::types::*;

pub type tty_draw_line_state = core::ffi::c_uint;
pub const TTY_DRAW_LINE_DONE: tty_draw_line_state = 6;
pub const TTY_DRAW_LINE_SAME: tty_draw_line_state = 5;
pub const TTY_DRAW_LINE_EMPTY: tty_draw_line_state = 4;
pub const TTY_DRAW_LINE_NEW2: tty_draw_line_state = 3;
pub const TTY_DRAW_LINE_NEW1: tty_draw_line_state = 2;
pub const TTY_DRAW_LINE_FLUSH: tty_draw_line_state = 1;
pub const TTY_DRAW_LINE_FIRST: tty_draw_line_state = 0;

static tty_draw_line_states: [&core::ffi::CStr; 7] = [
    c"FIRST", c"FLUSH", c"NEW1", c"NEW2", c"EMPTY", c"SAME", c"DONE",
];
unsafe fn tty_draw_line_clear(
    tty: &mut tty,
    px: u_int,
    py: u_int,
    nx: u_int,
    defaults: &grid_cell,
    bg: u_int,
    wrapped: core::ffi::c_int,
) {
    unsafe {
        if nx == 0 as u_int {
            return;
        }
        if tty_client(tty)
            .expect("the tty has a client")
            .overlay_check()
            .is_none()
            && wrapped == 0
            && nx >= 10 as u_int
            && tty_fake_bce(tty, defaults, bg) == 0
        {
            if px.wrapping_add(nx) >= tty.sx && tty_term_of(tty).has(TTYC_EL) {
                tty_cursor(tty, px, py);
                tty_putcode(tty, TTYC_EL);
                return;
            }
            if px == 0 as u_int && tty_term_of(tty).has(TTYC_EL1) {
                tty_cursor(tty, px.wrapping_add(nx).wrapping_sub(1 as u_int), py);
                tty_putcode(tty, TTYC_EL1);
                return;
            }
            if tty_term_of(tty).has(TTYC_ECH) {
                tty_cursor(tty, px, py);
                tty_putcode_i(tty, TTYC_ECH, nx as core::ffi::c_int);
                return;
            }
        }
        if px != 0 as u_int || wrapped == 0 {
            tty_cursor(tty, px, py);
        }
        if nx == 1 as u_int {
            tty_putc(tty, ' ' as i32 as u_char);
        } else if nx == 2 as u_int {
            tty_putn(tty, b"  ", 2 as u_int);
        } else {
            tty_repeat_space(tty, nx);
        };
    }
}
#[allow(clippy::too_many_arguments)]
pub unsafe fn tty_draw_line(
    tty: &mut tty,
    s: &RustScreen,
    mut px: u_int,
    py: u_int,
    mut nx: u_int,
    mut atx: u_int,
    aty: u_int,
    defaults: &grid_cell,
    palette: Option<&colour_palette>,
) {
    unsafe {
        let current_block: u64;
        let mut gc = grid_default_cell;
        let mut ngc;
        let mut last;
        let mut i: u_int;
        let mut j: u_int;
        let mut last_i: u_int;
        let mut cx: u_int;

        let mut width: u_int;

        let mut bg: u_int;

        let mut empty: core::ffi::c_int;
        let mut wrapped: core::ffi::c_int = 0 as core::ffi::c_int;
        let mut buf: [u8; 1000] = [0; 1000];
        let mut len: size_t;
        let mut current_state: tty_draw_line_state;
        let mut next_state: tty_draw_line_state;
        log_debug(
            c"%s: px=%u py=%u nx=%u atx=%u aty=%u",
            fmt_args![c"tty_draw_line".as_ptr(), px, py, nx, atx, aty],
        );
        if atx >= tty.sx {
            return;
        }
        if atx.wrapping_add(nx) >= tty.sx {
            nx = tty.sx.wrapping_sub(atx);
        }
        if nx == 0 as u_int {
            return;
        }
        let gd = s.grid();
        let line_y = gd.history_size().wrapping_add(py);
        let cellsize: u_int = grid_peek_info(gd, line_y)
            .expect("a drawn line belongs to the screen")
            .cells;
        let ex: u_int = if s.grid().width() > cellsize {
            cellsize
        } else {
            s.grid().width()
        };
        log_debug(
            c"%s: drawing %u-%u,%u (end %u) at %u,%u; defaults: fg=%d, bg=%d",
            fmt_args![
                c"tty_draw_line".as_ptr(),
                px,
                px.wrapping_add(nx),
                py,
                ex,
                atx,
                aty,
                defaults.fg,
                defaults.bg
            ],
        );
        let flags: core::ffi::c_int = tty.flags & TTY_NOCURSOR;
        tty.flags |= TTY_NOCURSOR;
        tty_update_mode(tty, tty.mode, Some(s.mode_state()));
        tty_region_off(tty);
        tty_margin_off(tty);
        last = grid_default_cell;
        last.bg = defaults.bg;
        tty_default_attributes(tty, defaults, palette, 8 as u_int, Some(s.hyperlinks()));
        cx = 0 as u_int;
        i = px;
        while i < px.wrapping_add(nx) {
            gc = grid_view_get_cell(s.grid(), i, py);
            if !(gc.flags as core::ffi::c_int) & GRID_FLAG_PADDING != 0 {
                break;
            }
            cx = cx.wrapping_add(1);
            i = i.wrapping_add(1);
        }
        if cx != 0 as u_int {
            i = px.wrapping_add(1 as u_int);
            while i > 0 as u_int {
                gc = grid_view_get_cell(s.grid(), i.wrapping_sub(1 as u_int), py);
                if !(gc.flags as core::ffi::c_int) & GRID_FLAG_PADDING != 0 {
                    break;
                }
                i = i.wrapping_sub(1);
            }
            if i == 0 as u_int {
                bg = defaults.bg as u_int;
            } else {
                bg = gc.bg as u_int;
                if gc.flags as core::ffi::c_int & GRID_FLAG_SELECTED != 0 {
                    ngc = gc;
                    if s.select_cell(&mut ngc, &gc) {
                        bg = ngc.bg as u_int;
                    }
                }
            }
            tty_attributes(tty, &last, defaults, palette, Some(s.hyperlinks()));
            log_debug(
                c"%s: clearing %u padding cells",
                fmt_args![c"tty_draw_line".as_ptr(), cx],
            );
            tty_draw_line_clear(tty, atx, aty, cx, defaults, bg, 0 as core::ffi::c_int);
            if cx == ex {
                current_block = 15793177467813045649;
            } else {
                atx = atx.wrapping_add(cx);
                px = px.wrapping_add(cx);
                nx = nx.wrapping_sub(cx);
                current_block = 7226443171521532240;
            }
        } else {
            current_block = 7226443171521532240;
        }
        if current_block == 7226443171521532240 {
            if py != 0 as u_int && atx == 0 as u_int && tty.cx >= tty.sx && nx == tty.sx {
                let gd = s.grid();
                let previous_y = gd.history_size().wrapping_add(py).wrapping_sub(1);
                let gl = grid_peek_info(gd, previous_y)
                    .expect("the preceding line belongs to the screen");
                if gl.flags & GRID_LINE_WRAPPED != 0 {
                    wrapped = 1 as core::ffi::c_int;
                }
            }
            i = 0 as u_int;
            last_i = i;
            len = 0 as size_t;
            width = 0 as u_int;
            current_state = TTY_DRAW_LINE_FIRST;
            loop {
                let mut gcp: &grid_cell;
                if i == nx {
                    empty = 0 as core::ffi::c_int;
                    next_state = TTY_DRAW_LINE_DONE;
                    gcp = &grid_default_cell;
                } else {
                    if i > nx {
                        fatalx(c"position %u > width %u", fmt_args![i, nx]);
                    }
                    gc = grid_view_get_cell(s.grid(), px.wrapping_add(i), py);
                    gc = tty_check_codeset(tty, &gc);
                    gcp = &gc;
                    if gcp.flags as core::ffi::c_int & GRID_FLAG_SELECTED != 0 {
                        ngc = *gcp;
                        if s.select_cell(&mut ngc, gcp) {
                            gcp = &ngc;
                        }
                    }
                    empty = 0 as core::ffi::c_int;
                    if px >= ex || i >= ex.wrapping_sub(px) {
                        empty = 1 as core::ffi::c_int;
                    } else if gcp.data.width as u_int > nx.wrapping_sub(i) {
                        empty = nx.wrapping_sub(i) as core::ffi::c_int;
                    } else if gcp.flags as core::ffi::c_int & GRID_FLAG_PADDING != 0 {
                        empty = 1 as core::ffi::c_int;
                    } else if gcp.bg == last.bg
                        && gcp.attr as core::ffi::c_int == 0 as core::ffi::c_int
                        && gcp.link == 0 as u_int
                    {
                        if gcp.flags as core::ffi::c_int & GRID_FLAG_CLEARED != 0 {
                            empty = 1 as core::ffi::c_int;
                        } else if gcp.flags as core::ffi::c_int & GRID_FLAG_TAB != 0 {
                            empty = gcp.data.width as core::ffi::c_int;
                        } else if gcp.data.size as core::ffi::c_int == 1 as core::ffi::c_int
                            && gcp.data.data[0] as core::ffi::c_int == ' ' as i32
                        {
                            empty = 1 as core::ffi::c_int;
                        }
                    }
                    if empty != 0 as core::ffi::c_int {
                        next_state = TTY_DRAW_LINE_EMPTY;
                    } else if current_state as core::ffi::c_uint
                        == TTY_DRAW_LINE_FIRST as core::ffi::c_int as core::ffi::c_uint
                    {
                        next_state = TTY_DRAW_LINE_SAME;
                    } else if grid_cells_look_equal(gcp, &last) != 0 {
                        if gcp.data.size as usize > buf.len().wrapping_sub(len) {
                            next_state = TTY_DRAW_LINE_FLUSH;
                        } else {
                            next_state = TTY_DRAW_LINE_SAME;
                        }
                    } else if current_state as core::ffi::c_uint
                        == TTY_DRAW_LINE_NEW1 as core::ffi::c_int as core::ffi::c_uint
                    {
                        next_state = TTY_DRAW_LINE_NEW2;
                    } else {
                        next_state = TTY_DRAW_LINE_NEW1;
                    }
                }
                if log_get_level() != 0 as core::ffi::c_int {
                    log_debug(
                        c"%s: cell %u empty %u, bg %u; state: current %s, next %s",
                        fmt_args![
                            c"tty_draw_line".as_ptr(),
                            px.wrapping_add(i),
                            empty,
                            gcp.bg,
                            tty_draw_line_states[current_state as usize],
                            tty_draw_line_states[next_state as usize]
                        ],
                    );
                }
                if next_state as core::ffi::c_uint != current_state as core::ffi::c_uint {
                    if current_state as core::ffi::c_uint
                        == TTY_DRAW_LINE_EMPTY as core::ffi::c_int as core::ffi::c_uint
                    {
                        tty_attributes(tty, &last, defaults, palette, Some(s.hyperlinks()));
                        tty_draw_line_clear(
                            tty,
                            atx.wrapping_add(last_i),
                            aty,
                            i.wrapping_sub(last_i),
                            defaults,
                            last.bg as u_int,
                            wrapped,
                        );
                        wrapped = 0 as core::ffi::c_int;
                    } else if next_state as core::ffi::c_uint
                        != TTY_DRAW_LINE_SAME as core::ffi::c_int as core::ffi::c_uint
                        && len != 0 as size_t
                    {
                        tty_attributes(tty, &last, defaults, palette, Some(s.hyperlinks()));
                        if atx.wrapping_add(i).wrapping_sub(width) != 0 as u_int || wrapped == 0 {
                            tty_cursor(tty, atx.wrapping_add(i).wrapping_sub(width), aty);
                        }
                        if !(last.attr as core::ffi::c_int) & GRID_ATTR_CHARSET != 0 {
                            tty_putn(tty, &buf[..len], width);
                        } else {
                            j = 0 as u_int;
                            while (j as size_t) < len {
                                tty_putc(tty, buf[j as usize]);
                                j = j.wrapping_add(1);
                            }
                        }
                        len = 0 as size_t;
                        width = 0 as u_int;
                        wrapped = 0 as core::ffi::c_int;
                    }
                    last_i = i;
                }
                if next_state as core::ffi::c_uint
                    != TTY_DRAW_LINE_EMPTY as core::ffi::c_int as core::ffi::c_uint
                {
                    let size = gcp.data.size as usize;
                    buf[len..len + size].copy_from_slice(&gcp.data.data[..size]);
                    len = len.wrapping_add(size);
                    width = width.wrapping_add(gcp.data.width as u_int);
                }
                if next_state as core::ffi::c_uint
                    == TTY_DRAW_LINE_DONE as core::ffi::c_int as core::ffi::c_uint
                {
                    break;
                }
                current_state = next_state;
                last = *gcp;
                if empty != 0 as core::ffi::c_int {
                    i = i.wrapping_add(empty as u_int);
                } else {
                    i = i.wrapping_add(gcp.data.width as u_int);
                }
            }
        }
        tty.flags = tty.flags & !TTY_NOCURSOR | flags;
        tty_update_mode(tty, tty.mode, Some(s.mode_state()));
    }
}
use crate::screen::RustScreen;

#[cfg(test)]
pub use crate::consts::{
    MSG_COMMAND, MSG_FLAGS, MSG_READ_CANCEL, MSG_READ_OPEN, MSG_VERSION, TTYC_ACSC, TTYC_BCE,
    TTYC_XT,
};
