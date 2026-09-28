use crate::src::options::options_owner_ptr;
use crate::src::arguments::{args_get, args_has};
use crate::src::cmd::find::{
    cmd_find_from_pane, cmd_find_from_winlink, cmd_find_from_winlink_pane,
};
use crate::src::cmd::queue::{
    cmdq_error, cmdq_get_state_owned, cmdq_get_target, cmdq_insert_hook, cmdq_print,
};
use crate::src::cmd::{cmd_get_args_mut, cmd_get_entry};
use crate::src::events::events_fire;
use crate::src::events_payload::{
    event_payload_create, event_payload_set_int, event_payload_set_pane, event_payload_set_string,
    event_payload_set_target, event_payload_set_window,
};
use crate::src::format::bytes::write_cstr;
use crate::src::format::format_single_from_target_cstring;
use crate::src::options::{options_get_string, options_set_string};
use crate::src::screen::screen_set_title;
use crate::src::server::clients;
use crate::src::server::{
    marked_pane, server_check_marked, server_clear_marked, server_is_marked, server_set_marked,
};
use crate::src::server_fn::{
    server_redraw_client, server_redraw_window, server_redraw_window_borders, server_status_window,
};
use crate::src::session::session_has;
use crate::src::shared::events::event_payload;
use crate::src::tty::tty_window_bigger;
use crate::src::window::{
    window_count_panes, window_pane_find_down, window_pane_find_left, window_pane_find_right,
    window_pane_find_up, window_pane_is_floating, window_pane_is_visible, window_pane_next,
    window_pane_previous, window_pane_stack_first, window_pop_zoom, window_push_zoom,
    window_redraw_active_switch, window_set_active_pane,
};

