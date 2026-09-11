use crate::WindowPane;
use crate::pane_identity::PaneIdentity;
use crate::window_scrollbar::WindowScrollbarState;
use crate::window_trait::Window as _;

use crate::window_dimensions::WindowDimensionsState;
use crate::window_fill_character::WindowFillCharacterState;

use super::RustScreen;
use super::writer::{ScreenWriteCtx, screen_write_ctx_on_screen};
use crate::fmt_args;
use crate::format::{format_create, format_create_defaults, format_defaults, format_expand_time};
use crate::grid::{Grid, grid_default_cell};
use crate::layout::{LAYOUT_CELL_FLOATING, layout_cell_for_pane};
use crate::log::log_debug;
use crate::modes::window_copy_get_current_offset;

use crate::pane_geometry::PaneGeometryState;
use crate::pane_scrollbar::{PaneScrollbarSlider};
use crate::server::client_ref_of;
use crate::server::server_client_get_pane;
use crate::server::{marked_pane, server_is_marked};

pub use crate::consts::{
    CELL_BORDERS, CELL_LEFTJOIN, CELL_LEFTRIGHT, CELL_RIGHTJOIN, CELL_TOPBOTTOM,
    CLIENT_ALLREDRAWFLAGS, CLIENT_REDRAWBORDERS, CLIENT_REDRAWOVERLAY, CLIENT_REDRAWSTATUS,
    CLIENT_REDRAWSTATUSALWAYS, CLIENT_REDRAWWINDOW, CLIENT_SUSPENDED, CLIENT_UTF8, FORMAT_PANE,
    FORMAT_STATUS, GRID_ATTR_CHARSET, GRID_ATTR_REVERSE, LAYOUT_LEFTRIGHT, LAYOUT_TOPBOTTOM,
    MODE_SYNC, PANE_LINES_DOUBLE, PANE_LINES_HEAVY, PANE_LINES_NUMBER, PANE_LINES_SIMPLE,
    PANE_LINES_SPACES, PANE_SCROLLBARS_LEFT, PANE_SCROLLBARS_MODAL, PANE_SCROLLBARS_OFF,
    PANE_SCROLLBARS_RIGHT, PANE_STATUS_BOTTOM, PANE_STATUS_OFF, PANE_STATUS_TOP, SIMPLE_BORDERS,
    UINT_MAX, WINDOW_PANE_NO_MODE,
};
use crate::status::{status_line_size, status_message_redraw, status_prompt_redraw, status_redraw};
use crate::style::{style_add, style_apply};
use crate::terminfo::{AlternateCharacterSet, BorderCharacterSet, RustAlternateCharacterSet};
use crate::terminfo::{TerminalCapabilities, tty_term_of};
use crate::text::{utf8_copy, utf8_set};
use crate::tty::tty_draw_line;
use crate::tty::{
    tty_cell, tty_check_overlay_range, tty_cursor, tty_default_colours, tty_puts, tty_reset,
    tty_sync_start, tty_update_mode, tty_window_offset,
};
pub use crate::types::*;
use crate::window::{
    window_pane_index, window_pane_is_floating, window_pane_mode,
    window_pane_show_scrollbar, window_pane_visible,
};

pub const TTYC_BIDI: tty_code_code = 5;
pub const SCREEN_REDRAW_BORDER_LEFT: screen_redraw_border_type = 2;
pub const SCREEN_REDRAW_BORDER_RIGHT: screen_redraw_border_type = 3;
pub const SCREEN_REDRAW_OUTSIDE: screen_redraw_border_type = 0;
pub const SCREEN_REDRAW_BORDER_TOP: screen_redraw_border_type = 4;
pub const SCREEN_REDRAW_BORDER_BOTTOM: screen_redraw_border_type = 5;
pub const SCREEN_REDRAW_INSIDE: screen_redraw_border_type = 1;
pub type screen_redraw_border_type = core::ffi::c_uint;

pub const CELL_INSIDE: core::ffi::c_int = 0 as core::ffi::c_int;

pub const CELL_TOPJOIN: core::ffi::c_int = 7 as core::ffi::c_int;
pub const CELL_BOTTOMJOIN: core::ffi::c_int = 8 as core::ffi::c_int;

pub const CELL_OUTSIDE: core::ffi::c_int = 12 as core::ffi::c_int;
pub const CELL_SCROLLBAR: core::ffi::c_int = 13 as core::ffi::c_int;

pub const PANE_BORDER_ARROWS: core::ffi::c_int = 2 as core::ffi::c_int;
pub const PANE_BORDER_BOTH: core::ffi::c_int = 3 as core::ffi::c_int;

