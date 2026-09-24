use crate::src::ffi::libc::memcpy;
use crate::src::server_client::server_client_ensure_ranges;
use crate::src::shared::abi::*;
use crate::src::shared::arguments::args;
use crate::src::shared::client::*;
use crate::src::shared::client::{
    client, client_file, client_file_cb, client_file_entry, client_files, overlay_check_cb,
    overlay_draw_cb, overlay_free_cb, overlay_key_cb, overlay_mode_cb, overlay_resize_cb,
};
use crate::src::shared::colour::*;
use crate::src::shared::command::{cmd_find_state, cmd_list, cmdq_item, cmdq_list, cmds};
use crate::src::shared::control::control_state;
use crate::src::shared::display::*;
use crate::src::shared::display::{visible_range, visible_ranges};
use crate::src::shared::environment::environ;
use crate::src::shared::event::*;
use crate::src::shared::format::{format_job_tree, format_tree};
use crate::src::shared::grid::*;
use crate::src::shared::hyperlinks::hyperlinks;
use crate::src::shared::input::{input_ctx, input_request, input_requests};
use crate::src::shared::key::*;
use crate::src::shared::key::{
    key_binding, key_binding_entry, key_bindings, key_event, key_table, key_table_entry,
};
use crate::src::shared::layout::layout_geometry;
use crate::src::shared::layout::*;
use crate::src::shared::layout::*;
use crate::src::shared::layout::{layout_cell, layout_cell_entry, layout_cells};
use crate::src::shared::menu::menu_data;
use crate::src::shared::message::*;
use crate::src::shared::mouse::mouse_event;
use crate::src::shared::options::options;
use crate::src::shared::pane::{
    window_pane, window_pane_modes, window_pane_prompt, window_pane_tree_entry, window_panes,
};
use crate::src::shared::pane::{
    window_pane_offset, window_pane_resize, window_pane_resizes, PANE_SCROLLBARS_LEFT,
};
use crate::src::shared::process::tmuxpeer;
use crate::src::shared::prompt::prompt;
use crate::src::shared::redraw::redraw_scene;
use crate::src::shared::screen::{screen, screen_sel, screen_titles};
use crate::src::shared::screen_write::screen_write_cline;
use crate::src::shared::session::{session, session_entry};
use crate::src::shared::spawn::spawn_editor_state;
use crate::src::shared::status::status_line;
use crate::src::shared::style::*;
use crate::src::shared::terminal::*;
use crate::src::shared::tty::{tty, tty_code, tty_key, tty_term, tty_term_entry};
use crate::src::shared::window::{
    window, window_entry, window_mode, window_mode_entry, window_winlinks, winlink, winlink_entry,
    winlink_stack, winlinks,
};
use crate::src::window::{
    window_pane_ensure_visible_ranges, window_pane_get_pane_lines, window_pane_is_floating,
    window_pane_is_visible, window_pane_scrollbar_reserve, window_pane_z_last,
    window_pane_z_previous,
};

use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_13;
use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_12;

#[no_mangle]
pub unsafe extern "C" fn window_position_is_visible(
    mut r: *mut visible_ranges,
    mut px: u_int,
) -> ::core::ffi::c_int {
    let mut i: u_int = 0;
    let mut ri: *mut visible_range = ::core::ptr::null_mut::<visible_range>();
    if r.is_null() {
        return 1 as ::core::ffi::c_int;
    }
    i = 0 as u_int;
    while i < (*r).used {
        ri = (*r).ranges.offset(i as isize) as *mut visible_range;
        if (*ri).nx != 0 as u_int && px >= (*ri).px && px < (*ri).px.wrapping_add((*ri).nx) {
            return 1 as ::core::ffi::c_int;
        }
        i = i.wrapping_add(1);
    }
    return 0 as ::core::ffi::c_int;
}

/// Grow the owned range storage and refresh its synchronous pointer view.
/// Growing the vector invalidates prior element pointers.
unsafe fn window_visible_ensure_ranges(wp: *mut window_pane, r: *mut visible_ranges, n: u_int) {
    if r != &raw mut (*wp).r {
        server_client_ensure_ranges(r, n);
        return;
    }
    window_pane_ensure_visible_ranges(wp, n);
}

