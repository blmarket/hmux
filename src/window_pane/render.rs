//! Pane rendering operations. Model access ends before terminal or format callbacks.
use super::*;
use crate::src::format::{format_create, format_defaults, format_free};
use crate::src::grid::grid_default_cell;
use crate::src::prompt::prompt_draw;
use crate::src::screen_redraw::{redraw_draw_ctx, REDRAW_SCROLLBAR_LEFT, REDRAW_STATUS_TOP};
use crate::src::screen_write::{screen_write_init, screen_write_start, screen_write_stop};
use crate::src::server_client::Client as _;
use crate::src::shared::client::ClientRef;
use crate::src::shared::format::{FORMAT_NOJOBS, FORMAT_PANE};
use crate::src::shared::prompt::prompt_draw_data;
use crate::src::shared::redraw::redraw_span;
use crate::src::shared::screen_write::screen_write_ctx;
use crate::src::shared::style::style;
use crate::src::shared::tty::tty_style_ctx;
use crate::src::tty::{tty_cell, tty_cursor};
use crate::src::tty_draw::tty_draw_line;
use crate::src::window::Window as _;
use crate::src::window_copy::window_copy_get_current_offset;

#[derive(Clone, Copy)]
pub struct PaneScrollbar {
    pub visible: bool,
    pub overlay: bool,
    pub reserved: bool,
    pub width: i32,
    pub pad: i32,
    pub slider_y: u32,
    pub slider_height: u32,
}

pub(super) unsafe fn scrollbar(pane: &Rc<UnsafeCell<window_pane>>) -> PaneScrollbar {
    let state = &*pane.get();
    PaneScrollbar {
        visible: window_pane_scrollbar_visible(state) != 0,
        overlay: window_pane_scrollbar_overlay(state) != 0,
        reserved: window_pane_scrollbar_reserve(state) != 0,
        width: state.scrollbar_style.width,
        pad: state.scrollbar_style.pad,
        slider_y: state.sb_slider_y,
        slider_height: state.sb_slider_h,
    }
}

pub(super) unsafe fn refresh_scrollbar_style(pane: &Rc<UnsafeCell<window_pane>>) {
    let mut updated = (*pane.get()).scrollbar_style;
    pane.with_options_mut(|options| style_set_scrollbar_style_from_option(&mut updated, options));
    (*pane.get()).scrollbar_style = updated;
}

/// Resolve the selected mode after releasing pane storage. The mode stack keeps
/// its screen alive until the bounded drawing or write operation finishes.
unsafe fn displayed_screen(pane: &Rc<UnsafeCell<window_pane>>) -> *mut screen {
    let mode = match &(*pane.get()).screen_source {
        PaneScreenSource::Base => return &raw mut (*pane.get()).base,
        PaneScreenSource::Mode(observer) => observer.clone(),
    };
    if !mode.is_alive() {
        return std::ptr::null_mut();
    }
    let display = mode
        .get_unchecked()
        .mode
        .display_screen
        .expect("mode display screen getter");
    display(mode)
}

pub(super) unsafe fn prepare_write(
    pane: &Rc<UnsafeCell<window_pane>>,
    context: &mut screen_write_ctx,
    requested: *mut screen,
) {
    let screen = if requested.is_null() {
        displayed_screen(pane)
    } else {
        requested
    };
    screen_write_init(context, screen);
    context.wp = Rc::downgrade(pane);
}

