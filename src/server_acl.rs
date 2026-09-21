use crate::src::cmd_queue::cmdq_print;
use crate::src::ffi::libc::{free, getgrgid, getpwuid, getuid};
use crate::src::proc::{proc_get_peer_gid, proc_get_peer_uid};
pub use crate::src::server::clients;
use crate::src::shared::abi::*;
pub use crate::src::shared::abi::{__gid_t, __id_t, __uid_t, gid_t, id_t, uid_t};
pub use crate::src::shared::account::{group, passwd};
pub use crate::src::shared::arguments::args;
use crate::src::shared::client::*;
pub use crate::src::shared::client::{
    client, client_entry, client_file, client_file_cb, client_file_entry, client_files,
    overlay_check_cb, overlay_draw_cb, overlay_free_cb, overlay_key_cb, overlay_mode_cb,
    overlay_resize_cb,
};
pub use crate::src::shared::client::{CLIENT_EXIT, CLIENT_READONLY};
use crate::src::shared::colour::*;
pub use crate::src::shared::command::{cmd_find_state, cmd_list, cmdq_item, cmdq_list, cmds};
pub use crate::src::shared::control::control_state;
use crate::src::shared::display::*;
pub use crate::src::shared::display::{visible_range, visible_ranges};
pub use crate::src::shared::environment::environ;
use crate::src::shared::event::*;
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
pub use crate::src::shared::server_acl::SERVER_ACL_IS_GROUP;
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
use crate::src::xmalloc::{xcalloc, xstrdup};

