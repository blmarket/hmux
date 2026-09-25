use crate::src::cmd::parse::cmd_parse_and_append;
use crate::src::cmd::queue::{
    cmdq_append, cmdq_free_state, cmdq_get_callback1, cmdq_get_client, cmdq_guard, cmdq_new_state,
    cmdq_set_cancel_data,
};
use crate::src::ffi::libc::{__errno_location, close, free, memcpy, memset, poll, strcmp, strlen};
use crate::src::ffi::libc::{nfds_t, pollfd};
use crate::src::log::{fatalx, log_debug};
use crate::src::monitor::{monitor_add, monitor_create_client, monitor_destroy, monitor_remove};
use crate::src::reactor::{
    bufferevent_disable, bufferevent_enable, bufferevent_free, bufferevent_new,
    bufferevent_setwatermark, bufferevent_write, bufferevent_write_buffer, evbuffer_add,
    evbuffer_add_printf, evbuffer_free, evbuffer_get_length, evbuffer_new, evbuffer_pullup,
    evbuffer_read, evbuffer_readln,
};
use crate::src::server_client::server_client_set_exit_message;
use crate::src::shared::abi::ssize_t;
use crate::src::shared::abi::*;
use crate::src::shared::arguments::args;
use crate::src::shared::client::*;
use crate::src::shared::client::{
    client, client_file, client_file_cb, client_file_entry, client_files, overlay_check_cb,
    overlay_draw_cb, overlay_free_cb, overlay_key_cb, overlay_mode_cb, overlay_resize_cb,
};
use crate::src::shared::client::{
    CLIENT_CONTROLCONTROL, CLIENT_CONTROL_DISCARD, CLIENT_CONTROL_NOOUTPUT,
    CLIENT_CONTROL_PAUSEAFTER, CLIENT_DEAD, CLIENT_EXIT, CLIENT_SUSPENDED, CLIENT_UNATTACHEDFLAGS,
};
use crate::src::shared::colour::*;
use crate::src::shared::command::cmd_parse_input;
use crate::src::shared::command::CMDQ_STATE_CONTROL;
use crate::src::shared::command::*;
use crate::src::shared::command::*;
use crate::src::shared::command::{
    cmd_find_state, cmd_list, cmdq_cb, cmdq_item, cmdq_list, cmdq_state, cmds,
};
use crate::src::shared::control::{
    control_block, control_pane, control_pane_entry, control_panes, control_state, control_window,
    control_window_entry, control_windows,
};
use crate::src::shared::display::*;
use crate::src::shared::display::{visible_range, visible_ranges};
use crate::src::shared::environment::environ;
use crate::src::shared::errno::{EAGAIN, EINTR};
use crate::src::shared::event::*;
use crate::src::shared::event::{
    evbuffer_eol_style, EVBUFFER_EOL_ANY, EVBUFFER_EOL_CRLF, EVBUFFER_EOL_CRLF_STRICT,
    EVBUFFER_EOL_LF, EVBUFFER_EOL_NUL, EV_READ, EV_WRITE,
};
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
use crate::src::shared::layout::{layout_cell, layout_cell_entry, layout_cells};
use crate::src::shared::limits::SIZE_MAX;
use crate::src::shared::menu::menu_data;
use crate::src::shared::message::*;
use crate::src::shared::monitor::{monitor_cb, monitor_change, monitor_set};
use crate::src::shared::monitor::{
    monitor_type, MONITOR_ALL_PANES, MONITOR_ALL_WINDOWS, MONITOR_NOTIFY_INITIAL, MONITOR_PANE,
    MONITOR_SESSION, MONITOR_WINDOW,
};
use crate::src::shared::mouse::mouse_event;
use crate::src::shared::options::options;
use crate::src::shared::pane::{
    window_pane, window_pane_modes, window_pane_prompt, window_pane_tree_entry, window_panes,
};
use crate::src::shared::pane::{window_pane_offset, window_pane_resize, window_pane_resizes};
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
use crate::src::shared::tree::{RB_BLACK, RB_NEGINF, RB_RED};
use crate::src::shared::tty::{tty, tty_code, tty_key, tty_term, tty_term_entry};
use crate::src::shared::variadic::{__builtin_va_list, __va_list_tag, va_list};
use crate::src::shared::window::{
    window, window_entry, window_mode, window_mode_entry, window_winlinks, winlink, winlink_entry,
    winlink_stack, winlinks,
};
use crate::src::tmux::{get_timer, setblocking};
use crate::src::window::{
    window_pane_find_by_id, window_pane_get_new_data, window_pane_update_used_data,
    winlink_find_by_window,
};
use crate::src::xmalloc::xvasprintf_cstring;
use std::collections::VecDeque;
use std::ffi::CString;