pub(super) unsafe fn is_obscured(pane: &Rc<UnsafeCell<window_pane>>) -> bool {
    let (sx, sy, xoff, yoff) = pane.geometry();
    let window = pane.window_observer().upgrade().expect("live pane parent");
    let (wsx, wsy) = window.size();
    let outside = xoff < 0
        || yoff < 0
        || (xoff as u32).wrapping_add(sx) > wsx
        || (yoff as u32).wrapping_add(sy) > wsy;
    if outside {
        window.release(c"pane obscured outside window");
        return true;
    }
    let panes = window.stacking_snapshot();
    window.release(c"pane obscured");
    for other in panes
        .into_iter()
        .rev()
        .skip_while(|other| !Rc::ptr_eq(other, pane))
        .skip(1)
    {
        let (other_sx, other_sy, other_xoff, other_yoff) = other.geometry();
        if other.is_floating()
            && (other_yoff >= yoff && other_yoff <= yoff + sy as i32
                || other_yoff + other_sy as i32 >= yoff
                    && (other_yoff as u32).wrapping_add(other_sy) <= (yoff as u32).wrapping_add(sy))
            && (other_xoff >= xoff && other_xoff <= xoff + sx as i32
                || other_xoff + other_sx as i32 >= xoff
                    && (other_xoff as u32).wrapping_add(other_sx) <= (xoff as u32).wrapping_add(sx))
        {
            return true;
        }
    }
    false
}

pub(super) unsafe fn alternate_screen_changed(pane: &Rc<UnsafeCell<window_pane>>, entered: bool) {
    if entered {
        window_pane_clear_resizes(&mut *pane.get(), std::ptr::null_mut());
        drop((*pane.get()).resize_timer.take());
    }
    let window = pane.window_observer().upgrade().expect("live pane parent");
    layout_fix_panes(&window, None);
    if entered && !(*pane.get()).resize_queue.is_empty() {
        let (sx, sy, _, _) = pane.geometry();
        window_pane_send_resize(&*pane.get(), sx, sy);
        window_pane_clear_resizes(&mut *pane.get(), std::ptr::null_mut());
    }
    server_redraw_window_borders(&window);
    window.release(c"pane alternate screen");
}

fn apply_style(cell: &mut grid_cell, parsed: style) {
    if parsed.gc.fg != 8 {
        cell.fg = parsed.gc.fg;
    }
    if parsed.gc.bg != 8 {
        cell.bg = parsed.gc.bg;
    }
    if parsed.gc.us != 8 {
        cell.us = parsed.gc.us;
    }
    cell.attr |= parsed.gc.attr;
}

pub(super) unsafe fn default_colours(pane: &Rc<UnsafeCell<window_pane>>) -> (grid_cell, u32) {
    if (*pane.get()).flags & PANE_STYLECHANGED != 0 {
        log_debug(format_args!("%{}: style changed", pane.id()));
        (*pane.get()).flags &= !PANE_STYLECHANGED;
        let mut context =
            format_create(None, None, (FORMAT_PANE | pane.id()) as i32, FORMAT_NOJOBS);
        format_defaults(&mut *context, None, None, refbox::Weak::new(), Some(pane));
        for (key, active) in [(c"window-active-style", true), (c"window-style", false)] {
            let (fg, bg) = {
                let palette = pane.borrow_palette();
                (palette.fg, palette.bg)
            };
            let mut cell = grid_cell {
                fg,
                bg,
                ..grid_default_cell
            };
            {
                let state = &mut *pane.get();
                if active {
                    state.cached_active_gc = cell;
                } else {
                    state.cached_gc = cell;
                }
            }
            let parsed =
                crate::src::style::style_resolve_with_options(key, Some(&mut context), |visit| {
                    pane.with_options_mut(|options| visit(options));
                })
                .unwrap_or_else(|| {
                    let mut style = style::default();
                    crate::src::style::style_set(&mut style, &grid_default_cell);
                    style
                });
            apply_style(&mut cell, parsed);
            let state = &mut *pane.get();
            if active {
                state.cached_active_gc = cell;
                state.cached_active_dim = parsed.dim as u32;
            } else {
                state.cached_gc = cell;
                state.cached_dim = parsed.dim as u32;
            }
        }
        format_free(context);
    }
    let window = pane.window_observer().upgrade().expect("live pane parent");
    let active = window
        .active_pane()
        .is_some_and(|owner| Rc::ptr_eq(&owner, pane));
    window.release(c"pane default colours");
    let state = &*pane.get();
    let mut cell = grid_default_cell;
    cell.fg = if active && state.cached_active_gc.fg != 8 {
        state.cached_active_gc.fg
    } else {
        state.cached_gc.fg
    };
    cell.bg = if active && state.cached_active_gc.bg != 8 {
        state.cached_active_gc.bg
    } else {
        state.cached_gc.bg
    };
    (
        cell,
        if active {
            state.cached_active_dim
        } else {
            state.cached_dim
        },
    )
}

