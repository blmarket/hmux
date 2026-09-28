use crate::src::options::options_owner_ptr;
use crate::src::arguments::{args_get, args_has, args_percentage_and_expand_result};
use crate::src::cmd::find::cmd_find_from_session;
use crate::src::cmd::queue::{
    cmdq_error, cmdq_get_client, cmdq_get_state_owned, cmdq_get_event, cmdq_get_source, cmdq_get_target,
};
use crate::src::cmd::{cmd_get_args_mut, cmd_get_entry, cmd_mouse_pane};
use crate::src::compat::strtonum::strtonum;
use crate::src::events::events_fire_window;
use crate::src::ffi::libc::strcmp;
use crate::src::format::bytes::write_cstr;
use crate::src::layout::{
    layout_assign_pane, layout_close_pane, layout_fix_offsets, layout_fix_panes,
    layout_get_tiled_cell, layout_insert_tile,
};
use crate::src::options::options_set_parent;
use crate::src::resize::recalculate_sizes;
use crate::src::screen_redraw::redraw_invalidate_scene;
use crate::src::server_client::server_client_remove_pane;
use crate::src::server_fn::{
    server_kill_window, server_redraw_session, server_redraw_window, server_redraw_window_borders,
    server_status_session, server_unzoom_window,
};
use crate::src::session::session_select;
use crate::src::shared::abi::*;
use crate::src::shared::arguments::{args, args_parse};
use crate::src::shared::client::client;
use crate::src::shared::command::CMD_FIND_DEFAULT_MARKED;
use crate::src::shared::command::*;
use crate::src::shared::command::{cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmdq_item};
use crate::src::shared::key::key_event;
use crate::src::shared::layout::layout_cell;
use crate::src::shared::layout::*;
use crate::src::shared::limits::{INT_MAX, INT_MIN, UINT_MAX};
use crate::src::shared::mouse::mouse_event;
use crate::src::shared::pane::window_pane;
use crate::src::shared::pane::{PANE_STYLECHANGED, PANE_THEMECHANGED};
use crate::src::shared::session::session;
use crate::src::shared::spawn::{SPAWN_BEFORE, SPAWN_FULLSIZE, SPAWN_HORIZONTAL};
use crate::src::shared::window::WINDOW_ZOOMED;
use crate::src::shared::window::{window, winlink};
use crate::src::style::colour::colour_palette_from_option;
use crate::src::window::{
    window_count_panes, window_fire_pane_moved, window_lost_pane, window_pane_get_pane_lines,
    window_pane_is_floating, window_pane_list_insert_after, window_pane_list_insert_before,
    window_pane_list_remove, window_pane_z_first, window_pane_z_insert_after,
    window_pane_z_insert_back, window_pane_z_insert_before, window_pane_z_insert_front,
    window_pane_z_next, window_pane_z_previous, window_pane_z_remove, window_redraw_active_switch,
    window_set_active_pane,
};
pub static cmd_join_pane_entry: cmd_entry = {
    cmd_entry {
        name: c"join-pane",
        alias: Some(c"joinp"),
        args: args_parse {
            template: c"bdfhvp:l:s:t:",
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: c"[-bdfhv] [-l size] [-s src-pane] [-t dst-pane]",
        source: cmd_entry_flag {
            flag: 's' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_PANE,
            flags: CMD_FIND_DEFAULT_MARKED,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_PANE,
            flags: 0 as ::core::ffi::c_int,
        },
        flags: 0 as ::core::ffi::c_int,
        exec: Some(cmd_join_pane_exec as unsafe fn(*mut cmd, *mut cmdq_item) -> cmd_retval),
    }
};
pub static cmd_move_pane_entry: cmd_entry = {
    cmd_entry {
        name: c"move-pane",
        alias: Some(c"movep"),
        args: args_parse {
            template: c"bdD::fhMvl:L::P:R::s:t:U::X:Y:z:",
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: c"[-bdfhMv] [-D lines] [-l size] [-L columns] [-P position] [-R columns] [-s src-pane] [-t dst-pane] [-U lines] [-X x-position] [-Y y-position] [-z z-index]",
        source: cmd_entry_flag {
            flag: 's' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_PANE,
            flags: CMD_FIND_DEFAULT_MARKED,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_PANE,
            flags: 0 as ::core::ffi::c_int,
        },
        flags: 0 as ::core::ffi::c_int,
        exec: Some(
            cmd_join_pane_exec
                as unsafe fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
unsafe fn cmd_join_pane_place(
    mut item: *mut cmdq_item,
    mut wl: *mut winlink,
    mut wp: *mut window_pane,
    mut position: *const ::core::ffi::c_char,
) -> cmd_retval {
    let mut w: *mut window = (*wl).window_ptr();
    let mut lc: *mut layout_cell = (*wp).layout_cell as *mut layout_cell;
    let mut owp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut wx: ::core::ffi::c_int = (*w).sx as ::core::ffi::c_int;
    let mut wy: ::core::ffi::c_int = (*w).sy as ::core::ffi::c_int;
    let mut px: ::core::ffi::c_int = (*lc).g.sx as ::core::ffi::c_int;
    let mut py: ::core::ffi::c_int = (*lc).g.sy as ::core::ffi::c_int;
    let mut xoff: ::core::ffi::c_int = (*lc).g.xoff;
    let mut yoff: ::core::ffi::c_int = (*lc).g.yoff;
    let mut border: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    if window_pane_get_pane_lines(&*wp) as ::core::ffi::c_uint
        == PANE_LINES_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        border = 0 as ::core::ffi::c_int;
    }
    if strcmp(
        position,
        b"top-left\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        xoff = border;
        yoff = border;
    } else if strcmp(
        position,
        b"top-centre\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
        || strcmp(
            position,
            b"top-center\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
    {
        xoff = (wx - px) / 2 as ::core::ffi::c_int;
        yoff = border;
    } else if strcmp(
        position,
        b"top-right\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        xoff = wx - px - border;
        yoff = border;
    } else if strcmp(
        position,
        b"centre-left\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
        || strcmp(
            position,
            b"center-left\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
    {
        xoff = border;
        yoff = (wy - py) / 2 as ::core::ffi::c_int;
    } else if strcmp(
        position,
        b"centre\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
        || strcmp(
            position,
            b"center\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
    {
        xoff = (wx - px) / 2 as ::core::ffi::c_int;
        yoff = (wy - py) / 2 as ::core::ffi::c_int;
    } else if strcmp(
        position,
        b"centre-right\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
        || strcmp(
            position,
            b"center-right\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
    {
        xoff = wx - px - border;
        yoff = (wy - py) / 2 as ::core::ffi::c_int;
    } else if strcmp(
        position,
        b"bottom-left\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        xoff = border;
        yoff = wy - py - border;
    } else if strcmp(
        position,
        b"bottom-centre\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
        || strcmp(
            position,
            b"bottom-center\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
    {
        xoff = (wx - px) / 2 as ::core::ffi::c_int;
        yoff = wy - py - border;
    } else if strcmp(
        position,
        b"bottom-right\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        xoff = wx - px - border;
        yoff = wy - py - border;
    } else if strcmp(
        position,
        b"top-left-centre\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
        || strcmp(
            position,
            b"top-left-center\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
    {
        xoff = wx / 4 as ::core::ffi::c_int - px / 2 as ::core::ffi::c_int;
        yoff = wy / 4 as ::core::ffi::c_int - py / 2 as ::core::ffi::c_int;
    } else if strcmp(
        position,
        b"top-right-centre\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
        || strcmp(
            position,
            b"top-right-center\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
    {
        xoff =
            3 as ::core::ffi::c_int * wx / 4 as ::core::ffi::c_int - px / 2 as ::core::ffi::c_int;
        yoff = wy / 4 as ::core::ffi::c_int - py / 2 as ::core::ffi::c_int;
    } else if strcmp(
        position,
        b"bottom-left-centre\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
        || strcmp(
            position,
            b"bottom-left-center\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
    {
        xoff = wx / 4 as ::core::ffi::c_int - px / 2 as ::core::ffi::c_int;
        yoff =
            3 as ::core::ffi::c_int * wy / 4 as ::core::ffi::c_int - py / 2 as ::core::ffi::c_int;
    } else if strcmp(
        position,
        b"bottom-right-centre\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
        || strcmp(
            position,
            b"bottom-right-center\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
    {
        xoff =
            3 as ::core::ffi::c_int * wx / 4 as ::core::ffi::c_int - px / 2 as ::core::ffi::c_int;
        yoff =
            3 as ::core::ffi::c_int * wy / 4 as ::core::ffi::c_int - py / 2 as ::core::ffi::c_int;
    } else if strcmp(
        position,
        b"front\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        window_pane_z_remove(&mut *w, &*wp);
        window_pane_z_insert_front(&mut *w, &*wp);
    } else if strcmp(
        position,
        b"back\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        window_pane_z_remove(&mut *w, &*wp);
        owp = window_pane_z_first(w.as_ref()).as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
        while !owp.is_null() {
            if window_pane_is_floating(&*owp) == 0 {
                break;
            }
            owp = window_pane_z_next(owp.as_ref()).as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
        }
        if !owp.is_null() {
            window_pane_z_insert_before(&mut *w, &*owp, &*wp);
        } else {
            window_pane_z_insert_back(&mut *w, &*wp);
        }
    } else if strcmp(
        position,
        b"forward\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        owp = window_pane_z_previous(wp.as_ref()).as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
        if !owp.is_null() {
            window_pane_z_remove(&mut *w, &*wp);
            window_pane_z_insert_before(&mut *w, &*owp, &*wp);
        }
    } else if strcmp(
        position,
        b"backward\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        owp = window_pane_z_next(wp.as_ref()).as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
        if !owp.is_null() && window_pane_is_floating(&*owp) != 0 {
            window_pane_z_remove(&mut *w, &*wp);
            window_pane_z_insert_after(&mut *w, &*owp, &*wp);
        }
    } else if strcmp(
        position,
        b"forward-loop\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        owp = window_pane_z_previous(wp.as_ref()).as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
        window_pane_z_remove(&mut *w, &*wp);
        if !owp.is_null() {
            window_pane_z_insert_before(&mut *w, &*owp, &*wp);
        } else {
            owp = window_pane_z_first(w.as_ref()).as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
            while !owp.is_null() {
                if window_pane_is_floating(&*owp) == 0 {
                    break;
                }
                owp = window_pane_z_next(owp.as_ref()).as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
            }
            if !owp.is_null() {
                window_pane_z_insert_before(&mut *w, &*owp, &*wp);
            } else {
                window_pane_z_insert_back(&mut *w, &*wp);
            }
        }
    } else if strcmp(
        position,
        b"backward-loop\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        owp = window_pane_z_next(wp.as_ref()).as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
        if !owp.is_null() && window_pane_is_floating(&*owp) != 0 {
            window_pane_z_remove(&mut *w, &*wp);
            window_pane_z_insert_after(&mut *w, &*owp, &*wp);
        } else {
            window_pane_z_remove(&mut *w, &*wp);
            window_pane_z_insert_front(&mut *w, &*wp);
        }
    } else {
        cmdq_error(item, |out| {
            out.write_all(b"unknown position: ")?;
            write_cstr(out, position)
        });
        return CMD_RETURN_ERROR;
    }
    if xoff != (*lc).g.xoff || yoff != (*lc).g.yoff {
        (*lc).g.xoff = xoff;
        (*lc).g.yoff = yoff;
        layout_fix_panes(w, ::core::ptr::null_mut::<window_pane>());
    }
    redraw_invalidate_scene(w);
    events_fire_window(
        b"window-layout-changed\0" as *const u8 as *const ::core::ffi::c_char,
        w,
    );
    server_redraw_window(&*(w));
    return CMD_RETURN_NORMAL;
}
unsafe fn cmd_join_pane_move(
    mut item: *mut cmdq_item,
    mut args: *mut args,
    mut wl: *mut winlink,
    mut wp: *mut window_pane,
) -> cmd_retval {
    let mut w: *mut window = (*wl).window_ptr();
    let mut lc: *mut layout_cell = (*wp).layout_cell as *mut layout_cell;
    let mut errstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut argval: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let flags: [::core::ffi::c_char; 4] = [
        'U' as i32 as ::core::ffi::c_char,
        'D' as i32 as ::core::ffi::c_char,
        'L' as i32 as ::core::ffi::c_char,
        'R' as i32 as ::core::ffi::c_char,
    ];
    let mut flag: ::core::ffi::c_char = 0;
    let mut xoff: ::core::ffi::c_int = (*lc).g.xoff;
    let mut yoff: ::core::ffi::c_int = (*lc).g.yoff;
    let mut adjust: ::core::ffi::c_int = 0;
    let mut i: u_int = 0;
    let mut lines: pane_lines = window_pane_get_pane_lines(&*wp);
    if args_has(args, 'X' as i32 as u_char) != 0 {
        xoff = match args_percentage_and_expand_result(
            args,
            'X' as i32 as u_char,
            -((*w).sx as ::core::ffi::c_int) as ::core::ffi::c_longlong,
            (*w).sx as ::core::ffi::c_longlong,
            (*w).sx as ::core::ffi::c_longlong,
            item,
        ) {
            Ok(value) => value as ::core::ffi::c_int,
            Err(error) => {
                cmdq_error(item, |out| {
                    out.write_all(b"position ")?;
                    write_cstr(out, error.message().as_ptr())
                });
                return CMD_RETURN_ERROR;
            }
        };
        if lines as ::core::ffi::c_uint
            != PANE_LINES_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            xoff += 1 as ::core::ffi::c_int;
        }
    }
    if args_has(args, 'Y' as i32 as u_char) != 0 {
        yoff = match args_percentage_and_expand_result(
            args,
            'Y' as i32 as u_char,
            -((*w).sy as ::core::ffi::c_int) as ::core::ffi::c_longlong,
            (*w).sy as ::core::ffi::c_longlong,
            (*w).sy as ::core::ffi::c_longlong,
            item,
        ) {
            Ok(value) => value as ::core::ffi::c_int,
            Err(error) => {
                cmdq_error(item, |out| {
                    out.write_all(b"position ")?;
                    write_cstr(out, error.message().as_ptr())
                });
                return CMD_RETURN_ERROR;
            }
        };
        if lines as ::core::ffi::c_uint
            != PANE_LINES_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            yoff += 1 as ::core::ffi::c_int;
        }
    }
    i = 0 as u_int;
    while (i as usize)
        < (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
            .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>() as usize)
    {
        flag = flags[i as usize];
        if !(args_has(args, flag as u_char) == 0) {
            argval = args_get(&*(args), flag as u_char).map_or(std::ptr::null(), |value| value.as_ptr());
            if argval.is_null() {
                argval = b"1\0" as *const u8 as *const ::core::ffi::c_char;
            }
            adjust = strtonum(
                argval,
                INT_MIN as ::core::ffi::c_longlong,
                INT_MAX as ::core::ffi::c_longlong,
                &raw mut errstr,
            ) as ::core::ffi::c_int;
            if !errstr.is_null() {
                cmdq_error(item, |out| {
                    out.write_all(b"offset ")?;
                    write_cstr(out, errstr)
                });
                return CMD_RETURN_ERROR;
            }
            if flag as ::core::ffi::c_int == 'U' as i32 {
                yoff -= adjust;
            } else if flag as ::core::ffi::c_int == 'D' as i32 {
                yoff += adjust;
            } else if flag as ::core::ffi::c_int == 'L' as i32 {
                xoff -= adjust;
            } else {
                xoff += adjust;
            }
        }
        i = i.wrapping_add(1);
    }
    if xoff != (*lc).g.xoff || yoff != (*lc).g.yoff {
        (*lc).g.xoff = xoff;
        (*lc).g.yoff = yoff;
        layout_fix_panes(w, ::core::ptr::null_mut::<window_pane>());
        events_fire_window(
            b"window-layout-changed\0" as *const u8 as *const ::core::ffi::c_char,
            w,
        );
        server_redraw_window(&*(w));
    }
    return CMD_RETURN_NORMAL;
}
unsafe fn cmd_join_pane_mouse_update(mut item: *mut cmdq_item) -> cmd_retval {
    let mouse_pane_owner;
    let mut target: *mut cmd_find_state = crate::src::cmd::queue::cmdq_get_target_mut(&mut *item);
    let mut event_snapshot = cmdq_get_event(item);
    let event: *mut key_event = &mut event_snapshot;
    let c_owner = cmdq_get_client(item);
    let mut c: *mut client = c_owner.as_ref().map_or(std::ptr::null_mut(), crate::src::shared::rc::as_ptr);
    let mut s: *mut session = (*target).s_ptr();
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    if (*event).m.valid == 0 {
        return CMD_RETURN_NORMAL;
    }
    mouse_pane_owner = cmd_mouse_pane(&raw mut (*event).m, &raw mut s, &raw mut wl);
    wp = mouse_pane_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    if wp.is_null() || c.is_null() || (*c).session_ptr() != s {
        return CMD_RETURN_NORMAL;
    }
    if window_pane_is_floating(&*wp) == 0 {
        return CMD_RETURN_NORMAL;
    }
    w = (*wl).window_ptr();
    window_redraw_active_switch(w, wp);
    window_set_active_pane(w, wp, 1 as ::core::ffi::c_int);
    (*c).tty.mouse_drag_update = Some(Box::new(crate::src::tty::tty_mouse_client_callback(
        c_owner.as_ref().expect("live drag client"),
        cmd_join_pane_mouse_move,
    )));
    cmd_join_pane_mouse_move(c_owner.as_ref().expect("live drag client"), &raw mut (*event).m);
    return CMD_RETURN_NORMAL;
}
unsafe fn cmd_join_pane_mouse_move(client_owner: &std::rc::Rc<std::cell::UnsafeCell<client>>, mut m: *mut mouse_event) {
    let c = client_owner.get();
    let mouse_pane_owner;
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut lc: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut y: ::core::ffi::c_int = 0;
    let mut ly: ::core::ffi::c_int = 0;
    let mut x: ::core::ffi::c_int = 0;
    let mut lx: ::core::ffi::c_int = 0;
    mouse_pane_owner = cmd_mouse_pane(m, ::core::ptr::null_mut::<*mut session>(), &raw mut wl);
    wp = mouse_pane_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    if wp.is_null() {
        (*c).tty.mouse_drag_update = None;
        return;
    }
    w = (*wl).window_ptr();
    lc = (*wp).layout_cell as *mut layout_cell;
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
    if x != lx || y != ly {
        (*lc).g.xoff += x - lx;
        (*lc).g.yoff += y - ly;
        layout_fix_panes(w, ::core::ptr::null_mut::<window_pane>());
        server_redraw_window(&*(w));
        server_redraw_window_borders(&*(w));
    }
}
unsafe fn cmd_join_pane_zindex(
    mut item: *mut cmdq_item,
    mut wl: *mut winlink,
    mut wp: *mut window_pane,
    mut s: *const ::core::ffi::c_char,
) -> cmd_retval {
    let mut w: *mut window = (*wl).window_ptr();
    let mut owp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut errstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut n: u_int = 0;
    let mut z: u_int = 0;
    z = strtonum(
        s,
        0 as ::core::ffi::c_longlong,
        UINT_MAX as ::core::ffi::c_longlong,
        &raw mut errstr,
    ) as u_int;
    if !errstr.is_null() {
        cmdq_error(item, |out| {
            out.write_all(b"z-index ")?;
            write_cstr(out, errstr)
        });
        return CMD_RETURN_ERROR;
    }
    window_pane_z_remove(&mut *w, &*wp);
    n = 0 as u_int;
    owp = window_pane_z_first(w.as_ref()).as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    while !owp.is_null() {
        if window_pane_is_floating(&*owp) == 0 {
            break;
        }
        if n >= z {
            break;
        }
        n = n.wrapping_add(1);
        owp = window_pane_z_next(owp.as_ref()).as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    }
    if !owp.is_null() {
        window_pane_z_insert_before(&mut *w, &*owp, &*wp);
    } else {
        window_pane_z_insert_back(&mut *w, &*wp);
    }
    redraw_invalidate_scene(w);
    events_fire_window(
        b"window-layout-changed\0" as *const u8 as *const ::core::ffi::c_char,
        w,
    );
    server_redraw_window(&*(w));
    return CMD_RETURN_NORMAL;
}
unsafe fn cmd_join_pane_tile(
    mut item: *mut cmdq_item,
    mut args: *mut args,
    mut w: *mut window,
    mut wp: *mut window_pane,
) -> cmd_retval {
    let mut lc: *mut layout_cell = (*wp).layout_cell as *mut layout_cell;
    if window_pane_is_floating(&*wp) == 0 {
        cmdq_error(item, |out| out.write_all(b"pane is not floating"));
        return CMD_RETURN_ERROR;
    }
    if (*w).flags & WINDOW_ZOOMED != 0 {
        cmdq_error(item, |out| {
            out.write_all(b"can't tile a pane while window is zoomed")
        });
        return CMD_RETURN_ERROR;
    }
    (*lc).fg.sx = (*lc).g.sx;
    (*lc).fg.sy = (*lc).g.sy;
    (*lc).fg.xoff = (*lc).g.xoff;
    (*lc).fg.yoff = (*lc).g.yoff;
    if layout_insert_tile(w, lc) != 0 as ::core::ffi::c_int {
        cmdq_error(item, |out| out.write_all(b"no space for a new pane"));
        return CMD_RETURN_ERROR;
    }
    (*lc).flags &= !LAYOUT_CELL_FLOATING;
    window_pane_z_remove(&mut *w, &*wp);
    window_pane_z_insert_back(&mut *w, &*wp);
    if args_has(args, 'd' as i32 as u_char) == 0 {
        window_set_active_pane(w, wp, 1 as ::core::ffi::c_int);
    }
    layout_fix_offsets(w);
    layout_fix_panes(w, ::core::ptr::null_mut::<window_pane>());
    redraw_invalidate_scene(w);
    events_fire_window(
        b"window-layout-changed\0" as *const u8 as *const ::core::ffi::c_char,
        w,
    );
    server_redraw_window(&*(w));
    return CMD_RETURN_NORMAL;
}
unsafe fn cmd_join_pane_exec(mut self_0: *mut cmd, mut item: *mut cmdq_item) -> cmd_retval {
    let mut args: *mut args = cmd_get_args_mut(&mut *self_0).map_or(std::ptr::null_mut(), |args| args);
    let current = cmdq_get_state_owned(item);
    let mut target: *mut cmd_find_state = crate::src::cmd::queue::cmdq_get_target_mut(&mut *item);
    let mut source: *mut cmd_find_state = crate::src::cmd::queue::cmdq_get_source_mut(&mut *item);
    let mut dst_s: *mut session = ::core::ptr::null_mut::<session>();
    let mut src_wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut dst_wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut src_w: *mut window = ::core::ptr::null_mut::<window>();
    let mut dst_w: *mut window = ::core::ptr::null_mut::<window>();
    let mut src_wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut dst_wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut dst_idx: ::core::ffi::c_int = 0;
    let mut lc: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    dst_s = (*target).s_ptr();
    dst_wl = (*target).wl_ptr();
    dst_wp = (*target).wp_ptr();
    dst_w = (*dst_wl).window_ptr();
    dst_idx = (*dst_wl).idx;
    if std::ptr::eq(cmd_get_entry(&*self_0), &cmd_move_pane_entry) {
        if args_has(args, 'M' as i32 as u_char) != 0 {
            return cmd_join_pane_mouse_update(item);
        }
        if args_has(args, 'P' as i32 as u_char) != 0
            || args_has(args, 'z' as i32 as u_char) != 0
            || args_has(args, 'X' as i32 as u_char) != 0
            || args_has(args, 'Y' as i32 as u_char) != 0
            || args_has(args, 'U' as i32 as u_char) != 0
            || args_has(args, 'D' as i32 as u_char) != 0
            || args_has(args, 'L' as i32 as u_char) != 0
            || args_has(args, 'R' as i32 as u_char) != 0
        {
            if window_pane_is_floating(&*dst_wp) == 0 {
                cmdq_error(item, |out| out.write_all(b"pane is not floating"));
                return CMD_RETURN_ERROR;
            }
            server_unzoom_window(dst_w);
            s = args_get(&*(args), 'P' as i32 as u_char).map_or(std::ptr::null(), |value| value.as_ptr());
            if !s.is_null() {
                return cmd_join_pane_place(item, dst_wl, dst_wp, s);
            }
            s = args_get(&*(args), 'z' as i32 as u_char).map_or(std::ptr::null(), |value| value.as_ptr());
            if !s.is_null() {
                return cmd_join_pane_zindex(item, dst_wl, dst_wp, s);
            }
            return cmd_join_pane_move(item, args, dst_wl, dst_wp);
        }
    }
    src_wl = (*source).wl_ptr();
    src_wp = (*source).wp_ptr();
    src_w = (*src_wl).window_ptr();
    if (*src_w).modal.ptr_eq(&(*src_wp).observer) || (*dst_w).modal.ptr_eq(&(*dst_wp).observer) {
        cmdq_error(item, |out| out.write_all(b"pane is modal"));
        return CMD_RETURN_ERROR;
    }
    server_unzoom_window(dst_w);
    server_unzoom_window(src_w);
    if src_wp == dst_wp {
        if window_pane_is_floating(&*src_wp) != 0 {
            return cmd_join_pane_tile(item, args, src_w, src_wp);
        }
        cmdq_error(item, |out| {
            out.write_all(b"source and target panes must be different")
        });
        return CMD_RETURN_ERROR;
    }
    if args_has(args, 'h' as i32 as u_char) != 0 {
        flags |= SPAWN_HORIZONTAL;
    }
    if args_has(args, 'b' as i32 as u_char) != 0 {
        flags |= SPAWN_BEFORE;
    }
    if args_has(args, 'f' as i32 as u_char) != 0 {
        flags |= SPAWN_FULLSIZE;
    }
    lc = match layout_get_tiled_cell(item, args, dst_w, dst_wp, flags) {
        Ok(cell) => cell,
        Err(cause) => {
            cmdq_error(item, |out| {
                out.write_all(b"size or position ")?;
                write_cstr(out, cause.as_ptr())
            });
            return CMD_RETURN_ERROR;
        }
    };
    layout_close_pane(src_wp);
    server_client_remove_pane(src_wp);
    window_lost_pane(src_w, src_wp);
    window_pane_list_remove(&mut *src_w, &*src_wp);
    window_pane_z_remove(&mut *src_w, &*src_wp);
    (*src_wp).window = (*dst_w).observer.clone();
    options_set_parent(options_owner_ptr(&mut (*src_wp).options).map_or(std::ptr::null_mut(), |options| options), options_owner_ptr(&mut (*dst_w).options).map_or(std::ptr::null_mut(), |options| options));
    (*src_wp).flags |= PANE_STYLECHANGED | PANE_THEMECHANGED;
    if flags & SPAWN_BEFORE != 0 {
        window_pane_list_insert_before(&mut *dst_w, &*dst_wp, &*src_wp);
        window_pane_z_insert_before(&mut *dst_w, &*dst_wp, &*src_wp);
    } else {
        window_pane_list_insert_after(&mut *dst_w, &*dst_wp, &*src_wp);
        window_pane_z_insert_after(&mut *dst_w, &*dst_wp, &*src_wp);
    }
    layout_assign_pane(lc, src_wp, 0 as ::core::ffi::c_int);
    colour_palette_from_option(Some(&mut (*src_wp).palette), options_owner_ptr(&mut (*src_wp).options).map_or(std::ptr::null_mut(), |options| options));
    recalculate_sizes();
    server_redraw_window(&*(src_w));
    server_redraw_window(&*(dst_w));
    if args_has(args, 'd' as i32 as u_char) == 0 {
        window_set_active_pane(dst_w, src_wp, 1 as ::core::ffi::c_int);
        session_select(dst_s, dst_idx);
        cmd_find_from_session(&mut *current.current.borrow_mut(), dst_s, 0 as ::core::ffi::c_int);
        server_redraw_session(&*(dst_s));
    } else {
        server_status_session(&*(dst_s));
    }
    window_fire_pane_moved(src_wp, src_w, (*src_wl).idx, dst_w, dst_idx);
    if window_count_panes(&*src_w, 1 as ::core::ffi::c_int) == 0 as u_int {
        server_kill_window((*source).w.upgrade().expect("live source window"), 1);
    } else {
        events_fire_window(
            b"window-layout-changed\0" as *const u8 as *const ::core::ffi::c_char,
            src_w,
        );
    }
    events_fire_window(
        b"window-layout-changed\0" as *const u8 as *const ::core::ffi::c_char,
        dst_w,
    );
    return CMD_RETURN_NORMAL;
}