pub const START_ISOLATE: &core::ffi::CStr = c"\u{2066}";
pub const END_ISOLATE: &core::ffi::CStr = c"\u{2069}";
pub const BORDER_MARKERS: [u8; 7] = *b"  +,.-\0";
pub(crate) unsafe fn screen_redraw_border_set(
    owner: &WindowRef,
    wp: Option<&(impl crate::WindowPane + ?Sized)>,
    pane_lines: pane_lines,
    cell_type: core::ffi::c_int,
    gc: &mut grid_cell,
) {
    unsafe {
        let w = owner.as_window();
        let idx: u_int;
        if cell_type == CELL_OUTSIDE
            && let Some(fill) = w.fill_character()
        {
            utf8_copy(&mut gc.data, &fill);
            return;
        }
        match pane_lines {
            PANE_LINES_NUMBER => {
                if cell_type == CELL_OUTSIDE {
                    gc.attr = (gc.attr as core::ffi::c_int | GRID_ATTR_CHARSET) as u_short;
                    utf8_set(&mut gc.data, CELL_BORDERS[CELL_OUTSIDE as usize]);
                } else {
                    gc.attr = (gc.attr as core::ffi::c_int & !GRID_ATTR_CHARSET) as u_short;
                    if let Some(wp) = wp
                        && {
                            let found;
                            (found, idx) = window_pane_index(&w, wp);
                            found == 0
                        }
                    {
                        utf8_set(
                            &mut gc.data,
                            ('0' as i32 as u_int).wrapping_add(idx.wrapping_rem(10 as u_int))
                                as u_char,
                        );
                    } else {
                        utf8_set(&mut gc.data, '*' as i32 as u_char);
                    }
                }
            }
            PANE_LINES_DOUBLE => {
                gc.attr = (gc.attr as core::ffi::c_int & !GRID_ATTR_CHARSET) as u_short;
                let border = RustAlternateCharacterSet
                    .border(BorderCharacterSet::Double, cell_type as u8)
                    .expect("pane border cell type is valid");
                utf8_copy(&mut gc.data, &border);
            }
            PANE_LINES_HEAVY => {
                gc.attr = (gc.attr as core::ffi::c_int & !GRID_ATTR_CHARSET) as u_short;
                let border = RustAlternateCharacterSet
                    .border(BorderCharacterSet::Heavy, cell_type as u8)
                    .expect("pane border cell type is valid");
                utf8_copy(&mut gc.data, &border);
            }
            PANE_LINES_SIMPLE => {
                gc.attr = (gc.attr as core::ffi::c_int & !GRID_ATTR_CHARSET) as u_short;
                utf8_set(&mut gc.data, SIMPLE_BORDERS[cell_type as usize]);
            }
            PANE_LINES_SPACES => {
                gc.attr = (gc.attr as core::ffi::c_int & !GRID_ATTR_CHARSET) as u_short;
                utf8_set(&mut gc.data, ' ' as i32 as u_char);
            }
            _ => {
                gc.attr = (gc.attr as core::ffi::c_int | GRID_ATTR_CHARSET) as u_short;
                utf8_set(&mut gc.data, CELL_BORDERS[cell_type as usize]);
            }
        };
    }
}
/// The way the window is split when it holds exactly two laid-out panes under
/// one parent, or `None` when it does not.
pub(crate) fn screen_redraw_two_panes(owner: &WindowRef) -> Option<layout_type> {
    let w = owner.as_window();
    let mut count = 0;
    let mut split_type = None;
    for pane in &w.panes {
        let Some((cell, parent)) =
            layout_cell_for_pane(w.layout_root.as_deref(), &pane.downgrade())
        else {
            continue;
        };
        if cell.flags & LAYOUT_CELL_FLOATING != 0 {
            continue;
        }
        count += 1;
        if count > 2 {
            return None;
        }
        split_type = Some(parent?.type_0);
    }
    if count == 2 { split_type } else { None }
}
pub(crate) fn screen_redraw_pane_border(
    ctx: &screen_redraw_ctx,
    wp: &(impl crate::WindowPane + ?Sized),
    px: core::ffi::c_int,
    py: core::ffi::c_int,
) -> screen_redraw_border_type {
    {
        let window = wp
            .window_context()
            .expect("a rendered pane has a window context");
        let w = window.as_window();
        let oo = (*w).options_ref();
        let ex: core::ffi::c_int =
            (wp.geometry().xoff as u_int).wrapping_add(wp.geometry().sx) as core::ffi::c_int;
        let ey: core::ffi::c_int =
            (wp.geometry().yoff as u_int).wrapping_add(wp.geometry().sy) as core::ffi::c_int;
        let mut hsplit: core::ffi::c_int = 0 as core::ffi::c_int;
        let mut vsplit: core::ffi::c_int = 0 as core::ffi::c_int;
        let pane_status: core::ffi::c_int = ctx.pane_status;
        let mut sb_w: core::ffi::c_int = 0 as core::ffi::c_int;
        let sx: core::ffi::c_int = wp.geometry().sx as core::ffi::c_int;
        let sy: core::ffi::c_int = wp.geometry().sy as core::ffi::c_int;
        let mut left: core::ffi::c_int;
        let mut right: core::ffi::c_int;
        if px >= wp.geometry().xoff && px < ex && py >= wp.geometry().yoff && py < ey {
            return SCREEN_REDRAW_INSIDE;
        }
        if window_pane_show_scrollbar(wp, (*w).scrollbar_settings().sb) != 0 {
            sb_w = wp.scrollbar_style().width + wp.scrollbar_style().padding;
        }
        if window_pane_is_floating(
            &w,
            &{ (wp).observation() }.expect("the pane allocation exists"),
        ) != 0
        {
            left = wp.geometry().xoff - 1 as core::ffi::c_int;
            right = wp.geometry().xoff + sx;
            if (*w).scrollbar_settings().sb != PANE_SCROLLBARS_OFF
                && (*w).scrollbar_settings().sb_pos == PANE_SCROLLBARS_LEFT
            {
                left -= sb_w;
            } else {
                right += sb_w;
            }
            if py >= wp.geometry().yoff - 1 as core::ffi::c_int && py <= wp.geometry().yoff + sy {
                if px == left {
                    return SCREEN_REDRAW_BORDER_LEFT;
                }
                if px == right {
                    return SCREEN_REDRAW_BORDER_RIGHT;
                }
            }
            if px > left && px <= right {
                if py == wp.geometry().yoff - 1 as core::ffi::c_int {
                    return SCREEN_REDRAW_BORDER_TOP;
                }
                if py == wp.geometry().yoff + sy {
                    return SCREEN_REDRAW_BORDER_BOTTOM;
                }
            }
            return SCREEN_REDRAW_OUTSIDE;
        }
        match (oo).number(c"pane-border-indicators") {
            1 | 3 => {
                if let Some(split_type) = screen_redraw_two_panes(&window) {
                    hsplit = (split_type as core::ffi::c_uint
                        == LAYOUT_LEFTRIGHT as core::ffi::c_int as core::ffi::c_uint)
                        as core::ffi::c_int;
                    vsplit = (split_type as core::ffi::c_uint
                        == LAYOUT_TOPBOTTOM as core::ffi::c_int as core::ffi::c_uint)
                        as core::ffi::c_int;
                }
            }
            _ => {}
        }
        if (wp.geometry().yoff == 0 as core::ffi::c_int
            || py >= wp.geometry().yoff - 1 as core::ffi::c_int)
            && py <= ey
        {
            if (*w).scrollbar_settings().sb != PANE_SCROLLBARS_OFF
                && (*w).scrollbar_settings().sb_pos == PANE_SCROLLBARS_LEFT
            {
                if wp.geometry().xoff - sb_w == 0 as core::ffi::c_int
                    && px == sx + sb_w
                    && (hsplit == 0 || hsplit != 0 && py <= sy / 2 as core::ffi::c_int)
                {
                    return SCREEN_REDRAW_BORDER_RIGHT;
                }
                if wp.geometry().xoff - sb_w != 0 as core::ffi::c_int {
                    if px == wp.geometry().xoff - sb_w - 1 as core::ffi::c_int
                        && (hsplit == 0 || hsplit != 0 && py > sy / 2 as core::ffi::c_int)
                    {
                        return SCREEN_REDRAW_BORDER_LEFT;
                    }
                    if px == wp.geometry().xoff + sx + sb_w - 1 as core::ffi::c_int {
                        return SCREEN_REDRAW_BORDER_RIGHT;
                    }
                }
            } else {
                if wp.geometry().xoff == 0 as core::ffi::c_int
                    && px == sx + sb_w
                    && (hsplit == 0 || hsplit != 0 && py <= sy / 2 as core::ffi::c_int)
                {
                    return SCREEN_REDRAW_BORDER_RIGHT;
                }
                if wp.geometry().xoff != 0 as core::ffi::c_int {
                    if px == wp.geometry().xoff - 1 as core::ffi::c_int
                        && (hsplit == 0 || hsplit != 0 && py > sy / 2 as core::ffi::c_int)
                    {
                        return SCREEN_REDRAW_BORDER_LEFT;
                    }
                    if px == wp.geometry().xoff + sx + sb_w {
                        return SCREEN_REDRAW_BORDER_RIGHT;
                    }
                }
            }
        }
        if vsplit != 0 && pane_status == PANE_STATUS_OFF {
            if wp.geometry().yoff == 0 as core::ffi::c_int
                && py == sy
                && px <= sx / 2 as core::ffi::c_int
            {
                return SCREEN_REDRAW_BORDER_BOTTOM;
            }
            if wp.geometry().yoff != 0 as core::ffi::c_int
                && py == wp.geometry().yoff - 1 as core::ffi::c_int
                && px > sx / 2 as core::ffi::c_int
            {
                return SCREEN_REDRAW_BORDER_TOP;
            }
        } else if (*w).scrollbar_settings().sb != PANE_SCROLLBARS_OFF
            && (*w).scrollbar_settings().sb_pos == PANE_SCROLLBARS_LEFT
        {
            if (wp.geometry().xoff - sb_w == 0 as core::ffi::c_int || px >= wp.geometry().xoff - sb_w)
                && (px <= ex || sb_w != 0 as core::ffi::c_int && px < ex + sb_w)
            {
                if pane_status != PANE_STATUS_BOTTOM
                    && wp.geometry().yoff != 0 as core::ffi::c_int
                    && py == wp.geometry().yoff - 1 as core::ffi::c_int
                {
                    return SCREEN_REDRAW_BORDER_TOP;
                }
                if pane_status != PANE_STATUS_TOP && py == ey {
                    return SCREEN_REDRAW_BORDER_BOTTOM;
                }
            }
        } else if (wp.geometry().xoff == 0 as core::ffi::c_int || px >= wp.geometry().xoff)
            && (px <= ex || sb_w != 0 as core::ffi::c_int && px < ex + sb_w)
        {
            if pane_status != PANE_STATUS_BOTTOM
                && wp.geometry().yoff != 0 as core::ffi::c_int
                && py == wp.geometry().yoff - 1 as core::ffi::c_int
            {
                return SCREEN_REDRAW_BORDER_TOP;
            }
            if pane_status != PANE_STATUS_TOP && py == ey {
                return SCREEN_REDRAW_BORDER_BOTTOM;
            }
        }
        SCREEN_REDRAW_OUTSIDE
    }
}
unsafe fn screen_redraw_current_window(ctx: &screen_redraw_ctx) -> Option<WindowRef> {
    unsafe {
        let client = ctx.c.as_ref()?.as_client();
        let session = client.attached_session()?;
        session.current_window()
    }
}

