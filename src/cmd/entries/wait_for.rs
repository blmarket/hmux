use crate::src::arguments::{args_get, args_has, args_string};
use crate::src::cmd::cmd_get_args;
use crate::src::cmd_queue::{cmdq_continue, cmdq_error, cmdq_get_client, cmdq_print};
use crate::src::events::{events_add_sink, events_remove_sink};
use crate::src::events_payload::{
    event_payload_add_formats, event_payload_first, event_payload_item_name,
    event_payload_item_print_owned, event_payload_next,
};
use crate::src::ffi::libc::{free, strcmp};
use crate::src::format::{format_create, format_expand, format_free, format_true};
use crate::src::hooks::hooks_valid_event_name;
use crate::src::log::log_debug;
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
use crate::src::shared::command::*;
pub use crate::src::shared::command::{
    cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmd_list, cmdq_item, cmdq_list, cmds,
    wait_item, wait_item_entry,
};
pub use crate::src::shared::control::control_state;
use crate::src::shared::display::*;
pub use crate::src::shared::display::{visible_range, visible_ranges};
pub use crate::src::shared::environment::environ;
use crate::src::shared::event::*;
pub use crate::src::shared::events::{event_payload, event_payload_item, events_cb, events_sink};
pub use crate::src::shared::format::{format_job_tree, format_tree};
pub use crate::src::shared::format::{FORMAT_NOJOBS, FORMAT_NONE};
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
pub use crate::src::shared::window::{
    window, window_alerts_entry, window_entry, window_mode, window_mode_entry,
    window_mode_entry_entry, window_winlinks, winlink, winlink_entry, winlink_sentry,
    winlink_stack, winlink_wentry, winlinks,
};
use std::ffi::{CStr, CString};