use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_13;
use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_12;

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

impl control_block {
    fn new(line: Option<CString>, size: size_t) -> Box<Self> {
        Box::new(control_block {
            size: size,
            line: line,
            t: 0,
        })
    }
}

impl control_state {
    fn new() -> Self {
        control_state {
            deferred: VecDeque::new(),
            all_blocks: VecDeque::new(),
            pending_panes: VecDeque::new(),
            ..control_state::empty()
        }
    }

    fn add_block(&mut self, owner: Box<control_block>) -> *mut control_block {
        let block = &*owner as *const control_block as *mut control_block;
        self.all_blocks.push_back(owner);
        block
    }

    fn block(&self, index: usize) -> *mut control_block {
        self.all_blocks
            .get(index)
            .map_or(std::ptr::null_mut(), |owner| {
                &**owner as *const control_block as *mut control_block
            })
    }

    fn remove_block(&mut self, block: *mut control_block) {
        let index = self
            .all_blocks
            .iter()
            .position(|owner| std::ptr::eq(&**owner, block))
            .expect("control block must be owned by its state");
        drop(
            self.all_blocks
                .remove(index)
                .expect("located control block"),
        );
    }

    fn pending_snapshot(&self) -> Vec<*mut control_pane> {
        self.pending_panes.iter().copied().collect()
    }

    fn remove_pending(&mut self, pane: *mut control_pane) -> bool {
        let Some(index) = self.pending_panes.iter().position(|queued| *queued == pane) else {
            return false;
        };
        self.pending_panes.remove(index).is_some()
    }
}

unsafe fn control_state_owner(cs: *mut control_state) -> *mut control_state {
    cs.cast()
}

#[doc(hidden)]
/// Allocate the owner backing a control client's state pointer.
pub fn control_state_new() -> *mut control_state {
    let owner = Box::into_raw(Box::new(control_state::new()));
    owner
}

#[doc(hidden)]
/// Release an allocated state after its external resources and indexes are gone.
///
/// # Safety
/// `cs` must be a live pointer returned by `control_state_new`, and it must not
/// be used after this call. Callers must first release its events and monitor
/// set and empty its raw-pointer pane and window indexes.
pub unsafe fn control_state_free(cs: *mut control_state) {
    if !cs.is_null() {
        drop(Box::from_raw(control_state_owner(cs)));
    }
}

unsafe fn control_first_block(cs: *mut control_state) -> *mut control_block {
    (*control_state_owner(cs)).block(0)
}

unsafe fn control_first_pane_block(cp: *mut control_pane) -> *mut control_block {
    (*cp)
        .blocks
        .front()
        .copied()
        .unwrap_or(std::ptr::null_mut())
}

unsafe fn control_remove_pane_block(cp: *mut control_pane, block: *mut control_block) {
    let index = (*cp)
        .blocks
        .iter()
        .position(|candidate| *candidate == block)
        .expect("control pane block must be queued on its pane");
    (*cp).blocks.remove(index).expect("located pane block");
}

unsafe fn control_add_block(
    cs: *mut control_state,
    owner: Box<control_block>,
) -> *mut control_block {
    (*control_state_owner(cs)).add_block(owner)
}

#[cfg(test)]
mod control_queue_tests {
    use super::*;