use crate::src::shared::abi::*;
use crate::src::shared::arguments::{args, args_parse};
use crate::src::shared::client::client;
use crate::src::shared::client::{CLIENT_CONTROL, CLIENT_REDRAWBORDERS, CLIENT_REDRAWSTATUS};
use crate::src::shared::command::*;
use crate::src::shared::command::{cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmdq_item};
use crate::src::shared::options::{options, options_entry};
use crate::src::shared::pane::window_pane;
use crate::src::shared::pane::{PANE_INPUTOFF, PANE_REDRAW, PANE_STYLECHANGED, PANE_THEMECHANGED};
use crate::src::shared::session::session;
use crate::src::shared::window::{window, winlink};
pub static cmd_select_pane_entry: cmd_entry = {
    cmd_entry {
        name: c"select-pane",
        alias: Some(c"selectp"),
        args: args_parse {
            template: c"DdegLlMmP:RT:t:UZ",
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: c"[-DdeLlMmRUZ] [-T title] [-t target-pane]",
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
        flags: 0 as ::core::ffi::c_int,
        exec: Some(cmd_select_pane_exec as unsafe fn(*mut cmd, *mut cmdq_item) -> cmd_retval),
    }
};
pub static cmd_last_pane_entry: cmd_entry = {
    cmd_entry {
        name: c"last-pane",
        alias: Some(c"lastp"),
        args: args_parse {
            template: c"det:Z",
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: c"[-deZ] [-t target-window]",
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_WINDOW,
            flags: 0 as ::core::ffi::c_int,
        },
        flags: 0 as ::core::ffi::c_int,
        exec: Some(cmd_select_pane_exec as unsafe fn(*mut cmd, *mut cmdq_item) -> cmd_retval),
    }
};
unsafe fn cmd_select_pane_redraw(mut w: *mut window) {
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut registry_c_owner = clients.first();
    c = registry_c_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    while !c.is_null() {
        if !((*c).session_ptr().is_null() || (*c).flags & CLIENT_CONTROL as uint64_t != 0) {
            if (*(*(*c).session_ptr()).curw_ptr()).window_ptr() == w && tty_window_bigger(&raw mut (*c).tty) != 0 {
                server_redraw_client(&mut *(c));
            } else {
                if (*(*(*c).session_ptr()).curw_ptr()).window_ptr() == w {
                    (*c).flags |= CLIENT_REDRAWBORDERS as uint64_t;
                }
                if session_has(&*(*c).session_ptr(), &*w) != 0 {
                    (*c).flags |= CLIENT_REDRAWSTATUS as uint64_t;
                }
            }
        }
        registry_c_owner = clients.next(registry_c_owner.as_ref().expect("current registry client"));
        c = registry_c_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    }
}
unsafe fn cmd_select_pane_marked_pane(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
) -> cmd_retval {
    let mut args: *mut args = cmd_get_args_mut(&mut *self_0).map_or(std::ptr::null_mut(), |args| args);
    let mut target: *mut cmd_find_state = crate::src::cmd::queue::cmdq_get_target_mut(&mut *item);
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        s: std::rc::Weak::new(),
        wl: refbox::Weak::new(),
        w: std::rc::Weak::new(),
        wp: std::rc::Weak::new(),
        idx: 0,
    };
    let mut wl: *mut winlink = (*target).wl_ptr();
    let mut wp: *mut window_pane = (*target).wp_ptr();
    let mut lwp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut mwp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut s: *mut session = (*target).s_ptr();
    if args_has(args, 'm' as i32 as u_char) != 0 && window_pane_is_visible(&*wp) == 0 {
        return CMD_RETURN_NORMAL;
    }
    if server_check_marked() != 0 {
        lwp = marked_pane.wp_ptr();
    }
    if args_has(args, 'M' as i32 as u_char) != 0 || server_is_marked(s, wl, wp) != 0 {
        server_clear_marked();
    } else {
        server_set_marked(s, wl, wp);
    }
    mwp = marked_pane.wp_ptr();
    let mut ep = event_payload_create();
    if !mwp.is_null() {
        cmd_find_from_pane(&raw mut fs, mwp, 0 as ::core::ffi::c_int);
    } else if !lwp.is_null() {
        cmd_find_from_pane(&raw mut fs, lwp, 0 as ::core::ffi::c_int);
    } else {
        cmd_find_from_pane(&raw mut fs, wp, 0 as ::core::ffi::c_int);
    }
    event_payload_set_target(&mut *ep, &fs);
    if !mwp.is_null() {
        event_payload_set_pane(
            &mut *ep,
            b"pane\0" as *const u8 as *const ::core::ffi::c_char,
            mwp,
        );
        event_payload_set_pane(
            &mut *ep,
            b"new_pane\0" as *const u8 as *const ::core::ffi::c_char,
            mwp,
        );
        event_payload_set_window(
            &mut *ep,
            b"window\0" as *const u8 as *const ::core::ffi::c_char,
            (*mwp).window_ptr(),
        );
    } else if !lwp.is_null() {
        event_payload_set_pane(
            &mut *ep,
            b"pane\0" as *const u8 as *const ::core::ffi::c_char,
            lwp,
        );
        event_payload_set_window(
            &mut *ep,
            b"window\0" as *const u8 as *const ::core::ffi::c_char,
            (*lwp).window_ptr(),
        );
    } else {
        event_payload_set_pane(
            &mut *ep,
            b"pane\0" as *const u8 as *const ::core::ffi::c_char,
            wp,
        );
        event_payload_set_window(
            &mut *ep,
            b"window\0" as *const u8 as *const ::core::ffi::c_char,
            (*wp).window_ptr(),
        );
    }
    if !lwp.is_null() {
        event_payload_set_pane(
            &mut *ep,
            b"old_pane\0" as *const u8 as *const ::core::ffi::c_char,
            lwp,
        );
    }
    event_payload_set_int(
        &mut *ep,
        b"marked\0" as *const u8 as *const ::core::ffi::c_char,
        (mwp != NULL as *mut window_pane) as ::core::ffi::c_int,
    );
    events_fire(
        b"marked-pane-changed\0" as *const u8 as *const ::core::ffi::c_char,
        ep,
    );
    if !lwp.is_null() {
        (*lwp).flags |= PANE_REDRAW | PANE_STYLECHANGED | PANE_THEMECHANGED;
        server_redraw_window_borders(&*((*lwp).window_ptr()));
        server_status_window(&*((*lwp).window_ptr()));
    }
    if !mwp.is_null() {
        (*mwp).flags |= PANE_REDRAW | PANE_STYLECHANGED | PANE_THEMECHANGED;
        server_redraw_window_borders(&*((*mwp).window_ptr()));
        server_status_window(&*((*mwp).window_ptr()));
    }
    if window_pane_is_floating(&*wp) != 0 {
        window_redraw_active_switch((*wp).window_ptr(), wp);
        window_set_active_pane((*wp).window_ptr(), wp, 1 as ::core::ffi::c_int);
    }
    return CMD_RETURN_NORMAL;
}
unsafe fn cmd_select_pane_exec(mut self_0: *mut cmd, mut item: *mut cmdq_item) -> cmd_retval {
    let mut args: *mut args = cmd_get_args_mut(&mut *self_0).map_or(std::ptr::null_mut(), |args| args);
    let entry = cmd_get_entry(&*self_0);
    let current = cmdq_get_state_owned(item);
    let mut target: *mut cmd_find_state = crate::src::cmd::queue::cmdq_get_target_mut(&mut *item);
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        s: std::rc::Weak::new(),
        wl: refbox::Weak::new(),
        w: std::rc::Weak::new(),
        wp: std::rc::Weak::new(),
        idx: 0,
    };
    let mut wl: *mut winlink = (*target).wl_ptr();
    let mut w: *mut window = (*wl).window_ptr();
    let mut s: *mut session = (*target).s_ptr();
    let mut wp: *mut window_pane = (*target).wp_ptr();
    let mut lastwp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut oo: *mut options = options_owner_ptr(&mut (*wp).options).map_or(std::ptr::null_mut(), |options| options);
    let mut style: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut visible: ::core::ffi::c_int = 0;
    let mut Zflag: ::core::ffi::c_int = args_has(args, 'Z' as i32 as u_char);
    if std::ptr::eq(entry, &cmd_last_pane_entry) || args_has(args, 'l' as i32 as u_char) != 0 {
        lastwp = window_pane_stack_first(w.as_ref()).as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
        if lastwp.is_null() && window_count_panes(&*w, 1 as ::core::ffi::c_int) == 2 as u_int {
            lastwp = window_pane_previous(((*w).active_ptr()).as_ref()).as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
            if lastwp.is_null() {
                lastwp = window_pane_next(((*w).active_ptr()).as_ref()).as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
            }
        }
        if lastwp.is_null() {
            cmdq_error(item, |out| out.write_all(b"no last pane"));
            return CMD_RETURN_ERROR;
        }
        if args_has(args, 'e' as i32 as u_char) != 0 {
            (*lastwp).flags &= !PANE_INPUTOFF;
            server_redraw_window_borders(&*((*lastwp).window_ptr()));
            server_status_window(&*((*lastwp).window_ptr()));
        } else if args_has(args, 'd' as i32 as u_char) != 0 {
            (*lastwp).flags |= PANE_INPUTOFF;
            server_redraw_window_borders(&*((*lastwp).window_ptr()));
            server_status_window(&*((*lastwp).window_ptr()));
        } else {
            if (*w).modal.upgrade().is_some() && !lastwp.as_ref().is_some_and(|pane| (*w).modal.ptr_eq(&pane.observer)) {
                visible = 1 as ::core::ffi::c_int;
            } else {
                visible = window_pane_is_visible(&*lastwp);
            }
            if visible == 0 && window_push_zoom(w, 0 as ::core::ffi::c_int, Zflag) != 0 {
                server_redraw_window(&*(w));
            }
            window_redraw_active_switch(w, lastwp);
            if window_set_active_pane(w, lastwp, 1 as ::core::ffi::c_int) != 0 {
                cmd_find_from_winlink(&mut *current.current.borrow_mut(), wl, 0 as ::core::ffi::c_int);
                cmd_select_pane_redraw(w);
            }
            if visible == 0 && window_pop_zoom(w) != 0 {
                server_redraw_window(&*(w));
            }
        }
        return CMD_RETURN_NORMAL;
    }
    if args_has(args, 'm' as i32 as u_char) != 0 || args_has(args, 'M' as i32 as u_char) != 0 {
        return cmd_select_pane_marked_pane(self_0, item);
    }
    style = args_get(&*(args), 'P' as i32 as u_char).map_or(std::ptr::null(), |value| value.as_ptr());
    if !style.is_null() {
        o = options_set_string(
            oo,
            b"window-style\0" as *const u8 as *const ::core::ffi::c_char,
            0 as ::core::ffi::c_int,
            |out| write_cstr(out, style),
        );
        if o.is_null() {
            cmdq_error(item, |out| {
                out.write_all(b"bad style: ")?;
                write_cstr(out, style)
            });
            return CMD_RETURN_ERROR;
        }
        options_set_string(
            oo,
            b"window-active-style\0" as *const u8 as *const ::core::ffi::c_char,
            0 as ::core::ffi::c_int,
            |out| write_cstr(out, style),
        );
        (*wp).flags |= PANE_REDRAW | PANE_STYLECHANGED | PANE_THEMECHANGED;
    }
    if args_has(args, 'g' as i32 as u_char) != 0 {
        cmdq_print(item, |out| {
            write_cstr(
                out,
                options_get_string(
                    oo,
                    b"window-style\0" as *const u8 as *const ::core::ffi::c_char,
                ),
            )
        });
        return CMD_RETURN_NORMAL;
    }
    let direction_source = wp.as_ref().and_then(|pane| pane.observer.upgrade());
    let selected_pane_owner;
    if args_has(args, 'L' as i32 as u_char) != 0 {
        window_push_zoom(w, 0 as ::core::ffi::c_int, 1 as ::core::ffi::c_int);
        selected_pane_owner = window_pane_find_left(direction_source.as_ref());
        wp = selected_pane_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
        window_pop_zoom(w);
    } else if args_has(args, 'R' as i32 as u_char) != 0 {
        window_push_zoom(w, 0 as ::core::ffi::c_int, 1 as ::core::ffi::c_int);
        selected_pane_owner = window_pane_find_right(direction_source.as_ref());
        wp = selected_pane_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
        window_pop_zoom(w);
    } else if args_has(args, 'U' as i32 as u_char) != 0 {
        window_push_zoom(w, 0 as ::core::ffi::c_int, 1 as ::core::ffi::c_int);
        selected_pane_owner = window_pane_find_up(direction_source.as_ref());
        wp = selected_pane_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
        window_pop_zoom(w);
    } else if args_has(args, 'D' as i32 as u_char) != 0 {
        window_push_zoom(w, 0 as ::core::ffi::c_int, 1 as ::core::ffi::c_int);
        selected_pane_owner = window_pane_find_down(direction_source.as_ref());
        wp = selected_pane_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
        window_pop_zoom(w);
    }
    if wp.is_null() {
        return CMD_RETURN_NORMAL;
    }
    if args_has(args, 'e' as i32 as u_char) != 0 {
        (*wp).flags &= !PANE_INPUTOFF;
        server_redraw_window_borders(&*((*wp).window_ptr()));
        server_status_window(&*((*wp).window_ptr()));
        return CMD_RETURN_NORMAL;
    }
    if args_has(args, 'd' as i32 as u_char) != 0 {
        (*wp).flags |= PANE_INPUTOFF;
        server_redraw_window_borders(&*((*wp).window_ptr()));
        server_status_window(&*((*wp).window_ptr()));
        return CMD_RETURN_NORMAL;
    }
    if args_has(args, 'T' as i32 as u_char) != 0 {
        let title = format_single_from_target_cstring(item, args_get(&*(args), 'T' as i32 as u_char).map_or(std::ptr::null(), |value| value.as_ptr()));
        if screen_set_title(&mut (*wp).base, &title, 0 as ::core::ffi::c_int) != 0 {
            let mut ep = event_payload_create();
            cmd_find_from_pane(&raw mut fs, wp, 0 as ::core::ffi::c_int);
            event_payload_set_target(&mut *ep, &fs);
            event_payload_set_pane(
                &mut *ep,
                b"pane\0" as *const u8 as *const ::core::ffi::c_char,
                wp,
            );
            event_payload_set_window(
                &mut *ep,
                b"window\0" as *const u8 as *const ::core::ffi::c_char,
                (*wp).window_ptr(),
            );
            event_payload_set_string(
                &mut *ep,
                b"new_title\0" as *const u8 as *const ::core::ffi::c_char,
                |out| write_cstr(out, title.as_ptr()),
            );
            events_fire(
                b"pane-title-changed\0" as *const u8 as *const ::core::ffi::c_char,
                ep,
            );
            server_redraw_window_borders(&*((*wp).window_ptr()));
            server_status_window(&*((*wp).window_ptr()));
        }
        return CMD_RETURN_NORMAL;
    }
    if wp == (*w).active_ptr() {
        return CMD_RETURN_NORMAL;
    }
    if (*w).modal.upgrade().is_some() && !wp.as_ref().is_some_and(|pane| (*w).modal.ptr_eq(&pane.observer)) {
        visible = 1 as ::core::ffi::c_int;
    } else {
        visible = window_pane_is_visible(&*wp);
    }
    if visible == 0 && window_push_zoom(w, 0 as ::core::ffi::c_int, Zflag) != 0 {
        server_redraw_window(&*(w));
    }
    window_redraw_active_switch(w, wp);
    if window_set_active_pane(w, wp, 1 as ::core::ffi::c_int) != 0 {
        cmd_find_from_winlink_pane(&mut *current.current.borrow_mut(), wl, wp, 0 as ::core::ffi::c_int);
    }
    cmdq_insert_hook(s, item, &mut current.current_snapshot(), |out| out.write_all(b"after-select-pane"));
    cmd_select_pane_redraw(w);
    if visible == 0 && window_pop_zoom(w) != 0 {
        server_redraw_window(&*(w));
    }
    return CMD_RETURN_NORMAL;
}
