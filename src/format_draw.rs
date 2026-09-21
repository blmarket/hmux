use crate::src::ffi::libc::{free, memcpy, memset, strcmp, strlcpy, strlen};
use crate::src::format::format_skip;
use crate::src::grid::grid_default_cell;
use crate::src::hyperlinks::hyperlinks_put;
use crate::src::log::log_debug;
use crate::src::screen::{screen_free, screen_init};
use crate::src::screen_write::{
    screen_write_cell, screen_write_clearendofline, screen_write_cursormove,
    screen_write_fast_copy, screen_write_putc, screen_write_start, screen_write_stop,
};
use crate::src::style::{style_copy, style_link, style_parse, style_set, style_tostring};
use crate::src::utf8::{utf8_append, utf8_open, utf8_set};
use crate::src::xmalloc::{xcalloc, xstrdup, xstrndup};
pub use crate::src::shared::arguments::{args};
pub use crate::src::shared::client::{
    client, client_entry, client_file, client_file_cb, client_file_entry, client_files,
    overlay_check_cb, overlay_draw_cb, overlay_free_cb, overlay_key_cb, overlay_mode_cb,
    overlay_resize_cb,
};
pub use crate::src::shared::command::{cmd_find_state, cmd_list, cmdq_item, cmdq_list, cmds};
pub use crate::src::shared::control::{control_state};
pub use crate::src::shared::format::{format_job_tree, format_tree};
pub use crate::src::shared::input::{input_ctx, input_request, input_requests};
pub use crate::src::shared::key::{
    key_binding, key_binding_entry, key_bindings, key_event, key_table, key_table_entry,
};
pub use crate::src::shared::layout::{layout_cell, layout_cell_entry, layout_cells};
pub use crate::src::shared::menu::{menu_data};
pub use crate::src::shared::options::{options};
pub use crate::src::shared::pane::{
    window_pane, window_pane_entry, window_pane_modes, window_pane_prompt, window_pane_sentry,
    window_pane_tree_entry, window_pane_zentry, window_panes,
};
pub use crate::src::shared::process::{tmuxpeer};
pub use crate::src::shared::prompt::{prompt};
pub use crate::src::shared::redraw::{redraw_scene};
pub use crate::src::shared::screen_write::{screen_write_ctx, screen_write_init_ctx_cb};
pub use crate::src::shared::session::{session, session_entry, session_gentry};
pub use crate::src::shared::spawn::{spawn_editor_state};
pub use crate::src::shared::status::{status_line};
pub use crate::src::shared::tty::{
    tty, tty_code, tty_ctx, tty_ctx_c2rust_unnamed, tty_ctx_c2rust_unnamed_data,
    tty_ctx_c2rust_unnamed_sel, tty_ctx_redraw_cb, tty_ctx_set_client_cb, tty_key,
    tty_style_ctx, tty_term, tty_term_entry,
};
pub use crate::src::shared::window::{
    window, window_alerts_entry, window_entry, window_mode, window_mode_entry,
    window_mode_entry_entry, window_winlinks, winlink, winlink_entry, winlink_sentry,
    winlink_stack, winlink_wentry, winlinks,
};
pub use crate::src::shared::environment::{environ};
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::{screen_write_citem, screen_write_cline};
pub use crate::src::shared::hyperlinks::{hyperlinks};
pub use crate::src::shared::pane::{
    window_pane_offset, window_pane_resize, window_pane_resize_entry, window_pane_resizes,
};
pub use crate::src::shared::display::{visible_range, visible_ranges};
pub use crate::src::shared::layout::{layout_geometry};
pub use crate::src::shared::mouse::{mouse_event};
use crate::src::shared::client::*;
use crate::src::shared::terminal::*;
use crate::src::shared::event::*;
use crate::src::shared::display::*;
use crate::src::shared::layout::*;
use crate::src::shared::message::*;
use crate::src::shared::abi::*;
use crate::src::shared::colour::*;
use crate::src::shared::grid::*;
use crate::src::shared::key::*;
use crate::src::shared::style::*;
use crate::src::shared::utf8::*;

pub use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_12;
pub use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_13;

