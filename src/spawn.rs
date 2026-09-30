use crate::src::cmd::find::cmd_find_from_winlink_pane;
use crate::src::cmd::queue::{cmdq_get_client, cmdq_get_target};
use crate::src::cmd::{cmd_log_argv, cmd_stringify_argv_cstring};
use crate::src::compat::fdforkpty::fdforkpty;
use crate::src::compat::stdio::CFile;
use crate::src::compat::systemd::systemd_move_to_new_cgroup;
use crate::src::control::control_reset_pane;
use crate::src::environ::{
    environ_copy, environ_create, environ_find, environ_for_session, environ_log, environ_push,
    environ_set,
};
use crate::src::events::{events_fire, events_fire_window, events_fire_winlink};
use crate::src::events_payload::{
    event_payload_create, event_payload_set_int, event_payload_set_pane, event_payload_set_session,
    event_payload_set_string, event_payload_set_target, event_payload_set_window,
};
use crate::src::ffi::libc::{
    __errno_location, _exit, chdir, close, closefrom, execl, execvp, fdopen, fopen, fread, fseeko,
    ftello, fwrite, getcwd, getpid, kill, memcpy, memset, mkstemp, sigfillset, sigprocmask,
    strerror, strrchr, tcgetattr, tcsetattr, unlink,
};
use crate::src::ffi::utempter::utempter_add_record;
use crate::src::format::bytes::write_cstr;
use crate::src::format::bytes::xformat;
use crate::src::format::format_single_cstring;
use crate::src::input::input_free;
use crate::src::layout::{
    layout_assign_pane, layout_close_pane, layout_floating_pane, layout_free, layout_init,
};
use crate::src::log::{log_close, log_cstr, log_debug, log_hex};
use crate::src::names::default_window_name_cstring;
use crate::src::options::options_owner_ptr;
use crate::src::options::{options_get_number, options_get_string, options_set_number};
use crate::src::proc::proc_clear_signals;
use crate::src::reactor::BufferEvent;
use crate::src::resize::default_window_size;
use crate::src::screen::screen_reinit;
use crate::src::server::clients;
use crate::src::server::server_proc;
use crate::src::server_client::Client as _;
use crate::src::server_client::Client;
use crate::src::server_client::{server_client_get_cwd, server_client_remove_pane};
use crate::src::session::Session;
use crate::src::shared::client::ClientRef;
use crate::src::shared::events::event_payload;
use crate::src::shared::spawn::spawn_context;
use crate::src::shared::window::WindowRef;
use crate::src::tmux::{checkshell, find_home_cstr, global_options, ptm_fd};
use crate::src::window::window_pane_resize;
use crate::src::window::Window as _;
use crate::src::window::{
    window_add_pane, window_create, window_destroy_panes, window_pane_index, window_pane_next,
    window_pane_reset_mode_all, window_pane_set_cwd, window_pane_set_event, window_pane_set_shell,
    window_pop_zoom, window_push_zoom, window_redraw_active_switch, window_remove_pane,
    window_set_active_pane, winlink_add, winlink_find_by_index, winlink_remove, winlink_set_window,
    winlink_stack_remove,
};
use crate::src::window_border::window_set_fill_cells;
use crate::src::window_pane::WindowPane as _;
use std::ffi::{CStr, CString};

pub(crate) fn set_spawn_cause(cause: Option<&mut Option<CString>>, parts: &[&[u8]]) {
    let Some(cause) = cause else {
        return;
    };
    let mut bytes = Vec::with_capacity(parts.iter().map(|part| part.len()).sum());
    for part in parts {
        bytes.extend_from_slice(part);
    }
    *cause = Some(CString::new(bytes).expect("spawn diagnostic contains no NUL"));
}

use crate::src::shared::abi::__off_t;
use crate::src::shared::abi::*;
use crate::src::shared::client::client;
use crate::src::shared::client::CLIENT_CONTROL;
use crate::src::shared::command::{cmd_find_state, cmdq_item};
use crate::src::shared::environment::{environ, environ_entry};
use crate::src::shared::event::*;
use crate::src::shared::input::input_ctx;
use crate::src::shared::key::*;
use crate::src::shared::layout::layout_geometry;
use crate::src::shared::layout::*;
use crate::src::shared::limits::SIZE_MAX;
use crate::src::shared::pane::window_pane;
use crate::src::shared::pane::{
    PANE_EMPTY, PANE_EXITED, PANE_FLOATOVERZOOM, PANE_STATUSDRAWN, PANE_STATUSREADY,
};
use crate::src::shared::posix_io::{_PATH_BSHELL, STDERR_FILENO, STDIN_FILENO};
use crate::src::shared::posix_terminal::{winsize, TCSANOW, VERASE};
use crate::src::shared::screen::{MODE_CRLF, MODE_CURSOR};
use crate::src::shared::session::session;
use crate::src::shared::signal::{__sigset_t, sigset_t, SIGCHLD, SIGHUP, SIG_BLOCK, SIG_SETMASK};
use crate::src::shared::spawn::{spawn_editor_state, spawn_finish_edit_cb};
use crate::src::shared::spawn::{
    SPAWN_DETACHED, SPAWN_EMPTY, SPAWN_FLOATING, SPAWN_FLOATOVERZOOM, SPAWN_KILL, SPAWN_MODAL,
    SPAWN_NONOTIFY, SPAWN_RESPAWN, SPAWN_ZOOM,
};
use crate::src::shared::stdio::FILE;
use crate::src::shared::terminal::*;
use crate::src::shared::window::WINLINK_ALERTFLAGS;
use crate::src::shared::window::{window, winlink};

use std::os::fd::{AsRawFd, FromRawFd, IntoRawFd, OwnedFd};

pub type off_t = __off_t;

pub type uintmax_t = ::libc::uintmax_t;

impl spawn_editor_state {
    fn new(path: CString, cb: spawn_finish_edit_cb) -> Box<Self> {
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT_EDITOR_ID: AtomicU64 = AtomicU64::new(1);
        let id = NEXT_EDITOR_ID
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
            .expect("editor identity exhausted");
        Box::new(spawn_editor_state {
            id: crate::src::shared::spawn::EditorId(id),
            path: path,
            pid: 0,
            cb: cb,
        })
    }
}

pub const SEEK_SET: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const SEEK_END: ::core::ffi::c_int = 2 as ::core::ffi::c_int;

pub const IUTF8: ::core::ffi::c_int = 0o40000 as ::core::ffi::c_int;

pub const _PATH_DEFPATH: [::core::ffi::c_char; 14] =
    unsafe { ::core::mem::transmute::<[u8; 14], [::core::ffi::c_char; 14]>(*b"/usr/bin:/bin\0") };

pub const _PATH_TMP: [::core::ffi::c_char; 6] =
    unsafe { ::core::mem::transmute::<[u8; 6], [::core::ffi::c_char; 6]>(*b"/tmp/\0") };