pub use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_13;
pub use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_12;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct wait_channel {
    pub name: *const ::core::ffi::c_char,
    pub locked: ::core::ffi::c_int,
    pub woken: ::core::ffi::c_int,
    pub waiters: C2RustUnnamed_38,
    pub lockers: C2RustUnnamed_36,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_36 {
    pub tqh_first: *mut wait_item,
    pub tqh_last: *mut *mut wait_item,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_38 {
    pub tqh_first: *mut wait_item,
    pub tqh_last: *mut *mut wait_item,
}
pub struct wait_channels {
    entries: std::collections::BTreeMap<Vec<u8>, Box<WaitChannelOwner>>,
}
// The map owns each channel while its intrusive queues borrow this stable
// C-shaped prefix. The name stays alive until the channel is removed.
#[repr(C)]
struct WaitChannelOwner {
    channel: wait_channel,
    name: CString,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct wait_event_item {
    pub item: *mut cmdq_item,
    pub sink: *mut events_sink,
    pub name: *mut ::core::ffi::c_char,
    pub filter: *mut ::core::ffi::c_char,
    pub verbose: ::core::ffi::c_int,
    pub entry: C2RustUnnamed_39,
}
// The queue and the event sink retain the C-shaped prefix address. The owner
// keeps its name and optional filter alive until the waiter is removed.
#[repr(C)]
struct WaitEventOwner {
    event: wait_event_item,
    name: CString,
    filter: Option<CString>,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_39 {
    pub tqe_next: *mut wait_event_item,
    pub tqe_prev: *mut *mut wait_event_item,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_40 {
    pub tqh_first: *mut wait_event_item,
    pub tqh_last: *mut *mut wait_event_item,
}

#[no_mangle]
pub static mut cmd_wait_for_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"wait-for\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"wait\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"EF:LSUlvw:\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 1 as ::core::ffi::c_int,
            upper: 1 as ::core::ffi::c_int,
            cb: None,
        },
        usage: b"[-ELSUlv] [-F format] [-w waiter] name\0" as *const u8
            as *const ::core::ffi::c_char,
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        flags: 0 as ::core::ffi::c_int,
        exec: Some(
            cmd_wait_for_exec as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
static mut wait_event_items: C2RustUnnamed_40 = C2RustUnnamed_40 {
    tqh_first: ::core::ptr::null::<wait_event_item>() as *mut wait_event_item,
    tqh_last: ::core::ptr::null::<*mut wait_event_item>() as *mut *mut wait_event_item,
};
static mut wait_channels: wait_channels = wait_channels {
    entries: std::collections::BTreeMap::new(),
};

unsafe fn wait_channel_key(name: *const ::core::ffi::c_char) -> Vec<u8> {
    std::ffi::CStr::from_ptr(name).to_bytes().to_vec()
}

unsafe fn wait_channels_find(
    head: *mut wait_channels,
    name: *const ::core::ffi::c_char,
) -> *mut wait_channel {
    (*head)
        .entries
        .get_mut(&wait_channel_key(name))
        .map(|owner| &raw mut owner.channel)
        .unwrap_or(::core::ptr::null_mut::<wait_channel>())
}

unsafe fn wait_channels_insert(
    head: *mut wait_channels,
    mut owner: Box<WaitChannelOwner>,
) -> *mut wait_channel {
    let key = wait_channel_key(owner.channel.name);
    match (*head).entries.entry(key) {
        std::collections::btree_map::Entry::Occupied(mut entry) => &raw mut entry.get_mut().channel,
        std::collections::btree_map::Entry::Vacant(entry) => {
            let channel = &raw mut owner.channel;
            entry.insert(owner);
            channel
        }
    }
}

unsafe fn wait_channels_remove(
    head: *mut wait_channels,
    elm: *mut wait_channel,
) -> Option<Box<WaitChannelOwner>> {
    (*head).entries.remove(&wait_channel_key((*elm).name))
}
unsafe extern "C" fn cmd_wait_for_add(name: *const ::core::ffi::c_char) -> *mut wait_channel {
    let name = CStr::from_ptr(name).to_owned();
    let mut owner = Box::new(WaitChannelOwner {
        channel: wait_channel {
            name: name.as_ptr(),
            locked: 0,
            woken: 0,
            waiters: C2RustUnnamed_38 {
                tqh_first: ::core::ptr::null_mut(),
                tqh_last: ::core::ptr::null_mut(),
            },
            lockers: C2RustUnnamed_36 {
                tqh_first: ::core::ptr::null_mut(),
                tqh_last: ::core::ptr::null_mut(),
            },
        },
        name,
    });
    owner.channel.waiters.tqh_last = &raw mut owner.channel.waiters.tqh_first;
    owner.channel.lockers.tqh_last = &raw mut owner.channel.lockers.tqh_first;
    let wc = wait_channels_insert(&raw mut wait_channels, owner);
    log_debug(
        b"add wait channel %s\0" as *const u8 as *const ::core::ffi::c_char,
        (*wc).name,
    );
    return wc;
}
unsafe extern "C" fn cmd_wait_for_remove(mut wc: *mut wait_channel) {
    if (*wc).locked != 0 {
        return;
    }
    if !(*wc).waiters.tqh_first.is_null() || (*wc).woken == 0 {
        return;
    }
    log_debug(
        b"remove wait channel %s\0" as *const u8 as *const ::core::ffi::c_char,
        (*wc).name,
    );
    drop(wait_channels_remove(&raw mut wait_channels, wc));
}
unsafe extern "C" fn cmd_wait_for_remove_empty(mut wc: *mut wait_channel) {
    if (*wc).locked != 0 || (*wc).woken != 0 {
        return;
    }
    if !(*wc).waiters.tqh_first.is_null() || !(*wc).lockers.tqh_first.is_null() {
        return;
    }
    log_debug(
        b"remove empty wait channel %s\0" as *const u8 as *const ::core::ffi::c_char,
        (*wc).name,
    );
    drop(wait_channels_remove(&raw mut wait_channels, wc));
}

fn wait_item_new(item: *mut cmdq_item) -> *mut wait_item {
    Box::into_raw(Box::new(wait_item {
        item,
        entry: wait_item_entry {
            tqe_next: ::core::ptr::null_mut(),
            tqe_prev: ::core::ptr::null_mut(),
        },
    }))
}
unsafe extern "C" fn cmd_wait_for_item_client_name(
    mut item: *mut cmdq_item,
) -> *const ::core::ffi::c_char {
    let mut c: *mut client = cmdq_get_client(item);
    if c.is_null() || (*c).name.is_null() {
        return b"\0" as *const u8 as *const ::core::ffi::c_char;
    }
    return (*c).name;
}
unsafe extern "C" fn cmd_wait_for_client_name(
    mut wei: *mut wait_event_item,
) -> *const ::core::ffi::c_char {
    return cmd_wait_for_item_client_name((*wei).item);
}
unsafe extern "C" fn cmd_wait_for_exec(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
) -> cmd_retval {
    let mut args: *mut args = cmd_get_args(self_0);
    let mut name: *const ::core::ffi::c_char = args_string(args, 0 as u_int);
    let mut wc: *mut wait_channel = ::core::ptr::null_mut::<wait_channel>();
    if args_has(args, 'E' as i32 as u_char) != 0 {
        return cmd_wait_for_event(item, name, args);
    }
    wc = wait_channels_find(&raw mut wait_channels, name);
    if args_has(args, 'l' as i32 as u_char) != 0 {
        return cmd_wait_for_list(item, wc);
    }
    if args_has(args, 'w' as i32 as u_char) != 0 {
        return cmd_wait_for_wake(item, name, args, wc);
    }
    if args_has(args, 'S' as i32 as u_char) != 0 {
        return cmd_wait_for_signal(item, name, wc);
    }
    if args_has(args, 'L' as i32 as u_char) != 0 {
        return cmd_wait_for_lock(item, name, wc);
    }
    if args_has(args, 'U' as i32 as u_char) != 0 {
        return cmd_wait_for_unlock(item, name, wc);
    }
    return cmd_wait_for_wait(item, name, wc);
}
unsafe extern "C" fn cmd_wait_for_event_print(
    mut wei: *mut wait_event_item,
    mut ep: *mut event_payload,
) {
    let mut epi: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    let mut key: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    epi = event_payload_first(ep);
    while !epi.is_null() {
        key = event_payload_item_name(epi);
        if *key as ::core::ffi::c_int != '_' as i32 {
            let value = event_payload_item_print_owned(epi);
            cmdq_print(
                (*wei).item,
                b"%s=%s\0" as *const u8 as *const ::core::ffi::c_char,
                key,
                value.as_ptr().cast::<::core::ffi::c_char>(),
            );
        }
        epi = event_payload_next(epi);
    }
}
unsafe extern "C" fn cmd_wait_for_event_cb(
    mut name: *const ::core::ffi::c_char,
    mut ep: *mut event_payload,
    mut item_data: *mut ::core::ffi::c_void,
) {
    let mut wei: *mut wait_event_item = item_data as *mut wait_event_item;
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut expanded: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut flag: ::core::ffi::c_int = 0;
    if (*wei).verbose != 0 {
        cmd_wait_for_event_print(wei, ep);
    }
    if !(*wei).filter.is_null() {
        ft = format_create(
            cmdq_get_client((*wei).item),
            (*wei).item,
            FORMAT_NONE,
            FORMAT_NOJOBS,
        );
        event_payload_add_formats(ep, ft, ::core::ptr::null::<::core::ffi::c_char>());
        expanded = format_expand(ft, (*wei).filter);
        flag = format_true(expanded);
        free(expanded as *mut ::core::ffi::c_void);
        format_free(ft);
        if flag == 0 {
            return;
        }
    }
    if !(*wei).entry.tqe_next.is_null() {
        (*(*wei).entry.tqe_next).entry.tqe_prev = (*wei).entry.tqe_prev;
    } else {
        wait_event_items.tqh_last = (*wei).entry.tqe_prev;
    }
    *(*wei).entry.tqe_prev = (*wei).entry.tqe_next;
    cmdq_continue((*wei).item);
    cmd_wait_for_event_free(wei);
}
unsafe extern "C" fn cmd_wait_for_event_free(mut wei: *mut wait_event_item) {
    events_remove_sink((*wei).sink);
    drop(Box::from_raw(wei.cast::<WaitEventOwner>()));
}
unsafe extern "C" fn cmd_wait_for_event(
    mut item: *mut cmdq_item,
    mut name: *const ::core::ffi::c_char,
    mut args: *mut args,
) -> cmd_retval {
    let mut wei: *mut wait_event_item = ::core::ptr::null_mut::<wait_event_item>();
    let mut filter: *const ::core::ffi::c_char = args_get(args, 'F' as i32 as u_char);
    if hooks_valid_event_name(name) == 0 {
        cmdq_error(
            item,
            b"invalid event: %s\0" as *const u8 as *const ::core::ffi::c_char,
            name,
        );
        return CMD_RETURN_ERROR;
    }
    if args_has(args, 'l' as i32 as u_char) != 0 {
        return cmd_wait_for_event_list(item, name);
    }
    if args_has(args, 'w' as i32 as u_char) != 0 {
        return cmd_wait_for_event_wake(item, name, args);
    }
    if cmdq_get_client(item).is_null() {
        cmdq_error(
            item,
            b"not able to wait\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return CMD_RETURN_ERROR;
    }
    let mut owner = Box::new(WaitEventOwner {
        event: wait_event_item {
            item,
            sink: ::core::ptr::null_mut(),
            name: ::core::ptr::null_mut(),
            filter: ::core::ptr::null_mut(),
            verbose: args_has(args, 'v' as i32 as u_char),
            entry: C2RustUnnamed_39 {
                tqe_next: ::core::ptr::null_mut(),
                tqe_prev: ::core::ptr::null_mut(),
            },
        },
        name: CStr::from_ptr(name).to_owned(),
        filter: if filter.is_null() {
            None
        } else {
            Some(CStr::from_ptr(filter).to_owned())
        },
    });
    owner.event.name = owner.name.as_ptr().cast_mut();
    owner.event.filter = owner
        .filter
        .as_ref()
        .map_or(::core::ptr::null_mut(), |filter| filter.as_ptr().cast_mut());
    wei = &raw mut owner.event;
    (*wei).sink = events_add_sink(
        name,
        Some(
            cmd_wait_for_event_cb
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_char,
                    *mut event_payload,
                    *mut ::core::ffi::c_void,
                ) -> (),
        ),
        wei as *mut ::core::ffi::c_void,
    );
    (*wei).entry.tqe_next = ::core::ptr::null_mut::<wait_event_item>();
    (*wei).entry.tqe_prev = wait_event_items.tqh_last;
    *wait_event_items.tqh_last = wei;
    wait_event_items.tqh_last = &raw mut (*wei).entry.tqe_next;
    let _ = Box::into_raw(owner);
    return CMD_RETURN_WAIT;
}
unsafe extern "C" fn cmd_wait_for_event_list(
    mut item: *mut cmdq_item,
    mut name: *const ::core::ffi::c_char,
) -> cmd_retval {
    let mut wei: *mut wait_event_item = ::core::ptr::null_mut::<wait_event_item>();
    wei = wait_event_items.tqh_first;
    while !wei.is_null() {
        if strcmp((*wei).name, name) == 0 as ::core::ffi::c_int {
            cmdq_print(
                item,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                cmd_wait_for_client_name(wei),
            );
        }
        wei = (*wei).entry.tqe_next;
    }
    return CMD_RETURN_NORMAL;
}
unsafe extern "C" fn cmd_wait_for_event_wake(
    mut item: *mut cmdq_item,
    mut name: *const ::core::ffi::c_char,
    mut args: *mut args,
) -> cmd_retval {
    let mut wei: *mut wait_event_item = ::core::ptr::null_mut::<wait_event_item>();
    let mut wei1: *mut wait_event_item = ::core::ptr::null_mut::<wait_event_item>();
    let mut client_name: *const ::core::ffi::c_char = args_get(args, 'w' as i32 as u_char);
    wei = wait_event_items.tqh_first;
    while !wei.is_null() && {
        wei1 = (*wei).entry.tqe_next;
        1 as ::core::ffi::c_int != 0
    } {
        if !(strcmp((*wei).name, name) != 0 as ::core::ffi::c_int) {
            if !(strcmp(cmd_wait_for_client_name(wei), client_name) != 0 as ::core::ffi::c_int) {
                if !(*wei).entry.tqe_next.is_null() {
                    (*(*wei).entry.tqe_next).entry.tqe_prev = (*wei).entry.tqe_prev;
                } else {
                    wait_event_items.tqh_last = (*wei).entry.tqe_prev;
                }
                *(*wei).entry.tqe_prev = (*wei).entry.tqe_next;
                cmdq_continue((*wei).item);
                cmd_wait_for_event_free(wei);
                return CMD_RETURN_NORMAL;
            }
        }
        wei = wei1;
    }
    cmdq_error(
        item,
        b"waiter %s not found\0" as *const u8 as *const ::core::ffi::c_char,
        client_name,
    );
    return CMD_RETURN_ERROR;
}
unsafe extern "C" fn cmd_wait_for_list(
    mut item: *mut cmdq_item,
    mut wc: *mut wait_channel,
) -> cmd_retval {
    let mut wi: *mut wait_item = ::core::ptr::null_mut::<wait_item>();
    if wc.is_null() {
        return CMD_RETURN_NORMAL;
    }
    wi = (*wc).waiters.tqh_first;
    while !wi.is_null() {
        cmdq_print(
            item,
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            cmd_wait_for_item_client_name((*wi).item),
        );
        wi = (*wi).entry.tqe_next;
    }
    wi = (*wc).lockers.tqh_first;
    while !wi.is_null() {
        cmdq_print(
            item,
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            cmd_wait_for_item_client_name((*wi).item),
        );
        wi = (*wi).entry.tqe_next;
    }
    return CMD_RETURN_NORMAL;
}
unsafe extern "C" fn cmd_wait_for_wake(
    mut item: *mut cmdq_item,
    mut name: *const ::core::ffi::c_char,
    mut args: *mut args,
    mut wc: *mut wait_channel,
) -> cmd_retval {
    let mut wi: *mut wait_item = ::core::ptr::null_mut::<wait_item>();
    let mut wi1: *mut wait_item = ::core::ptr::null_mut::<wait_item>();
    let mut client_name: *const ::core::ffi::c_char = args_get(args, 'w' as i32 as u_char);
    if !wc.is_null() {
        wi = (*wc).waiters.tqh_first;
        while !wi.is_null() && {
            wi1 = (*wi).entry.tqe_next;
            1 as ::core::ffi::c_int != 0
        } {
            name = cmd_wait_for_item_client_name((*wi).item);
            if strcmp(name, client_name) != 0 as ::core::ffi::c_int {
                wi = wi1;
            } else {
                cmdq_continue((*wi).item);
                if !(*wi).entry.tqe_next.is_null() {
                    (*(*wi).entry.tqe_next).entry.tqe_prev = (*wi).entry.tqe_prev;
                } else {
                    (*wc).waiters.tqh_last = (*wi).entry.tqe_prev;
                }
                *(*wi).entry.tqe_prev = (*wi).entry.tqe_next;
                drop(Box::from_raw(wi));
                cmd_wait_for_remove_empty(wc);
                return CMD_RETURN_NORMAL;
            }
        }
        wi = (*wc).lockers.tqh_first;
        while !wi.is_null() && {
            wi1 = (*wi).entry.tqe_next;
            1 as ::core::ffi::c_int != 0
        } {
            name = cmd_wait_for_item_client_name((*wi).item);
            if strcmp(name, client_name) != 0 as ::core::ffi::c_int {
                wi = wi1;
            } else {
                cmdq_continue((*wi).item);
                if !(*wi).entry.tqe_next.is_null() {
                    (*(*wi).entry.tqe_next).entry.tqe_prev = (*wi).entry.tqe_prev;
                } else {
                    (*wc).lockers.tqh_last = (*wi).entry.tqe_prev;
                }
                *(*wi).entry.tqe_prev = (*wi).entry.tqe_next;
                drop(Box::from_raw(wi));
                cmd_wait_for_remove_empty(wc);
                return CMD_RETURN_NORMAL;
            }
        }
    }
    return CMD_RETURN_NORMAL;
}
unsafe extern "C" fn cmd_wait_for_signal(
    mut item: *mut cmdq_item,
    mut name: *const ::core::ffi::c_char,
    mut wc: *mut wait_channel,
) -> cmd_retval {
    let mut wi: *mut wait_item = ::core::ptr::null_mut::<wait_item>();
    let mut wi1: *mut wait_item = ::core::ptr::null_mut::<wait_item>();
    if wc.is_null() {
        wc = cmd_wait_for_add(name);
    }
    if (*wc).waiters.tqh_first.is_null() && (*wc).woken == 0 {
        log_debug(
            b"signal wait channel %s, no waiters\0" as *const u8 as *const ::core::ffi::c_char,
            (*wc).name,
        );
        (*wc).woken = 1 as ::core::ffi::c_int;
        return CMD_RETURN_NORMAL;
    }
    log_debug(
        b"signal wait channel %s, with waiters\0" as *const u8 as *const ::core::ffi::c_char,
        (*wc).name,
    );
    wi = (*wc).waiters.tqh_first;
    while !wi.is_null() && {
        wi1 = (*wi).entry.tqe_next;
        1 as ::core::ffi::c_int != 0
    } {
        cmdq_continue((*wi).item);
        if !(*wi).entry.tqe_next.is_null() {
            (*(*wi).entry.tqe_next).entry.tqe_prev = (*wi).entry.tqe_prev;
        } else {
            (*wc).waiters.tqh_last = (*wi).entry.tqe_prev;
        }
        *(*wi).entry.tqe_prev = (*wi).entry.tqe_next;
        drop(Box::from_raw(wi));
        wi = wi1;
    }
    cmd_wait_for_remove(wc);
    return CMD_RETURN_NORMAL;
}
unsafe extern "C" fn cmd_wait_for_wait(
    mut item: *mut cmdq_item,
    mut name: *const ::core::ffi::c_char,
    mut wc: *mut wait_channel,
) -> cmd_retval {
    let mut c: *mut client = cmdq_get_client(item);
    let mut wi: *mut wait_item = ::core::ptr::null_mut::<wait_item>();
    if c.is_null() {
        cmdq_error(
            item,
            b"not able to wait\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return CMD_RETURN_ERROR;
    }
    if wc.is_null() {
        wc = cmd_wait_for_add(name);
    }
    if (*wc).woken != 0 {
        log_debug(
            b"wait channel %s already woken (%p)\0" as *const u8 as *const ::core::ffi::c_char,
            (*wc).name,
            c,
        );
        cmd_wait_for_remove(wc);
        return CMD_RETURN_NORMAL;
    }
    log_debug(
        b"wait channel %s not woken (%p)\0" as *const u8 as *const ::core::ffi::c_char,
        (*wc).name,
        c,
    );
    wi = wait_item_new(item);
    (*wi).entry.tqe_next = ::core::ptr::null_mut::<wait_item>();
    (*wi).entry.tqe_prev = (*wc).waiters.tqh_last;
    *(*wc).waiters.tqh_last = wi;
    (*wc).waiters.tqh_last = &raw mut (*wi).entry.tqe_next;
    return CMD_RETURN_WAIT;
}
unsafe extern "C" fn cmd_wait_for_lock(
    mut item: *mut cmdq_item,
    mut name: *const ::core::ffi::c_char,
    mut wc: *mut wait_channel,
) -> cmd_retval {
    let mut wi: *mut wait_item = ::core::ptr::null_mut::<wait_item>();
    if cmdq_get_client(item).is_null() {
        cmdq_error(
            item,
            b"not able to lock\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return CMD_RETURN_ERROR;
    }
    if wc.is_null() {
        wc = cmd_wait_for_add(name);
    }
    if (*wc).locked != 0 {
        wi = wait_item_new(item);
        (*wi).entry.tqe_next = ::core::ptr::null_mut::<wait_item>();
        (*wi).entry.tqe_prev = (*wc).lockers.tqh_last;
        *(*wc).lockers.tqh_last = wi;
        (*wc).lockers.tqh_last = &raw mut (*wi).entry.tqe_next;
        return CMD_RETURN_WAIT;
    }
    (*wc).locked = 1 as ::core::ffi::c_int;
    return CMD_RETURN_NORMAL;
}
unsafe extern "C" fn cmd_wait_for_unlock(
    mut item: *mut cmdq_item,
    mut name: *const ::core::ffi::c_char,
    mut wc: *mut wait_channel,
) -> cmd_retval {
    let mut wi: *mut wait_item = ::core::ptr::null_mut::<wait_item>();
    if wc.is_null() || (*wc).locked == 0 {
        cmdq_error(
            item,
            b"channel %s not locked\0" as *const u8 as *const ::core::ffi::c_char,
            name,
        );
        return CMD_RETURN_ERROR;
    }
    wi = (*wc).lockers.tqh_first;
    if !wi.is_null() {
        cmdq_continue((*wi).item);
        if !(*wi).entry.tqe_next.is_null() {
            (*(*wi).entry.tqe_next).entry.tqe_prev = (*wi).entry.tqe_prev;
        } else {
            (*wc).lockers.tqh_last = (*wi).entry.tqe_prev;
        }
        *(*wi).entry.tqe_prev = (*wi).entry.tqe_next;
        drop(Box::from_raw(wi));
    } else {
        (*wc).locked = 0 as ::core::ffi::c_int;
        cmd_wait_for_remove(wc);
    }
    return CMD_RETURN_NORMAL;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_wait_for_flush() {
    let mut wi: *mut wait_item = ::core::ptr::null_mut::<wait_item>();
    let mut wi1: *mut wait_item = ::core::ptr::null_mut::<wait_item>();
    let mut wei: *mut wait_event_item = ::core::ptr::null_mut::<wait_event_item>();
    let mut wei1: *mut wait_event_item = ::core::ptr::null_mut::<wait_event_item>();
    wei = wait_event_items.tqh_first;
    while !wei.is_null() && {
        wei1 = (*wei).entry.tqe_next;
        1 as ::core::ffi::c_int != 0
    } {
        if !(*wei).entry.tqe_next.is_null() {
            (*(*wei).entry.tqe_next).entry.tqe_prev = (*wei).entry.tqe_prev;
        } else {
            wait_event_items.tqh_last = (*wei).entry.tqe_prev;
        }
        *(*wei).entry.tqe_prev = (*wei).entry.tqe_next;
        cmdq_continue((*wei).item);
        cmd_wait_for_event_free(wei);
        wei = wei1;
    }
    let channels = (&raw mut wait_channels)
        .as_mut()
        .unwrap()
        .entries
        .values_mut()
        .map(|owner| &raw mut owner.channel)
        .collect::<Vec<_>>();
    for wc in channels {
        wi = (*wc).waiters.tqh_first;
        while !wi.is_null() && {
            wi1 = (*wi).entry.tqe_next;
            1 as ::core::ffi::c_int != 0
        } {
            cmdq_continue((*wi).item);
            if !(*wi).entry.tqe_next.is_null() {
                (*(*wi).entry.tqe_next).entry.tqe_prev = (*wi).entry.tqe_prev;
            } else {
                (*wc).waiters.tqh_last = (*wi).entry.tqe_prev;
            }
            *(*wi).entry.tqe_prev = (*wi).entry.tqe_next;
            drop(Box::from_raw(wi));
            wi = wi1;
        }
        (*wc).woken = 1 as ::core::ffi::c_int;
        wi = (*wc).lockers.tqh_first;
        while !wi.is_null() && {
            wi1 = (*wi).entry.tqe_next;
            1 as ::core::ffi::c_int != 0
        } {
            cmdq_continue((*wi).item);
            if !(*wi).entry.tqe_next.is_null() {
                (*(*wi).entry.tqe_next).entry.tqe_prev = (*wi).entry.tqe_prev;
            } else {
                (*wc).lockers.tqh_last = (*wi).entry.tqe_prev;
            }
            *(*wi).entry.tqe_prev = (*wi).entry.tqe_next;
            drop(Box::from_raw(wi));
            wi = wi1;
        }
        (*wc).locked = 0 as ::core::ffi::c_int;
        cmd_wait_for_remove(wc);
    }
}
unsafe extern "C" fn run_static_initializers() {
    wait_event_items = C2RustUnnamed_40 {
        tqh_first: ::core::ptr::null_mut::<wait_event_item>(),
        tqh_last: &raw mut wait_event_items.tqh_first,
    };
}
#[used]
#[cfg_attr(target_os = "linux", link_section = ".init_array")]
#[cfg_attr(target_os = "windows", link_section = ".CRT$XIB")]
#[cfg_attr(target_os = "macos", link_section = "__DATA,__mod_init_func")]
static INIT_ARRAY: [unsafe extern "C" fn(); 1] = [run_static_initializers];

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    fn test_channel(name: &CStr) -> Box<WaitChannelOwner> {
        let name = name.to_owned();
        let mut owner = Box::new(WaitChannelOwner {
            channel: wait_channel {
                name: name.as_ptr(),
                locked: 0,
                woken: 0,
                waiters: C2RustUnnamed_38 {
                    tqh_first: ::core::ptr::null_mut(),
                    tqh_last: ::core::ptr::null_mut(),
                },
                lockers: C2RustUnnamed_36 {
                    tqh_first: ::core::ptr::null_mut(),
                    tqh_last: ::core::ptr::null_mut(),
                },
            },
            name,
        });
        owner.channel.waiters.tqh_last = &raw mut owner.channel.waiters.tqh_first;
        owner.channel.lockers.tqh_last = &raw mut owner.channel.lockers.tqh_first;
        owner
    }

    #[test]
    fn wait_channels_match_c_string_tree_semantics() {
        let name_a = CString::new("a").unwrap();
        let name_a0 = CString::new("a0").unwrap();
        let name_a_high = CString::new(vec![b'a', 0xff]).unwrap();
        let name_z = CString::new("z").unwrap();
        let lookup_a = CString::new("a").unwrap();

        let mut channels = wait_channels {
            entries: std::collections::BTreeMap::new(),
        };
        unsafe {
            let a = wait_channels_insert(&raw mut channels, test_channel(&name_a));
            let a0 = wait_channels_insert(&raw mut channels, test_channel(&name_a0));
            let a_high = wait_channels_insert(&raw mut channels, test_channel(&name_a_high));
            let z = wait_channels_insert(&raw mut channels, test_channel(&name_z));
            assert!(!a.is_null() && !a0.is_null() && !a_high.is_null() && !z.is_null());
            assert_eq!(
                wait_channels_insert(&raw mut channels, test_channel(&name_a)),
                a
            );
            assert_eq!(wait_channels_find(&raw mut channels, lookup_a.as_ptr()), a);

            let ordered = channels
                .entries
                .values_mut()
                .map(|owner| &raw mut owner.channel)
                .collect::<Vec<_>>();
            assert_eq!(ordered, vec![a, a0, a_high, z]);

            let mut removed = wait_channels_remove(&raw mut channels, a_high).unwrap();
            assert_eq!(&raw mut removed.channel, a_high);
            assert!(wait_channels_find(&raw mut channels, name_a_high.as_ptr()).is_null());
        }
    }
}