fn screen_redraw_cell_border1(
    ctx: &screen_redraw_ctx,
    sb_pos: core::ffi::c_int,
    sb_w: core::ffi::c_int,
    wp: &(impl crate::WindowPane + ?Sized),
    px: core::ffi::c_int,
    py: core::ffi::c_int,
) -> core::ffi::c_int {
    {
        if sb_pos == PANE_SCROLLBARS_LEFT {
            if (px < wp.geometry().xoff - 1 as core::ffi::c_int - sb_w
                || px > wp.geometry().xoff + wp.geometry().sx as core::ffi::c_int)
                && (py < wp.geometry().yoff - 1 as core::ffi::c_int
                    || py > wp.geometry().yoff + wp.geometry().sy as core::ffi::c_int)
            {
                return -(1 as core::ffi::c_int);
            }
        } else if (px < wp.geometry().xoff - 1 as core::ffi::c_int
            || px > wp.geometry().xoff + wp.geometry().sx as core::ffi::c_int + sb_w)
            && (py < wp.geometry().yoff - 1 as core::ffi::c_int
                || py > wp.geometry().yoff + wp.geometry().sy as core::ffi::c_int)
        {
            return -(1 as core::ffi::c_int);
        }
        match screen_redraw_pane_border(ctx, wp, px, py) {
            SCREEN_REDRAW_INSIDE => 0 as core::ffi::c_int,
            SCREEN_REDRAW_OUTSIDE => -(1 as core::ffi::c_int),
            _ => 1 as core::ffi::c_int,
        }
    }
}
pub(crate) unsafe fn screen_redraw_cell_border(
    ctx: &screen_redraw_ctx,
    wp: &(impl crate::WindowPane + ?Sized),
    px: core::ffi::c_int,
    py: core::ffi::c_int,
) -> core::ffi::c_int {
    unsafe {
        let Some(window) = screen_redraw_current_window(ctx) else {
            return 0;
        };
        let w = window.as_window();
        let sx: core::ffi::c_int = (*w).dimensions().size.width as core::ffi::c_int;
        let mut sy: core::ffi::c_int = (*w).dimensions().size.height as core::ffi::c_int;

        let mut n: core::ffi::c_int;
        let sb_w: core::ffi::c_int =
            (*wp).scrollbar_style().width + (*wp).scrollbar_style().padding;
        if window_pane_is_floating(
            &w,
            &{ (wp).observation() }.expect("the pane allocation exists"),
        ) != 0
        {
            n = screen_redraw_cell_border1(
                ctx,
                if (*w).scrollbar_settings().sb != PANE_SCROLLBARS_OFF {
                    (*w).scrollbar_settings().sb_pos
                } else {
                    0 as core::ffi::c_int
                },
                sb_w,
                wp,
                px,
                py,
            );
            if n == -(1 as core::ffi::c_int) {
                return 0 as core::ffi::c_int;
            }
            return n;
        }
        if ctx.pane_status == PANE_STATUS_BOTTOM {
            sy -= 1;
        }
        if px > sx || py > sy {
            return 0 as core::ffi::c_int;
        }
        if px == sx || py == sy {
            return 1 as core::ffi::c_int;
        }
        for pane in &w.z_index {
            let Some(wp2) = pane.get() else {
                break;
            };
            if !(window_pane_visible(&w, wp2) == 0
                || window_pane_is_floating(
                    &w,
                    &{ (wp2).observation() }
                        .expect("the pane allocation exists"),
                ) != 0)
            {
                n = screen_redraw_cell_border1(
                    ctx,
                    if (*w).scrollbar_settings().sb != PANE_SCROLLBARS_OFF {
                        (*w).scrollbar_settings().sb_pos
                    } else {
                        0 as core::ffi::c_int
                    },
                    sb_w,
                    wp2,
                    px,
                    py,
                );
                if n != -(1 as core::ffi::c_int) {
                    return n;
                }
            }
        }
        0 as core::ffi::c_int
    }
}
pub(crate) unsafe fn screen_redraw_type_of_cell(
    ctx: &screen_redraw_ctx,
    wp: &(impl crate::WindowPane + ?Sized),
    px: core::ffi::c_int,
    py: core::ffi::c_int,
) -> core::ffi::c_int {
    unsafe {
        let Some(window) = screen_redraw_current_window(ctx) else {
            return CELL_OUTSIDE;
        };
        let w = window.as_window();
        let pane_status: core::ffi::c_int = ctx.pane_status;
        let mut borders: core::ffi::c_int = 0 as core::ffi::c_int;
        let sx: core::ffi::c_int = (*w).dimensions().size.width as core::ffi::c_int;
        let mut sy: core::ffi::c_int = (*w).dimensions().size.height as core::ffi::c_int;
        if pane_status == PANE_STATUS_BOTTOM {
            sy -= 1;
        }
        if px > sx || py > sy {
            return 12 as core::ffi::c_int;
        }
        if window_pane_is_floating(
            &w,
            &{ (wp).observation() }.expect("the pane allocation exists"),
        ) == 0
        {
            if px == 0 as core::ffi::c_int
                || screen_redraw_cell_border(ctx, wp, px - 1 as core::ffi::c_int, py) != 0
            {
                borders |= 8 as core::ffi::c_int;
            }
            if px <= sx && screen_redraw_cell_border(ctx, wp, px + 1 as core::ffi::c_int, py) != 0 {
                borders |= 4 as core::ffi::c_int;
            }
            if pane_status == PANE_STATUS_TOP {
                if py != 0 as core::ffi::c_int
                    && screen_redraw_cell_border(ctx, wp, px, py - 1 as core::ffi::c_int) != 0
                {
                    borders |= 2 as core::ffi::c_int;
                }
                if screen_redraw_cell_border(ctx, wp, px, py + 1 as core::ffi::c_int) != 0 {
                    borders |= 1 as core::ffi::c_int;
                }
            } else if pane_status == PANE_STATUS_BOTTOM {
                if py == 0 as core::ffi::c_int
                    || screen_redraw_cell_border(ctx, wp, px, py - 1 as core::ffi::c_int) != 0
                {
                    borders |= 2 as core::ffi::c_int;
                }
                if py != sy
                    && screen_redraw_cell_border(ctx, wp, px, py + 1 as core::ffi::c_int) != 0
                {
                    borders |= 1 as core::ffi::c_int;
                }
            } else {
                if py == 0 as core::ffi::c_int
                    || screen_redraw_cell_border(ctx, wp, px, py - 1 as core::ffi::c_int) != 0
                {
                    borders |= 2 as core::ffi::c_int;
                }
                if screen_redraw_cell_border(ctx, wp, px, py + 1 as core::ffi::c_int) != 0 {
                    borders |= 1 as core::ffi::c_int;
                }
            }
        } else {
            if screen_redraw_cell_border(ctx, wp, px - 1 as core::ffi::c_int, py) != 0 {
                borders |= 8 as core::ffi::c_int;
            }
            if px <= sx && screen_redraw_cell_border(ctx, wp, px + 1 as core::ffi::c_int, py) != 0 {
                borders |= 4 as core::ffi::c_int;
            }
            if pane_status == PANE_STATUS_TOP {
                if py != 0 as core::ffi::c_int
                    && screen_redraw_cell_border(ctx, wp, px, py - 1 as core::ffi::c_int) != 0
                {
                    borders |= 2 as core::ffi::c_int;
                }
                if screen_redraw_cell_border(ctx, wp, px, py + 1 as core::ffi::c_int) != 0 {
                    borders |= 1 as core::ffi::c_int;
                }
            } else if pane_status == PANE_STATUS_BOTTOM {
                if screen_redraw_cell_border(ctx, wp, px, py - 1 as core::ffi::c_int) != 0 {
                    borders |= 2 as core::ffi::c_int;
                }
                if py != sy
                    && screen_redraw_cell_border(ctx, wp, px, py + 1 as core::ffi::c_int) != 0
                {
                    borders |= 1 as core::ffi::c_int;
                }
            } else {
                if screen_redraw_cell_border(ctx, wp, px, py - 1 as core::ffi::c_int) != 0 {
                    borders |= 2 as core::ffi::c_int;
                }
                if screen_redraw_cell_border(ctx, wp, px, py + 1 as core::ffi::c_int) != 0 {
                    borders |= 1 as core::ffi::c_int;
                }
            }
        }
        match borders {
            15 => return 11 as core::ffi::c_int,
            14 => return 8 as core::ffi::c_int,
            13 => return 7 as core::ffi::c_int,
            12 => return 2 as core::ffi::c_int,
            11 => return 10 as core::ffi::c_int,
            10 => return 6 as core::ffi::c_int,
            9 => return 4 as core::ffi::c_int,
            7 => return 9 as core::ffi::c_int,
            6 => return 5 as core::ffi::c_int,
            5 => return 3 as core::ffi::c_int,
            3 => return 1 as core::ffi::c_int,
            _ => {}
        }
        12 as core::ffi::c_int
    }
}
struct ScreenRedrawCell {
    cell_type: core::ffi::c_int,
    pane_ref: Option<RustWindowPaneWeak>,
}