pub(crate) unsafe fn spawn_log(mut from: *const ::core::ffi::c_char, mut sc: *mut spawn_context) {
    let session_owner = (*sc).s.upgrade().expect("spawn context session");
    let mut wl: refbox::Weak<winlink> = (*sc).winlink_handle();
    let wp0_owner = (*sc).wp0.upgrade();
    let wp0 = wp0_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    let name = (*sc).name.as_deref().unwrap_or(c"none");
    let mut tmp: [::core::ffi::c_char; 128] = [0; 128];
    log_debug(format_args!(
        "{}: name={}, flags={}",
        log_cstr((from) as *const _),
        log_cstr(name.as_ptr()),
        log_hex((((*sc).flags) as u32) as u64)
    ));
    if wl.is_alive() && !wp0.is_null() {
        xformat(
            &mut tmp,
            format_args!(
                "wl={} wp0=%{}",
                (wl.get_unchecked().idx) as i32,
                ((*wp0).id) as u32
            ),
        );
    } else if wl.is_alive() {
        xformat(
            &mut tmp,
            format_args!("wl={} wp0=none", (wl.get_unchecked().idx) as i32),
        );
    } else if !wp0.is_null() {
        xformat(
            &mut tmp,
            format_args!("wl=none wp0=%{}", ((*wp0).id) as u32),
        );
    } else {
        xformat(&mut tmp, format_args!("wl=none wp0=none"));
    }
    log_debug(format_args!(
        "{}: s=${} {} idx={}",
        log_cstr((from) as *const _),
        session_owner.id(),
        log_cstr((&raw mut tmp as *mut ::core::ffi::c_char) as *const _),
        ((*sc).idx) as i32
    ));
}
unsafe fn spawn_fire_pane_created(
    mut sc: *mut spawn_context,
    wp_owner: &std::rc::Rc<std::cell::UnsafeCell<window_pane>>,
) {
    let mut wp = wp_owner.get();
    let session_owner = (*sc).s.upgrade();
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        s: Default::default(),
        wl: Default::default(),
        w: Default::default(),
        wp: Default::default(),
        idx: 0,
    };
    let mut cwd: *const ::core::ffi::c_char = (*wp)
        .cwd
        .as_ref()
        .map_or(::core::ptr::null(), |value| value.as_ptr());
    let mut ep = event_payload_create();
    cmd_find_from_winlink_pane(
        &raw mut fs,
        ((*sc).winlink_handle()).clone(),
        wp_owner,
        0 as ::core::ffi::c_int,
    );
    event_payload_set_target(&mut *ep, &fs);
    event_payload_set_session(
        &mut *ep,
        b"session\0" as *const u8 as *const ::core::ffi::c_char,
        session_owner.expect("spawn context session"),
    );
    event_payload_set_window(
        &mut *ep,
        b"window\0" as *const u8 as *const ::core::ffi::c_char,
        std::rc::Rc::clone(&(((*wp).window_handle().as_ref()).expect("live window"))),
    );
    event_payload_set_int(
        &mut *ep,
        b"window_index\0" as *const u8 as *const ::core::ffi::c_char,
        ((*sc).winlink_handle()).get_unchecked().idx,
    );
    event_payload_set_pane(
        &mut *ep,
        b"pane\0" as *const u8 as *const ::core::ffi::c_char,
        (*(wp)).observer.upgrade().expect("live window_pane"),
    );
    let cmd = if !(*wp).argv.is_empty() {
        cmd_stringify_argv_cstring(&(*wp).argv)
    } else {
        None
    };
    if let Some(cmd) = cmd.as_ref().filter(|text| !text.as_bytes().is_empty()) {
        event_payload_set_string(
            &mut *ep,
            b"pane_command\0" as *const u8 as *const ::core::ffi::c_char,
            |out| write_cstr(out, cmd.as_ptr()),
        );
    } else if (*wp).shell.is_some() {
        event_payload_set_string(
            &mut *ep,
            b"pane_command\0" as *const u8 as *const ::core::ffi::c_char,
            |out| {
                write_cstr(
                    out,
                    (*wp)
                        .shell
                        .as_ref()
                        .map_or(::core::ptr::null(), |value| value.as_ptr()),
                )
            },
        );
    }
    if !cwd.is_null() {
        event_payload_set_string(
            &mut *ep,
            b"pane_current_path\0" as *const u8 as *const ::core::ffi::c_char,
            |out| write_cstr(out, cwd),
        );
    }
    if (*sc).flags & SPAWN_EMPTY != 0 {
        event_payload_set_int(
            &mut *ep,
            b"created_empty\0" as *const u8 as *const ::core::ffi::c_char,
            1 as ::core::ffi::c_int,
        );
    } else {
        event_payload_set_int(
            &mut *ep,
            b"created_empty\0" as *const u8 as *const ::core::ffi::c_char,
            0 as ::core::ffi::c_int,
        );
    }
    if (*sc).flags & SPAWN_RESPAWN != 0 {
        event_payload_set_int(
            &mut *ep,
            b"created_respawn\0" as *const u8 as *const ::core::ffi::c_char,
            1 as ::core::ffi::c_int,
        );
    } else {
        event_payload_set_int(
            &mut *ep,
            b"created_respawn\0" as *const u8 as *const ::core::ffi::c_char,
            0 as ::core::ffi::c_int,
        );
    }
    events_fire(
        b"pane-created\0" as *const u8 as *const ::core::ffi::c_char,
        ep,
    );
}
pub unsafe fn spawn_window(
    sc: *mut spawn_context,
    cause: *mut Option<CString>,
) -> refbox::Weak<winlink> {
    let session = (*sc).s.upgrade().expect("spawn context session");
    match session.spawn_window(&mut *sc) {
        Ok(link) => link,
        Err(error) => {
            if let Some(cause) = cause.as_mut() {
                *cause = Some(error);
            }
            refbox::Weak::new()
        }
    }
}
/// Reset an existing window's panes before its process is respawned.
/// Session link mutation remains in the Session-owned spawn transaction.
pub(crate) unsafe fn prepare_respawn_window(
    sc: *mut spawn_context,
    cause: *mut Option<CString>,
) -> bool {
    let window = (*sc)
        .winlink_handle()
        .get_unchecked()
        .window_handle()
        .cloned()
        .expect("live respawn window");
    let result = (|| {
        let mut wp: *mut window_pane = std::ptr::null_mut();
        if !(*sc).flags & SPAWN_KILL != 0 {
            wp = window
                .next_pane(None)
                .as_ref()
                .map_or(std::ptr::null_mut(), |owner| owner.get());
            while !wp.is_null() {
                if (*wp).fd != -(1 as ::core::ffi::c_int) {
                    break;
                }
                wp = window_pane_next(wp.as_ref())
                    .as_ref()
                    .map_or(std::ptr::null_mut(), |owner| owner.get());
            }
            if !wp.is_null() {
                set_spawn_cause(
                    cause.as_mut(),
                    &[
                        b"window ",
                        (*sc).s.upgrade().expect("spawn session").name().as_bytes(),
                        b":",
                        ((*sc).winlink_handle())
                            .get_unchecked()
                            .idx
                            .to_string()
                            .as_bytes(),
                        b" still active",
                    ],
                );
                return false;
            }
        }
        let source_pane_owner = window.next_pane(None).expect("respawn window has a pane");
        (*sc).wp0 = std::rc::Rc::downgrade(&source_pane_owner);
        let source = std::rc::Rc::downgrade(&source_pane_owner);
        for order in [
            crate::src::window::PaneOrder::Index,
            crate::src::window::PaneOrder::Stacking,
        ] {
            assert!(
                window.borrow_pane_order_mut(order).remove(&source),
                "pane is not in its window order"
            );
        }
        layout_free(&window);
        window_destroy_panes(&window);
        window
            .borrow_pane_order_mut(crate::src::window::PaneOrder::Index)
            .push_front(source.clone());
        window
            .borrow_pane_order_mut(crate::src::window::PaneOrder::Stacking)
            .push_back(source);
        let (sx, sy) = window.size();
        window_pane_resize(&source_pane_owner, sx, sy);
        layout_init(&window, &source_pane_owner);
        window.select_respawned_pane(&source_pane_owner);
        true
    })();
    window.release(c"prepare respawn window");
    result
}
/// Initialize the newly-created window's name and fill cells without emitting
/// a rename notification (the window-created notification follows spawning).
pub(crate) unsafe fn initialize_spawned_window(sc: *mut spawn_context, window: &WindowRef) {
    if let Some(name) = (*sc).name.as_ref() {
        window.initialize_name(name.clone(), true);
    } else {
        window.initialize_name(default_window_name_cstring(window), false);
    }
    window.refresh_fill_cells();
}
pub unsafe fn spawn_pane(
    mut sc: *mut spawn_context,
    cause: *mut Option<CString>,
) -> Option<std::rc::Rc<std::cell::UnsafeCell<window_pane>>> {
    let source_pane_owner = (*sc).wp0.upgrade();
    let source_pane = source_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    let item_owner = (*sc).item.upgrade();
    let mut item: *mut cmdq_item = item_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    let mut c: Option<ClientRef> = None;
    let mut loop_0: Option<ClientRef> = None;
    let session_owner = (*sc).s.upgrade().expect("spawn context session");
    let original_window = (*sc)
        .winlink_handle()
        .get_unchecked()
        .window_handle()
        .cloned()
        .expect("spawn window");
    let result = (|| {
        let target_session_owner;
        let new_pane_owner;
        let mut new_wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
        let mut cp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut argvp: *mut *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
        let mut cwd: Option<CString> = None;
        let mut path: [::core::ffi::c_char; 4096] = [0; 4096];
        let mut cmd: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut tmp: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut home: *const ::core::ffi::c_char =
            find_home_cstr().map_or(::core::ptr::null(), CStr::as_ptr);
        let mut actual_cwd: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut idx: u_int = 0;
        let mut now: termios = termios {
            c_iflag: 0,
            c_oflag: 0,
            c_cflag: 0,
            c_lflag: 0,
            c_line: 0,
            c_cc: [0; 32],
            c2rust_unnamed: termios_input_speed { __ispeed: 0 },
            c2rust_unnamed_0: termios_output_speed { __ospeed: 0 },
        };
        let mut hlimit: u_int = 0;
        let mut ws: winsize = winsize {
            ws_row: 0,
            ws_col: 0,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };
        let mut set: sigset_t = __sigset_t { __val: [0; 16] };
        let mut oldset: sigset_t = __sigset_t { __val: [0; 16] };
        let mut key: key_code = 0;
        let c_owner;
        if !item.is_null() {
            target_session_owner =
                (*crate::src::cmd::queue::cmdq_get_target_mut(&mut *item)).session_handle();
            c_owner = cmdq_get_client((item).as_ref());
            c = c_owner.clone();
        } else {
            target_session_owner = Some(session_owner.clone());
            c_owner = (*sc).tc.upgrade();
            c = c_owner.clone();
        }
        spawn_log(
            b"spawn_pane\0" as *const u8 as *const ::core::ffi::c_char,
            sc,
        );
        if (*sc).flags & SPAWN_MODAL != 0 {
            if !(*sc).flags & SPAWN_FLOATING != 0 {
                set_spawn_cause(cause.as_mut(), &[b"modal pane must be floating"]);
                return None;
            }
            if original_window.modal_pane().is_some() {
                set_spawn_cause(cause.as_mut(), &[b"window already has a modal pane"]);
                return None;
            }
        }
        if let Some(requested_cwd) = (*sc).cwd.as_ref() {
            if !item.is_null() {
                cwd = Some(format_single_cstring(
                    (item)
                        .as_ref()
                        .and_then(|item| item.observer.upgrade())
                        .as_ref(),
                    requested_cwd.as_ptr(),
                    c.as_ref(),
                    target_session_owner.as_ref(),
                    (refbox::Weak::new()).clone(),
                    None,
                ));
            } else {
                cwd = Some(requested_cwd.clone());
            }
            let value = cwd.as_ref().expect("spawn cwd was just set");
            if !value.as_bytes().starts_with(b"/") {
                let base_owner = match c_owner.as_ref() {
                    Some(client) => client.cwd(target_session_owner.as_ref()),
                    None => server_client_get_cwd(None, target_session_owner.as_ref()),
                };
                // Preserve the old formatter's rendering for an absent startup cwd.
                let base = base_owner
                    .as_deref()
                    .map_or(b"(null)".as_slice(), CStr::to_bytes);
                let mut combined = Vec::with_capacity(base.len() + value.as_bytes().len() + 1);
                combined.extend_from_slice(base);
                if !value.as_bytes().is_empty() {
                    combined.push(b'/');
                }
                combined.extend_from_slice(value.as_bytes());
                cwd = Some(CString::new(combined).expect("combined cwd contains no NUL"));
            }
        } else if !(*sc).flags & SPAWN_RESPAWN != 0 {
            cwd = match c_owner.as_ref() {
                Some(client) => client.cwd(target_session_owner.as_ref()),
                None => server_client_get_cwd(None, target_session_owner.as_ref()),
            };
        }
        hlimit = session_owner.with_options_mut(|options| {
            options_get_number(options, c"history-limit".as_ptr()) as u_int
        });
        if (*sc).flags & SPAWN_RESPAWN != 0 {
            if (*source_pane).fd != -(1 as ::core::ffi::c_int) && !(*sc).flags & SPAWN_KILL != 0 {
                idx = window_pane_index(&*source_pane).expect("pane belongs to window ordering");
                set_spawn_cause(
                    cause.as_mut(),
                    &[
                        b"pane ",
                        session_owner.name().as_bytes(),
                        b":",
                        ((*sc).winlink_handle())
                            .get_unchecked()
                            .idx
                            .to_string()
                            .as_bytes(),
                        b".",
                        idx.to_string().as_bytes(),
                        b" still active",
                    ],
                );
                return None;
            }
            std::mem::take(&mut (*source_pane).event).free();
            if (*source_pane).fd != -(1 as ::core::ffi::c_int) {
                close((*source_pane).fd);
                (*source_pane).fd = -(1 as ::core::ffi::c_int);
            }
            window_pane_reset_mode_all(source_pane_owner.as_ref().expect("respawn source pane"));
            screen_reinit(&mut (*source_pane).base, 0 as ::core::ffi::c_int);
            if let Some(ictx) = (*source_pane).ictx.take() {
                input_free(ictx);
            }
            (*source_pane).offset.used = 0 as size_t;
            (*source_pane).base_offset = 0 as size_t;
            (*source_pane).pipe_offset.used = 0 as size_t;
            let mut registry_loop_0_owner = clients.first();
            loop_0 = registry_loop_0_owner.clone();
            while !loop_0.is_none() {
                if loop_0.as_ref().expect("live client").flags() & CLIENT_CONTROL as uint64_t != 0 {
                    control_reset_pane(
                        &loop_0.clone().expect("live client"),
                        &(*(source_pane))
                            .observer
                            .upgrade()
                            .expect("live window_pane"),
                    );
                }
                registry_loop_0_owner = clients.next(
                    registry_loop_0_owner
                        .as_ref()
                        .expect("current registry client"),
                );
                loop_0 = registry_loop_0_owner.clone();
            }
            new_pane_owner = source_pane_owner.as_ref().expect("respawn pane").clone();
            new_wp = new_pane_owner.get();
            (*new_wp).flags &= !(PANE_STATUSREADY | PANE_STATUSDRAWN);
        } else {
            if (*sc).layout.is_none() {
                new_pane_owner = window_add_pane(
                    &std::rc::Rc::clone(
                        &((((*sc).winlink_handle())
                            .get_unchecked()
                            .window_handle()
                            .as_ref())
                        .expect("live window")),
                    ),
                    None,
                    hlimit,
                    (*sc).flags,
                );
                new_wp = new_pane_owner.get();
                layout_init(
                    &std::rc::Rc::clone(
                        &((((*sc).winlink_handle())
                            .get_unchecked()
                            .window_handle()
                            .as_ref())
                        .expect("live window")),
                    ),
                    &(*(new_wp)).observer.upgrade().expect("live window_pane"),
                );
            } else {
                new_pane_owner = window_add_pane(
                    &std::rc::Rc::clone(
                        &((((*sc).winlink_handle())
                            .get_unchecked()
                            .window_handle()
                            .as_ref())
                        .expect("live window")),
                    ),
                    (source_pane)
                        .as_ref()
                        .and_then(|model| model.observer.upgrade())
                        .as_ref(),
                    hlimit,
                    (*sc).flags,
                );
                new_wp = new_pane_owner.get();
                if (*sc).flags & SPAWN_ZOOM != 0 {
                    layout_assign_pane(
                        &original_window,
                        (*sc).layout.expect("reserved pane layout"),
                        &(*(new_wp)).observer.upgrade().expect("live window_pane"),
                        1 as ::core::ffi::c_int,
                    );
                } else {
                    layout_assign_pane(
                        &original_window,
                        (*sc).layout.expect("reserved pane layout"),
                        &(*(new_wp)).observer.upgrade().expect("live window_pane"),
                        0 as ::core::ffi::c_int,
                    );
                }
            }
            if (*sc).flags & SPAWN_FLOATING != 0 {
                // Assignment resizes panes and may reenter. Preserve the fresh
                // post-assignment lookup, then borrow the current owning tree.
                let cell_id = (*new_wp).layout_cell.expect("spawned pane layout");
                let window = new_pane_owner
                    .window_observer()
                    .upgrade()
                    .expect("spawned pane window");
                {
                    let mut cell = window
                        .borrow_layout_cell_mut(cell_id)
                        .expect("spawned pane belongs to layout");
                    cell.flags |= LAYOUT_CELL_FLOATING;
                }
                window.release(c"spawn floating pane layout");
            }
            if (*sc).flags & SPAWN_FLOATOVERZOOM != 0 {
                (*new_wp).flags |= PANE_FLOATOVERZOOM;
            }
            if original_window.is_zoomed() {
                (*new_wp).saved_layout_cell = (*new_wp).layout_cell;
            }
        }
        if (*sc).argv.is_empty() {
            if (*sc).flags & SPAWN_RESPAWN == 0 {
                let command = session_owner.with_options_mut(|options| {
                    crate::src::options::options_get_string_optional(
                        options,
                        c"default-command".as_ptr(),
                    )
                });
                if let Some(command) = command.filter(|command| !command.as_bytes().is_empty()) {
                    (*new_wp).argv = vec![command];
                } else {
                    (*new_wp).argv.clear();
                }
            }
        } else {
            (*new_wp).argv = (*sc).argv.clone();
        }
        if let Some(cwd) = cwd.take() {
            window_pane_set_cwd(&mut *new_wp, Some(cwd));
        }
        // Each fork path owns its environment until it has been installed.
        let mut child_owner = Some(environ_for_session(Some(&session_owner), 0));
        let child = child_owner.as_deref_mut().expect("spawn environment");
        if let Some(overrides) = (*sc).environ.as_deref() {
            environ_copy(overrides, child);
        }
        environ_set(
            child,
            b"TMUX_PANE\0" as *const u8 as *const ::core::ffi::c_char,
            0 as ::core::ffi::c_int,
            |out| write!(out, "%{}", ((*new_wp).id) as u32),
        );
        if !c.is_none()
            && c.as_ref()
                .expect("live client")
                .attached_session()
                .upgrade()
                .is_none()
        {
            let path = c.as_ref().expect("live client").with_environment(|env| {
                env.and_then(|env| env.find(c"PATH"))
                    .map(|entry| entry.value.clone())
            });
            if let Some(value) = path {
                environ_set(child, c"PATH".as_ptr(), 0, |out| {
                    write_cstr(
                        out,
                        value
                            .as_ref()
                            .map_or(std::ptr::null(), |value| value.as_ptr()),
                    )
                });
            }
        }
        if child.find(c"PATH").is_none() {
            environ_set(
                child,
                b"PATH\0" as *const u8 as *const ::core::ffi::c_char,
                0 as ::core::ffi::c_int,
                |out| write_cstr(out, _PATH_DEFPATH.as_ptr()),
            );
        }
        if !(*sc).flags & SPAWN_RESPAWN != 0 {
            let shell = session_owner
                .with_options_mut(|options| options_get_string(options, c"default-shell".as_ptr()));
            tmp = shell.as_ptr();
            if checkshell(tmp) == 0 {
                tmp = _PATH_BSHELL.as_ptr();
            }
            window_pane_set_shell(&mut *new_wp, Some(CStr::from_ptr(tmp).to_owned()));
        }
        environ_set(
            child,
            b"SHELL\0" as *const u8 as *const ::core::ffi::c_char,
            0 as ::core::ffi::c_int,
            |out| {
                write_cstr(
                    out,
                    (*new_wp)
                        .shell
                        .as_ref()
                        .map_or(::core::ptr::null(), |value| value.as_ptr()),
                )
            },
        );
        log_debug(format_args!(
            "{}: shell={}",
            "spawn_pane",
            log_cstr(
                ((*new_wp)
                    .shell
                    .as_ref()
                    .map_or(::core::ptr::null(), |value| value.as_ptr()))
                    as *const _
            )
        ));
        if !(*new_wp).argv.is_empty() {
            let command = cmd_stringify_argv_cstring(&(*new_wp).argv);
            log_debug(format_args!(
                "{}: cmd={}",
                "spawn_pane",
                log_cstr(
                    (command
                        .as_ref()
                        .map_or(::core::ptr::null(), |text| text.as_ptr()))
                        as *const _
                )
            ));
        }
        log_debug(format_args!(
            "{}: cwd={}",
            "spawn_pane",
            log_cstr(
                ((*new_wp)
                    .cwd
                    .as_ref()
                    .map_or(::core::ptr::null(), |value| value.as_ptr()))
                    as *const _
            )
        ));
        cmd_log_argv(&(*new_wp).argv, c"spawn_pane");
        environ_log(child, |out| {
            write_cstr(
                out,
                b"spawn_pane\0" as *const u8 as *const ::core::ffi::c_char,
            )?;
            out.write_all(b": environment ")
        });
        memset(
            &raw mut ws as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<winsize>() as size_t,
        );
        ws.ws_col = (*new_wp).base.grid().sx as ::core::ffi::c_ushort;
        ws.ws_row = (*new_wp).base.grid().sy as ::core::ffi::c_ushort;
        ws.ws_xpixel = ((((*sc).winlink_handle())
            .get_unchecked()
            .window_handle()
            .as_ref())
        .expect("live window"))
        .cell_size()
        .0
        .wrapping_mul(ws.ws_col as u_int) as ::core::ffi::c_ushort;
        ws.ws_ypixel = ((((*sc).winlink_handle())
            .get_unchecked()
            .window_handle()
            .as_ref())
        .expect("live window"))
        .cell_size()
        .1
        .wrapping_mul(ws.ws_row as u_int) as ::core::ffi::c_ushort;
        sigfillset(&raw mut set);
        sigprocmask(SIG_BLOCK, &raw mut set, &raw mut oldset);
        if (*sc).flags & SPAWN_EMPTY != 0 {
            (*new_wp).flags |= PANE_EMPTY;
            (*new_wp).base.mode &= !MODE_CURSOR;
            (*new_wp).base.mode |= MODE_CRLF;
        } else {
            (*new_wp).flags &= !PANE_EMPTY;
            if !getcwd(
                &raw mut path as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 4096]>() as size_t,
            )
            .is_null()
            {
                if chdir(
                    (*new_wp)
                        .cwd
                        .as_ref()
                        .map_or(::core::ptr::null(), |value| value.as_ptr()),
                ) == 0 as ::core::ffi::c_int
                {
                    actual_cwd = (*new_wp)
                        .cwd
                        .as_ref()
                        .map_or(::core::ptr::null(), |value| value.as_ptr());
                } else if !home.is_null() && chdir(home) == 0 as ::core::ffi::c_int {
                    actual_cwd = home;
                } else if chdir(b"/\0" as *const u8 as *const ::core::ffi::c_char)
                    == 0 as ::core::ffi::c_int
                {
                    actual_cwd = b"/\0" as *const u8 as *const ::core::ffi::c_char;
                }
            }
            (*new_wp).pid = fdforkpty(
                ptm_fd,
                &raw mut (*new_wp).fd,
                &raw mut (*new_wp).tty as *mut ::core::ffi::c_char,
                ::core::ptr::null_mut::<termios>(),
                &raw mut ws,
            );
            if (*new_wp).pid == -(1 as ::core::ffi::c_int) {
                set_spawn_cause(
                    cause.as_mut(),
                    &[
                        b"fork failed: ",
                        CStr::from_ptr(strerror(*__errno_location())).to_bytes(),
                    ],
                );
                (*new_wp).fd = -(1 as ::core::ffi::c_int);
                if !(*sc).flags & SPAWN_RESPAWN != 0 {
                    let pane_owner = (*new_wp).observer.upgrade().expect("new pane owner");
                    server_client_remove_pane(
                        &(*(new_wp)).observer.upgrade().expect("live window_pane"),
                    );
                    layout_close_pane(&(*(new_wp)).observer.upgrade().expect("live window_pane"));
                    window_remove_pane(
                        &std::rc::Rc::clone(
                            &((((*sc).winlink_handle())
                                .get_unchecked()
                                .window_handle()
                                .as_ref())
                            .expect("live window")),
                        ),
                        &pane_owner,
                    );
                }
                sigprocmask(
                    SIG_SETMASK,
                    &raw mut oldset,
                    ::core::ptr::null_mut::<sigset_t>(),
                );
                return None;
            }
            if (*new_wp).pid != 0 as ::core::ffi::c_int {
                if !actual_cwd.is_null()
                    && chdir(&raw mut path as *mut ::core::ffi::c_char) != 0 as ::core::ffi::c_int
                    && (home.is_null() || chdir(home) != 0 as ::core::ffi::c_int)
                {
                    chdir(b"/\0" as *const u8 as *const ::core::ffi::c_char);
                }
            } else {
                let (systemd_status, systemd_error) = systemd_move_to_new_cgroup();
                if systemd_status < 0 as ::core::ffi::c_int {
                    log_debug(format_args!(
                        "{}: moving pane to new cgroup failed: {}",
                        "spawn_pane",
                        log_cstr(
                            (systemd_error
                                .as_ref()
                                .map_or(::core::ptr::null(), |error| error.as_ptr()))
                                as *const _
                        )
                    ));
                }
                drop(systemd_error);
                if !actual_cwd.is_null() {
                    environ_set(
                        child,
                        b"PWD\0" as *const u8 as *const ::core::ffi::c_char,
                        0 as ::core::ffi::c_int,
                        |out| write_cstr(out, actual_cwd),
                    );
                }
                if tcgetattr(STDIN_FILENO, &raw mut now) != 0 as ::core::ffi::c_int {
                    _exit(1 as ::core::ffi::c_int);
                }
                if let Some(terminal) = session_owner.termios() {
                    now.c_cc = terminal.c_cc;
                }
                key = options_get_number(
                    global_options,
                    b"backspace\0" as *const u8 as *const ::core::ffi::c_char,
                ) as key_code;
                if key >= 0x7f as key_code {
                    now.c_cc[VERASE as usize] = '\u{7f}' as i32 as cc_t;
                } else {
                    now.c_cc[VERASE as usize] = key as cc_t;
                }
                now.c_iflag |= IUTF8 as tcflag_t;
                if tcsetattr(STDIN_FILENO, TCSANOW, &raw mut now) != 0 as ::core::ffi::c_int {
                    _exit(1 as ::core::ffi::c_int);
                }
                proc_clear_signals(server_proc, 1 as ::core::ffi::c_int);
                closefrom(STDERR_FILENO + 1 as ::core::ffi::c_int);
                sigprocmask(
                    SIG_SETMASK,
                    &raw mut oldset,
                    ::core::ptr::null_mut::<sigset_t>(),
                );
                log_close();
                environ_push(child);
                // After fork this is the child's private copy. Release it after
                // publishing the process environment; the parent retains its own
                // owner and drops it when this function returns.
                drop(child_owner.take());
                if (*new_wp).argv.len() > 1 {
                    let mut exec_argv: Vec<_> =
                        (*new_wp).argv.iter().map(|arg| arg.as_ptr()).collect();
                    exec_argv.push(::core::ptr::null());
                    argvp = exec_argv.as_mut_ptr().cast();
                    execvp(
                        *argvp.offset(0 as ::core::ffi::c_int as isize),
                        argvp as *const *mut ::core::ffi::c_char,
                    );
                    _exit(1 as ::core::ffi::c_int);
                }
                cp = strrchr(
                    (*new_wp)
                        .shell
                        .as_ref()
                        .map_or(::core::ptr::null(), |value| value.as_ptr()),
                    '/' as i32,
                );
                let shell_name = if !cp.is_null() && *cp.add(1) != 0 {
                    cp.add(1)
                } else {
                    (*new_wp)
                        .shell
                        .as_ref()
                        .map_or(::core::ptr::null(), |value| value.as_ptr())
                };
                if (*new_wp).argv.len() == 1 {
                    tmp = (&(*new_wp).argv)[0].as_ptr();
                    let argv0 = CStr::from_ptr(shell_name).to_owned();
                    execl(
                        (*new_wp)
                            .shell
                            .as_ref()
                            .map_or(::core::ptr::null(), |value| value.as_ptr()),
                        argv0.as_ptr(),
                        b"-c\0" as *const u8 as *const ::core::ffi::c_char,
                        tmp,
                        NULL as *mut ::core::ffi::c_char,
                    );
                    _exit(1 as ::core::ffi::c_int);
                }
                let mut login_name = vec![b'-'];
                login_name.extend_from_slice(CStr::from_ptr(shell_name).to_bytes());
                let argv0 = CString::new(login_name).expect("shell name contains no NUL");
                execl(
                    (*new_wp)
                        .shell
                        .as_ref()
                        .map_or(::core::ptr::null(), |value| value.as_ptr()),
                    argv0.as_ptr(),
                    NULL as *mut ::core::ffi::c_char,
                );
                _exit(1 as ::core::ffi::c_int);
            }
        }
        if !(*new_wp).flags & PANE_EMPTY != 0 {
            let record = CString::new(format!("tmux({}).%{}", getpid(), (*new_wp).id))
                .expect("pane login record contains no NUL");
            utempter_add_record((*new_wp).fd, record.as_ptr());
            kill(getpid(), SIGCHLD);
        }
        (*new_wp).flags &= !PANE_EXITED;
        sigprocmask(
            SIG_SETMASK,
            &raw mut oldset,
            ::core::ptr::null_mut::<sigset_t>(),
        );
        window_pane_set_event(&(*(new_wp)).observer.upgrade().expect("live window_pane"));
        drop(child_owner.take());
        spawn_fire_pane_created(
            sc,
            &(*(new_wp)).observer.upgrade().expect("live window_pane"),
        );
        if (*sc).flags & SPAWN_RESPAWN != 0 {
            return Some(new_pane_owner);
        }
        if (*sc).flags & SPAWN_MODAL != 0 {
            original_window.begin_modal_pane(&new_pane_owner);
            window_redraw_active_switch(
                &std::rc::Rc::clone(
                    &((((*sc).winlink_handle())
                        .get_unchecked()
                        .window_handle()
                        .as_ref())
                    .expect("live window")),
                ),
                (new_wp)
                    .as_ref()
                    .and_then(|model| model.observer.upgrade())
                    .as_ref(),
            );
            if (*sc).flags & SPAWN_NONOTIFY != 0 {
                window_set_active_pane(
                    &std::rc::Rc::clone(
                        &((((*sc).winlink_handle())
                            .get_unchecked()
                            .window_handle()
                            .as_ref())
                        .expect("live window")),
                    ),
                    &(*(new_wp)).observer.upgrade().expect("live window_pane"),
                    0 as ::core::ffi::c_int,
                );
            } else {
                window_set_active_pane(
                    &std::rc::Rc::clone(
                        &((((*sc).winlink_handle())
                            .get_unchecked()
                            .window_handle()
                            .as_ref())
                        .expect("live window")),
                    ),
                    &(*(new_wp)).observer.upgrade().expect("live window_pane"),
                    1 as ::core::ffi::c_int,
                );
            }
        } else if (!(*sc).flags & SPAWN_DETACHED != 0
            || ((((*sc).winlink_handle())
                .get_unchecked()
                .window_handle()
                .as_ref())
            .expect("live window"))
            .active_pane()
            .is_none())
            && original_window.modal_pane().is_none()
        {
            if (*sc).flags & SPAWN_NONOTIFY != 0 {
                window_set_active_pane(
                    &std::rc::Rc::clone(
                        &((((*sc).winlink_handle())
                            .get_unchecked()
                            .window_handle()
                            .as_ref())
                        .expect("live window")),
                    ),
                    &(*(new_wp)).observer.upgrade().expect("live window_pane"),
                    0 as ::core::ffi::c_int,
                );
            } else {
                window_set_active_pane(
                    &std::rc::Rc::clone(
                        &((((*sc).winlink_handle())
                            .get_unchecked()
                            .window_handle()
                            .as_ref())
                        .expect("live window")),
                    ),
                    &(*(new_wp)).observer.upgrade().expect("live window_pane"),
                    1 as ::core::ffi::c_int,
                );
            }
        }
        if !(*sc).flags & SPAWN_NONOTIFY != 0 {
            events_fire_window(
                b"window-layout-changed\0" as *const u8 as *const ::core::ffi::c_char,
                std::rc::Rc::clone(
                    &((((*sc).winlink_handle())
                        .get_unchecked()
                        .window_handle()
                        .as_ref())
                    .expect("live window")),
                ),
            );
        }
        Some(new_pane_owner)
    })();
    original_window.release(c"spawn pane");
    result
}
impl Drop for spawn_editor_state {
    fn drop(&mut self) {
        unsafe {
            unlink(self.path.as_ptr());
        }
    }
}
pub unsafe fn spawn_cancel_editor(mut es: *mut spawn_editor_state) {
    if es.is_null() {
        return;
    }
    (*es).cb = None;
}
pub unsafe fn spawn_get_editor_pid(mut es: *mut spawn_editor_state) -> pid_t {
    if es.is_null() {
        return -(1 as pid_t);
    }
    return (*es).pid;
}
pub unsafe fn spawn_editor_finish(wp_owner: &std::rc::Rc<std::cell::UnsafeCell<window_pane>>) {
    let mut wp = wp_owner.get();
    let Some(mut owner) = (*wp).editor.take() else {
        return;
    };
    let es = &raw mut *owner;
    let mut f: *mut FILE = ::core::ptr::null_mut::<FILE>();
    let mut result: Option<Vec<u8>> = None;
    let mut len: off_t = 0 as off_t;
    let mut status: ::core::ffi::c_int = 128 as ::core::ffi::c_int + SIGHUP;
    if (*wp).flags & PANE_STATUSREADY != 0 {
        if (*wp).status & 0x7f as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            status = ((*wp).status & 0xff00 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int;
        } else if (((*wp).status & 0x7f as ::core::ffi::c_int) + 1 as ::core::ffi::c_int)
            as ::core::ffi::c_schar as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int
            > 0 as ::core::ffi::c_int
        {
            status = ((*wp).status & 0x7f as ::core::ffi::c_int) + 128 as ::core::ffi::c_int;
        }
    }
    if (*es).cb.is_none() {
        return;
    }
    if status != 0 as ::core::ffi::c_int {
        (*es).cb.take().expect("non-null editor callback")(
            std::ptr::NonNull::new(es).unwrap(),
            None,
        );
        return;
    }
    f = fopen(
        ((*es).path).as_ptr().cast_mut(),
        b"r\0" as *const u8 as *const ::core::ffi::c_char,
    ) as *mut FILE;
    if !f.is_null() {
        let stream = CFile::from_raw(f).expect("fopen returned a non-null stream");
        if fseeko(stream.as_ptr(), 0 as __off_t, SEEK_END) == 0 as ::core::ffi::c_int {
            len = ftello(stream.as_ptr()) as off_t;
            if len >= 0 as off_t && len as uintmax_t <= SIZE_MAX as uintmax_t {
                if fseeko(stream.as_ptr(), 0 as __off_t, SEEK_SET) == 0 as ::core::ffi::c_int {
                    if len == 0 as off_t {
                        result = Some(Vec::new());
                    } else {
                        let mut bytes = Vec::new();
                        if bytes.try_reserve_exact(len as usize).is_ok() {
                            bytes.resize(len as usize, 0);
                            if fread(
                                bytes.as_mut_ptr().cast::<::core::ffi::c_void>(),
                                len as size_t,
                                1 as size_t,
                                stream.as_ptr(),
                            ) == 1 as ::core::ffi::c_ulong
                            {
                                result = Some(bytes);
                            }
                        }
                    }
                }
            } else {
                len = 0 as off_t;
            }
        }
        drop(stream);
    }
    (*es).cb.take().expect("non-null editor callback")(std::ptr::NonNull::new(es).unwrap(), result);
}

