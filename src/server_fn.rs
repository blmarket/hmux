use crate::src::tty_term::tty_term_owner_ptr;
use crate::src::options::options_owner_ptr;
use crate::src::cmd::find::cmd_find_from_pane;
use crate::src::events::{events_fire, events_fire_winlink};
use crate::src::events_payload::{
    event_payload_create, event_payload_set_int, event_payload_set_pane, event_payload_set_string,
    event_payload_set_target, event_payload_set_window,
};
use crate::src::ffi::libc::{close, getpid, gettimeofday, kill, memcpy, strlen};
use crate::src::ffi::utempter::utempter_remove_record;
use crate::src::format::bytes::write_cstr;
use crate::src::format::format_single_cstring;
use crate::src::format_draw::format_draw;
use crate::src::grid::grid_default_cell;
use crate::src::layout::layout_close_pane;
use crate::src::options::{options_get_number, options_get_string};
use crate::src::proc::proc_send;
use crate::src::reactor::bufferevent_free;
use crate::src::resize::recalculate_sizes;
use crate::src::screen_write::{
    screen_write_cursormove, screen_write_linefeed, screen_write_scrollregion,
    screen_write_start_pane, screen_write_stop,
};
use crate::src::server::clients;
use crate::src::server::marked_pane;
use crate::src::server_client::{server_client_remove_pane, server_client_set_session};
use crate::src::session::sessions;
use crate::src::session::{
    session_alive, session_attach, session_destroy, session_detach, session_group_contains,
    session_group_count, session_has, session_next_session, session_previous_session,
    session_renumber_windows, session_select, sessions_after, sessions_key, sessions_minmax,
    sessions_next,
};
use crate::src::shared::events::event_payload;
use crate::src::shared::session::session_group;
use crate::src::tmux::sig2name;
use crate::src::tty::{tty_raw, tty_stop_tty};
use crate::src::tty_term::tty_term_string;
use crate::src::window::{
    window_add_ref, window_count_panes, window_pane_first, window_pop_zoom, window_push_zoom,
    window_remove_pane, window_remove_ref, window_unzoom, winlink_find_by_index,
    winlink_find_by_window, winlink_remove, winlink_stack_remove,
};

use crate::src::compat::imsg::*;
use crate::src::compat::imsg::{IMSG_HEADER_SIZE, MAX_IMSGSIZE};
use crate::src::shared::abi::*;
use crate::src::shared::client::client;
use crate::src::shared::client::{
    CLIENT_ALLREDRAWFLAGS, CLIENT_CONTROL, CLIENT_EXIT, CLIENT_NO_DETACH_ON_DESTROY,
    CLIENT_REDRAWBORDERS, CLIENT_REDRAWMENU, CLIENT_REDRAWSTATUS, CLIENT_SUSPENDED,
};
use crate::src::shared::command::{cmd_find_state, cmdq_item};
use crate::src::shared::event::*;
use crate::src::shared::grid::*;
use crate::src::shared::pane::window_pane;
use crate::src::shared::pane::{
    PANE_FLOATOVERZOOM, PANE_REDRAW, PANE_STATUSDRAWN, PANE_STATUSREADY,
};
use crate::src::shared::screen::{screen, MODE_CURSOR};
use crate::src::shared::screen_write::screen_write_ctx;
use crate::src::shared::session::session;
use crate::src::shared::signal::SIGCHLD;
use crate::src::shared::sort::sort_criteria;
use crate::src::shared::sort::*;
use crate::src::shared::style::*;
use crate::src::shared::tree::RB_NEGINF;
use crate::src::shared::tty::*;
use crate::src::shared::window::WINLINK_ALERTFLAGS;
use crate::src::shared::window::{window, winlink};