pub const TOTAL: C2RustUnnamed_39 = 8;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct format_range {
    pub index: u_int,
    pub s: *mut screen,
    pub start: u_int,
    pub end: u_int,
    pub type_0: style_range_type,
    pub argument: u_int,
    pub string: [::core::ffi::c_char; 16],
    pub entry: C2RustUnnamed_38,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_38 {
    pub tqe_next: *mut format_range,
    pub tqe_prev: *mut *mut format_range,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct format_ranges {
    pub tqh_first: *mut format_range,
    pub tqh_last: *mut *mut format_range,
}
pub const AFTER: C2RustUnnamed_39 = 7;
pub const LIST_RIGHT: C2RustUnnamed_39 = 6;
pub const LIST_LEFT: C2RustUnnamed_39 = 5;
pub const LIST: C2RustUnnamed_39 = 4;
pub const ABSOLUTE_CENTRE: C2RustUnnamed_39 = 3;
pub const RIGHT: C2RustUnnamed_39 = 2;
pub const CENTRE: C2RustUnnamed_39 = 1;
pub const LEFT: C2RustUnnamed_39 = 0;
pub type C2RustUnnamed_39 = ::core::ffi::c_uint;
unsafe extern "C" fn format_is_type(
    mut fr: *mut format_range,
    mut sy: *mut style,
) -> ::core::ffi::c_int {
    if (*fr).type_0 as ::core::ffi::c_uint != (*sy).range_type as ::core::ffi::c_uint {
        return 0 as ::core::ffi::c_int;
    }
    match (*fr).type_0 as ::core::ffi::c_uint {
        0 | 1 | 2 | 7 => return 1 as ::core::ffi::c_int,
        3 | 4 | 5 => {
            return ((*fr).argument == (*sy).range_argument) as ::core::ffi::c_int;
        }
        6 => {
            return (strcmp(
                &raw mut (*fr).string as *mut ::core::ffi::c_char,
                &raw mut (*sy).range_string as *mut ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
        }
        _ => {}
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn format_free_range(mut frs: *mut format_ranges, mut fr: *mut format_range) {
    if !(*fr).entry.tqe_next.is_null() {
        (*(*fr).entry.tqe_next).entry.tqe_prev = (*fr).entry.tqe_prev;
    } else {
        (*frs).tqh_last = (*fr).entry.tqe_prev;
    }
    *(*fr).entry.tqe_prev = (*fr).entry.tqe_next;
    free(fr as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn format_update_ranges(
    mut frs: *mut format_ranges,
    mut s: *mut screen,
    mut offset: u_int,
    mut start: u_int,
    mut width: u_int,
) {
    let mut fr: *mut format_range = ::core::ptr::null_mut::<format_range>();
    let mut fr1: *mut format_range = ::core::ptr::null_mut::<format_range>();
    if frs.is_null() {
        return;
    }
    fr = (*frs).tqh_first;
    while !fr.is_null() && {
        fr1 = (*fr).entry.tqe_next;
        1 as ::core::ffi::c_int != 0
    } {
        if !((*fr).s != s) {
            if (*fr).end <= start || (*fr).start >= start.wrapping_add(width) {
                format_free_range(frs, fr);
            } else {
                if (*fr).start < start {
                    (*fr).start = start;
                }
                if (*fr).end > start.wrapping_add(width) {
                    (*fr).end = start.wrapping_add(width);
                }
                if (*fr).start == (*fr).end {
                    format_free_range(frs, fr);
                } else {
                    (*fr).start = (*fr).start.wrapping_sub(start);
                    (*fr).end = (*fr).end.wrapping_sub(start);
                    (*fr).start = (*fr).start.wrapping_add(offset);
                    (*fr).end = (*fr).end.wrapping_add(offset);
                }
            }
        }
        fr = fr1;
    }
}
unsafe extern "C" fn format_draw_put(
    mut octx: *mut screen_write_ctx,
    mut ocx: u_int,
    mut ocy: u_int,
    mut s: *mut screen,
    mut frs: *mut format_ranges,
    mut offset: u_int,
    mut start: u_int,
    mut width: u_int,
) {
    screen_write_cursormove(
        octx,
        ocx.wrapping_add(offset) as ::core::ffi::c_int,
        ocy as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    screen_write_fast_copy(octx, s, start, 0 as u_int, width, 1 as u_int);
    format_update_ranges(frs, s, offset, start, width);
}
unsafe extern "C" fn format_draw_put_list(
    mut octx: *mut screen_write_ctx,
    mut ocx: u_int,
    mut ocy: u_int,
    mut offset: u_int,
    mut width: u_int,
    mut list: *mut screen,
    mut list_left: *mut screen,
    mut list_right: *mut screen,
    mut focus_start: ::core::ffi::c_int,
    mut focus_end: ::core::ffi::c_int,
    mut frs: *mut format_ranges,
) {
    let mut start: u_int = 0;
    let mut focus_centre: u_int = 0;
    if width >= (*list).cx {
        format_draw_put(octx, ocx, ocy, list, frs, offset, 0 as u_int, width);
        return;
    }
    focus_centre = (focus_start + (focus_end - focus_start) / 2 as ::core::ffi::c_int) as u_int;
    if focus_centre < width.wrapping_div(2 as u_int) {
        start = 0 as u_int;
    } else {
        start = focus_centre.wrapping_sub(width.wrapping_div(2 as u_int));
    }
    if start.wrapping_add(width) > (*list).cx {
        start = (*list).cx.wrapping_sub(width);
    }
    if start != 0 as u_int && width > (*list_left).cx {
        screen_write_cursormove(
            octx,
            ocx.wrapping_add(offset) as ::core::ffi::c_int,
            ocy as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        screen_write_fast_copy(
            octx,
            list_left,
            0 as u_int,
            0 as u_int,
            (*list_left).cx,
            1 as u_int,
        );
        offset = offset.wrapping_add((*list_left).cx);
        start = start.wrapping_add((*list_left).cx);
        width = width.wrapping_sub((*list_left).cx);
    }
    if start.wrapping_add(width) < (*list).cx && width > (*list_right).cx {
        screen_write_cursormove(
            octx,
            ocx.wrapping_add(offset)
                .wrapping_add(width)
                .wrapping_sub((*list_right).cx) as ::core::ffi::c_int,
            ocy as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        screen_write_fast_copy(
            octx,
            list_right,
            0 as u_int,
            0 as u_int,
            (*list_right).cx,
            1 as u_int,
        );
        width = width.wrapping_sub((*list_right).cx);
    }
    format_draw_put(octx, ocx, ocy, list, frs, offset, start, width);
}
unsafe extern "C" fn format_draw_none(
    mut octx: *mut screen_write_ctx,
    mut available: u_int,
    mut ocx: u_int,
    mut ocy: u_int,
    mut left: *mut screen,
    mut centre: *mut screen,
    mut right: *mut screen,
    mut abs_centre: *mut screen,
    mut frs: *mut format_ranges,
) {
    let mut width_left: u_int = 0;
    let mut width_centre: u_int = 0;
    let mut width_right: u_int = 0;
    let mut width_abs_centre: u_int = 0;
    width_left = (*left).cx;
    width_centre = (*centre).cx;
    width_right = (*right).cx;
    width_abs_centre = (*abs_centre).cx;
    while width_left
        .wrapping_add(width_centre)
        .wrapping_add(width_right)
        > available
    {
        if width_centre > 0 as u_int {
            width_centre = width_centre.wrapping_sub(1);
        } else if width_right > 0 as u_int {
            width_right = width_right.wrapping_sub(1);
        } else {
            width_left = width_left.wrapping_sub(1);
        }
    }
    format_draw_put(
        octx, ocx, ocy, left, frs, 0 as u_int, 0 as u_int, width_left,
    );
    format_draw_put(
        octx,
        ocx,
        ocy,
        right,
        frs,
        available.wrapping_sub(width_right),
        (*right).cx.wrapping_sub(width_right),
        width_right,
    );
    format_draw_put(
        octx,
        ocx,
        ocy,
        centre,
        frs,
        width_left
            .wrapping_add(
                available
                    .wrapping_sub(width_right)
                    .wrapping_sub(width_left)
                    .wrapping_div(2 as u_int),
            )
            .wrapping_sub(width_centre.wrapping_div(2 as u_int)),
        (*centre)
            .cx
            .wrapping_div(2 as u_int)
            .wrapping_sub(width_centre.wrapping_div(2 as u_int)),
        width_centre,
    );
    if width_abs_centre > available {
        width_abs_centre = available;
    }
    format_draw_put(
        octx,
        ocx,
        ocy,
        abs_centre,
        frs,
        available
            .wrapping_sub(width_abs_centre)
            .wrapping_div(2 as u_int),
        0 as u_int,
        width_abs_centre,
    );
}
unsafe extern "C" fn format_draw_left(
    mut octx: *mut screen_write_ctx,
    mut available: u_int,
    mut ocx: u_int,
    mut ocy: u_int,
    mut left: *mut screen,
    mut centre: *mut screen,
    mut right: *mut screen,
    mut abs_centre: *mut screen,
    mut list: *mut screen,
    mut list_left: *mut screen,
    mut list_right: *mut screen,
    mut after: *mut screen,
    mut focus_start: ::core::ffi::c_int,
    mut focus_end: ::core::ffi::c_int,
    mut frs: *mut format_ranges,
) {
    let mut width_left: u_int = 0;
    let mut width_centre: u_int = 0;
    let mut width_right: u_int = 0;
    let mut width_list: u_int = 0;
    let mut width_after: u_int = 0;
    let mut width_abs_centre: u_int = 0;
    let mut ctx: screen_write_ctx = screen_write_ctx {
        wp: ::core::ptr::null_mut::<window_pane>(),
        s: ::core::ptr::null_mut::<screen>(),
        flags: 0,
        init_ctx_cb: None,
        arg: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        item: ::core::ptr::null_mut::<screen_write_citem>(),
        scrolled: 0,
        bg: 0,
    };
    width_left = (*left).cx;
    width_centre = (*centre).cx;
    width_right = (*right).cx;
    width_abs_centre = (*abs_centre).cx;
    width_list = (*list).cx;
    width_after = (*after).cx;
    while width_left
        .wrapping_add(width_centre)
        .wrapping_add(width_right)
        .wrapping_add(width_list)
        .wrapping_add(width_after)
        > available
    {
        if width_centre > 0 as u_int {
            width_centre = width_centre.wrapping_sub(1);
        } else if width_list > 0 as u_int {
            width_list = width_list.wrapping_sub(1);
        } else if width_right > 0 as u_int {
            width_right = width_right.wrapping_sub(1);
        } else if width_after > 0 as u_int {
            width_after = width_after.wrapping_sub(1);
        } else {
            width_left = width_left.wrapping_sub(1);
        }
    }
    if width_list == 0 as u_int {
        screen_write_start(&raw mut ctx, left);
        screen_write_fast_copy(
            &raw mut ctx,
            after,
            0 as u_int,
            0 as u_int,
            width_after,
            1 as u_int,
        );
        screen_write_stop(&raw mut ctx);
        format_draw_none(
            octx, available, ocx, ocy, left, centre, right, abs_centre, frs,
        );
        return;
    }
    format_draw_put(
        octx, ocx, ocy, left, frs, 0 as u_int, 0 as u_int, width_left,
    );
    format_draw_put(
        octx,
        ocx,
        ocy,
        right,
        frs,
        available.wrapping_sub(width_right),
        (*right).cx.wrapping_sub(width_right),
        width_right,
    );
    format_draw_put(
        octx,
        ocx,
        ocy,
        after,
        frs,
        width_left.wrapping_add(width_list),
        0 as u_int,
        width_after,
    );
    format_draw_put(
        octx,
        ocx,
        ocy,
        centre,
        frs,
        width_left
            .wrapping_add(width_list)
            .wrapping_add(width_after)
            .wrapping_add(
                available
                    .wrapping_sub(width_right)
                    .wrapping_sub(
                        width_left
                            .wrapping_add(width_list)
                            .wrapping_add(width_after),
                    )
                    .wrapping_div(2 as u_int),
            )
            .wrapping_sub(width_centre.wrapping_div(2 as u_int)),
        (*centre)
            .cx
            .wrapping_div(2 as u_int)
            .wrapping_sub(width_centre.wrapping_div(2 as u_int)),
        width_centre,
    );
    if focus_start == -(1 as ::core::ffi::c_int) || focus_end == -(1 as ::core::ffi::c_int) {
        focus_end = 0 as ::core::ffi::c_int;
        focus_start = focus_end;
    }
    format_draw_put_list(
        octx,
        ocx,
        ocy,
        width_left,
        width_list,
        list,
        list_left,
        list_right,
        focus_start,
        focus_end,
        frs,
    );
    if width_abs_centre > available {
        width_abs_centre = available;
    }
    format_draw_put(
        octx,
        ocx,
        ocy,
        abs_centre,
        frs,
        available
            .wrapping_sub(width_abs_centre)
            .wrapping_div(2 as u_int),
        0 as u_int,
        width_abs_centre,
    );
}
unsafe extern "C" fn format_draw_centre(
    mut octx: *mut screen_write_ctx,
    mut available: u_int,
    mut ocx: u_int,
    mut ocy: u_int,
    mut left: *mut screen,
    mut centre: *mut screen,
    mut right: *mut screen,
    mut abs_centre: *mut screen,
    mut list: *mut screen,
    mut list_left: *mut screen,
    mut list_right: *mut screen,
    mut after: *mut screen,
    mut focus_start: ::core::ffi::c_int,
    mut focus_end: ::core::ffi::c_int,
    mut frs: *mut format_ranges,
) {
    let mut width_left: u_int = 0;
    let mut width_centre: u_int = 0;
    let mut width_right: u_int = 0;
    let mut middle: u_int = 0;
    let mut width_list: u_int = 0;
    let mut width_after: u_int = 0;
    let mut width_abs_centre: u_int = 0;
    let mut ctx: screen_write_ctx = screen_write_ctx {
        wp: ::core::ptr::null_mut::<window_pane>(),
        s: ::core::ptr::null_mut::<screen>(),
        flags: 0,
        init_ctx_cb: None,
        arg: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        item: ::core::ptr::null_mut::<screen_write_citem>(),
        scrolled: 0,
        bg: 0,
    };
    width_left = (*left).cx;
    width_centre = (*centre).cx;
    width_right = (*right).cx;
    width_abs_centre = (*abs_centre).cx;
    width_list = (*list).cx;
    width_after = (*after).cx;
    while width_left
        .wrapping_add(width_centre)
        .wrapping_add(width_right)
        .wrapping_add(width_list)
        .wrapping_add(width_after)
        > available
    {
        if width_list > 0 as u_int {
            width_list = width_list.wrapping_sub(1);
        } else if width_after > 0 as u_int {
            width_after = width_after.wrapping_sub(1);
        } else if width_centre > 0 as u_int {
            width_centre = width_centre.wrapping_sub(1);
        } else if width_right > 0 as u_int {
            width_right = width_right.wrapping_sub(1);
        } else {
            width_left = width_left.wrapping_sub(1);
        }
    }
    if width_list == 0 as u_int {
        screen_write_start(&raw mut ctx, centre);
        screen_write_fast_copy(
            &raw mut ctx,
            after,
            0 as u_int,
            0 as u_int,
            width_after,
            1 as u_int,
        );
        screen_write_stop(&raw mut ctx);
        format_draw_none(
            octx, available, ocx, ocy, left, centre, right, abs_centre, frs,
        );
        return;
    }
    format_draw_put(
        octx, ocx, ocy, left, frs, 0 as u_int, 0 as u_int, width_left,
    );
    format_draw_put(
        octx,
        ocx,
        ocy,
        right,
        frs,
        available.wrapping_sub(width_right),
        (*right).cx.wrapping_sub(width_right),
        width_right,
    );
    middle = width_left.wrapping_add(
        available
            .wrapping_sub(width_right)
            .wrapping_sub(width_left)
            .wrapping_div(2 as u_int),
    );
    format_draw_put(
        octx,
        ocx,
        ocy,
        centre,
        frs,
        middle
            .wrapping_sub(width_list.wrapping_div(2 as u_int))
            .wrapping_sub(width_centre),
        0 as u_int,
        width_centre,
    );
    format_draw_put(
        octx,
        ocx,
        ocy,
        after,
        frs,
        middle
            .wrapping_sub(width_list.wrapping_div(2 as u_int))
            .wrapping_add(width_list),
        0 as u_int,
        width_after,
    );
    if focus_start == -(1 as ::core::ffi::c_int) || focus_end == -(1 as ::core::ffi::c_int) {
        focus_end = (*list).cx.wrapping_div(2 as u_int) as ::core::ffi::c_int;
        focus_start = focus_end;
    }
    format_draw_put_list(
        octx,
        ocx,
        ocy,
        middle.wrapping_sub(width_list.wrapping_div(2 as u_int)),
        width_list,
        list,
        list_left,
        list_right,
        focus_start,
        focus_end,
        frs,
    );
    if width_abs_centre > available {
        width_abs_centre = available;
    }
    format_draw_put(
        octx,
        ocx,
        ocy,
        abs_centre,
        frs,
        available
            .wrapping_sub(width_abs_centre)
            .wrapping_div(2 as u_int),
        0 as u_int,
        width_abs_centre,
    );
}
unsafe extern "C" fn format_draw_right(
    mut octx: *mut screen_write_ctx,
    mut available: u_int,
    mut ocx: u_int,
    mut ocy: u_int,
    mut left: *mut screen,
    mut centre: *mut screen,
    mut right: *mut screen,
    mut abs_centre: *mut screen,
    mut list: *mut screen,
    mut list_left: *mut screen,
    mut list_right: *mut screen,
    mut after: *mut screen,
    mut focus_start: ::core::ffi::c_int,
    mut focus_end: ::core::ffi::c_int,
    mut frs: *mut format_ranges,
) {
    let mut width_left: u_int = 0;
    let mut width_centre: u_int = 0;
    let mut width_right: u_int = 0;
    let mut width_list: u_int = 0;
    let mut width_after: u_int = 0;
    let mut width_abs_centre: u_int = 0;
    let mut ctx: screen_write_ctx = screen_write_ctx {
        wp: ::core::ptr::null_mut::<window_pane>(),
        s: ::core::ptr::null_mut::<screen>(),
        flags: 0,
        init_ctx_cb: None,
        arg: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        item: ::core::ptr::null_mut::<screen_write_citem>(),
        scrolled: 0,
        bg: 0,
    };
    width_left = (*left).cx;
    width_centre = (*centre).cx;
    width_right = (*right).cx;
    width_abs_centre = (*abs_centre).cx;
    width_list = (*list).cx;
    width_after = (*after).cx;
    while width_left
        .wrapping_add(width_centre)
        .wrapping_add(width_right)
        .wrapping_add(width_list)
        .wrapping_add(width_after)
        > available
    {
        if width_centre > 0 as u_int {
            width_centre = width_centre.wrapping_sub(1);
        } else if width_list > 0 as u_int {
            width_list = width_list.wrapping_sub(1);
        } else if width_right > 0 as u_int {
            width_right = width_right.wrapping_sub(1);
        } else if width_after > 0 as u_int {
            width_after = width_after.wrapping_sub(1);
        } else {
            width_left = width_left.wrapping_sub(1);
        }
    }
    if width_list == 0 as u_int {
        screen_write_start(&raw mut ctx, right);
        screen_write_fast_copy(
            &raw mut ctx,
            after,
            0 as u_int,
            0 as u_int,
            width_after,
            1 as u_int,
        );
        screen_write_stop(&raw mut ctx);
        format_draw_none(
            octx, available, ocx, ocy, left, centre, right, abs_centre, frs,
        );
        return;
    }
    format_draw_put(
        octx, ocx, ocy, left, frs, 0 as u_int, 0 as u_int, width_left,
    );
    format_draw_put(
        octx,
        ocx,
        ocy,
        after,
        frs,
        available.wrapping_sub(width_after),
        (*after).cx.wrapping_sub(width_after),
        width_after,
    );
    format_draw_put(
        octx,
        ocx,
        ocy,
        right,
        frs,
        available
            .wrapping_sub(width_right)
            .wrapping_sub(width_list)
            .wrapping_sub(width_after),
        0 as u_int,
        width_right,
    );
    format_draw_put(
        octx,
        ocx,
        ocy,
        centre,
        frs,
        width_left
            .wrapping_add(
                available
                    .wrapping_sub(width_right)
                    .wrapping_sub(width_list)
                    .wrapping_sub(width_after)
                    .wrapping_sub(width_left)
                    .wrapping_div(2 as u_int),
            )
            .wrapping_sub(width_centre.wrapping_div(2 as u_int)),
        (*centre)
            .cx
            .wrapping_div(2 as u_int)
            .wrapping_sub(width_centre.wrapping_div(2 as u_int)),
        width_centre,
    );
    if focus_start == -(1 as ::core::ffi::c_int) || focus_end == -(1 as ::core::ffi::c_int) {
        focus_end = 0 as ::core::ffi::c_int;
        focus_start = focus_end;
    }
    format_draw_put_list(
        octx,
        ocx,
        ocy,
        available.wrapping_sub(width_list).wrapping_sub(width_after),
        width_list,
        list,
        list_left,
        list_right,
        focus_start,
        focus_end,
        frs,
    );
    if width_abs_centre > available {
        width_abs_centre = available;
    }
    format_draw_put(
        octx,
        ocx,
        ocy,
        abs_centre,
        frs,
        available
            .wrapping_sub(width_abs_centre)
            .wrapping_div(2 as u_int),
        0 as u_int,
        width_abs_centre,
    );
}
unsafe extern "C" fn format_draw_absolute_centre(
    mut octx: *mut screen_write_ctx,
    mut available: u_int,
    mut ocx: u_int,
    mut ocy: u_int,
    mut left: *mut screen,
    mut centre: *mut screen,
    mut right: *mut screen,
    mut abs_centre: *mut screen,
    mut list: *mut screen,
    mut list_left: *mut screen,
    mut list_right: *mut screen,
    mut after: *mut screen,
    mut focus_start: ::core::ffi::c_int,
    mut focus_end: ::core::ffi::c_int,
    mut frs: *mut format_ranges,
) {
    let mut width_left: u_int = 0;
    let mut width_centre: u_int = 0;
    let mut width_right: u_int = 0;
    let mut width_abs_centre: u_int = 0;
    let mut width_list: u_int = 0;
    let mut width_after: u_int = 0;
    let mut middle: u_int = 0;
    let mut abs_centre_offset: u_int = 0;
    width_left = (*left).cx;
    width_centre = (*centre).cx;
    width_right = (*right).cx;
    width_abs_centre = (*abs_centre).cx;
    width_list = (*list).cx;
    width_after = (*after).cx;
    while width_left
        .wrapping_add(width_centre)
        .wrapping_add(width_right)
        > available
    {
        if width_centre > 0 as u_int {
            width_centre = width_centre.wrapping_sub(1);
        } else if width_right > 0 as u_int {
            width_right = width_right.wrapping_sub(1);
        } else {
            width_left = width_left.wrapping_sub(1);
        }
    }
    while width_list
        .wrapping_add(width_after)
        .wrapping_add(width_abs_centre)
        > available
    {
        if width_list > 0 as u_int {
            width_list = width_list.wrapping_sub(1);
        } else if width_after > 0 as u_int {
            width_after = width_after.wrapping_sub(1);
        } else {
            width_abs_centre = width_abs_centre.wrapping_sub(1);
        }
    }
    format_draw_put(
        octx, ocx, ocy, left, frs, 0 as u_int, 0 as u_int, width_left,
    );
    format_draw_put(
        octx,
        ocx,
        ocy,
        right,
        frs,
        available.wrapping_sub(width_right),
        (*right).cx.wrapping_sub(width_right),
        width_right,
    );
    middle = width_left.wrapping_add(
        available
            .wrapping_sub(width_right)
            .wrapping_sub(width_left)
            .wrapping_div(2 as u_int),
    );
    format_draw_put(
        octx,
        ocx,
        ocy,
        centre,
        frs,
        middle.wrapping_sub(width_centre),
        0 as u_int,
        width_centre,
    );
    if focus_start == -(1 as ::core::ffi::c_int) || focus_end == -(1 as ::core::ffi::c_int) {
        focus_end = (*list).cx.wrapping_div(2 as u_int) as ::core::ffi::c_int;
        focus_start = focus_end;
    }
    abs_centre_offset = available
        .wrapping_sub(width_list)
        .wrapping_sub(width_abs_centre)
        .wrapping_div(2 as u_int);
    format_draw_put(
        octx,
        ocx,
        ocy,
        abs_centre,
        frs,
        abs_centre_offset,
        0 as u_int,
        width_abs_centre,
    );
    abs_centre_offset = abs_centre_offset.wrapping_add(width_abs_centre);
    format_draw_put_list(
        octx,
        ocx,
        ocy,
        abs_centre_offset,
        width_list,
        list,
        list_left,
        list_right,
        focus_start,
        focus_end,
        frs,
    );
    abs_centre_offset = abs_centre_offset.wrapping_add(width_list);
    format_draw_put(
        octx,
        ocx,
        ocy,
        after,
        frs,
        abs_centre_offset,
        0 as u_int,
        width_after,
    );
}
unsafe extern "C" fn format_leading_hashes(
    mut cp: *const ::core::ffi::c_char,
    mut n: *mut u_int,
    mut width: *mut u_int,
) -> *const ::core::ffi::c_char {
    *n = 0 as u_int;
    while *cp.offset(*n as isize) as ::core::ffi::c_int == '#' as i32 {
        *n = (*n).wrapping_add(1);
    }
    if *n == 0 as u_int {
        *width = 0 as u_int;
        return cp;
    }
    if *cp.offset(*n as isize) as ::core::ffi::c_int != '[' as i32 {
        if (*n).wrapping_rem(2 as u_int) == 0 as u_int {
            *width = (*n).wrapping_div(2 as u_int);
        } else {
            *width = (*n).wrapping_div(2 as u_int).wrapping_add(1 as u_int);
        }
        return cp.offset(*n as isize);
    }
    *width = (*n).wrapping_div(2 as u_int);
    if (*n).wrapping_rem(2 as u_int) == 0 as u_int {
        return cp.offset(*n as isize);
    }
    return cp
        .offset(*n as isize)
        .offset(-(1 as ::core::ffi::c_int as isize));
}
unsafe extern "C" fn format_draw_many(
    mut ctx: *mut screen_write_ctx,
    mut sy: *mut style,
    mut ch: ::core::ffi::c_char,
    mut n: u_int,
) {
    let mut i: u_int = 0;
    utf8_set(&raw mut (*sy).gc.data, ch as u_char);
    i = 0 as u_int;
    while i < n {
        screen_write_cell(ctx, &raw mut (*sy).gc);
        i = i.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn format_draw(
    mut octx: *mut screen_write_ctx,
    mut base: *const grid_cell,
    mut available: u_int,
    mut expanded: *const ::core::ffi::c_char,
    mut srs: *mut style_ranges,
    mut default_colours: ::core::ffi::c_int,
) {
    let mut current_block: u64;
    let mut current: C2RustUnnamed_39 = LEFT;
    let mut last: C2RustUnnamed_39 = LEFT;
    let mut names: [*const ::core::ffi::c_char; 8] = [
        b"LEFT\0" as *const u8 as *const ::core::ffi::c_char,
        b"CENTRE\0" as *const u8 as *const ::core::ffi::c_char,
        b"RIGHT\0" as *const u8 as *const ::core::ffi::c_char,
        b"ABSOLUTE_CENTRE\0" as *const u8 as *const ::core::ffi::c_char,
        b"LIST\0" as *const u8 as *const ::core::ffi::c_char,
        b"LIST_LEFT\0" as *const u8 as *const ::core::ffi::c_char,
        b"LIST_RIGHT\0" as *const u8 as *const ::core::ffi::c_char,
        b"AFTER\0" as *const u8 as *const ::core::ffi::c_char,
    ];
    let mut size: size_t = strlen(expanded);
    let mut os: *mut screen = (*octx).s;
    let mut s: [screen; 8] = [screen {
        title: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        path: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        titles: ::core::ptr::null_mut::<screen_titles>(),
        ntitles: 0,
        grid: ::core::ptr::null_mut::<grid>(),
        cx: 0,
        cy: 0,
        cstyle: SCREEN_CURSOR_DEFAULT,
        default_cstyle: SCREEN_CURSOR_DEFAULT,
        ccolour: 0,
        default_ccolour: 0,
        rupper: 0,
        rlower: 0,
        mode: 0,
        default_mode: 0,
        saved_cx: 0,
        saved_cy: 0,
        saved_grid: ::core::ptr::null_mut::<grid>(),
        saved_cell: grid_cell {
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
        },
        saved_flags: 0,
        tabs: ::core::ptr::null_mut::<bitstr_t>(),
        sel: ::core::ptr::null_mut::<screen_sel>(),
        write_list: ::core::ptr::null_mut::<screen_write_cline>(),
        hyperlinks: ::core::ptr::null_mut::<hyperlinks>(),
        progress_bar: progress_bar {
            state: PROGRESS_BAR_HIDDEN,
            progress: 0,
        },
    }; 8];
    let mut hl: *mut hyperlinks = (*os).hyperlinks;
    let mut ctx: [screen_write_ctx; 8] = [screen_write_ctx {
        wp: ::core::ptr::null_mut::<window_pane>(),
        s: ::core::ptr::null_mut::<screen>(),
        flags: 0,
        init_ctx_cb: None,
        arg: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        item: ::core::ptr::null_mut::<screen_write_citem>(),
        scrolled: 0,
        bg: 0,
    }; 8];
    let mut ocx: u_int = (*os).cx;
    let mut ocy: u_int = (*os).cy;
    let mut n: u_int = 0;
    let mut i: u_int = 0;
    let mut width: [u_int; 8] = [0; 8];
    let mut map: [u_int; 5] = [
        LEFT as ::core::ffi::c_int as u_int,
        LEFT as ::core::ffi::c_int as u_int,
        CENTRE as ::core::ffi::c_int as u_int,
        RIGHT as ::core::ffi::c_int as u_int,
        ABSOLUTE_CENTRE as ::core::ffi::c_int as u_int,
    ];
    let mut focus_start: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
    let mut focus_end: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
    let mut list_state: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
    let mut fill: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
    let mut even: ::core::ffi::c_int = 0;
    let mut list_align: style_align = STYLE_ALIGN_DEFAULT;
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
    let mut current_default: grid_cell = grid_cell {
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
    let mut base_default: grid_cell = grid_cell {
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
    let mut sy: style = style {
        gc: grid_cell {
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
        },
        ignore: 0,
        dim: 0,
        fill: 0,
        align: STYLE_ALIGN_DEFAULT,
        list: STYLE_LIST_OFF,
        range_type: STYLE_RANGE_NONE,
        range_argument: 0,
        range_string: [0; 16],
        width: 0,
        width_percentage: 0,
        pad: 0,
        default_type: STYLE_DEFAULT_BASE,
        link: 0,
    };
    let mut saved_sy: style = style {
        gc: grid_cell {
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
        },
        ignore: 0,
        dim: 0,
        fill: 0,
        align: STYLE_ALIGN_DEFAULT,
        list: STYLE_LIST_OFF,
        range_type: STYLE_RANGE_NONE,
        range_argument: 0,
        range_string: [0; 16],
        width: 0,
        width_percentage: 0,
        pad: 0,
        default_type: STYLE_DEFAULT_BASE,
        link: 0,
    };
    let mut ud: *mut utf8_data = &raw mut sy.gc.data;
    let mut cp: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut end: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut link_uri: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut more: utf8_state = UTF8_MORE;
    let mut tmp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut fr: *mut format_range = ::core::ptr::null_mut::<format_range>();
    let mut fr1: *mut format_range = ::core::ptr::null_mut::<format_range>();
    let mut frs: format_ranges = format_ranges {
        tqh_first: ::core::ptr::null_mut::<format_range>(),
        tqh_last: ::core::ptr::null_mut::<*mut format_range>(),
    };
    let mut sr: *mut style_range = ::core::ptr::null_mut::<style_range>();
    memcpy(
        &raw mut base_default as *mut ::core::ffi::c_void,
        base as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    memcpy(
        &raw mut current_default as *mut ::core::ffi::c_void,
        base as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    base = &raw mut base_default;
    style_set(&raw mut sy, &raw mut current_default);
    frs.tqh_first = ::core::ptr::null_mut::<format_range>();
    frs.tqh_last = &raw mut frs.tqh_first;
    log_debug(
        b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"format_draw\0" as *const u8 as *const ::core::ffi::c_char,
        expanded,
    );
    i = 0 as u_int;
    while i < TOTAL as ::core::ffi::c_int as u_int {
        screen_init(
            (&raw mut s as *mut screen).offset(i as isize) as *mut screen,
            size as u_int,
            1 as u_int,
            0 as u_int,
        );
        screen_write_start(
            (&raw mut ctx as *mut screen_write_ctx).offset(i as isize) as *mut screen_write_ctx,
            (&raw mut s as *mut screen).offset(i as isize) as *mut screen,
        );
        screen_write_clearendofline(
            (&raw mut ctx as *mut screen_write_ctx).offset(i as isize) as *mut screen_write_ctx,
            current_default.bg as u_int,
        );
        width[i as usize] = 0 as u_int;
        i = i.wrapping_add(1);
    }
    cp = expanded;
    loop {
        if !(*cp as ::core::ffi::c_int != '\0' as i32) {
            current_block = 1830138855519935310;
            break;
        }
        if *cp.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '#' as i32
            && *cp.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '[' as i32
            && *cp.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '\0' as i32
        {
            n = 1 as u_int;
            while *cp.offset(n as isize) as ::core::ffi::c_int == '#' as i32 {
                n = n.wrapping_add(1);
            }
            even = (n.wrapping_rem(2 as u_int) == 0 as u_int) as ::core::ffi::c_int;
            if *cp.offset(n as isize) as ::core::ffi::c_int != '[' as i32 {
                cp = cp.offset(n as isize);
                if even != 0 {
                    n = n.wrapping_div(2 as u_int);
                } else {
                    n = n.wrapping_div(2 as u_int).wrapping_add(1 as u_int);
                }
                width[current as usize] = width[current as usize].wrapping_add(n);
                format_draw_many(
                    (&raw mut ctx as *mut screen_write_ctx).offset(current as isize)
                        as *mut screen_write_ctx,
                    &raw mut sy,
                    '#' as i32 as ::core::ffi::c_char,
                    n,
                );
            } else {
                if even != 0 {
                    cp = cp.offset(n.wrapping_add(1 as u_int) as isize);
                } else {
                    cp = cp.offset(n.wrapping_sub(1 as u_int) as isize);
                }
                if sy.ignore != 0 {
                    continue;
                }
                format_draw_many(
                    (&raw mut ctx as *mut screen_write_ctx).offset(current as isize)
                        as *mut screen_write_ctx,
                    &raw mut sy,
                    '#' as i32 as ::core::ffi::c_char,
                    n.wrapping_div(2 as u_int),
                );
                width[current as usize] =
                    width[current as usize].wrapping_add(n.wrapping_div(2 as u_int));
                if even != 0 {
                    utf8_set(ud, '[' as i32 as u_char);
                    screen_write_cell(
                        (&raw mut ctx as *mut screen_write_ctx).offset(current as isize)
                            as *mut screen_write_ctx,
                        &raw mut sy.gc,
                    );
                    width[current as usize] = width[current as usize].wrapping_add(1);
                }
            }
        } else if *cp.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '#' as i32
            || *cp.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '[' as i32
            || sy.ignore != 0
        {
            more = utf8_open(ud, *cp as u_char);
            if more as ::core::ffi::c_uint == UTF8_MORE as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                loop {
                    cp = cp.offset(1);
                    if !(*cp as ::core::ffi::c_int != '\0' as i32
                        && more as ::core::ffi::c_uint
                            == UTF8_MORE as ::core::ffi::c_int as ::core::ffi::c_uint)
                    {
                        break;
                    }
                    more = utf8_append(ud, *cp as u_char);
                }
                if more as ::core::ffi::c_uint
                    != UTF8_DONE as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    cp = cp.offset(-((*ud).have as ::core::ffi::c_int as isize));
                }
            }
            if more as ::core::ffi::c_uint != UTF8_DONE as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                if (*cp as ::core::ffi::c_int) < 0x20 as ::core::ffi::c_int
                    || *cp as ::core::ffi::c_int > 0x7e as ::core::ffi::c_int
                {
                    cp = cp.offset(1);
                    continue;
                } else {
                    utf8_set(ud, *cp as u_char);
                    cp = cp.offset(1);
                }
            }
            screen_write_cell(
                (&raw mut ctx as *mut screen_write_ctx).offset(current as isize)
                    as *mut screen_write_ctx,
                &raw mut sy.gc,
            );
            width[current as usize] = width[current as usize].wrapping_add((*ud).width as u_int);
        } else {
            end = format_skip(
                cp.offset(2 as ::core::ffi::c_int as isize),
                b"]\0" as *const u8 as *const ::core::ffi::c_char,
            );
            if end.is_null() {
                log_debug(
                    b"%s: no terminating ] at '%s'\0" as *const u8 as *const ::core::ffi::c_char,
                    b"format_draw\0" as *const u8 as *const ::core::ffi::c_char,
                    cp.offset(2 as ::core::ffi::c_int as isize),
                );
                fr = frs.tqh_first;
                while !fr.is_null() && {
                    fr1 = (*fr).entry.tqe_next;
                    1 as ::core::ffi::c_int != 0
                } {
                    format_free_range(&raw mut frs, fr);
                    fr = fr1;
                }
                i = 0 as u_int;
                while i < TOTAL as ::core::ffi::c_int as u_int {
                    screen_write_stop((&raw mut ctx as *mut screen_write_ctx).offset(i as isize)
                        as *mut screen_write_ctx);
                    i = i.wrapping_add(1);
                }
                current_block = 4329038292887906754;
                break;
            } else {
                tmp = xstrndup(
                    cp.offset(2 as ::core::ffi::c_int as isize),
                    end.offset_from(cp.offset(2 as ::core::ffi::c_int as isize))
                        as ::core::ffi::c_long as size_t,
                );
                style_copy(&raw mut saved_sy, &raw mut sy);
                if style_parse(&raw mut sy, &raw mut current_default, tmp)
                    != 0 as ::core::ffi::c_int
                {
                    log_debug(
                        b"%s: invalid style '%s'\0" as *const u8 as *const ::core::ffi::c_char,
                        b"format_draw\0" as *const u8 as *const ::core::ffi::c_char,
                        tmp,
                    );
                    free(tmp as *mut ::core::ffi::c_void);
                    cp = end.offset(1 as ::core::ffi::c_int as isize);
                } else {
                    log_debug(
                        b"%s: style '%s' -> '%s'\0" as *const u8 as *const ::core::ffi::c_char,
                        b"format_draw\0" as *const u8 as *const ::core::ffi::c_char,
                        tmp,
                        style_tostring(&raw mut sy),
                    );
                    free(tmp as *mut ::core::ffi::c_void);
                    if default_colours != 0 {
                        sy.gc.bg = (*base).bg;
                        sy.gc.fg = (*base).fg;
                    }
                    link_uri = style_link(&raw mut sy);
                    if !link_uri.is_null() && !hl.is_null() {
                        sy.gc.link = hyperlinks_put(hl, link_uri, link_uri);
                    } else {
                        sy.gc.link = 0 as u_int;
                    }
                    if sy.fill != 8 as ::core::ffi::c_int {
                        fill = sy.fill;
                    }
                    if sy.default_type as ::core::ffi::c_uint
                        == STYLE_DEFAULT_PUSH as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        memcpy(
                            &raw mut current_default as *mut ::core::ffi::c_void,
                            &raw mut saved_sy.gc as *const ::core::ffi::c_void,
                            ::core::mem::size_of::<grid_cell>() as size_t,
                        );
                        sy.default_type = STYLE_DEFAULT_BASE;
                    } else if sy.default_type as ::core::ffi::c_uint
                        == STYLE_DEFAULT_POP as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        memcpy(
                            &raw mut current_default as *mut ::core::ffi::c_void,
                            base as *const ::core::ffi::c_void,
                            ::core::mem::size_of::<grid_cell>() as size_t,
                        );
                        sy.default_type = STYLE_DEFAULT_BASE;
                    } else if sy.default_type as ::core::ffi::c_uint
                        == STYLE_DEFAULT_SET as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        memcpy(
                            &raw mut base_default as *mut ::core::ffi::c_void,
                            &raw mut saved_sy.gc as *const ::core::ffi::c_void,
                            ::core::mem::size_of::<grid_cell>() as size_t,
                        );
                        memcpy(
                            &raw mut current_default as *mut ::core::ffi::c_void,
                            &raw mut saved_sy.gc as *const ::core::ffi::c_void,
                            ::core::mem::size_of::<grid_cell>() as size_t,
                        );
                        sy.default_type = STYLE_DEFAULT_BASE;
                    }
                    match sy.list as ::core::ffi::c_uint {
                        1 => {
                            if list_state != 0 as ::core::ffi::c_int {
                                if !fr.is_null() {
                                    free(fr as *mut ::core::ffi::c_void);
                                    fr = ::core::ptr::null_mut::<format_range>();
                                }
                                list_state = 0 as ::core::ffi::c_int;
                                list_align = sy.align;
                            }
                            if focus_start != -(1 as ::core::ffi::c_int)
                                && focus_end == -(1 as ::core::ffi::c_int)
                            {
                                focus_end =
                                    s[LIST as ::core::ffi::c_int as usize].cx as ::core::ffi::c_int;
                            }
                            current = LIST;
                        }
                        2 => {
                            if !(list_state != 0 as ::core::ffi::c_int) {
                                if focus_start == -(1 as ::core::ffi::c_int) {
                                    focus_start = s[LIST as ::core::ffi::c_int as usize].cx
                                        as ::core::ffi::c_int;
                                }
                            }
                        }
                        0 => {
                            if list_state == 0 as ::core::ffi::c_int {
                                if !fr.is_null() {
                                    free(fr as *mut ::core::ffi::c_void);
                                    fr = ::core::ptr::null_mut::<format_range>();
                                }
                                if focus_start != -(1 as ::core::ffi::c_int)
                                    && focus_end == -(1 as ::core::ffi::c_int)
                                {
                                    focus_end = s[LIST as ::core::ffi::c_int as usize].cx
                                        as ::core::ffi::c_int;
                                }
                                map[list_align as usize] = AFTER as ::core::ffi::c_int as u_int;
                                if list_align as ::core::ffi::c_uint
                                    == STYLE_ALIGN_LEFT as ::core::ffi::c_int as ::core::ffi::c_uint
                                {
                                    map[STYLE_ALIGN_DEFAULT as ::core::ffi::c_int as usize] =
                                        AFTER as ::core::ffi::c_int as u_int;
                                }
                                list_state = 1 as ::core::ffi::c_int;
                            }
                            current = map[sy.align as usize] as C2RustUnnamed_39;
                        }
                        3 => {
                            if !(list_state != 0 as ::core::ffi::c_int) {
                                if !(s[LIST_LEFT as ::core::ffi::c_int as usize].cx != 0 as u_int) {
                                    if !fr.is_null() {
                                        free(fr as *mut ::core::ffi::c_void);
                                        fr = ::core::ptr::null_mut::<format_range>();
                                    }
                                    if focus_start != -(1 as ::core::ffi::c_int)
                                        && focus_end == -(1 as ::core::ffi::c_int)
                                    {
                                        focus_end = -(1 as ::core::ffi::c_int);
                                        focus_start = focus_end;
                                    }
                                    current = LIST_LEFT;
                                }
                            }
                        }
                        4 => {
                            if !(list_state != 0 as ::core::ffi::c_int) {
                                if !(s[LIST_RIGHT as ::core::ffi::c_int as usize].cx != 0 as u_int)
                                {
                                    if !fr.is_null() {
                                        free(fr as *mut ::core::ffi::c_void);
                                        fr = ::core::ptr::null_mut::<format_range>();
                                    }
                                    if focus_start != -(1 as ::core::ffi::c_int)
                                        && focus_end == -(1 as ::core::ffi::c_int)
                                    {
                                        focus_end = -(1 as ::core::ffi::c_int);
                                        focus_start = focus_end;
                                    }
                                    current = LIST_RIGHT;
                                }
                            }
                        }
                        _ => {}
                    }
                    if current as ::core::ffi::c_uint != last as ::core::ffi::c_uint {
                        log_debug(
                            b"%s: change %s -> %s\0" as *const u8 as *const ::core::ffi::c_char,
                            b"format_draw\0" as *const u8 as *const ::core::ffi::c_char,
                            names[last as usize],
                            names[current as usize],
                        );
                        last = current;
                    }
                    if !srs.is_null() {
                        if !fr.is_null() && format_is_type(fr, &raw mut sy) == 0 {
                            if s[current as usize].cx != (*fr).start {
                                (*fr).end = s[current as usize].cx;
                                (*fr).entry.tqe_next = ::core::ptr::null_mut::<format_range>();
                                (*fr).entry.tqe_prev = frs.tqh_last;
                                *frs.tqh_last = fr;
                                frs.tqh_last = &raw mut (*fr).entry.tqe_next;
                            } else {
                                free(fr as *mut ::core::ffi::c_void);
                            }
                            fr = ::core::ptr::null_mut::<format_range>();
                        }
                        if fr.is_null()
                            && sy.range_type as ::core::ffi::c_uint
                                != STYLE_RANGE_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            fr = xcalloc(
                                1 as size_t,
                                ::core::mem::size_of::<format_range>() as size_t,
                            ) as *mut format_range;
                            (*fr).index = current as u_int;
                            (*fr).s =
                                (&raw mut s as *mut screen).offset(current as isize) as *mut screen;
                            (*fr).start = s[current as usize].cx;
                            (*fr).type_0 = sy.range_type;
                            (*fr).argument = sy.range_argument;
                            strlcpy(
                                &raw mut (*fr).string as *mut ::core::ffi::c_char,
                                &raw mut sy.range_string as *mut ::core::ffi::c_char,
                                ::core::mem::size_of::<[::core::ffi::c_char; 16]>() as size_t,
                            );
                        }
                    }
                    cp = end.offset(1 as ::core::ffi::c_int as isize);
                }
            }
        }
    }
    match current_block {
        1830138855519935310 => {
            free(fr as *mut ::core::ffi::c_void);
            i = 0 as u_int;
            while i < TOTAL as ::core::ffi::c_int as u_int {
                screen_write_stop((&raw mut ctx as *mut screen_write_ctx).offset(i as isize)
                    as *mut screen_write_ctx);
                log_debug(
                    b"%s: width %s is %u\0" as *const u8 as *const ::core::ffi::c_char,
                    b"format_draw\0" as *const u8 as *const ::core::ffi::c_char,
                    names[i as usize],
                    width[i as usize],
                );
                i = i.wrapping_add(1);
            }
            if focus_start != -(1 as ::core::ffi::c_int) && focus_end != -(1 as ::core::ffi::c_int)
            {
                log_debug(
                    b"%s: focus %d-%d\0" as *const u8 as *const ::core::ffi::c_char,
                    b"format_draw\0" as *const u8 as *const ::core::ffi::c_char,
                    focus_start,
                    focus_end,
                );
            }
            fr = frs.tqh_first;
            while !fr.is_null() {
                log_debug(
                    b"%s: range %d|%u is %s %u-%u\0" as *const u8 as *const ::core::ffi::c_char,
                    b"format_draw\0" as *const u8 as *const ::core::ffi::c_char,
                    (*fr).type_0 as ::core::ffi::c_uint,
                    (*fr).argument,
                    names[(*fr).index as usize],
                    (*fr).start,
                    (*fr).end,
                );
                fr = (*fr).entry.tqe_next;
            }
            if fill != -(1 as ::core::ffi::c_int) {
                memcpy(
                    &raw mut gc as *mut ::core::ffi::c_void,
                    &raw const grid_default_cell as *const ::core::ffi::c_void,
                    ::core::mem::size_of::<grid_cell>() as size_t,
                );
                gc.bg = fill;
                i = 0 as u_int;
                while i < available {
                    screen_write_putc(octx, &raw mut gc, ' ' as i32 as u_char);
                    i = i.wrapping_add(1);
                }
            }
            match list_align as ::core::ffi::c_uint {
                0 => {
                    format_draw_none(
                        octx,
                        available,
                        ocx,
                        ocy,
                        (&raw mut s as *mut screen).offset(LEFT as ::core::ffi::c_int as isize)
                            as *mut screen,
                        (&raw mut s as *mut screen).offset(CENTRE as ::core::ffi::c_int as isize)
                            as *mut screen,
                        (&raw mut s as *mut screen).offset(RIGHT as ::core::ffi::c_int as isize)
                            as *mut screen,
                        (&raw mut s as *mut screen)
                            .offset(ABSOLUTE_CENTRE as ::core::ffi::c_int as isize)
                            as *mut screen,
                        &raw mut frs,
                    );
                }
                1 => {
                    format_draw_left(
                        octx,
                        available,
                        ocx,
                        ocy,
                        (&raw mut s as *mut screen).offset(LEFT as ::core::ffi::c_int as isize)
                            as *mut screen,
                        (&raw mut s as *mut screen).offset(CENTRE as ::core::ffi::c_int as isize)
                            as *mut screen,
                        (&raw mut s as *mut screen).offset(RIGHT as ::core::ffi::c_int as isize)
                            as *mut screen,
                        (&raw mut s as *mut screen)
                            .offset(ABSOLUTE_CENTRE as ::core::ffi::c_int as isize)
                            as *mut screen,
                        (&raw mut s as *mut screen).offset(LIST as ::core::ffi::c_int as isize)
                            as *mut screen,
                        (&raw mut s as *mut screen).offset(LIST_LEFT as ::core::ffi::c_int as isize)
                            as *mut screen,
                        (&raw mut s as *mut screen)
                            .offset(LIST_RIGHT as ::core::ffi::c_int as isize)
                            as *mut screen,
                        (&raw mut s as *mut screen).offset(AFTER as ::core::ffi::c_int as isize)
                            as *mut screen,
                        focus_start,
                        focus_end,
                        &raw mut frs,
                    );
                }
                2 => {
                    format_draw_centre(
                        octx,
                        available,
                        ocx,
                        ocy,
                        (&raw mut s as *mut screen).offset(LEFT as ::core::ffi::c_int as isize)
                            as *mut screen,
                        (&raw mut s as *mut screen).offset(CENTRE as ::core::ffi::c_int as isize)
                            as *mut screen,
                        (&raw mut s as *mut screen).offset(RIGHT as ::core::ffi::c_int as isize)
                            as *mut screen,
                        (&raw mut s as *mut screen)
                            .offset(ABSOLUTE_CENTRE as ::core::ffi::c_int as isize)
                            as *mut screen,
                        (&raw mut s as *mut screen).offset(LIST as ::core::ffi::c_int as isize)
                            as *mut screen,
                        (&raw mut s as *mut screen).offset(LIST_LEFT as ::core::ffi::c_int as isize)
                            as *mut screen,
                        (&raw mut s as *mut screen)
                            .offset(LIST_RIGHT as ::core::ffi::c_int as isize)
                            as *mut screen,
                        (&raw mut s as *mut screen).offset(AFTER as ::core::ffi::c_int as isize)
                            as *mut screen,
                        focus_start,
                        focus_end,
                        &raw mut frs,
                    );
                }
                3 => {
                    format_draw_right(
                        octx,
                        available,
                        ocx,
                        ocy,
                        (&raw mut s as *mut screen).offset(LEFT as ::core::ffi::c_int as isize)
                            as *mut screen,
                        (&raw mut s as *mut screen).offset(CENTRE as ::core::ffi::c_int as isize)
                            as *mut screen,
                        (&raw mut s as *mut screen).offset(RIGHT as ::core::ffi::c_int as isize)
                            as *mut screen,
                        (&raw mut s as *mut screen)
                            .offset(ABSOLUTE_CENTRE as ::core::ffi::c_int as isize)
                            as *mut screen,
                        (&raw mut s as *mut screen).offset(LIST as ::core::ffi::c_int as isize)
                            as *mut screen,
                        (&raw mut s as *mut screen).offset(LIST_LEFT as ::core::ffi::c_int as isize)
                            as *mut screen,
                        (&raw mut s as *mut screen)
                            .offset(LIST_RIGHT as ::core::ffi::c_int as isize)
                            as *mut screen,
                        (&raw mut s as *mut screen).offset(AFTER as ::core::ffi::c_int as isize)
                            as *mut screen,
                        focus_start,
                        focus_end,
                        &raw mut frs,
                    );
                }
                4 => {
                    format_draw_absolute_centre(
                        octx,
                        available,
                        ocx,
                        ocy,
                        (&raw mut s as *mut screen).offset(LEFT as ::core::ffi::c_int as isize)
                            as *mut screen,
                        (&raw mut s as *mut screen).offset(CENTRE as ::core::ffi::c_int as isize)
                            as *mut screen,
                        (&raw mut s as *mut screen).offset(RIGHT as ::core::ffi::c_int as isize)
                            as *mut screen,
                        (&raw mut s as *mut screen)
                            .offset(ABSOLUTE_CENTRE as ::core::ffi::c_int as isize)
                            as *mut screen,
                        (&raw mut s as *mut screen).offset(LIST as ::core::ffi::c_int as isize)
                            as *mut screen,
                        (&raw mut s as *mut screen).offset(LIST_LEFT as ::core::ffi::c_int as isize)
                            as *mut screen,
                        (&raw mut s as *mut screen)
                            .offset(LIST_RIGHT as ::core::ffi::c_int as isize)
                            as *mut screen,
                        (&raw mut s as *mut screen).offset(AFTER as ::core::ffi::c_int as isize)
                            as *mut screen,
                        focus_start,
                        focus_end,
                        &raw mut frs,
                    );
                }
                _ => {}
            }
            fr = frs.tqh_first;
            while !fr.is_null() && {
                fr1 = (*fr).entry.tqe_next;
                1 as ::core::ffi::c_int != 0
            } {
                sr = xcalloc(1 as size_t, ::core::mem::size_of::<style_range>() as size_t)
                    as *mut style_range;
                (*sr).type_0 = (*fr).type_0;
                (*sr).argument = (*fr).argument;
                strlcpy(
                    &raw mut (*sr).string as *mut ::core::ffi::c_char,
                    &raw mut (*fr).string as *mut ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 16]>() as size_t,
                );
                (*sr).start = (*fr).start;
                (*sr).end = (*fr).end;
                (*sr).entry.tqe_next = ::core::ptr::null_mut::<style_range>();
                (*sr).entry.tqe_prev = (*srs).tqh_last;
                *(*srs).tqh_last = sr;
                (*srs).tqh_last = &raw mut (*sr).entry.tqe_next;
                match (*sr).type_0 as ::core::ffi::c_uint {
                    1 => {
                        log_debug(
                            b"%s: range left at %u-%u\0" as *const u8 as *const ::core::ffi::c_char,
                            b"format_draw\0" as *const u8 as *const ::core::ffi::c_char,
                            (*sr).start,
                            (*sr).end,
                        );
                    }
                    2 => {
                        log_debug(
                            b"%s: range right at %u-%u\0" as *const u8
                                as *const ::core::ffi::c_char,
                            b"format_draw\0" as *const u8 as *const ::core::ffi::c_char,
                            (*sr).start,
                            (*sr).end,
                        );
                    }
                    3 => {
                        log_debug(
                            b"%s: range pane|%%%u at %u-%u\0" as *const u8
                                as *const ::core::ffi::c_char,
                            b"format_draw\0" as *const u8 as *const ::core::ffi::c_char,
                            (*sr).argument,
                            (*sr).start,
                            (*sr).end,
                        );
                    }
                    4 => {
                        log_debug(
                            b"%s: range window|%u at %u-%u\0" as *const u8
                                as *const ::core::ffi::c_char,
                            b"format_draw\0" as *const u8 as *const ::core::ffi::c_char,
                            (*sr).argument,
                            (*sr).start,
                            (*sr).end,
                        );
                    }
                    5 => {
                        log_debug(
                            b"%s: range session|$%u at %u-%u\0" as *const u8
                                as *const ::core::ffi::c_char,
                            b"format_draw\0" as *const u8 as *const ::core::ffi::c_char,
                            (*sr).argument,
                            (*sr).start,
                            (*sr).end,
                        );
                    }
                    6 => {
                        log_debug(
                            b"%s: range user|%u at %u-%u\0" as *const u8
                                as *const ::core::ffi::c_char,
                            b"format_draw\0" as *const u8 as *const ::core::ffi::c_char,
                            (*sr).argument,
                            (*sr).start,
                            (*sr).end,
                        );
                    }
                    7 => {
                        log_debug(
                            b"%s: range control|%u at %u-%u\0" as *const u8
                                as *const ::core::ffi::c_char,
                            b"format_draw\0" as *const u8 as *const ::core::ffi::c_char,
                            (*sr).argument,
                            (*sr).start,
                            (*sr).end,
                        );
                    }
                    0 | _ => {}
                }
                format_free_range(&raw mut frs, fr);
                fr = fr1;
            }
        }
        _ => {}
    }
    i = 0 as u_int;
    while i < TOTAL as ::core::ffi::c_int as u_int {
        screen_free((&raw mut s as *mut screen).offset(i as isize) as *mut screen);
        i = i.wrapping_add(1);
    }
    screen_write_cursormove(
        octx,
        ocx as ::core::ffi::c_int,
        ocy as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
}
#[no_mangle]
pub unsafe extern "C" fn format_width(mut expanded: *const ::core::ffi::c_char) -> u_int {
    let mut cp: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut end: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut n: u_int = 0;
    let mut leading_width: u_int = 0;
    let mut width: u_int = 0 as u_int;
    let mut ud: utf8_data = utf8_data {
        data: [0; 32],
        have: 0,
        size: 0,
        width: 0,
    };
    let mut more: utf8_state = UTF8_MORE;
    cp = expanded;
    while *cp as ::core::ffi::c_int != '\0' as i32 {
        if *cp as ::core::ffi::c_int == '#' as i32 {
            end = format_leading_hashes(cp, &raw mut n, &raw mut leading_width);
            width = width.wrapping_add(leading_width);
            cp = end;
            if *cp as ::core::ffi::c_int == '#' as i32 {
                end = format_skip(
                    cp.offset(2 as ::core::ffi::c_int as isize),
                    b"]\0" as *const u8 as *const ::core::ffi::c_char,
                );
                if end.is_null() {
                    return 0 as u_int;
                }
                cp = end.offset(1 as ::core::ffi::c_int as isize);
            }
        } else {
            more = utf8_open(&raw mut ud, *cp as u_char);
            if more as ::core::ffi::c_uint == UTF8_MORE as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                loop {
                    cp = cp.offset(1);
                    if !(*cp as ::core::ffi::c_int != '\0' as i32
                        && more as ::core::ffi::c_uint
                            == UTF8_MORE as ::core::ffi::c_int as ::core::ffi::c_uint)
                    {
                        break;
                    }
                    more = utf8_append(&raw mut ud, *cp as u_char);
                }
                if more as ::core::ffi::c_uint
                    == UTF8_DONE as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    width = width.wrapping_add(ud.width as u_int);
                }
            } else if *cp as ::core::ffi::c_int > 0x1f as ::core::ffi::c_int
                && (*cp as ::core::ffi::c_int) < 0x7f as ::core::ffi::c_int
            {
                width = width.wrapping_add(1);
                cp = cp.offset(1);
            } else {
                cp = cp.offset(1);
            }
        }
    }
    return width;
}
#[no_mangle]
pub unsafe extern "C" fn format_trim_left(
    mut expanded: *const ::core::ffi::c_char,
    mut limit: u_int,
) -> *mut ::core::ffi::c_char {
    let mut copy: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut out: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut cp: *const ::core::ffi::c_char = expanded;
    let mut end: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut n: u_int = 0;
    let mut width: u_int = 0 as u_int;
    let mut leading_width: u_int = 0;
    let mut ud: utf8_data = utf8_data {
        data: [0; 32],
        have: 0,
        size: 0,
        width: 0,
    };
    let mut more: utf8_state = UTF8_MORE;
    copy = xcalloc(2 as size_t, strlen(expanded).wrapping_add(1 as size_t))
        as *mut ::core::ffi::c_char;
    out = copy;
    while *cp as ::core::ffi::c_int != '\0' as i32 {
        if width >= limit {
            break;
        }
        if *cp as ::core::ffi::c_int == '#' as i32 {
            end = format_leading_hashes(cp, &raw mut n, &raw mut leading_width);
            if leading_width > limit.wrapping_sub(width) {
                leading_width = limit.wrapping_sub(width);
            }
            if leading_width != 0 as u_int {
                if n == 1 as u_int {
                    let fresh0 = out;
                    out = out.offset(1);
                    *fresh0 = '#' as i32 as ::core::ffi::c_char;
                } else {
                    memset(
                        out as *mut ::core::ffi::c_void,
                        '#' as i32,
                        (2 as u_int).wrapping_mul(leading_width) as size_t,
                    );
                    out = out.offset((2 as u_int).wrapping_mul(leading_width) as isize);
                }
                width = width.wrapping_add(leading_width);
            }
            cp = end;
            if !(*cp as ::core::ffi::c_int == '#' as i32) {
                continue;
            }
            end = format_skip(
                cp.offset(2 as ::core::ffi::c_int as isize),
                b"]\0" as *const u8 as *const ::core::ffi::c_char,
            );
            if end.is_null() {
                break;
            }
            memcpy(
                out as *mut ::core::ffi::c_void,
                cp as *const ::core::ffi::c_void,
                end.offset(1 as ::core::ffi::c_int as isize).offset_from(cp) as ::core::ffi::c_long
                    as size_t,
            );
            out = out.offset(end.offset(1 as ::core::ffi::c_int as isize).offset_from(cp)
                as ::core::ffi::c_long as isize);
            cp = end.offset(1 as ::core::ffi::c_int as isize);
        } else {
            more = utf8_open(&raw mut ud, *cp as u_char);
            if more as ::core::ffi::c_uint == UTF8_MORE as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                loop {
                    cp = cp.offset(1);
                    if !(*cp as ::core::ffi::c_int != '\0' as i32
                        && more as ::core::ffi::c_uint
                            == UTF8_MORE as ::core::ffi::c_int as ::core::ffi::c_uint)
                    {
                        break;
                    }
                    more = utf8_append(&raw mut ud, *cp as u_char);
                }
                if more as ::core::ffi::c_uint
                    == UTF8_DONE as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    if width.wrapping_add(ud.width as u_int) <= limit {
                        memcpy(
                            out as *mut ::core::ffi::c_void,
                            &raw mut ud.data as *mut u_char as *const ::core::ffi::c_void,
                            ud.size as size_t,
                        );
                        out = out.offset(ud.size as ::core::ffi::c_int as isize);
                    }
                    width = width.wrapping_add(ud.width as u_int);
                } else {
                    cp = cp.offset(-(ud.have as ::core::ffi::c_int as isize));
                    cp = cp.offset(1);
                }
            } else if *cp as ::core::ffi::c_int > 0x1f as ::core::ffi::c_int
                && (*cp as ::core::ffi::c_int) < 0x7f as ::core::ffi::c_int
            {
                if width.wrapping_add(1 as u_int) <= limit {
                    let fresh1 = out;
                    out = out.offset(1);
                    *fresh1 = *cp;
                }
                width = width.wrapping_add(1);
                cp = cp.offset(1);
            } else {
                cp = cp.offset(1);
            }
        }
    }
    *out = '\0' as i32 as ::core::ffi::c_char;
    return copy;
}
#[no_mangle]
pub unsafe extern "C" fn format_trim_right(
    mut expanded: *const ::core::ffi::c_char,
    mut limit: u_int,
) -> *mut ::core::ffi::c_char {
    let mut copy: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut out: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut cp: *const ::core::ffi::c_char = expanded;
    let mut end: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut width: u_int = 0 as u_int;
    let mut total_width: u_int = 0;
    let mut skip: u_int = 0;
    let mut n: u_int = 0;
    let mut leading_width: u_int = 0;
    let mut copy_width: u_int = 0;
    let mut ud: utf8_data = utf8_data {
        data: [0; 32],
        have: 0,
        size: 0,
        width: 0,
    };
    let mut more: utf8_state = UTF8_MORE;
    total_width = format_width(expanded);
    if total_width <= limit {
        return xstrdup(expanded);
    }
    skip = total_width.wrapping_sub(limit);
    copy = xcalloc(2 as size_t, strlen(expanded).wrapping_add(1 as size_t))
        as *mut ::core::ffi::c_char;
    out = copy;
    while *cp as ::core::ffi::c_int != '\0' as i32 {
        if *cp as ::core::ffi::c_int == '#' as i32 {
            end = format_leading_hashes(cp, &raw mut n, &raw mut leading_width);
            copy_width = leading_width;
            if width <= skip {
                if skip.wrapping_sub(width) >= copy_width {
                    copy_width = 0 as u_int;
                } else {
                    copy_width = copy_width.wrapping_sub(skip.wrapping_sub(width));
                }
            }
            if copy_width != 0 as u_int {
                if n == 1 as u_int {
                    let fresh2 = out;
                    out = out.offset(1);
                    *fresh2 = '#' as i32 as ::core::ffi::c_char;
                } else {
                    memset(
                        out as *mut ::core::ffi::c_void,
                        '#' as i32,
                        (2 as u_int).wrapping_mul(copy_width) as size_t,
                    );
                    out = out.offset((2 as u_int).wrapping_mul(copy_width) as isize);
                }
            }
            width = width.wrapping_add(leading_width);
            cp = end;
            if !(*cp as ::core::ffi::c_int == '#' as i32) {
                continue;
            }
            end = format_skip(
                cp.offset(2 as ::core::ffi::c_int as isize),
                b"]\0" as *const u8 as *const ::core::ffi::c_char,
            );
            if end.is_null() {
                break;
            }
            memcpy(
                out as *mut ::core::ffi::c_void,
                cp as *const ::core::ffi::c_void,
                end.offset(1 as ::core::ffi::c_int as isize).offset_from(cp) as ::core::ffi::c_long
                    as size_t,
            );
            out = out.offset(end.offset(1 as ::core::ffi::c_int as isize).offset_from(cp)
                as ::core::ffi::c_long as isize);
            cp = end.offset(1 as ::core::ffi::c_int as isize);
        } else {
            more = utf8_open(&raw mut ud, *cp as u_char);
            if more as ::core::ffi::c_uint == UTF8_MORE as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                loop {
                    cp = cp.offset(1);
                    if !(*cp as ::core::ffi::c_int != '\0' as i32
                        && more as ::core::ffi::c_uint
                            == UTF8_MORE as ::core::ffi::c_int as ::core::ffi::c_uint)
                    {
                        break;
                    }
                    more = utf8_append(&raw mut ud, *cp as u_char);
                }
                if more as ::core::ffi::c_uint
                    == UTF8_DONE as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    if width >= skip {
                        memcpy(
                            out as *mut ::core::ffi::c_void,
                            &raw mut ud.data as *mut u_char as *const ::core::ffi::c_void,
                            ud.size as size_t,
                        );
                        out = out.offset(ud.size as ::core::ffi::c_int as isize);
                    }
                    width = width.wrapping_add(ud.width as u_int);
                } else {
                    cp = cp.offset(-(ud.have as ::core::ffi::c_int as isize));
                    cp = cp.offset(1);
                }
            } else if *cp as ::core::ffi::c_int > 0x1f as ::core::ffi::c_int
                && (*cp as ::core::ffi::c_int) < 0x7f as ::core::ffi::c_int
            {
                if width >= skip {
                    let fresh3 = out;
                    out = out.offset(1);
                    *fresh3 = *cp;
                }
                width = width.wrapping_add(1);
                cp = cp.offset(1);
            } else {
                cp = cp.offset(1);
            }
        }
    }
    *out = '\0' as i32 as ::core::ffi::c_char;
    return copy;
}