unsafe fn screen_redraw_check_cell(
    ctx: &screen_redraw_ctx,
    px: core::ffi::c_int,
    py: core::ffi::c_int,
) -> ScreenRedrawCell {
    unsafe {
        let mut current_block: u64;
        let Some(window) = screen_redraw_current_window(ctx) else {
            return ScreenRedrawCell {
                cell_type: CELL_OUTSIDE,
                pane_ref: None,
            };
        };
        let w = window.as_window();
        let mut selected = None;
        let sx: core::ffi::c_int = (*w).dimensions().size.width as core::ffi::c_int;
        let sy: core::ffi::c_int = (*w).dimensions().size.height as core::ffi::c_int;
        let pane_status: core::ffi::c_int = ctx.pane_status;
        let mut border: core::ffi::c_int;
        let mut pane_status_line: core::ffi::c_int;
        let mut tiled_only: core::ffi::c_int = 0 as core::ffi::c_int;
        let mut left: core::ffi::c_int;
        let mut right: core::ffi::c_int;
        let mut sb_w: core::ffi::c_int;
        let mut pane_ref = None;
        if px > sx || py > sy {
            return ScreenRedrawCell {
                cell_type: 12,
                pane_ref,
            };
        }
        for pane in &w.z_index {
            let Some(wp) = pane.get() else {
                break;
            };
            if !(window_pane_is_floating(
                &w,
                &{ (wp).observation() }.expect("the pane allocation exists"),
            ) != 0
                && (px >= sx || py >= sy))
            {
                sb_w = wp.scrollbar_style().width + wp.scrollbar_style().padding;
                if (*w).scrollbar_settings().sb != PANE_SCROLLBARS_OFF
                    && (*w).scrollbar_settings().sb_pos == PANE_SCROLLBARS_LEFT
                {
                    if px >= wp.geometry().xoff - 1 as core::ffi::c_int - sb_w
                        && px <= wp.geometry().xoff + wp.geometry().sx as core::ffi::c_int
                        && (py >= wp.geometry().yoff - 1 as core::ffi::c_int
                            && py <= wp.geometry().yoff + wp.geometry().sy as core::ffi::c_int)
                    {
                        selected = Some(pane.clone());
                        break;
                    }
                } else if px >= wp.geometry().xoff - 1 as core::ffi::c_int
                    && px <= wp.geometry().xoff + wp.geometry().sx as core::ffi::c_int + sb_w
                    && (py >= wp.geometry().yoff - 1 as core::ffi::c_int
                        && py <= wp.geometry().yoff + wp.geometry().sy as core::ffi::c_int)
                {
                    selected = Some(pane.clone());
                    break;
                }
            }
        }
        let Some(selected) = selected.or_else(|| {
            ctx.c
                .as_ref()
                .and_then(|client| server_client_get_pane(client.as_client()))
        }) else {
            return ScreenRedrawCell {
                cell_type: CELL_OUTSIDE,
                pane_ref,
            };
        };
        let Some(wp) = selected.get() else {
            return ScreenRedrawCell {
                cell_type: CELL_OUTSIDE,
                pane_ref,
            };
        };
        if px == sx || py == sy {
            return ScreenRedrawCell {
                cell_type: screen_redraw_type_of_cell(ctx, wp, px, py),
                pane_ref,
            };
        }
        if window_pane_is_floating(
            &w,
            &{ (wp).observation() }.expect("the pane allocation exists"),
        ) == 0
        {
            tiled_only = 1 as core::ffi::c_int;
        }
        let (next, wrap) = w
            .z_index
            .iter()
            .position(|pane| pane.ptr_eq(&selected))
            .map_or((0, 0), |index| (index + 1, index));
        let remaining = w
            .z_index
            .iter()
            .skip(next)
            .chain(w.z_index.iter().take(wrap))
            .take_while(|pane| pane.is_alive())
            .cloned();
        for pane in std::iter::once(selected.clone()).chain(remaining) {
            let Some(wp) = pane.get() else {
                continue;
            };
            if !(window_pane_visible(&w, wp) == 0)
                && !(tiled_only != 0
                    && window_pane_is_floating(
                        &w,
                        &{ (wp).observation() }
                            .expect("the pane allocation exists"),
                    ) != 0)
            {
                pane_ref = Some(pane.clone());
                sb_w = wp.scrollbar_style().width + wp.scrollbar_style().padding;
                if (*w).scrollbar_settings().sb != PANE_SCROLLBARS_OFF
                    && (*w).scrollbar_settings().sb_pos == PANE_SCROLLBARS_LEFT
                {
                    if (px < wp.geometry().xoff - 1 as core::ffi::c_int - sb_w
                        || px > wp.geometry().xoff + wp.geometry().sx as core::ffi::c_int)
                        && (py < wp.geometry().yoff - 1 as core::ffi::c_int
                            || py > wp.geometry().yoff + wp.geometry().sy as core::ffi::c_int)
                    {
                        current_block = 13503835911103092327;
                    } else {
                        current_block = 16924917904204750491;
                    }
                } else if (px < wp.geometry().xoff - 1 as core::ffi::c_int
                    || px > wp.geometry().xoff + wp.geometry().sx as core::ffi::c_int + sb_w)
                    && (py < wp.geometry().yoff - 1 as core::ffi::c_int
                        || py > wp.geometry().yoff + wp.geometry().sy as core::ffi::c_int)
                {
                    current_block = 13503835911103092327;
                } else {
                    current_block = 16924917904204750491;
                }
                match current_block {
                    13503835911103092327 => {}
                    _ => {
                        if pane_status != PANE_STATUS_OFF {
                            if pane_status == PANE_STATUS_TOP {
                                pane_status_line = wp.geometry().yoff - 1 as core::ffi::c_int;
                            } else {
                                pane_status_line =
                                    wp.geometry().yoff + wp.geometry().sy as core::ffi::c_int;
                            }
                            left = wp.geometry().xoff + 2 as core::ffi::c_int;
                            right = wp.geometry().xoff
                                + 2 as core::ffi::c_int
                                + wp.status_line_width() as core::ffi::c_int
                                - 1 as core::ffi::c_int;
                            if py == pane_status_line + ctx.oy && px >= left && px <= right {
                                return ScreenRedrawCell {
                                    cell_type: 0,
                                    pane_ref,
                                };
                            }
                        }
                        if window_pane_show_scrollbar(wp, (*w).scrollbar_settings().sb) != 0 {
                            sb_w = wp.scrollbar_style().width + wp.scrollbar_style().padding;
                            if (wp.geometry().yoff == 0 as core::ffi::c_int
                                && py < wp.geometry().sy as core::ffi::c_int
                                || py >= wp.geometry().yoff
                                    && py
                                        < wp.geometry().yoff
                                            + wp.geometry().sy as core::ffi::c_int)
                                && ((*w).scrollbar_settings().sb_pos == PANE_SCROLLBARS_RIGHT
                                    && (px
                                        >= wp.geometry().xoff
                                            + wp.geometry().sx as core::ffi::c_int
                                        && px
                                            < wp.geometry().xoff
                                                + wp.geometry().sx as core::ffi::c_int
                                                + sb_w)
                                    || (*w).scrollbar_settings().sb_pos == PANE_SCROLLBARS_LEFT
                                        && (px >= wp.geometry().xoff - sb_w && px < wp.geometry().xoff))
                            {
                                return ScreenRedrawCell {
                                    cell_type: 13,
                                    pane_ref,
                                };
                            }
                        }
                        border = screen_redraw_pane_border(ctx, wp, px, py) as core::ffi::c_int;
                        if border == SCREEN_REDRAW_INSIDE as core::ffi::c_int {
                            return ScreenRedrawCell {
                                cell_type: 0,
                                pane_ref,
                            };
                        }
                        if !(border == SCREEN_REDRAW_OUTSIDE as core::ffi::c_int) {
                            return ScreenRedrawCell {
                                cell_type: screen_redraw_type_of_cell(ctx, wp, px, py),
                                pane_ref,
                            };
                        }
                    }
                }
            }
        }
        ScreenRedrawCell {
            cell_type: 12,
            pane_ref,
        }
    }
}
pub(crate) fn screen_redraw_check_is(
    ctx: &screen_redraw_ctx,
    px: core::ffi::c_int,
    py: core::ffi::c_int,
    wp: Option<&(impl crate::WindowPane + ?Sized)>,
) -> core::ffi::c_int {
    {
        let Some(wp) = wp else {
            return 0 as core::ffi::c_int;
        };
        let border: screen_redraw_border_type = screen_redraw_pane_border(ctx, wp, px, py);
        if border as core::ffi::c_uint
            != SCREEN_REDRAW_INSIDE as core::ffi::c_int as core::ffi::c_uint
            && border as core::ffi::c_uint
                != SCREEN_REDRAW_OUTSIDE as core::ffi::c_int as core::ffi::c_uint
        {
            return 1 as core::ffi::c_int;
        }
        0 as core::ffi::c_int
    }
}
unsafe fn screen_redraw_make_pane_status(
    c: &client,
    pane: &mut RustWindowPaneWeak,
    rctx: &screen_redraw_ctx,
    pane_lines: pane_lines,
) -> core::ffi::c_int {
    unsafe {
        let Some(session) = c.attached_session() else {
            return 0;
        };
        let Some(window) = pane.window() else {
            return 0;
        };
        let Some(wp) = pane.get() else { return 0 };
        let options = wp.options_ref().clone();
        let sb_w = if window_pane_show_scrollbar(wp, window.scrollbar_settings().sb) != 0 {
            wp.scrollbar_style().width + wp.scrollbar_style().padding
        } else {
            0
        };
        let mut gc = grid_default_cell;
        let mut ft = format_create(
            Some(c),
            None,
            (FORMAT_PANE | pane.id()) as core::ffi::c_int,
            FORMAT_STATUS,
        );
        format_defaults(
            &mut ft,
            Some(c),
            Some(session.as_session()),
            session
                .curw()
                .as_ref()
                .and_then(crate::window::WinlinkRef::get),
            pane.get(),
        );
        let border_option = if server_client_get_pane(c).is_some_and(|active| active.ptr_eq(pane)) {
            c"pane-active-border-style"
        } else {
            c"pane-border-style"
        };
        style_apply(&mut gc, &options, border_option, Some(&mut ft));
        let fmt = options.string_ref(c"pane-border-format");
        let expanded = format_expand_time(&mut ft, &fmt);
        let Some(wp) = pane.get() else { return 0 };
        let geometry = wp.geometry();
        let width = if geometry.sx < 4 {
            0
        } else {
            geometry.sx.wrapping_add(sb_w as u_int).wrapping_sub(2)
        };
        let max_width =
            (window.dimensions().size.width as core::ffi::c_int - (geometry.xoff + 2)).max(0) as u_int;
        let width = width.min(max_width);
        let mut status_screen = RustScreen::new_with_server_options(width, 1, 0);
        status_screen.0.mode = 0;
        let mut ranges = Vec::new();
        let mut writer = screen_write_ctx_on_screen(&mut status_screen);
        for i in 0..width {
            let px = ((geometry.xoff + 2) as u_int).wrapping_add(i);
            let py = if rctx.pane_status == PANE_STATUS_TOP {
                (geometry.yoff - 1) as u_int
            } else {
                (geometry.yoff as u_int).wrapping_add(geometry.sy)
            };
            let Some(wp) = pane.get() else { return 0 };
            let cell_type = screen_redraw_type_of_cell(
                rctx,
                wp,
                px as core::ffi::c_int,
                py as core::ffi::c_int,
            );
            screen_redraw_border_set(&window, Some(wp), pane_lines, cell_type, &mut gc);
            writer.cell(&gc);
        }
        gc.attr = (gc.attr as core::ffi::c_int & !GRID_ATTR_CHARSET) as u_short;
        writer.cursormove(0, 0, 0);
        writer.format_draw(&gc, width, expanded.as_bytes(), Some(&mut ranges), 0);
        writer.finish();
        let Some(wp) = pane.get_mut() else { return 0 };
        wp.publish_border_status(width as usize, status_screen, ranges, expanded) as core::ffi::c_int
    }
}
unsafe fn screen_redraw_draw_pane_status(ctx: &mut screen_redraw_ctx) {
    unsafe {
        let mut client = ctx.c.clone().expect("the redraw context has a client");
        let Some(window) = screen_redraw_current_window(ctx) else {
            return;
        };
        let mut i: u_int;
        let mut l: u_int;
        let mut x: u_int;
        let mut width: u_int;
        let mut size: u_int;
        let mut xoff: core::ffi::c_int;
        let mut yoff: core::ffi::c_int;
        log_debug(
            c"%s: %s @%u",
            fmt_args![
                c"screen_redraw_draw_pane_status",
                client.name(),
                window.window_id()
            ],
        );
        let panes = window.panes();
        for pane in panes {
            let Some(wp) = pane.get() else { continue };
            if window_pane_visible(&window.as_window(), wp) != 0 {
                size = wp.status_line_width() as u_int;
                if ctx.pane_status == PANE_STATUS_TOP {
                    yoff = wp.geometry().yoff - 1 as core::ffi::c_int;
                } else {
                    yoff = (wp.geometry().yoff as u_int).wrapping_add(wp.geometry().sy)
                        as core::ffi::c_int;
                }
                xoff = wp.geometry().xoff + 2 as core::ffi::c_int;
                if !(xoff + size as core::ffi::c_int <= ctx.ox
                    || xoff >= ctx.ox + ctx.sx as core::ffi::c_int
                    || yoff < ctx.oy
                    || yoff >= ctx.oy + ctx.sy as core::ffi::c_int)
                {
                    if xoff >= ctx.ox
                        && (xoff as u_int).wrapping_add(size)
                            <= (ctx.ox as u_int).wrapping_add(ctx.sx)
                    {
                        l = 0 as u_int;
                        x = (xoff - ctx.ox) as u_int;
                        width = size;
                    } else if xoff < ctx.ox
                        && (xoff as u_int).wrapping_add(size)
                            > (ctx.ox as u_int).wrapping_add(ctx.sx)
                    {
                        l = (ctx.ox - xoff) as u_int;
                        x = 0 as u_int;
                        width = ctx.sx;
                    } else if xoff < ctx.ox {
                        l = (ctx.ox - xoff) as u_int;
                        x = 0 as u_int;
                        width = size.wrapping_sub(l);
                    } else {
                        l = 0 as u_int;
                        x = (xoff - ctx.ox) as u_int;
                        width = size.wrapping_sub(x);
                    }
                    let r = tty_check_overlay_range(client.as_tty_mut(), x, yoff as u_int, width);
                    let Some(wp) = pane.get() else { continue };
                    r.clip_visible_ranges(Some(wp), x as core::ffi::c_int, yoff, width);
                    if ctx.statustop != 0 {
                        yoff = (yoff as u_int).wrapping_add(ctx.statuslines) as core::ffi::c_int
                            as core::ffi::c_int;
                    }
                    i = 0 as u_int;
                    while let Some(ri) = r.range_at(i) {
                        if !(ri.nx == 0 as u_int) {
                            tty_draw_line(
                                client.as_tty_mut(),
                                wp.status_screen(),
                                l.wrapping_add(ri.px.wrapping_sub(x)),
                                0 as u_int,
                                ri.nx,
                                ri.px,
                                (yoff - ctx.oy) as u_int,
                                &grid_default_cell,
                                None,
                            );
                        }
                        i = i.wrapping_add(1);
                    }
                }
            }
        }
        tty_cursor(client.as_tty_mut(), 0 as u_int, 0 as u_int);
    }
}
unsafe fn screen_redraw_update(ctx: &mut screen_redraw_ctx, mut flags: uint64_t) -> uint64_t {
    unsafe {
        let mut client = ctx.c.clone().expect("the redraw context has a client");
        let Some(window) = screen_redraw_current_window(ctx) else {
            return 0;
        };
        let redraw = if client.as_client().message_string.is_some() {
            status_message_redraw(client.as_client_mut())
        } else if client.as_client().prompt_string.is_some() {
            status_prompt_redraw(client.as_client_mut())
        } else {
            status_redraw(client.as_client_mut())
        };
        if redraw == 0 && flags & CLIENT_REDRAWSTATUSALWAYS as uint64_t == 0 {
            flags &= !CLIENT_REDRAWSTATUS as uint64_t;
        }
        if client.overlay().is_some() {
            flags |= CLIENT_REDRAWOVERLAY as uint64_t;
        }
        if ctx.pane_status != PANE_STATUS_OFF {
            let mut redraw = false;
            let panes = window.panes();
            for mut pane in panes {
                if screen_redraw_make_pane_status(
                    client.as_client(),
                    &mut pane,
                    ctx,
                    ctx.pane_lines,
                ) != 0
                {
                    redraw = true;
                }
            }
            if redraw {
                flags |= CLIENT_REDRAWBORDERS as uint64_t;
            }
        }
        flags
    }
}
unsafe fn screen_redraw_set_context(c: &client, ctx: &mut screen_redraw_ctx) -> bool {
    unsafe {
        *ctx = screen_redraw_ctx::default();
        let Some(session) = c.attached_session() else {
            return false;
        };
        let Some(window) = session.current_window() else {
            return false;
        };
        let w = window.as_window();
        let wo = w.options_ref();
        let oo = session.options();
        let mut lines: u_int;
        ctx.c = client_ref_of(c);
        lines = status_line_size(c);
        if c.message_string.is_some() || c.prompt_string.is_some() {
            lines = if lines == 0 as u_int {
                1 as u_int
            } else {
                lines
            };
        }
        if lines != 0 as u_int && (oo).number(c"status-position") == 0 as core::ffi::c_longlong {
            ctx.statustop = 1 as core::ffi::c_int;
        }
        ctx.statuslines = lines;
        ctx.pane_status = (wo).number(c"pane-border-status") as core::ffi::c_int;
        ctx.pane_lines = (wo).number(c"pane-border-lines") as pane_lines;
        {
            let (_bigger, off_x, off_y, off_sx, off_sy) = tty_window_offset(&c.tty);
            (ctx.ox, ctx.oy) = (off_x as core::ffi::c_int, off_y as core::ffi::c_int);
            (ctx.sx, ctx.sy) = (off_sx, off_sy);
        }
        log_debug(
            c"%s: %s @%u ox=%u oy=%u sx=%u sy=%u %u/%d",
            fmt_args![
                c"screen_redraw_set_context",
                c.name.as_deref(),
                w.window_id(),
                ctx.ox,
                ctx.oy,
                ctx.sx,
                ctx.sy,
                ctx.statuslines,
                ctx.statustop
            ],
        );
        true
    }
}
pub unsafe fn screen_redraw_screen(c: &mut client) {
    unsafe {
        let mut ctx = screen_redraw_ctx::default();

        if c.flags & CLIENT_SUSPENDED as uint64_t != 0 {
            return;
        }
        if !screen_redraw_set_context(c, &mut ctx) {
            return;
        }
        let flags: uint64_t = screen_redraw_update(&mut ctx, c.flags);
        if flags as core::ffi::c_ulonglong & CLIENT_ALLREDRAWFLAGS == 0 as core::ffi::c_ulonglong {
            return;
        }
        tty_sync_start(&mut c.tty);
        let mode = c.tty.mode;
        tty_update_mode(&mut c.tty, mode, None);
        if flags & (CLIENT_REDRAWWINDOW | CLIENT_REDRAWBORDERS) as uint64_t != 0 {
            log_debug(c"%s: redrawing borders", fmt_args![c.name.as_deref()]);
            screen_redraw_draw_borders(&mut ctx);
            if ctx.pane_status != PANE_STATUS_OFF {
                screen_redraw_draw_pane_status(&mut ctx);
            }
            screen_redraw_draw_pane_scrollbars(&mut ctx);
        }
        if flags & CLIENT_REDRAWWINDOW as uint64_t != 0 {
            log_debug(c"%s: redrawing panes", fmt_args![c.name.as_deref()]);
            screen_redraw_draw_panes(&mut ctx);
            screen_redraw_draw_pane_scrollbars(&mut ctx);
        }
        if ctx.statuslines != 0 as u_int
            && flags & (CLIENT_REDRAWSTATUS | CLIENT_REDRAWSTATUSALWAYS) as uint64_t != 0
        {
            log_debug(c"%s: redrawing status", fmt_args![c.name.as_deref()]);
            screen_redraw_draw_status(&mut ctx);
        }
        if (*c).overlay().is_some() && flags & CLIENT_REDRAWOVERLAY as uint64_t != 0 {
            log_debug(c"%s: redrawing overlay", fmt_args![c.name.as_deref()]);
            let overlay = (*c).overlay();
            let data = (*c).current_overlay_data();
            overlay.draw(&mut *c, data, &mut ctx);
        }
        tty_reset(&mut c.tty);
    }
}
pub unsafe fn screen_redraw_pane(
    c: &mut client,
    pane: &mut RustWindowPaneWeak,
    redraw_scrollbar_only: core::ffi::c_int,
) {
    unsafe {
        let Some(window) = pane.window() else { return };
        let Some(wp) = pane.get() else { return };
        if window_pane_visible(&window.as_window(), wp) == 0 {
            return;
        }
        let mut ctx = screen_redraw_ctx::default();
        if !screen_redraw_set_context(c, &mut ctx) {
            return;
        }
        tty_sync_start(&mut c.tty);
        let mode = c.tty.mode;
        tty_update_mode(&mut c.tty, mode, None);
        if redraw_scrollbar_only == 0 {
            screen_redraw_draw_pane(&mut ctx, pane);
        }
        let show_scrollbar = pane.get().is_some_and(|wp| {
            pane.window().is_some_and(|window| {
                window_pane_show_scrollbar(wp, window.scrollbar_settings().sb) != 0
            })
        });
        if show_scrollbar {
            screen_redraw_draw_pane_scrollbar(&mut ctx, pane);
        }
        tty_reset(&mut c.tty);
    }
}
#[derive(Default)]
struct BorderPassCache(Vec<(RustWindowPaneWeak, [Option<grid_cell>; 2])>);

