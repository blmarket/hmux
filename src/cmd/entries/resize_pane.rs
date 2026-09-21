use crate::src::arguments::{args_count, args_get, args_has, args_percentage_result, args_string};
use crate::src::cmd::{cmd_get_args, cmd_mouse_pane, cmd_mouse_window};
use crate::src::cmd_queue::{cmdq_error, cmdq_get_client, cmdq_get_event, cmdq_get_target};
use crate::src::compat::strtonum::strtonum;
use crate::src::events::events_fire_window;
use crate::src::ffi::libc::free;
use crate::src::grid::grid_remove_history;
use crate::src::layout::{
    layout_fix_offsets, layout_fix_panes, layout_resize_floating_pane,
    layout_resize_floating_pane_to, layout_resize_layout, layout_resize_pane,
    layout_resize_pane_to, layout_search_by_border, layout_set_size,
};
use crate::src::server_fn::{
    server_redraw_window, server_redraw_window_borders, server_unzoom_window,
};
use crate::src::shared::abi::*;
use crate::src::shared::arguments::*;
pub use crate::src::shared::arguments::{args, args_parse, args_parse_cb};
use crate::src::shared::client::*;
pub use crate::src::shared::client::{
    client, client_entry, client_file, client_file_cb, client_file_entry, client_files,
    overlay_check_cb, overlay_draw_cb, overlay_free_cb, overlay_key_cb, overlay_mode_cb,
    overlay_resize_cb,
};
use crate::src::shared::colour::*;
pub use crate::src::shared::command::CMD_AFTERHOOK;
use crate::src::shared::command::*;
pub use crate::src::shared::command::{
    cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmd_list, cmdq_item, cmdq_list, cmds,
};
pub use crate::src::shared::control::control_state;
use crate::src::shared::display::*;
pub use crate::src::shared::display::{visible_range, visible_ranges};
pub use crate::src::shared::environment::environ;
use crate::src::shared::event::*;
pub use crate::src::shared::format::{format_job_tree, format_tree};
use crate::src::shared::grid::*;
pub use crate::src::shared::hyperlinks::hyperlinks;
pub use crate::src::shared::input::{input_ctx, input_request, input_requests};
use crate::src::shared::key::*;
pub use crate::src::shared::key::{
    key_binding, key_binding_entry, key_bindings, key_event, key_table, key_table_entry,
};
pub use crate::src::shared::layout::layout_geometry;
use crate::src::shared::layout::*;
pub use crate::src::shared::layout::{layout_cell, layout_cell_entry, layout_cells};
pub use crate::src::shared::limits::{__INT_MAX__, INT_MAX, INT_MIN};
pub use crate::src::shared::menu::menu_data;
use crate::src::shared::message::*;
pub use crate::src::shared::mouse::mouse_event;
pub use crate::src::shared::options::options;
pub use crate::src::shared::pane::{
    window_pane, window_pane_entry, window_pane_modes, window_pane_prompt, window_pane_sentry,
    window_pane_tree_entry, window_pane_zentry, window_panes,
};
pub use crate::src::shared::pane::{
    window_pane_offset, window_pane_resize, window_pane_resize_entry, window_pane_resizes,
    PANE_MAXIMUM, PANE_MINIMUM, PANE_REDRAW, PANE_SCROLLBARS_LEFT, PANE_SCROLLBARS_RIGHT,
    PANE_STATUS_BOTTOM, PANE_STATUS_TOP,
};
pub use crate::src::shared::process::tmuxpeer;
pub use crate::src::shared::prompt::prompt;
pub use crate::src::shared::redraw::redraw_scene;
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::screen_write_cline;
pub use crate::src::shared::session::{session, session_entry, session_gentry};
pub use crate::src::shared::spawn::spawn_editor_state;
pub use crate::src::shared::status::status_line;
use crate::src::shared::style::*;
use crate::src::shared::terminal::*;
pub use crate::src::shared::tty::{tty, tty_code, tty_key, tty_term, tty_term_entry};
pub use crate::src::shared::window::WINDOW_ZOOMED;
pub use crate::src::shared::window::{
    window, window_alerts_entry, window_entry, window_mode, window_mode_entry,
    window_mode_entry_entry, window_winlinks, winlink, winlink_entry, winlink_sentry,
    winlink_stack, winlink_wentry, winlinks,
};
use crate::src::window::{
    window_get_pane_status, window_pane_is_floating, window_pane_scrollbar_reserve,
    window_redraw_active_switch, window_set_active_pane, window_unzoom, window_zoom,
};