/// Open the editor's temporary descriptor as a C `FILE` while keeping its
/// ownership explicit at the Rust/stdio boundary.
///
/// `fd_owner` owns the live descriptor returned by `mkstemp`. On `fdopen`
/// failure, `FILE` never owns the descriptor and `OwnedFd::drop` closes it
/// exactly once. On success, `into_raw_fd` relinquishes the Rust owner before
/// the caller can `fclose` the stream, making `FILE` the sole closer.
unsafe fn spawn_editor_fdopen(fd_owner: OwnedFd) -> *mut FILE {
    let mode: &CStr = c"w";
    let file = fdopen(fd_owner.as_raw_fd(), mode.as_ptr());
    if file.is_null() {
        // Keep the fdopen errno available to the caller while the Rust owner
        // closes the descriptor on this failure path.
        let fdopen_errno = *__errno_location();
        drop(fd_owner);
        *__errno_location() = fdopen_errno;
        return ::core::ptr::null_mut::<FILE>();
    }
    let _fd_owned_by_file = fd_owner.into_raw_fd();
    file
}

/// Write the existing bytes before editor pane creation can dispatch events.
/// Keep the fwrite item-count behavior for empty input and short writes.
pub(crate) fn spawn_editor_write(stream: &CFile, bytes: &[u8]) -> bool {
    unsafe { fwrite(bytes.as_ptr().cast(), bytes.len(), 1, stream.as_ptr()) == 1 }
}