#[no_mangle]
pub unsafe extern "C" fn window_visible_ranges(
    mut base_wp: *mut window_pane,
    mut px: ::core::ffi::c_int,
    mut py: ::core::ffi::c_int,
    mut width: u_int,
    mut r: *mut visible_ranges,
) -> *mut visible_ranges {
    let mut current_block: u64;
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut ri: *mut visible_range = ::core::ptr::null_mut::<visible_range>();
    static mut sr: visible_ranges = visible_ranges {
        ranges: ::core::ptr::null_mut(),
        used: 0 as u_int,
        storage: Vec::new(),
    };
    let mut found_self: ::core::ffi::c_int = 0;
    let mut sb_w: ::core::ffi::c_int = 0;
    let mut sb_pos: ::core::ffi::c_int = 0;
    let mut lb: ::core::ffi::c_int = 0;
    let mut rb: ::core::ffi::c_int = 0;
    let mut tb: ::core::ffi::c_int = 0;
    let mut bb: ::core::ffi::c_int = 0;
    let mut sx: ::core::ffi::c_int = 0;
    let mut ex: ::core::ffi::c_int = 0;
    let mut no_border: ::core::ffi::c_int = 0;
    let mut i: u_int = 0;
    let mut s: u_int = 0;
    if !(py < 0 as ::core::ffi::c_int || width == 0 as u_int) {
        if px < 0 as ::core::ffi::c_int {
            if -px as u_int >= width {
                current_block = 14598516150315858936;
            } else {
                width = width.wrapping_sub(-px as u_int);
                px = 0 as ::core::ffi::c_int;
                current_block = 11006700562992250127;
            }
        } else {
            current_block = 11006700562992250127;
        }
        match current_block {
            14598516150315858936 => {}
            _ => {
                if base_wp.is_null() {
                    if !r.is_null() {
                        return r;
                    }
                    sr.ensure(1);
                    (*sr.ranges.offset(0 as ::core::ffi::c_int as isize)).px = px as u_int;
                    (*sr.ranges.offset(0 as ::core::ffi::c_int as isize)).nx = width;
                    sr.used = 1 as u_int;
                    return &raw mut sr;
                }
                w = (*base_wp).window as *mut window;
                if !(py as u_int >= (*w).sy || px as u_int >= (*w).sx) {
                    if (px as u_int).wrapping_add(width) > (*w).sx {
                        width = (*w).sx.wrapping_sub(px as u_int);
                    }
                    if r.is_null() {
                        window_visible_ensure_ranges(base_wp, &raw mut (*base_wp).r, 1 as u_int);
                        r = &raw mut (*base_wp).r;
                        (*(*r).ranges.offset(0 as ::core::ffi::c_int as isize)).px = px as u_int;
                        (*(*r).ranges.offset(0 as ::core::ffi::c_int as isize)).nx = width;
                        (*r).used = 1 as u_int;
                    }
                    found_self = 0 as ::core::ffi::c_int;
                    wp = window_pane_z_last(w);
                    while !wp.is_null() {
                        if wp == base_wp {
                            found_self = 1 as ::core::ffi::c_int;
                        } else {
                            if window_pane_is_floating(wp) != 0
                                && window_pane_get_pane_lines(wp) as ::core::ffi::c_uint
                                    == PANE_LINES_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
                            {
                                no_border = 1 as ::core::ffi::c_int;
                            } else {
                                no_border = 0 as ::core::ffi::c_int;
                            }
                            if no_border != 0 {
                                tb = (*wp).yoff;
                                bb = (*wp).yoff + (*wp).sy as ::core::ffi::c_int
                                    - 1 as ::core::ffi::c_int;
                            } else {
                                tb = if (*wp).yoff > 0 as ::core::ffi::c_int {
                                    (*wp).yoff - 1 as ::core::ffi::c_int
                                } else {
                                    0 as ::core::ffi::c_int
                                };
                                bb = (*wp).yoff + (*wp).sy as ::core::ffi::c_int;
                            }
                            if !(found_self == 0
                                || window_pane_is_visible(wp) == 0
                                || py < tb
                                || py > bb)
                            {
                                if !(window_pane_is_floating(wp) == 0 && (py == tb || py == bb)) {
                                    sb_w = (*wp).scrollbar_style.width + (*wp).scrollbar_style.pad;
                                    if window_pane_scrollbar_reserve(wp) != 0 {
                                        sb_pos = (*w).sb_pos;
                                    } else {
                                        sb_pos = 0 as ::core::ffi::c_int;
                                        sb_w = sb_pos;
                                    }
                                    i = 0 as u_int;
                                    while i < (*r).used {
                                        ri = (*r).ranges.offset(i as isize) as *mut visible_range;
                                        if !((*ri).nx == 0 as u_int) {
                                            if no_border != 0 {
                                                lb = (*wp).xoff;
                                                rb = (*wp).xoff + (*wp).sx as ::core::ffi::c_int
                                                    - 1 as ::core::ffi::c_int;
                                            } else if sb_pos == PANE_SCROLLBARS_LEFT {
                                                if (*wp).xoff > sb_w {
                                                    lb =
                                                        (*wp).xoff - 1 as ::core::ffi::c_int - sb_w;
                                                } else {
                                                    lb = 0 as ::core::ffi::c_int;
                                                }
                                            } else if (*wp).xoff > 0 as ::core::ffi::c_int {
                                                lb = (*wp).xoff - 1 as ::core::ffi::c_int;
                                            } else {
                                                lb = 0 as ::core::ffi::c_int;
                                            }
                                            if no_border == 0 {
                                                if sb_pos == PANE_SCROLLBARS_LEFT {
                                                    rb =
                                                        (*wp).xoff + (*wp).sx as ::core::ffi::c_int;
                                                } else {
                                                    rb = (*wp).xoff
                                                        + (*wp).sx as ::core::ffi::c_int
                                                        + sb_w;
                                                }
                                            }
                                            if lb < 0 as ::core::ffi::c_int {
                                                lb = 0 as ::core::ffi::c_int;
                                            }
                                            if !(rb < 0 as ::core::ffi::c_int) {
                                                if no_border != 0
                                                    && rb >= (*w).sx as ::core::ffi::c_int
                                                {
                                                    rb = (*w).sx.wrapping_sub(1 as u_int)
                                                        as ::core::ffi::c_int;
                                                } else if no_border == 0
                                                    && rb > (*w).sx as ::core::ffi::c_int
                                                {
                                                    rb = (*w).sx.wrapping_sub(1 as u_int)
                                                        as ::core::ffi::c_int;
                                                }
                                                if !(lb > rb) {
                                                    sx = (*ri).px as ::core::ffi::c_int;
                                                    ex = (sx as u_int)
                                                        .wrapping_add((*ri).nx)
                                                        .wrapping_sub(1 as u_int)
                                                        as ::core::ffi::c_int;
                                                    if lb > sx && lb <= ex && rb > ex {
                                                        (*ri).nx = (lb - sx) as u_int;
                                                    } else if rb >= sx && rb <= ex && lb <= sx {
                                                        (*ri).nx = (ex - rb) as u_int;
                                                        (*ri).px =
                                                            (rb + 1 as ::core::ffi::c_int) as u_int;
                                                    } else if lb > sx && rb <= ex {
                                                        window_visible_ensure_ranges(
                                                            base_wp,
                                                            r,
                                                            (*r).used.wrapping_add(1 as u_int),
                                                        );
                                                        s = (*r).used;
                                                        while s > i {
                                                            memcpy(
                                                                (*r).ranges.offset(s as isize)
                                                                    as *mut visible_range
                                                                    as *mut ::core::ffi::c_void,
                                                                (*r).ranges.offset(
                                                                    s.wrapping_sub(1 as u_int)
                                                                        as isize,
                                                                )
                                                                    as *mut visible_range
                                                                    as *const ::core::ffi::c_void,
                                                                ::core::mem::size_of::<visible_range>(
                                                                )
                                                                    as size_t,
                                                            );
                                                            s = s.wrapping_sub(1);
                                                        }
                                                        ri = (*r).ranges.offset(i as isize)
                                                            as *mut visible_range;
                                                        (*(*r).ranges.offset(
                                                            i.wrapping_add(1 as u_int) as isize,
                                                        ))
                                                        .px =
                                                            (rb + 1 as ::core::ffi::c_int) as u_int;
                                                        (*(*r).ranges.offset(
                                                            i.wrapping_add(1 as u_int) as isize,
                                                        ))
                                                        .nx = (ex - rb) as u_int;
                                                        (*ri).nx = (lb - sx) as u_int;
                                                        (*r).used = (*r).used.wrapping_add(1);
                                                    } else if lb <= sx && rb > ex {
                                                        (*ri).nx = 0 as u_int;
                                                    }
                                                }
                                            }
                                        }
                                        i = i.wrapping_add(1);
                                    }
                                }
                            }
                        }
                        wp = window_pane_z_previous(wp);
                    }
                    return r;
                }
            }
        }
    }
    if r.is_null() {
        sr.used = 0 as u_int;
        return &raw mut sr;
    }
    (*r).used = 0 as u_int;
    return r;
}
