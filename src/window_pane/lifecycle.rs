//! Pane process exit and explicit teardown transactions.
use crate::src::cmd::find::cmd_find_from_pane;
use crate::src::events::{events_fire, events_fire_winlink};
use crate::src::events_payload::{
    event_payload_create, event_payload_set_int, event_payload_set_pane, event_payload_set_string,
    event_payload_set_target, event_payload_set_window,
};
use crate::src::ffi::libc::{close, getpid, kill, memcpy, strlen};
use crate::src::ffi::utempter::utempter_remove_record;
use crate::src::format::bytes::write_cstr;
use crate::src::format::format_single_cstring;
use crate::src::format_draw::format_draw;
use crate::src::grid::grid_default_cell;
use crate::src::layout::layout_close_pane;
use crate::src::log::log_debug;
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
use crate::src::server_client::{Client};
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
use crate::src::shared::session::SessionRef;
use crate::src::shared::window::WindowRef;
use crate::src::tmux::sig2name;
use crate::src::tty::{tty_raw, tty_stop_tty};
use crate::src::tty_term::tty_term_owner_ptr;
use crate::src::tty_term::tty_term_string;
use crate::src::window::Window as _;
use crate::src::window::{
    window_add_ref, window_pop_zoom, window_push_zoom, window_remove_pane, window_remove_ref,
    window_unzoom, winlink_find_by_index, winlink_find_by_window, winlink_remove,
    winlink_stack_remove,
};
use crate::src::window_pane::WindowPane as _;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

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
use crate::src::shared::window::winlink;
use crate::src::shared::window::WINLINK_ALERTFLAGS;

use crate::src::server_fn::{server_kill_window, server_redraw_window};
use crate::src::shared::pane::PANE_EXITED;
use crate::src::spawn::spawn_editor_finish;
use crate::src::window::window_pane_wait_finish;
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
        std::rc::Rc::clone(&(((*wp).window_handle().as_ref()).expect("live window"))),
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
pub(super) unsafe fn kill_process(pane_owner: &std::rc::Rc<std::cell::UnsafeCell<window_pane>>) {
    let wp = pane_owner.get();
    let window_owner = pane_owner
        .window_observer()
        .upgrade()
        .expect("live pane window");
    if window_owner.pane_snapshot().len() == 1 {
        server_kill_window(window_owner, 1);
        recalculate_sizes();
    } else {
        window_owner.release(c"server_kill_pane");
        window_push_zoom(
            &std::rc::Rc::clone(&(((*wp).window_handle().as_ref()).expect("live window"))),
            0 as ::core::ffi::c_int,
            (*wp).flags & PANE_FLOATOVERZOOM,
        );
        ClientRef::forget_pane(&(*(wp)).observer.upgrade().expect("live window_pane"));
        layout_close_pane(&(*(wp)).observer.upgrade().expect("live window_pane"));
        window_remove_pane(
            &std::rc::Rc::clone(&(((*wp).window_handle().as_ref()).expect("live window"))),
            pane_owner,
        );
        window_pop_zoom(&std::rc::Rc::clone(
            &(((*wp).window_handle().as_ref()).expect("live window")),
        ));
        server_redraw_window(&(((*wp).window_handle().as_ref()).expect("live window")));
    };
}
pub(super) unsafe fn finish_process(
    pane_owner: &std::rc::Rc<std::cell::UnsafeCell<window_pane>>,
    mut notify: ::core::ffi::c_int,
) {
    let wp = pane_owner.get();
    // Keep the original Window identity across exit notifications and pane teardown.
    let window_owner = pane_owner
        .window_observer()
        .upgrade()
        .expect("live pane window");
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
        window_owner.release(c"server_destroy_pane");
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
                window_owner.release(c"server_destroy_pane");
                return;
            }
            (*wp).flags |= PANE_STATUSDRAWN;
            (*wp).dead_time = SystemTime::now();
            if notify != 0 {
                server_fire_pane_exit(
                    b"pane-died\0" as *const u8 as *const ::core::ffi::c_char,
                    &(*(wp)).observer.upgrade().expect("live window_pane"),
                );
            }
            let format = (*wp)
                .observer
                .upgrade()
                .expect("live pane")
                .with_options_mut(|options| {
                    options_get_string(options, c"remain-on-exit-format".as_ptr())
                });
            s = format.as_ptr();
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
            window_owner.release(c"server_destroy_pane");
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
        &std::rc::Rc::clone(&(((*wp).window_handle().as_ref()).expect("live window"))),
        0 as ::core::ffi::c_int,
        (*wp).flags & PANE_FLOATOVERZOOM,
    );
    ClientRef::forget_pane(&(*(wp)).observer.upgrade().expect("live window_pane"));
    layout_close_pane(&(*(wp)).observer.upgrade().expect("live window_pane"));
    window_remove_pane(
        &std::rc::Rc::clone(&(((*wp).window_handle().as_ref()).expect("live window"))),
        pane_owner,
    );
    if window_owner.next_pane(None).is_none() {
        server_kill_window(
            std::rc::Rc::downgrade(&(((*wp).window_handle().as_ref()).expect("live window")))
                .upgrade()
                .expect("live pane window"),
            1,
        );
    } else {
        window_pop_zoom(&std::rc::Rc::clone(
            &(((*wp).window_handle().as_ref()).expect("live window")),
        ));
        server_redraw_window(&(((*wp).window_handle().as_ref()).expect("live window")));
    };
    window_owner.release(c"server_destroy_pane");
}
pub(super) unsafe fn process_exited(
    pane: &std::rc::Rc<std::cell::UnsafeCell<window_pane>>,
    pid: pid_t,
    status: i32,
) -> bool {
    {
        let state = &mut *pane.get();
        if state.pid != pid {
            return false;
        }
        state.status = status;
        state.flags |= PANE_STATUSREADY;
        log_debug(format_args!("%{} exited", state.id));
        state.flags |= PANE_EXITED;
    }
    window_pane_wait_finish(pane);
    spawn_editor_finish(pane);
    if pane.destroy_ready() {
        finish_process(pane, 1);
    }
    true
}