impl BorderPassCache {
    fn cell(&self, pane: &RustWindowPaneWeak, active: bool) -> Option<grid_cell> {
        self.0.iter().find(|(cached, _)| cached.ptr_eq(pane))
            .and_then(|(_, cells)| cells[usize::from(active)])
    }

    fn store(&mut self, pane: &RustWindowPaneWeak, active: bool, cell: grid_cell) {
        if let Some((_, cells)) = self.0.iter_mut().find(|(cached, _)| cached.ptr_eq(pane)) {
            cells[usize::from(active)] = Some(cell);
        } else {
            let mut cells = [None; 2];
            cells[usize::from(active)] = Some(cell);
            self.0.push((pane.clone(), cells));
        }
    }
}

unsafe fn screen_redraw_draw_borders_style(
    ctx: &screen_redraw_ctx,
    cache: &mut BorderPassCache,
    x: u_int,
    y: u_int,
    pane: &mut RustWindowPaneWeak,
    ngc: &mut grid_cell,
) {
    unsafe {
        let client = ctx.c.clone().expect("the redraw context has a client");
        let Some(session) = client.attached_session() else {
            return;
        };
        let active = server_client_get_pane(client.as_client());
        let Some(window) = pane.window() else {
            return;
        };
        let Some(wp) = pane.get() else {
            return;
        };
        let is_active = if window_pane_is_floating(&window.as_window(), pane) != 0 {
            active.as_ref().is_some_and(|active| active.ptr_eq(pane))
        } else {
            screen_redraw_check_is(
                ctx,
                x as core::ffi::c_int,
                y as core::ffi::c_int,
                active.as_ref().and_then(|active| active.get()),
            ) != 0
        };
        let option = if is_active { c"pane-active-border-style" } else { c"pane-border-style" };
        if let Some(cell) = cache.cell(pane, is_active) {
            *ngc = cell;
            return;
        }
        let options = wp.options_ref().clone();
        let mut ft = format_create_defaults(
            None,
            Some(client.as_client()),
            Some(session.as_session()),
            session
                .curw()
                .as_ref()
                .and_then(crate::window::WinlinkRef::get),
            pane.get(),
        );
        let mut cell = grid_default_cell;
        style_apply(&mut cell, &options, option, Some(&mut ft));
        cache.store(pane, is_active, cell);
        *ngc = cell;
    }
}
unsafe fn screen_redraw_draw_border_arrows(
    ctx: &screen_redraw_ctx,
    i: core::ffi::c_int,
    j: core::ffi::c_int,
    cell_type: u_int,
    pane: Option<&RustWindowPaneWeak>,
    active_pane: &RustWindowPaneWeak,
    gc: &mut grid_cell,
) {
    unsafe {
        let Some(window) = screen_redraw_current_window(ctx) else {
            return;
        };
        let w = window.as_window();
        let oo = (*w).options_ref();
        let x: u_int = (ctx.ox + i) as u_int;
        let y: u_int = (ctx.oy + j) as u_int;

        let mut arrows: core::ffi::c_int = 0 as core::ffi::c_int;
        let mut border: core::ffi::c_int;
        let Some(pane) = pane else {
            return;
        };
        let Some(pane_window) = pane.window() else {
            return;
        };
        let (Some(wp), Some(active)) = (pane.get(), active_pane.get()) else {
            return;
        };
        if i != wp.geometry().xoff + 1 as core::ffi::c_int
            && j != wp.geometry().yoff + 1 as core::ffi::c_int
        {
            return;
        }
        if !pane.ptr_eq(active_pane) {
            if active_pane.window().is_some_and(|window| {
                window_pane_is_floating(&window.as_window(), active_pane) != 0
            }) {
                return;
            }
            if window_pane_is_floating(&pane_window.as_window(), pane) != 0 {
                return;
            }
        }
        let value: core::ffi::c_int = (oo).number(c"pane-border-indicators") as core::ffi::c_int;
        if value != PANE_BORDER_ARROWS && value != PANE_BORDER_BOTH {
            return;
        }
        border =
            screen_redraw_pane_border(ctx, active, x as core::ffi::c_int, y as core::ffi::c_int)
                as core::ffi::c_int;
        if border == SCREEN_REDRAW_INSIDE as core::ffi::c_int {
            return;
        }
        if i == wp.geometry().xoff + 1 as core::ffi::c_int {
            if border == SCREEN_REDRAW_OUTSIDE as core::ffi::c_int {
                if screen_redraw_two_panes(&pane_window).is_some() {
                    if w.panes
                        .first()
                        .is_some_and(|pane| pane.id() == active.pane_id())
                    {
                        border = SCREEN_REDRAW_BORDER_BOTTOM as core::ffi::c_int;
                    } else {
                        border = SCREEN_REDRAW_BORDER_TOP as core::ffi::c_int;
                    }
                    arrows = 1 as core::ffi::c_int;
                }
            } else if cell_type == CELL_LEFTRIGHT as u_int
                || (cell_type == CELL_TOPJOIN as u_int
                    && border == SCREEN_REDRAW_BORDER_BOTTOM as core::ffi::c_int)
                || (cell_type == CELL_BOTTOMJOIN as u_int
                    && border == SCREEN_REDRAW_BORDER_TOP as core::ffi::c_int)
            {
                arrows = 1 as core::ffi::c_int;
            }
        }
        if j == wp.geometry().yoff + 1 as core::ffi::c_int {
            if border == SCREEN_REDRAW_OUTSIDE as core::ffi::c_int {
                if screen_redraw_two_panes(&pane_window).is_some() {
                    if w.panes
                        .first()
                        .is_some_and(|pane| pane.id() == active.pane_id())
                    {
                        border = SCREEN_REDRAW_BORDER_RIGHT as core::ffi::c_int;
                    } else {
                        border = SCREEN_REDRAW_BORDER_LEFT as core::ffi::c_int;
                    }
                    arrows = 1 as core::ffi::c_int;
                }
            } else if cell_type == CELL_TOPBOTTOM as u_int
                || (cell_type == CELL_LEFTJOIN as u_int
                    && border == SCREEN_REDRAW_BORDER_RIGHT as core::ffi::c_int)
                || (cell_type == CELL_RIGHTJOIN as u_int
                    && border == SCREEN_REDRAW_BORDER_LEFT as core::ffi::c_int)
            {
                arrows = 1 as core::ffi::c_int;
            }
        }
        if arrows != 0 {
            gc.attr = (gc.attr as core::ffi::c_int | GRID_ATTR_CHARSET) as u_short;
            utf8_set(&mut gc.data, BORDER_MARKERS[border as usize]);
        }
    }
}
unsafe fn screen_redraw_draw_borders_cell(ctx: &mut screen_redraw_ctx, cache: &mut BorderPassCache, i: u_int, j: u_int) {
    unsafe {
        let mut client = ctx.c.clone().expect("the redraw context has a client");
        let Some(session) = client.attached_session() else {
            return;
        };
        let Some(window) = session.current_window() else {
            return;
        };
        let active = server_client_get_pane(client.as_client());
        let mut gc = grid_default_cell;
        let x = (ctx.ox as u_int).wrapping_add(i);
        let y = (ctx.oy as u_int).wrapping_add(j);
        let check = client.overlay_check();
        if check.is_some() {
            let data = client.current_overlay_data();
            let ranges = check.call(data, x, y, 1);
            if ranges.is_empty() {
                return;
            }
        }
        let checked = screen_redraw_check_cell(ctx, x as core::ffi::c_int, y as core::ffi::c_int);
        let cell_type = checked.cell_type as u_int;
        let mut pane = checked.pane_ref;
        if cell_type == CELL_INSIDE as u_int || cell_type == CELL_SCROLLBAR as u_int {
            return;
        }
        if let Some(pane) = pane.as_mut()
            && cell_type != CELL_OUTSIDE as u_int
        {
            screen_redraw_draw_borders_style(ctx, cache, x, y, pane, &mut gc);
            let marked = marked_pane.get().pane_ref();
            if server_is_marked(
                Some(session.as_session()),
                session
                    .curw()
                    .as_ref()
                    .and_then(crate::window::WinlinkRef::get),
                marked.as_ref().and_then(|pane| pane.get()),
            ) != 0
                && screen_redraw_check_is(
                    ctx,
                    x as core::ffi::c_int,
                    y as core::ffi::c_int,
                    marked.as_ref().and_then(|pane| pane.get()),
                ) != 0
            {
                gc.attr ^= GRID_ATTR_REVERSE as u_short;
            }
        } else {
            if ctx.no_pane_gc_set == 0 {
                let options = window.options();
                let mut ft = format_create_defaults(
                    None,
                    Some(client.as_client()),
                    Some(session.as_session()),
                    session
                        .curw()
                        .as_ref()
                        .and_then(crate::window::WinlinkRef::get),
                    None::<&dyn crate::WindowPane>,
                );
                ctx.no_pane_gc = grid_default_cell;
                style_add(
                    &mut ctx.no_pane_gc,
                    &options,
                    c"pane-border-style",
                    Some(&mut ft),
                );
                ctx.no_pane_gc_set = 1;
            }
            gc = ctx.no_pane_gc;
        }
        screen_redraw_border_set(
            &window,
            pane.as_ref().and_then(|pane| pane.get()),
            ctx.pane_lines,
            cell_type as core::ffi::c_int,
            &mut gc,
        );
        let isolates = cell_type == CELL_TOPBOTTOM as u_int
            && client.flags() & CLIENT_UTF8 as uint64_t != 0
            && tty_term_of(client.as_tty()).has(TTYC_BIDI);
        {
            let tty = client.as_tty_mut();
            if ctx.statustop != 0 {
                tty_cursor(tty, i, ctx.statuslines.wrapping_add(j));
            } else {
                tty_cursor(tty, i, j);
            }
            if isolates {
                tty_puts(tty, END_ISOLATE);
            }
        }
        if let Some(active) = active.as_ref() {
            screen_redraw_draw_border_arrows(
                ctx,
                i as core::ffi::c_int,
                j as core::ffi::c_int,
                cell_type,
                pane.as_ref(),
                active,
                &mut gc,
            );
        }
        let tty = client.as_tty_mut();
        tty_cell(tty, &gc, &grid_default_cell, None, None);
        if isolates {
            tty_puts(tty, START_ISOLATE);
        }
    }
}
unsafe fn screen_redraw_draw_borders(ctx: &mut screen_redraw_ctx) {
    unsafe {
        let client = ctx.c.clone().expect("the redraw context has a client");
        let Some(window) = screen_redraw_current_window(ctx) else {
            return;
        };
        log_debug(
            c"%s: %s @%u",
            fmt_args![
                c"screen_redraw_draw_borders",
                client.name(),
                window.window_id()
            ],
        );
        let mut cache = BorderPassCache::default();
        let mut j = 0;
        while j < client.as_tty().sy.wrapping_sub(ctx.statuslines) {
            let mut i = 0;
            while i < client.as_tty().sx {
                screen_redraw_draw_borders_cell(ctx, &mut cache, i, j);
                i = i.wrapping_add(1);
            }
            j = j.wrapping_add(1);
        }
    }
}
unsafe fn screen_redraw_draw_panes(ctx: &mut screen_redraw_ctx) {
    unsafe {
        let Some(window) = screen_redraw_current_window(ctx) else {
            return;
        };
        let client = ctx.c.as_ref().expect("the redraw context has a client");
        log_debug(
            c"%s: %s @%u",
            fmt_args![
                c"screen_redraw_draw_panes",
                client.name(),
                window.window_id()
            ],
        );
        let panes = window.panes();
        for mut pane in panes {
            if pane
                .get()
                .is_some_and(|wp| window_pane_visible(&window.as_window(), wp) != 0)
            {
                screen_redraw_draw_pane(ctx, &mut pane);
            }
        }
    }
}
unsafe fn screen_redraw_draw_status(ctx: &mut screen_redraw_ctx) {
    unsafe {
        let mut client = ctx.c.clone().expect("the redraw context has a client");
        let Some(window) = screen_redraw_current_window(ctx) else {
            return;
        };
        let c = client.as_client_mut();
        let mut i: u_int = 0;

        log_debug(
            c"%s: %s @%u",
            fmt_args![
                c"screen_redraw_draw_status",
                c.name.as_deref(),
                window.window_id()
            ],
        );
        let y: u_int = if ctx.statustop != 0 {
            0 as u_int
        } else {
            c.tty.sy.wrapping_sub(ctx.statuslines)
        };
        let tty = &mut c.tty;
        c.status.with_active(|s| {
            i = 0 as u_int;
            while i < ctx.statuslines {
                tty_draw_line(
                    tty,
                    s,
                    0 as u_int,
                    i,
                    UINT_MAX,
                    0 as u_int,
                    y.wrapping_add(i),
                    &grid_default_cell,
                    None,
                );
                i = i.wrapping_add(1);
            }
        });
    }
}
/// Whether the column `px` shows through the ranges `r`. No ranges at all
/// means nothing hides it; otherwise some range of real width has to cover
/// it.
pub fn screen_redraw_is_visible(r: Option<&visible_ranges>, px: u_int) -> bool {
    let Some(r) = r else {
        return true;
    };
    r.ranges
        .iter()
        .take(r.used as usize)
        .any(|ri| ri.nx != 0 as u_int && px >= ri.px && px < ri.px.wrapping_add(ri.nx))
}

