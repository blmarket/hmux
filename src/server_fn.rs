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
use crate::src::options::options_owner_ptr;
use crate::src::options::{options_get_number, options_get_string};
use crate::src::proc::proc_send;
use crate::src::reactor::BufferEvent;
use crate::src::resize::recalculate_sizes;
use crate::src::screen_write::{
    screen_write_cursormove, screen_write_linefeed, screen_write_scrollregion,
    screen_write_start_pane, screen_write_stop,
};
use crate::src::server::clients;
use crate::src::server::marked_pane;
use crate::src::server_client::Client as _;
use crate::src::server_client::{server_client_remove_pane, Client};
use crate::src::session::sessions;
use crate::src::session::Session;
use crate::src::session::{
    session_attach, session_destroy, session_detach, session_group_count, session_next_session,
    session_previous_session, session_renumber_windows, session_select, sessions_after,
    sessions_minmax,
};
use crate::src::shared::client::ClientRef;
use crate::src::shared::events::event_payload;
use crate::src::shared::session::session_group;
use crate::src::tmux::sig2name;
use crate::src::tty::{tty_raw, tty_stop_tty};
use crate::src::tty_term::tty_term_owner_ptr;
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

unsafe fn server_fire_pane_exit(
    mut name: *const ::core::ffi::c_char,
    wp_owner: &std::rc::Rc<std::cell::UnsafeCell<window_pane>>,
) {
    let mut wp = wp_owner.get();
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
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
    cmd_find_from_pane(&raw mut fs, wp_owner, 0 as ::core::ffi::c_int);
    event_payload_set_target(&mut *ep, &fs);
    event_payload_set_pane(
        &mut *ep,
        b"pane\0" as *const u8 as *const ::core::ffi::c_char,
        (*(wp)).observer.upgrade().expect("live window_pane"),
    );
    event_payload_set_window(
        &mut *ep,
        b"window\0" as *const u8 as *const ::core::ffi::c_char,
        (*((*wp)
            .window_handle()
            .as_ref()
            .map_or(std::ptr::null_mut(), |owner| owner.get())))
        .observer
        .upgrade()
        .expect("live window"),
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
pub unsafe fn server_redraw_client(c: &ClientRef) {
    c.request_redraw(CLIENT_ALLREDRAWFLAGS as u64);
}
pub unsafe fn server_status_client(c: &ClientRef) {
    c.request_redraw(CLIENT_REDRAWSTATUS as u64);
}
pub unsafe fn server_redraw_session(session: &std::rc::Rc<std::cell::UnsafeCell<session>>) {
    let mut next = clients.first();
    while let Some(client_owner) = next {
        next = clients.next(&client_owner);
        let client = &client_owner;
        if client
            .attached_session()
            .ptr_eq(&std::rc::Rc::downgrade(session))
        {
            server_redraw_client(client);
        }
    }
}
pub unsafe fn server_redraw_session_group(session: &std::rc::Rc<std::cell::UnsafeCell<session>>) {
    let group = crate::src::session::session_group_for(&std::rc::Rc::downgrade(session));
    if group.is_null() {
        server_redraw_session(session);
    } else {
        for owner in crate::src::session::session_group_members(group) {
            server_redraw_session(&owner);
        }
    }
}

pub unsafe fn server_status_session(session: &std::rc::Rc<std::cell::UnsafeCell<session>>) {
    let mut next = clients.first();
    while let Some(client_owner) = next {
        next = clients.next(&client_owner);
        let client = &client_owner;
        if client
            .attached_session()
            .ptr_eq(&std::rc::Rc::downgrade(session))
        {
            client.request_redraw(CLIENT_REDRAWSTATUS as u64);
        }
    }
}
pub unsafe fn server_status_session_group(session: &std::rc::Rc<std::cell::UnsafeCell<session>>) {
    let group = crate::src::session::session_group_for(&std::rc::Rc::downgrade(session));
    if group.is_null() {
        server_status_session(session);
    } else {
        for owner in crate::src::session::session_group_members(group) {
            server_status_session(&owner);
        }
    }
}

pub unsafe fn server_redraw_window(window: &window) {
    let mut next = clients.first();
    while let Some(client_owner) = next {
        next = clients.next(&client_owner);
        let client = &client_owner;
        let matches = client
            .attached_session()
            .upgrade()
            .and_then(|session| {
                session
                    .current_winlink()
                    .try_borrow_mut()
                    .ok()
                    .and_then(|link| link.window_owner.clone())
            })
            .is_some_and(|current| {
                std::rc::Weak::ptr_eq(&window.observer, &std::rc::Rc::downgrade(&current))
            });
        if matches {
            server_redraw_client(client);
        }
    }
}
pub unsafe fn server_redraw_window_menu(window_owner: &std::rc::Rc<std::cell::UnsafeCell<window>>) {
    let mut next = clients.first();
    while let Some(client_owner) = next {
        next = clients.next(&client_owner);
        let client = &client_owner;
        let matches = client
            .attached_session()
            .upgrade()
            .and_then(|session| {
                session
                    .current_winlink()
                    .try_borrow_mut()
                    .ok()
                    .and_then(|link| link.window_owner.clone())
            })
            .is_some_and(|current| std::rc::Rc::ptr_eq(&current, window_owner));
        if matches {
            client.request_redraw(CLIENT_REDRAWMENU as u64);
        }
    }
}
pub unsafe fn server_redraw_window_borders(window: &window) {
    let mut next = clients.first();
    while let Some(client_owner) = next {
        next = clients.next(&client_owner);
        let client = &client_owner;
        let matches = client
            .attached_session()
            .upgrade()
            .and_then(|session| {
                session
                    .current_winlink()
                    .try_borrow_mut()
                    .ok()
                    .and_then(|link| link.window_owner.clone())
            })
            .is_some_and(|current| {
                std::rc::Weak::ptr_eq(&window.observer, &std::rc::Rc::downgrade(&current))
            });
        if matches {
            client.request_redraw(CLIENT_REDRAWBORDERS as u64);
        }
    }
}
pub unsafe fn server_status_window(window: &window) {
    let mut next = sessions_minmax(&sessions);
    while let Some(session_owner) = next {
        if session_owner.contains_window(&window.observer.upgrade().expect("live window")) {
            server_status_session(&session_owner);
        }
        next = session_owner.next_session();
    }
}
pub unsafe fn server_lock() {
    let mut next = clients.first();
    while let Some(owner) = next {
        if !owner.attached_session().upgrade().is_none() {
            server_lock_client(&owner);
        }
        next = clients.next(&owner);
    }
}
pub unsafe fn server_lock_session(session_owner: &std::rc::Rc<std::cell::UnsafeCell<session>>) {
    let observer = std::rc::Rc::downgrade(session_owner);
    let mut next = clients.first();
    while let Some(owner) = next {
        let matches = owner.attached_session().ptr_eq(&observer);
        if matches {
            server_lock_client(&owner);
        }
        next = clients.next(&owner);
    }
}
pub unsafe fn server_lock_client(client_owner: &ClientRef) {
    if client_owner.flags() & (CLIENT_CONTROL | CLIENT_SUSPENDED) as u64 != 0 {
        return;
    }
    let command = client_owner
        .attached_session()
        .upgrade()
        .expect("live session")
        .with_options_mut(|options| {
            std::ffi::CStr::from_ptr(options_get_string(options, c"lock-command".as_ptr()))
                .to_owned()
        });
    client_owner.lock(&command);
}

pub unsafe fn server_kill_pane(pane_owner: &std::rc::Rc<std::cell::UnsafeCell<window_pane>>) {
    let wp = pane_owner.get();
    let mut w: *mut window = (*wp)
        .window_handle()
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    if window_count_panes(&*w, 1 as ::core::ffi::c_int) == 1 as u_int {
        server_kill_window((*w).observer.upgrade().expect("live pane window"), 1);
        recalculate_sizes();
    } else {
        window_push_zoom(
            &(*(w)).observer.upgrade().expect("live window"),
            0 as ::core::ffi::c_int,
            (*wp).flags & PANE_FLOATOVERZOOM,
        );
        server_client_remove_pane(&(*(wp)).observer.upgrade().expect("live window_pane"));
        layout_close_pane(&(*(wp)).observer.upgrade().expect("live window_pane"));
        window_remove_pane(&(*(w)).observer.upgrade().expect("live window"), pane_owner);
        window_pop_zoom(&(*(w)).observer.upgrade().expect("live window"));
        server_redraw_window(&*(w));
    };
}
pub unsafe fn server_kill_window(
    owner: std::rc::Rc<std::cell::UnsafeCell<window>>,
    mut renumber: ::core::ffi::c_int,
) {
    let w = owner.get();
    let mut s: Option<std::rc::Rc<std::cell::UnsafeCell<session>>> = None;
    let mut wl: refbox::Weak<winlink> = refbox::Weak::new();
    let mut s_owner = sessions_minmax(&sessions);
    s = s_owner.clone();
    while !s.is_none() {
        // Destroying a group may remove both s and its next session.
        let name = s.as_ref().expect("live session").name().into_bytes();
        if !(!s.as_ref().expect("live session").contains_window(&owner)) {
            server_unzoom_window(&(*(w)).observer.upgrade().expect("live window"));
            loop {
                wl = s.as_ref().expect("live session").with_winlinks(|links| {
                    winlink_find_by_window(links, &(*(w)).observer.upgrade().expect("live window"))
                });
                if !wl.is_alive() {
                    break;
                }
                if session_detach(s.as_ref().expect("live session"), wl.clone()) != 0 {
                    server_destroy_session_group(s.as_ref().expect("live session"));
                    break;
                } else {
                    server_redraw_session_group(s.as_ref().expect("live session"));
                }
            }
            if renumber != 0
                && (s.as_ref().is_some_and(|session| session.is_registered()) as i32) != 0
            {
                server_renumber_session(s.as_ref().expect("live session"));
            }
        }
        s_owner = sessions_after(&sessions, &name);
        s = s_owner.clone();
    }
    recalculate_sizes();
    window_remove_ref(
        owner,
        b"server_kill_window\0" as *const u8 as *const ::core::ffi::c_char,
    );
}
pub unsafe fn server_renumber_session(s_owner: &std::rc::Rc<std::cell::UnsafeCell<session>>) {
    let s = Some(s_owner.clone());
    let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
    if s.as_ref()
        .expect("live session")
        .with_options_mut(|options| {
            options_get_number(
                options,
                b"renumber-windows\0" as *const u8 as *const ::core::ffi::c_char,
            )
        })
        != 0
    {
        sg = crate::src::session::session_group_for(
            &s.as_ref()
                .map_or_else(std::rc::Weak::new, std::rc::Rc::downgrade),
        );
        if !sg.is_null() {
            for session_owner in crate::src::session::session_group_members(sg) {
                session_renumber_windows(&session_owner);
            }
        } else {
            session_renumber_windows(s.as_ref().expect("live session"));
        }
    }
}
pub unsafe fn server_renumber_all() {
    let mut s: Option<std::rc::Rc<std::cell::UnsafeCell<session>>> = None;
    let mut s_owner = sessions_minmax(&sessions);
    s = s_owner.clone();
    while !s.is_none() {
        server_renumber_session(s.as_ref().expect("live session"));
        s_owner = s.as_ref().expect("live session").next_session();
        s = s_owner.clone();
    }
}
pub unsafe fn server_link_window(
    src_owner: &std::rc::Rc<std::cell::UnsafeCell<session>>,
    mut srcwl: refbox::Weak<winlink>,
    dst_owner: &std::rc::Rc<std::cell::UnsafeCell<session>>,
    mut dstidx: ::core::ffi::c_int,
    mut killflag: ::core::ffi::c_int,
    mut selectflag: ::core::ffi::c_int,
) -> Result<(), std::ffi::CString> {
    let src = Some(src_owner.clone());
    let dst = Some(dst_owner.clone());
    let mut dstwl: refbox::Weak<winlink> = refbox::Weak::new();
    let mut srcsg: *mut session_group = ::core::ptr::null_mut::<session_group>();
    let mut dstsg: *mut session_group = ::core::ptr::null_mut::<session_group>();
    srcsg = crate::src::session::session_group_for(
        &src.as_ref()
            .map_or_else(std::rc::Weak::new, std::rc::Rc::downgrade),
    );
    dstsg = crate::src::session::session_group_for(
        &dst.as_ref()
            .map_or_else(std::rc::Weak::new, std::rc::Rc::downgrade),
    );
    if !std::rc::Rc::ptr_eq(src_owner, dst_owner)
        && !srcsg.is_null()
        && !dstsg.is_null()
        && srcsg == dstsg
    {
        return Err(c"sessions are grouped".to_owned());
    }
    dstwl = refbox::Weak::new();
    if dstidx != -(1 as ::core::ffi::c_int) {
        dstwl = dst_owner.with_winlinks(|links| winlink_find_by_index(links, dstidx));
    }
    if dstwl.is_alive() {
        if dstwl
            .get_unchecked()
            .window_handle()
            .as_ref()
            .map_or(std::ptr::null_mut(), |owner| owner.get())
            == srcwl
                .get_unchecked()
                .window_handle()
                .as_ref()
                .map_or(std::ptr::null_mut(), |owner| owner.get())
        {
            return Err(std::ffi::CString::new(format!("same index: {dstidx}"))
                .expect("numeric diagnostic contains no NUL"));
        }
        if killflag != 0 {
            if dst_owner.remove_replaced_window(dstwl.clone()) {
                selectflag = 1;
            }
        }
    }
    if dstidx == -(1 as ::core::ffi::c_int) {
        dstidx = (-(1 as ::core::ffi::c_int) as ::core::ffi::c_longlong
            - dst
                .as_ref()
                .expect("live session")
                .with_options_mut(|options| {
                    options_get_number(
                        options,
                        b"base-index\0" as *const u8 as *const ::core::ffi::c_char,
                    )
                })) as ::core::ffi::c_int;
    }
    let window_owner = srcwl
        .get_unchecked()
        .window_owner
        .as_ref()
        .expect("source window")
        .clone();
    let attached = session_attach(dst.as_ref().expect("live session"), &window_owner, dstidx);
    window_remove_ref(window_owner, c"server_link_window".as_ptr());
    dstwl = attached?;
    if marked_pane.winlink_handle() == srcwl {
        marked_pane.set_wl((dstwl).clone());
    }
    if selectflag != 0 {
        session_select(
            dst.as_ref().expect("live session"),
            dstwl.get_unchecked().idx,
        );
    }
    server_redraw_session_group(dst.as_ref().expect("live session"));
    Ok(())
}
pub unsafe fn server_unlink_window(
    s_owner: &std::rc::Rc<std::cell::UnsafeCell<session>>,
    mut wl: refbox::Weak<winlink>,
) {
    let s = Some(s_owner.clone());
    if session_detach(s.as_ref().expect("live session"), wl.clone()) != 0 {
        server_destroy_session_group(s_owner);
    } else {
        server_redraw_session_group(s.as_ref().expect("live session"));
    };
}
pub unsafe fn server_destroy_pane(
    pane_owner: &std::rc::Rc<std::cell::UnsafeCell<window_pane>>,
    mut notify: ::core::ffi::c_int,
) {
    let wp = pane_owner.get();
    let mut w: *mut window = (*wp)
        .window_handle()
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    let mut ctx: screen_write_ctx = screen_write_ctx {
        wp: std::rc::Weak::new(),
        target: Default::default(),
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
        std::mem::take(&mut (*wp).event).free();
        close((*wp).fd);
        (*wp).fd = -(1 as ::core::ffi::c_int);
    }
    if (*wp).pipe_fd != -(1 as ::core::ffi::c_int) {
        std::mem::take(&mut (*wp).pipe_event).free();
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
                    &(*(wp)).observer.upgrade().expect("live window_pane"),
                );
            }
            s = options_get_string(
                options_owner_ptr(&mut (*wp).options)
                    .map_or(std::ptr::null_mut(), |options| options),
                b"remain-on-exit-format\0" as *const u8 as *const ::core::ffi::c_char,
            );
            if *s as ::core::ffi::c_int != '\0' as i32 {
                screen_write_start_pane(
                    &mut ctx,
                    &(*wp).observer.upgrade().expect("live screen-write pane"),
                    &raw mut (*wp).base,
                );
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
                    None,
                    s,
                    None,
                    None,
                    (refbox::Weak::new()).clone(),
                    (wp).as_ref()
                        .and_then(|model| model.observer.upgrade())
                        .as_ref(),
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
            &(*(wp)).observer.upgrade().expect("live window_pane"),
        );
    }
    window_push_zoom(
        &(*(w)).observer.upgrade().expect("live window"),
        0 as ::core::ffi::c_int,
        (*wp).flags & PANE_FLOATOVERZOOM,
    );
    server_client_remove_pane(&(*(wp)).observer.upgrade().expect("live window_pane"));
    layout_close_pane(&(*(wp)).observer.upgrade().expect("live window_pane"));
    window_remove_pane(&(*(w)).observer.upgrade().expect("live window"), pane_owner);
    if window_pane_first(w.as_ref()).is_none() {
        server_kill_window((*w).observer.upgrade().expect("live pane window"), 1);
    } else {
        window_pop_zoom(&(*(w)).observer.upgrade().expect("live window"));
        server_redraw_window(&*(w));
    };
}
unsafe fn server_destroy_session_group(s_owner: &std::rc::Rc<std::cell::UnsafeCell<session>>) {
    let s = Some(s_owner.clone());
    let source = s_owner.clone();
    let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
    sg = crate::src::session::session_group_for(
        &s.as_ref()
            .map_or_else(std::rc::Weak::new, std::rc::Rc::downgrade),
    );
    if sg.is_null() {
        server_destroy_session(&source);
        session_destroy(
            &source,
            1 as ::core::ffi::c_int,
            b"server_destroy_session_group\0" as *const u8 as *const ::core::ffi::c_char,
        );
    } else {
        for session_owner in crate::src::session::session_group_members(sg) {
            server_destroy_session(&session_owner);
            session_destroy(
                &session_owner,
                1 as ::core::ffi::c_int,
                b"server_destroy_session_group\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
    };
}
unsafe fn server_find_session(
    head: &crate::src::shared::session::sessions,
    excluded: &std::rc::Rc<std::cell::UnsafeCell<session>>,
    mut choose: impl FnMut(
        &std::rc::Rc<std::cell::UnsafeCell<session>>,
        Option<&std::rc::Rc<std::cell::UnsafeCell<session>>>,
    ) -> bool,
) -> Option<std::rc::Rc<std::cell::UnsafeCell<session>>> {
    let mut selected: Option<std::rc::Rc<std::cell::UnsafeCell<session>>> = None;
    let mut current = sessions_minmax(head);
    while let Some(owner) = current {
        current = owner.next_session();
        if !std::rc::Rc::ptr_eq(&owner, excluded) && choose(&owner, selected.as_ref()) {
            selected = Some(owner);
        }
    }
    selected
}
unsafe fn server_newer_session(
    s: &std::rc::Rc<std::cell::UnsafeCell<session>>,
    other: Option<&std::rc::Rc<std::cell::UnsafeCell<session>>>,
) -> bool {
    let Some(other) = other else { return true };
    let (a, b) = (s.activity_time(), other.activity_time());
    (a.tv_sec, a.tv_usec) > (b.tv_sec, b.tv_usec)
}
unsafe fn server_newer_detached_session(
    s: &std::rc::Rc<std::cell::UnsafeCell<session>>,
    other: Option<&std::rc::Rc<std::cell::UnsafeCell<session>>>,
) -> bool {
    !s.is_attached() && server_newer_session(s, other)
}
pub unsafe fn server_destroy_session(source: &std::rc::Rc<std::cell::UnsafeCell<session>>) {
    let s = Some(source.clone());
    let mut c: Option<ClientRef> = None;
    let mut sort_crit: sort_criteria = sort_criteria {
        order: SORT_NAME,
        reversed: 0,
        order_seq: &[],
    };
    let mut detach_on_destroy: ::core::ffi::c_int = 0;
    detach_on_destroy = s
        .as_ref()
        .expect("live session")
        .with_options_mut(|options| {
            options_get_number(
                options,
                b"detach-on-destroy\0" as *const u8 as *const ::core::ffi::c_char,
            )
        }) as ::core::ffi::c_int;
    let head = &sessions;
    let mut replacement = match detach_on_destroy {
        0 => server_find_session(head, s.as_ref().expect("live session"), |a, b| {
            server_newer_session(a, b)
        }),
        2 => server_find_session(head, s.as_ref().expect("live session"), |a, b| {
            server_newer_detached_session(a, b)
        }),
        3 => session_previous_session(Some(s.as_ref().expect("live session")), &sort_crit),
        4 => session_next_session(Some(s.as_ref().expect("live session")), &sort_crit),
        _ => None,
    };
    if replacement
        .as_ref()
        .is_some_and(|owner| std::rc::Rc::ptr_eq(owner, source))
    {
        replacement = None;
    }
    let fallback = if replacement.is_none() && matches!(detach_on_destroy, 1 | 2) {
        server_find_session(head, s.as_ref().expect("live session"), |a, b| {
            server_newer_session(a, b)
        })
    } else {
        None
    };
    let mut registry_c_owner = clients.first();
    c = registry_c_owner.clone();
    while !c.is_none() {
        if !(!crate::src::shared::rc::same(
            c.as_ref()
                .expect("live client")
                .attached_session()
                .upgrade()
                .as_ref(),
            s.as_ref(),
        )) {
            let target = replacement.as_ref().or_else(|| {
                (c.as_ref().expect("live client").flags() & CLIENT_NO_DETACH_ON_DESTROY != 0)
                    .then_some(fallback.as_ref())
                    .flatten()
            });
            registry_c_owner
                .as_ref()
                .expect("current registry client")
                .reattach_after_session_destroy(target);
        }
        registry_c_owner =
            clients.next(registry_c_owner.as_ref().expect("current registry client"));
        c = registry_c_owner.clone();
    }
    recalculate_sizes();
}
pub unsafe fn server_check_unattached() {
    let mut s: Option<std::rc::Rc<std::cell::UnsafeCell<session>>> = None;
    let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
    let mut current_block_4: u64;
    let mut s_owner = sessions_minmax(&sessions);
    s = s_owner.clone();
    while !s.is_none() {
        let name = s.as_ref().expect("live session").name().into_bytes();
        if !(s.as_ref().expect("live session").is_attached()) {
            match s
                .as_ref()
                .expect("live session")
                .with_options_mut(|options| {
                    options_get_number(
                        options,
                        b"destroy-unattached\0" as *const u8 as *const ::core::ffi::c_char,
                    )
                }) {
                0 => {}
                2 => {
                    current_block_4 = 6116987625208566775;
                    match current_block_4 {
                        11000743977270914936 => {
                            sg = crate::src::session::session_group_for(
                                &s.as_ref()
                                    .map_or_else(std::rc::Weak::new, std::rc::Rc::downgrade),
                            );
                            if !sg.is_null() && session_group_count(sg) == 1 as u_int {
                                current_block_4 = 16668937799742929182;
                            } else {
                                current_block_4 = 13109137661213826276;
                            }
                        }
                        6116987625208566775 => {
                            sg = crate::src::session::session_group_for(
                                &s.as_ref()
                                    .map_or_else(std::rc::Weak::new, std::rc::Rc::downgrade),
                            );
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
                            server_destroy_session(s_owner.as_ref().expect("registered session"));
                            session_destroy(
                                s_owner.as_ref().expect("registered session"),
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
                            sg = crate::src::session::session_group_for(
                                &s.as_ref()
                                    .map_or_else(std::rc::Weak::new, std::rc::Rc::downgrade),
                            );
                            if !sg.is_null() && session_group_count(sg) == 1 as u_int {
                                current_block_4 = 16668937799742929182;
                            } else {
                                current_block_4 = 13109137661213826276;
                            }
                        }
                        6116987625208566775 => {
                            sg = crate::src::session::session_group_for(
                                &s.as_ref()
                                    .map_or_else(std::rc::Weak::new, std::rc::Rc::downgrade),
                            );
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
                            server_destroy_session(s_owner.as_ref().expect("registered session"));
                            session_destroy(
                                s_owner.as_ref().expect("registered session"),
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
                            sg = crate::src::session::session_group_for(
                                &s.as_ref()
                                    .map_or_else(std::rc::Weak::new, std::rc::Rc::downgrade),
                            );
                            if !sg.is_null() && session_group_count(sg) == 1 as u_int {
                                current_block_4 = 16668937799742929182;
                            } else {
                                current_block_4 = 13109137661213826276;
                            }
                        }
                        6116987625208566775 => {
                            sg = crate::src::session::session_group_for(
                                &s.as_ref()
                                    .map_or_else(std::rc::Weak::new, std::rc::Rc::downgrade),
                            );
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
                            server_destroy_session(s_owner.as_ref().expect("registered session"));
                            session_destroy(
                                s_owner.as_ref().expect("registered session"),
                                1 as ::core::ffi::c_int,
                                b"server_check_unattached\0" as *const u8
                                    as *const ::core::ffi::c_char,
                            );
                        }
                    }
                }
            }
        }
        s_owner = sessions_after(&sessions, &name);
        s = s_owner.clone();
    }
}
pub unsafe fn server_unzoom_window(w_owner: &std::rc::Rc<std::cell::UnsafeCell<window>>) {
    let mut w = w_owner.get();
    if window_unzoom(w_owner, 1 as ::core::ffi::c_int) == 0 as ::core::ffi::c_int {
        server_redraw_window(&*(w));
    }
}

#[cfg(test)]
mod session_selection_tests {
    use super::*;
    use crate::src::session::{sessions_insert, sessions_remove};
    use std::rc::Rc;

    #[test]
    fn replacement_selection_retains_winner_and_excludes_source() {
        unsafe {
            let mut head = crate::src::shared::session::sessions { storage: None };
            let source = session::new();
            let detached = session::new();
            let attached = session::new();
            for (owner, name, activity) in [
                (&source, c"source", 30),
                (&detached, c"detached", 10),
                (&attached, c"attached", 20),
            ] {
                crate::src::session::test_support::metadata(
                    &owner,
                    Some(name.to_owned()),
                    None,
                    None,
                );
                crate::src::session::test_support::activity(
                    owner,
                    timeval {
                        tv_sec: activity,
                        tv_usec: 0,
                    },
                );
                sessions_insert(&mut head, owner.clone());
            }
            crate::src::session::test_support::metadata(&attached, None, None, Some(1));
            let selected =
                server_find_session(&head, &source, |a, b| server_newer_session(a, b)).unwrap();
            assert!(Rc::ptr_eq(&selected, &attached));
            let detached_selected =
                server_find_session(&head, &source, |a, b| server_newer_detached_session(a, b))
                    .unwrap();
            assert!(Rc::ptr_eq(&detached_selected, &detached));
            let observer = Rc::downgrade(&attached);
            sessions_remove(&mut head, &attached);
            drop(attached);
            assert!(observer.upgrade().is_some());
            assert_eq!(selected.name().as_c_str(), c"attached");
            drop(selected);
            assert!(observer.upgrade().is_none());
            sessions_remove(&mut head, &detached);
            assert!(
                server_find_session(&head, &source, |a, b| server_newer_session(a, b)).is_none()
            );
            sessions_remove(&mut head, &source);
        }
    }
}
