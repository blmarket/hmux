use crate::src::cmd_parse::cmd_parse_and_append;
use crate::src::cmd_queue::{
    cmdq_append, cmdq_free_state, cmdq_get_callback1, cmdq_get_client, cmdq_guard, cmdq_new_state,
    cmdq_set_cancel_data,
};
use crate::src::ffi::libc::{__errno_location, close, free, memcpy, memset, poll, strcmp, strlen};
pub use crate::src::ffi::libc::{nfds_t, pollfd};
use crate::src::log::{fatalx, log_debug};
use crate::src::monitor::{monitor_add, monitor_create_client, monitor_destroy, monitor_remove};
use crate::src::reactor::{
    bufferevent_disable, bufferevent_enable, bufferevent_free, bufferevent_new,
    bufferevent_setwatermark, bufferevent_write, bufferevent_write_buffer, evbuffer_add,
    evbuffer_add_printf, evbuffer_free, evbuffer_get_length, evbuffer_new, evbuffer_pullup,
    evbuffer_read, evbuffer_readln,
};
use crate::src::server_client::server_client_set_exit_message;
pub use crate::src::shared::abi::ssize_t;
use crate::src::shared::abi::*;
pub use crate::src::shared::arguments::args;
use crate::src::shared::client::*;
pub use crate::src::shared::client::{
    client, client_entry, client_file, client_file_cb, client_file_entry, client_files,
    overlay_check_cb, overlay_draw_cb, overlay_free_cb, overlay_key_cb, overlay_mode_cb,
    overlay_resize_cb,
};
pub use crate::src::shared::client::{
    CLIENT_CONTROLCONTROL, CLIENT_CONTROL_DISCARD, CLIENT_CONTROL_NOOUTPUT,
    CLIENT_CONTROL_PAUSEAFTER, CLIENT_DEAD, CLIENT_EXIT, CLIENT_SUSPENDED, CLIENT_UNATTACHEDFLAGS,
};
use crate::src::shared::colour::*;
pub use crate::src::shared::command::cmd_parse_input;
pub use crate::src::shared::command::CMDQ_STATE_CONTROL;
use crate::src::shared::command::*;
use crate::src::shared::command::*;
pub use crate::src::shared::command::{
    cmd_find_state, cmd_list, cmdq_cb, cmdq_item, cmdq_list, cmdq_state, cmds,
};
pub use crate::src::shared::control::{
    control_block, control_block_all_entry, control_block_entry, control_line, control_line_entry,
    control_pane, control_pane_blocks, control_pane_entry, control_pane_pending_entry,
    control_panes, control_state, control_state_all_blocks, control_state_deferred,
    control_state_pending_list, control_window, control_window_entry, control_windows,
};
use crate::src::shared::display::*;
pub use crate::src::shared::display::{visible_range, visible_ranges};
pub use crate::src::shared::environment::environ;
pub use crate::src::shared::errno::{EAGAIN, EINTR};
use crate::src::shared::event::*;
pub use crate::src::shared::event::{
    evbuffer_eol_style, EVBUFFER_EOL_ANY, EVBUFFER_EOL_CRLF, EVBUFFER_EOL_CRLF_STRICT,
    EVBUFFER_EOL_LF, EVBUFFER_EOL_NUL, EV_READ, EV_WRITE,
};
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
pub use crate::src::shared::limits::SIZE_MAX;
pub use crate::src::shared::menu::menu_data;
use crate::src::shared::message::*;
pub use crate::src::shared::monitor::{monitor_cb, monitor_change, monitor_set};
pub use crate::src::shared::monitor::{
    monitor_type, MONITOR_ALL_PANES, MONITOR_ALL_WINDOWS, MONITOR_NOTIFY_INITIAL, MONITOR_PANE,
    MONITOR_SESSION, MONITOR_WINDOW,
};
pub use crate::src::shared::mouse::mouse_event;
pub use crate::src::shared::options::options;
pub use crate::src::shared::pane::{
    window_pane, window_pane_entry, window_pane_modes, window_pane_prompt, window_pane_sentry,
    window_pane_tree_entry, window_pane_zentry, window_panes,
};
pub use crate::src::shared::pane::{
    window_pane_offset, window_pane_resize, window_pane_resize_entry, window_pane_resizes,
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
pub use crate::src::shared::tree::{RB_BLACK, RB_NEGINF, RB_RED};
pub use crate::src::shared::tty::{tty, tty_code, tty_key, tty_term, tty_term_entry};
pub use crate::src::shared::variadic::{__builtin_va_list, __va_list_tag, va_list};
pub use crate::src::shared::window::{
    window, window_alerts_entry, window_entry, window_mode, window_mode_entry,
    window_mode_entry_entry, window_winlinks, winlink, winlink_entry, winlink_sentry,
    winlink_stack, winlink_wentry, winlinks,
};
use crate::src::tmux::{get_timer, setblocking};
use crate::src::window::{
    window_pane_find_by_id, window_pane_get_new_data, window_pane_update_used_data,
    winlink_find_by_window,
};
use crate::src::xmalloc::xvasprintf_cstring;
use std::ffi::CString;

pub use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_13;
pub use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_12;

pub const POLLIN: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const INFTIM: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
pub const CONTROL_PANE_OFF: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const CONTROL_PANE_PAUSED: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const CONTROL_BUFFER_LOW: ::core::ffi::c_int = 512 as ::core::ffi::c_int;
pub const CONTROL_BUFFER_HIGH: ::core::ffi::c_int = 8192 as ::core::ffi::c_int;
pub const CONTROL_WRITE_MINIMUM: ::core::ffi::c_int = 32 as ::core::ffi::c_int;
pub const CONTROL_MAXIMUM_AGE: ::core::ffi::c_int = 300000 as ::core::ffi::c_int;
pub const CONTROL_MAXIMUM_REPLY_BUFFER: ::core::ffi::c_int =
    64 as ::core::ffi::c_int * 1024 as ::core::ffi::c_int * 1024 as ::core::ffi::c_int;
pub const CONTROL_IGNORE_FLAGS: ::core::ffi::c_int =
    CLIENT_CONTROL_NOOUTPUT | CLIENT_UNATTACHEDFLAGS;
unsafe extern "C" fn control_pane_cmp(
    mut cp1: *mut control_pane,
    mut cp2: *mut control_pane,
) -> ::core::ffi::c_int {
    if (*cp1).pane < (*cp2).pane {
        return -(1 as ::core::ffi::c_int);
    }
    if (*cp1).pane > (*cp2).pane {
        return 1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}

unsafe extern "C" fn control_window_cmp(
    mut cw1: *mut control_window,
    mut cw2: *mut control_window,
) -> ::core::ffi::c_int {
    if (*cw1).window < (*cw2).window {
        return -(1 as ::core::ffi::c_int);
    }
    if (*cw1).window > (*cw2).window {
        return 1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}

// The C-layout records remain at the start of these private allocations.
// Their raw `line` fields borrow bytes owned by the adjacent CString.
#[repr(C)]
struct ControlBlockOwner {
    block: control_block,
    line: Option<CString>,
}
const _: () = assert!(std::mem::offset_of!(ControlBlockOwner, block) == 0);

impl ControlBlockOwner {
    fn new(line: Option<CString>, size: size_t) -> *mut control_block {
        let line_ptr = line
            .as_ref()
            .map_or(std::ptr::null_mut(), |line| line.as_ptr().cast_mut());
        Box::into_raw(Box::new(Self {
            block: control_block {
                size,
                line: line_ptr,
                t: 0,
                entry: control_block_entry {
                    tqe_next: std::ptr::null_mut(),
                    tqe_prev: std::ptr::null_mut(),
                },
                all_entry: control_block_all_entry {
                    tqe_next: std::ptr::null_mut(),
                    tqe_prev: std::ptr::null_mut(),
                },
            },
            line,
        }))
        .cast()
    }
}

#[repr(C)]
struct ControlLineOwner {
    record: control_line,
    line: CString,
}
const _: () = assert!(std::mem::offset_of!(ControlLineOwner, record) == 0);

impl ControlLineOwner {
    fn new(line: CString) -> *mut control_line {
        let line_ptr = line.as_ptr().cast_mut();
        Box::into_raw(Box::new(Self {
            record: control_line {
                line: line_ptr,
                entry: control_line_entry {
                    tqe_next: std::ptr::null_mut(),
                    tqe_prev: std::ptr::null_mut(),
                },
            },
            line,
        }))
        .cast()
    }
}

#[cfg(test)]
mod line_ownership_tests {
    use super::*;

    #[test]
    fn mixed_block_queue_releases_owned_reply_and_preserves_accounting() {
        unsafe {
            let mut state: control_state = std::mem::zeroed();
            state.all_blocks.tqh_last = &raw mut state.all_blocks.tqh_first;

            let output = ControlBlockOwner::new(None, 10);
            (*output).all_entry.tqe_prev = state.all_blocks.tqh_last;
            *state.all_blocks.tqh_last = output;
            state.all_blocks.tqh_last = &raw mut (*output).all_entry.tqe_next;

            let line = CString::new(b"reply-\xff".as_slice()).unwrap();
            let reply = ControlBlockOwner::new(Some(line), 0);
            (*reply).all_entry.tqe_prev = state.all_blocks.tqh_last;
            *state.all_blocks.tqh_last = reply;
            state.all_blocks.tqh_last = &raw mut (*reply).all_entry.tqe_next;
            state.queued_reply_bytes = 8;
            assert_eq!(
                std::ffi::CStr::from_ptr((*reply).line).to_bytes(),
                b"reply-\xff"
            );

            control_free_block(&mut state, output);
            assert_eq!(state.all_blocks.tqh_first, reply);
            assert_eq!(state.queued_reply_bytes, 8);
            control_free_block(&mut state, reply);
            assert!(state.all_blocks.tqh_first.is_null());
            assert_eq!(
                state.all_blocks.tqh_last,
                &raw mut state.all_blocks.tqh_first
            );
            assert_eq!(state.queued_reply_bytes, 0);
        }
    }

    #[test]
    fn deferred_line_transfer_keeps_bytes_alive() {
        unsafe {
            let deferred = ControlLineOwner::new(CString::new(b"notice-\xff".as_slice()).unwrap());
            let ControlLineOwner { line, .. } = *Box::from_raw(deferred.cast::<ControlLineOwner>());
            assert_eq!(line.as_bytes(), b"notice-\xff");
        }
    }
}

unsafe extern "C" fn control_free_block(mut cs: *mut control_state, mut cb: *mut control_block) {
    let mut size: size_t = 0;
    if (*cb).size == 0 as size_t && !(*cb).line.is_null() {
        size = (*cb.cast::<ControlBlockOwner>())
            .line
            .as_ref()
            .expect("reply block has an owned line")
            .as_bytes_with_nul()
            .len();
        if (*cs).queued_reply_bytes > size {
            (*cs).queued_reply_bytes = (*cs).queued_reply_bytes.wrapping_sub(size);
        } else {
            (*cs).queued_reply_bytes = 0 as size_t;
        }
    }
    if !(*cb).all_entry.tqe_next.is_null() {
        (*(*cb).all_entry.tqe_next).all_entry.tqe_prev = (*cb).all_entry.tqe_prev;
    } else {
        (*cs).all_blocks.tqh_last = (*cb).all_entry.tqe_prev;
    }
    *(*cb).all_entry.tqe_prev = (*cb).all_entry.tqe_next;
    drop(Box::from_raw(cb.cast::<ControlBlockOwner>()));
}
unsafe extern "C" fn control_get_pane(
    mut c: *mut client,
    mut wp: *mut window_pane,
) -> *mut control_pane {
    let mut cs: *mut control_state = (*c).control_state;
    let mut cp: control_pane = control_pane {
        pane: (*wp).id,
        offset: window_pane_offset { used: 0 },
        queued: window_pane_offset { used: 0 },
        flags: 0,
        pending_flag: 0,
        pending_entry: control_pane_pending_entry {
            tqe_next: ::core::ptr::null_mut::<control_pane>(),
            tqe_prev: ::core::ptr::null_mut::<*mut control_pane>(),
        },
        blocks: control_pane_blocks {
            tqh_first: ::core::ptr::null_mut::<control_block>(),
            tqh_last: ::core::ptr::null_mut::<*mut control_block>(),
        },
        entry: control_pane_entry {
            owner: std::ptr::null_mut(),
        },
    };
    return control_panes_find(&raw mut (*cs).panes, &raw mut cp);
}
unsafe extern "C" fn control_add_pane(
    mut c: *mut client,
    mut wp: *mut window_pane,
) -> *mut control_pane {
    let mut cs: *mut control_state = (*c).control_state;
    let mut cp: *mut control_pane = ::core::ptr::null_mut::<control_pane>();
    cp = control_get_pane(c, wp);
    if !cp.is_null() {
        return cp;
    }
    cp = Box::into_raw(Box::new(::core::mem::zeroed::<control_pane>()));
    (*cp).pane = (*wp).id;
    control_panes_insert(&raw mut (*cs).panes, cp);
    memcpy(
        &raw mut (*cp).offset as *mut ::core::ffi::c_void,
        &raw mut (*wp).offset as *const ::core::ffi::c_void,
        ::core::mem::size_of::<window_pane_offset>() as size_t,
    );
    memcpy(
        &raw mut (*cp).queued as *mut ::core::ffi::c_void,
        &raw mut (*wp).offset as *const ::core::ffi::c_void,
        ::core::mem::size_of::<window_pane_offset>() as size_t,
    );
    (*cp).blocks.tqh_first = ::core::ptr::null_mut::<control_block>();
    (*cp).blocks.tqh_last = &raw mut (*cp).blocks.tqh_first;
    return cp;
}
unsafe extern "C" fn control_get_window(
    mut c: *mut client,
    mut window: u_int,
) -> *mut control_window {
    let mut cs: *mut control_state = (*c).control_state;
    let mut cw: control_window = control_window {
        window: window,
        sx: 0,
        sy: 0,
        entry: control_window_entry {
            owner: std::ptr::null_mut(),
        },
    };
    if cs.is_null() {
        return ::core::ptr::null_mut::<control_window>();
    }
    return control_windows_find(&raw mut (*cs).windows, &raw mut cw);
}
#[no_mangle]
pub unsafe extern "C" fn control_set_window_size(
    mut c: *mut client,
    mut window: u_int,
    mut sx: u_int,
    mut sy: u_int,
) {
    let mut cs: *mut control_state = (*c).control_state;
    let mut cw: *mut control_window = ::core::ptr::null_mut::<control_window>();
    if cs.is_null() {
        return;
    }
    cw = control_get_window(c, window);
    if cw.is_null() {
        cw = Box::into_raw(Box::new(::core::mem::zeroed::<control_window>()));
        (*cw).window = window;
        control_windows_insert(&raw mut (*cs).windows, cw);
    }
    (*cw).sx = sx;
    (*cw).sy = sy;
}
#[no_mangle]
pub unsafe extern "C" fn control_get_window_size(
    mut c: *mut client,
    mut window: u_int,
    mut sx: *mut u_int,
    mut sy: *mut u_int,
) -> ::core::ffi::c_int {
    let mut cw: *mut control_window = ::core::ptr::null_mut::<control_window>();
    cw = control_get_window(c, window);
    if cw.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    *sx = (*cw).sx;
    *sy = (*cw).sy;
    return 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn control_clear_window_size(mut c: *mut client, mut window: u_int) {
    let mut cs: *mut control_state = (*c).control_state;
    let mut cw: *mut control_window = ::core::ptr::null_mut::<control_window>();
    if cs.is_null() {
        return;
    }
    cw = control_get_window(c, window);
    if !cw.is_null() {
        control_windows_remove(&raw mut (*cs).windows, cw);
        drop(Box::from_raw(cw));
    }
}
unsafe extern "C" fn control_discard_pane(mut c: *mut client, mut cp: *mut control_pane) {
    let mut cs: *mut control_state = (*c).control_state;
    let mut cb: *mut control_block = ::core::ptr::null_mut::<control_block>();
    let mut cb1: *mut control_block = ::core::ptr::null_mut::<control_block>();
    cb = (*cp).blocks.tqh_first;
    while !cb.is_null() && {
        cb1 = (*cb).entry.tqe_next;
        1 as ::core::ffi::c_int != 0
    } {
        if !(*cb).entry.tqe_next.is_null() {
            (*(*cb).entry.tqe_next).entry.tqe_prev = (*cb).entry.tqe_prev;
        } else {
            (*cp).blocks.tqh_last = (*cb).entry.tqe_prev;
        }
        *(*cb).entry.tqe_prev = (*cb).entry.tqe_next;
        control_free_block(cs, cb);
        cb = cb1;
    }
}
unsafe extern "C" fn control_window_pane(mut c: *mut client, mut pane: u_int) -> *mut window_pane {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    if (*c).session.is_null() {
        return ::core::ptr::null_mut::<window_pane>();
    }
    wp = window_pane_find_by_id(pane);
    if wp.is_null() {
        return ::core::ptr::null_mut::<window_pane>();
    }
    if winlink_find_by_window(
        &raw mut (*(*c).session).windows,
        (*wp).window as *mut window,
    )
    .is_null()
    {
        return ::core::ptr::null_mut::<window_pane>();
    }
    return wp;
}
#[no_mangle]
pub unsafe extern "C" fn control_reset_offsets(mut c: *mut client) {
    let mut cs: *mut control_state = (*c).control_state;
    let mut cp: *mut control_pane = ::core::ptr::null_mut::<control_pane>();
    let mut cp1: *mut control_pane = ::core::ptr::null_mut::<control_pane>();
    cp = control_panes_minmax(&raw mut (*cs).panes, RB_NEGINF);
    while !cp.is_null() && {
        cp1 = control_panes_next(cp);
        1 as ::core::ffi::c_int != 0
    } {
        control_discard_pane(c, cp);
        control_panes_remove(&raw mut (*cs).panes, cp);
        drop(Box::from_raw(cp));
        cp = cp1;
    }
    (*cs).pending_list.tqh_first = ::core::ptr::null_mut::<control_pane>();
    (*cs).pending_list.tqh_last = &raw mut (*cs).pending_list.tqh_first;
    (*cs).pending_count = 0 as u_int;
}
#[no_mangle]
pub unsafe extern "C" fn control_pane_offset(
    mut c: *mut client,
    mut wp: *mut window_pane,
    mut off: *mut ::core::ffi::c_int,
) -> *mut window_pane_offset {
    let mut cs: *mut control_state = (*c).control_state;
    let mut cp: *mut control_pane = ::core::ptr::null_mut::<control_pane>();
    if (*c).flags & CLIENT_CONTROL_NOOUTPUT as uint64_t != 0 {
        *off = 0 as ::core::ffi::c_int;
        return ::core::ptr::null_mut::<window_pane_offset>();
    }
    cp = control_get_pane(c, wp);
    if cp.is_null() || (*cp).flags & CONTROL_PANE_PAUSED != 0 {
        *off = 0 as ::core::ffi::c_int;
        return ::core::ptr::null_mut::<window_pane_offset>();
    }
    if (*cp).flags & CONTROL_PANE_OFF != 0 {
        *off = 1 as ::core::ffi::c_int;
        return ::core::ptr::null_mut::<window_pane_offset>();
    }
    *off = (evbuffer_get_length((*(*cs).write_event).output) >= CONTROL_BUFFER_LOW as size_t)
        as ::core::ffi::c_int;
    return &raw mut (*cp).offset;
}
#[no_mangle]
pub unsafe extern "C" fn control_set_pane_on(mut c: *mut client, mut wp: *mut window_pane) {
    let mut cp: *mut control_pane = ::core::ptr::null_mut::<control_pane>();
    cp = control_get_pane(c, wp);
    if !cp.is_null() && (*cp).flags & CONTROL_PANE_OFF != 0 {
        (*cp).flags &= !CONTROL_PANE_OFF;
        memcpy(
            &raw mut (*cp).offset as *mut ::core::ffi::c_void,
            &raw mut (*wp).offset as *const ::core::ffi::c_void,
            ::core::mem::size_of::<window_pane_offset>() as size_t,
        );
        memcpy(
            &raw mut (*cp).queued as *mut ::core::ffi::c_void,
            &raw mut (*wp).offset as *const ::core::ffi::c_void,
            ::core::mem::size_of::<window_pane_offset>() as size_t,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn control_set_pane_off(mut c: *mut client, mut wp: *mut window_pane) {
    let mut cp: *mut control_pane = ::core::ptr::null_mut::<control_pane>();
    cp = control_add_pane(c, wp);
    control_discard_pane(c, cp);
    memcpy(
        &raw mut (*cp).offset as *mut ::core::ffi::c_void,
        &raw mut (*wp).offset as *const ::core::ffi::c_void,
        ::core::mem::size_of::<window_pane_offset>() as size_t,
    );
    memcpy(
        &raw mut (*cp).queued as *mut ::core::ffi::c_void,
        &raw mut (*wp).offset as *const ::core::ffi::c_void,
        ::core::mem::size_of::<window_pane_offset>() as size_t,
    );
    (*cp).flags |= CONTROL_PANE_OFF;
}
#[no_mangle]
pub unsafe extern "C" fn control_continue_pane(mut c: *mut client, mut wp: *mut window_pane) {
    let mut cp: *mut control_pane = ::core::ptr::null_mut::<control_pane>();
    cp = control_get_pane(c, wp);
    if !cp.is_null() && (*cp).flags & CONTROL_PANE_PAUSED != 0 {
        (*cp).flags &= !CONTROL_PANE_PAUSED;
        memcpy(
            &raw mut (*cp).offset as *mut ::core::ffi::c_void,
            &raw mut (*wp).offset as *const ::core::ffi::c_void,
            ::core::mem::size_of::<window_pane_offset>() as size_t,
        );
        memcpy(
            &raw mut (*cp).queued as *mut ::core::ffi::c_void,
            &raw mut (*wp).offset as *const ::core::ffi::c_void,
            ::core::mem::size_of::<window_pane_offset>() as size_t,
        );
        control_notify_write(
            c,
            b"%%continue %%%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*wp).id,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn control_pause_pane(mut c: *mut client, mut wp: *mut window_pane) {
    let mut cp: *mut control_pane = ::core::ptr::null_mut::<control_pane>();
    cp = control_add_pane(c, wp);
    if !(*cp).flags & CONTROL_PANE_PAUSED != 0 {
        (*cp).flags |= CONTROL_PANE_PAUSED;
        control_discard_pane(c, cp);
        control_notify_write(
            c,
            b"%%pause %%%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*wp).id,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn control_reset_pane(mut c: *mut client, mut wp: *mut window_pane) {
    let mut cp: *mut control_pane = ::core::ptr::null_mut::<control_pane>();
    if (*c).control_state.is_null() {
        return;
    }
    cp = control_get_pane(c, wp);
    if cp.is_null() {
        return;
    }
    control_discard_pane(c, cp);
    memcpy(
        &raw mut (*cp).offset as *mut ::core::ffi::c_void,
        &raw mut (*wp).offset as *const ::core::ffi::c_void,
        ::core::mem::size_of::<window_pane_offset>() as size_t,
    );
    memcpy(
        &raw mut (*cp).queued as *mut ::core::ffi::c_void,
        &raw mut (*wp).offset as *const ::core::ffi::c_void,
        ::core::mem::size_of::<window_pane_offset>() as size_t,
    );
}
unsafe extern "C" fn control_check_reply_buffer(
    mut c: *mut client,
    mut added: size_t,
) -> ::core::ffi::c_int {
    let mut cs: *mut control_state = (*c).control_state;
    let mut size: size_t = 0;
    if (*c).flags as ::core::ffi::c_ulonglong & CLIENT_CONTROL_DISCARD != 0 {
        return 1 as ::core::ffi::c_int;
    }
    size = evbuffer_get_length((*(*cs).write_event).output);
    size = size.wrapping_add((*cs).queued_reply_bytes);
    size = size.wrapping_add(added);
    if size < CONTROL_MAXIMUM_REPLY_BUFFER as size_t {
        return 0 as ::core::ffi::c_int;
    }
    log_debug(
        b"%s: %s: %zu bytes of replies buffered\0" as *const u8 as *const ::core::ffi::c_char,
        b"control_check_reply_buffer\0" as *const u8 as *const ::core::ffi::c_char,
        (*c).name,
        size,
    );
    if !(*c).flags & CLIENT_EXIT as uint64_t != 0 {
        server_client_set_exit_message(c, Some(CString::new("too far behind").unwrap()));
        (*c).flags |= CLIENT_EXIT as uint64_t;
        control_discard(c);
    }
    (*c).flags = ((*c).flags as ::core::ffi::c_ulonglong | CLIENT_CONTROL_DISCARD) as uint64_t;
    return 1 as ::core::ffi::c_int;
}
unsafe fn control_write_line(c: *mut client, line: CString) {
    let mut cs: *mut control_state = (*c).control_state;
    let mut cb: *mut control_block = ::core::ptr::null_mut::<control_block>();
    let size = line.as_bytes_with_nul().len() as size_t;
    if control_check_reply_buffer(c, size) != 0 {
        return;
    }
    if (*cs).all_blocks.tqh_first.is_null() {
        log_debug(
            b"%s: %s: writing line: %s\0" as *const u8 as *const ::core::ffi::c_char,
            b"control_write_line\0" as *const u8 as *const ::core::ffi::c_char,
            (*c).name,
            line.as_ptr(),
        );
        bufferevent_write(
            (*cs).write_event,
            line.as_ptr() as *const ::core::ffi::c_void,
            size.wrapping_sub(1 as size_t),
        );
        bufferevent_write(
            (*cs).write_event,
            b"\n\0" as *const u8 as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
            1 as size_t,
        );
        bufferevent_enable((*cs).write_event, EV_WRITE as ::core::ffi::c_short);
        return;
    }
    cb = ControlBlockOwner::new(Some(line), 0);
    (*cb).all_entry.tqe_next = ::core::ptr::null_mut::<control_block>();
    (*cb).all_entry.tqe_prev = (*cs).all_blocks.tqh_last;
    *(*cs).all_blocks.tqh_last = cb;
    (*cs).all_blocks.tqh_last = &raw mut (*cb).all_entry.tqe_next;
    (*cs).queued_reply_bytes = (*cs).queued_reply_bytes.wrapping_add(size);
    (*cb).t = get_timer();
    log_debug(
        b"%s: %s: storing line: %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"control_write_line\0" as *const u8 as *const ::core::ffi::c_char,
        (*c).name,
        (*cb).line,
    );
    bufferevent_enable((*cs).write_event, EV_WRITE as ::core::ffi::c_short);
}
unsafe extern "C" fn control_flush_deferred(mut c: *mut client) {
    let mut cs: *mut control_state = (*c).control_state;
    let mut cl: *mut control_line = ::core::ptr::null_mut::<control_line>();
    let mut cl1: *mut control_line = ::core::ptr::null_mut::<control_line>();
    cl = (*cs).deferred.tqh_first;
    while !cl.is_null() && {
        cl1 = (*cl).entry.tqe_next;
        1 as ::core::ffi::c_int != 0
    } {
        if !(*cl).entry.tqe_next.is_null() {
            (*(*cl).entry.tqe_next).entry.tqe_prev = (*cl).entry.tqe_prev;
        } else {
            (*cs).deferred.tqh_last = (*cl).entry.tqe_prev;
        }
        *(*cl).entry.tqe_prev = (*cl).entry.tqe_next;
        let ControlLineOwner { line, .. } = *Box::from_raw(cl.cast::<ControlLineOwner>());
        control_write_line(c, line);
        cl = cl1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn control_write(
    mut c: *mut client,
    mut fmt: *const ::core::ffi::c_char,
    mut args: ...
) {
    let mut cs: *mut control_state = (*c).control_state;
    let mut ap: ::core::ffi::VaList;
    if cs.is_null() {
        return;
    }
    ap = args.clone();
    let line = xvasprintf_cstring(fmt, ap);
    control_write_line(c, line);
}
#[no_mangle]
pub unsafe extern "C" fn control_write_guard(
    mut c: *mut client,
    mut guard: *const ::core::ffi::c_char,
    mut t: ::core::ffi::c_long,
    mut number: u_int,
    mut flags: ::core::ffi::c_int,
) {
    let mut cs: *mut control_state = (*c).control_state;
    if cs.is_null() {
        return;
    }
    if strcmp(guard, b"begin\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        (*cs).guard_depth += 1;
    }
    control_write(
        c,
        b"%%%s %ld %u %d\0" as *const u8 as *const ::core::ffi::c_char,
        guard,
        t,
        number,
        flags,
    );
    if strcmp(guard, b"begin\0" as *const u8 as *const ::core::ffi::c_char)
        != 0 as ::core::ffi::c_int
        && (*cs).guard_depth > 0 as ::core::ffi::c_int
        && {
            (*cs).guard_depth -= 1;
            (*cs).guard_depth == 0 as ::core::ffi::c_int
        }
    {
        control_flush_deferred(c);
    }
}
#[no_mangle]
pub unsafe extern "C" fn control_notify_write(
    mut c: *mut client,
    mut fmt: *const ::core::ffi::c_char,
    mut args: ...
) {
    let mut cs: *mut control_state = (*c).control_state;
    let mut cl: *mut control_line = ::core::ptr::null_mut::<control_line>();
    let mut ap: ::core::ffi::VaList;
    if cs.is_null() {
        return;
    }
    ap = args.clone();
    let line = xvasprintf_cstring(fmt, ap);
    if (*cs).guard_depth == 0 as ::core::ffi::c_int {
        control_write_line(c, line);
        return;
    }
    log_debug(
        b"%s: %s: deferring notification: %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"control_notify_write\0" as *const u8 as *const ::core::ffi::c_char,
        (*c).name,
        line.as_ptr(),
    );
    cl = ControlLineOwner::new(line);
    (*cl).entry.tqe_next = ::core::ptr::null_mut::<control_line>();
    (*cl).entry.tqe_prev = (*cs).deferred.tqh_last;
    *(*cs).deferred.tqh_last = cl;
    (*cs).deferred.tqh_last = &raw mut (*cl).entry.tqe_next;
}
unsafe extern "C" fn control_check_age(
    mut c: *mut client,
    mut wp: *mut window_pane,
    mut cp: *mut control_pane,
) -> ::core::ffi::c_int {
    let mut cb: *mut control_block = ::core::ptr::null_mut::<control_block>();
    let mut t: uint64_t = 0;
    let mut age: uint64_t = 0;
    cb = (*cp).blocks.tqh_first;
    if cb.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    t = get_timer();
    if (*cb).t >= t {
        return 0 as ::core::ffi::c_int;
    }
    age = t.wrapping_sub((*cb).t);
    log_debug(
        b"%s: %s: %%%u is %llu behind\0" as *const u8 as *const ::core::ffi::c_char,
        b"control_check_age\0" as *const u8 as *const ::core::ffi::c_char,
        (*c).name,
        (*wp).id,
        age as ::core::ffi::c_ulonglong,
    );
    if (*c).flags as ::core::ffi::c_ulonglong & CLIENT_CONTROL_PAUSEAFTER != 0 {
        if age < (*c).pause_age as uint64_t {
            return 0 as ::core::ffi::c_int;
        }
        (*cp).flags |= CONTROL_PANE_PAUSED;
        control_discard_pane(c, cp);
        control_notify_write(
            c,
            b"%%pause %%%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*wp).id,
        );
    } else {
        if age < CONTROL_MAXIMUM_AGE as uint64_t {
            return 0 as ::core::ffi::c_int;
        }
        server_client_set_exit_message(c, Some(CString::new("too far behind").unwrap()));
        (*c).flags |= CLIENT_EXIT as uint64_t;
        control_discard(c);
    }
    return 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn control_write_output(mut c: *mut client, mut wp: *mut window_pane) {
    let mut cs: *mut control_state = (*c).control_state;
    let mut cp: *mut control_pane = ::core::ptr::null_mut::<control_pane>();
    let mut cb: *mut control_block = ::core::ptr::null_mut::<control_block>();
    let mut new_size: size_t = 0;
    if winlink_find_by_window(
        &raw mut (*(*c).session).windows,
        (*wp).window as *mut window,
    )
    .is_null()
    {
        return;
    }
    if (*c).flags & (CONTROL_IGNORE_FLAGS | CLIENT_EXIT) as uint64_t != 0 {
        cp = control_get_pane(c, wp);
        if cp.is_null() {
            return;
        }
    } else {
        cp = control_add_pane(c, wp);
        if !((*cp).flags & (CONTROL_PANE_OFF | CONTROL_PANE_PAUSED) != 0) {
            if control_check_age(c, wp, cp) != 0 {
                return;
            }
            window_pane_get_new_data(wp, &raw mut (*cp).queued, &raw mut new_size);
            if new_size == 0 as size_t {
                return;
            }
            window_pane_update_used_data(wp, &raw mut (*cp).queued, new_size);
            cb = ControlBlockOwner::new(None, new_size);
            (*cb).all_entry.tqe_next = ::core::ptr::null_mut::<control_block>();
            (*cb).all_entry.tqe_prev = (*cs).all_blocks.tqh_last;
            *(*cs).all_blocks.tqh_last = cb;
            (*cs).all_blocks.tqh_last = &raw mut (*cb).all_entry.tqe_next;
            (*cb).t = get_timer();
            (*cb).entry.tqe_next = ::core::ptr::null_mut::<control_block>();
            (*cb).entry.tqe_prev = (*cp).blocks.tqh_last;
            *(*cp).blocks.tqh_last = cb;
            (*cp).blocks.tqh_last = &raw mut (*cb).entry.tqe_next;
            log_debug(
                b"%s: %s: new output block of %zu for %%%u\0" as *const u8
                    as *const ::core::ffi::c_char,
                b"control_write_output\0" as *const u8 as *const ::core::ffi::c_char,
                (*c).name,
                (*cb).size,
                (*wp).id,
            );
            if (*cp).pending_flag == 0 {
                log_debug(
                    b"%s: %s: %%%u now pending\0" as *const u8 as *const ::core::ffi::c_char,
                    b"control_write_output\0" as *const u8 as *const ::core::ffi::c_char,
                    (*c).name,
                    (*wp).id,
                );
                (*cp).pending_entry.tqe_next = ::core::ptr::null_mut::<control_pane>();
                (*cp).pending_entry.tqe_prev = (*cs).pending_list.tqh_last;
                *(*cs).pending_list.tqh_last = cp;
                (*cs).pending_list.tqh_last = &raw mut (*cp).pending_entry.tqe_next;
                (*cp).pending_flag = 1 as ::core::ffi::c_int;
                (*cs).pending_count = (*cs).pending_count.wrapping_add(1);
            }
            bufferevent_enable((*cs).write_event, EV_WRITE as ::core::ffi::c_short);
            return;
        }
    }
    log_debug(
        b"%s: %s: ignoring pane %%%u\0" as *const u8 as *const ::core::ffi::c_char,
        b"control_write_output\0" as *const u8 as *const ::core::ffi::c_char,
        (*c).name,
        (*wp).id,
    );
    window_pane_update_used_data(wp, &raw mut (*cp).offset, SIZE_MAX as size_t);
    window_pane_update_used_data(wp, &raw mut (*cp).queued, SIZE_MAX as size_t);
}
unsafe extern "C" fn control_error(
    mut item: *mut cmdq_item,
    mut data: *mut ::core::ffi::c_void,
) -> cmd_retval {
    let mut c: *mut client = cmdq_get_client(item);
    let mut error: *mut ::core::ffi::c_char = data as *mut ::core::ffi::c_char;
    cmdq_guard(
        item,
        b"begin\0" as *const u8 as *const ::core::ffi::c_char,
        1 as ::core::ffi::c_int,
    );
    control_write(
        c,
        b"parse error: %s\0" as *const u8 as *const ::core::ffi::c_char,
        error,
    );
    cmdq_guard(
        item,
        b"error\0" as *const u8 as *const ::core::ffi::c_char,
        1 as ::core::ffi::c_int,
    );
    free(error as *mut ::core::ffi::c_void);
    return CMD_RETURN_NORMAL;
}

unsafe fn control_cancel_error(data: *mut ::core::ffi::c_void) {
    free(data);
}
unsafe extern "C" fn control_error_callback(
    mut bufev: *mut bufferevent,
    mut what: ::core::ffi::c_short,
    mut data: *mut ::core::ffi::c_void,
) {
    let mut c: *mut client = data as *mut client;
    (*c).flags |= CLIENT_EXIT as uint64_t;
}
unsafe extern "C" fn control_read_callback(
    mut bufev: *mut bufferevent,
    mut data: *mut ::core::ffi::c_void,
) {
    let mut c: *mut client = data as *mut client;
    let mut cs: *mut control_state = (*c).control_state;
    let mut buffer: *mut evbuffer = (*(*cs).read_event).input;
    let mut line: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut error: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut state: *mut cmdq_state = ::core::ptr::null_mut::<cmdq_state>();
    let mut status: cmd_parse_status = CMD_PARSE_ERROR;
    loop {
        line = evbuffer_readln(buffer, ::core::ptr::null_mut::<size_t>(), EVBUFFER_EOL_LF);
        if line.is_null() {
            break;
        }
        log_debug(
            b"%s: %s: %s\0" as *const u8 as *const ::core::ffi::c_char,
            b"control_read_callback\0" as *const u8 as *const ::core::ffi::c_char,
            (*c).name,
            line,
        );
        if *line as ::core::ffi::c_int == '\0' as i32 {
            free(line as *mut ::core::ffi::c_void);
            (*c).flags |= CLIENT_EXIT as uint64_t;
            break;
        } else {
            state = cmdq_new_state(
                ::core::ptr::null_mut::<cmd_find_state>(),
                ::core::ptr::null_mut::<key_event>(),
                CMDQ_STATE_CONTROL,
            );
            status = cmd_parse_and_append(
                line,
                ::core::ptr::null_mut::<cmd_parse_input>(),
                c,
                state,
                &raw mut error,
            );
            if status as ::core::ffi::c_uint
                == CMD_PARSE_ERROR as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                let error_item = cmdq_get_callback1(
                    b"control_error\0" as *const u8 as *const ::core::ffi::c_char,
                    Some(
                        control_error
                            as unsafe extern "C" fn(
                                *mut cmdq_item,
                                *mut ::core::ffi::c_void,
                            ) -> cmd_retval,
                    ),
                    error as *mut ::core::ffi::c_void,
                );
                cmdq_set_cancel_data(error_item, control_cancel_error);
                cmdq_append(c, error_item);
            }
            cmdq_free_state(state);
            free(line as *mut ::core::ffi::c_void);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn control_all_done(mut c: *mut client) -> ::core::ffi::c_int {
    let mut cs: *mut control_state = (*c).control_state;
    if !(*cs).all_blocks.tqh_first.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    return (evbuffer_get_length((*(*cs).write_event).output) == 0 as size_t) as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn control_wait_exit(mut fd: ::core::ffi::c_int) {
    let mut pfd: pollfd = pollfd {
        fd: 0,
        events: 0,
        revents: 0,
    };
    let mut evb: *mut evbuffer = ::core::ptr::null_mut::<evbuffer>();
    let mut line: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut n: ::core::ffi::c_int = 0;
    evb = evbuffer_new();
    if evb.is_null() {
        fatalx(b"out of memory\0" as *const u8 as *const ::core::ffi::c_char);
    }
    loop {
        line = evbuffer_readln(evb, ::core::ptr::null_mut::<size_t>(), EVBUFFER_EOL_LF);
        if !line.is_null() {
            if *line as ::core::ffi::c_int == '\0' as i32 {
                free(line as *mut ::core::ffi::c_void);
                break;
            } else {
                free(line as *mut ::core::ffi::c_void);
            }
        } else {
            memset(
                &raw mut pfd as *mut ::core::ffi::c_void,
                0 as ::core::ffi::c_int,
                ::core::mem::size_of::<pollfd>() as size_t,
            );
            pfd.fd = fd;
            pfd.events = POLLIN as ::core::ffi::c_short;
            if poll(&raw mut pfd, 1 as nfds_t, INFTIM) == -(1 as ::core::ffi::c_int) {
                if !(*__errno_location() == EINTR) {
                    break;
                }
            } else {
                n = evbuffer_read(evb, fd, -(1 as ::core::ffi::c_int));
                if n == 0 as ::core::ffi::c_int {
                    break;
                }
                if n == -(1 as ::core::ffi::c_int)
                    && *__errno_location() != EAGAIN
                    && *__errno_location() != EINTR
                {
                    break;
                }
            }
        }
    }
    evbuffer_free(evb);
}
unsafe extern "C" fn control_flush_all_blocks(mut c: *mut client) {
    let mut cs: *mut control_state = (*c).control_state;
    let mut cb: *mut control_block = ::core::ptr::null_mut::<control_block>();
    let mut cb1: *mut control_block = ::core::ptr::null_mut::<control_block>();
    cb = (*cs).all_blocks.tqh_first;
    while !cb.is_null() && {
        cb1 = (*cb).all_entry.tqe_next;
        1 as ::core::ffi::c_int != 0
    } {
        if (*cb).size != 0 as size_t {
            break;
        }
        log_debug(
            b"%s: %s: flushing line: %s\0" as *const u8 as *const ::core::ffi::c_char,
            b"control_flush_all_blocks\0" as *const u8 as *const ::core::ffi::c_char,
            (*c).name,
            (*cb).line,
        );
        bufferevent_write(
            (*cs).write_event,
            (*cb).line as *const ::core::ffi::c_void,
            strlen((*cb).line),
        );
        bufferevent_write(
            (*cs).write_event,
            b"\n\0" as *const u8 as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
            1 as size_t,
        );
        control_free_block(cs, cb);
        cb = cb1;
    }
}
unsafe extern "C" fn control_append_data(
    mut c: *mut client,
    mut cp: *mut control_pane,
    mut age: uint64_t,
    mut message: *mut evbuffer,
    mut wp: *mut window_pane,
    mut size: size_t,
) -> *mut evbuffer {
    let mut new_data: *mut u_char = ::core::ptr::null_mut::<u_char>();
    let mut new_size: size_t = 0;
    let mut start: size_t = 0;
    let mut i: u_int = 0;
    if message.is_null() {
        message = evbuffer_new();
        if message.is_null() {
            fatalx(b"out of memory\0" as *const u8 as *const ::core::ffi::c_char);
        }
        if (*c).flags as ::core::ffi::c_ulonglong & CLIENT_CONTROL_PAUSEAFTER != 0 {
            evbuffer_add_printf(
                message,
                b"%%extended-output %%%u %llu : \0" as *const u8 as *const ::core::ffi::c_char,
                (*wp).id,
                age as ::core::ffi::c_ulonglong,
            );
        } else {
            evbuffer_add_printf(
                message,
                b"%%output %%%u \0" as *const u8 as *const ::core::ffi::c_char,
                (*wp).id,
            );
        }
    }
    new_data =
        window_pane_get_new_data(wp, &raw mut (*cp).offset, &raw mut new_size) as *mut u_char;
    if new_size < size {
        fatalx(
            b"not enough data: %zu < %zu\0" as *const u8 as *const ::core::ffi::c_char,
            new_size,
            size,
        );
    }
    i = 0 as u_int;
    while (i as size_t) < size {
        if (*new_data.offset(i as isize) as ::core::ffi::c_int) < ' ' as i32
            || *new_data.offset(i as isize) as ::core::ffi::c_int == '\\' as i32
        {
            evbuffer_add_printf(
                message,
                b"\\%03o\0" as *const u8 as *const ::core::ffi::c_char,
                *new_data.offset(i as isize) as ::core::ffi::c_int,
            );
        } else {
            start = i as size_t;
            while (i.wrapping_add(1 as u_int) as size_t) < size
                && *new_data.offset(i.wrapping_add(1 as u_int) as isize) as ::core::ffi::c_int
                    >= ' ' as i32
                && *new_data.offset(i.wrapping_add(1 as u_int) as isize) as ::core::ffi::c_int
                    != '\\' as i32
            {
                i = i.wrapping_add(1);
            }
            evbuffer_add(
                message,
                new_data.offset(start as isize) as *const ::core::ffi::c_void,
                (i as size_t).wrapping_sub(start).wrapping_add(1 as size_t),
            );
        }
        i = i.wrapping_add(1);
    }
    window_pane_update_used_data(wp, &raw mut (*cp).offset, size);
    return message;
}
unsafe extern "C" fn control_write_data(mut c: *mut client, mut message: *mut evbuffer) {
    let mut cs: *mut control_state = (*c).control_state;
    log_debug(
        b"%s: %s: %.*s\0" as *const u8 as *const ::core::ffi::c_char,
        b"control_write_data\0" as *const u8 as *const ::core::ffi::c_char,
        (*c).name,
        evbuffer_get_length(message) as ::core::ffi::c_int,
        evbuffer_pullup(message, -(1 as ::core::ffi::c_int) as ssize_t),
    );
    evbuffer_add(
        message,
        b"\n\0" as *const u8 as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
        1 as size_t,
    );
    bufferevent_write_buffer((*cs).write_event, message);
    evbuffer_free(message);
}
unsafe extern "C" fn control_write_pending(
    mut c: *mut client,
    mut cp: *mut control_pane,
    mut limit: size_t,
) -> ::core::ffi::c_int {
    let mut cs: *mut control_state = (*c).control_state;
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut message: *mut evbuffer = ::core::ptr::null_mut::<evbuffer>();
    let mut used: size_t = 0 as size_t;
    let mut size: size_t = 0;
    let mut cb: *mut control_block = ::core::ptr::null_mut::<control_block>();
    let mut cb1: *mut control_block = ::core::ptr::null_mut::<control_block>();
    let mut age: uint64_t = 0;
    let mut t: uint64_t = get_timer();
    wp = control_window_pane(c, (*cp).pane);
    if wp.is_null() || (*wp).fd == -(1 as ::core::ffi::c_int) {
        cb = (*cp).blocks.tqh_first;
        while !cb.is_null() && {
            cb1 = (*cb).entry.tqe_next;
            1 as ::core::ffi::c_int != 0
        } {
            if !(*cb).entry.tqe_next.is_null() {
                (*(*cb).entry.tqe_next).entry.tqe_prev = (*cb).entry.tqe_prev;
            } else {
                (*cp).blocks.tqh_last = (*cb).entry.tqe_prev;
            }
            *(*cb).entry.tqe_prev = (*cb).entry.tqe_next;
            control_free_block(cs, cb);
            cb = cb1;
        }
        control_flush_all_blocks(c);
        return 0 as ::core::ffi::c_int;
    }
    while used != limit && !(*cp).blocks.tqh_first.is_null() {
        if control_check_age(c, wp, cp) != 0 {
            if !message.is_null() {
                evbuffer_free(message);
            }
            message = ::core::ptr::null_mut::<evbuffer>();
            break;
        } else {
            cb = (*cp).blocks.tqh_first;
            if (*cb).t < t {
                age = t.wrapping_sub((*cb).t);
            } else {
                age = 0 as uint64_t;
            }
            log_debug(
                b"%s: %s: output block %zu (age %llu) for %%%u (used %zu/%zu)\0" as *const u8
                    as *const ::core::ffi::c_char,
                b"control_write_pending\0" as *const u8 as *const ::core::ffi::c_char,
                (*c).name,
                (*cb).size,
                age as ::core::ffi::c_ulonglong,
                (*cp).pane,
                used,
                limit,
            );
            size = (*cb).size;
            if size > limit.wrapping_sub(used) {
                size = limit.wrapping_sub(used);
            }
            used = used.wrapping_add(size);
            message = control_append_data(c, cp, age, message, wp, size);
            (*cb).size = (*cb).size.wrapping_sub(size);
            if (*cb).size == 0 as size_t {
                if !(*cb).entry.tqe_next.is_null() {
                    (*(*cb).entry.tqe_next).entry.tqe_prev = (*cb).entry.tqe_prev;
                } else {
                    (*cp).blocks.tqh_last = (*cb).entry.tqe_prev;
                }
                *(*cb).entry.tqe_prev = (*cb).entry.tqe_next;
                control_free_block(cs, cb);
                cb = (*cs).all_blocks.tqh_first;
                if !cb.is_null() && (*cb).size == 0 as size_t {
                    if !wp.is_null() && !message.is_null() {
                        control_write_data(c, message);
                        message = ::core::ptr::null_mut::<evbuffer>();
                    }
                    control_flush_all_blocks(c);
                }
            }
        }
    }
    if !message.is_null() {
        control_write_data(c, message);
    }
    return !(*cp).blocks.tqh_first.is_null() as ::core::ffi::c_int;
}
unsafe extern "C" fn control_write_callback(
    mut bufev: *mut bufferevent,
    mut data: *mut ::core::ffi::c_void,
) {
    let mut c: *mut client = data as *mut client;
    let mut cs: *mut control_state = (*c).control_state;
    let mut cp: *mut control_pane = ::core::ptr::null_mut::<control_pane>();
    let mut cp1: *mut control_pane = ::core::ptr::null_mut::<control_pane>();
    let mut evb: *mut evbuffer = (*(*cs).write_event).output;
    let mut space: size_t = 0;
    let mut limit: size_t = 0;
    control_flush_all_blocks(c);
    while evbuffer_get_length(evb) < CONTROL_BUFFER_HIGH as size_t {
        if (*cs).pending_count == 0 as u_int {
            break;
        }
        space = (CONTROL_BUFFER_HIGH as size_t).wrapping_sub(evbuffer_get_length(evb));
        log_debug(
            b"%s: %s: %zu bytes available, %u panes\0" as *const u8 as *const ::core::ffi::c_char,
            b"control_write_callback\0" as *const u8 as *const ::core::ffi::c_char,
            (*c).name,
            space,
            (*cs).pending_count,
        );
        limit = space
            .wrapping_div((*cs).pending_count as size_t)
            .wrapping_div(3 as size_t);
        if limit < CONTROL_WRITE_MINIMUM as size_t {
            limit = CONTROL_WRITE_MINIMUM as size_t;
        }
        cp = (*cs).pending_list.tqh_first;
        while !cp.is_null() && {
            cp1 = (*cp).pending_entry.tqe_next;
            1 as ::core::ffi::c_int != 0
        } {
            if evbuffer_get_length(evb) >= CONTROL_BUFFER_HIGH as size_t {
                break;
            }
            if !(control_write_pending(c, cp, limit) != 0) {
                if !(*cp).pending_entry.tqe_next.is_null() {
                    (*(*cp).pending_entry.tqe_next).pending_entry.tqe_prev =
                        (*cp).pending_entry.tqe_prev;
                } else {
                    (*cs).pending_list.tqh_last = (*cp).pending_entry.tqe_prev;
                }
                *(*cp).pending_entry.tqe_prev = (*cp).pending_entry.tqe_next;
                (*cp).pending_flag = 0 as ::core::ffi::c_int;
                (*cs).pending_count = (*cs).pending_count.wrapping_sub(1);
            }
            cp = cp1;
        }
    }
    if evbuffer_get_length(evb) == 0 as size_t {
        bufferevent_disable((*cs).write_event, EV_WRITE as ::core::ffi::c_short);
    }
}
unsafe extern "C" fn control_sub_change(
    mut change: *mut monitor_change,
    mut data: *mut ::core::ffi::c_void,
) {
    let mut c: *mut client = (*change).c;
    let mut s: *mut session = (*change).s;
    let mut wl: *mut winlink = (*change).wl;
    let mut wp: *mut window_pane = (*change).wp;
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    if !wp.is_null() {
        w = (*wp).window as *mut window;
        control_notify_write(
            c,
            b"%%subscription-changed %s $%u @%u %u %%%u : %s\0" as *const u8
                as *const ::core::ffi::c_char,
            (*change).name,
            (*s).id,
            (*w).id,
            (*wl).idx,
            (*wp).id,
            (*change).value,
        );
    } else if !wl.is_null() {
        w = (*wl).window;
        control_notify_write(
            c,
            b"%%subscription-changed %s $%u @%u %u - : %s\0" as *const u8
                as *const ::core::ffi::c_char,
            (*change).name,
            (*s).id,
            (*w).id,
            (*wl).idx,
            (*change).value,
        );
    } else {
        control_notify_write(
            c,
            b"%%subscription-changed %s $%u - - - : %s\0" as *const u8
                as *const ::core::ffi::c_char,
            (*change).name,
            (*s).id,
            (*change).value,
        );
    };
}
#[no_mangle]
pub unsafe extern "C" fn control_start(mut c: *mut client) {
    let mut cs: *mut control_state = ::core::ptr::null_mut::<control_state>();
    if (*c).flags & CLIENT_CONTROLCONTROL as uint64_t != 0 {
        close((*c).out_fd);
        (*c).out_fd = -(1 as ::core::ffi::c_int);
    } else {
        setblocking((*c).out_fd, 0 as ::core::ffi::c_int);
    }
    setblocking((*c).fd, 0 as ::core::ffi::c_int);
    (*c).control_state = Box::into_raw(Box::new(::core::mem::zeroed::<control_state>()));
    cs = (*c).control_state;
    (*cs).panes.storage = std::ptr::null_mut();
    (*cs).windows.storage = std::ptr::null_mut();
    (*cs).pending_list.tqh_first = ::core::ptr::null_mut::<control_pane>();
    (*cs).pending_list.tqh_last = &raw mut (*cs).pending_list.tqh_first;
    (*cs).all_blocks.tqh_first = ::core::ptr::null_mut::<control_block>();
    (*cs).all_blocks.tqh_last = &raw mut (*cs).all_blocks.tqh_first;
    (*cs).deferred.tqh_first = ::core::ptr::null_mut::<control_line>();
    (*cs).deferred.tqh_last = &raw mut (*cs).deferred.tqh_first;
    (*cs).subs = monitor_create_client(
        c,
        Some(
            control_sub_change
                as unsafe extern "C" fn(*mut monitor_change, *mut ::core::ffi::c_void) -> (),
        ),
        NULL,
    ) as *mut monitor_set;
    (*cs).read_event = bufferevent_new(
        (*c).fd,
        Some(
            control_read_callback
                as unsafe extern "C" fn(*mut bufferevent, *mut ::core::ffi::c_void) -> (),
        ),
        Some(
            control_write_callback
                as unsafe extern "C" fn(*mut bufferevent, *mut ::core::ffi::c_void) -> (),
        ),
        Some(
            control_error_callback
                as unsafe extern "C" fn(
                    *mut bufferevent,
                    ::core::ffi::c_short,
                    *mut ::core::ffi::c_void,
                ) -> (),
        ),
        c as *mut ::core::ffi::c_void,
    );
    if (*cs).read_event.is_null() {
        fatalx(b"out of memory\0" as *const u8 as *const ::core::ffi::c_char);
    }
    if (*c).flags & CLIENT_CONTROLCONTROL as uint64_t != 0 {
        (*cs).write_event = (*cs).read_event;
    } else {
        (*cs).write_event = bufferevent_new(
            (*c).out_fd,
            None,
            Some(
                control_write_callback
                    as unsafe extern "C" fn(*mut bufferevent, *mut ::core::ffi::c_void) -> (),
            ),
            Some(
                control_error_callback
                    as unsafe extern "C" fn(
                        *mut bufferevent,
                        ::core::ffi::c_short,
                        *mut ::core::ffi::c_void,
                    ) -> (),
            ),
            c as *mut ::core::ffi::c_void,
        );
        if (*cs).write_event.is_null() {
            fatalx(b"out of memory\0" as *const u8 as *const ::core::ffi::c_char);
        }
    }
    bufferevent_setwatermark(
        (*cs).write_event,
        EV_WRITE as ::core::ffi::c_short,
        CONTROL_BUFFER_LOW as size_t,
        0 as size_t,
    );
    if (*c).flags & CLIENT_CONTROLCONTROL as uint64_t != 0 {
        bufferevent_write(
            (*cs).write_event,
            b"\x1BP1000p\0" as *const u8 as *const ::core::ffi::c_char
                as *const ::core::ffi::c_void,
            7 as size_t,
        );
        bufferevent_enable((*cs).write_event, EV_WRITE as ::core::ffi::c_short);
    }
}
#[no_mangle]
pub unsafe extern "C" fn control_ready(mut c: *mut client) {
    bufferevent_enable(
        (*(*c).control_state).read_event,
        EV_READ as ::core::ffi::c_short,
    );
}
#[no_mangle]
pub unsafe extern "C" fn control_discard(mut c: *mut client) {
    let mut cs: *mut control_state = (*c).control_state;
    let mut cp: *mut control_pane = ::core::ptr::null_mut::<control_pane>();
    cp = control_panes_minmax(&raw mut (*cs).panes, RB_NEGINF);
    while !cp.is_null() {
        control_discard_pane(c, cp);
        cp = control_panes_next(cp);
    }
    bufferevent_disable((*cs).read_event, EV_READ as ::core::ffi::c_short);
}
#[no_mangle]
pub unsafe extern "C" fn control_discard_all(mut c: *mut client) {
    let mut cs: *mut control_state = (*c).control_state;
    let mut cb: *mut control_block = ::core::ptr::null_mut::<control_block>();
    let mut cb1: *mut control_block = ::core::ptr::null_mut::<control_block>();
    control_discard(c);
    cb = (*cs).all_blocks.tqh_first;
    while !cb.is_null() && {
        cb1 = (*cb).all_entry.tqe_next;
        1 as ::core::ffi::c_int != 0
    } {
        control_free_block(cs, cb);
        cb = cb1;
    }
    (*cs).queued_reply_bytes = 0 as size_t;
    bufferevent_disable((*cs).write_event, EV_WRITE as ::core::ffi::c_short);
}
#[no_mangle]
pub unsafe extern "C" fn control_stop(mut c: *mut client) {
    let mut cs: *mut control_state = (*c).control_state;
    let mut cb: *mut control_block = ::core::ptr::null_mut::<control_block>();
    let mut cb1: *mut control_block = ::core::ptr::null_mut::<control_block>();
    let mut cw: *mut control_window = ::core::ptr::null_mut::<control_window>();
    let mut cw1: *mut control_window = ::core::ptr::null_mut::<control_window>();
    let mut cl: *mut control_line = ::core::ptr::null_mut::<control_line>();
    let mut cl1: *mut control_line = ::core::ptr::null_mut::<control_line>();
    if cs.is_null() {
        return;
    }
    monitor_destroy((*cs).subs);
    cl = (*cs).deferred.tqh_first;
    while !cl.is_null() && {
        cl1 = (*cl).entry.tqe_next;
        1 as ::core::ffi::c_int != 0
    } {
        if !(*cl).entry.tqe_next.is_null() {
            (*(*cl).entry.tqe_next).entry.tqe_prev = (*cl).entry.tqe_prev;
        } else {
            (*cs).deferred.tqh_last = (*cl).entry.tqe_prev;
        }
        *(*cl).entry.tqe_prev = (*cl).entry.tqe_next;
        drop(Box::from_raw(cl.cast::<ControlLineOwner>()));
        cl = cl1;
    }
    if !(*c).flags & CLIENT_CONTROLCONTROL as uint64_t != 0 {
        bufferevent_free((*cs).write_event);
    }
    bufferevent_free((*cs).read_event);
    control_reset_offsets(c);
    cw = control_windows_minmax(&raw mut (*cs).windows, RB_NEGINF);
    while !cw.is_null() && {
        cw1 = control_windows_next(cw);
        1 as ::core::ffi::c_int != 0
    } {
        control_windows_remove(&raw mut (*cs).windows, cw);
        drop(Box::from_raw(cw));
        cw = cw1;
    }
    cb = (*cs).all_blocks.tqh_first;
    while !cb.is_null() && {
        cb1 = (*cb).all_entry.tqe_next;
        1 as ::core::ffi::c_int != 0
    } {
        control_free_block(cs, cb);
        cb = cb1;
    }
    (*c).control_state = ::core::ptr::null_mut::<control_state>();
    drop(Box::from_raw(cs));
}
#[no_mangle]
pub unsafe extern "C" fn control_add_sub(
    mut c: *mut client,
    mut name: *const ::core::ffi::c_char,
    mut type_0: monitor_type,
    mut id: ::core::ffi::c_int,
    mut format: *const ::core::ffi::c_char,
) {
    let mut cs: *mut control_state = (*c).control_state;
    monitor_add((*cs).subs, name, type_0, id, format, MONITOR_NOTIFY_INITIAL);
}
#[no_mangle]
pub unsafe extern "C" fn control_remove_sub(
    mut c: *mut client,
    mut name: *const ::core::ffi::c_char,
) {
    let mut cs: *mut control_state = (*c).control_state;
    monitor_remove((*cs).subs, name);
}

unsafe fn control_panes_key(elm: *mut control_pane) -> u32 {
    (*elm).pane
}
pub unsafe fn control_panes_find(
    head: *mut control_panes,
    elm: *mut control_pane,
) -> *mut control_pane {
    let Some(map) = (*head).storage.as_ref() else {
        return std::ptr::null_mut();
    };
    let key = control_panes_key(elm);
    map.get(&key).copied().unwrap_or(std::ptr::null_mut())
}
pub unsafe fn control_panes_nfind(
    head: *mut control_panes,
    elm: *mut control_pane,
) -> *mut control_pane {
    let Some(map) = (*head).storage.as_ref() else {
        return std::ptr::null_mut();
    };
    let key = control_panes_key(elm);
    map.range((std::ops::Bound::Included(&key), std::ops::Bound::Unbounded))
        .next()
        .map_or(std::ptr::null_mut(), |(_, node)| *node)
}
pub unsafe fn control_panes_insert(
    head: *mut control_panes,
    elm: *mut control_pane,
) -> *mut control_pane {
    let key = control_panes_key(elm);
    if (*head).storage.is_null() {
        (*head).storage = Box::into_raw(Box::new(std::collections::BTreeMap::new()));
    }
    let map = &mut *(*head).storage;
    match map.entry(key) {
        std::collections::btree_map::Entry::Occupied(entry) => return *entry.get(),
        std::collections::btree_map::Entry::Vacant(entry) => {
            entry.insert(elm);
        }
    }
    (*elm).entry.owner = map as *mut _;
    std::ptr::null_mut()
}
pub unsafe fn control_panes_remove(
    head: *mut control_panes,
    elm: *mut control_pane,
) -> *mut control_pane {
    if elm.is_null() {
        return std::ptr::null_mut();
    }
    let key = control_panes_key(elm);
    let Some(map) = (*head).storage.as_mut() else {
        return std::ptr::null_mut();
    };
    if map.get(&key).copied() != Some(elm) {
        return std::ptr::null_mut();
    }
    map.remove(&key);
    (*elm).entry.owner = std::ptr::null_mut();
    if map.is_empty() {
        drop(Box::from_raw((*head).storage));
        (*head).storage = std::ptr::null_mut();
    }
    elm
}
pub unsafe fn control_panes_minmax(
    head: *mut control_panes,
    direction: ::core::ffi::c_int,
) -> *mut control_pane {
    let Some(map) = (*head).storage.as_ref() else {
        return std::ptr::null_mut();
    };
    let pair = if direction < 0 {
        map.first_key_value()
    } else {
        map.last_key_value()
    };
    pair.map_or(std::ptr::null_mut(), |(_, node)| *node)
}
pub unsafe fn control_panes_next(elm: *mut control_pane) -> *mut control_pane {
    let Some(map) = (*elm).entry.owner.as_ref() else {
        return std::ptr::null_mut();
    };
    let key = control_panes_key(elm);
    map.range((std::ops::Bound::Excluded(&key), std::ops::Bound::Unbounded))
        .next()
        .map_or(std::ptr::null_mut(), |(_, node)| *node)
}
pub unsafe fn control_panes_prev(elm: *mut control_pane) -> *mut control_pane {
    let Some(map) = (*elm).entry.owner.as_ref() else {
        return std::ptr::null_mut();
    };
    let key = control_panes_key(elm);
    map.range((std::ops::Bound::Unbounded, std::ops::Bound::Excluded(&key)))
        .next_back()
        .map_or(std::ptr::null_mut(), |(_, node)| *node)
}

unsafe fn control_windows_key(elm: *mut control_window) -> u32 {
    (*elm).window
}
pub unsafe fn control_windows_find(
    head: *mut control_windows,
    elm: *mut control_window,
) -> *mut control_window {
    let Some(map) = (*head).storage.as_ref() else {
        return std::ptr::null_mut();
    };
    let key = control_windows_key(elm);
    map.get(&key).copied().unwrap_or(std::ptr::null_mut())
}
pub unsafe fn control_windows_nfind(
    head: *mut control_windows,
    elm: *mut control_window,
) -> *mut control_window {
    let Some(map) = (*head).storage.as_ref() else {
        return std::ptr::null_mut();
    };
    let key = control_windows_key(elm);
    map.range((std::ops::Bound::Included(&key), std::ops::Bound::Unbounded))
        .next()
        .map_or(std::ptr::null_mut(), |(_, node)| *node)
}
pub unsafe fn control_windows_insert(
    head: *mut control_windows,
    elm: *mut control_window,
) -> *mut control_window {
    let key = control_windows_key(elm);
    if (*head).storage.is_null() {
        (*head).storage = Box::into_raw(Box::new(std::collections::BTreeMap::new()));
    }
    let map = &mut *(*head).storage;
    match map.entry(key) {
        std::collections::btree_map::Entry::Occupied(entry) => return *entry.get(),
        std::collections::btree_map::Entry::Vacant(entry) => {
            entry.insert(elm);
        }
    }
    (*elm).entry.owner = map as *mut _;
    std::ptr::null_mut()
}
pub unsafe fn control_windows_remove(
    head: *mut control_windows,
    elm: *mut control_window,
) -> *mut control_window {
    if elm.is_null() {
        return std::ptr::null_mut();
    }
    let key = control_windows_key(elm);
    let Some(map) = (*head).storage.as_mut() else {
        return std::ptr::null_mut();
    };
    if map.get(&key).copied() != Some(elm) {
        return std::ptr::null_mut();
    }
    map.remove(&key);
    (*elm).entry.owner = std::ptr::null_mut();
    if map.is_empty() {
        drop(Box::from_raw((*head).storage));
        (*head).storage = std::ptr::null_mut();
    }
    elm
}
pub unsafe fn control_windows_minmax(
    head: *mut control_windows,
    direction: ::core::ffi::c_int,
) -> *mut control_window {
    let Some(map) = (*head).storage.as_ref() else {
        return std::ptr::null_mut();
    };
    let pair = if direction < 0 {
        map.first_key_value()
    } else {
        map.last_key_value()
    };
    pair.map_or(std::ptr::null_mut(), |(_, node)| *node)
}
pub unsafe fn control_windows_next(elm: *mut control_window) -> *mut control_window {
    let Some(map) = (*elm).entry.owner.as_ref() else {
        return std::ptr::null_mut();
    };
    let key = control_windows_key(elm);
    map.range((std::ops::Bound::Excluded(&key), std::ops::Bound::Unbounded))
        .next()
        .map_or(std::ptr::null_mut(), |(_, node)| *node)
}
pub unsafe fn control_windows_prev(elm: *mut control_window) -> *mut control_window {
    let Some(map) = (*elm).entry.owner.as_ref() else {
        return std::ptr::null_mut();
    };
    let key = control_windows_key(elm);
    map.range((std::ops::Bound::Unbounded, std::ops::Bound::Excluded(&key)))
        .next_back()
        .map_or(std::ptr::null_mut(), |(_, node)| *node)
}