unsafe fn screen_redraw_draw_pane(ctx: &mut screen_redraw_ctx, pane: &mut RustWindowPaneWeak) {
    unsafe {
        let mut client = ctx.c.clone().expect("the redraw context has a client");
        let Some(window) = screen_redraw_current_window(ctx) else {
            return;
        };
        let Some(wp) = pane.get() else { return };
        if wp.base().0.mode & MODE_SYNC != 0 {
            if let Some(wp) = pane.get_mut() { wp.stop_sync(); }
        }
        log_debug(
            c"%s: %s @%u %%%u",
            fmt_args![
                c"screen_redraw_draw_pane",
                client.name(),
                window.window_id(),
                pane.id()
            ],
        );
        let Some(geometry) = pane.get().map(|wp| wp.geometry()) else {
            return;
        };
        if geometry.xoff + geometry.sx as core::ffi::c_int <= ctx.ox
            || geometry.xoff >= ctx.ox + ctx.sx as core::ffi::c_int
        {
            return;
        }
        let woy = if ctx.statustop != 0 {
            ctx.statuslines
        } else {
            0
        };
        let mut j = 0;
        loop {
            let Some(geometry) = pane.get().map(|wp| wp.geometry()) else {
                return;
            };
            if j >= geometry.sy {
                break;
            }
            let row = geometry.yoff + j as core::ffi::c_int;
            if row < ctx.oy || row >= ctx.oy + ctx.sy as core::ffi::c_int {
                j += 1;
                continue;
            }
            let wy = (geometry.yoff as u_int).wrapping_add(j);
            let py = woy.wrapping_add(wy).wrapping_sub(ctx.oy as u_int);
            if py > client.as_tty().sy {
                j += 1;
                continue;
            }
            let (wx, width) = if geometry.xoff >= ctx.ox
                && geometry.xoff + geometry.sx as core::ffi::c_int
                    <= ctx.ox + ctx.sx as core::ffi::c_int
            {
                ((geometry.xoff - ctx.ox) as u_int, geometry.sx)
            } else if geometry.xoff < ctx.ox
                && geometry.xoff + geometry.sx as core::ffi::c_int
                    > ctx.ox + ctx.sx as core::ffi::c_int
            {
                (0, ctx.sx)
            } else if geometry.xoff < ctx.ox {
                (
                    0,
                    geometry.sx.wrapping_sub((ctx.ox - geometry.xoff) as u_int),
                )
            } else {
                let wx = (geometry.xoff - ctx.ox) as u_int;
                (wx, ctx.sx.wrapping_sub(wx))
            };
            let ranges = tty_check_overlay_range(client.as_tty_mut(), wx, wy, width);
            let Some(wp) = pane.get() else { return };
            ranges.clip_visible_ranges(
                Some(wp),
                wx as core::ffi::c_int,
                wy as core::ffi::c_int,
                width,
            );
            let mut defaults = grid_default_cell;
            let Some(wp) = pane.get_mut() else { return };
            tty_default_colours(&mut defaults, wp);
            let mut k = 0;
            while let Some(range) = ranges.range_at(k) {
                if range.nx != 0 {
                    let Some(wp) = pane.get() else { return };
                    let x = range
                        .px
                        .wrapping_add(ctx.ox as u_int)
                        .wrapping_sub(wp.geometry().xoff as u_int);
                    log_debug(
                        c"%s: %s %%%u range %u (%u,%u) width %u, tty (%u,%u) width %u",
                        fmt_args![
                            c"screen_redraw_draw_pane",
                            client.name(),
                            pane.id(),
                            k,
                            x,
                            j,
                            range.nx,
                            range.px,
                            py,
                            range.nx
                        ],
                    );
                    tty_draw_line(
                        client.as_tty_mut(),
                        &wp.screen_ref(),
                        x,
                        j,
                        range.nx,
                        range.px,
                        py,
                        &defaults,
                        Some(&wp.palette_snapshot()),
                    );
                }
                k += 1;
            }
            j += 1;
        }
    }
}
unsafe fn screen_redraw_draw_pane_scrollbars(ctx: &mut screen_redraw_ctx) {
    unsafe {
        let Some(window) = screen_redraw_current_window(ctx) else {
            return;
        };
        let client = ctx.c.as_ref().expect("the redraw context has a client");
        log_debug(
            c"%s: %s @%u",
            fmt_args![
                c"screen_redraw_draw_pane_scrollbars",
                client.name(),
                window.window_id()
            ],
        );
        let panes = window.panes();
        for mut pane in panes {
            let Some(wp) = pane.get() else { continue };
            if window_pane_show_scrollbar(wp, window.scrollbar_settings().sb) != 0
                && window_pane_visible(&window.as_window(), wp) != 0
            {
                screen_redraw_draw_pane_scrollbar(ctx, &mut pane);
            }
        }
    }
}
unsafe fn screen_redraw_draw_pane_scrollbar(
    ctx: &mut screen_redraw_ctx,
    pane: &mut RustWindowPaneWeak,
) {
    unsafe {
        let Some(wp) = pane.get() else { return };
        let Some(window) = pane.window() else { return };
        let settings = window.scrollbar_settings();
        let percent_view: core::ffi::c_double;
        let sb: u_int = settings.sb as u_int;
        let total_height: u_int;
        let sb_h: u_int = (*wp).geometry().sy;
        let sb_pos: u_int = settings.sb_pos as u_int;
        let mut slider_h: u_int;
        let mut slider_y: u_int;
        let sb_w: core::ffi::c_int = (*wp).scrollbar_style().width;
        let sb_pad: core::ffi::c_int = (*wp).scrollbar_style().padding;
        let cm_y: core::ffi::c_int;
        let cm_size: core::ffi::c_int;
        let xoff: core::ffi::c_int = (*wp).geometry().xoff;

        let sb_y: core::ffi::c_int = (*wp).geometry().yoff;
        if window_pane_mode(wp) == WINDOW_PANE_NO_MODE {
            if sb == PANE_SCROLLBARS_MODAL as u_int {
                return;
            }
            let screen = wp.screen_ref();
            total_height = RustScreen::grid(&screen)
                .sy
                .wrapping_add(RustScreen::grid(&screen).hsize);
            percent_view = sb_h as core::ffi::c_double / total_height as core::ffi::c_double;
            slider_h = (sb_h as core::ffi::c_double * percent_view) as u_int;
            slider_y = sb_h.wrapping_sub(slider_h);
        } else {
            if wp.active_mode().is_none() {
                return;
            }
            let Some((offset, size)) = window_copy_get_current_offset(wp) else {
                return;
            };
            (cm_y, cm_size) = (offset as core::ffi::c_int, size as core::ffi::c_int);
            total_height = (cm_size as u_int).wrapping_add(sb_h);
            percent_view = sb_h as core::ffi::c_double
                / (cm_size as u_int).wrapping_add(sb_h) as core::ffi::c_double;
            slider_h = (sb_h as core::ffi::c_double * percent_view) as u_int;
            slider_y = (sb_h.wrapping_add(1 as u_int) as core::ffi::c_double
                * (cm_y as core::ffi::c_double / total_height as core::ffi::c_double))
                as u_int;
        }
        let sb_x: core::ffi::c_int = if sb_pos == PANE_SCROLLBARS_LEFT as u_int {
            xoff - sb_w - sb_pad
        } else {
            (xoff as u_int).wrapping_add((*wp).geometry().sx) as core::ffi::c_int
        };
        if slider_h < 1 as u_int {
            slider_h = 1 as u_int;
        }
        if slider_y >= sb_h {
            slider_y = sb_h.wrapping_sub(1 as u_int);
        }
        screen_redraw_draw_scrollbar(
            ctx,
            pane,
            sb_pos as core::ffi::c_int,
            sb_x,
            sb_y,
            sb_h,
            slider_h,
            slider_y,
        );
        if let Some(wp) = pane.get_mut() {
            wp.publish_slider(PaneScrollbarSlider {
                sb_slider_y: slider_y,
                sb_slider_h: slider_h,
            });
        }
    }
}
#[allow(clippy::too_many_arguments)]
unsafe fn screen_redraw_draw_scrollbar(
    ctx: &mut screen_redraw_ctx,
    pane: &RustWindowPaneWeak,
    sb_pos: core::ffi::c_int,
    sb_x: core::ffi::c_int,
    mut sb_y: core::ffi::c_int,
    sb_h: u_int,
    slider_h: u_int,
    slider_y: u_int,
) {
    unsafe {
        let mut client = ctx.c.clone().expect("the redraw context has a client");
        let Some((geometry, sb_style)) = pane.get().map(|wp| (wp.geometry(), wp.scrollbar_style()))
        else {
            return;
        };

        let mut slgc;
        let mut i: u_int;
        let mut j: u_int;
        let mut imin: u_int = 0 as u_int;
        let mut jmin: u_int = 0 as u_int;
        let mut imax: u_int;
        let mut jmax: u_int;
        let sb_w: u_int = sb_style.width as u_int;
        let sb_pad: u_int = sb_style.padding as u_int;
        let mut px: core::ffi::c_int;
        let mut py: core::ffi::c_int;
        let mut wx: core::ffi::c_int;
        let mut wy: core::ffi::c_int;

        let mut sy: core::ffi::c_int;

        let xoff: core::ffi::c_int = geometry.xoff;
        let yoff: core::ffi::c_int = geometry.yoff;
        let sb_wy: core::ffi::c_int = sb_y;
        let sx: core::ffi::c_int = ctx.sx as core::ffi::c_int;
        sy = client.as_tty().sy.wrapping_sub(ctx.statuslines) as core::ffi::c_int;
        let ox: core::ffi::c_int = ctx.ox;
        let oy: core::ffi::c_int = ctx.oy;
        if ctx.statustop != 0 {
            sb_y = (sb_y as u_int).wrapping_add(ctx.statuslines) as core::ffi::c_int
                as core::ffi::c_int;
            sy =
                (sy as u_int).wrapping_add(ctx.statuslines) as core::ffi::c_int as core::ffi::c_int;
        }
        let gc = sb_style.cell;
        slgc = gc;
        slgc.fg = gc.bg;
        slgc.bg = gc.fg;
        if (sb_x + sb_w as core::ffi::c_int) < 0 as core::ffi::c_int || sb_x >= sx || sb_y >= sy {
            return;
        }
        if sb_x < 0 as core::ffi::c_int {
            imin = -sb_x as u_int;
        }
        imax = sb_w.wrapping_add(sb_pad);
        if imax as core::ffi::c_int + sb_x > sx {
            if sb_x > sx {
                return;
            }
            imax = (sx - sb_x) as u_int;
        }
        jmax = sb_h;
        if jmax as core::ffi::c_int + sb_y > sy && sb_y >= sy {
            return;
        }
        let sb_tty_y: core::ffi::c_int = sb_y - oy;
        if sb_tty_y > sy {
            return;
        }
        if sb_tty_y < 0 as core::ffi::c_int {
            jmin = -sb_tty_y as u_int;
        }
        if sb_tty_y + sb_h as core::ffi::c_int <= 0 as core::ffi::c_int {
            return;
        }
        jmax = sb_h;
        if sb_tty_y + jmax as core::ffi::c_int > sy {
            jmax = (sy - sb_tty_y) as u_int;
        }
        j = jmin;
        while j < jmax {
            wy = (sb_wy as u_int).wrapping_add(j) as core::ffi::c_int;
            py = (sb_tty_y as u_int).wrapping_add(j) as core::ffi::c_int;
            let r = tty_check_overlay_range(client.as_tty_mut(), sb_x as u_int, wy as u_int, imax);
            let Some(wp) = pane.get() else { return };
            r.clip_visible_ranges(Some(wp), sb_x, wy, imax);
            i = imin;
            while i < imax {
                px = ((sb_x + ox) as u_int).wrapping_add(i) as core::ffi::c_int;
                wx = (sb_x as u_int).wrapping_add(i) as core::ffi::c_int;
                if !(wx < xoff - sb_w as core::ffi::c_int - sb_pad as core::ffi::c_int
                    || px >= sx
                    || px < 0 as core::ffi::c_int
                    || wy < yoff - 1 as core::ffi::c_int
                    || py >= sy
                    || py < 0 as core::ffi::c_int
                    || !screen_redraw_is_visible(Some(&r.borrow()), wx as u_int))
                {
                    tty_cursor(client.as_tty_mut(), px as u_int, py as u_int);
                    if sb_pos == PANE_SCROLLBARS_LEFT && i >= sb_w && i < sb_w.wrapping_add(sb_pad)
                        || sb_pos == PANE_SCROLLBARS_RIGHT && i < sb_pad
                    {
                        tty_cell(
                            client.as_tty_mut(),
                            &grid_default_cell,
                            &grid_default_cell,
                            None,
                            None,
                        );
                    } else {
                        let gcp = if j >= slider_y && j < slider_y.wrapping_add(slider_h) {
                            &slgc
                        } else {
                            &gc
                        };
                        tty_cell(client.as_tty_mut(), gcp, &grid_default_cell, None, None);
                    }
                }
                i = i.wrapping_add(1);
            }
            j = j.wrapping_add(1);
        }
    }
}