pub use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_13;
pub use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_12;

#[no_mangle]
pub static mut cmd_resize_pane_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"resize-pane\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"resizep\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"D::L::MR::Tt:U::x:y:Z\0" as *const u8
                as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 1 as ::core::ffi::c_int,
            cb: None,
        },
        usage: b"[-MTZ] [-D lines] [-L columns] [-R columns] [-U lines] [-x width] [-y height] [-t target-pane]\0"
            as *const u8 as *const ::core::ffi::c_char,
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_PANE,
            flags: 0 as ::core::ffi::c_int,
        },
        flags: CMD_AFTERHOOK,
        exec: Some(
            cmd_resize_pane_exec
                as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
unsafe extern "C" fn cmd_resize_pane_exec(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
) -> cmd_retval {
    let mut args: *mut args = cmd_get_args(self_0);
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    let mut wp: *mut window_pane = (*target).wp;
    let mut wl: *mut winlink = (*target).wl;
    let mut w: *mut window = (*wl).window;
    let mut lc: *mut layout_cell = (*wp).layout_cell as *mut layout_cell;
    let mut type_0: layout_type = LAYOUT_LEFTRIGHT;
    let mut errstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut argval: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let flags: [::core::ffi::c_char; 4] = [
        'U' as i32 as ::core::ffi::c_char,
        'D' as i32 as ::core::ffi::c_char,
        'L' as i32 as ::core::ffi::c_char,
        'R' as i32 as ::core::ffi::c_char,
    ];
    let mut cause: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut flag: ::core::ffi::c_char = 0;
    let mut adjust: ::core::ffi::c_int = 0;
    let mut x: ::core::ffi::c_int = 0;
    let mut y: ::core::ffi::c_int = 0;
    let mut status: ::core::ffi::c_int = 0;
    let mut opposite: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut i: ::core::ffi::c_ulong = 0;
    let mut gd: *mut grid = (*wp).base.grid;
    if args_has(args, 'T' as i32 as u_char) != 0 {
        if !(*wp).modes.tqh_first.is_null() {
            return CMD_RETURN_NORMAL;
        }
        adjust = (*(*wp).base.grid)
            .sy
            .wrapping_sub(1 as u_int)
            .wrapping_sub((*wp).base.cy) as ::core::ffi::c_int;
        if adjust > (*gd).hsize as ::core::ffi::c_int {
            adjust = (*gd).hsize as ::core::ffi::c_int;
        }
        grid_remove_history(gd, adjust as u_int);
        (*wp).base.cy = (*wp).base.cy.wrapping_add(adjust as u_int);
        (*wp).flags |= PANE_REDRAW;
        return CMD_RETURN_NORMAL;
    }
    if args_has(args, 'M' as i32 as u_char) != 0 {
        return cmd_resize_pane_mouse_update(self_0, item);
    }
    if args_has(args, 'Z' as i32 as u_char) != 0 {
        if (*w).flags & WINDOW_ZOOMED != 0 {
            window_unzoom(w, 1 as ::core::ffi::c_int);
        } else {
            window_zoom(wp);
        }
        server_redraw_window(w);
        return CMD_RETURN_NORMAL;
    }
    server_unzoom_window(w);
    lc = (*wp).layout_cell as *mut layout_cell;
    if args_has(args, 'x' as i32 as u_char) != 0 {
        x = match args_percentage_result(
            args,
            'x' as i32 as u_char,
            0 as ::core::ffi::c_longlong,
            PANE_MAXIMUM as ::core::ffi::c_longlong,
            (*w).sx as ::core::ffi::c_longlong,
        ) {
            Ok(value) => value as ::core::ffi::c_int,
            Err(error) => {
                cmdq_error(
                    item,
                    b"width %s\0" as *const u8 as *const ::core::ffi::c_char,
                    error.message().as_ptr(),
                );
                return CMD_RETURN_ERROR;
            }
        };
        if window_pane_is_floating(wp) != 0 {
            if layout_resize_floating_pane_to(wp, LAYOUT_LEFTRIGHT, x as u_int, &raw mut cause)
                != 0 as ::core::ffi::c_int
            {
                cmdq_error(
                    item,
                    b"size %s\0" as *const u8 as *const ::core::ffi::c_char,
                    cause,
                );
                free(cause as *mut ::core::ffi::c_void);
                return CMD_RETURN_ERROR;
            }
        } else {
            layout_resize_pane_to(wp, LAYOUT_LEFTRIGHT, x as u_int);
        }
    }
    if args_has(args, 'y' as i32 as u_char) != 0 {
        y = match args_percentage_result(
            args,
            'y' as i32 as u_char,
            0 as ::core::ffi::c_longlong,
            PANE_MAXIMUM as ::core::ffi::c_longlong,
            (*w).sy as ::core::ffi::c_longlong,
        ) {
            Ok(value) => value as ::core::ffi::c_int,
            Err(error) => {
                cmdq_error(
                    item,
                    b"height %s\0" as *const u8 as *const ::core::ffi::c_char,
                    error.message().as_ptr(),
                );
                return CMD_RETURN_ERROR;
            }
        };
        status = window_get_pane_status(w);
        match status {
            PANE_STATUS_TOP => {
                if y != INT_MAX && (*wp).yoff == 1 as ::core::ffi::c_int {
                    y += 1;
                }
            }
            PANE_STATUS_BOTTOM => {
                if y != INT_MAX
                    && ((*wp).yoff as u_int).wrapping_add((*wp).sy)
                        == (*w).sy.wrapping_sub(1 as u_int)
                {
                    y += 1;
                }
            }
            _ => {}
        }
        if window_pane_is_floating(wp) != 0 {
            if layout_resize_floating_pane_to(wp, LAYOUT_TOPBOTTOM, y as u_int, &raw mut cause)
                != 0 as ::core::ffi::c_int
            {
                cmdq_error(
                    item,
                    b"size %s\0" as *const u8 as *const ::core::ffi::c_char,
                    cause,
                );
                free(cause as *mut ::core::ffi::c_void);
                return CMD_RETURN_ERROR;
            }
        } else {
            layout_resize_pane_to(wp, LAYOUT_TOPBOTTOM, y as u_int);
        }
    }
    i = 0 as ::core::ffi::c_ulong;
    while (i as usize)
        < (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
            .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>() as usize)
    {
        flag = flags[i as usize];
        if !(args_has(args, flag as u_char) == 0) {
            argval = args_get(args, flag as u_char);
            if argval.is_null() {
                if args_count(args) == 0 as u_int {
                    argval = b"1\0" as *const u8 as *const ::core::ffi::c_char;
                } else {
                    argval = args_string(args, 0 as u_int);
                }
            }
            adjust = strtonum(
                argval,
                INT_MIN as ::core::ffi::c_longlong,
                INT_MAX as ::core::ffi::c_longlong,
                &raw mut errstr,
            ) as ::core::ffi::c_int;
            if !errstr.is_null() {
                cmdq_error(
                    item,
                    b"adjustment %s\0" as *const u8 as *const ::core::ffi::c_char,
                    errstr,
                );
                return CMD_RETURN_ERROR;
            }
            type_0 = LAYOUT_TOPBOTTOM;
            if flag as ::core::ffi::c_int == 'L' as i32 || flag as ::core::ffi::c_int == 'R' as i32
            {
                type_0 = LAYOUT_LEFTRIGHT;
            }
            if window_pane_is_floating(wp) != 0 {
                if flag as ::core::ffi::c_int == 'L' as i32
                    || flag as ::core::ffi::c_int == 'U' as i32
                {
                    opposite = 1 as ::core::ffi::c_int;
                }
                if layout_resize_floating_pane(wp, type_0, adjust, opposite, &raw mut cause)
                    != 0 as ::core::ffi::c_int
                {
                    cmdq_error(
                        item,
                        b"adjustment %s\0" as *const u8 as *const ::core::ffi::c_char,
                        cause,
                    );
                    free(cause as *mut ::core::ffi::c_void);
                    return CMD_RETURN_ERROR;
                }
            } else {
                if flag as ::core::ffi::c_int == 'L' as i32
                    || flag as ::core::ffi::c_int == 'U' as i32
                {
                    adjust = -adjust;
                }
                layout_resize_pane(wp, type_0, adjust, 1 as ::core::ffi::c_int);
            }
        }
        i = i.wrapping_add(1);
    }
    if !(*lc).parent.is_null() {
        layout_fix_offsets(w);
    }
    layout_fix_panes(w, ::core::ptr::null_mut::<window_pane>());
    events_fire_window(
        b"window-layout-changed\0" as *const u8 as *const ::core::ffi::c_char,
        w,
    );
    server_redraw_window(w);
    return CMD_RETURN_NORMAL;
}
unsafe extern "C" fn cmd_resize_pane_mouse_update(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
) -> cmd_retval {
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    let mut event: *mut key_event = cmdq_get_event(item);
    let mut wp: *mut window_pane = (*target).wp;
    let mut wl: *mut winlink = (*target).wl;
    let mut w: *mut window = (*wl).window;
    let mut c: *mut client = cmdq_get_client(item);
    let mut s: *mut session = (*target).s;
    if (*event).m.valid == 0 {
        return CMD_RETURN_NORMAL;
    }
    wp = cmd_mouse_pane(
        &raw mut (*event).m,
        &raw mut s,
        ::core::ptr::null_mut::<*mut winlink>(),
    );
    if wp.is_null() || c.is_null() || (*c).session != s {
        return CMD_RETURN_NORMAL;
    }
    if window_pane_is_floating(wp) == 0 {
        (*c).tty.mouse_drag_update = Some(
            cmd_resize_pane_mouse_resize_tiled
                as unsafe extern "C" fn(*mut client, *mut mouse_event) -> (),
        )
            as Option<unsafe extern "C" fn(*mut client, *mut mouse_event) -> ()>;
        cmd_resize_pane_mouse_resize_tiled(c, &raw mut (*event).m);
        return CMD_RETURN_NORMAL;
    }
    window_redraw_active_switch(w, wp);
    window_set_active_pane(w, wp, 1 as ::core::ffi::c_int);
    (*c).tty.mouse_drag_update = Some(
        cmd_resize_pane_mouse_resize_move_floating
            as unsafe extern "C" fn(*mut client, *mut mouse_event) -> (),
    )
        as Option<unsafe extern "C" fn(*mut client, *mut mouse_event) -> ()>;
    cmd_resize_pane_mouse_resize_move_floating(c, &raw mut (*event).m);
    return CMD_RETURN_NORMAL;
}
unsafe extern "C" fn cmd_resize_pane_mouse_resize_move_floating(
    mut c: *mut client,
    mut m: *mut mouse_event,
) {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut lc: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut y: ::core::ffi::c_int = 0;
    let mut ly: ::core::ffi::c_int = 0;
    let mut x: ::core::ffi::c_int = 0;
    let mut lx: ::core::ffi::c_int = 0;
    let mut sx: ::core::ffi::c_int = 0;
    let mut sy: ::core::ffi::c_int = 0;
    let mut new_sx: ::core::ffi::c_int = 0;
    let mut new_sy: ::core::ffi::c_int = 0;
    let mut left: ::core::ffi::c_int = 0;
    let mut right: ::core::ffi::c_int = 0;
    let mut new_xoff: ::core::ffi::c_int = 0;
    let mut new_yoff: ::core::ffi::c_int = 0;
    let mut resizes: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    wp = cmd_mouse_pane(m, ::core::ptr::null_mut::<*mut session>(), &raw mut wl);
    if wp.is_null() {
        (*c).tty.mouse_drag_update = None;
        return;
    }
    w = (*wl).window;
    lc = (*wp).layout_cell as *mut layout_cell;
    sx = (*wp).sx as ::core::ffi::c_int;
    sy = (*wp).sy as ::core::ffi::c_int;
    left = (*wp).xoff - 1 as ::core::ffi::c_int;
    right = (*wp).xoff + sx;
    if window_pane_scrollbar_reserve(wp) != 0 && (*w).sb_pos == PANE_SCROLLBARS_LEFT {
        left -= (*wp).scrollbar_style.width + (*wp).scrollbar_style.pad;
    } else if window_pane_scrollbar_reserve(wp) != 0 && (*w).sb_pos == PANE_SCROLLBARS_RIGHT {
        right += (*wp).scrollbar_style.width + (*wp).scrollbar_style.pad;
    }
    y = (*m).y.wrapping_add((*m).oy) as ::core::ffi::c_int;
    x = (*m).x.wrapping_add((*m).ox) as ::core::ffi::c_int;
    if (*m).statusat == 0 as ::core::ffi::c_int && y >= (*m).statuslines as ::core::ffi::c_int {
        y = (y as u_int).wrapping_sub((*m).statuslines) as ::core::ffi::c_int as ::core::ffi::c_int;
    } else if (*m).statusat > 0 as ::core::ffi::c_int && y >= (*m).statusat {
        y = (*m).statusat - 1 as ::core::ffi::c_int;
    }
    ly = (*m).ly.wrapping_add((*m).oy) as ::core::ffi::c_int;
    lx = (*m).lx.wrapping_add((*m).ox) as ::core::ffi::c_int;
    if (*m).statusat == 0 as ::core::ffi::c_int && ly >= (*m).statuslines as ::core::ffi::c_int {
        ly = (ly as u_int).wrapping_sub((*m).statuslines) as ::core::ffi::c_int
            as ::core::ffi::c_int;
    } else if (*m).statusat > 0 as ::core::ffi::c_int && ly >= (*m).statusat {
        ly = (*m).statusat - 1 as ::core::ffi::c_int;
    }
    if (lx == left || lx == left + 1 as ::core::ffi::c_int)
        && ly == (*wp).yoff - 1 as ::core::ffi::c_int
    {
        new_sx = (*lc).g.sx.wrapping_add((lx - x) as u_int) as ::core::ffi::c_int;
        if new_sx < PANE_MINIMUM {
            new_sx = PANE_MINIMUM;
        }
        new_sy = (*lc).g.sy.wrapping_add((ly - y) as u_int) as ::core::ffi::c_int;
        if new_sy < PANE_MINIMUM {
            new_sy = PANE_MINIMUM;
        }
        new_xoff = x + 1 as ::core::ffi::c_int;
        new_yoff = y + 1 as ::core::ffi::c_int;
        layout_set_size(lc, new_sx as u_int, new_sy as u_int, new_xoff, new_yoff);
        resizes += 1;
    } else if (lx == right + 1 as ::core::ffi::c_int || lx == right)
        && ly == (*wp).yoff - 1 as ::core::ffi::c_int
    {
        new_sx = x - (*lc).g.xoff;
        if new_sx < PANE_MINIMUM {
            new_sx = PANE_MINIMUM;
        }
        new_sy = (*lc).g.sy.wrapping_add((ly - y) as u_int) as ::core::ffi::c_int;
        if new_sy < PANE_MINIMUM {
            new_sy = PANE_MINIMUM;
        }
        new_yoff = y + 1 as ::core::ffi::c_int;
        layout_set_size(lc, new_sx as u_int, new_sy as u_int, (*lc).g.xoff, new_yoff);
        resizes += 1;
    } else if (lx == left || lx == left + 1 as ::core::ffi::c_int) && ly == (*wp).yoff + sy {
        new_sx = (*lc).g.sx.wrapping_add((lx - x) as u_int) as ::core::ffi::c_int;
        if new_sx < PANE_MINIMUM {
            new_sx = PANE_MINIMUM;
        }
        new_sy = y - (*lc).g.yoff;
        if new_sy < PANE_MINIMUM {
            return;
        }
        new_xoff = x + 1 as ::core::ffi::c_int;
        layout_set_size(lc, new_sx as u_int, new_sy as u_int, new_xoff, (*lc).g.yoff);
        resizes += 1;
    } else if (lx == right + 1 as ::core::ffi::c_int || lx == right) && ly == (*wp).yoff + sy {
        new_sx = x - (*lc).g.xoff;
        if new_sx < PANE_MINIMUM {
            new_sx = PANE_MINIMUM;
        }
        new_sy = y - (*lc).g.yoff;
        if new_sy < PANE_MINIMUM {
            new_sy = PANE_MINIMUM;
        }
        layout_set_size(
            lc,
            new_sx as u_int,
            new_sy as u_int,
            (*lc).g.xoff,
            (*lc).g.yoff,
        );
        resizes += 1;
    } else if lx == right {
        new_sx = x - (*lc).g.xoff;
        if new_sx < PANE_MINIMUM {
            return;
        }
        layout_set_size(lc, new_sx as u_int, (*lc).g.sy, (*lc).g.xoff, (*lc).g.yoff);
        resizes += 1;
    } else if lx == left {
        new_sx = (*lc).g.sx.wrapping_add((lx - x) as u_int) as ::core::ffi::c_int;
        if new_sx < PANE_MINIMUM {
            return;
        }
        new_xoff = x + 1 as ::core::ffi::c_int;
        layout_set_size(lc, new_sx as u_int, (*lc).g.sy, new_xoff, (*lc).g.yoff);
        resizes += 1;
    } else if ly == (*wp).yoff + sy {
        new_sy = y - (*lc).g.yoff;
        if new_sy < PANE_MINIMUM {
            return;
        }
        layout_set_size(lc, (*lc).g.sx, new_sy as u_int, (*lc).g.xoff, (*lc).g.yoff);
        resizes += 1;
    } else if ly == (*wp).yoff - 1 as ::core::ffi::c_int {
        new_xoff = (*lc).g.xoff + (x - lx);
        new_yoff = y + 1 as ::core::ffi::c_int;
        layout_set_size(lc, (*lc).g.sx, (*lc).g.sy, new_xoff, new_yoff);
        resizes += 1;
    }
    if resizes != 0 as ::core::ffi::c_int {
        layout_fix_panes(w, ::core::ptr::null_mut::<window_pane>());
        server_redraw_window(w);
        server_redraw_window_borders(w);
    }
}
unsafe extern "C" fn cmd_resize_pane_mouse_resize_tiled(
    mut c: *mut client,
    mut m: *mut mouse_event,
) {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut y: u_int = 0;
    let mut ly: u_int = 0;
    let mut x: u_int = 0;
    let mut lx: u_int = 0;
    static mut offsets: [[::core::ffi::c_int; 2]; 5] = [
        [0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int],
        [0 as ::core::ffi::c_int, 1 as ::core::ffi::c_int],
        [1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int],
        [0 as ::core::ffi::c_int, -(1 as ::core::ffi::c_int)],
        [-(1 as ::core::ffi::c_int), 0 as ::core::ffi::c_int],
    ];
    let mut cells: [*mut layout_cell; 5] = [::core::ptr::null_mut::<layout_cell>(); 5];
    let mut lc: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut ncells: u_int = 0 as u_int;
    let mut i: u_int = 0;
    let mut j: u_int = 0;
    let mut resizes: u_int = 0 as u_int;
    let mut type_0: layout_type = LAYOUT_LEFTRIGHT;
    wl = cmd_mouse_window(m, ::core::ptr::null_mut::<*mut session>());
    if wl.is_null() {
        (*c).tty.mouse_drag_update = None;
        return;
    }
    w = (*wl).window;
    y = (*m).y.wrapping_add((*m).oy);
    x = (*m).x.wrapping_add((*m).ox);
    if (*m).statusat == 0 as ::core::ffi::c_int && y >= (*m).statuslines {
        y = y.wrapping_sub((*m).statuslines);
    } else if (*m).statusat > 0 as ::core::ffi::c_int && y >= (*m).statusat as u_int {
        y = ((*m).statusat - 1 as ::core::ffi::c_int) as u_int;
    }
    ly = (*m).ly.wrapping_add((*m).oy);
    lx = (*m).lx.wrapping_add((*m).ox);
    if (*m).statusat == 0 as ::core::ffi::c_int && ly >= (*m).statuslines {
        ly = ly.wrapping_sub((*m).statuslines);
    } else if (*m).statusat > 0 as ::core::ffi::c_int && ly >= (*m).statusat as u_int {
        ly = ((*m).statusat - 1 as ::core::ffi::c_int) as u_int;
    }
    i = 0 as u_int;
    while (i as usize)
        < (::core::mem::size_of::<[*mut layout_cell; 5]>() as usize)
            .wrapping_div(::core::mem::size_of::<*mut layout_cell>() as usize)
    {
        lc = layout_search_by_border(
            (*w).layout_root,
            lx.wrapping_add(offsets[i as usize][0 as ::core::ffi::c_int as usize] as u_int),
            ly.wrapping_add(offsets[i as usize][1 as ::core::ffi::c_int as usize] as u_int),
        );
        if !lc.is_null() {
            j = 0 as u_int;
            while j < ncells {
                if cells[j as usize] == lc {
                    lc = ::core::ptr::null_mut::<layout_cell>();
                    break;
                } else {
                    j = j.wrapping_add(1);
                }
            }
            if !lc.is_null() {
                cells[ncells as usize] = lc;
                ncells = ncells.wrapping_add(1);
            }
        }
        i = i.wrapping_add(1);
    }
    if ncells == 0 as u_int {
        return;
    }
    i = 0 as u_int;
    while i < ncells {
        type_0 = (*(*cells[i as usize]).parent).type_0;
        if y != ly
            && type_0 as ::core::ffi::c_uint
                == LAYOUT_TOPBOTTOM as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            layout_resize_layout(
                w,
                cells[i as usize],
                type_0,
                y.wrapping_sub(ly) as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
            resizes = resizes.wrapping_add(1);
        } else if x != lx
            && type_0 as ::core::ffi::c_uint
                == LAYOUT_LEFTRIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            layout_resize_layout(
                w,
                cells[i as usize],
                type_0,
                x.wrapping_sub(lx) as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
            resizes = resizes.wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }
    if resizes != 0 as u_int {
        server_redraw_window(w);
    }
}