pub(super) unsafe fn draw_line(
    pane: &Rc<UnsafeCell<window_pane>>,
    client: &ClientRef,
    source: (u32, u32),
    width: u32,
    destination: (u32, u32),
    status: bool,
) {
    let (defaults, dim) = if status {
        (grid_default_cell, 0)
    } else {
        pane.default_colours()
    };
    // The selected screen and its grid remain explicitly owned by this operation.
    // Terminal drawing may inspect the pane palette after pane storage is released.
    let target = if status {
        &raw mut (*pane.get()).status_screen
    } else {
        displayed_screen(pane)
    };
    let selected = std::mem::take(&mut *target);
    let context = tty_style_ctx {
        defaults,
        dim,
        palette: crate::src::shared::tty::PaletteSource::Pane(Rc::downgrade(pane)),
        hyperlinks: selected.hyperlinks.clone(),
    };
    let (px, py) = source;
    let (x, y) = destination;
    let available = selected.grid().sx.saturating_sub(px);
    if available != 0 {
        tty_draw_line(
            client,
            &selected,
            px,
            py,
            width.min(available),
            x,
            y,
            (!status).then_some(&context),
        );
    }
    *target = selected;
}

pub(super) unsafe fn draw_scrollbar(
    pane_owner: &Rc<UnsafeCell<window_pane>>,
    client_owner: &ClientRef,
    span: &redraw_span,
    mut x: u32,
    mut y: u32,
    mut n: u32,
) {
    let wp = pane_owner.get();
    let sb_style = (*wp).scrollbar_style;
    let mut gc: grid_cell = grid_cell {
        data: utf8_data {
            data: [0; 32],
            have: 0,
            size: 0,
            width: 0,
        },
        attr: 0,
        flags: 0,
        fg: 0,
        bg: 0,
        us: 0,
        link: 0,
    };
    let mut slgc: grid_cell = grid_cell {
        data: utf8_data {
            data: [0; 32],
            have: 0,
            size: 0,
            width: 0,
        },
        attr: 0,
        flags: 0,
        fg: 0,
        bg: 0,
        us: 0,
        link: 0,
    };
    let mut pad_gc: grid_cell = grid_cell {
        data: utf8_data {
            data: [0; 32],
            have: 0,
            size: 0,
            width: 0,
        },
        attr: 0,
        flags: 0,
        fg: 0,
        bg: 0,
        us: 0,
        link: 0,
    };
    let mut gcp: *mut grid_cell = ::core::ptr::null_mut::<grid_cell>();
    let mut pct_view: ::core::ffi::c_double = 0.;
    let mut total_height: u_int = 0;
    let mut slider_h: u_int = 0;
    let mut slider_y: u_int = 0;
    let mut sb_h: u_int = span.data.scrollbar().height;
    let mut sb_y: u_int = span.data.scrollbar().y;
    let mut i: u_int = 0;
    let mut off: u_int = 0;
    let mut sb_w: u_int = 0;
    let mut sb_pad: u_int = 0;
    if window_pane_mode(&*wp) == WINDOW_PANE_NO_MODE {
        total_height = {
            let screen = &*displayed_screen(pane_owner);
            screen.grid().sy.wrapping_add(screen.grid().hsize)
        };
        if total_height == 0 as u_int {
            return;
        }
        pct_view = sb_h as ::core::ffi::c_double / total_height as ::core::ffi::c_double;
        slider_h = (sb_h as ::core::ffi::c_double * pct_view) as u_int;
        slider_y = sb_h.wrapping_sub(slider_h);
    } else {
        if (*wp).modes.is_empty() {
            return;
        }
        let Some((cm_y, cm_size)) = window_copy_get_current_offset(pane_owner) else {
            return;
        };
        total_height = (cm_size as u_int).wrapping_add(sb_h);
        if total_height == 0 as u_int {
            return;
        }
        pct_view = sb_h as ::core::ffi::c_double / total_height as ::core::ffi::c_double;
        slider_h = (sb_h as ::core::ffi::c_double * pct_view) as u_int;
        slider_y = (sb_h.wrapping_add(1 as u_int) as ::core::ffi::c_double
            * (cm_y as ::core::ffi::c_double / total_height as ::core::ffi::c_double))
            as u_int;
    }
    if slider_h < 1 as u_int {
        slider_h = 1 as u_int;
    }
    if slider_y >= sb_h {
        slider_y = sb_h.wrapping_sub(1 as u_int);
    }
    (*wp).sb_slider_y = slider_y;
    (*wp).sb_slider_h = slider_h;
    gc = sb_style.gc;
    memcpy(
        &raw mut slgc as *mut ::core::ffi::c_void,
        &raw mut gc as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    slgc.fg = gc.bg;
    slgc.bg = gc.fg;
    pad_gc = pane_owner.default_colours().0;
    sb_w = sb_style.width as u_int;
    sb_pad = sb_style.pad as u_int;
    off = x.wrapping_sub(span.x);
    {
        let terminal = &(client_owner);
        tty_cursor(terminal, x, y)
    };
    let mut current_block_40: u64;
    i = 0 as u_int;
    while i < n {
        if span.data.scrollbar().flags & REDRAW_SCROLLBAR_LEFT != 0 {
            if off.wrapping_add(i) >= sb_w && off.wrapping_add(i) < sb_w.wrapping_add(sb_pad) {
                tty_cell(client_owner, &pad_gc, None);
                current_block_40 = 3437258052017859086;
            } else {
                current_block_40 = 7828949454673616476;
            }
        } else if off.wrapping_add(i) < sb_pad {
            tty_cell(client_owner, &pad_gc, None);
            current_block_40 = 3437258052017859086;
        } else {
            current_block_40 = 7828949454673616476;
        }
        match current_block_40 {
            7828949454673616476 => {
                if sb_y >= slider_y && sb_y < slider_y.wrapping_add(slider_h) {
                    gcp = &raw mut slgc;
                } else {
                    gcp = &raw mut gc;
                }
                tty_cell(client_owner, &*gcp, None);
            }
            _ => {}
        }
        i = i.wrapping_add(1);
    }
}

pub(super) unsafe fn draw_prompt(
    wp_owner: &Rc<UnsafeCell<window_pane>>,
    dctx: &mut redraw_draw_ctx<'_>,
) {
    let wp = wp_owner.get();
    let prompt = (*wp).prompt.as_ref().map(|prompt| prompt.downgrade());
    let (pane_width, pane_height, xoff, yoff) = wp_owner.geometry();
    let scene = dctx.scene;
    let Some(client_owner) = scene.c.upgrade() else {
        return;
    };
    let mut c: Option<ClientRef> = Some(client_owner.clone());
    let mut screen: screen = screen::empty();
    let mut ctx: screen_write_ctx = screen_write_ctx {
        wp: std::rc::Weak::new(),
        target: Default::default(),
        flags: 0,
        init_ctx_cb: None,
        item: None,
        scrolled: 0,
        bg: 0,
    };
    let mut ox: ::core::ffi::c_int = scene.ox as ::core::ffi::c_int;
    let mut oy: ::core::ffi::c_int = scene.oy as ::core::ffi::c_int;
    let mut sx: ::core::ffi::c_int = scene.sx as ::core::ffi::c_int;
    let mut sy: ::core::ffi::c_int = scene.sy as ::core::ffi::c_int;
    let mut line: ::core::ffi::c_int = 0;
    let mut cy: ::core::ffi::c_int = 0;
    let mut px: ::core::ffi::c_int = 0;
    let mut offset: ::core::ffi::c_int = 0;
    let mut width: ::core::ffi::c_int = 0;
    let mut wy: ::core::ffi::c_int = 0;
    if prompt.is_none() || pane_width == 0 as u_int || pane_height == 0 as u_int {
        return;
    }
    if !dctx.flags & REDRAW_STATUS_TOP != 0 {
        wy = yoff + pane_height as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
    } else {
        wy = yoff;
    }
    if wy < oy || wy >= oy + sy {
        return;
    }
    line = wy - oy;
    if dctx.flags & REDRAW_STATUS_TOP != 0 {
        cy = dctx.status_lines.wrapping_add(line as u_int) as ::core::ffi::c_int;
    } else {
        cy = line;
    }
    if xoff + pane_width as ::core::ffi::c_int <= ox || xoff >= ox + sx {
        return;
    }
    if xoff < ox {
        offset = ox - xoff;
        px = 0 as ::core::ffi::c_int;
    } else {
        offset = 0 as ::core::ffi::c_int;
        px = xoff - ox;
    }
    width = pane_width.wrapping_sub(offset as u_int) as ::core::ffi::c_int;
    if px + width > sx {
        width = sx - px;
    }
    screen_init(&mut screen, pane_width, 1 as u_int, 0 as u_int);
    screen_write_start(&mut ctx, &raw mut screen);
    let pdd = prompt_draw_data {
        area_x: 0 as u_int,
        area_width: pane_width,
        prompt_line: 0 as u_int,
    };
    let prompt_cx = prompt_draw(
        &prompt
            .as_ref()
            .expect("active prompt")
            .try_borrow_mut()
            .expect("unborrowed prompt"),
        &mut ctx,
        pdd,
    );
    (*wp).prompt_cx = prompt_cx;
    screen_write_stop(&mut ctx);
    {
        let terminal = &(client_owner);
        tty_draw_line(
            terminal,
            &screen,
            0 as u_int,
            offset as u_int,
            width as u_int,
            px as u_int,
            cy as u_int,
            None,
        );
    };
    screen_free(&mut screen);
}

pub(super) unsafe fn visible_ranges(
    base_wp: Option<&Rc<UnsafeCell<window_pane>>>,
    mut px: ::core::ffi::c_int,
    py: ::core::ffi::c_int,
    mut width: u_int,
    ranges: &mut Vec<visible_range>,
) {
    ranges.clear();
    if py < 0 || width == 0 {
        return;
    }
    if px < 0 {
        if -px as u_int >= width {
            return;
        }
        width = width.wrapping_sub(-px as u_int);
        px = 0;
    }
    let Some(base_wp) = base_wp else {
        ranges.push(visible_range {
            px: px as u_int,
            nx: width,
        });
        return;
    };

    let window = base_wp
        .window_observer()
        .upgrade()
        .expect("live pane parent");
    let window_size = window.size();
    let scrollbars = window.scrollbars();
    if py as u_int >= window_size.1 || px as u_int >= window_size.0 {
        window.release(c"visible pane range outside");
        return;
    }
    if (px as u_int).wrapping_add(width) > window_size.0 {
        width = window_size.0.wrapping_sub(px as u_int);
    }
    ranges.push(visible_range {
        px: px as u_int,
        nx: width,
    });
    let mut found_self = false;
    let mut cursor = window.step_pane(PaneOrder::Stacking, None, true);
    while let Some(pane_owner) = cursor {
        let wp = &pane_owner;
        if Rc::ptr_eq(wp, base_wp) {
            found_self = true;
        } else {
            let floating = wp.is_floating();
            let no_border = floating && wp.pane_lines() == PANE_LINES_NONE as pane_lines;
            let (tb, bb) = if no_border {
                (
                    wp.geometry().3,
                    wp.geometry().3 + wp.geometry().1 as ::core::ffi::c_int - 1,
                )
            } else {
                (
                    if wp.geometry().3 > 0 {
                        wp.geometry().3 - 1
                    } else {
                        0
                    },
                    wp.geometry().3 + wp.geometry().1 as ::core::ffi::c_int,
                )
            };
            if found_self
                && pane_owner.is_visible()
                && py >= tb
                && py <= bb
                && (floating || (py != tb && py != bb))
            {
                let (sb_w, sb_pos) = if wp.scrollbar().reserved {
                    (
                        wp.scrollbar().width + wp.scrollbar().pad,
                        scrollbars.position,
                    )
                } else {
                    (0, 0)
                };
                let (mut lb, mut rb) = if no_border {
                    (
                        wp.geometry().2,
                        wp.geometry().2 + wp.geometry().0 as ::core::ffi::c_int - 1,
                    )
                } else if sb_pos == PANE_SCROLLBARS_LEFT {
                    (
                        if wp.geometry().2 > sb_w {
                            wp.geometry().2 - 1 - sb_w
                        } else {
                            0
                        },
                        wp.geometry().2 + wp.geometry().0 as ::core::ffi::c_int,
                    )
                } else {
                    (
                        if wp.geometry().2 > 0 {
                            wp.geometry().2 - 1
                        } else {
                            0
                        },
                        wp.geometry().2 + wp.geometry().0 as ::core::ffi::c_int + sb_w,
                    )
                };
                lb = lb.max(0);
                if rb >= 0 {
                    if (no_border && rb >= window_size.0 as ::core::ffi::c_int)
                        || (!no_border && rb > window_size.0 as ::core::ffi::c_int)
                    {
                        rb = window_size.0.wrapping_sub(1) as ::core::ffi::c_int;
                    }
                    if lb <= rb {
                        let mut i = 0;
                        while i < ranges.len() {
                            let range = ranges[i];
                            if range.nx != 0 {
                                let sx = range.px as ::core::ffi::c_int;
                                let ex = range.px.wrapping_add(range.nx).wrapping_sub(1)
                                    as ::core::ffi::c_int;
                                if lb > sx && lb <= ex && rb > ex {
                                    ranges[i].nx = (lb - sx) as u_int;
                                } else if rb >= sx && rb <= ex && lb <= sx {
                                    ranges[i].nx = (ex - rb) as u_int;
                                    ranges[i].px = (rb + 1) as u_int;
                                } else if lb > sx && rb <= ex {
                                    ranges.insert(
                                        i + 1,
                                        visible_range {
                                            px: (rb + 1) as u_int,
                                            nx: (ex - rb) as u_int,
                                        },
                                    );
                                    ranges[i].nx = (lb - sx) as u_int;
                                } else if lb <= sx && rb > ex {
                                    ranges[i].nx = 0;
                                }
                            }
                            i += 1;
                        }
                    }
                }
            }
        }
        cursor = window.step_pane(PaneOrder::Stacking, Some(&Rc::downgrade(wp)), true);
    }
    window.release(c"visible pane ranges");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::src::grid::view::grid_view_set_cells;
    use crate::src::reactor::{evbuffer_new, evbuffer_pullup};
    use crate::src::shared::client::{client, CLIENT_UTF8};
    use crate::src::shared::tty::{tty, tty_code, tty_term};
    use crate::src::tty_term::tty_term_ncodes;
    use std::sync::LazyLock;

    unsafe fn terminal() -> ClientRef {
        let client = ClientRef::allocate();
        client.update_flags(CLIENT_UTF8 as u64, 0);
        let mut term = tty_term::empty();
        term.codes = vec![tty_code::None; tty_term_ncodes() as usize].into_boxed_slice();
        *client.borrow_terminal_mut() = tty {
            term: Some(Box::new(term)),
            out: Some(evbuffer_new()),
            cell: grid_default_cell,
            last_cell: grid_default_cell,
            sx: 10,
            sy: 2,
            ..Default::default()
        };
        client
    }

    unsafe fn output(client: &ClientRef) -> Vec<u8> {
        evbuffer_pullup(client.borrow_terminal_mut().out.as_deref_mut().unwrap(), -1)
            .unwrap_or_default()
            .to_vec()
    }

    unsafe fn display_mode(mode: refbox::Weak<window_mode_entry>) -> *mut screen {
        let entry = mode.get_unchecked();
        let pane = entry.wp.upgrade().expect("mode pane remains live");
        assert_eq!(pane.window_size(), (0, 0));
        entry.boxed_data_ptr::<screen>().expect("mode screen")
    }

    static MODE: LazyLock<window_mode> = LazyLock::new(|| window_mode {
        name: c"render-test",
        display_screen: Some(display_mode),
        ..Default::default()
    });

    #[test]
    fn style_refresh_preserves_palette_defaults_and_active_dimming() {
        use crate::src::options::{
            options_create, options_default, options_free, options_set_string,
        };
        unsafe {
            let window = crate::src::shared::window::WindowRef::empty();
            let pane = window_pane::new();
            (*pane.get()).window = Rc::downgrade(&window);
            (*pane.get()).options = Some(options_create(None));
            pane.with_options_mut(|options| {
                for key in [c"window-style", c"window-active-style"] {
                    let definition = crate::src::options_table::options_table
                        .iter()
                        .find(|entry| entry.name == Some(key))
                        .unwrap();
                    options_default(options, definition);
                }
                options_set_string(options, c"window-style".as_ptr(), 0, |out| {
                    out.write_all(b"fg=red,bg=blue,dim=25%")
                });
                options_set_string(options, c"window-active-style".as_ptr(), 0, |out| {
                    out.write_all(b"fg=green,dim=50%")
                });
            });
            (*pane.get()).palette.fg = 6;
            (*pane.get()).palette.bg = 7;
            window.initialize_pane(&pane, None);
            pane.mark_style_changed(true);
            let (cell, dim) = pane.default_colours();
            assert_eq!((cell.fg, cell.bg, dim), (2, 7, 50));
            assert_eq!(
                (
                    (*pane.get()).cached_gc.fg,
                    (*pane.get()).cached_gc.bg,
                    (*pane.get()).cached_dim
                ),
                (1, 4, 25)
            );
            assert_eq!(((*pane.get()).palette.fg, (*pane.get()).palette.bg), (6, 7));
            assert_eq!((*pane.get()).flags & PANE_STYLECHANGED, 0);
            assert_ne!((*pane.get()).flags & PANE_THEMECHANGED, 0);
            assert_eq!(pane.default_colours().1, 50);
            options_free((*pane.get()).options.take().unwrap());
            for order in [PaneOrder::Index, PaneOrder::Stacking] {
                window.borrow_pane_order_mut(order).storage.clear();
            }
            (*pane.get()).window = Weak::new();
            window.release(c"render style test");
        }
    }

    #[test]
    fn drawing_restores_the_selected_screen_and_grid_owner() {
        unsafe {
            for displayed_mode in [false, true] {
                let window = crate::src::shared::window::WindowRef::empty();
                let pane = window_pane::new();
                (*pane.get()).window = Rc::downgrade(&window);
                (*pane.get()).cached_gc = grid_default_cell;
                let mut selected = screen::empty();
                selected.grid = Some(crate::src::grid::grid_create(3, 1, 0));
                grid_view_set_cells(selected.grid_mut(), 0, 0, &grid_default_cell, b"abc");
                let target = if displayed_mode {
                    let entry = refbox::RefBox::new(window_mode_entry {
                        wp: Rc::downgrade(&pane),
                        swp: Weak::new(),
                        mode: &MODE,
                        boxed_data: Some(Box::new(UnsafeCell::new(selected))),
                        data_owner: None,
                        prefix: 1,
                        kill: 0,
                    });
                    let target = entry
                        .try_borrow_mut()
                        .unwrap()
                        .boxed_data_ptr::<screen>()
                        .unwrap();
                    (*pane.get()).screen_source = PaneScreenSource::Mode(entry.downgrade());
                    (*pane.get()).modes.push(entry);
                    target
                } else {
                    (*pane.get()).base = selected;
                    &raw mut (*pane.get()).base
                };
                let grid = (*target).grid.as_deref().unwrap() as *const _;
                let client = terminal();
                pane.draw_line(&client, (0, 0), 3, (0, 0), false);
                assert_eq!(output(&client), b"abc");
                assert_eq!(displayed_screen(&pane), target);
                assert_eq!((*target).grid.as_deref().unwrap() as *const _, grid);
                assert_eq!((*target).grid().sx, 3);
                assert_eq!((*target).grid().sy, 1);
                screen_free(&mut *target);
                (*pane.get()).modes.clear();
                (*pane.get()).screen_source = PaneScreenSource::Base;
                (*pane.get()).window = Weak::new();
                window.release(c"render test");
            }
        }
    }
}
