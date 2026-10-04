//! Pane border cache.
use super::*;
use crate::src::format::{format_create_defaults, format_free};
use crate::src::grid::grid_default_cell;
use crate::src::style::style_apply_with_options;
use crate::src::window_border::window_get_border_cell;

pub(super) unsafe fn cell(pane: &Rc<UnsafeCell<window_pane>>, cell: &mut grid_cell) {
    let window = pane.window_observer().upgrade().expect("live pane parent");
    let index = window.pane_index(&Rc::downgrade(pane));
    window.release(c"pane border cell");
    window_get_border_cell(index, pane.pane_lines(), cell);
}

pub(super) unsafe fn border_style(
    pane: &Rc<UnsafeCell<window_pane>>,
    client: &ClientRef,
) -> grid_cell {
    let session = client
        .attached_session()
        .upgrade()
        .expect("live border session");
    let link = session.current_winlink();
    let window = link
        .get_unchecked()
        .window_handle()
        .expect("live border window")
        .clone();
    let active = window
        .active_pane()
        .is_some_and(|active| Rc::ptr_eq(&active, pane));
    window.release(c"pane border style");
    let cached = {
        let state = &*pane.get();
        if active {
            (state.active_border_gc_set != 0, state.active_border_gc)
        } else {
            (state.border_gc_set != 0, state.border_gc)
        }
    };
    if cached.0 {
        return cached.1;
    }
    let mut context = format_create_defaults(None, Some(client), Some(&session), link, Some(pane));
    let mut cell = grid_default_cell;
    let key = if active {
        c"pane-active-border-style"
    } else {
        c"pane-border-style"
    };
    style_apply_with_options(&mut cell, key, Some(&mut context), |visit| {
        pane.with_options_mut(|options| visit(options));
    });
    format_free(context);
    let state = &mut *pane.get();
    if active {
        state.active_border_gc = cell;
        state.active_border_gc_set = 1;
    } else {
        state.border_gc = cell;
        state.border_gc_set = 1;
    }
    cell
}