pub(crate) unsafe fn spawn_editor(
    client_owner: &ClientRef,
    write: impl FnOnce(&CFile) -> bool,
    mut cb: spawn_finish_edit_cb,
) -> Option<crate::src::shared::spawn::EditorHandle> {
    let Some(session_owner) = client_owner.attached_session().upgrade() else {
        return None;
    };
    let mut es: *mut spawn_editor_state = ::core::ptr::null_mut::<spawn_editor_state>();
    let mut sc: spawn_context = spawn_context {
        item: std::rc::Weak::new(),
        s: std::rc::Weak::new(),
        wl: refbox::Weak::new(),
        tc: std::rc::Weak::new(),
        wp0: std::rc::Weak::new(),
        layout: None,
        name: None,
        argv: Vec::new(),
        environ: None,
        idx: 0,
        cwd: None,
        flags: 0,
    };
    let mut wl: refbox::Weak<winlink> = session_owner.current_winlink();
    let original_window = wl
        .get_unchecked()
        .window_handle()
        .cloned()
        .expect("editor window");
    let result = (|| {
        let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
        let mut lg: layout_geometry = layout_geometry {
            sx: 0,
            sy: 0,
            xoff: 0,
            yoff: 0,
        };
        let mut f: *mut FILE = ::core::ptr::null_mut::<FILE>();
        let mut cause: Option<CString> = None;
        let mut path: [::core::ffi::c_char; 19] =
            ::core::mem::transmute::<[u8; 19], [::core::ffi::c_char; 19]>(*b"/tmp/tmux.XXXXXXXX\0");
        let mut editor: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut fd: ::core::ffi::c_int = 0;
        if original_window.modal_pane().is_some() {
            return None;
        }
        let editor_value = options_get_string(
            global_options,
            b"editor\0" as *const u8 as *const ::core::ffi::c_char,
        );
        editor = editor_value.as_ptr();
        fd = mkstemp(&raw mut path as *mut ::core::ffi::c_char);
        if fd == -(1 as ::core::ffi::c_int) {
            return None;
        }
        let fd_owner = OwnedFd::from_raw_fd(fd);
        f = spawn_editor_fdopen(fd_owner);
        if f.is_null() {
            unlink(&raw mut path as *mut ::core::ffi::c_char);
            return None;
        }
        let stream = CFile::from_raw(f).expect("fdopen returned a non-null stream");
        if !write(&stream) {
            drop(stream);
            unlink(&raw mut path as *mut ::core::ffi::c_char);
            return None;
        }
        drop(stream);
        let mut owner = spawn_editor_state::new(CStr::from_ptr(path.as_ptr()).to_owned(), cb);
        es = &raw mut *owner;
        lg.sx = ((wl.get_unchecked().window_handle().as_ref()).expect("live window"))
            .size()
            .0
            .wrapping_mul(9 as u_int)
            .wrapping_div(10 as u_int);
        lg.sy = ((wl.get_unchecked().window_handle().as_ref()).expect("live window"))
            .size()
            .1
            .wrapping_mul(9 as u_int)
            .wrapping_div(10 as u_int);
        lg.xoff = ((wl.get_unchecked().window_handle().as_ref()).expect("live window"))
            .size()
            .0
            .wrapping_div(2 as u_int)
            .wrapping_sub(lg.sx.wrapping_div(2 as u_int)) as ::core::ffi::c_int;
        lg.yoff = ((wl.get_unchecked().window_handle().as_ref()).expect("live window"))
            .size()
            .1
            .wrapping_div(2 as u_int)
            .wrapping_sub(lg.sy.wrapping_div(2 as u_int)) as ::core::ffi::c_int;
        window_push_zoom(
            &std::rc::Rc::clone(
                &((wl.get_unchecked().window_handle().as_ref()).expect("live window")),
            ),
            0 as ::core::ffi::c_int,
            1 as ::core::ffi::c_int,
        );
        let layout_id = layout_floating_pane(
            &std::rc::Rc::clone(
                &((wl.get_unchecked().window_handle().as_ref()).expect("live window")),
            ),
            None,
            &raw mut lg,
        );
        let cmd = CString::new(
            [
                CStr::from_ptr(editor).to_bytes(),
                b" ",
                CStr::from_ptr(path.as_ptr()).to_bytes(),
            ]
            .concat(),
        )
        .expect("editor command components contain no NUL");
        sc.s = std::rc::Rc::downgrade(&session_owner);
        sc.set_wl(wl.clone());
        sc.tc = std::rc::Rc::downgrade(client_owner);
        sc.wp0 = original_window.active_pane_observer();
        sc.layout = Some(layout_id);
        sc.argv = vec![cmd];
        sc.environ = Some(environ_create());
        sc.idx = -(1 as ::core::ffi::c_int);
        sc.cwd = Some(c"/tmp/".to_owned());
        sc.flags = SPAWN_FLOATING | SPAWN_MODAL | SPAWN_FLOATOVERZOOM;
        let spawned_pane = spawn_pane(&raw mut sc, &raw mut cause);
        wp = spawned_pane
            .as_ref()
            .map_or(std::ptr::null_mut(), |owner| owner.get());
        if wp.is_null() {
            window_pop_zoom(&std::rc::Rc::clone(
                &((wl.get_unchecked().window_handle().as_ref()).expect("live window")),
            ));
            return None;
        }
        window_pop_zoom(&std::rc::Rc::clone(
            &((wl.get_unchecked().window_handle().as_ref()).expect("live window")),
        ));
        options_set_number(
            options_owner_ptr(&mut (*wp).options).map_or(std::ptr::null_mut(), |options| options),
            b"remain-on-exit\0" as *const u8 as *const ::core::ffi::c_char,
            0 as ::core::ffi::c_longlong,
        );
        (*es).pid = (*wp).pid;
        (*wp).editor = Some(owner);
        let pane_owner = (*wp)
            .observer
            .upgrade()
            .expect("new editor pane has an owner");
        Some(crate::src::shared::spawn::EditorHandle::new(
            &pane_owner,
            std::ptr::NonNull::new_unchecked(es),
        ))
    })();
    original_window.release(c"spawn editor");
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::src::ffi::libc::fclose;
    use crate::src::shared::spawn::EditorHandle;
    use std::ffi::CStr;
    use std::fs;
    use std::process::{Command, Stdio};
    use std::thread;
    use std::time::{Duration, Instant};

    #[test]
    fn stale_editor_handle_does_not_cancel_replacement() {
        let pane = window_pane::new();
        unsafe {
            let first = spawn_editor_state::new(c"".to_owned(), Some(Box::new(|_, _| {})));
            (*pane.get()).editor = Some(first);
            let first_ptr =
                std::ptr::NonNull::from((*pane.get()).editor.as_mut().unwrap().as_mut());
            let stale = EditorHandle::new(&pane, first_ptr);
            let previous = (*pane.get()).editor.take();
            let second = spawn_editor_state::new(c"".to_owned(), Some(Box::new(|_, _| {})));
            (*pane.get()).editor = Some(second);
            let second_ptr =
                std::ptr::NonNull::from((*pane.get()).editor.as_mut().unwrap().as_mut());
            let current = EditorHandle::new(&pane, second_ptr);

            assert_eq!(stale.pid(), -1);
            assert!(!stale.matches(second_ptr.as_ref()));
            stale.cancel();
            assert!((*pane.get()).editor.as_ref().unwrap().cb.is_some());
            assert!(current.matches(second_ptr.as_ref()));
            current.cancel();
            assert!((*pane.get()).editor.as_ref().unwrap().cb.is_none());
            drop(previous);
            drop(pane);
            assert_eq!(current.pid(), -1);
            current.cancel();
        }
    }

    #[test]
    fn editor_without_a_session_releases_callbacks_without_starting() {
        unsafe {
            let client = client::new();
            let callback_payload = std::rc::Rc::new(());
            let observed = std::rc::Rc::downgrade(&callback_payload);
            let editor = spawn_editor(
                &client,
                |_| panic!("detached client must not write an editor file"),
                Some(Box::new(move |_, _| {
                    drop(callback_payload);
                    panic!("detached client must not complete an editor");
                })),
            );
            assert!(editor.is_none());
            assert!(observed.upgrade().is_none());
            assert_eq!(std::rc::Rc::strong_count(&client), 1);
        }
    }

    #[test]
    fn spawn_context_observes_models_without_retaining_them() {
        unsafe {
            let owner = client::new();
            let observer = std::rc::Rc::downgrade(&owner);
            let pane_owner = window_pane::new();
            let pane_observer = std::rc::Rc::downgrade(&pane_owner);
            let session_owner = session::new();
            let session_observer = std::rc::Rc::downgrade(&session_owner);
            let mut context = spawn_context {
                item: std::rc::Weak::new(),
                s: std::rc::Rc::downgrade(&session_owner),
                wl: refbox::Weak::new(),
                tc: std::rc::Rc::downgrade(&owner),
                wp0: std::rc::Rc::downgrade(&pane_owner),
                layout: None,
                name: None,
                argv: Vec::new(),
                environ: None,
                idx: 0,
                cwd: None,
                flags: 0,
            };
            let mut links = None;
            let wl = winlink_add(&mut links, 0);
            context.set_wl(wl.clone());
            assert_eq!(context.winlink_handle(), wl);
            winlink_remove(&mut links, wl.clone());
            assert!(!context.winlink_handle().is_alive());
            drop(owner);
            drop(pane_owner);
            drop(session_owner);
            assert!(session_observer.upgrade().is_none());
            assert!(context.s.upgrade().is_none());
            assert!(pane_observer.upgrade().is_none());
            assert!(context.wp0.upgrade().is_none());
            assert!(observer.upgrade().is_none());
            assert!(context.tc.upgrade().is_none());
            drop(context);
            assert!(observer.upgrade().is_none());
            assert!(pane_observer.upgrade().is_none());
            assert!(session_observer.upgrade().is_none());
        }
    }

    const CHILD_CASE: &str = "HMUX2_EDITOR_FD_OWNER_CASE";
    const SUCCESS_CASE: &str = "success";
    const SUCCESS_TEST: &str = "src::spawn::tests::editor_completion_reads_and_unlinks";

    fn run_isolated(test_name: &str, case: &str) {
        let test_exe = std::env::current_exe().expect("test executable");
        let mut child = Command::new(test_exe)
            .args(["--exact", test_name, "--nocapture"])
            .env(CHILD_CASE, case)
            .stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("spawn isolated editor ownership test");
        let deadline = Instant::now() + Duration::from_secs(5);
        let status = loop {
            match child.try_wait().expect("poll isolated editor test") {
                Some(status) => break status,
                None if Instant::now() >= deadline => {
                    let _ = child.kill();
                    let _ = child.wait();
                    panic!("isolated editor ownership test timed out");
                }
                None => thread::sleep(Duration::from_millis(10)),
            }
        };
        assert!(status.success(), "isolated test exited with {status}");
    }

    unsafe fn create_temp_file() -> (OwnedFd, std::ffi::CString) {
        let mut template = b"/tmp/hmux2-editor-owner-XXXXXX\0".to_vec();
        let fd = mkstemp(template.as_mut_ptr() as *mut ::core::ffi::c_char);
        assert!(
            fd >= 0,
            "mkstemp failed: {}",
            *strerror(*__errno_location())
        );
        (
            OwnedFd::from_raw_fd(fd),
            CStr::from_ptr(template.as_ptr() as *const ::core::ffi::c_char).to_owned(),
        )
    }

    fn capture_editor_result(result: refbox::Weak<Option<Vec<u8>>>) -> spawn_finish_edit_cb {
        Some(Box::new(move |_, value| {
            *result.try_borrow_mut().expect("live editor result") = value;
        }))
    }

    #[test]
    fn editor_completion_reads_and_unlinks() {
        if std::env::var(CHILD_CASE).as_deref() == Ok(SUCCESS_CASE) {
            unsafe {
                let (fd_owner, path) = create_temp_file();
                let fd = fd_owner.as_raw_fd();
                let flags = ::libc::fcntl(fd, ::libc::F_GETFL);
                assert!(flags >= 0);
                assert_eq!(flags & ::libc::O_ACCMODE, ::libc::O_RDWR);
                assert!(std::path::Path::new(path.to_str().unwrap()).exists());
                let file = spawn_editor_fdopen(fd_owner);
                assert!(!file.is_null());
                assert_eq!(::libc::fcntl(fd, ::libc::F_GETFL), flags);
                assert!(::libc::fcntl(fd, ::libc::F_GETFD) >= 0);
                let original = b"created through fdopen";
                assert_eq!(
                    fwrite(
                        original.as_ptr() as *const ::core::ffi::c_void,
                        original.len(),
                        1,
                        file,
                    ),
                    1
                );
                assert_eq!(fclose(file), 0);
                assert_eq!(::libc::fcntl(fd, ::libc::F_GETFD), -1);
                assert_eq!(fs::read(path.to_str().unwrap()).unwrap(), original);

                let edited = b"edited by child\0\xff";
                fs::write(path.to_str().unwrap(), edited).unwrap();
                let result = refbox::RefBox::new(None::<Vec<u8>>);
                let state = spawn_editor_state::new(
                    path.to_owned(),
                    capture_editor_result(result.downgrade()),
                );
                let pane_owner = window_pane::new();
                let wp = pane_owner.get();
                (*wp).editor = Some(state);
                (*wp).flags = PANE_STATUSREADY;
                (*wp).status = 0;
                spawn_editor_finish(&pane_owner);
                assert_eq!(
                    result.try_borrow_mut().expect("live editor result").take(),
                    Some(edited.to_vec())
                );
                assert!(!std::path::Path::new(path.to_str().unwrap()).exists());
                drop(pane_owner);

                let (fd_owner, path) = create_temp_file();
                drop(fd_owner);
                fs::write(path.to_str().unwrap(), []).unwrap();
                let result = refbox::RefBox::new(None::<Vec<u8>>);
                let state = spawn_editor_state::new(
                    path.to_owned(),
                    capture_editor_result(result.downgrade()),
                );
                let pane_owner = window_pane::new();
                let wp = pane_owner.get();
                (*wp).editor = Some(state);
                (*wp).flags = PANE_STATUSREADY;
                (*wp).status = 0;
                spawn_editor_finish(&pane_owner);
                assert_eq!(
                    result.try_borrow_mut().expect("live editor result").take(),
                    Some(Vec::new())
                );
                assert!(!std::path::Path::new(path.to_str().unwrap()).exists());
                drop(pane_owner);

                let (fd_owner, path) = create_temp_file();
                drop(fd_owner);
                fs::write(path.to_str().unwrap(), b"ignored after failure").unwrap();
                let result = refbox::RefBox::new(None::<Vec<u8>>);
                let state = spawn_editor_state::new(
                    path.to_owned(),
                    capture_editor_result(result.downgrade()),
                );
                let pane_owner = window_pane::new();
                let wp = pane_owner.get();
                (*wp).editor = Some(state);
                (*wp).flags = PANE_STATUSREADY;
                (*wp).status = 1 << 8;
                spawn_editor_finish(&pane_owner);
                assert_eq!(
                    result.try_borrow_mut().expect("live editor result").take(),
                    None
                );
                assert!(!std::path::Path::new(path.to_str().unwrap()).exists());
                drop(pane_owner);
            }
            return;
        }
        run_isolated(SUCCESS_TEST, SUCCESS_CASE);
    }
}