pub use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_13;
pub use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_12;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct server_acl_entry {
    pub id: id_t,
    pub flags: ::core::ffi::c_int,
    pub entry: C2RustUnnamed_35,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_35 {
    pub rbe_left: *mut server_acl_entry,
    pub rbe_right: *mut server_acl_entry,
    pub rbe_parent: *mut server_acl_entry,
    pub rbe_color: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct server_acl_entries {
    pub rbh_root: *mut server_acl_entry,
}
pub const SERVER_ACL_READONLY: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;

unsafe extern "C" fn server_acl_cmp(
    mut entry1: *mut server_acl_entry,
    mut entry2: *mut server_acl_entry,
) -> ::core::ffi::c_int {
    if ((*entry1).flags ^ (*entry2).flags) & SERVER_ACL_IS_GROUP != 0 {
        if (*entry1).flags & SERVER_ACL_IS_GROUP != 0 {
            return 1 as ::core::ffi::c_int;
        }
        return -(1 as ::core::ffi::c_int);
    }
    if (*entry1).id < (*entry2).id {
        return -(1 as ::core::ffi::c_int);
    }
    return ((*entry1).id > (*entry2).id) as ::core::ffi::c_int;
}
#[no_mangle]
pub static mut server_acl_entries: server_acl_entries = server_acl_entries {
    rbh_root: ::core::ptr::null::<server_acl_entry>() as *mut server_acl_entry,
};
unsafe extern "C" fn server_acl_entries_RB_FIND(
    mut head: *mut server_acl_entries,
    mut elm: *mut server_acl_entry,
) -> *mut server_acl_entry {
    let mut tmp: *mut server_acl_entry = (*head).rbh_root;
    let mut comp: ::core::ffi::c_int = 0;
    while !tmp.is_null() {
        comp = server_acl_cmp(elm, tmp);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_right;
        } else {
            return tmp;
        }
    }
    return ::core::ptr::null_mut::<server_acl_entry>();
}
unsafe extern "C" fn server_acl_entries_RB_NEXT(
    mut elm: *mut server_acl_entry,
) -> *mut server_acl_entry {
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
unsafe extern "C" fn server_acl_entries_RB_INSERT_COLOR(
    mut head: *mut server_acl_entries,
    mut elm: *mut server_acl_entry,
) {
    let mut parent: *mut server_acl_entry = ::core::ptr::null_mut::<server_acl_entry>();
    let mut gparent: *mut server_acl_entry = ::core::ptr::null_mut::<server_acl_entry>();
    let mut tmp: *mut server_acl_entry = ::core::ptr::null_mut::<server_acl_entry>();
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
unsafe extern "C" fn server_acl_entries_RB_INSERT(
    mut head: *mut server_acl_entries,
    mut elm: *mut server_acl_entry,
) -> *mut server_acl_entry {
    let mut tmp: *mut server_acl_entry = ::core::ptr::null_mut::<server_acl_entry>();
    let mut parent: *mut server_acl_entry = ::core::ptr::null_mut::<server_acl_entry>();
    let mut comp: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    tmp = (*head).rbh_root;
    while !tmp.is_null() {
        parent = tmp;
        comp = server_acl_cmp(elm, parent);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_right;
        } else {
            return tmp;
        }
    }
    (*elm).entry.rbe_parent = parent;
    (*elm).entry.rbe_right = ::core::ptr::null_mut::<server_acl_entry>();
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
    server_acl_entries_RB_INSERT_COLOR(head, elm);
    return ::core::ptr::null_mut::<server_acl_entry>();
}
unsafe extern "C" fn server_acl_entries_RB_MINMAX(
    mut head: *mut server_acl_entries,
    mut val: ::core::ffi::c_int,
) -> *mut server_acl_entry {
    let mut tmp: *mut server_acl_entry = (*head).rbh_root;
    let mut parent: *mut server_acl_entry = ::core::ptr::null_mut::<server_acl_entry>();
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
unsafe extern "C" fn server_acl_entries_RB_REMOVE(
    mut head: *mut server_acl_entries,
    mut elm: *mut server_acl_entry,
) -> *mut server_acl_entry {
    let mut current_block: u64;
    let mut child: *mut server_acl_entry = ::core::ptr::null_mut::<server_acl_entry>();
    let mut parent: *mut server_acl_entry = ::core::ptr::null_mut::<server_acl_entry>();
    let mut old: *mut server_acl_entry = elm;
    let mut color: ::core::ffi::c_int = 0;
    if (*elm).entry.rbe_left.is_null() {
        child = (*elm).entry.rbe_right;
        current_block = 7245201122033322888;
    } else if (*elm).entry.rbe_right.is_null() {
        child = (*elm).entry.rbe_left;
        current_block = 7245201122033322888;
    } else {
        let mut left: *mut server_acl_entry = ::core::ptr::null_mut::<server_acl_entry>();
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
        current_block = 10977582915827884067;
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
        server_acl_entries_RB_REMOVE_COLOR(head, parent, child);
    }
    return old;
}
unsafe extern "C" fn server_acl_entries_RB_REMOVE_COLOR(
    mut head: *mut server_acl_entries,
    mut parent: *mut server_acl_entry,
    mut elm: *mut server_acl_entry,
) {
    let mut tmp: *mut server_acl_entry = ::core::ptr::null_mut::<server_acl_entry>();
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
                    let mut oleft: *mut server_acl_entry =
                        ::core::ptr::null_mut::<server_acl_entry>();
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
                    let mut oright: *mut server_acl_entry =
                        ::core::ptr::null_mut::<server_acl_entry>();
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
unsafe extern "C" fn server_acl_entry_find(
    mut id: id_t,
    mut flags: ::core::ffi::c_int,
) -> *mut server_acl_entry {
    let mut find: server_acl_entry = server_acl_entry {
        id: id,
        flags: flags & SERVER_ACL_IS_GROUP,
        entry: C2RustUnnamed_35 {
            rbe_left: ::core::ptr::null_mut::<server_acl_entry>(),
            rbe_right: ::core::ptr::null_mut::<server_acl_entry>(),
            rbe_parent: ::core::ptr::null_mut::<server_acl_entry>(),
            rbe_color: 0,
        },
    };
    return server_acl_entries_RB_FIND(&raw mut server_acl_entries, &raw mut find);
}
unsafe extern "C" fn server_acl_check(mut c: *mut client) -> *mut server_acl_entry {
    let mut entry: *mut server_acl_entry = ::core::ptr::null_mut::<server_acl_entry>();
    let mut uid: uid_t = 0;
    let mut gid: gid_t = 0;
    uid = proc_get_peer_uid((*c).peer);
    if uid == -(1 as ::core::ffi::c_int) as uid_t {
        return ::core::ptr::null_mut::<server_acl_entry>();
    }
    entry = server_acl_entry_find(uid as id_t, 0 as ::core::ffi::c_int);
    if !entry.is_null() {
        return entry;
    }
    gid = proc_get_peer_gid((*c).peer);
    if gid == -(1 as ::core::ffi::c_int) as gid_t {
        return ::core::ptr::null_mut::<server_acl_entry>();
    }
    return server_acl_entry_find(gid as id_t, SERVER_ACL_IS_GROUP);
}
unsafe extern "C" fn server_acl_update() {
    let mut entry: *mut server_acl_entry = ::core::ptr::null_mut::<server_acl_entry>();
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    c = clients.tqh_first;
    while !c.is_null() {
        entry = server_acl_check(c);
        if entry.is_null() {
            (*c).exit_message =
                xstrdup(b"access not allowed\0" as *const u8 as *const ::core::ffi::c_char);
            (*c).flags |= CLIENT_EXIT as uint64_t;
        } else if (*entry).flags & SERVER_ACL_READONLY != 0 {
            (*c).flags |= CLIENT_READONLY as uint64_t;
        } else {
            (*c).flags &= !CLIENT_READONLY as uint64_t;
        }
        c = (*c).entry.tqe_next;
    }
}
#[no_mangle]
pub unsafe extern "C" fn server_acl_init() {
    server_acl_entries.rbh_root = ::core::ptr::null_mut::<server_acl_entry>();
    if getuid() != 0 as __uid_t {
        server_acl_allow(0 as id_t, 0 as ::core::ffi::c_int);
    }
    server_acl_allow(getuid() as id_t, 0 as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn server_acl_find(
    mut id: id_t,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    return (server_acl_entry_find(id, flags) != NULL as *mut server_acl_entry)
        as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn server_acl_display(mut item: *mut cmdq_item) {
    let mut loop_0: *mut server_acl_entry = ::core::ptr::null_mut::<server_acl_entry>();
    let mut pw: *mut passwd = ::core::ptr::null_mut::<passwd>();
    let mut gr: *mut group = ::core::ptr::null_mut::<group>();
    let mut name: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut type_0: ::core::ffi::c_char = 0;
    let mut current_block_12: u64;
    loop_0 = server_acl_entries_RB_MINMAX(&raw mut server_acl_entries, RB_NEGINF);
    while !loop_0.is_null() {
        if !(*loop_0).flags & SERVER_ACL_IS_GROUP != 0 {
            if (*loop_0).id == 0 as id_t {
                current_block_12 = 14916268686031723178;
            } else {
                pw = getpwuid((*loop_0).id as __uid_t);
                if !pw.is_null() {
                    name = (*pw).pw_name;
                } else {
                    name = b"unknown\0" as *const u8 as *const ::core::ffi::c_char;
                }
                type_0 = 'U' as i32 as ::core::ffi::c_char;
                current_block_12 = 11050875288958768710;
            }
        } else {
            gr = getgrgid((*loop_0).id as __gid_t);
            if !gr.is_null() {
                name = (*gr).gr_name;
            } else {
                name = b"unknown\0" as *const u8 as *const ::core::ffi::c_char;
            }
            type_0 = 'G' as i32 as ::core::ffi::c_char;
            current_block_12 = 11050875288958768710;
        }
        match current_block_12 {
            11050875288958768710 => {
                if (*loop_0).flags & SERVER_ACL_READONLY != 0 {
                    cmdq_print(
                        item,
                        b"%s (%c,R)\0" as *const u8 as *const ::core::ffi::c_char,
                        name,
                        type_0 as ::core::ffi::c_int,
                    );
                } else {
                    cmdq_print(
                        item,
                        b"%s (%c,W)\0" as *const u8 as *const ::core::ffi::c_char,
                        name,
                        type_0 as ::core::ffi::c_int,
                    );
                }
            }
            _ => {}
        }
        loop_0 = server_acl_entries_RB_NEXT(loop_0);
    }
}
#[no_mangle]
pub unsafe extern "C" fn server_acl_allow(mut id: id_t, mut flags: ::core::ffi::c_int) {
    let mut entry: *mut server_acl_entry = ::core::ptr::null_mut::<server_acl_entry>();
    entry = server_acl_entry_find(id, flags);
    if entry.is_null() {
        entry = xcalloc(
            1 as size_t,
            ::core::mem::size_of::<server_acl_entry>() as size_t,
        ) as *mut server_acl_entry;
        (*entry).id = id;
        (*entry).flags = flags & SERVER_ACL_IS_GROUP;
        server_acl_entries_RB_INSERT(&raw mut server_acl_entries, entry);
    }
}
#[no_mangle]
pub unsafe extern "C" fn server_acl_deny(mut id: id_t, mut flags: ::core::ffi::c_int) {
    let mut entry: *mut server_acl_entry = ::core::ptr::null_mut::<server_acl_entry>();
    entry = server_acl_entry_find(id, flags);
    if !entry.is_null() {
        server_acl_entries_RB_REMOVE(&raw mut server_acl_entries, entry);
        free(entry as *mut ::core::ffi::c_void);
        server_acl_update();
    }
}
#[no_mangle]
pub unsafe extern "C" fn server_acl_allow_write(mut id: id_t, mut flags: ::core::ffi::c_int) {
    let mut entry: *mut server_acl_entry = ::core::ptr::null_mut::<server_acl_entry>();
    entry = server_acl_entry_find(id, flags);
    if entry.is_null() {
        return;
    }
    (*entry).flags &= !SERVER_ACL_READONLY;
    server_acl_update();
}
#[no_mangle]
pub unsafe extern "C" fn server_acl_deny_write(mut id: id_t, mut flags: ::core::ffi::c_int) {
    let mut entry: *mut server_acl_entry = ::core::ptr::null_mut::<server_acl_entry>();
    entry = server_acl_entry_find(id, flags);
    if entry.is_null() {
        return;
    }
    (*entry).flags |= SERVER_ACL_READONLY;
    server_acl_update();
}
#[no_mangle]
pub unsafe extern "C" fn server_acl_join(mut c: *mut client) -> ::core::ffi::c_int {
    let mut entry: *mut server_acl_entry = ::core::ptr::null_mut::<server_acl_entry>();
    entry = server_acl_check(c);
    if entry.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    if (*entry).flags & SERVER_ACL_READONLY != 0 {
        (*c).flags |= CLIENT_READONLY as uint64_t;
    }
    return 1 as ::core::ffi::c_int;
}
