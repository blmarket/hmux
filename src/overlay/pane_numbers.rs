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

use crate::fmt_args;
use crate::grid::grid_default_cell;
use crate::log::{fatalx, log_debug};
use crate::modes::window_clock_table;
use crate::pane_geometry::PaneGeometryState;
use crate::tty::{tty_attributes, tty_cursor, tty_putn};
use crate::types::*;
use core::ffi::c_int;

/// Writes `text` into one of the drawing's sixteen-byte buffers the way
/// `xsnprintf` fills one, answering how many bytes it wrote. The bytes past the
/// text keep the zeroes the buffer started with — the one-line drawing reads
/// them back out — and a string that does not fit aborts, as `xsnprintf` does.
fn cmd_display_panes_fill(buf: &mut [u8; 16], text: &str) -> size_t {
    if text.len() >= buf.len() {
        fatalx(c"xsnprintf: overflow", fmt_args![]);
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
    wp: &(impl crate::WindowPane + ?Sized),
    cx: u_int,
    cy: u_int,
    buf: &[u8],
) {
    unsafe {
        let mut client = ctx.c.clone().expect("the redraw context has a client");
        let tty = client.as_tty_mut();
        let mut ranges = visible_ranges::default();
        ranges.set_visible_ranges(
            Some(wp),
            (ctx.ox as u_int).wrapping_add(cx) as c_int,
            (ctx.oy as u_int).wrapping_add(cy) as c_int,
            buf.len() as u_int,
        );
        for i in 0..ranges.used {
            let ri = ranges.ranges[i as usize];
            let mut j = ri.px;
            while j < ri.px.wrapping_add(ri.nx) {
                tty_cursor(tty, j.wrapping_sub(ctx.ox as u_int), cy);
                tty_putn(
                    tty,
                    core::slice::from_ref(&buf[j.wrapping_sub(ri.px) as usize]),
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
unsafe fn cmd_display_panes_draw_pane(
    ctx: &mut screen_redraw_ctx,
    pane_owner: &RustWindowPaneWeak,
) {
    unsafe {
        let mut client = ctx.c.clone().expect("the redraw context has a client");
        let Some(session) = client.attached_session() else {
            return;
        };
        let Some(window) = pane_owner.window() else {
            return;
        };
        let Some(wp) = pane_owner.get() else { return };
        let oo = session.options();
        let tty = client.as_tty_mut();

        if wp.geometry().xoff.wrapping_add(wp.geometry().sx as c_int) <= ctx.ox
            || wp.geometry().xoff >= ctx.ox.wrapping_add(ctx.sx as c_int)
            || wp.geometry().yoff.wrapping_add(wp.geometry().sy as c_int) <= ctx.oy
            || wp.geometry().yoff >= ctx.oy.wrapping_add(ctx.sy as c_int)
        {
            return;
        }
        let (xoff, sx) =
            cmd_display_panes_clip(wp.geometry().xoff, wp.geometry().sx, ctx.ox, ctx.sx);
        let (mut yoff, sy) =
            cmd_display_panes_clip(wp.geometry().yoff, wp.geometry().sy, ctx.oy, ctx.sy);
        if ctx.statustop != 0 {
            yoff = yoff.wrapping_add(ctx.statuslines);
        }
        let mut px = sx.wrapping_div(2);
        let mut py = sy.wrapping_div(2);

        let (found, pane) = window.pane_index(wp);
        if found != 0 {
            fatalx(c"index not found", fmt_args![]);
        }
        let mut buf = [0u8; 16];
        let mut len = cmd_display_panes_fill(&mut buf, &pane.to_string());
        if (sx as size_t) < len {
            return;
        }

        let colour = (oo).number(c"display-panes-colour") as c_int;
        let active_colour = (oo).number(c"display-panes-active-colour") as c_int;
        let mut fgc = grid_default_cell;
        let mut bgc = grid_default_cell;
        if window
            .active_pane()
            .is_some_and(|active| active.ptr_eq(pane_owner))
        {
            fgc.fg = active_colour;
            bgc.bg = active_colour;
        } else {
            fgc.fg = colour;
            bgc.bg = colour;
        }

        let mut rbuf = [0u8; 16];
        let rlen = cmd_display_panes_fill(
            &mut rbuf,
            &format!("{}x{}", wp.geometry().sx, wp.geometry().sy),
        );
        let mut lbuf = [0u8; 16];
        let llen = match pane > 9 && pane < 35 {
            true => cmd_display_panes_fill(
                &mut lbuf,
                &(b'a'.wrapping_add(pane.wrapping_sub(10) as u8) as char).to_string(),
            ),
            false => 0,
        };

        if (sx as size_t) < len.wrapping_mul(6) || sy < 5 {
            tty_attributes(tty, &fgc, &grid_default_cell, None, None);
            if sx as size_t >= len.wrapping_add(llen).wrapping_add(1) {
                len = len.wrapping_add(llen.wrapping_add(1));
                let mut cx =
                    (xoff.wrapping_add(px) as size_t).wrapping_sub(len.wrapping_div(2)) as u_int;
                let cy = yoff.wrapping_add(py);
                cmd_display_panes_put(ctx, wp, cx, cy, &buf[..len as usize]);
                cx = (cx as size_t).wrapping_add(len) as u_int;
                cmd_display_panes_put(ctx, wp, cx, cy, b" ");
                cx = cx.wrapping_add(1);
                cmd_display_panes_put(ctx, wp, cx, cy, &lbuf[..llen as usize]);
            } else {
                let cx =
                    (xoff.wrapping_add(px) as size_t).wrapping_sub(len.wrapping_div(2)) as u_int;
                let cy = yoff.wrapping_add(py);
                cmd_display_panes_put(ctx, wp, cx, cy, &buf[..len as usize]);
            }
            tty_cursor(tty, 0, 0);
            return;
        }

        px = (px as size_t).wrapping_sub(len.wrapping_mul(3)) as u_int;
        py = py.wrapping_sub(2);
        tty_attributes(tty, &bgc, &grid_default_cell, None, None);
        for digit in &buf[..len as usize] {
            let idx = digit.wrapping_sub(b'0') as usize;
            for j in 0..5 {
                let mut i = px;
                while i < px.wrapping_add(5) {
                    if window_clock_table[idx][j as usize][i.wrapping_sub(px) as usize] {
                        cmd_display_panes_put(
                            ctx,
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
            tty_attributes(tty, &fgc, &grid_default_cell, None, None);
            if rlen != 0 && sx as size_t >= rlen {
                let cx = (xoff.wrapping_add(sx) as size_t).wrapping_sub(rlen) as u_int;
                cmd_display_panes_put(ctx, wp, cx, yoff, &rbuf[..rlen as usize]);
            }
            if llen != 0 {
                let cx = (xoff.wrapping_add(sx.wrapping_div(2)) as size_t)
                    .wrapping_add(len.wrapping_mul(3))
                    .wrapping_sub(llen)
                    .wrapping_sub(1) as u_int;
                let cy = yoff.wrapping_add(py).wrapping_add(5);
                cmd_display_panes_put(ctx, wp, cx, cy, &lbuf[..llen as usize]);
            }
        }
        tty_cursor(tty, 0, 0);
    }
}

/// The overlay's draw callback: every visible pane of the client's current
/// window gets its number.
pub(crate) unsafe fn draw_pane_numbers(c: &mut ClientRef, ctx: &mut screen_redraw_ctx) {
    unsafe {
        let Some(session) = c.attached_session() else {
            return;
        };
        let Some(window) = session.current_window() else {
            return;
        };
        log_debug(
            c"%s: %s @%u",
            fmt_args![c"cmd_display_panes_draw", c.name(), window.window_id()],
        );
        let panes = window.panes();
        for pane in panes {
            if pane.get().is_some_and(|wp| window.pane_visible(wp)) {
                cmd_display_panes_draw_pane(ctx, &pane);
            }
        }
    }
}