    #[test]
    fn state_block_owner_preserves_order_addresses_and_reply_accounting() {
        unsafe {
            let mut owner = control_state::new();
            let cs = &raw mut owner;

            let output = control_add_block(cs, control_block::new(None, 10));
            let reply_text = CString::new(b"reply-\xff".as_slice()).unwrap();
            let reply = control_add_block(cs, control_block::new(Some(reply_text), 0));
            (*cs).queued_reply_bytes = 8;
            for size in 1..=64 {
                control_add_block(cs, control_block::new(None, size));
            }

            assert_eq!(control_first_block(cs), output);
            assert_eq!((*control_state_owner(cs)).block(1), reply);
            assert_eq!(
                std::ffi::CStr::from_ptr(((*reply).line).as_ref().map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut())).to_bytes(),
                b"reply-\xff"
            );

            control_free_block(cs, output);
            assert_eq!(control_first_block(cs), reply);
            assert_eq!((*cs).queued_reply_bytes, 8);
            control_free_block(cs, reply);
            assert_eq!((*control_state_owner(cs)).block(0), control_first_block(cs));
            assert_eq!((*cs).queued_reply_bytes, 0);
        }
    }

    #[test]
    fn deferred_line_transfer_keeps_bytes_alive() {
        let mut owner = control_state::new();
        owner
            .deferred
            .push_back(CString::new(b"first-\xff".as_slice()).unwrap());
        owner.deferred.push_back(CString::new("second").unwrap());
        let line = owner.deferred.pop_front().unwrap();
        assert_eq!(line.as_bytes(), b"first-\xff");
        let line = owner.deferred.pop_front().unwrap();
        assert_eq!(line.as_bytes(), b"second");
    }

    #[test]
    fn pane_block_handles_keep_order_and_addresses_while_state_owns_blocks() {
        unsafe {
            let mut owner = control_state::new();
            let cs = &raw mut owner;
            let first = control_add_block(cs, control_block::new(None, 12));
            let middle = control_add_block(cs, control_block::new(None, 23));
            let last = control_add_block(cs, control_block::new(None, 34));
            let mut pane = control_pane {
                pane: 7,
                offset: window_pane_offset { used: 0 },
                queued: window_pane_offset { used: 0 },
                flags: 0,
                pending_flag: 0,
                blocks: VecDeque::from([first, middle, last]),
                entry: control_pane_entry {
                    owner: std::ptr::null_mut(),
                },
            };
            let pane_ptr = &raw mut pane;

            assert_eq!(control_first_pane_block(pane_ptr), first);
            control_remove_pane_block(pane_ptr, middle);
            control_free_block(cs, middle);
            assert_eq!(control_first_pane_block(pane_ptr), first);
            assert_eq!((*last).size, 34);
            assert_eq!((*control_state_owner(cs)).block(0), first);
            assert_eq!((*control_state_owner(cs)).block(1), last);
            control_remove_pane_block(pane_ptr, first);
            control_free_block(cs, first);
            assert_eq!(control_first_pane_block(pane_ptr), last);
            assert_eq!(control_first_block(cs), last);
            control_remove_pane_block(pane_ptr, last);
            control_free_block(cs, last);
            assert!(control_first_block(cs).is_null());
        }
    }

    #[test]
    fn pending_snapshot_defers_reentrant_appends_to_the_next_pass() {
        let mut owner = control_state::new();
        let mut panes: Vec<Box<control_pane>> = (0..3)
            .map(|pane| {
                Box::new(control_pane {
                    pane,
                    offset: window_pane_offset { used: 0 },
                    queued: window_pane_offset { used: 0 },
                    flags: 0,
                    pending_flag: 1,
                    blocks: VecDeque::new(),
                    entry: control_pane_entry {
                        owner: std::ptr::null_mut(),
                    },
                })
            })
            .collect();
        let pointers: Vec<_> = panes.iter_mut().map(|pane| &raw mut **pane).collect();
        owner.pending_panes.extend(pointers[..2].iter().copied());

        let pass = owner.pending_snapshot();
        owner.pending_panes.push_back(pointers[2]);
        assert_eq!(pass, pointers[..2]);
        assert!(owner.remove_pending(pointers[0]));
        assert!(!owner.remove_pending(pointers[0]));
        assert_eq!(owner.pending_snapshot(), pointers[1..]);
    }

    #[test]
    fn pane_index_owns_pane_boxes_and_keeps_addresses_stable() {
        unsafe {
            fn pane(id: u_int) -> *mut control_pane {
                Box::into_raw(Box::new(control_pane {
                    pane: id,
                    offset: window_pane_offset { used: 0 },
                    queued: window_pane_offset { used: 0 },
                    flags: 0,
                    pending_flag: 0,
                    blocks: VecDeque::new(),
                    entry: control_pane_entry {
                        owner: std::ptr::null_mut(),
                    },
                }))
            }

            let mut index = control_panes {
                storage: std::ptr::null_mut(),
            };
            let first = pane(4);
            let second = pane(9);
            assert!(control_panes_insert(&raw mut index, first).is_null());
            assert!(control_panes_insert(&raw mut index, second).is_null());
            assert_eq!(control_panes_find(&raw mut index, first), first);
            assert_eq!(control_panes_next(first), second);
            assert_eq!(control_panes_prev(second), first);

            drop(control_panes_remove(&raw mut index, first));
            assert_eq!(control_panes_minmax(&raw mut index, RB_NEGINF), second);
            drop(control_panes_remove(&raw mut index, second));
            assert!(index.storage.is_null());
        }
    }
}