unsafe fn server_fire_pane_exit(mut name: *const ::core::ffi::c_char, mut wp: *mut window_pane) {
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null::<cmd_find_state>() as *mut cmd_find_state,
        s: std::rc::Weak::new(),
        wl: refbox::Weak::new(),
        w: std::rc::Weak::new(),
        wp: std::rc::Weak::new(),
        idx: 0,
    };
    let mut status: ::core::ffi::c_int = (*wp).status;
    let mut signame: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if ((status & 0x7f as ::core::ffi::c_int) + 1 as ::core::ffi::c_int) as ::core::ffi::c_schar
        as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int
        > 0 as ::core::ffi::c_int
    {
        signame = sig2name(status & 0x7f as ::core::ffi::c_int);
    }
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
        (*wp).window as *mut window,
    );
    if status & 0x7f as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        event_payload_set_int(
            &mut *ep,
            b"exit_status\0" as *const u8 as *const ::core::ffi::c_char,
            (status & 0xff00 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int,
        );
    } else if ((status & 0x7f as ::core::ffi::c_int) + 1 as ::core::ffi::c_int)
        as ::core::ffi::c_schar as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int
        > 0 as ::core::ffi::c_int
    {
        event_payload_set_string(
            &mut *ep,
            b"exit_signal\0" as *const u8 as *const ::core::ffi::c_char,
            |out| write_cstr(out, signame),
        );
    }
    event_payload_set_int(
        &mut *ep,
        b"exit_success\0" as *const u8 as *const ::core::ffi::c_char,
        (status == 0 as ::core::ffi::c_int) as ::core::ffi::c_int,
    );
    events_fire(name, ep);
}
pub unsafe fn server_redraw_client(mut c: *mut client) {
    (*c).flags |= CLIENT_ALLREDRAWFLAGS as uint64_t;
}
pub unsafe fn server_status_client(mut c: *mut client) {
    (*c).flags |= CLIENT_REDRAWSTATUS as uint64_t;
}
pub unsafe fn server_redraw_session(mut s: *mut session) {
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    c = clients.first();
    while !c.is_null() {
        if (*c).session == s {
            server_redraw_client(c);
        }
        c = clients.next(c);
    }
}
pub unsafe fn server_redraw_session_group(mut s: *mut session) {
    let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
    sg = session_group_contains(s);
    if sg.is_null() {
        server_redraw_session(s);
    } else {
        for s in crate::src::session::session_group_members(sg) {
            server_redraw_session(s);
        }
    };
}
pub unsafe fn server_status_session(mut s: *mut session) {
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    c = clients.first();
    while !c.is_null() {
        if (*c).session == s {
            server_status_client(c);
        }
        c = clients.next(c);
    }
}
pub unsafe fn server_status_session_group(mut s: *mut session) {
    let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
    sg = session_group_contains(s);
    if sg.is_null() {
        server_status_session(s);
    } else {
        for s in crate::src::session::session_group_members(sg) {
            server_status_session(s);
        }
    };
}
pub unsafe fn server_redraw_window(mut w: *mut window) {
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    c = clients.first();
    while !c.is_null() {
        if !(*c).session.is_null()
            && !(*(*c).session).curw.is_null()
            && (*(*(*c).session).curw).window_ptr() == w
        {
            server_redraw_client(c);
        }
        c = clients.next(c);
    }
}
pub unsafe fn server_redraw_window_menu(mut w: *mut window) {
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    c = clients.first();
    while !c.is_null() {
        if !(*c).session.is_null()
            && !(*(*c).session).curw.is_null()
            && (*(*(*c).session).curw).window_ptr() == w
        {
            (*c).flags |= CLIENT_REDRAWMENU as uint64_t;
        }
        c = clients.next(c);
    }
}
pub unsafe fn server_redraw_window_borders(mut w: *mut window) {
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    c = clients.first();
    while !c.is_null() {
        if !(*c).session.is_null()
            && !(*(*c).session).curw.is_null()
            && (*(*(*c).session).curw).window_ptr() == w
        {
            (*c).flags |= CLIENT_REDRAWBORDERS as uint64_t;
        }
        c = clients.next(c);
    }
}
pub unsafe fn server_status_window(mut w: *mut window) {
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut s_owner = sessions_minmax(&*std::ptr::addr_of!(sessions));
    s = s_owner.as_ref().map_or(std::ptr::null_mut(), crate::src::shared::rc::as_ptr);
    while !s.is_null() {
        if session_has(s, w) != 0 {
            server_status_session(s);
        }
        s_owner = sessions_next(&*s);
        s = s_owner.as_ref().map_or(std::ptr::null_mut(), crate::src::shared::rc::as_ptr);
    }
}
pub unsafe fn server_lock() {
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    c = clients.first();
    while !c.is_null() {
        if !(*c).session.is_null() {
            server_lock_client(c);
        }
        c = clients.next(c);
    }
}
pub unsafe fn server_lock_session(mut s: *mut session) {
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    c = clients.first();
    while !c.is_null() {
        if (*c).session == s {
            server_lock_client(c);
        }
        c = clients.next(c);
    }
}
pub unsafe fn server_lock_client(mut c: *mut client) {
    let mut cmd: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
        return;
    }
    if (*c).flags & CLIENT_SUSPENDED as uint64_t != 0 {
        return;
    }
    cmd = options_get_string(
        options_owner_ptr(&mut (*(*c).session).options).map_or(std::ptr::null_mut(), |options| options),
        b"lock-command\0" as *const u8 as *const ::core::ffi::c_char,
    );
    if *cmd as ::core::ffi::c_int == '\0' as i32
        || strlen(cmd).wrapping_add(1 as size_t)
            > (MAX_IMSGSIZE as usize).wrapping_sub(IMSG_HEADER_SIZE)
    {
        return;
    }
    tty_stop_tty(&raw mut (*c).tty);
    tty_raw(
        &raw mut (*c).tty,
        tty_term_string(&*(tty_term_owner_ptr(&(*c).tty.term).map_or(std::ptr::null(), |term| term)), TTYC_SMCUP).as_ptr(),
    );
    tty_raw(
        &raw mut (*c).tty,
        tty_term_string(&*(tty_term_owner_ptr(&(*c).tty.term).map_or(std::ptr::null(), |term| term)), TTYC_CLEAR).as_ptr(),
    );
    tty_raw(&raw mut (*c).tty, tty_term_string(&*(tty_term_owner_ptr(&(*c).tty.term).map_or(std::ptr::null(), |term| term)), TTYC_E3).as_ptr());
    (*c).flags |= CLIENT_SUSPENDED as uint64_t;
    proc_send(
        (*c).peer,
        MSG_LOCK,
        -(1 as ::core::ffi::c_int),
        cmd as *const ::core::ffi::c_void,
        strlen(cmd).wrapping_add(1 as size_t),
    );
}
pub unsafe fn server_kill_pane(mut wp: *mut window_pane) {
    let mut w: *mut window = (*wp).window as *mut window;
    if window_count_panes(w, 1 as ::core::ffi::c_int) == 1 as u_int {
        server_kill_window(w, 1 as ::core::ffi::c_int);
        recalculate_sizes();
    } else {
        window_push_zoom(w, 0 as ::core::ffi::c_int, (*wp).flags & PANE_FLOATOVERZOOM);
        server_client_remove_pane(wp);
        layout_close_pane(wp);
        window_remove_pane(w, wp);
        window_pop_zoom(w);
        server_redraw_window(w);
    };
}
pub unsafe fn server_kill_window(mut w: *mut window, mut renumber: ::core::ffi::c_int) {
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let owner = window_add_ref(
        w,
        b"server_kill_window\0" as *const u8 as *const ::core::ffi::c_char,
    );
    let mut s_owner = sessions_minmax(&*std::ptr::addr_of!(sessions));
    s = s_owner.as_ref().map_or(std::ptr::null_mut(), crate::src::shared::rc::as_ptr);
    while !s.is_null() {
        // Destroying a group may remove both s and its next session.
        let name = sessions_key(&*s);
        if !(session_has(s, w) == 0) {
            server_unzoom_window(w);
            loop {
                wl = winlink_find_by_window(&raw mut (*s).windows, w);
                if wl.is_null() {
                    break;
                }
                if session_detach(s, wl) != 0 {
                    server_destroy_session_group(s);
                    break;
                } else {
                    server_redraw_session_group(s);
                }
            }
            if renumber != 0 && session_alive(s) != 0 {
                server_renumber_session(s);
            }
        }
        s_owner = sessions_after(&*std::ptr::addr_of!(sessions), &name);
        s = s_owner.as_ref().map_or(std::ptr::null_mut(), crate::src::shared::rc::as_ptr);
    }
    recalculate_sizes();
    window_remove_ref(
        owner,
        b"server_kill_window\0" as *const u8 as *const ::core::ffi::c_char,
    );
}
pub unsafe fn server_renumber_session(mut s: *mut session) {
    let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
    if options_get_number(
        options_owner_ptr(&mut (*s).options).map_or(std::ptr::null_mut(), |options| options),
        b"renumber-windows\0" as *const u8 as *const ::core::ffi::c_char,
    ) != 0
    {
        sg = session_group_contains(s);
        if !sg.is_null() {
            for s in crate::src::session::session_group_members(sg) {
                session_renumber_windows(s);
            }
        } else {
            session_renumber_windows(s);
        }
    }
}
pub unsafe fn server_renumber_all() {
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut s_owner = sessions_minmax(&*std::ptr::addr_of!(sessions));
    s = s_owner.as_ref().map_or(std::ptr::null_mut(), crate::src::shared::rc::as_ptr);
    while !s.is_null() {
        server_renumber_session(s);
        s_owner = sessions_next(&*s);
        s = s_owner.as_ref().map_or(std::ptr::null_mut(), crate::src::shared::rc::as_ptr);
    }
}
pub unsafe fn server_link_window(
    mut src: *mut session,
    mut srcwl: *mut winlink,
    mut dst: *mut session,
    mut dstidx: ::core::ffi::c_int,
    mut killflag: ::core::ffi::c_int,
    mut selectflag: ::core::ffi::c_int,
) -> Result<(), std::ffi::CString> {
    let mut dstwl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut srcsg: *mut session_group = ::core::ptr::null_mut::<session_group>();
    let mut dstsg: *mut session_group = ::core::ptr::null_mut::<session_group>();
    srcsg = session_group_contains(src);
    dstsg = session_group_contains(dst);
    if src != dst && !srcsg.is_null() && !dstsg.is_null() && srcsg == dstsg {
        return Err(c"sessions are grouped".to_owned());
    }
    dstwl = ::core::ptr::null_mut::<winlink>();
    if dstidx != -(1 as ::core::ffi::c_int) {
        dstwl = winlink_find_by_index(&raw mut (*dst).windows, dstidx);
    }
    if !dstwl.is_null() {
        if (*dstwl).window_ptr() == (*srcwl).window_ptr() {
            return Err(std::ffi::CString::new(format!("same index: {dstidx}"))
                .expect("numeric diagnostic contains no NUL"));
        }
        if killflag != 0 {
            events_fire_winlink(
                b"window-unlinked\0" as *const u8 as *const ::core::ffi::c_char,
                dstwl,
            );
            (*dstwl).flags &= !WINLINK_ALERTFLAGS;
            winlink_stack_remove(&raw mut (*dst).lastw, dstwl);
            winlink_remove(&raw mut (*dst).windows, dstwl);
            if dstwl == (*dst).curw {
                selectflag = 1 as ::core::ffi::c_int;
                (*dst).curw = ::core::ptr::null_mut::<winlink>();
            }
        }
    }
    if dstidx == -(1 as ::core::ffi::c_int) {
        dstidx = (-(1 as ::core::ffi::c_int) as ::core::ffi::c_longlong
            - options_get_number(
                options_owner_ptr(&mut (*dst).options).map_or(std::ptr::null_mut(), |options| options),
                b"base-index\0" as *const u8 as *const ::core::ffi::c_char,
            )) as ::core::ffi::c_int;
    }
    dstwl = session_attach(dst, (*srcwl).window_ptr(), dstidx)?;
    if marked_pane.wl_ptr() == srcwl {
        marked_pane.set_wl(dstwl);
    }
    if selectflag != 0 {
        session_select(dst, (*dstwl).idx);
    }
    server_redraw_session_group(dst);
    Ok(())
}
pub unsafe fn server_unlink_window(mut s: *mut session, mut wl: *mut winlink) {
    if session_detach(s, wl) != 0 {
        server_destroy_session_group(s);
    } else {
        server_redraw_session_group(s);
    };
}
pub unsafe fn server_destroy_pane(mut wp: *mut window_pane, mut notify: ::core::ffi::c_int) {
    let mut w: *mut window = (*wp).window as *mut window;
    let mut ctx: screen_write_ctx = screen_write_ctx {
        wp: ::core::ptr::null_mut::<window_pane>(),
        s: ::core::ptr::null_mut::<screen>(),
        flags: 0,
        init_ctx_cb: None,
        item: None,
        scrolled: 0,
        bg: 0,
    };
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
    let mut remain_on_exit: ::core::ffi::c_int = 0;
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut sx: u_int = (*wp).base.grid().sx;
    let mut sy: u_int = (*wp).base.grid().sy;
    if (*wp).fd != -(1 as ::core::ffi::c_int) {
        utempter_remove_record((*wp).fd);
        kill(getpid(), SIGCHLD);
        bufferevent_free((*wp).event);
        (*wp).event = ::core::ptr::null_mut::<bufferevent>();
        close((*wp).fd);
        (*wp).fd = -(1 as ::core::ffi::c_int);
    }
    if (*wp).pipe_fd != -(1 as ::core::ffi::c_int) {
        bufferevent_free((*wp).pipe_event);
        (*wp).pipe_event = ::core::ptr::null_mut::<bufferevent>();
        close((*wp).pipe_fd);
        (*wp).pipe_fd = -(1 as ::core::ffi::c_int);
    }
    if !(*wp).flags & PANE_STATUSREADY != 0 {
        return;
    }
    remain_on_exit = options_get_number(
        options_owner_ptr(&mut (*wp).options).map_or(std::ptr::null_mut(), |options| options),
        b"remain-on-exit\0" as *const u8 as *const ::core::ffi::c_char,
    ) as ::core::ffi::c_int;
    let mut current_block_37: u64;
    match remain_on_exit {
        2 | 4 => {
            if (*wp).status & 0x7f as ::core::ffi::c_int == 0 as ::core::ffi::c_int
                && ((*wp).status & 0xff00 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int
                    == 0 as ::core::ffi::c_int
            {
                current_block_37 = 3275366147856559585;
            } else {
                current_block_37 = 2300157484894416861;
            }
        }
        1 | 3 => {
            current_block_37 = 2300157484894416861;
        }
        0 | _ => {
            current_block_37 = 3275366147856559585;
        }
    }
    match current_block_37 {
        3275366147856559585 => {}
        _ => {
            if (*wp).flags & PANE_STATUSDRAWN != 0 {
                return;
            }
            (*wp).flags |= PANE_STATUSDRAWN;
            gettimeofday(&raw mut (*wp).dead_time, NULL);
            if notify != 0 {
                server_fire_pane_exit(
                    b"pane-died\0" as *const u8 as *const ::core::ffi::c_char,
                    wp,
                );
            }
            s = options_get_string(
                options_owner_ptr(&mut (*wp).options).map_or(std::ptr::null_mut(), |options| options),
                b"remain-on-exit-format\0" as *const u8 as *const ::core::ffi::c_char,
            );
            if *s as ::core::ffi::c_int != '\0' as i32 {
                screen_write_start_pane(&mut ctx, wp, &raw mut (*wp).base);
                screen_write_scrollregion(&mut ctx, 0 as u_int, sy.wrapping_sub(1 as u_int));
                screen_write_cursormove(
                    &mut ctx,
                    0 as ::core::ffi::c_int,
                    sy.wrapping_sub(1 as u_int) as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                );
                screen_write_linefeed(&mut ctx, 1 as ::core::ffi::c_int, 8 as u_int);
                memcpy(
                    &raw mut gc as *mut ::core::ffi::c_void,
                    &raw const grid_default_cell as *const ::core::ffi::c_void,
                    ::core::mem::size_of::<grid_cell>() as size_t,
                );
                let expanded = format_single_cstring(
                    ::core::ptr::null_mut::<cmdq_item>(),
                    s,
                    ::core::ptr::null_mut::<client>(),
                    ::core::ptr::null_mut::<session>(),
                    ::core::ptr::null_mut::<winlink>(),
                    wp,
                );
                format_draw(
                    &raw mut ctx,
                    &raw mut gc,
                    sx,
                    expanded.as_ptr(),
                    ::core::ptr::null_mut::<style_ranges>(),
                    0 as ::core::ffi::c_int,
                );
                screen_write_stop(&mut ctx);
            }
            (*wp).base.mode &= !MODE_CURSOR;
            (*wp).flags |= PANE_REDRAW;
            return;
        }
    }
    if notify != 0 {
        server_fire_pane_exit(
            b"pane-exited\0" as *const u8 as *const ::core::ffi::c_char,
            wp,
        );
    }
    window_push_zoom(w, 0 as ::core::ffi::c_int, (*wp).flags & PANE_FLOATOVERZOOM);
    server_client_remove_pane(wp);
    layout_close_pane(wp);
    window_remove_pane(w, wp);
    if window_pane_first(w).as_ref().map_or(std::ptr::null_mut(), |owner| owner.get()).is_null() {
        server_kill_window(w, 1 as ::core::ffi::c_int);
    } else {
        window_pop_zoom(w);
        server_redraw_window(w);
    };
}
unsafe fn server_destroy_session_group(mut s: *mut session) {
    let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
    sg = session_group_contains(s);
    if sg.is_null() {
        server_destroy_session(s);
        session_destroy(
            s,
            1 as ::core::ffi::c_int,
            b"server_destroy_session_group\0" as *const u8 as *const ::core::ffi::c_char,
        );
    } else {
        for s in crate::src::session::session_group_members(sg) {
            server_destroy_session(s);
            session_destroy(
                s,
                1 as ::core::ffi::c_int,
                b"server_destroy_session_group\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
    };
}
unsafe fn server_find_session(
    s: *mut session,
    mut choose: impl FnMut(&session, Option<&session>) -> bool,
) -> *mut session {
    let mut s_loop: *mut session = ::core::ptr::null_mut::<session>();
    let mut s_out: *mut session = ::core::ptr::null_mut::<session>();
    let mut s_loop_owner = sessions_minmax(&*std::ptr::addr_of!(sessions));
    s_loop = s_loop_owner.as_ref().map_or(std::ptr::null_mut(), crate::src::shared::rc::as_ptr);
    while !s_loop.is_null() {
        if s_loop != s && choose(&*s_loop, (!s_out.is_null()).then(|| &*s_out)) {
            s_out = s_loop;
        }
        s_loop_owner = sessions_next(&*s_loop);
        s_loop = s_loop_owner.as_ref().map_or(std::ptr::null_mut(), crate::src::shared::rc::as_ptr);
    }
    return s_out;
}
fn server_newer_session(s_loop: &session, s_out: Option<&session>) -> bool {
    let Some(s_out) = s_out else {
        return true;
    };
    if s_loop.activity_time.tv_sec == s_out.activity_time.tv_sec {
        s_loop.activity_time.tv_usec > s_out.activity_time.tv_usec
    } else {
        s_loop.activity_time.tv_sec > s_out.activity_time.tv_sec
    }
}
fn server_newer_detached_session(s_loop: &session, s_out: Option<&session>) -> bool {
    if s_loop.attached != 0 {
        return false;
    }
    return server_newer_session(s_loop, s_out);
}
pub unsafe fn server_destroy_session(mut s: *mut session) {
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut s_new: *mut session = ::core::ptr::null_mut::<session>();
    let mut cs_new: *mut session = ::core::ptr::null_mut::<session>();
    let mut use_s: *mut session = ::core::ptr::null_mut::<session>();
    let mut sort_crit: sort_criteria = sort_criteria {
        order: SORT_NAME,
        reversed: 0,
        order_seq: &[],
    };
    let mut detach_on_destroy: ::core::ffi::c_int = 0;
    detach_on_destroy = options_get_number(
        options_owner_ptr(&mut (*s).options).map_or(std::ptr::null_mut(), |options| options),
        b"detach-on-destroy\0" as *const u8 as *const ::core::ffi::c_char,
    ) as ::core::ffi::c_int;
    if detach_on_destroy == 0 as ::core::ffi::c_int {
        s_new = server_find_session(s, server_newer_session);
    } else if detach_on_destroy == 2 as ::core::ffi::c_int {
        s_new = server_find_session(s, server_newer_detached_session);
    } else if detach_on_destroy == 3 as ::core::ffi::c_int {
        s_new = session_previous_session(s, &raw mut sort_crit);
    } else if detach_on_destroy == 4 as ::core::ffi::c_int {
        s_new = session_next_session(s, &raw mut sort_crit);
    }
    if s_new == s {
        s_new = ::core::ptr::null_mut::<session>();
    }
    if s_new.is_null()
        && (detach_on_destroy == 1 as ::core::ffi::c_int
            || detach_on_destroy == 2 as ::core::ffi::c_int)
    {
        cs_new = server_find_session(s, server_newer_session);
    }
    c = clients.first();
    while !c.is_null() {
        if !((*c).session != s) {
            use_s = s_new;
            if use_s.is_null()
                && (*c).flags as ::core::ffi::c_ulonglong & CLIENT_NO_DETACH_ON_DESTROY != 0
            {
                use_s = cs_new;
            }
            (*c).session = ::core::ptr::null_mut::<session>();
            (*c).last_session = ::core::ptr::null_mut::<session>();
            server_client_set_session(c, use_s);
            if use_s.is_null() {
                (*c).flags |= CLIENT_EXIT as uint64_t;
            }
        }
        c = clients.next(c);
    }
    recalculate_sizes();
}
pub unsafe fn server_check_unattached() {
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
    let mut current_block_4: u64;
    let mut s_owner = sessions_minmax(&*std::ptr::addr_of!(sessions));
    s = s_owner.as_ref().map_or(std::ptr::null_mut(), crate::src::shared::rc::as_ptr);
    while !s.is_null() {
        let name = sessions_key(&*s);
        if !((*s).attached != 0 as u_int) {
            match options_get_number(
                options_owner_ptr(&mut (*s).options).map_or(std::ptr::null_mut(), |options| options),
                b"destroy-unattached\0" as *const u8 as *const ::core::ffi::c_char,
            ) {
                0 => {}
                2 => {
                    current_block_4 = 6116987625208566775;
                    match current_block_4 {
                        11000743977270914936 => {
                            sg = session_group_contains(s);
                            if !sg.is_null() && session_group_count(sg) == 1 as u_int {
                                current_block_4 = 16668937799742929182;
                            } else {
                                current_block_4 = 13109137661213826276;
                            }
                        }
                        6116987625208566775 => {
                            sg = session_group_contains(s);
                            if sg.is_null() || session_group_count(sg) <= 1 as u_int {
                                current_block_4 = 16668937799742929182;
                            } else {
                                current_block_4 = 13109137661213826276;
                            }
                        }
                        _ => {}
                    }
                    match current_block_4 {
                        16668937799742929182 => {}
                        _ => {
                            server_destroy_session(s);
                            session_destroy(
                                s,
                                1 as ::core::ffi::c_int,
                                b"server_check_unattached\0" as *const u8
                                    as *const ::core::ffi::c_char,
                            );
                        }
                    }
                }
                3 => {
                    current_block_4 = 11000743977270914936;
                    match current_block_4 {
                        11000743977270914936 => {
                            sg = session_group_contains(s);
                            if !sg.is_null() && session_group_count(sg) == 1 as u_int {
                                current_block_4 = 16668937799742929182;
                            } else {
                                current_block_4 = 13109137661213826276;
                            }
                        }
                        6116987625208566775 => {
                            sg = session_group_contains(s);
                            if sg.is_null() || session_group_count(sg) <= 1 as u_int {
                                current_block_4 = 16668937799742929182;
                            } else {
                                current_block_4 = 13109137661213826276;
                            }
                        }
                        _ => {}
                    }
                    match current_block_4 {
                        16668937799742929182 => {}
                        _ => {
                            server_destroy_session(s);
                            session_destroy(
                                s,
                                1 as ::core::ffi::c_int,
                                b"server_check_unattached\0" as *const u8
                                    as *const ::core::ffi::c_char,
                            );
                        }
                    }
                }
                1 | _ => {
                    current_block_4 = 13109137661213826276;
                    match current_block_4 {
                        11000743977270914936 => {
                            sg = session_group_contains(s);
                            if !sg.is_null() && session_group_count(sg) == 1 as u_int {
                                current_block_4 = 16668937799742929182;
                            } else {
                                current_block_4 = 13109137661213826276;
                            }
                        }
                        6116987625208566775 => {
                            sg = session_group_contains(s);
                            if sg.is_null() || session_group_count(sg) <= 1 as u_int {
                                current_block_4 = 16668937799742929182;
                            } else {
                                current_block_4 = 13109137661213826276;
                            }
                        }
                        _ => {}
                    }
                    match current_block_4 {
                        16668937799742929182 => {}
                        _ => {
                            server_destroy_session(s);
                            session_destroy(
                                s,
                                1 as ::core::ffi::c_int,
                                b"server_check_unattached\0" as *const u8
                                    as *const ::core::ffi::c_char,
                            );
                        }
                    }
                }
            }
        }
        s_owner = sessions_after(&*std::ptr::addr_of!(sessions), &name);
        s = s_owner.as_ref().map_or(std::ptr::null_mut(), crate::src::shared::rc::as_ptr);
    }
}
pub unsafe fn server_unzoom_window(mut w: *mut window) {
    if window_unzoom(w, 1 as ::core::ffi::c_int) == 0 as ::core::ffi::c_int {
        server_redraw_window(w);
    }
}