#[cfg(test)]
#[path = "redraw_focused_tests.rs"]
mod focused_tests;

impl visible_ranges {
    /// The ranges of the `width` cells at `px`,`py` of `base_wp` that no pane
    /// in front of it covers, written into `r`.
    pub unsafe fn set_visible_ranges(
        &mut self,
        base_wp: Option<&(impl crate::WindowPane + ?Sized)>,
        px: core::ffi::c_int,
        py: core::ffi::c_int,
        width: u_int,
    ) {
        let r = self;

        unsafe { r.visible_ranges(base_wp, px, py, width, true) }
    }
    /// Narrows the ranges `r` already holds to what no pane in front of
    /// `base_wp` covers. The caller has already worked out which cells of the
    /// `width` at `px`,`py` an overlay leaves it.
    pub unsafe fn clip_visible_ranges(
        &mut self,
        base_wp: Option<&(impl crate::WindowPane + ?Sized)>,
        px: core::ffi::c_int,
        py: core::ffi::c_int,
        width: u_int,
    ) {
        let r = self;

        unsafe { r.visible_ranges(base_wp, px, py, width, false) }
    }
    /// The two of them: `seed` starts from the whole span asked for, and
    /// otherwise the ranges the caller handed in are the ones to narrow.
    unsafe fn visible_ranges(
        &mut self,
        base_wp: Option<&(impl crate::WindowPane + ?Sized)>,
        mut px: core::ffi::c_int,
        py: core::ffi::c_int,
        mut width: u_int,
        seed: bool,
    ) {
        let out = self;

        unsafe {
            let r = out;
            let mut found_self: core::ffi::c_int;
            let mut sb_w: core::ffi::c_int;
            let mut lb: core::ffi::c_int;
            let mut rb: core::ffi::c_int;
            let mut tb: core::ffi::c_int;
            let mut bb: core::ffi::c_int;
            let mut sx: core::ffi::c_int;
            let mut ex: core::ffi::c_int;
            let mut i: u_int;
            if py < 0 as core::ffi::c_int || width == 0 as u_int {
                r.used = 0 as u_int;
                return;
            }
            if px < 0 as core::ffi::c_int {
                if -px as u_int >= width {
                    r.used = 0 as u_int;
                    return;
                }
                width = width.wrapping_sub(-px as u_int);
                px = 0 as core::ffi::c_int;
            }
            let Some(base_wp) = base_wp else {
                if seed {
                    r.ensure_capacity(1 as u_int);
                    r.ranges[0_usize].px = px as u_int;
                    r.ranges[0_usize].nx = width;
                    r.used = 1 as u_int;
                }
                return;
            };
            let window = base_wp
                .window_context()
                .expect("a rendered pane has a window context");
            let w = window.as_window();
            if py as u_int >= w.dimensions().size.height {
                r.used = 0 as u_int;
                return;
            }
            if (px as u_int).wrapping_add(width) > w.dimensions().size.width {
                width = w.dimensions().size.width.wrapping_sub(px as u_int);
            }
            if seed {
                r.ensure_capacity(1 as u_int);
                r.ranges[0_usize].px = px as u_int;
                r.ranges[0_usize].nx = width;
                r.used = 1 as u_int;
            }
            found_self = 0 as core::ffi::c_int;
            for wp in w.z_index.iter().rev().take_while(|pane| pane.is_alive()) {
                let wp = wp.as_pane();
                if core::ptr::addr_eq(wp, base_wp) {
                    found_self = 1 as core::ffi::c_int;
                } else {
                    tb = if wp.geometry().yoff > 0 as core::ffi::c_int {
                        wp.geometry().yoff - 1 as core::ffi::c_int
                    } else {
                        0 as core::ffi::c_int
                    };
                    bb = (wp.geometry().yoff as u_int).wrapping_add(wp.geometry().sy)
                        as core::ffi::c_int;
                    if !(found_self == 0 || window_pane_visible(&w, wp) == 0 || py < tb || py > bb)
                        && !(window_pane_is_floating(
                            &w,
                            &{ (wp).observation() }
                                .expect("the pane allocation exists"),
                        ) == 0
                            && (py == tb || py == bb))
                    {
                        sb_w = wp.scrollbar_style().width + wp.scrollbar_style().padding;
                        if window_pane_show_scrollbar(wp, w.scrollbar_settings().sb) == 0 {
                            sb_w = 0 as core::ffi::c_int;
                        }
                        i = 0 as u_int;
                        while i < r.used {
                            let range = r.ranges[i as usize];
                            if w.scrollbar_settings().sb_pos == PANE_SCROLLBARS_LEFT {
                                if wp.geometry().xoff > sb_w {
                                    lb = wp.geometry().xoff - 1 as core::ffi::c_int - sb_w;
                                } else {
                                    lb = 0 as core::ffi::c_int;
                                }
                            } else if wp.geometry().xoff > 0 as core::ffi::c_int {
                                lb = wp.geometry().xoff - 1 as core::ffi::c_int;
                            } else {
                                lb = 0 as core::ffi::c_int;
                            }
                            if w.scrollbar_settings().sb_pos == PANE_SCROLLBARS_LEFT {
                                rb = (wp.geometry().xoff as u_int).wrapping_add(wp.geometry().sx)
                                    as core::ffi::c_int;
                            } else {
                                rb = (wp.geometry().xoff as u_int)
                                    .wrapping_add(wp.geometry().sx)
                                    .wrapping_add(sb_w as u_int)
                                    as core::ffi::c_int;
                            }
                            if rb > w.dimensions().size.width as core::ffi::c_int {
                                rb = w.dimensions().size.width.wrapping_sub(1 as u_int)
                                    as core::ffi::c_int;
                            }
                            sx = range.px as core::ffi::c_int;
                            ex = (sx as u_int)
                                .wrapping_add(range.nx)
                                .wrapping_sub(1 as u_int)
                                as core::ffi::c_int;
                            if lb > sx && lb <= ex && rb > ex {
                                r.ranges[i as usize].nx = (lb - sx) as u_int;
                            } else if rb >= sx && rb <= ex && lb <= sx {
                                r.ranges[i as usize].nx = (ex - rb) as u_int;
                                r.ranges[i as usize].px = (rb + 1 as core::ffi::c_int) as u_int;
                            } else if lb > sx && rb <= ex {
                                let used = r.used as usize;
                                r.ensure_capacity(r.used.wrapping_add(1));
                                r.ranges.copy_within(i as usize..used, i as usize + 1);
                                r.ranges[i as usize + 1] = visible_range {
                                    px: (rb + 1 as core::ffi::c_int) as u_int,
                                    nx: (ex - rb) as u_int,
                                };
                                r.ranges[i as usize].nx = (lb - sx) as u_int;
                                r.used = r.used.wrapping_add(1);
                            } else if lb <= sx && rb > ex {
                                r.ranges[i as usize].nx = 0 as u_int;
                            }
                            i = i.wrapping_add(1);
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
pub use crate::consts::{
    CELL_BOTTOMLEFT, CELL_BOTTOMRIGHT, CELL_TOPLEFT, CELL_TOPRIGHT, PANE_LINES_SINGLE,
};

#[cfg(test)]
pub const CELL_JOIN: core::ffi::c_int = 11 as core::ffi::c_int;

#[cfg(test)]
pub use crate::consts::{CLIENT_REDRAWPANES, CLIENT_REDRAWSCROLLBARS};