unsafe extern "C" fn control_free_block(mut cs: *mut control_state, mut cb: *mut control_block) {
    let mut size: size_t = 0;
    if (*cb).size == 0 as size_t && !(*cb).line.is_none() {
        size = (*cb)
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
    (*control_state_owner(cs)).remove_block(cb);
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
        blocks: VecDeque::new(),
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
    cp = Box::into_raw(Box::new(control_pane {
        pane: 0,
        offset: window_pane_offset { used: 0 },
        queued: window_pane_offset { used: 0 },
        flags: 0,
        pending_flag: 0,
        blocks: VecDeque::new(),
        entry: control_pane_entry {
            owner: std::ptr::null_mut(),
        },
    }));
    (*cp).pane = (*wp).id;
    let existing = control_panes_insert(&raw mut (*cs).panes, cp);
    if !existing.is_null() {
        return existing;
    }
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
    while let Some(cb) = (*cp).blocks.pop_front() {
        control_free_block(cs, cb);
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
        drop(control_panes_remove(&raw mut (*cs).panes, cp));
        cp = cp1;
    }
    (*control_state_owner(cs)).pending_panes.clear();
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
    *off = (evbuffer_get_length(&*((*(*cs).write_event).output)) >= CONTROL_BUFFER_LOW as size_t)
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
    size = evbuffer_get_length(&*((*(*cs).write_event).output));
    size = size.wrapping_add((*cs).queued_reply_bytes);
    size = size.wrapping_add(added);
    if size < CONTROL_MAXIMUM_REPLY_BUFFER as size_t {
        return 0 as ::core::ffi::c_int;
    }
    log_debug(
        b"%s: %s: %zu bytes of replies buffered\0" as *const u8 as *const ::core::ffi::c_char,
        b"control_check_reply_buffer\0" as *const u8 as *const ::core::ffi::c_char,
        ((*c).name).as_ref().map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
        size,
    );
    if !(*c).flags & CLIENT_EXIT as uint64_t != 0 {
        server_client_set_exit_message(&mut *c, Some(CString::new("too far behind").unwrap()));
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
    if control_first_block(cs).is_null() {
        log_debug(
            b"%s: %s: writing line: %s\0" as *const u8 as *const ::core::ffi::c_char,
            b"control_write_line\0" as *const u8 as *const ::core::ffi::c_char,
            ((*c).name).as_ref().map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
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
    cb = control_add_block(cs, control_block::new(Some(line), 0));
    (*cs).queued_reply_bytes = (*cs).queued_reply_bytes.wrapping_add(size);
    (*cb).t = get_timer();
    log_debug(
        b"%s: %s: storing line: %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"control_write_line\0" as *const u8 as *const ::core::ffi::c_char,
        ((*c).name).as_ref().map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
        ((*cb).line).as_ref().map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
    );
    bufferevent_enable((*cs).write_event, EV_WRITE as ::core::ffi::c_short);
}
unsafe extern "C" fn control_flush_deferred(mut c: *mut client) {
    let mut cs: *mut control_state = (*c).control_state;
    while let Some(line) = (*control_state_owner(cs)).deferred.pop_front() {
        control_write_line(c, line);
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
        ((*c).name).as_ref().map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
        line.as_ptr(),
    );
    (*control_state_owner(cs)).deferred.push_back(line);
}
unsafe extern "C" fn control_check_age(
    mut c: *mut client,
    mut wp: *mut window_pane,
    mut cp: *mut control_pane,
) -> ::core::ffi::c_int {
    let mut cb: *mut control_block = ::core::ptr::null_mut::<control_block>();
    let mut t: uint64_t = 0;
    let mut age: uint64_t = 0;
    cb = control_first_pane_block(cp);
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
        ((*c).name).as_ref().map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
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
        server_client_set_exit_message(&mut *c, Some(CString::new("too far behind").unwrap()));
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
            cb = control_add_block(cs, control_block::new(None, new_size));
            (*cb).t = get_timer();
            (*cp).blocks.push_back(cb);
            log_debug(
                b"%s: %s: new output block of %zu for %%%u\0" as *const u8
                    as *const ::core::ffi::c_char,
                b"control_write_output\0" as *const u8 as *const ::core::ffi::c_char,
                ((*c).name).as_ref().map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
                (*cb).size,
                (*wp).id,
            );
            if (*cp).pending_flag == 0 {
                log_debug(
                    b"%s: %s: %%%u now pending\0" as *const u8 as *const ::core::ffi::c_char,
                    b"control_write_output\0" as *const u8 as *const ::core::ffi::c_char,
                    ((*c).name).as_ref().map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
                    (*wp).id,
                );
                (*control_state_owner(cs)).pending_panes.push_back(cp);
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
        ((*c).name).as_ref().map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
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
    let error = Box::from_raw(data as *mut Option<CString>);
    cmdq_guard(
        item,
        b"begin\0" as *const u8 as *const ::core::ffi::c_char,
        1 as ::core::ffi::c_int,
    );
    control_write(
        c,
        b"parse error: %s\0" as *const u8 as *const ::core::ffi::c_char,
        error
            .as_ref()
            .as_ref()
            .map_or(::core::ptr::null(), |cause| cause.as_ptr()),
    );
    cmdq_guard(
        item,
        b"error\0" as *const u8 as *const ::core::ffi::c_char,
        1 as ::core::ffi::c_int,
    );
    drop(error);
    return CMD_RETURN_NORMAL;
}

unsafe fn control_cancel_error(data: *mut ::core::ffi::c_void) {
    drop(Box::from_raw(data as *mut Option<CString>));
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
    let mut state: *mut cmdq_state = ::core::ptr::null_mut::<cmdq_state>();
    loop {
        let Some(line) = evbuffer_readln(
            buffer,
            ::core::ptr::null_mut::<size_t>(),
            EVBUFFER_EOL_LF,
        ) else {
            break;
        };
        log_debug(
            b"%s: %s: %s\0" as *const u8 as *const ::core::ffi::c_char,
            b"control_read_callback\0" as *const u8 as *const ::core::ffi::c_char,
            ((*c).name).as_ref().map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
            line.as_ptr().cast::<::core::ffi::c_char>(),
        );
        if line[0] == 0 {
            (*c).flags |= CLIENT_EXIT as uint64_t;
            break;
        } else {
            state = cmdq_new_state(
                ::core::ptr::null_mut::<cmd_find_state>(),
                ::core::ptr::null_mut::<key_event>(),
                CMDQ_STATE_CONTROL,
            );
            match cmd_parse_and_append(
                line.as_ptr().cast::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<cmd_parse_input>(),
                c,
                state,
            ) {
                Err(error) => {
                    let error_item = cmdq_get_callback1(
                        b"control_error\0" as *const u8 as *const ::core::ffi::c_char,
                        Some(
                            control_error
                                as unsafe extern "C" fn(
                                    *mut cmdq_item,
                                    *mut ::core::ffi::c_void,
                                ) -> cmd_retval,
                        ),
                        Box::into_raw(Box::new(error)) as *mut ::core::ffi::c_void,
                    );
                    cmdq_set_cancel_data(&mut *error_item, control_cancel_error);
                    cmdq_append(c, error_item);
                }
                Ok(_) => {}
            }
            cmdq_free_state(state);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn control_all_done(mut c: *mut client) -> ::core::ffi::c_int {
    let mut cs: *mut control_state = (*c).control_state;
    if !control_first_block(cs).is_null() {
        return 0 as ::core::ffi::c_int;
    }
    return (evbuffer_get_length(&*((*(*cs).write_event).output)) == 0 as size_t) as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn control_wait_exit(mut fd: ::core::ffi::c_int) {
    let mut pfd: pollfd = pollfd {
        fd: 0,
        events: 0,
        revents: 0,
    };
    let mut evb: *mut evbuffer = ::core::ptr::null_mut::<evbuffer>();
    let mut n: ::core::ffi::c_int = 0;
    evb = evbuffer_new();
    if evb.is_null() {
        fatalx(b"out of memory\0" as *const u8 as *const ::core::ffi::c_char);
    }
    loop {
        if let Some(line) = evbuffer_readln(
            evb,
            ::core::ptr::null_mut::<size_t>(),
            EVBUFFER_EOL_LF,
        ) {
            if line[0] == 0 {
                break;
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
    loop {
        let cb = control_first_block(cs);
        if cb.is_null() || (*cb).size != 0 as size_t {
            break;
        }
        log_debug(
            b"%s: %s: flushing line: %s\0" as *const u8 as *const ::core::ffi::c_char,
            b"control_flush_all_blocks\0" as *const u8 as *const ::core::ffi::c_char,
            ((*c).name).as_ref().map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
            ((*cb).line).as_ref().map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
        );
        bufferevent_write(
            (*cs).write_event,
            ((*cb).line).as_ref().map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()) as *const ::core::ffi::c_void,
            strlen(((*cb).line).as_ref().map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut())),
        );
        bufferevent_write(
            (*cs).write_event,
            b"\n\0" as *const u8 as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
            1 as size_t,
        );
        control_free_block(cs, cb);
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
        ((*c).name).as_ref().map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
        evbuffer_get_length(&*(message)) as ::core::ffi::c_int,
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
    let mut age: uint64_t = 0;
    let mut t: uint64_t = get_timer();
    wp = control_window_pane(c, (*cp).pane);
    if wp.is_null() || (*wp).fd == -(1 as ::core::ffi::c_int) {
        while let Some(cb) = (*cp).blocks.pop_front() {
            control_free_block(cs, cb);
        }
        control_flush_all_blocks(c);
        return 0 as ::core::ffi::c_int;
    }
    while used != limit && !(*cp).blocks.is_empty() {
        if control_check_age(c, wp, cp) != 0 {
            if !message.is_null() {
                evbuffer_free(message);
            }
            message = ::core::ptr::null_mut::<evbuffer>();
            break;
        } else {
            cb = control_first_pane_block(cp);
            if (*cb).t < t {
                age = t.wrapping_sub((*cb).t);
            } else {
                age = 0 as uint64_t;
            }
            log_debug(
                b"%s: %s: output block %zu (age %llu) for %%%u (used %zu/%zu)\0" as *const u8
                    as *const ::core::ffi::c_char,
                b"control_write_pending\0" as *const u8 as *const ::core::ffi::c_char,
                ((*c).name).as_ref().map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
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
                control_remove_pane_block(cp, cb);
                control_free_block(cs, cb);
                cb = control_first_block(cs);
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
    return !(*cp).blocks.is_empty() as ::core::ffi::c_int;
}
unsafe extern "C" fn control_write_callback(
    mut bufev: *mut bufferevent,
    mut data: *mut ::core::ffi::c_void,
) {
    let mut c: *mut client = data as *mut client;
    let mut cs: *mut control_state = (*c).control_state;
    let mut evb: *mut evbuffer = (*(*cs).write_event).output;
    let mut space: size_t = 0;
    let mut limit: size_t = 0;
    control_flush_all_blocks(c);
    while evbuffer_get_length(&*(evb)) < CONTROL_BUFFER_HIGH as size_t {
        if (*cs).pending_count == 0 as u_int {
            break;
        }
        space = (CONTROL_BUFFER_HIGH as size_t).wrapping_sub(evbuffer_get_length(&*(evb)));
        log_debug(
            b"%s: %s: %zu bytes available, %u panes\0" as *const u8 as *const ::core::ffi::c_char,
            b"control_write_callback\0" as *const u8 as *const ::core::ffi::c_char,
            ((*c).name).as_ref().map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
            space,
            (*cs).pending_count,
        );
        limit = space
            .wrapping_div((*cs).pending_count as size_t)
            .wrapping_div(3 as size_t);
        if limit < CONTROL_WRITE_MINIMUM as size_t {
            limit = CONTROL_WRITE_MINIMUM as size_t;
        }
        let pending = (*control_state_owner(cs)).pending_snapshot();
        for cp in pending {
            if evbuffer_get_length(&*(evb)) >= CONTROL_BUFFER_HIGH as size_t {
                break;
            }
            if !(*control_state_owner(cs))
                .pending_panes
                .iter()
                .any(|pane| *pane == cp)
            {
                continue;
            }
            if !(control_write_pending(c, cp, limit) != 0) {
                let owner = &mut *control_state_owner(cs);
                if owner.remove_pending(cp) {
                    (*cp).pending_flag = 0 as ::core::ffi::c_int;
                    (*cs).pending_count = (*cs).pending_count.wrapping_sub(1);
                }
            }
        }
    }
    if evbuffer_get_length(&*(evb)) == 0 as size_t {
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
    cs = control_state_new();
    (*c).control_state = cs;
    (*cs).panes.storage = std::ptr::null_mut();
    (*cs).windows.storage = std::ptr::null_mut();
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
    control_discard(c);
    loop {
        let cb = control_first_block(cs);
        if cb.is_null() {
            break;
        }
        control_free_block(cs, cb);
    }
    (*cs).queued_reply_bytes = 0 as size_t;
    bufferevent_disable((*cs).write_event, EV_WRITE as ::core::ffi::c_short);
}
#[no_mangle]
pub unsafe extern "C" fn control_stop(mut c: *mut client) {
    let mut cs: *mut control_state = (*c).control_state;
    let mut cw: *mut control_window = ::core::ptr::null_mut::<control_window>();
    let mut cw1: *mut control_window = ::core::ptr::null_mut::<control_window>();
    if cs.is_null() {
        return;
    }
    monitor_destroy((*cs).subs);
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
    loop {
        let cb = control_first_block(cs);
        if cb.is_null() {
            break;
        }
        control_free_block(cs, cb);
    }
    (*c).control_state = ::core::ptr::null_mut::<control_state>();
    control_state_free(cs);
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

fn control_panes_key(elm: &control_pane) -> u32 {
    elm.pane
}
pub unsafe fn control_panes_find(
    head: *mut control_panes,
    elm: *mut control_pane,
) -> *mut control_pane {
    let Some(map) = (*head).storage.as_ref() else {
        return std::ptr::null_mut();
    };
    let key = control_panes_key(&*elm);
    map.get(&key).map_or(std::ptr::null_mut(), |node| {
        &**node as *const control_pane as *mut control_pane
    })
}
pub unsafe fn control_panes_nfind(
    head: *mut control_panes,
    elm: *mut control_pane,
) -> *mut control_pane {
    let Some(map) = (*head).storage.as_ref() else {
        return std::ptr::null_mut();
    };
    let key = control_panes_key(&*elm);
    map.range((std::ops::Bound::Included(&key), std::ops::Bound::Unbounded))
        .next()
        .map_or(std::ptr::null_mut(), |(_, node)| {
            &**node as *const control_pane as *mut control_pane
        })
}
pub unsafe fn control_panes_insert(
    head: *mut control_panes,
    elm: *mut control_pane,
) -> *mut control_pane {
    let key = control_panes_key(&*elm);
    if (*head).storage.is_null() {
        (*head).storage = Box::into_raw(Box::new(std::collections::BTreeMap::new()));
    }
    let map = &mut *(*head).storage;
    match map.entry(key) {
        std::collections::btree_map::Entry::Occupied(entry) => {
            drop(Box::from_raw(elm));
            return &**entry.get() as *const control_pane as *mut control_pane;
        }
        std::collections::btree_map::Entry::Vacant(entry) => {
            entry.insert(Box::from_raw(elm));
        }
    }
    (*elm).entry.owner = map as *mut _;
    std::ptr::null_mut()
}
pub unsafe fn control_panes_remove(
    head: *mut control_panes,
    elm: *mut control_pane,
) -> Option<Box<control_pane>> {
    if elm.is_null() {
        return None;
    }
    let key = control_panes_key(&*elm);
    let Some(map) = (*head).storage.as_mut() else {
        return None;
    };
    if map
        .get(&key)
        .map(|node| &**node as *const control_pane as *mut control_pane)
        != Some(elm)
    {
        return None;
    }
    let owner = map.remove(&key);
    (*elm).entry.owner = std::ptr::null_mut();
    if map.is_empty() {
        drop(Box::from_raw((*head).storage));
        (*head).storage = std::ptr::null_mut();
    }
    owner
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
    pair.map_or(std::ptr::null_mut(), |(_, node)| {
        &**node as *const control_pane as *mut control_pane
    })
}
pub unsafe fn control_panes_next(elm: *mut control_pane) -> *mut control_pane {
    let Some(map) = (*elm).entry.owner.as_ref() else {
        return std::ptr::null_mut();
    };
    let key = control_panes_key(&*elm);
    map.range((std::ops::Bound::Excluded(&key), std::ops::Bound::Unbounded))
        .next()
        .map_or(std::ptr::null_mut(), |(_, node)| {
            &**node as *const control_pane as *mut control_pane
        })
}
pub unsafe fn control_panes_prev(elm: *mut control_pane) -> *mut control_pane {
    let Some(map) = (*elm).entry.owner.as_ref() else {
        return std::ptr::null_mut();
    };
    let key = control_panes_key(&*elm);
    map.range((std::ops::Bound::Unbounded, std::ops::Bound::Excluded(&key)))
        .next_back()
        .map_or(std::ptr::null_mut(), |(_, node)| {
            &**node as *const control_pane as *mut control_pane
        })
}

fn control_windows_key(elm: &control_window) -> u32 {
    elm.window
}
pub unsafe fn control_windows_find(
    head: *mut control_windows,
    elm: *mut control_window,
) -> *mut control_window {
    let Some(map) = (*head).storage.as_ref() else {
        return std::ptr::null_mut();
    };
    let key = control_windows_key(&*elm);
    map.get(&key).copied().unwrap_or(std::ptr::null_mut())
}
pub unsafe fn control_windows_nfind(
    head: *mut control_windows,
    elm: *mut control_window,
) -> *mut control_window {
    let Some(map) = (*head).storage.as_ref() else {
        return std::ptr::null_mut();
    };
    let key = control_windows_key(&*elm);
    map.range((std::ops::Bound::Included(&key), std::ops::Bound::Unbounded))
        .next()
        .map_or(std::ptr::null_mut(), |(_, node)| *node)
}
pub unsafe fn control_windows_insert(
    head: *mut control_windows,
    elm: *mut control_window,
) -> *mut control_window {
    let key = control_windows_key(&*elm);
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
    let key = control_windows_key(&*elm);
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
    let key = control_windows_key(&*elm);
    map.range((std::ops::Bound::Excluded(&key), std::ops::Bound::Unbounded))
        .next()
        .map_or(std::ptr::null_mut(), |(_, node)| *node)
}
pub unsafe fn control_windows_prev(elm: *mut control_window) -> *mut control_window {
    let Some(map) = (*elm).entry.owner.as_ref() else {
        return std::ptr::null_mut();
    };
    let key = control_windows_key(&*elm);
    map.range((std::ops::Bound::Unbounded, std::ops::Bound::Excluded(&key)))
        .next_back()
        .map_or(std::ptr::null_mut(), |(_, node)| *node)
}
