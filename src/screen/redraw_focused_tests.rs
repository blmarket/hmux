use crate::grid::Grid as _;
use crate::WindowPane;
use crate::options::OptionsRef;
use crate::pane_geometry::PaneGeometryState;
use crate::window_scrollbar::{WindowScrollbarSettings, WindowScrollbarState};

use super::*;
use crate::server::client_ref_of;
use crate::tests::test_fixtures::{Pane, Session, Window, globals, link, unlink, zeroed_client};
use crate::types::{ClientRef, client, winlink};
use core::{ffi::c_int, ptr::null_mut};

struct View {
    client: ClientRef,
    panes: Vec<Pane>,
    window: Window,
    session: Session,
    wl: *mut winlink,
}

unsafe fn set_scrollbar_dimensions(wp: *mut (dyn crate::WindowPane + 'static), width: c_int, padding: c_int) {
    unsafe {
        let mut style = (*wp).scrollbar_style();
        style.width = width;
        style.padding = padding;
        (*wp).configure_test(crate::window_pane::PaneTestSetup::ScrollbarStyle(style));
    }
}

impl View {
    fn new(sx: u_int, sy: u_int) -> Self {
        let mut view = Self {
            client: zeroed_client(),
            panes: Vec::new(),
            window: Window::new(41, "redraw-focused", sx, sy),
            session: Session::new(42, "redraw-focused"),
            wl: null_mut(),
        };
        view.wl = link(&mut view.session, &mut view.window, 0);
        unsafe { (*view.client_ptr()).set_attached_session(Some(view.session.handle())) };
        view
    }

    fn add(&mut self, x: c_int, y: c_int, sx: u_int, sy: u_int) -> usize {
        let mut pane = Pane::new(50 + self.panes.len() as u_int, sx, sy, 20);
        unsafe {
            (*pane.ptr()).set_position(x, y);
        }
        self.window.add_pane(&mut pane);
        self.panes.push(pane);
        self.panes.len() - 1
    }

    fn pane(&mut self, index: usize) -> *mut (dyn crate::WindowPane + 'static) {
        self.panes[index].ptr()
    }

    fn client_ptr(&mut self) -> *mut client {
        unsafe { self.client.as_client_mut() }
    }

    fn ctx(&mut self) -> screen_redraw_ctx {
        screen_redraw_ctx {
            c: unsafe { client_ref_of(&*self.client_ptr()) },
            ..Default::default()
        }
    }
}

impl Drop for View {
    fn drop(&mut self) {
        if !self.wl.is_null() {
            unlink(&mut self.session, self.wl);
        }
    }
}

#[test]
fn check_cell_selects_body_window_edges_and_status() {
    let _guard = globals();
    let mut view = View::new(10, 5);
    view.add(0, 0, 10, 5);
    let mut ctx = view.ctx();
    unsafe {
        let checked = screen_redraw_check_cell(&mut ctx, 3, 2);
        assert_eq!(checked.cell_type, CELL_INSIDE);
        assert_eq!(checked.pane_ref.as_ref().map(|pane| pane.id()), Some(50));
        assert_eq!(
            screen_redraw_check_cell(&mut ctx, 10, 2).cell_type,
            CELL_TOPBOTTOM
        );
        assert_eq!(
            screen_redraw_check_cell(&mut ctx, 11, 2).cell_type,
            CELL_OUTSIDE
        );
        (*view.pane(0)).publish_border_status(4, RustScreen::new_with_server_options(4, 1, 0), Vec::new(), c"".to_owned());
        ctx.pane_status = PANE_STATUS_TOP;
        ctx.oy = 1;
        assert_eq!(
            screen_redraw_check_cell(&mut ctx, 3, 0).cell_type,
            CELL_INSIDE
        );
    }
}

#[test]
fn check_cell_selects_scrollbar_and_front_floating_pane() {
    let _guard = globals();
    let mut view = View::new(12, 7);
    let front = view.add(3, 2, 4, 2);
    let base = view.add(0, 0, 12, 7);
    unsafe {
        crate::tests::test_fixtures::set_pane_floating(
            &mut *view.window.ptr(),
            (*view.pane(front)).pane_id(),
            true,
        )
    };
    let mut ctx = view.ctx();
    unsafe {
        let checked = screen_redraw_check_cell(&mut ctx, 4, 2);
        assert_eq!(checked.cell_type, CELL_INSIDE);
        assert_eq!(checked.pane_ref.as_ref().map(|pane| pane.id()), Some(50));
        assert_eq!(
            screen_redraw_check_cell(&mut ctx, 2, 2).cell_type,
            CELL_TOPBOTTOM
        );
        assert_eq!(
            screen_redraw_check_cell(&mut ctx, 2, 2)
                .pane_ref
                .as_ref()
                .map(|pane| pane.id()),
            Some(50)
        );

        (*view.window.ptr()).set_scrollbar_settings(WindowScrollbarSettings {
            sb: 2,
            sb_pos: PANE_SCROLLBARS_RIGHT,
        });
        set_scrollbar_dimensions(view.pane(base), 1, 1);
        let geometry = (*view.pane(base)).geometry();
        (*view.pane(base)).configure_test(crate::window_pane::PaneTestSetup::Size(crate::pane_resize::PaneSize {
            width: 10,
            height: geometry.sy,
        }));
        let checked = screen_redraw_check_cell(&mut ctx, 10, 5);
        assert_eq!(checked.cell_type, CELL_SCROLLBAR);
        assert_eq!(checked.pane_ref.as_ref().map(|pane| pane.id()), Some(51));
    }
}

#[test]
fn cell_scan_uses_an_active_pane_absent_from_the_stacking_order_once() {
    let _guard = globals();
    let mut view = View::new(12, 8);
    view.add(2, 2, 3, 2);
    let ctx = view.ctx();
    unsafe {
        let window = view.window.handle().clone();
        window.as_window_mut().z_index.clear();
        let inside = screen_redraw_check_cell(&ctx, 3, 2);
        assert_eq!(
            (
                inside.cell_type,
                inside.pane_ref.as_ref().map(|pane| pane.id())
            ),
            (CELL_INSIDE, Some(50))
        );
        let outside = screen_redraw_check_cell(&ctx, 9, 6);
        assert_eq!(
            (
                outside.cell_type,
                outside.pane_ref.as_ref().map(|pane| pane.id())
            ),
            (CELL_OUTSIDE, Some(50))
        );
        let active = window.as_window_mut().active.take();
        let missing = screen_redraw_check_cell(&ctx, 3, 2);
        assert_eq!(
            (
                missing.cell_type,
                missing.pane_ref.as_ref().map(|pane| pane.id())
            ),
            (CELL_OUTSIDE, None)
        );
        window.as_window_mut().active = active;
    }
}

#[test]
fn cell_scan_wraps_to_the_visible_pane_before_its_start() {
    let _guard = globals();
    let mut view = View::new(20, 10);
    view.add(0, 0, 4, 3);
    view.add(7, 0, 4, 3);
    view.add(14, 0, 4, 3);
    let ctx = view.ctx();
    unsafe {
        let window = view.window.handle().clone();
        window.as_window_mut().flags |= crate::window::WINDOW_ZOOMED;
        let checked = screen_redraw_check_cell(&ctx, 8, 1);
        assert_eq!(
            (
                checked.cell_type,
                checked.pane_ref.as_ref().map(|pane| pane.id())
            ),
            (CELL_OUTSIDE, Some(50))
        );
        window.as_window_mut().flags &= !crate::window::WINDOW_ZOOMED;
    }
}

#[test]
fn border_cells_cover_all_line_styles_and_numbered_panes() {
    let _guard = globals();
    let mut view = View::new(8, 4);
    view.add(0, 0, 8, 4);
    let mut gc = grid_default_cell;
    unsafe {
        let owner = view.window.reference();
        let w = &owner;
        let wp = &*view.pane(0);
        for (lines, cell_type) in [
            (PANE_LINES_SINGLE, 1),
            (PANE_LINES_DOUBLE, 2),
            (PANE_LINES_HEAVY, 3),
            (PANE_LINES_SIMPLE, 4),
            (PANE_LINES_SPACES, 5),
        ] {
            screen_redraw_border_set(w, Some(wp), lines, cell_type, &mut gc);
            assert!(gc.data.size <= 4);
        }
        screen_redraw_border_set(w, Some(wp), PANE_LINES_NUMBER, CELL_INSIDE, &mut gc);
        assert_eq!(gc.data.data[0], b'0');
        screen_redraw_border_set(
            w,
            None::<&dyn crate::WindowPane>,
            PANE_LINES_NUMBER,
            CELL_INSIDE,
            &mut gc,
        );
        assert_eq!(gc.data.data[0], b'*');
        screen_redraw_border_set(
            w,
            None::<&dyn crate::WindowPane>,
            PANE_LINES_NUMBER,
            CELL_OUTSIDE,
            &mut gc,
        );
    }
}

#[test]
fn floating_pane_reports_each_border_edge_and_outside() {
    let _guard = globals();
    let mut view = View::new(20, 10);
    let pane = view.add(4, 3, 6, 4);
    unsafe {
        crate::tests::test_fixtures::set_pane_floating(
            &mut *view.window.ptr(),
            (*view.pane(pane)).pane_id(),
            true,
        )
    };
    let mut ctx = view.ctx();
    unsafe {
        let wp = &mut *view.pane(pane);
        assert_eq!(
            screen_redraw_pane_border(&mut ctx, wp, 5, 4),
            SCREEN_REDRAW_INSIDE
        );
        assert_eq!(
            screen_redraw_pane_border(&mut ctx, wp, 3, 4),
            SCREEN_REDRAW_BORDER_LEFT
        );
        assert_eq!(
            screen_redraw_pane_border(&mut ctx, wp, 10, 4),
            SCREEN_REDRAW_BORDER_RIGHT
        );
        assert_eq!(
            screen_redraw_pane_border(&mut ctx, wp, 5, 2),
            SCREEN_REDRAW_BORDER_TOP
        );
        assert_eq!(
            screen_redraw_pane_border(&mut ctx, wp, 5, 7),
            SCREEN_REDRAW_BORDER_BOTTOM
        );
        assert_eq!(
            screen_redraw_pane_border(&mut ctx, wp, 0, 0),
            SCREEN_REDRAW_OUTSIDE
        );

        (*view.window.ptr()).set_scrollbar_settings(WindowScrollbarSettings {
            sb: 1,
            sb_pos: PANE_SCROLLBARS_LEFT,
        });
        set_scrollbar_dimensions(wp, 1, 1);
        let _ = screen_redraw_pane_border(&mut ctx, wp, 2, 4);
    }
}

#[test]
fn border_classification_covers_window_limits_and_status_positions() {
    let _guard = globals();
    let mut view = View::new(9, 5);
    view.add(0, 0, 9, 5);
    let mut ctx = view.ctx();
    unsafe {
        let wp = &mut *view.pane(0);
        let _ = screen_redraw_cell_border(&mut ctx, wp, 9, 2);
        assert_eq!(screen_redraw_cell_border(&mut ctx, wp, 10, 2), 0);
        assert_eq!(screen_redraw_type_of_cell(&mut ctx, wp, 10, 2), 12);
        assert_eq!(
            screen_redraw_check_is(&mut ctx, 0, 0, None::<&dyn crate::WindowPane>),
            0
        );
        assert_eq!(screen_redraw_check_is(&mut ctx, 9, 2, Some(wp)), 1);

        for status in [PANE_STATUS_TOP, PANE_STATUS_BOTTOM, PANE_STATUS_OFF] {
            ctx.pane_status = status;
            for x in 0..=9 {
                for y in 0..=5 {
                    let _ = screen_redraw_type_of_cell(&mut ctx, wp, x, y);
                }
            }
        }
    }
}

#[test]
fn border_probes_handle_a_missing_current_window() {
    let _guard = globals();
    let mut view = View::new(9, 5);
    view.add(0, 0, 9, 5);
    let mut ctx = view.ctx();
    unsafe {
        let mut session = view.session.handle().clone();
        let current = session.as_session_mut().curw.take();
        let pane = &*view.pane(0);
        assert_eq!(screen_redraw_cell_border(&ctx, pane, 9, 2), 0);
        assert_eq!(screen_redraw_type_of_cell(&ctx, pane, 9, 2), CELL_OUTSIDE);
        session.as_session_mut().curw = current;
        assert_eq!(screen_redraw_cell_border(&ctx, pane, 9, 2), 1);
        ctx.c = None;
        assert_eq!(screen_redraw_type_of_cell(&ctx, pane, 9, 2), CELL_OUTSIDE);
    }
}

#[test]
fn context_uses_terminal_viewport_and_screen_honours_suspension() {
    let _guard = globals();
    let mut view = View::new(30, 12);
    view.add(0, 0, 30, 12);
    unsafe {
        let c = &mut *view.client_ptr();
        c.tty.sx = 17;
        c.tty.sy = 8;
        ((*view.window.ptr()).options_ref())
            .set_number(c"pane-border-status", PANE_STATUS_TOP.into());
        ((*view.window.ptr()).options_ref())
            .set_number(c"pane-border-lines", PANE_LINES_HEAVY.into());
        let mut ctx = screen_redraw_ctx::default();
        assert!(screen_redraw_set_context(c, &mut ctx));
        assert!(ctx.sx <= 17);
        assert!(ctx.sy <= 8);
        assert_eq!(ctx.pane_status, PANE_STATUS_TOP);
        assert_eq!(ctx.pane_lines, PANE_LINES_HEAVY);

        c.flags = CLIENT_SUSPENDED as u64;
        screen_redraw_screen(c);
        assert_eq!(c.flags, CLIENT_SUSPENDED as u64);
    }
}

#[test]
fn visible_ranges_cover_empty_clamped_seed_and_clip_inputs() {
    let _guard = globals();
    unsafe {
        let mut r = visible_ranges::default();
        r.set_visible_ranges(None::<&dyn crate::WindowPane>, 3, -1, 5);
        assert_eq!(r.used, 0);
        r.set_visible_ranges(None::<&dyn crate::WindowPane>, 3, 2, 0);
        assert_eq!(r.used, 0);
        r.set_visible_ranges(None::<&dyn crate::WindowPane>, -6, 2, 5);
        assert_eq!(r.used, 0);
        r.set_visible_ranges(None::<&dyn crate::WindowPane>, -2, 2, 7);
        assert_eq!(r.used, 1);
        assert_eq!((r.ranges[0].px, r.ranges[0].nx), (0, 5));

        r.ranges[0].px = 4;
        r.ranges[0].nx = 3;
        r.used = 1;
        r.clip_visible_ranges(None::<&dyn crate::WindowPane>, 0, 2, 20);
        assert_eq!((r.used, r.ranges[0].px, r.ranges[0].nx), (1, 4, 3));
    }
}

#[test]
fn visible_ranges_clip_to_window_and_front_pane_occlusion() {
    let _guard = globals();
    let mut view = View::new(20, 10);
    let front = view.add(7, 3, 5, 3);
    let base = view.add(0, 0, 20, 10);
    unsafe {
        crate::tests::test_fixtures::set_pane_floating(
            &mut *view.window.ptr(),
            (*view.pane(front)).pane_id(),
            true,
        );
        let mut r = visible_ranges::default();
        r.set_visible_ranges(Some(&*view.pane(base)), 0, 4, 25);
        assert!(r.used >= 1);
        assert!(
            r.ranges
                .iter()
                .take(r.used as usize)
                .all(|range| range.px + range.nx <= 20)
        );
        assert!(
            r.ranges
                .iter()
                .take(r.used as usize)
                .all(|range| range.nx == 0 || range.px > 12 || range.px + range.nx <= 6)
        );

        r.set_visible_ranges(Some(&*view.pane(base)), 0, 20, 10);
        assert_eq!(r.used, 0);

        let mut scrollbar = (*view.window.ptr()).scrollbar_settings();
        scrollbar.sb_pos = PANE_SCROLLBARS_LEFT;
        (*view.window.ptr()).set_scrollbar_settings(scrollbar);
        set_scrollbar_dimensions(view.pane(front), 1, 1);
        r.set_visible_ranges(Some(&*view.pane(base)), 0, 4, 20);
        assert!(r.used >= 1);
    }
}

#[test]
fn border_style_expands_the_target_pane_and_keeps_separate_cache_entries() {
    let _guard = globals();
    let mut view = View::new(20, 8);
    view.add(0, 0, 9, 8);
    view.add(10, 0, 10, 8);
    let mut ctx = view.ctx();
    unsafe {
        let mut pane = view.window.handle().as_window().panes[1].downgrade();
        let options = pane.get().unwrap().options_ref().clone();
        options.set_string(
            c"pane-active-border-style",
            0,
            c"#{?pane_active,fg=red,fg=green}",
            fmt_args![],
        );
        options.set_string(c"pane-border-style", 0, c"fg=blue", fmt_args![]);
        let mut cell = grid_default_cell;
        let mut cache = BorderPassCache::default();
        screen_redraw_draw_borders_style(&ctx, &mut cache, 9, 2, &mut pane, &mut cell);
        assert_eq!(cell.fg, 2);
        screen_redraw_draw_borders_style(&ctx, &mut cache, 19, 2, &mut pane, &mut cell);
        assert_eq!(cell.fg, 4);
        options.set_string(c"pane-active-border-style", 0, c"fg=yellow", fmt_args![]);
        screen_redraw_draw_borders_style(&ctx, &mut cache, 9, 2, &mut pane, &mut cell);
        assert_eq!(cell.fg, 2);
        screen_redraw_draw_borders(&mut ctx);
        let mut cache = BorderPassCache::default();
        screen_redraw_draw_borders_style(&ctx, &mut cache, 9, 2, &mut pane, &mut cell);
        assert_eq!(cell.fg, 3);
        options.set_string(c"pane-active-border-style", 0,
            c"#{?#{==:#{client_width},30},fg=red,fg=green}", fmt_args![]);
        let mut cache = BorderPassCache::default();
        screen_redraw_draw_borders_style(&ctx, &mut cache, 9, 2, &mut pane, &mut cell);
        assert_eq!(cell.fg, 2);
        let mut other = zeroed_client();
        other.as_client_mut().set_attached_session(Some(view.session.handle()));
        other.as_client_mut().tty.sx = 30;
        let other_ctx = screen_redraw_ctx { c: Some(other), ..Default::default() };
        let mut other_cache = BorderPassCache::default();
        screen_redraw_draw_borders_style(&other_ctx, &mut other_cache, 9, 2, &mut pane, &mut cell);
        assert_eq!(cell.fg, 1);
    }
}

#[test]
fn scrollbar_redraw_tracks_owned_panes_and_skips_removed_targets() {
    if crate::test_process::run() {
        return;
    }
    let _guard = globals();
    let mut view = View::new(12, 6);
    view.add(1, 1, 4, 3);
    view.add(6, 1, 4, 3);
    unsafe {
        let weak = view.client.downgrade();
        let client = view.client.as_client_mut();
        client.tty.client = Some(weak);
        client.tty.sx = 12;
        client.tty.sy = 6;
        client.tty.term = Some(crate::tests::test_fixtures::zeroed_term());
        client.tty.out = Some(Box::new(crate::reactor::ByteBuffer::new()));
        let window = view.window.handle().clone();
        window.set_scrollbar_settings(WindowScrollbarSettings {
            sb: crate::window::PANE_SCROLLBARS_ALWAYS,
            sb_pos: PANE_SCROLLBARS_RIGHT,
        });
        let mut panes = window.panes();
        for pane in &mut panes {
            let pane = pane.get_mut().unwrap();
            let mut style = pane.scrollbar_style();
            style.width = 1;
            style.padding = 0;
            pane.configure_test(crate::window_pane::PaneTestSetup::ScrollbarStyle(style));
            pane.publish_slider(PaneScrollbarSlider { sb_slider_y: 99, sb_slider_h: 99 });
        }
        let mut ctx = view.ctx();
        ctx.sx = 12;
        ctx.sy = 6;
        screen_redraw_draw_pane_scrollbars(&mut ctx);
        for pane in &panes {
            let slider = pane.get().unwrap().slider();
            assert_eq!((slider.sb_slider_y, slider.sb_slider_h), (0, 3));
        }
        window.set_scrollbar_settings(WindowScrollbarSettings {
            sb: PANE_SCROLLBARS_MODAL,
            sb_pos: PANE_SCROLLBARS_RIGHT,
        });
        panes[0]
            .get_mut()
            .unwrap()
            .publish_slider(PaneScrollbarSlider { sb_slider_y: 7, sb_slider_h: 8 });
        screen_redraw_draw_pane_scrollbars(&mut ctx);
        assert_eq!(panes[0].get().unwrap().slider().sb_slider_y, 7);
        let id = panes[1].id();
        window.remove_pane(&crate::window::window_pane_find_by_id(id).expect("the pane exists"));
        assert!(panes[1].get().is_none());
        screen_redraw_draw_pane_scrollbar(&mut ctx, &mut panes[1]);
    }
}

#[test]
fn pane_drawing_borrows_the_shown_screen_after_style_evaluation_and_clips_rows() {
    use crate::screen::Screen;
    let _guard = globals();
    let mut view = View::new(8, 3);
    view.add(1, 0, 6, 1);
    unsafe {
        let weak = view.client.downgrade();
        let client = view.client.as_client_mut();
        client.tty.client = Some(weak);
        client.tty.sx = 8;
        client.tty.sy = 3;
        client.tty.term = Some(crate::tests::test_fixtures::zeroed_term());
        client.tty.out = Some(Box::new(crate::reactor::ByteBuffer::new()));
        let mut pane = view.window.handle().as_window().panes[0].downgrade();
        {
            let wp = pane.get_mut().unwrap();
            let mut writer = screen_write_ctx_on_screen(wp.base_mut());
            writer.puts(&grid_default_cell, c"abcdef", fmt_args![]);
            writer.finish();
            let pane_screen_mode = wp.base().mode() | MODE_SYNC;
            wp.base_mut().set_mode(pane_screen_mode);
        }
        let mut ctx = view.ctx();
        ctx.ox = 2;
        ctx.sx = 3;
        ctx.sy = 1;
        screen_redraw_draw_pane(&mut ctx, &mut pane);
        assert_eq!(
            view.client.as_tty_mut().out.as_mut().unwrap().as_slice(),
            b"bcd"
        );
        assert_eq!(pane.get().unwrap().base().mode() & MODE_SYNC, 0);
        view.client.as_tty_mut().out.as_mut().unwrap().clear();
        (pane.get_mut().unwrap()).set_mode(
            None,
            WindowMode::View,
            None,
            None,
        );
        let display = pane.get().unwrap().active_mode().unwrap()
            .state
            .copy_mode_data_ref()
            .unwrap()
            .screen
            .clone();
        {
            let mut screen = display.borrow_mut();
            let mut writer = screen_write_ctx_on_screen(&mut screen);
            writer.puts(&grid_default_cell, c"uvwxyz", fmt_args![]);
            writer.finish();
        }
        *pane.get_mut().unwrap().flags_mut() |= crate::window::PANE_STYLECHANGED;
        pane.get().unwrap().options_ref().set_string(
            c"window-style",
            0,
            c"#{?pane_in_mode,fg=default,fg=red}",
            fmt_args![],
        );
        let _display_borrow = display.borrow();
        screen_redraw_draw_pane(&mut ctx, &mut pane);
        assert_eq!(
            view.client.as_tty_mut().out.as_mut().unwrap().as_slice(),
            b"\rvwx"
        );
    }
}

#[test]
fn pane_status_drawing_preserves_clipping_zoom_and_top_or_bottom_rows() {
    if crate::test_process::run() {
        return;
    }
    let _guard = globals();
    let mut view = View::new(12, 6);
    view.add(0, 1, 5, 3);
    view.add(6, 1, 5, 3);
    unsafe {
        let weak = view.client.downgrade();
        let client = view.client.as_client_mut();
        client.tty.client = Some(weak);
        client.tty.sx = 12;
        client.tty.sy = 6;
        client.tty.term = Some(crate::tests::test_fixtures::zeroed_term());
        client.tty.out = Some(Box::new(crate::reactor::ByteBuffer::new()));
        let window = view.window.handle().clone();
        let mut panes = window.panes();
        for (pane, text) in panes.iter_mut().zip([c"abc", c"XYZ"]) {
            let pane = pane.get_mut().unwrap();
            let mut screen = RustScreen::new_with_server_options(3, 1, 0);
            let mut writer = screen_write_ctx_on_screen(&mut screen);
            writer.puts(&grid_default_cell, text, fmt_args![]);
            writer.finish();
            pane.publish_border_status(3, screen, Vec::new(), text.to_owned());
        }
        let mut ctx = view.ctx();
        ctx.sx = 12;
        ctx.sy = 6;
        ctx.pane_status = PANE_STATUS_TOP;
        screen_redraw_draw_pane_status(&mut ctx);
        assert_eq!(
            view.client.as_tty_mut().out.as_mut().unwrap().as_slice(),
            b"abcXYZ\r"
        );
        view.client.as_tty_mut().out.as_mut().unwrap().clear();
        window.as_window_mut().flags |= crate::window::WINDOW_ZOOMED;
        {
            let mut payload = window.as_window_mut();
            payload.active = payload
                .panes
                .iter()
                .find(|pane| pane.pane_id() == panes[1].id())
                .map(|pane| pane.downgrade());
        }
        screen_redraw_draw_pane_status(&mut ctx);
        assert_eq!(
            view.client.as_tty_mut().out.as_mut().unwrap().as_slice(),
            b"XYZ\r"
        );
        view.client.as_tty_mut().out.as_mut().unwrap().clear();
        window.as_window_mut().flags &= !crate::window::WINDOW_ZOOMED;
        ctx.ox = 3;
        ctx.sx = 2;
        screen_redraw_draw_pane_status(&mut ctx);
        assert_eq!(
            view.client.as_tty_mut().out.as_mut().unwrap().as_slice(),
            b"bc\r"
        );
        view.client.as_tty_mut().out.as_mut().unwrap().clear();
        ctx.ox = 0;
        ctx.sx = 12;
        ctx.oy = 4;
        ctx.sy = 1;
        ctx.pane_status = PANE_STATUS_BOTTOM;
        screen_redraw_draw_pane_status(&mut ctx);
        assert_eq!(
            view.client.as_tty_mut().out.as_mut().unwrap().as_slice(),
            b"abcXYZ\r"
        );
        view.client.as_tty_mut().out.as_mut().unwrap().clear();
        view.client.set_attached_session(None);
        screen_redraw_draw_pane_status(&mut ctx);
        assert!(
            view.client
                .as_tty_mut()
                .out
                .as_mut()
                .unwrap()
                .as_slice()
                .is_empty()
        );
    }
}

#[test]
fn redraw_context_reads_shared_owners_and_rejects_detached_or_missing_windows() {
    let _guard = globals();
    let mut view = View::new(30, 12);
    view.add(0, 0, 30, 12);
    unsafe {
        let mut session = view.session.handle().clone();
        let options = session.options();
        let window = view.window.handle().clone();
        window
            .options()
            .set_number(c"pane-border-status", PANE_STATUS_BOTTOM.into());
        window
            .options()
            .set_number(c"pane-border-lines", PANE_LINES_DOUBLE.into());
        let c = view.client.as_client_mut();
        c.tty.oox = 4;
        c.tty.ooy = 2;
        c.tty.osx = 17;
        c.tty.osy = 8;
        options.set_number(c"status", 2);
        options.set_number(c"status-position", 0);
        session.update_status_cache();
        let mut ctx = screen_redraw_ctx::default();
        assert!(screen_redraw_set_context(c, &mut ctx));
        assert_eq!((ctx.ox, ctx.oy, ctx.sx, ctx.sy), (4, 2, 17, 8));
        assert_eq!((ctx.statuslines, ctx.statustop), (2, 1));
        assert_eq!(ctx.pane_status, PANE_STATUS_BOTTOM);
        assert_eq!(ctx.pane_lines, PANE_LINES_DOUBLE);
        options.set_number(c"status", 0);
        options.set_number(c"status-position", 1);
        session.update_status_cache();
        assert!(screen_redraw_set_context(c, &mut ctx));
        assert_eq!((ctx.statuslines, ctx.statustop), (0, 0));
        c.message_string = Some(c"message".to_owned());
        assert!(screen_redraw_set_context(c, &mut ctx));
        assert_eq!((ctx.statuslines, ctx.statustop), (1, 0));
        c.message_string = None;
        c.prompt_string = Some(c"prompt".to_owned());
        options.set_number(c"status-position", 0);
        assert!(screen_redraw_set_context(c, &mut ctx));
        assert_eq!((ctx.statuslines, ctx.statustop), (1, 1));
        c.prompt_string = None;
        let current = session.as_session_mut().curw.take();
        assert!(!screen_redraw_set_context(c, &mut ctx));
        assert!(ctx.c.is_none());
        assert_eq!((ctx.sx, ctx.sy), (0, 0));
        screen_redraw_screen(c);
        session.as_session_mut().curw = current;
        c.set_attached_session(None);
        assert!(!screen_redraw_set_context(c, &mut ctx));
        screen_redraw_screen(c);
    }
}

#[test]
fn pane_status_generation_tracks_formats_widths_and_owner_lifetime() {
    let _guard = globals();
    let mut view = View::new(12, 6);
    view.add(0, 1, 10, 3);
    unsafe {
        let window = view.window.handle().clone();
        let mut pane = window.as_window().panes[0].downgrade();
        let options = pane.get().unwrap().options_ref().clone();
        pane.get_mut().unwrap().base_mut().set_title(c"alpha", 0);
        options.set_string(
            c"pane-border-format",
            0,
            c"#[range=left]#{pane_index}:#{pane_title}#[norange]",
            fmt_args![],
        );
        let mut ctx = view.ctx();
        ctx.pane_status = PANE_STATUS_TOP;
        ctx.sx = 12;
        ctx.sy = 6;
        assert_eq!(
            screen_redraw_make_pane_status(
                view.client.as_client(),
                &mut pane,
                &ctx,
                PANE_LINES_SINGLE
            ),
            1
        );
        let wp = pane.get().unwrap();
        assert_eq!(wp.status_line_width(), 8);
        assert!(wp.border_status_range(0).is_some());
        assert_eq!(
            (wp.status_screen().grid()).string_cells(0, 0, 7, None, 0, None)
                .as_bytes(),
            b"0:alpha"
        );
        assert_eq!(
            screen_redraw_make_pane_status(
                view.client.as_client(),
                &mut pane,
                &ctx,
                PANE_LINES_SINGLE
            ),
            0
        );
        options.set_string(c"pane-border-format", 0, c"#{pane_title}", fmt_args![]);
        pane.get_mut().unwrap().base_mut().set_title(c"beta", 0);
        assert_eq!(
            screen_redraw_make_pane_status(
                view.client.as_client(),
                &mut pane,
                &ctx,
                PANE_LINES_SINGLE
            ),
            1
        );
        let wp = pane.get().unwrap();
        assert!(wp.border_status_range(0).is_none());
        assert_eq!(
            (wp.status_screen().grid()).string_cells(0, 0, 4, None, 0, None)
                .as_bytes(),
            b"beta"
        );
        pane.get_mut().unwrap().set_position(8, 1);
        assert_eq!(
            screen_redraw_make_pane_status(
                view.client.as_client(),
                &mut pane,
                &ctx,
                PANE_LINES_SINGLE
            ),
            1
        );
        assert_eq!(pane.get().unwrap().status_line_width(), 2);
        pane.get_mut()
            .unwrap()
            .configure_test(crate::window_pane::PaneTestSetup::Size(crate::pane_resize::PaneSize {
                width: 3,
                height: 3,
            }));
        screen_redraw_make_pane_status(view.client.as_client(), &mut pane, &ctx, PANE_LINES_SINGLE);
        assert_eq!(pane.get().unwrap().status_line_width(), 0);
        assert_eq!(
            screen_redraw_make_pane_status(
                view.client.as_client(),
                &mut pane,
                &ctx,
                PANE_LINES_SINGLE
            ),
            0
        );
        view.client.set_attached_session(None);
        assert_eq!(
            screen_redraw_make_pane_status(
                view.client.as_client(),
                &mut pane,
                &ctx,
                PANE_LINES_SINGLE
            ),
            0
        );
        view.client
            .set_attached_session(Some(view.session.handle()));
        window.remove_pane(
            &crate::window::window_pane_find_by_id(pane.id()).expect("the pane exists"),
        );
        assert!(pane.get().is_none());
        assert_eq!(
            screen_redraw_make_pane_status(
                view.client.as_client(),
                &mut pane,
                &ctx,
                PANE_LINES_SINGLE
            ),
            0
        );
    }
}

#[test]
fn status_drawing_skips_a_missing_current_window_before_accessing_the_terminal() {
    let _guard = globals();
    let mut view = View::new(12, 6);
    let mut ctx = view.ctx();
    ctx.statuslines = 1;
    unsafe {
        let mut session = view.session.handle().clone();
        session.as_session_mut().curw = None;
        screen_redraw_draw_status(&mut ctx);
        session.as_session_mut().curw = Some(99);
        screen_redraw_draw_status(&mut ctx);
        view.client.set_attached_session(None);
        screen_redraw_draw_status(&mut ctx);
    }
}
