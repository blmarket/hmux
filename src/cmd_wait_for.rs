pub use crate::src::shared::events::{event_payload, event_payload_item, events_cb, events_sink};
pub use crate::src::shared::arguments::{args, args_parse, args_parse_cb};
pub use crate::src::shared::client::{
    client, client_entry, client_file, client_file_cb, client_file_entry, client_files,
    overlay_check_cb, overlay_draw_cb, overlay_free_cb, overlay_key_cb, overlay_mode_cb,
    overlay_resize_cb,
};
pub use crate::src::shared::command::{
    cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmd_list, cmdq_item, cmdq_list, cmds,
    wait_item, wait_item_entry,
};
pub use crate::src::shared::control::{control_state};
pub use crate::src::shared::format::{format_job_tree, format_tree};
pub use crate::src::shared::input::{input_ctx, input_request, input_requests};
pub use crate::src::shared::key::{
    key_binding, key_binding_entry, key_bindings, key_event, key_table, key_table_entry,
};
pub use crate::src::shared::layout::{layout_cell, layout_cell_entry, layout_cells};
pub use crate::src::shared::menu::{menu_data};
pub use crate::src::shared::options::{options};
pub use crate::src::shared::pane::{
    window_pane, window_pane_entry, window_pane_modes, window_pane_prompt, window_pane_sentry,
    window_pane_tree_entry, window_pane_zentry, window_panes,
};
pub use crate::src::shared::process::{tmuxpeer};
pub use crate::src::shared::prompt::{prompt};
pub use crate::src::shared::redraw::{redraw_scene};
pub use crate::src::shared::session::{session, session_entry, session_gentry};
pub use crate::src::shared::spawn::{spawn_editor_state};
pub use crate::src::shared::status::{status_line};
pub use crate::src::shared::tty::{tty, tty_code, tty_key, tty_term, tty_term_entry};
pub use crate::src::shared::window::{
    window, window_alerts_entry, window_entry, window_mode, window_mode_entry,
    window_mode_entry_entry, window_winlinks, winlink, winlink_entry, winlink_sentry,
    winlink_stack, winlink_wentry, winlinks,
};
pub use crate::src::shared::environment::{environ};
pub use crate::src::shared::format::{FORMAT_NOJOBS, FORMAT_NONE};
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::{screen_write_cline};
pub use crate::src::shared::hyperlinks::{hyperlinks};
pub use crate::src::shared::pane::{
    window_pane_offset, window_pane_resize, window_pane_resize_entry, window_pane_resizes,
};
pub use crate::src::shared::tree::{RB_BLACK, RB_NEGINF, RB_RED};
pub use crate::src::shared::display::{visible_range, visible_ranges};
pub use crate::src::shared::layout::{layout_geometry};
pub use crate::src::shared::mouse::{mouse_event};
use crate::src::shared::client::*;
use crate::src::shared::arguments::*;
use crate::src::shared::terminal::*;
use crate::src::shared::event::*;
use crate::src::shared::display::*;
use crate::src::shared::layout::*;
use crate::src::shared::message::*;
use crate::src::shared::abi::*;
use crate::src::shared::colour::*;
use crate::src::shared::command::*;
use crate::src::shared::grid::*;
use crate::src::shared::key::*;
use crate::src::shared::style::*;
extern "C" {

    fn strcmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn xmalloc(_: size_t) -> *mut ::core::ffi::c_void;
    fn xcalloc(_: size_t, _: size_t) -> *mut ::core::ffi::c_void;
    fn xstrdup(_: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn format_true(_: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn format_create(
        _: *mut client,
        _: *mut cmdq_item,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
    ) -> *mut format_tree;
    fn format_free(_: *mut format_tree);
    fn format_expand(
        _: *mut format_tree,
        _: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn event_payload_item_print(_: *mut event_payload_item) -> *mut ::core::ffi::c_char;
    fn event_payload_add_formats(
        _: *mut event_payload,
        _: *mut format_tree,
        _: *const ::core::ffi::c_char,
    );
    fn event_payload_first(_: *mut event_payload) -> *mut event_payload_item;
    fn event_payload_next(_: *mut event_payload_item) -> *mut event_payload_item;
    fn event_payload_item_name(_: *mut event_payload_item) -> *const ::core::ffi::c_char;
    fn events_add_sink(
        _: *const ::core::ffi::c_char,
        _: events_cb,
        _: *mut ::core::ffi::c_void,
    ) -> *mut events_sink;
    fn events_remove_sink(_: *mut events_sink);
    fn hooks_valid_event_name(_: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn args_has(_: *mut args, _: u_char) -> ::core::ffi::c_int;
    fn args_get(_: *mut args, _: u_char) -> *const ::core::ffi::c_char;
    fn args_string(_: *mut args, _: u_int) -> *const ::core::ffi::c_char;
    fn cmd_get_args(_: *mut cmd) -> *mut args;
    fn cmdq_get_client(_: *mut cmdq_item) -> *mut client;
    fn cmdq_continue(_: *mut cmdq_item);
    fn cmdq_print(_: *mut cmdq_item, _: *const ::core::ffi::c_char, ...);
    fn cmdq_error(_: *mut cmdq_item, _: *const ::core::ffi::c_char, ...);
    fn log_debug(_: *const ::core::ffi::c_char, ...);
}

pub use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_12;
pub use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_13;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct wait_channel {
    pub name: *const ::core::ffi::c_char,
    pub locked: ::core::ffi::c_int,
    pub woken: ::core::ffi::c_int,
    pub waiters: C2RustUnnamed_38,
    pub lockers: C2RustUnnamed_36,
    pub entry: C2RustUnnamed_35,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_35 {
    pub rbe_left: *mut wait_channel,
    pub rbe_right: *mut wait_channel,
    pub rbe_parent: *mut wait_channel,
    pub rbe_color: ::core::ffi::c_int,
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct wait_channels {
    pub rbh_root: *mut wait_channel,
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
    rbh_root: ::core::ptr::null::<wait_channel>() as *mut wait_channel,
};
unsafe extern "C" fn wait_channels_RB_NEXT(mut elm: *mut wait_channel) -> *mut wait_channel {
    if !(*elm).entry.rbe_right.is_null() {
        elm = (*elm).entry.rbe_right;
        while !(*elm).entry.rbe_left.is_null() {
            elm = (*elm).entry.rbe_left;
        }
    } else if !(*elm).entry.rbe_parent.is_null() && elm == (*(*elm).entry.rbe_parent).entry.rbe_left
    {
        elm = (*elm).entry.rbe_parent;
    } else {
        while !(*elm).entry.rbe_parent.is_null()
            && elm == (*(*elm).entry.rbe_parent).entry.rbe_right
        {
            elm = (*elm).entry.rbe_parent;
        }
        elm = (*elm).entry.rbe_parent;
    }
    return elm;
}
unsafe extern "C" fn wait_channels_RB_MINMAX(
    mut head: *mut wait_channels,
    mut val: ::core::ffi::c_int,
) -> *mut wait_channel {
    let mut tmp: *mut wait_channel = (*head).rbh_root;
    let mut parent: *mut wait_channel = ::core::ptr::null_mut::<wait_channel>();
    while !tmp.is_null() {
        parent = tmp;
        if val < 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_left;
        } else {
            tmp = (*tmp).entry.rbe_right;
        }
    }
    return parent;
}
unsafe extern "C" fn wait_channels_RB_INSERT(
    mut head: *mut wait_channels,
    mut elm: *mut wait_channel,
) -> *mut wait_channel {
    let mut tmp: *mut wait_channel = ::core::ptr::null_mut::<wait_channel>();
    let mut parent: *mut wait_channel = ::core::ptr::null_mut::<wait_channel>();
    let mut comp: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    tmp = (*head).rbh_root;
    while !tmp.is_null() {
        parent = tmp;
        comp = wait_channel_cmp(elm, parent);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_right;
        } else {
            return tmp;
        }
    }
    (*elm).entry.rbe_parent = parent;
    (*elm).entry.rbe_right = ::core::ptr::null_mut::<wait_channel>();
    (*elm).entry.rbe_left = (*elm).entry.rbe_right;
    (*elm).entry.rbe_color = RB_RED;
    if !parent.is_null() {
        if comp < 0 as ::core::ffi::c_int {
            (*parent).entry.rbe_left = elm;
        } else {
            (*parent).entry.rbe_right = elm;
        }
    } else {
        (*head).rbh_root = elm;
    }
    wait_channels_RB_INSERT_COLOR(head, elm);
    return ::core::ptr::null_mut::<wait_channel>();
}
unsafe extern "C" fn wait_channels_RB_INSERT_COLOR(
    mut head: *mut wait_channels,
    mut elm: *mut wait_channel,
) {
    let mut parent: *mut wait_channel = ::core::ptr::null_mut::<wait_channel>();
    let mut gparent: *mut wait_channel = ::core::ptr::null_mut::<wait_channel>();
    let mut tmp: *mut wait_channel = ::core::ptr::null_mut::<wait_channel>();
    loop {
        parent = (*elm).entry.rbe_parent;
        if !(!parent.is_null() && (*parent).entry.rbe_color == RB_RED) {
            break;
        }
        gparent = (*parent).entry.rbe_parent;
        if parent == (*gparent).entry.rbe_left {
            tmp = (*gparent).entry.rbe_right;
            if !tmp.is_null() && (*tmp).entry.rbe_color == RB_RED {
                (*tmp).entry.rbe_color = RB_BLACK;
                (*parent).entry.rbe_color = RB_BLACK;
                (*gparent).entry.rbe_color = RB_RED;
                elm = gparent;
            } else {
                if (*parent).entry.rbe_right == elm {
                    tmp = (*parent).entry.rbe_right;
                    (*parent).entry.rbe_right = (*tmp).entry.rbe_left;
                    if !(*parent).entry.rbe_right.is_null() {
                        (*(*tmp).entry.rbe_left).entry.rbe_parent = parent;
                    }
                    (*tmp).entry.rbe_parent = (*parent).entry.rbe_parent;
                    if !(*tmp).entry.rbe_parent.is_null() {
                        if parent == (*(*parent).entry.rbe_parent).entry.rbe_left {
                            (*(*parent).entry.rbe_parent).entry.rbe_left = tmp;
                        } else {
                            (*(*parent).entry.rbe_parent).entry.rbe_right = tmp;
                        }
                    } else {
                        (*head).rbh_root = tmp;
                    }
                    (*tmp).entry.rbe_left = parent;
                    (*parent).entry.rbe_parent = tmp;
                    !(*tmp).entry.rbe_parent.is_null();
                    tmp = parent;
                    parent = elm;
                    elm = tmp;
                }
                (*parent).entry.rbe_color = RB_BLACK;
                (*gparent).entry.rbe_color = RB_RED;
                tmp = (*gparent).entry.rbe_left;
                (*gparent).entry.rbe_left = (*tmp).entry.rbe_right;
                if !(*gparent).entry.rbe_left.is_null() {
                    (*(*tmp).entry.rbe_right).entry.rbe_parent = gparent;
                }
                (*tmp).entry.rbe_parent = (*gparent).entry.rbe_parent;
                if !(*tmp).entry.rbe_parent.is_null() {
                    if gparent == (*(*gparent).entry.rbe_parent).entry.rbe_left {
                        (*(*gparent).entry.rbe_parent).entry.rbe_left = tmp;
                    } else {
                        (*(*gparent).entry.rbe_parent).entry.rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).entry.rbe_right = gparent;
                (*gparent).entry.rbe_parent = tmp;
                !(*tmp).entry.rbe_parent.is_null();
            }
        } else {
            tmp = (*gparent).entry.rbe_left;
            if !tmp.is_null() && (*tmp).entry.rbe_color == RB_RED {
                (*tmp).entry.rbe_color = RB_BLACK;
                (*parent).entry.rbe_color = RB_BLACK;
                (*gparent).entry.rbe_color = RB_RED;
                elm = gparent;
            } else {
                if (*parent).entry.rbe_left == elm {
                    tmp = (*parent).entry.rbe_left;
                    (*parent).entry.rbe_left = (*tmp).entry.rbe_right;
                    if !(*parent).entry.rbe_left.is_null() {
                        (*(*tmp).entry.rbe_right).entry.rbe_parent = parent;
                    }
                    (*tmp).entry.rbe_parent = (*parent).entry.rbe_parent;
                    if !(*tmp).entry.rbe_parent.is_null() {
                        if parent == (*(*parent).entry.rbe_parent).entry.rbe_left {
                            (*(*parent).entry.rbe_parent).entry.rbe_left = tmp;
                        } else {
                            (*(*parent).entry.rbe_parent).entry.rbe_right = tmp;
                        }
                    } else {
                        (*head).rbh_root = tmp;
                    }
                    (*tmp).entry.rbe_right = parent;
                    (*parent).entry.rbe_parent = tmp;
                    !(*tmp).entry.rbe_parent.is_null();
                    tmp = parent;
                    parent = elm;
                    elm = tmp;
                }
                (*parent).entry.rbe_color = RB_BLACK;
                (*gparent).entry.rbe_color = RB_RED;
                tmp = (*gparent).entry.rbe_right;
                (*gparent).entry.rbe_right = (*tmp).entry.rbe_left;
                if !(*gparent).entry.rbe_right.is_null() {
                    (*(*tmp).entry.rbe_left).entry.rbe_parent = gparent;
                }
                (*tmp).entry.rbe_parent = (*gparent).entry.rbe_parent;
                if !(*tmp).entry.rbe_parent.is_null() {
                    if gparent == (*(*gparent).entry.rbe_parent).entry.rbe_left {
                        (*(*gparent).entry.rbe_parent).entry.rbe_left = tmp;
                    } else {
                        (*(*gparent).entry.rbe_parent).entry.rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).entry.rbe_left = gparent;
                (*gparent).entry.rbe_parent = tmp;
                !(*tmp).entry.rbe_parent.is_null();
            }
        }
    }
    (*(*head).rbh_root).entry.rbe_color = RB_BLACK;
}
unsafe extern "C" fn wait_channels_RB_FIND(
    mut head: *mut wait_channels,
    mut elm: *mut wait_channel,
) -> *mut wait_channel {
    let mut tmp: *mut wait_channel = (*head).rbh_root;
    let mut comp: ::core::ffi::c_int = 0;
    while !tmp.is_null() {
        comp = wait_channel_cmp(elm, tmp);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_right;
        } else {
            return tmp;
        }
    }
    return ::core::ptr::null_mut::<wait_channel>();
}
unsafe extern "C" fn wait_channels_RB_REMOVE(
    mut head: *mut wait_channels,
    mut elm: *mut wait_channel,
) -> *mut wait_channel {
    let mut current_block: u64;
    let mut child: *mut wait_channel = ::core::ptr::null_mut::<wait_channel>();
    let mut parent: *mut wait_channel = ::core::ptr::null_mut::<wait_channel>();
    let mut old: *mut wait_channel = elm;
    let mut color: ::core::ffi::c_int = 0;
    if (*elm).entry.rbe_left.is_null() {
        child = (*elm).entry.rbe_right;
        current_block = 7245201122033322888;
    } else if (*elm).entry.rbe_right.is_null() {
        child = (*elm).entry.rbe_left;
        current_block = 7245201122033322888;
    } else {
        let mut left: *mut wait_channel = ::core::ptr::null_mut::<wait_channel>();
        elm = (*elm).entry.rbe_right;
        loop {
            left = (*elm).entry.rbe_left;
            if left.is_null() {
                break;
            }
            elm = left;
        }
        child = (*elm).entry.rbe_right;
        parent = (*elm).entry.rbe_parent;
        color = (*elm).entry.rbe_color;
        if !child.is_null() {
            (*child).entry.rbe_parent = parent;
        }
        if !parent.is_null() {
            if (*parent).entry.rbe_left == elm {
                (*parent).entry.rbe_left = child;
            } else {
                (*parent).entry.rbe_right = child;
            }
        } else {
            (*head).rbh_root = child;
        }
        if (*elm).entry.rbe_parent == old {
            parent = elm;
        }
        (*elm).entry = (*old).entry;
        if !(*old).entry.rbe_parent.is_null() {
            if (*(*old).entry.rbe_parent).entry.rbe_left == old {
                (*(*old).entry.rbe_parent).entry.rbe_left = elm;
            } else {
                (*(*old).entry.rbe_parent).entry.rbe_right = elm;
            }
        } else {
            (*head).rbh_root = elm;
        }
        (*(*old).entry.rbe_left).entry.rbe_parent = elm;
        if !(*old).entry.rbe_right.is_null() {
            (*(*old).entry.rbe_right).entry.rbe_parent = elm;
        }
        if !parent.is_null() {
            left = parent;
            loop {
                left = (*left).entry.rbe_parent;
                if left.is_null() {
                    break;
                }
            }
        }
        current_block = 14064449874662041479;
    }
    match current_block {
        7245201122033322888 => {
            parent = (*elm).entry.rbe_parent;
            color = (*elm).entry.rbe_color;
            if !child.is_null() {
                (*child).entry.rbe_parent = parent;
            }
            if !parent.is_null() {
                if (*parent).entry.rbe_left == elm {
                    (*parent).entry.rbe_left = child;
                } else {
                    (*parent).entry.rbe_right = child;
                }
            } else {
                (*head).rbh_root = child;
            }
        }
        _ => {}
    }
    if color == RB_BLACK {
        wait_channels_RB_REMOVE_COLOR(head, parent, child);
    }
    return old;
}
unsafe extern "C" fn wait_channels_RB_REMOVE_COLOR(
    mut head: *mut wait_channels,
    mut parent: *mut wait_channel,
    mut elm: *mut wait_channel,
) {
    let mut tmp: *mut wait_channel = ::core::ptr::null_mut::<wait_channel>();
    while (elm.is_null() || (*elm).entry.rbe_color == RB_BLACK) && elm != (*head).rbh_root {
        if (*parent).entry.rbe_left == elm {
            tmp = (*parent).entry.rbe_right;
            if (*tmp).entry.rbe_color == RB_RED {
                (*tmp).entry.rbe_color = RB_BLACK;
                (*parent).entry.rbe_color = RB_RED;
                tmp = (*parent).entry.rbe_right;
                (*parent).entry.rbe_right = (*tmp).entry.rbe_left;
                if !(*parent).entry.rbe_right.is_null() {
                    (*(*tmp).entry.rbe_left).entry.rbe_parent = parent;
                }
                (*tmp).entry.rbe_parent = (*parent).entry.rbe_parent;
                if !(*tmp).entry.rbe_parent.is_null() {
                    if parent == (*(*parent).entry.rbe_parent).entry.rbe_left {
                        (*(*parent).entry.rbe_parent).entry.rbe_left = tmp;
                    } else {
                        (*(*parent).entry.rbe_parent).entry.rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).entry.rbe_left = parent;
                (*parent).entry.rbe_parent = tmp;
                !(*tmp).entry.rbe_parent.is_null();
                tmp = (*parent).entry.rbe_right;
            }
            if ((*tmp).entry.rbe_left.is_null()
                || (*(*tmp).entry.rbe_left).entry.rbe_color == RB_BLACK)
                && ((*tmp).entry.rbe_right.is_null()
                    || (*(*tmp).entry.rbe_right).entry.rbe_color == RB_BLACK)
            {
                (*tmp).entry.rbe_color = RB_RED;
                elm = parent;
                parent = (*elm).entry.rbe_parent;
            } else {
                if (*tmp).entry.rbe_right.is_null()
                    || (*(*tmp).entry.rbe_right).entry.rbe_color == RB_BLACK
                {
                    let mut oleft: *mut wait_channel = ::core::ptr::null_mut::<wait_channel>();
                    oleft = (*tmp).entry.rbe_left;
                    if !oleft.is_null() {
                        (*oleft).entry.rbe_color = RB_BLACK;
                    }
                    (*tmp).entry.rbe_color = RB_RED;
                    oleft = (*tmp).entry.rbe_left;
                    (*tmp).entry.rbe_left = (*oleft).entry.rbe_right;
                    if !(*tmp).entry.rbe_left.is_null() {
                        (*(*oleft).entry.rbe_right).entry.rbe_parent = tmp;
                    }
                    (*oleft).entry.rbe_parent = (*tmp).entry.rbe_parent;
                    if !(*oleft).entry.rbe_parent.is_null() {
                        if tmp == (*(*tmp).entry.rbe_parent).entry.rbe_left {
                            (*(*tmp).entry.rbe_parent).entry.rbe_left = oleft;
                        } else {
                            (*(*tmp).entry.rbe_parent).entry.rbe_right = oleft;
                        }
                    } else {
                        (*head).rbh_root = oleft;
                    }
                    (*oleft).entry.rbe_right = tmp;
                    (*tmp).entry.rbe_parent = oleft;
                    !(*oleft).entry.rbe_parent.is_null();
                    tmp = (*parent).entry.rbe_right;
                }
                (*tmp).entry.rbe_color = (*parent).entry.rbe_color;
                (*parent).entry.rbe_color = RB_BLACK;
                if !(*tmp).entry.rbe_right.is_null() {
                    (*(*tmp).entry.rbe_right).entry.rbe_color = RB_BLACK;
                }
                tmp = (*parent).entry.rbe_right;
                (*parent).entry.rbe_right = (*tmp).entry.rbe_left;
                if !(*parent).entry.rbe_right.is_null() {
                    (*(*tmp).entry.rbe_left).entry.rbe_parent = parent;
                }
                (*tmp).entry.rbe_parent = (*parent).entry.rbe_parent;
                if !(*tmp).entry.rbe_parent.is_null() {
                    if parent == (*(*parent).entry.rbe_parent).entry.rbe_left {
                        (*(*parent).entry.rbe_parent).entry.rbe_left = tmp;
                    } else {
                        (*(*parent).entry.rbe_parent).entry.rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).entry.rbe_left = parent;
                (*parent).entry.rbe_parent = tmp;
                !(*tmp).entry.rbe_parent.is_null();
                elm = (*head).rbh_root;
                break;
            }
        } else {
            tmp = (*parent).entry.rbe_left;
            if (*tmp).entry.rbe_color == RB_RED {
                (*tmp).entry.rbe_color = RB_BLACK;
                (*parent).entry.rbe_color = RB_RED;
                tmp = (*parent).entry.rbe_left;
                (*parent).entry.rbe_left = (*tmp).entry.rbe_right;
                if !(*parent).entry.rbe_left.is_null() {
                    (*(*tmp).entry.rbe_right).entry.rbe_parent = parent;
                }
                (*tmp).entry.rbe_parent = (*parent).entry.rbe_parent;
                if !(*tmp).entry.rbe_parent.is_null() {
                    if parent == (*(*parent).entry.rbe_parent).entry.rbe_left {
                        (*(*parent).entry.rbe_parent).entry.rbe_left = tmp;
                    } else {
                        (*(*parent).entry.rbe_parent).entry.rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).entry.rbe_right = parent;
                (*parent).entry.rbe_parent = tmp;
                !(*tmp).entry.rbe_parent.is_null();
                tmp = (*parent).entry.rbe_left;
            }
            if ((*tmp).entry.rbe_left.is_null()
                || (*(*tmp).entry.rbe_left).entry.rbe_color == RB_BLACK)
                && ((*tmp).entry.rbe_right.is_null()
                    || (*(*tmp).entry.rbe_right).entry.rbe_color == RB_BLACK)
            {
                (*tmp).entry.rbe_color = RB_RED;
                elm = parent;
                parent = (*elm).entry.rbe_parent;
            } else {
                if (*tmp).entry.rbe_left.is_null()
                    || (*(*tmp).entry.rbe_left).entry.rbe_color == RB_BLACK
                {
                    let mut oright: *mut wait_channel = ::core::ptr::null_mut::<wait_channel>();
                    oright = (*tmp).entry.rbe_right;
                    if !oright.is_null() {
                        (*oright).entry.rbe_color = RB_BLACK;
                    }
                    (*tmp).entry.rbe_color = RB_RED;
                    oright = (*tmp).entry.rbe_right;
                    (*tmp).entry.rbe_right = (*oright).entry.rbe_left;
                    if !(*tmp).entry.rbe_right.is_null() {
                        (*(*oright).entry.rbe_left).entry.rbe_parent = tmp;
                    }
                    (*oright).entry.rbe_parent = (*tmp).entry.rbe_parent;
                    if !(*oright).entry.rbe_parent.is_null() {
                        if tmp == (*(*tmp).entry.rbe_parent).entry.rbe_left {
                            (*(*tmp).entry.rbe_parent).entry.rbe_left = oright;
                        } else {
                            (*(*tmp).entry.rbe_parent).entry.rbe_right = oright;
                        }
                    } else {
                        (*head).rbh_root = oright;
                    }
                    (*oright).entry.rbe_left = tmp;
                    (*tmp).entry.rbe_parent = oright;
                    !(*oright).entry.rbe_parent.is_null();
                    tmp = (*parent).entry.rbe_left;
                }
                (*tmp).entry.rbe_color = (*parent).entry.rbe_color;
                (*parent).entry.rbe_color = RB_BLACK;
                if !(*tmp).entry.rbe_left.is_null() {
                    (*(*tmp).entry.rbe_left).entry.rbe_color = RB_BLACK;
                }
                tmp = (*parent).entry.rbe_left;
                (*parent).entry.rbe_left = (*tmp).entry.rbe_right;
                if !(*parent).entry.rbe_left.is_null() {
                    (*(*tmp).entry.rbe_right).entry.rbe_parent = parent;
                }
                (*tmp).entry.rbe_parent = (*parent).entry.rbe_parent;
                if !(*tmp).entry.rbe_parent.is_null() {
                    if parent == (*(*parent).entry.rbe_parent).entry.rbe_left {
                        (*(*parent).entry.rbe_parent).entry.rbe_left = tmp;
                    } else {
                        (*(*parent).entry.rbe_parent).entry.rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).entry.rbe_right = parent;
                (*parent).entry.rbe_parent = tmp;
                !(*tmp).entry.rbe_parent.is_null();
                elm = (*head).rbh_root;
                break;
            }
        }
    }
    if !elm.is_null() {
        (*elm).entry.rbe_color = RB_BLACK;
    }
}
unsafe extern "C" fn wait_channel_cmp(
    mut wc1: *mut wait_channel,
    mut wc2: *mut wait_channel,
) -> ::core::ffi::c_int {
    return strcmp((*wc1).name, (*wc2).name);
}
unsafe extern "C" fn cmd_wait_for_add(mut name: *const ::core::ffi::c_char) -> *mut wait_channel {
    let mut wc: *mut wait_channel = ::core::ptr::null_mut::<wait_channel>();
    wc = xmalloc(::core::mem::size_of::<wait_channel>() as size_t) as *mut wait_channel;
    (*wc).name = xstrdup(name);
    (*wc).locked = 0 as ::core::ffi::c_int;
    (*wc).woken = 0 as ::core::ffi::c_int;
    (*wc).waiters.tqh_first = ::core::ptr::null_mut::<wait_item>();
    (*wc).waiters.tqh_last = &raw mut (*wc).waiters.tqh_first;
    (*wc).lockers.tqh_first = ::core::ptr::null_mut::<wait_item>();
    (*wc).lockers.tqh_last = &raw mut (*wc).lockers.tqh_first;
    wait_channels_RB_INSERT(&raw mut wait_channels, wc);
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
    wait_channels_RB_REMOVE(&raw mut wait_channels, wc);
    free((*wc).name as *mut ::core::ffi::c_void);
    free(wc as *mut ::core::ffi::c_void);
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
    wait_channels_RB_REMOVE(&raw mut wait_channels, wc);
    free((*wc).name as *mut ::core::ffi::c_void);
    free(wc as *mut ::core::ffi::c_void);
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
    let mut find: wait_channel = wait_channel {
        name: name,
        locked: 0,
        woken: 0,
        waiters: C2RustUnnamed_38 {
            tqh_first: ::core::ptr::null_mut::<wait_item>(),
            tqh_last: ::core::ptr::null_mut::<*mut wait_item>(),
        },
        lockers: C2RustUnnamed_36 {
            tqh_first: ::core::ptr::null_mut::<wait_item>(),
            tqh_last: ::core::ptr::null_mut::<*mut wait_item>(),
        },
        entry: C2RustUnnamed_35 {
            rbe_left: ::core::ptr::null_mut::<wait_channel>(),
            rbe_right: ::core::ptr::null_mut::<wait_channel>(),
            rbe_parent: ::core::ptr::null_mut::<wait_channel>(),
            rbe_color: 0,
        },
    };
    if args_has(args, 'E' as i32 as u_char) != 0 {
        return cmd_wait_for_event(item, name, args);
    }
    wc = wait_channels_RB_FIND(&raw mut wait_channels, &raw mut find);
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
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    epi = event_payload_first(ep);
    while !epi.is_null() {
        key = event_payload_item_name(epi);
        if *key as ::core::ffi::c_int != '_' as i32 {
            value = event_payload_item_print(epi);
            cmdq_print(
                (*wei).item,
                b"%s=%s\0" as *const u8 as *const ::core::ffi::c_char,
                key,
                value,
            );
            free(value as *mut ::core::ffi::c_void);
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
    free((*wei).name as *mut ::core::ffi::c_void);
    free((*wei).filter as *mut ::core::ffi::c_void);
    free(wei as *mut ::core::ffi::c_void);
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
    wei = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<wait_event_item>() as size_t,
    ) as *mut wait_event_item;
    (*wei).item = item;
    (*wei).name = xstrdup(name);
    (*wei).filter = if !filter.is_null() {
        xstrdup(filter)
    } else {
        ::core::ptr::null_mut::<::core::ffi::c_char>()
    };
    (*wei).verbose = args_has(args, 'v' as i32 as u_char);
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
                free(wi as *mut ::core::ffi::c_void);
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
                free(wi as *mut ::core::ffi::c_void);
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
        free(wi as *mut ::core::ffi::c_void);
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
    wi = xcalloc(1 as size_t, ::core::mem::size_of::<wait_item>() as size_t) as *mut wait_item;
    (*wi).item = item;
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
        wi = xcalloc(1 as size_t, ::core::mem::size_of::<wait_item>() as size_t) as *mut wait_item;
        (*wi).item = item;
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
        free(wi as *mut ::core::ffi::c_void);
    } else {
        (*wc).locked = 0 as ::core::ffi::c_int;
        cmd_wait_for_remove(wc);
    }
    return CMD_RETURN_NORMAL;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_wait_for_flush() {
    let mut wc: *mut wait_channel = ::core::ptr::null_mut::<wait_channel>();
    let mut wc1: *mut wait_channel = ::core::ptr::null_mut::<wait_channel>();
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
    wc = wait_channels_RB_MINMAX(&raw mut wait_channels, RB_NEGINF);
    while !wc.is_null() && {
        wc1 = wait_channels_RB_NEXT(wc);
        1 as ::core::ffi::c_int != 0
    } {
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
            free(wi as *mut ::core::ffi::c_void);
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
            free(wi as *mut ::core::ffi::c_void);
            wi = wi1;
        }
        (*wc).locked = 0 as ::core::ffi::c_int;
        cmd_wait_for_remove(wc);
        wc = wc1;
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
