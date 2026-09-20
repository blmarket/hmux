use crate::src::ffi::libc::{
    __errno_location, close, dup, fclose, ferror, fopen, fread, free, fwrite, memcpy, open,
    strcmp, strlen, strncmp,
};
use crate::src::ffi::libevent::{
    bufferevent_enable, bufferevent_free, bufferevent_new, bufferevent_write, evbuffer_add,
    evbuffer_add_vprintf, evbuffer_drain, evbuffer_free, evbuffer_get_length, evbuffer_new,
    evbuffer_pullup, event_once,
};
use crate::src::log::{fatalx, log_debug};
use crate::src::proc::proc_send;
use crate::src::server_client::{server_client_get_cwd, server_client_unref};
use crate::src::tmux::find_home;
use crate::src::xmalloc::{xasprintf, xcalloc, xmalloc, xrealloc, xstrdup};
pub use crate::src::shared::arguments::{args};
pub use crate::src::shared::client::{
    client, client_entry, client_file, client_file_cb, client_file_entry, client_files,
    overlay_check_cb, overlay_draw_cb, overlay_free_cb, overlay_key_cb, overlay_mode_cb,
    overlay_resize_cb,
};
pub use crate::src::shared::command::{cmd_find_state, cmd_list, cmdq_item, cmdq_list, cmds};
pub use crate::src::shared::control::{control_state};
pub use crate::src::shared::format::{format_job_tree, format_tree};
pub use crate::src::shared::input::{input_ctx, input_request, input_requests};
pub use crate::src::shared::key::{
    key_binding, key_binding_entry, key_bindings, key_event, key_table, key_table_entry,
};
pub use crate::src::shared::layout::{layout_cell, layout_cell_entry, layout_cells};
pub use crate::src::shared::menu::{menu_data};
pub use crate::src::shared::message::{ibuf, ibuf_entry, imsg};
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
pub use crate::src::shared::message::{IMSG_HEADER_SIZE, MAX_IMSGSIZE, imsg_hdr};
pub use crate::src::shared::posix_io::{
    O_APPEND, O_CREAT, O_NONBLOCK, O_WRONLY, STDERR_FILENO, STDIN_FILENO, STDOUT_FILENO,
};
pub use crate::src::shared::errno::{E2BIG, EINVAL, ENOMEM};
pub use crate::src::shared::variadic::{__builtin_va_list, __gnuc_va_list, __va_list_tag, va_list};
pub use crate::src::shared::stdio::{
    FILE, _IO_FILE, _IO_codecvt, _IO_lock_t, _IO_marker, _IO_wide_data,
};
pub use crate::src::shared::abi::{__off64_t, __off_t, __uint32_t, ssize_t, uint32_t};
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::{screen_write_cline};
pub use crate::src::shared::hyperlinks::{hyperlinks};
pub use crate::src::shared::pane::{
    window_pane_offset, window_pane_resize, window_pane_resize_entry, window_pane_resizes,
};
pub use crate::src::shared::tree::{RB_BLACK, RB_NEGINF, RB_RED};
pub use crate::src::shared::event::{EV_READ, EV_TIMEOUT, EV_WRITE};
pub use crate::src::shared::client::{
    CLIENT_ATTACHED, CLIENT_CONTROL, CLIENT_DEAD, CLIENT_WRITE_ACK,
};
pub use crate::src::shared::display::{visible_range, visible_ranges};
pub use crate::src::shared::layout::{layout_geometry};
pub use crate::src::shared::mouse::{mouse_event};
use crate::src::shared::client::*;
use crate::src::shared::terminal::*;
use crate::src::shared::event::*;
use crate::src::shared::display::*;
use crate::src::shared::layout::*;
use crate::src::shared::message::*;
use crate::src::shared::abi::*;
use crate::src::shared::colour::*;
use crate::src::shared::grid::*;
use crate::src::shared::key::*;
use crate::src::shared::style::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct msg_read_open {
    pub stream: ::core::ffi::c_int,
    pub fd: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct msg_read_data {
    pub stream: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct msg_read_done {
    pub stream: ::core::ffi::c_int,
    pub error: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct msg_read_cancel {
    pub stream: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct msg_write_open {
    pub stream: ::core::ffi::c_int,
    pub fd: ::core::ffi::c_int,
    pub flags: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct msg_write_data {
    pub stream: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct msg_write_ready {
    pub stream: ::core::ffi::c_int,
    pub error: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct msg_write_close {
    pub stream: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct msg_write_done {
    pub stream: ::core::ffi::c_int,
    pub error: ::core::ffi::c_int,
}

pub use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_13;
pub use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_14;

pub const EIO: ::core::ffi::c_int = 5 as ::core::ffi::c_int;

pub const EBADF: ::core::ffi::c_int = 9 as ::core::ffi::c_int;

pub const O_RDONLY: ::core::ffi::c_int = 0 as ::core::ffi::c_int;

pub const BEV_EVENT_ERROR: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const EVBUFFER_ERROR: ::core::ffi::c_int = BEV_EVENT_ERROR;

static mut file_next_stream: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
#[no_mangle]
pub unsafe extern "C" fn client_files_RB_PREV(mut elm: *mut client_file) -> *mut client_file {
    if !(*elm).entry.rbe_left.is_null() {
        elm = (*elm).entry.rbe_left;
        while !(*elm).entry.rbe_right.is_null() {
            elm = (*elm).entry.rbe_right;
        }
    } else if !(*elm).entry.rbe_parent.is_null()
        && elm == (*(*elm).entry.rbe_parent).entry.rbe_right
    {
        elm = (*elm).entry.rbe_parent;
    } else {
        while !(*elm).entry.rbe_parent.is_null() && elm == (*(*elm).entry.rbe_parent).entry.rbe_left
        {
            elm = (*elm).entry.rbe_parent;
        }
        elm = (*elm).entry.rbe_parent;
    }
    return elm;
}
#[no_mangle]
pub unsafe extern "C" fn client_files_RB_MINMAX(
    mut head: *mut client_files,
    mut val: ::core::ffi::c_int,
) -> *mut client_file {
    let mut tmp: *mut client_file = (*head).rbh_root;
    let mut parent: *mut client_file = ::core::ptr::null_mut::<client_file>();
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
#[no_mangle]
pub unsafe extern "C" fn client_files_RB_NFIND(
    mut head: *mut client_files,
    mut elm: *mut client_file,
) -> *mut client_file {
    let mut tmp: *mut client_file = (*head).rbh_root;
    let mut res: *mut client_file = ::core::ptr::null_mut::<client_file>();
    let mut comp: ::core::ffi::c_int = 0;
    while !tmp.is_null() {
        comp = file_cmp(elm, tmp);
        if comp < 0 as ::core::ffi::c_int {
            res = tmp;
            tmp = (*tmp).entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_right;
        } else {
            return tmp;
        }
    }
    return res;
}
#[no_mangle]
pub unsafe extern "C" fn client_files_RB_REMOVE(
    mut head: *mut client_files,
    mut elm: *mut client_file,
) -> *mut client_file {
    let mut current_block: u64;
    let mut child: *mut client_file = ::core::ptr::null_mut::<client_file>();
    let mut parent: *mut client_file = ::core::ptr::null_mut::<client_file>();
    let mut old: *mut client_file = elm;
    let mut color: ::core::ffi::c_int = 0;
    if (*elm).entry.rbe_left.is_null() {
        child = (*elm).entry.rbe_right;
        current_block = 7245201122033322888;
    } else if (*elm).entry.rbe_right.is_null() {
        child = (*elm).entry.rbe_left;
        current_block = 7245201122033322888;
    } else {
        let mut left: *mut client_file = ::core::ptr::null_mut::<client_file>();
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
        current_block = 7372538432882326502;
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
        client_files_RB_REMOVE_COLOR(head, parent, child);
    }
    return old;
}
#[no_mangle]
pub unsafe extern "C" fn client_files_RB_INSERT(
    mut head: *mut client_files,
    mut elm: *mut client_file,
) -> *mut client_file {
    let mut tmp: *mut client_file = ::core::ptr::null_mut::<client_file>();
    let mut parent: *mut client_file = ::core::ptr::null_mut::<client_file>();
    let mut comp: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    tmp = (*head).rbh_root;
    while !tmp.is_null() {
        parent = tmp;
        comp = file_cmp(elm, parent);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_right;
        } else {
            return tmp;
        }
    }
    (*elm).entry.rbe_parent = parent;
    (*elm).entry.rbe_right = ::core::ptr::null_mut::<client_file>();
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
    client_files_RB_INSERT_COLOR(head, elm);
    return ::core::ptr::null_mut::<client_file>();
}
#[no_mangle]
pub unsafe extern "C" fn client_files_RB_INSERT_COLOR(
    mut head: *mut client_files,
    mut elm: *mut client_file,
) {
    let mut parent: *mut client_file = ::core::ptr::null_mut::<client_file>();
    let mut gparent: *mut client_file = ::core::ptr::null_mut::<client_file>();
    let mut tmp: *mut client_file = ::core::ptr::null_mut::<client_file>();
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
#[no_mangle]
pub unsafe extern "C" fn client_files_RB_REMOVE_COLOR(
    mut head: *mut client_files,
    mut parent: *mut client_file,
    mut elm: *mut client_file,
) {
    let mut tmp: *mut client_file = ::core::ptr::null_mut::<client_file>();
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
                    let mut oleft: *mut client_file = ::core::ptr::null_mut::<client_file>();
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
                    let mut oright: *mut client_file = ::core::ptr::null_mut::<client_file>();
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
#[no_mangle]
pub unsafe extern "C" fn client_files_RB_FIND(
    mut head: *mut client_files,
    mut elm: *mut client_file,
) -> *mut client_file {
    let mut tmp: *mut client_file = (*head).rbh_root;
    let mut comp: ::core::ffi::c_int = 0;
    while !tmp.is_null() {
        comp = file_cmp(elm, tmp);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_right;
        } else {
            return tmp;
        }
    }
    return ::core::ptr::null_mut::<client_file>();
}
#[no_mangle]
pub unsafe extern "C" fn client_files_RB_NEXT(mut elm: *mut client_file) -> *mut client_file {
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
unsafe extern "C" fn file_get_path(
    mut c: *mut client,
    mut file: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut home: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut path: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut full_path: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if strncmp(
        file,
        b"~/\0" as *const u8 as *const ::core::ffi::c_char,
        2 as size_t,
    ) != 0 as ::core::ffi::c_int
    {
        path = xstrdup(file);
    } else {
        home = find_home();
        if home.is_null() {
            home = b"\0" as *const u8 as *const ::core::ffi::c_char;
        }
        xasprintf(
            &raw mut path,
            b"%s%s\0" as *const u8 as *const ::core::ffi::c_char,
            home,
            file.offset(1 as ::core::ffi::c_int as isize),
        );
    }
    if *path as ::core::ffi::c_int == '/' as i32 {
        return path;
    }
    xasprintf(
        &raw mut full_path,
        b"%s/%s\0" as *const u8 as *const ::core::ffi::c_char,
        server_client_get_cwd(c, ::core::ptr::null_mut::<session>()),
        path,
    );
    free(path as *mut ::core::ffi::c_void);
    return full_path;
}
#[no_mangle]
pub unsafe extern "C" fn file_cmp(
    mut cf1: *mut client_file,
    mut cf2: *mut client_file,
) -> ::core::ffi::c_int {
    if (*cf1).stream < (*cf2).stream {
        return -(1 as ::core::ffi::c_int);
    }
    if (*cf1).stream > (*cf2).stream {
        return 1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn file_create_with_peer(
    mut peer: *mut tmuxpeer,
    mut files: *mut client_files,
    mut stream: ::core::ffi::c_int,
    mut cb: client_file_cb,
    mut cbdata: *mut ::core::ffi::c_void,
) -> *mut client_file {
    let mut cf: *mut client_file = ::core::ptr::null_mut::<client_file>();
    cf = xcalloc(1 as size_t, ::core::mem::size_of::<client_file>() as size_t) as *mut client_file;
    (*cf).c = ::core::ptr::null_mut::<client>();
    (*cf).references = 1 as ::core::ffi::c_int;
    (*cf).stream = stream;
    (*cf).buffer = evbuffer_new();
    if (*cf).buffer.is_null() {
        fatalx(b"out of memory\0" as *const u8 as *const ::core::ffi::c_char);
    }
    (*cf).cb = cb;
    (*cf).data = cbdata;
    (*cf).peer = peer;
    (*cf).tree = files as *mut client_files;
    client_files_RB_INSERT(files, cf);
    return cf;
}
#[no_mangle]
pub unsafe extern "C" fn file_create_with_client(
    mut c: *mut client,
    mut stream: ::core::ffi::c_int,
    mut cb: client_file_cb,
    mut cbdata: *mut ::core::ffi::c_void,
) -> *mut client_file {
    let mut cf: *mut client_file = ::core::ptr::null_mut::<client_file>();
    if !c.is_null() && (*c).flags & CLIENT_ATTACHED as uint64_t != 0 {
        c = ::core::ptr::null_mut::<client>();
    }
    cf = xcalloc(1 as size_t, ::core::mem::size_of::<client_file>() as size_t) as *mut client_file;
    (*cf).c = c;
    (*cf).references = 1 as ::core::ffi::c_int;
    (*cf).stream = stream;
    (*cf).buffer = evbuffer_new();
    if (*cf).buffer.is_null() {
        fatalx(b"out of memory\0" as *const u8 as *const ::core::ffi::c_char);
    }
    (*cf).cb = cb;
    (*cf).data = cbdata;
    if !(*cf).c.is_null() {
        (*cf).peer = (*(*cf).c).peer;
        (*cf).tree = &raw mut (*(*cf).c).files as *mut client_files;
        client_files_RB_INSERT(&raw mut (*(*cf).c).files, cf);
        (*(*cf).c).references += 1;
    }
    return cf;
}
#[no_mangle]
pub unsafe extern "C" fn file_free(mut cf: *mut client_file) {
    (*cf).references -= 1;
    if (*cf).references != 0 as ::core::ffi::c_int {
        return;
    }
    evbuffer_free((*cf).buffer);
    free((*cf).path as *mut ::core::ffi::c_void);
    if !(*cf).tree.is_null() {
        client_files_RB_REMOVE((*cf).tree as *mut client_files, cf);
    }
    if !(*cf).c.is_null() {
        server_client_unref((*cf).c);
    }
    free(cf as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn file_fire_done_cb(
    mut fd: ::core::ffi::c_int,
    mut events: ::core::ffi::c_short,
    mut arg: *mut ::core::ffi::c_void,
) {
    let mut cf: *mut client_file = arg as *mut client_file;
    let mut c: *mut client = (*cf).c;
    if (*cf).cb.is_some()
        && ((*cf).closed != 0 || c.is_null() || !(*c).flags & CLIENT_DEAD as uint64_t != 0)
    {
        (*cf).cb.expect("non-null function pointer")(
            c,
            (*cf).path,
            (*cf).error,
            1 as ::core::ffi::c_int,
            (*cf).buffer,
            (*cf).data,
        );
    }
    file_free(cf);
}
#[no_mangle]
pub unsafe extern "C" fn file_fire_done(mut cf: *mut client_file) {
    event_once(
        -(1 as ::core::ffi::c_int),
        EV_TIMEOUT as ::core::ffi::c_short,
        Some(
            file_fire_done_cb
                as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    ::core::ffi::c_short,
                    *mut ::core::ffi::c_void,
                ) -> (),
        ),
        cf as *mut ::core::ffi::c_void,
        ::core::ptr::null::<timeval>(),
    );
}
#[no_mangle]
pub unsafe extern "C" fn file_fire_read(mut cf: *mut client_file) {
    if (*cf).cb.is_some() {
        (*cf).cb.expect("non-null function pointer")(
            (*cf).c,
            (*cf).path,
            (*cf).error,
            0 as ::core::ffi::c_int,
            (*cf).buffer,
            (*cf).data,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn file_can_print(mut c: *mut client) -> ::core::ffi::c_int {
    if c.is_null()
        || (*c).flags & CLIENT_ATTACHED as uint64_t != 0
        || (*c).flags & CLIENT_DEAD as uint64_t != 0
        || (*c).flags & CLIENT_CONTROL as uint64_t != 0
    {
        return 0 as ::core::ffi::c_int;
    }
    return 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn file_print(
    mut c: *mut client,
    mut fmt: *const ::core::ffi::c_char,
    mut args: ...
) {
    let mut ap: ::core::ffi::VaList;
    ap = args.clone();
    file_vprint(c, fmt, ap);
}
#[no_mangle]
pub unsafe extern "C" fn file_vprint(
    mut c: *mut client,
    mut fmt: *const ::core::ffi::c_char,
    mut ap: ::core::ffi::VaList,
) {
    let mut find: client_file = client_file {
        c: ::core::ptr::null_mut::<client>(),
        peer: ::core::ptr::null_mut::<tmuxpeer>(),
        tree: ::core::ptr::null_mut::<client_files>(),
        references: 0,
        stream: 0,
        path: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        buffer: ::core::ptr::null_mut::<evbuffer>(),
        event: ::core::ptr::null_mut::<bufferevent>(),
        fd: 0,
        error: 0,
        closed: 0,
        cb: None,
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        entry: client_file_entry {
            rbe_left: ::core::ptr::null_mut::<client_file>(),
            rbe_right: ::core::ptr::null_mut::<client_file>(),
            rbe_parent: ::core::ptr::null_mut::<client_file>(),
            rbe_color: 0,
        },
    };
    let mut cf: *mut client_file = ::core::ptr::null_mut::<client_file>();
    let mut msg: msg_write_open = msg_write_open {
        stream: 0,
        fd: 0,
        flags: 0,
    };
    if file_can_print(c) == 0 {
        return;
    }
    find.stream = 1 as ::core::ffi::c_int;
    cf = client_files_RB_FIND(&raw mut (*c).files, &raw mut find);
    if cf.is_null() {
        cf = file_create_with_client(c, 1 as ::core::ffi::c_int, None, NULL);
        (*cf).path = xstrdup(b"-\0" as *const u8 as *const ::core::ffi::c_char);
        evbuffer_add_vprintf((*cf).buffer, fmt, ap);
        msg.stream = 1 as ::core::ffi::c_int;
        msg.fd = STDOUT_FILENO;
        msg.flags = 0 as ::core::ffi::c_int;
        proc_send(
            (*c).peer,
            MSG_WRITE_OPEN,
            -(1 as ::core::ffi::c_int),
            &raw mut msg as *const ::core::ffi::c_void,
            ::core::mem::size_of::<msg_write_open>() as size_t,
        );
    } else {
        evbuffer_add_vprintf((*cf).buffer, fmt, ap);
        file_push(cf);
    };
}
#[no_mangle]
pub unsafe extern "C" fn file_print_buffer(
    mut c: *mut client,
    mut data: *mut ::core::ffi::c_void,
    mut size: size_t,
) {
    let mut find: client_file = client_file {
        c: ::core::ptr::null_mut::<client>(),
        peer: ::core::ptr::null_mut::<tmuxpeer>(),
        tree: ::core::ptr::null_mut::<client_files>(),
        references: 0,
        stream: 0,
        path: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        buffer: ::core::ptr::null_mut::<evbuffer>(),
        event: ::core::ptr::null_mut::<bufferevent>(),
        fd: 0,
        error: 0,
        closed: 0,
        cb: None,
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        entry: client_file_entry {
            rbe_left: ::core::ptr::null_mut::<client_file>(),
            rbe_right: ::core::ptr::null_mut::<client_file>(),
            rbe_parent: ::core::ptr::null_mut::<client_file>(),
            rbe_color: 0,
        },
    };
    let mut cf: *mut client_file = ::core::ptr::null_mut::<client_file>();
    let mut msg: msg_write_open = msg_write_open {
        stream: 0,
        fd: 0,
        flags: 0,
    };
    if file_can_print(c) == 0 {
        return;
    }
    find.stream = 1 as ::core::ffi::c_int;
    cf = client_files_RB_FIND(&raw mut (*c).files, &raw mut find);
    if cf.is_null() {
        cf = file_create_with_client(c, 1 as ::core::ffi::c_int, None, NULL);
        (*cf).path = xstrdup(b"-\0" as *const u8 as *const ::core::ffi::c_char);
        evbuffer_add((*cf).buffer, data, size);
        msg.stream = 1 as ::core::ffi::c_int;
        msg.fd = STDOUT_FILENO;
        msg.flags = 0 as ::core::ffi::c_int;
        proc_send(
            (*c).peer,
            MSG_WRITE_OPEN,
            -(1 as ::core::ffi::c_int),
            &raw mut msg as *const ::core::ffi::c_void,
            ::core::mem::size_of::<msg_write_open>() as size_t,
        );
    } else {
        evbuffer_add((*cf).buffer, data, size);
        file_push(cf);
    };
}
#[no_mangle]
pub unsafe extern "C" fn file_error(
    mut c: *mut client,
    mut fmt: *const ::core::ffi::c_char,
    mut args: ...
) {
    let mut find: client_file = client_file {
        c: ::core::ptr::null_mut::<client>(),
        peer: ::core::ptr::null_mut::<tmuxpeer>(),
        tree: ::core::ptr::null_mut::<client_files>(),
        references: 0,
        stream: 0,
        path: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        buffer: ::core::ptr::null_mut::<evbuffer>(),
        event: ::core::ptr::null_mut::<bufferevent>(),
        fd: 0,
        error: 0,
        closed: 0,
        cb: None,
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        entry: client_file_entry {
            rbe_left: ::core::ptr::null_mut::<client_file>(),
            rbe_right: ::core::ptr::null_mut::<client_file>(),
            rbe_parent: ::core::ptr::null_mut::<client_file>(),
            rbe_color: 0,
        },
    };
    let mut cf: *mut client_file = ::core::ptr::null_mut::<client_file>();
    let mut msg: msg_write_open = msg_write_open {
        stream: 0,
        fd: 0,
        flags: 0,
    };
    let mut ap: ::core::ffi::VaList;
    if file_can_print(c) == 0 {
        return;
    }
    ap = args.clone();
    find.stream = 2 as ::core::ffi::c_int;
    cf = client_files_RB_FIND(&raw mut (*c).files, &raw mut find);
    if cf.is_null() {
        cf = file_create_with_client(c, 2 as ::core::ffi::c_int, None, NULL);
        (*cf).path = xstrdup(b"-\0" as *const u8 as *const ::core::ffi::c_char);
        evbuffer_add_vprintf((*cf).buffer, fmt, ap);
        msg.stream = 2 as ::core::ffi::c_int;
        msg.fd = STDERR_FILENO;
        msg.flags = 0 as ::core::ffi::c_int;
        proc_send(
            (*c).peer,
            MSG_WRITE_OPEN,
            -(1 as ::core::ffi::c_int),
            &raw mut msg as *const ::core::ffi::c_void,
            ::core::mem::size_of::<msg_write_open>() as size_t,
        );
    } else {
        evbuffer_add_vprintf((*cf).buffer, fmt, ap);
        file_push(cf);
    };
}
#[no_mangle]
pub unsafe extern "C" fn file_write(
    mut c: *mut client,
    mut path: *const ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
    mut bdata: *const ::core::ffi::c_void,
    mut bsize: size_t,
    mut cb: client_file_cb,
    mut cbdata: *mut ::core::ffi::c_void,
) {
    let mut current_block: u64;
    let mut cf: *mut client_file = ::core::ptr::null_mut::<client_file>();
    let mut msg: *mut msg_write_open = ::core::ptr::null_mut::<msg_write_open>();
    let mut msglen: size_t = 0;
    let mut fd: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
    let fresh0 = file_next_stream;
    file_next_stream = file_next_stream + 1;
    let mut stream: u_int = fresh0 as u_int;
    let mut f: *mut FILE = ::core::ptr::null_mut::<FILE>();
    let mut mode: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if strcmp(path, b"-\0" as *const u8 as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int {
        cf = file_create_with_client(c, stream as ::core::ffi::c_int, cb, cbdata);
        (*cf).path = xstrdup(b"-\0" as *const u8 as *const ::core::ffi::c_char);
        fd = STDOUT_FILENO;
        if c.is_null()
            || (*c).flags & CLIENT_ATTACHED as uint64_t != 0
            || (*c).flags & CLIENT_CONTROL as uint64_t != 0
        {
            (*cf).error = EBADF;
            current_block = 4636144702248558238;
        } else {
            current_block = 8821498768635335055;
        }
    } else {
        cf = file_create_with_client(c, stream as ::core::ffi::c_int, cb, cbdata);
        (*cf).path = file_get_path(c, path);
        if c.is_null() || (*c).flags & CLIENT_ATTACHED as uint64_t != 0 {
            if flags & O_APPEND != 0 {
                mode = b"ab\0" as *const u8 as *const ::core::ffi::c_char;
            } else {
                mode = b"wb\0" as *const u8 as *const ::core::ffi::c_char;
            }
            f = fopen((*cf).path, mode) as *mut FILE;
            if f.is_null() {
                (*cf).error = *__errno_location();
            } else if fwrite(bdata, 1 as size_t, bsize, f) as size_t != bsize {
                fclose(f);
                (*cf).error = EIO;
            } else {
                fclose(f);
            }
            current_block = 4636144702248558238;
        } else {
            current_block = 8821498768635335055;
        }
    }
    match current_block {
        8821498768635335055 => {
            evbuffer_add((*cf).buffer, bdata, bsize);
            msglen = strlen((*cf).path)
                .wrapping_add(1 as size_t)
                .wrapping_add(::core::mem::size_of::<msg_write_open>() as size_t);
            if msglen > (MAX_IMSGSIZE as usize).wrapping_sub(IMSG_HEADER_SIZE) {
                (*cf).error = E2BIG;
            } else {
                msg = xmalloc(msglen) as *mut msg_write_open;
                (*msg).stream = (*cf).stream;
                (*msg).fd = fd;
                (*msg).flags = flags;
                memcpy(
                    msg.offset(1 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
                    (*cf).path as *const ::core::ffi::c_void,
                    msglen.wrapping_sub(::core::mem::size_of::<msg_write_open>() as size_t),
                );
                if proc_send(
                    (*cf).peer,
                    MSG_WRITE_OPEN,
                    -(1 as ::core::ffi::c_int),
                    msg as *const ::core::ffi::c_void,
                    msglen,
                ) != 0 as ::core::ffi::c_int
                {
                    free(msg as *mut ::core::ffi::c_void);
                    (*cf).error = EINVAL;
                } else {
                    free(msg as *mut ::core::ffi::c_void);
                    return;
                }
            }
        }
        _ => {}
    }
    file_fire_done(cf);
}
#[no_mangle]
pub unsafe extern "C" fn file_read(
    mut c: *mut client,
    mut path: *const ::core::ffi::c_char,
    mut cb: client_file_cb,
    mut cbdata: *mut ::core::ffi::c_void,
) -> *mut client_file {
    let mut current_block: u64;
    let mut cf: *mut client_file = ::core::ptr::null_mut::<client_file>();
    let mut msg: *mut msg_read_open = ::core::ptr::null_mut::<msg_read_open>();
    let mut msglen: size_t = 0;
    let mut fd: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
    let fresh1 = file_next_stream;
    file_next_stream = file_next_stream + 1;
    let mut stream: u_int = fresh1 as u_int;
    let mut f: *mut FILE = ::core::ptr::null_mut::<FILE>();
    let mut size: size_t = 0;
    let mut buffer: [::core::ffi::c_char; 8192] = [0; 8192];
    if strcmp(path, b"-\0" as *const u8 as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int {
        cf = file_create_with_client(c, stream as ::core::ffi::c_int, cb, cbdata);
        (*cf).path = xstrdup(b"-\0" as *const u8 as *const ::core::ffi::c_char);
        fd = STDIN_FILENO;
        if c.is_null()
            || (*c).flags & CLIENT_ATTACHED as uint64_t != 0
            || (*c).flags & CLIENT_CONTROL as uint64_t != 0
        {
            (*cf).error = EBADF;
            current_block = 17369485759464587280;
        } else {
            current_block = 17710118112003399050;
        }
    } else {
        cf = file_create_with_client(c, stream as ::core::ffi::c_int, cb, cbdata);
        (*cf).path = file_get_path(c, path);
        if c.is_null() || (*c).flags & CLIENT_ATTACHED as uint64_t != 0 {
            f = fopen(
                (*cf).path,
                b"rb\0" as *const u8 as *const ::core::ffi::c_char,
            ) as *mut FILE;
            if f.is_null() {
                (*cf).error = *__errno_location();
            } else {
                loop {
                    size = fread(
                        &raw mut buffer as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
                        1 as size_t,
                        ::core::mem::size_of::<[::core::ffi::c_char; 8192]>() as size_t,
                        f,
                    ) as size_t;
                    if ferror(f) != 0 {
                        (*cf).error = *__errno_location();
                        current_block = 17369485759464587280;
                        break;
                    } else if evbuffer_add(
                        (*cf).buffer,
                        &raw mut buffer as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
                        size,
                    ) != 0 as ::core::ffi::c_int
                    {
                        (*cf).error = ENOMEM;
                        current_block = 17369485759464587280;
                        break;
                    } else if size != ::core::mem::size_of::<[::core::ffi::c_char; 8192]>() as usize
                    {
                        current_block = 4808432441040389987;
                        break;
                    }
                }
                match current_block {
                    17369485759464587280 => {}
                    _ => {
                        if ferror(f) != 0 {
                            (*cf).error = EIO;
                        }
                    }
                }
            }
            current_block = 17369485759464587280;
        } else {
            current_block = 17710118112003399050;
        }
    }
    match current_block {
        17710118112003399050 => {
            msglen = strlen((*cf).path)
                .wrapping_add(1 as size_t)
                .wrapping_add(::core::mem::size_of::<msg_read_open>() as size_t);
            if msglen > (MAX_IMSGSIZE as usize).wrapping_sub(IMSG_HEADER_SIZE) {
                (*cf).error = E2BIG;
            } else {
                msg = xmalloc(msglen) as *mut msg_read_open;
                (*msg).stream = (*cf).stream;
                (*msg).fd = fd;
                memcpy(
                    msg.offset(1 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
                    (*cf).path as *const ::core::ffi::c_void,
                    msglen.wrapping_sub(::core::mem::size_of::<msg_read_open>() as size_t),
                );
                if proc_send(
                    (*cf).peer,
                    MSG_READ_OPEN,
                    -(1 as ::core::ffi::c_int),
                    msg as *const ::core::ffi::c_void,
                    msglen,
                ) != 0 as ::core::ffi::c_int
                {
                    free(msg as *mut ::core::ffi::c_void);
                    (*cf).error = EINVAL;
                } else {
                    free(msg as *mut ::core::ffi::c_void);
                    return cf;
                }
            }
        }
        _ => {}
    }
    if !f.is_null() {
        fclose(f);
    }
    file_fire_done(cf);
    return ::core::ptr::null_mut::<client_file>();
}
#[no_mangle]
pub unsafe extern "C" fn file_cancel(mut cf: *mut client_file) {
    let mut msg: msg_read_cancel = msg_read_cancel { stream: 0 };
    log_debug(
        b"read cancel file %d\0" as *const u8 as *const ::core::ffi::c_char,
        (*cf).stream,
    );
    if (*cf).closed != 0 {
        return;
    }
    (*cf).closed = 1 as ::core::ffi::c_int;
    msg.stream = (*cf).stream;
    proc_send(
        (*cf).peer,
        MSG_READ_CANCEL,
        -(1 as ::core::ffi::c_int),
        &raw mut msg as *const ::core::ffi::c_void,
        ::core::mem::size_of::<msg_read_cancel>() as size_t,
    );
}
unsafe extern "C" fn file_push_cb(
    mut fd: ::core::ffi::c_int,
    mut events: ::core::ffi::c_short,
    mut arg: *mut ::core::ffi::c_void,
) {
    let mut cf: *mut client_file = arg as *mut client_file;
    if (*cf).c.is_null() || !(*(*cf).c).flags & CLIENT_DEAD as uint64_t != 0 {
        file_push(cf);
    }
    file_free(cf);
}
#[no_mangle]
pub unsafe extern "C" fn file_push(mut cf: *mut client_file) {
    let mut msg: *mut msg_write_data = ::core::ptr::null_mut::<msg_write_data>();
    let mut msglen: size_t = 0;
    let mut sent: size_t = 0;
    let mut left: size_t = 0;
    let mut close_0: msg_write_close = msg_write_close { stream: 0 };
    msg = xmalloc(::core::mem::size_of::<msg_write_data>() as size_t) as *mut msg_write_data;
    left = evbuffer_get_length((*cf).buffer);
    while left != 0 as size_t {
        sent = left;
        if sent
            > (MAX_IMSGSIZE as usize)
                .wrapping_sub(IMSG_HEADER_SIZE)
                .wrapping_sub(::core::mem::size_of::<msg_write_data>() as usize)
        {
            sent = (MAX_IMSGSIZE as usize)
                .wrapping_sub(IMSG_HEADER_SIZE)
                .wrapping_sub(::core::mem::size_of::<msg_write_data>() as usize)
                as size_t;
        }
        msglen = (::core::mem::size_of::<msg_write_data>() as usize).wrapping_add(sent as usize)
            as size_t;
        msg = xrealloc(msg as *mut ::core::ffi::c_void, msglen) as *mut msg_write_data;
        (*msg).stream = (*cf).stream;
        memcpy(
            msg.offset(1 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
            evbuffer_pullup((*cf).buffer, -(1 as ::core::ffi::c_int) as ssize_t)
                as *const ::core::ffi::c_void,
            sent,
        );
        if proc_send(
            (*cf).peer,
            MSG_WRITE,
            -(1 as ::core::ffi::c_int),
            msg as *const ::core::ffi::c_void,
            msglen,
        ) != 0 as ::core::ffi::c_int
        {
            break;
        }
        evbuffer_drain((*cf).buffer, sent);
        left = evbuffer_get_length((*cf).buffer);
        log_debug(
            b"file %d sent %zu, left %zu\0" as *const u8 as *const ::core::ffi::c_char,
            (*cf).stream,
            sent,
            left,
        );
    }
    if left != 0 as size_t {
        (*cf).references += 1;
        event_once(
            -(1 as ::core::ffi::c_int),
            EV_TIMEOUT as ::core::ffi::c_short,
            Some(
                file_push_cb
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        ::core::ffi::c_short,
                        *mut ::core::ffi::c_void,
                    ) -> (),
            ),
            cf as *mut ::core::ffi::c_void,
            ::core::ptr::null::<timeval>(),
        );
    } else if (*cf).stream > 2 as ::core::ffi::c_int {
        close_0.stream = (*cf).stream;
        proc_send(
            (*cf).peer,
            MSG_WRITE_CLOSE,
            -(1 as ::core::ffi::c_int),
            &raw mut close_0 as *const ::core::ffi::c_void,
            ::core::mem::size_of::<msg_write_close>() as size_t,
        );
        if (*cf).c.is_null()
            || !(*(*cf).c).flags as ::core::ffi::c_ulonglong & CLIENT_WRITE_ACK != 0
        {
            file_fire_done(cf);
        }
    }
    free(msg as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn file_write_left(mut files: *mut client_files) -> ::core::ffi::c_int {
    let mut cf: *mut client_file = ::core::ptr::null_mut::<client_file>();
    let mut left: size_t = 0;
    let mut waiting: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    cf = client_files_RB_MINMAX(files, RB_NEGINF);
    while !cf.is_null() {
        if !(*cf).event.is_null() {
            left = evbuffer_get_length((*(*cf).event).output);
            if left != 0 as size_t {
                waiting += 1;
                log_debug(
                    b"file %u %zu bytes left\0" as *const u8 as *const ::core::ffi::c_char,
                    (*cf).stream,
                    left,
                );
            }
        }
        cf = client_files_RB_NEXT(cf);
    }
    return (waiting != 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
}
unsafe extern "C" fn file_write_finished(mut cf: *mut client_file) {
    let mut msg: msg_write_done = msg_write_done {
        stream: 0,
        error: 0,
    };
    if !(*cf).event.is_null() {
        bufferevent_free((*cf).event);
        (*cf).event = ::core::ptr::null_mut::<bufferevent>();
    }
    if (*cf).fd != -(1 as ::core::ffi::c_int) {
        if close((*cf).fd) != 0 as ::core::ffi::c_int && (*cf).error == 0 as ::core::ffi::c_int {
            (*cf).error = *__errno_location();
        }
        (*cf).fd = -(1 as ::core::ffi::c_int);
    }
    msg.stream = (*cf).stream;
    msg.error = (*cf).error;
    proc_send(
        (*cf).peer,
        MSG_WRITE_DONE,
        -(1 as ::core::ffi::c_int),
        &raw mut msg as *const ::core::ffi::c_void,
        ::core::mem::size_of::<msg_write_done>() as size_t,
    );
    if (*cf).cb.is_some() {
        (*cf).cb.expect("non-null function pointer")(
            ::core::ptr::null_mut::<client>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
            0 as ::core::ffi::c_int,
            -(1 as ::core::ffi::c_int),
            ::core::ptr::null_mut::<evbuffer>(),
            (*cf).data,
        );
    }
    file_free(cf);
}
unsafe extern "C" fn file_write_error_callback(
    mut bev: *mut bufferevent,
    mut what: ::core::ffi::c_short,
    mut arg: *mut ::core::ffi::c_void,
) {
    let mut cf: *mut client_file = arg as *mut client_file;
    let mut error: ::core::ffi::c_int = 0;
    if what as ::core::ffi::c_int & EVBUFFER_ERROR != 0 {
        error = *__errno_location();
    } else {
        error = EIO;
    }
    if error == 0 as ::core::ffi::c_int {
        error = EIO;
    }
    log_debug(
        b"write error file %d\0" as *const u8 as *const ::core::ffi::c_char,
        (*cf).stream,
    );
    (*cf).error = error;
    bufferevent_free((*cf).event);
    (*cf).event = ::core::ptr::null_mut::<bufferevent>();
    close((*cf).fd);
    (*cf).fd = -(1 as ::core::ffi::c_int);
    if (*cf).closed != 0 {
        file_write_finished(cf);
    } else if (*cf).cb.is_some() {
        (*cf).cb.expect("non-null function pointer")(
            ::core::ptr::null_mut::<client>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
            0 as ::core::ffi::c_int,
            -(1 as ::core::ffi::c_int),
            ::core::ptr::null_mut::<evbuffer>(),
            (*cf).data,
        );
    }
}
unsafe extern "C" fn file_write_callback(
    mut bev: *mut bufferevent,
    mut arg: *mut ::core::ffi::c_void,
) {
    let mut cf: *mut client_file = arg as *mut client_file;
    log_debug(
        b"write check file %d\0" as *const u8 as *const ::core::ffi::c_char,
        (*cf).stream,
    );
    if (*cf).closed != 0 && evbuffer_get_length((*(*cf).event).output) == 0 as size_t {
        file_write_finished(cf);
    } else if (*cf).cb.is_some() {
        (*cf).cb.expect("non-null function pointer")(
            ::core::ptr::null_mut::<client>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
            0 as ::core::ffi::c_int,
            -(1 as ::core::ffi::c_int),
            ::core::ptr::null_mut::<evbuffer>(),
            (*cf).data,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn file_write_open(
    mut files: *mut client_files,
    mut peer: *mut tmuxpeer,
    mut imsg: *mut imsg,
    mut allow_streams: ::core::ffi::c_int,
    mut close_received: ::core::ffi::c_int,
    mut cb: client_file_cb,
    mut cbdata: *mut ::core::ffi::c_void,
) {
    let mut msg: *mut msg_write_open = (*imsg).data as *mut msg_write_open;
    let mut msglen: size_t = ((*imsg).hdr.len as size_t).wrapping_sub(IMSG_HEADER_SIZE);
    let mut path: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut reply: msg_write_ready = msg_write_ready {
        stream: 0,
        error: 0,
    };
    let mut find: client_file = client_file {
        c: ::core::ptr::null_mut::<client>(),
        peer: ::core::ptr::null_mut::<tmuxpeer>(),
        tree: ::core::ptr::null_mut::<client_files>(),
        references: 0,
        stream: 0,
        path: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        buffer: ::core::ptr::null_mut::<evbuffer>(),
        event: ::core::ptr::null_mut::<bufferevent>(),
        fd: 0,
        error: 0,
        closed: 0,
        cb: None,
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        entry: client_file_entry {
            rbe_left: ::core::ptr::null_mut::<client_file>(),
            rbe_right: ::core::ptr::null_mut::<client_file>(),
            rbe_parent: ::core::ptr::null_mut::<client_file>(),
            rbe_color: 0,
        },
    };
    let mut cf: *mut client_file = ::core::ptr::null_mut::<client_file>();
    let flags: ::core::ffi::c_int = O_NONBLOCK | O_WRONLY | O_CREAT;
    let mut error: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if msglen < ::core::mem::size_of::<msg_write_open>() as usize {
        fatalx(b"bad MSG_WRITE_OPEN size\0" as *const u8 as *const ::core::ffi::c_char);
    }
    if msglen == ::core::mem::size_of::<msg_write_open>() as usize {
        path = b"-\0" as *const u8 as *const ::core::ffi::c_char;
    } else {
        path = msg.offset(1 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_char;
    }
    log_debug(
        b"open write file %d %s\0" as *const u8 as *const ::core::ffi::c_char,
        (*msg).stream,
        path,
    );
    find.stream = (*msg).stream;
    if !client_files_RB_FIND(files, &raw mut find).is_null() {
        error = EBADF;
    } else {
        cf = file_create_with_peer(peer, files, (*msg).stream, cb, cbdata);
        if (*cf).closed != 0 {
            error = EBADF;
        } else {
            (*cf).fd = -(1 as ::core::ffi::c_int);
            if (*msg).fd == -(1 as ::core::ffi::c_int) {
                (*cf).fd = open(path, (*msg).flags | flags, 0o644 as ::core::ffi::c_int);
            } else if allow_streams != 0 {
                if (*msg).fd != STDOUT_FILENO && (*msg).fd != STDERR_FILENO {
                    *__errno_location() = EBADF;
                } else {
                    (*cf).fd = dup((*msg).fd);
                    if close_received != 0 {
                        close((*msg).fd);
                    }
                }
            } else {
                *__errno_location() = EBADF;
            }
            if (*cf).fd == -(1 as ::core::ffi::c_int) {
                error = *__errno_location();
            } else {
                (*cf).event = bufferevent_new(
                    (*cf).fd,
                    None,
                    Some(
                        file_write_callback
                            as unsafe extern "C" fn(
                                *mut bufferevent,
                                *mut ::core::ffi::c_void,
                            ) -> (),
                    ),
                    Some(
                        file_write_error_callback
                            as unsafe extern "C" fn(
                                *mut bufferevent,
                                ::core::ffi::c_short,
                                *mut ::core::ffi::c_void,
                            ) -> (),
                    ),
                    cf as *mut ::core::ffi::c_void,
                );
                if (*cf).event.is_null() {
                    fatalx(b"out of memory\0" as *const u8 as *const ::core::ffi::c_char);
                }
                bufferevent_enable((*cf).event, EV_WRITE as ::core::ffi::c_short);
            }
        }
    }
    reply.stream = (*msg).stream;
    reply.error = error;
    proc_send(
        peer,
        MSG_WRITE_READY,
        -(1 as ::core::ffi::c_int),
        &raw mut reply as *const ::core::ffi::c_void,
        ::core::mem::size_of::<msg_write_ready>() as size_t,
    );
}
#[no_mangle]
pub unsafe extern "C" fn file_write_data(mut files: *mut client_files, mut imsg: *mut imsg) {
    let mut msg: *mut msg_write_data = (*imsg).data as *mut msg_write_data;
    let mut msglen: size_t = ((*imsg).hdr.len as size_t).wrapping_sub(IMSG_HEADER_SIZE);
    let mut find: client_file = client_file {
        c: ::core::ptr::null_mut::<client>(),
        peer: ::core::ptr::null_mut::<tmuxpeer>(),
        tree: ::core::ptr::null_mut::<client_files>(),
        references: 0,
        stream: 0,
        path: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        buffer: ::core::ptr::null_mut::<evbuffer>(),
        event: ::core::ptr::null_mut::<bufferevent>(),
        fd: 0,
        error: 0,
        closed: 0,
        cb: None,
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        entry: client_file_entry {
            rbe_left: ::core::ptr::null_mut::<client_file>(),
            rbe_right: ::core::ptr::null_mut::<client_file>(),
            rbe_parent: ::core::ptr::null_mut::<client_file>(),
            rbe_color: 0,
        },
    };
    let mut cf: *mut client_file = ::core::ptr::null_mut::<client_file>();
    let mut size: size_t = msglen.wrapping_sub(::core::mem::size_of::<msg_write_data>() as size_t);
    if msglen < ::core::mem::size_of::<msg_write_data>() as usize {
        fatalx(b"bad MSG_WRITE size\0" as *const u8 as *const ::core::ffi::c_char);
    }
    find.stream = (*msg).stream;
    cf = client_files_RB_FIND(files, &raw mut find);
    if cf.is_null() {
        fatalx(b"unknown stream number\0" as *const u8 as *const ::core::ffi::c_char);
    }
    log_debug(
        b"write %zu to file %d\0" as *const u8 as *const ::core::ffi::c_char,
        size,
        (*cf).stream,
    );
    if !(*cf).event.is_null() {
        bufferevent_write(
            (*cf).event,
            msg.offset(1 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
            size,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn file_write_close(mut files: *mut client_files, mut imsg: *mut imsg) {
    let mut msg: *mut msg_write_close = (*imsg).data as *mut msg_write_close;
    let mut msglen: size_t = ((*imsg).hdr.len as size_t).wrapping_sub(IMSG_HEADER_SIZE);
    let mut find: client_file = client_file {
        c: ::core::ptr::null_mut::<client>(),
        peer: ::core::ptr::null_mut::<tmuxpeer>(),
        tree: ::core::ptr::null_mut::<client_files>(),
        references: 0,
        stream: 0,
        path: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        buffer: ::core::ptr::null_mut::<evbuffer>(),
        event: ::core::ptr::null_mut::<bufferevent>(),
        fd: 0,
        error: 0,
        closed: 0,
        cb: None,
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        entry: client_file_entry {
            rbe_left: ::core::ptr::null_mut::<client_file>(),
            rbe_right: ::core::ptr::null_mut::<client_file>(),
            rbe_parent: ::core::ptr::null_mut::<client_file>(),
            rbe_color: 0,
        },
    };
    let mut cf: *mut client_file = ::core::ptr::null_mut::<client_file>();
    if msglen != ::core::mem::size_of::<msg_write_close>() as usize {
        fatalx(b"bad MSG_WRITE_CLOSE size\0" as *const u8 as *const ::core::ffi::c_char);
    }
    find.stream = (*msg).stream;
    cf = client_files_RB_FIND(files, &raw mut find);
    if cf.is_null() {
        fatalx(b"unknown stream number\0" as *const u8 as *const ::core::ffi::c_char);
    }
    log_debug(
        b"close file %d\0" as *const u8 as *const ::core::ffi::c_char,
        (*cf).stream,
    );
    (*cf).closed = 1 as ::core::ffi::c_int;
    if (*cf).event.is_null() || evbuffer_get_length((*(*cf).event).output) == 0 as size_t {
        file_write_finished(cf);
    }
}
unsafe extern "C" fn file_read_error_callback(
    mut bev: *mut bufferevent,
    mut what: ::core::ffi::c_short,
    mut arg: *mut ::core::ffi::c_void,
) {
    let mut cf: *mut client_file = arg as *mut client_file;
    let mut msg: msg_read_done = msg_read_done {
        stream: 0,
        error: 0,
    };
    log_debug(
        b"read error file %d\0" as *const u8 as *const ::core::ffi::c_char,
        (*cf).stream,
    );
    msg.stream = (*cf).stream;
    msg.error = if what as ::core::ffi::c_int & EVBUFFER_ERROR != 0 {
        EIO
    } else {
        0 as ::core::ffi::c_int
    };
    proc_send(
        (*cf).peer,
        MSG_READ_DONE,
        -(1 as ::core::ffi::c_int),
        &raw mut msg as *const ::core::ffi::c_void,
        ::core::mem::size_of::<msg_read_done>() as size_t,
    );
    bufferevent_free((*cf).event);
    close((*cf).fd);
    client_files_RB_REMOVE((*cf).tree as *mut client_files, cf);
    file_free(cf);
}
unsafe extern "C" fn file_read_callback(
    mut bev: *mut bufferevent,
    mut arg: *mut ::core::ffi::c_void,
) {
    let mut cf: *mut client_file = arg as *mut client_file;
    let mut bdata: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
    let mut bsize: size_t = 0;
    let mut msg: *mut msg_read_data = ::core::ptr::null_mut::<msg_read_data>();
    let mut msglen: size_t = 0;
    msg = xmalloc(::core::mem::size_of::<msg_read_data>() as size_t) as *mut msg_read_data;
    loop {
        bdata = evbuffer_pullup((*(*cf).event).input, -(1 as ::core::ffi::c_int) as ssize_t)
            as *mut ::core::ffi::c_void;
        bsize = evbuffer_get_length((*(*cf).event).input);
        if bsize == 0 as size_t {
            break;
        }
        if bsize
            > (MAX_IMSGSIZE as usize)
                .wrapping_sub(IMSG_HEADER_SIZE)
                .wrapping_sub(::core::mem::size_of::<msg_read_data>() as usize)
        {
            bsize = (MAX_IMSGSIZE as usize)
                .wrapping_sub(IMSG_HEADER_SIZE)
                .wrapping_sub(::core::mem::size_of::<msg_read_data>() as usize)
                as size_t;
        }
        log_debug(
            b"read %zu from file %d\0" as *const u8 as *const ::core::ffi::c_char,
            bsize,
            (*cf).stream,
        );
        msglen = (::core::mem::size_of::<msg_read_data>() as usize).wrapping_add(bsize as usize)
            as size_t;
        msg = xrealloc(msg as *mut ::core::ffi::c_void, msglen) as *mut msg_read_data;
        (*msg).stream = (*cf).stream;
        memcpy(
            msg.offset(1 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
            bdata,
            bsize,
        );
        proc_send(
            (*cf).peer,
            MSG_READ,
            -(1 as ::core::ffi::c_int),
            msg as *const ::core::ffi::c_void,
            msglen,
        );
        evbuffer_drain((*(*cf).event).input, bsize);
    }
    free(msg as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn file_read_open(
    mut files: *mut client_files,
    mut peer: *mut tmuxpeer,
    mut imsg: *mut imsg,
    mut allow_streams: ::core::ffi::c_int,
    mut close_received: ::core::ffi::c_int,
    mut cb: client_file_cb,
    mut cbdata: *mut ::core::ffi::c_void,
) {
    let mut msg: *mut msg_read_open = (*imsg).data as *mut msg_read_open;
    let mut msglen: size_t = ((*imsg).hdr.len as size_t).wrapping_sub(IMSG_HEADER_SIZE);
    let mut path: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut reply: msg_read_done = msg_read_done {
        stream: 0,
        error: 0,
    };
    let mut find: client_file = client_file {
        c: ::core::ptr::null_mut::<client>(),
        peer: ::core::ptr::null_mut::<tmuxpeer>(),
        tree: ::core::ptr::null_mut::<client_files>(),
        references: 0,
        stream: 0,
        path: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        buffer: ::core::ptr::null_mut::<evbuffer>(),
        event: ::core::ptr::null_mut::<bufferevent>(),
        fd: 0,
        error: 0,
        closed: 0,
        cb: None,
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        entry: client_file_entry {
            rbe_left: ::core::ptr::null_mut::<client_file>(),
            rbe_right: ::core::ptr::null_mut::<client_file>(),
            rbe_parent: ::core::ptr::null_mut::<client_file>(),
            rbe_color: 0,
        },
    };
    let mut cf: *mut client_file = ::core::ptr::null_mut::<client_file>();
    let flags: ::core::ffi::c_int = O_NONBLOCK | O_RDONLY;
    let mut error: ::core::ffi::c_int = 0;
    if msglen < ::core::mem::size_of::<msg_read_open>() as usize {
        fatalx(b"bad MSG_READ_OPEN size\0" as *const u8 as *const ::core::ffi::c_char);
    }
    if msglen == ::core::mem::size_of::<msg_read_open>() as usize {
        path = b"-\0" as *const u8 as *const ::core::ffi::c_char;
    } else {
        path = msg.offset(1 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_char;
    }
    log_debug(
        b"open read file %d %s\0" as *const u8 as *const ::core::ffi::c_char,
        (*msg).stream,
        path,
    );
    find.stream = (*msg).stream;
    if !client_files_RB_FIND(files, &raw mut find).is_null() {
        error = EBADF;
    } else {
        cf = file_create_with_peer(peer, files, (*msg).stream, cb, cbdata);
        if (*cf).closed != 0 {
            error = EBADF;
        } else {
            (*cf).fd = -(1 as ::core::ffi::c_int);
            if (*msg).fd == -(1 as ::core::ffi::c_int) {
                (*cf).fd = open(path, flags);
            } else if allow_streams != 0 {
                if (*msg).fd != STDIN_FILENO {
                    *__errno_location() = EBADF;
                } else {
                    (*cf).fd = dup((*msg).fd);
                    if close_received != 0 {
                        close((*msg).fd);
                    }
                }
            } else {
                *__errno_location() = EBADF;
            }
            if (*cf).fd == -(1 as ::core::ffi::c_int) {
                error = *__errno_location();
            } else {
                (*cf).event = bufferevent_new(
                    (*cf).fd,
                    Some(
                        file_read_callback
                            as unsafe extern "C" fn(
                                *mut bufferevent,
                                *mut ::core::ffi::c_void,
                            ) -> (),
                    ),
                    None,
                    Some(
                        file_read_error_callback
                            as unsafe extern "C" fn(
                                *mut bufferevent,
                                ::core::ffi::c_short,
                                *mut ::core::ffi::c_void,
                            ) -> (),
                    ),
                    cf as *mut ::core::ffi::c_void,
                );
                if (*cf).event.is_null() {
                    fatalx(b"out of memory\0" as *const u8 as *const ::core::ffi::c_char);
                }
                bufferevent_enable((*cf).event, EV_READ as ::core::ffi::c_short);
                return;
            }
        }
    }
    reply.stream = (*msg).stream;
    reply.error = error;
    proc_send(
        peer,
        MSG_READ_DONE,
        -(1 as ::core::ffi::c_int),
        &raw mut reply as *const ::core::ffi::c_void,
        ::core::mem::size_of::<msg_read_done>() as size_t,
    );
}
#[no_mangle]
pub unsafe extern "C" fn file_read_cancel(mut files: *mut client_files, mut imsg: *mut imsg) {
    let mut msg: *mut msg_read_cancel = (*imsg).data as *mut msg_read_cancel;
    let mut msglen: size_t = ((*imsg).hdr.len as size_t).wrapping_sub(IMSG_HEADER_SIZE);
    let mut find: client_file = client_file {
        c: ::core::ptr::null_mut::<client>(),
        peer: ::core::ptr::null_mut::<tmuxpeer>(),
        tree: ::core::ptr::null_mut::<client_files>(),
        references: 0,
        stream: 0,
        path: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        buffer: ::core::ptr::null_mut::<evbuffer>(),
        event: ::core::ptr::null_mut::<bufferevent>(),
        fd: 0,
        error: 0,
        closed: 0,
        cb: None,
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        entry: client_file_entry {
            rbe_left: ::core::ptr::null_mut::<client_file>(),
            rbe_right: ::core::ptr::null_mut::<client_file>(),
            rbe_parent: ::core::ptr::null_mut::<client_file>(),
            rbe_color: 0,
        },
    };
    let mut cf: *mut client_file = ::core::ptr::null_mut::<client_file>();
    if msglen != ::core::mem::size_of::<msg_read_cancel>() as usize {
        fatalx(b"bad MSG_READ_CANCEL size\0" as *const u8 as *const ::core::ffi::c_char);
    }
    find.stream = (*msg).stream;
    cf = client_files_RB_FIND(files, &raw mut find);
    if cf.is_null() {
        fatalx(b"unknown stream number\0" as *const u8 as *const ::core::ffi::c_char);
    }
    log_debug(
        b"cancel file %d\0" as *const u8 as *const ::core::ffi::c_char,
        (*cf).stream,
    );
    file_read_error_callback(
        ::core::ptr::null_mut::<bufferevent>(),
        0 as ::core::ffi::c_short,
        cf as *mut ::core::ffi::c_void,
    );
}
#[no_mangle]
pub unsafe extern "C" fn file_write_ready(
    mut files: *mut client_files,
    mut imsg: *mut imsg,
) -> ::core::ffi::c_int {
    let mut msg: *mut msg_write_ready = (*imsg).data as *mut msg_write_ready;
    let mut msglen: size_t = ((*imsg).hdr.len as size_t).wrapping_sub(IMSG_HEADER_SIZE);
    let mut find: client_file = client_file {
        c: ::core::ptr::null_mut::<client>(),
        peer: ::core::ptr::null_mut::<tmuxpeer>(),
        tree: ::core::ptr::null_mut::<client_files>(),
        references: 0,
        stream: 0,
        path: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        buffer: ::core::ptr::null_mut::<evbuffer>(),
        event: ::core::ptr::null_mut::<bufferevent>(),
        fd: 0,
        error: 0,
        closed: 0,
        cb: None,
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        entry: client_file_entry {
            rbe_left: ::core::ptr::null_mut::<client_file>(),
            rbe_right: ::core::ptr::null_mut::<client_file>(),
            rbe_parent: ::core::ptr::null_mut::<client_file>(),
            rbe_color: 0,
        },
    };
    let mut cf: *mut client_file = ::core::ptr::null_mut::<client_file>();
    if msglen != ::core::mem::size_of::<msg_write_ready>() as usize {
        return -(1 as ::core::ffi::c_int);
    }
    find.stream = (*msg).stream;
    cf = client_files_RB_FIND(files, &raw mut find);
    if cf.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    if (*msg).error != 0 as ::core::ffi::c_int {
        (*cf).error = (*msg).error;
        file_fire_done(cf);
    } else {
        file_push(cf);
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn file_write_done(
    mut files: *mut client_files,
    mut imsg: *mut imsg,
) -> ::core::ffi::c_int {
    let mut msg: *mut msg_write_done = (*imsg).data as *mut msg_write_done;
    let mut msglen: size_t = ((*imsg).hdr.len as size_t).wrapping_sub(IMSG_HEADER_SIZE);
    let mut find: client_file = client_file {
        c: ::core::ptr::null_mut::<client>(),
        peer: ::core::ptr::null_mut::<tmuxpeer>(),
        tree: ::core::ptr::null_mut::<client_files>(),
        references: 0,
        stream: 0,
        path: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        buffer: ::core::ptr::null_mut::<evbuffer>(),
        event: ::core::ptr::null_mut::<bufferevent>(),
        fd: 0,
        error: 0,
        closed: 0,
        cb: None,
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        entry: client_file_entry {
            rbe_left: ::core::ptr::null_mut::<client_file>(),
            rbe_right: ::core::ptr::null_mut::<client_file>(),
            rbe_parent: ::core::ptr::null_mut::<client_file>(),
            rbe_color: 0,
        },
    };
    let mut cf: *mut client_file = ::core::ptr::null_mut::<client_file>();
    if msglen != ::core::mem::size_of::<msg_write_done>() as usize {
        return -(1 as ::core::ffi::c_int);
    }
    find.stream = (*msg).stream;
    cf = client_files_RB_FIND(files, &raw mut find);
    if cf.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    if (*cf).c.is_null() || !(*(*cf).c).flags as ::core::ffi::c_ulonglong & CLIENT_WRITE_ACK != 0 {
        return 0 as ::core::ffi::c_int;
    }
    log_debug(
        b"file %d write done\0" as *const u8 as *const ::core::ffi::c_char,
        (*cf).stream,
    );
    (*cf).error = (*msg).error;
    file_fire_done(cf);
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn file_read_data(
    mut files: *mut client_files,
    mut imsg: *mut imsg,
) -> ::core::ffi::c_int {
    let mut msg: *mut msg_read_data = (*imsg).data as *mut msg_read_data;
    let mut msglen: size_t = ((*imsg).hdr.len as size_t).wrapping_sub(IMSG_HEADER_SIZE);
    let mut find: client_file = client_file {
        c: ::core::ptr::null_mut::<client>(),
        peer: ::core::ptr::null_mut::<tmuxpeer>(),
        tree: ::core::ptr::null_mut::<client_files>(),
        references: 0,
        stream: 0,
        path: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        buffer: ::core::ptr::null_mut::<evbuffer>(),
        event: ::core::ptr::null_mut::<bufferevent>(),
        fd: 0,
        error: 0,
        closed: 0,
        cb: None,
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        entry: client_file_entry {
            rbe_left: ::core::ptr::null_mut::<client_file>(),
            rbe_right: ::core::ptr::null_mut::<client_file>(),
            rbe_parent: ::core::ptr::null_mut::<client_file>(),
            rbe_color: 0,
        },
    };
    let mut cf: *mut client_file = ::core::ptr::null_mut::<client_file>();
    let mut bdata: *mut ::core::ffi::c_void =
        msg.offset(1 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void;
    let mut bsize: size_t = msglen.wrapping_sub(::core::mem::size_of::<msg_read_data>() as size_t);
    if msglen < ::core::mem::size_of::<msg_read_data>() as usize {
        return -(1 as ::core::ffi::c_int);
    }
    find.stream = (*msg).stream;
    cf = client_files_RB_FIND(files, &raw mut find);
    if cf.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    log_debug(
        b"file %d read %zu bytes\0" as *const u8 as *const ::core::ffi::c_char,
        (*cf).stream,
        bsize,
    );
    if (*cf).error == 0 as ::core::ffi::c_int && (*cf).closed == 0 {
        if evbuffer_add((*cf).buffer, bdata, bsize) != 0 as ::core::ffi::c_int {
            (*cf).error = ENOMEM;
            file_fire_done(cf);
        } else {
            file_fire_read(cf);
        }
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn file_read_done(
    mut files: *mut client_files,
    mut imsg: *mut imsg,
) -> ::core::ffi::c_int {
    let mut msg: *mut msg_read_done = (*imsg).data as *mut msg_read_done;
    let mut msglen: size_t = ((*imsg).hdr.len as size_t).wrapping_sub(IMSG_HEADER_SIZE);
    let mut find: client_file = client_file {
        c: ::core::ptr::null_mut::<client>(),
        peer: ::core::ptr::null_mut::<tmuxpeer>(),
        tree: ::core::ptr::null_mut::<client_files>(),
        references: 0,
        stream: 0,
        path: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        buffer: ::core::ptr::null_mut::<evbuffer>(),
        event: ::core::ptr::null_mut::<bufferevent>(),
        fd: 0,
        error: 0,
        closed: 0,
        cb: None,
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        entry: client_file_entry {
            rbe_left: ::core::ptr::null_mut::<client_file>(),
            rbe_right: ::core::ptr::null_mut::<client_file>(),
            rbe_parent: ::core::ptr::null_mut::<client_file>(),
            rbe_color: 0,
        },
    };
    let mut cf: *mut client_file = ::core::ptr::null_mut::<client_file>();
    if msglen != ::core::mem::size_of::<msg_read_done>() as usize {
        return -(1 as ::core::ffi::c_int);
    }
    find.stream = (*msg).stream;
    cf = client_files_RB_FIND(files, &raw mut find);
    if cf.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    log_debug(
        b"file %d read done\0" as *const u8 as *const ::core::ffi::c_char,
        (*cf).stream,
    );
    (*cf).error = (*msg).error;
    file_fire_done(cf);
    return 0 as ::core::ffi::c_int;
}
