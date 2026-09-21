use crate::src::cmd_find::{
    cmd_find_clear_state, cmd_find_from_nothing, cmd_find_from_pane, cmd_find_from_session,
    cmd_find_from_session_window, cmd_find_from_winlink, cmd_find_from_winlink_pane,
    cmd_find_valid_state,
};
use crate::src::ffi::libc::{free, strcmp};
use crate::src::format::format_add;
use crate::src::log::{fatalx, log_debug};
use crate::src::reactor::{
    evbuffer_add_printf, evbuffer_free, evbuffer_get_length, evbuffer_new, evbuffer_pullup,
};
use crate::src::server_client::server_client_unref;
use crate::src::session::{session_add_ref, session_alive, session_remove_ref};
pub use crate::src::shared::abi::ssize_t;
use crate::src::shared::abi::*;
pub use crate::src::shared::arguments::args;
use crate::src::shared::client::*;
pub use crate::src::shared::client::{
    client, client_entry, client_file, client_file_cb, client_file_entry, client_files,
    overlay_check_cb, overlay_draw_cb, overlay_free_cb, overlay_key_cb, overlay_mode_cb,
    overlay_resize_cb,
};
use crate::src::shared::colour::*;
pub use crate::src::shared::command::{cmd_find_state, cmd_list, cmdq_item, cmdq_list, cmds};
pub use crate::src::shared::control::control_state;
use crate::src::shared::display::*;
pub use crate::src::shared::display::{visible_range, visible_ranges};
pub use crate::src::shared::environment::environ;
use crate::src::shared::event::*;
pub use crate::src::shared::events::{
    event_payload, event_payload_free_cb, event_payload_item, event_payload_item_c2rust_unnamed,
    event_payload_item_c2rust_unnamed_pointer, event_payload_item_entry, event_payload_print_cb,
    event_payload_tree, event_payload_type,
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
pub use crate::src::shared::variadic::{__builtin_va_list, __va_list_tag, va_list};
pub use crate::src::shared::window::{
    window, window_alerts_entry, window_entry, window_mode, window_mode_entry,
    window_mode_entry_entry, window_winlinks, winlink, winlink_entry, winlink_sentry,
    winlink_stack, winlink_wentry, winlinks,
};
use crate::src::window::{
    window_add_ref, window_has_pane, window_pane_add_ref, window_pane_remove_ref,
    window_remove_ref, winlink_find_by_index,
};
use crate::src::xmalloc::{xasprintf, xcalloc, xmemdup, xstrdup, xvasprintf};

pub use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_13;
pub use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_12;

pub const EVENT_PAYLOAD_POINTER: event_payload_type = 8;
pub const EVENT_PAYLOAD_PANE: event_payload_type = 7;
pub const EVENT_PAYLOAD_WINDOW: event_payload_type = 6;
pub const EVENT_PAYLOAD_SESSION: event_payload_type = 5;
pub const EVENT_PAYLOAD_CLIENT: event_payload_type = 4;
pub const EVENT_PAYLOAD_UINT: event_payload_type = 3;
pub const EVENT_PAYLOAD_INT: event_payload_type = 2;
pub const EVENT_PAYLOAD_TIME: event_payload_type = 1;
pub const EVENT_PAYLOAD_STRING: event_payload_type = 0;
unsafe extern "C" fn event_payload_cmp(
    mut epi1: *mut event_payload_item,
    mut epi2: *mut event_payload_item,
) -> ::core::ffi::c_int {
    return strcmp((*epi1).name, (*epi2).name);
}
unsafe extern "C" fn event_payload_tree_RB_NEXT(
    mut elm: *mut event_payload_item,
) -> *mut event_payload_item {
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
unsafe extern "C" fn event_payload_tree_RB_REMOVE(
    mut head: *mut event_payload_tree,
    mut elm: *mut event_payload_item,
) -> *mut event_payload_item {
    let mut current_block: u64;
    let mut child: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    let mut parent: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    let mut old: *mut event_payload_item = elm;
    let mut color: ::core::ffi::c_int = 0;
    if (*elm).entry.rbe_left.is_null() {
        child = (*elm).entry.rbe_right;
        current_block = 7245201122033322888;
    } else if (*elm).entry.rbe_right.is_null() {
        child = (*elm).entry.rbe_left;
        current_block = 7245201122033322888;
    } else {
        let mut left: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
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
        current_block = 4623090504028811011;
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
        event_payload_tree_RB_REMOVE_COLOR(head, parent, child);
    }
    return old;
}
unsafe extern "C" fn event_payload_tree_RB_REMOVE_COLOR(
    mut head: *mut event_payload_tree,
    mut parent: *mut event_payload_item,
    mut elm: *mut event_payload_item,
) {
    let mut tmp: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
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
                    let mut oleft: *mut event_payload_item =
                        ::core::ptr::null_mut::<event_payload_item>();
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
                    let mut oright: *mut event_payload_item =
                        ::core::ptr::null_mut::<event_payload_item>();
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
unsafe extern "C" fn event_payload_tree_RB_MINMAX(
    mut head: *mut event_payload_tree,
    mut val: ::core::ffi::c_int,
) -> *mut event_payload_item {
    let mut tmp: *mut event_payload_item = (*head).rbh_root;
    let mut parent: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
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
unsafe extern "C" fn event_payload_tree_RB_INSERT_COLOR(
    mut head: *mut event_payload_tree,
    mut elm: *mut event_payload_item,
) {
    let mut parent: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    let mut gparent: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    let mut tmp: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
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
unsafe extern "C" fn event_payload_tree_RB_INSERT(
    mut head: *mut event_payload_tree,
    mut elm: *mut event_payload_item,
) -> *mut event_payload_item {
    let mut tmp: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    let mut parent: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    let mut comp: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    tmp = (*head).rbh_root;
    while !tmp.is_null() {
        parent = tmp;
        comp = event_payload_cmp(elm, parent);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_right;
        } else {
            return tmp;
        }
    }
    (*elm).entry.rbe_parent = parent;
    (*elm).entry.rbe_right = ::core::ptr::null_mut::<event_payload_item>();
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
    event_payload_tree_RB_INSERT_COLOR(head, elm);
    return ::core::ptr::null_mut::<event_payload_item>();
}
unsafe extern "C" fn event_payload_tree_RB_FIND(
    mut head: *mut event_payload_tree,
    mut elm: *mut event_payload_item,
) -> *mut event_payload_item {
    let mut tmp: *mut event_payload_item = (*head).rbh_root;
    let mut comp: ::core::ffi::c_int = 0;
    while !tmp.is_null() {
        comp = event_payload_cmp(elm, tmp);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_right;
        } else {
            return tmp;
        }
    }
    return ::core::ptr::null_mut::<event_payload_item>();
}
unsafe extern "C" fn event_payload_find(
    mut ep: *mut event_payload,
    mut name: *const ::core::ffi::c_char,
) -> *mut event_payload_item {
    let mut find: event_payload_item = event_payload_item {
        name: name as *mut ::core::ffi::c_char,
        type_0: EVENT_PAYLOAD_STRING,
        c2rust_unnamed: event_payload_item_c2rust_unnamed {
            string: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        },
        entry: event_payload_item_entry {
            rbe_left: ::core::ptr::null_mut::<event_payload_item>(),
            rbe_right: ::core::ptr::null_mut::<event_payload_item>(),
            rbe_parent: ::core::ptr::null_mut::<event_payload_item>(),
            rbe_color: 0,
        },
    };
    return event_payload_tree_RB_FIND(&raw mut (*ep).items, &raw mut find);
}
unsafe extern "C" fn event_payload_free_target(mut ep: *mut event_payload) {
    let mut target: *mut cmd_find_state = &raw mut (*ep).target;
    if !(*target).s.is_null() {
        session_remove_ref(
            (*target).s,
            b"event_payload_free_target\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    if !(*target).w.is_null() {
        window_remove_ref(
            (*target).w,
            b"event_payload_free_target\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    if !(*target).wp.is_null() {
        window_pane_remove_ref(
            (*target).wp,
            b"event_payload_free_target\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    cmd_find_clear_state(target, 0 as ::core::ffi::c_int);
}
unsafe extern "C" fn event_payload_free_value(mut epi: *mut event_payload_item) {
    match (*epi).type_0 as ::core::ffi::c_uint {
        0 => {
            free((*epi).c2rust_unnamed.string as *mut ::core::ffi::c_void);
        }
        4 => {
            server_client_unref((*epi).c2rust_unnamed.client);
        }
        5 => {
            session_remove_ref(
                (*epi).c2rust_unnamed.session,
                b"event_payload_free_value\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        6 => {
            window_remove_ref(
                (*epi).c2rust_unnamed.window,
                b"event_payload_free_value\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        7 => {
            window_pane_remove_ref(
                (*epi).c2rust_unnamed.pane,
                b"event_payload_free_value\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        8 => {
            if (*epi).c2rust_unnamed.pointer.free_cb.is_some() {
                (*epi)
                    .c2rust_unnamed
                    .pointer
                    .free_cb
                    .expect("non-null function pointer")(
                    (*epi).c2rust_unnamed.pointer.ptr
                );
            }
        }
        2 | 3 | 1 | _ => {}
    };
}
unsafe extern "C" fn event_payload_set_item(
    mut ep: *mut event_payload,
    mut name: *const ::core::ffi::c_char,
    mut new: *mut event_payload_item,
) {
    let mut old: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    (*new).name = xstrdup(name);
    old = event_payload_tree_RB_INSERT(&raw mut (*ep).items, new);
    if !old.is_null() {
        event_payload_tree_RB_REMOVE(&raw mut (*ep).items, old);
        event_payload_free_value(old);
        free((*old).name as *mut ::core::ffi::c_void);
        free(old as *mut ::core::ffi::c_void);
        event_payload_tree_RB_INSERT(&raw mut (*ep).items, new);
    }
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_create() -> *mut event_payload {
    let mut ep: *mut event_payload = ::core::ptr::null_mut::<event_payload>();
    ep = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<event_payload>() as size_t,
    ) as *mut event_payload;
    (*ep).items.rbh_root = ::core::ptr::null_mut::<event_payload_item>();
    cmd_find_clear_state(&raw mut (*ep).target, 0 as ::core::ffi::c_int);
    return ep;
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_free(mut ep: *mut event_payload) {
    let mut epi: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    let mut epi1: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    if !ep.is_null() {
        epi = event_payload_tree_RB_MINMAX(&raw mut (*ep).items, RB_NEGINF);
        while !epi.is_null() && {
            epi1 = event_payload_tree_RB_NEXT(epi);
            1 as ::core::ffi::c_int != 0
        } {
            event_payload_tree_RB_REMOVE(&raw mut (*ep).items, epi);
            event_payload_free_value(epi);
            free((*epi).name as *mut ::core::ffi::c_void);
            free(epi as *mut ::core::ffi::c_void);
            epi = epi1;
        }
        event_payload_free_target(ep);
        free(ep as *mut ::core::ffi::c_void);
    }
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_set_target(
    mut ep: *mut event_payload,
    mut fs: *mut cmd_find_state,
) {
    let mut target: *mut cmd_find_state = &raw mut (*ep).target;
    event_payload_free_target(ep);
    if !(*fs).s.is_null() {
        session_add_ref(
            (*fs).s,
            b"event_payload_set_target\0" as *const u8 as *const ::core::ffi::c_char,
        );
        (*target).s = (*fs).s;
    }
    if !(*fs).wl.is_null() {
        (*target).idx = (*(*fs).wl).idx;
        if (*target).s.is_null() {
            session_add_ref(
                (*(*fs).wl).session,
                b"event_payload_set_target\0" as *const u8 as *const ::core::ffi::c_char,
            );
            (*target).s = (*(*fs).wl).session;
        }
    } else {
        (*target).idx = -(1 as ::core::ffi::c_int);
    }
    if !(*fs).w.is_null() {
        window_add_ref(
            (*fs).w,
            b"event_payload_set_target\0" as *const u8 as *const ::core::ffi::c_char,
        );
        (*target).w = (*fs).w;
    } else if !(*fs).wl.is_null() {
        window_add_ref(
            (*(*fs).wl).window,
            b"event_payload_set_target\0" as *const u8 as *const ::core::ffi::c_char,
        );
        (*target).w = (*(*fs).wl).window;
    }
    if !(*fs).wp.is_null() {
        window_pane_add_ref(
            (*fs).wp,
            b"event_payload_set_target\0" as *const u8 as *const ::core::ffi::c_char,
        );
        (*target).wp = (*fs).wp;
    }
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_get_target(
    mut ep: *mut event_payload,
    mut fs: *mut cmd_find_state,
) -> ::core::ffi::c_int {
    let mut t: *mut cmd_find_state = &raw mut (*ep).target;
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut flags: ::core::ffi::c_int = (*fs).flags;
    if (*t).idx != -(1 as ::core::ffi::c_int)
        && !(*t).s.is_null()
        && !(*t).w.is_null()
        && session_alive((*t).s) != 0
    {
        wl = winlink_find_by_index(&raw mut (*(*t).s).windows, (*t).idx);
        if !wl.is_null() && (*wl).window != (*t).w {
            wl = ::core::ptr::null_mut::<winlink>();
        }
    }
    cmd_find_clear_state(fs, flags);
    (*fs).s = (*t).s;
    (*fs).w = (*t).w;
    (*fs).wp = (*t).wp;
    (*fs).wl = wl;
    (*fs).idx = if !wl.is_null() {
        (*wl).idx
    } else {
        -(1 as ::core::ffi::c_int)
    };
    if cmd_find_valid_state(fs) != 0 {
        return 1 as ::core::ffi::c_int;
    }
    if !wl.is_null() && !(*t).wp.is_null() && window_has_pane((*wl).window, (*t).wp) != 0 {
        cmd_find_from_winlink_pane(fs, wl, (*t).wp, flags);
        if cmd_find_valid_state(fs) != 0 {
            return 1 as ::core::ffi::c_int;
        }
    }
    if !(*t).wp.is_null()
        && cmd_find_from_pane(fs, (*t).wp, flags) == 0 as ::core::ffi::c_int
        && cmd_find_valid_state(fs) != 0
    {
        return 1 as ::core::ffi::c_int;
    }
    if !wl.is_null() {
        cmd_find_from_winlink(fs, wl, flags);
        if cmd_find_valid_state(fs) != 0 {
            return 1 as ::core::ffi::c_int;
        }
    }
    if !(*t).s.is_null()
        && !(*t).w.is_null()
        && session_alive((*t).s) != 0
        && cmd_find_from_session_window(fs, (*t).s, (*t).w, flags) == 0 as ::core::ffi::c_int
        && cmd_find_valid_state(fs) != 0
    {
        return 1 as ::core::ffi::c_int;
    }
    if !(*t).s.is_null() && session_alive((*t).s) != 0 {
        cmd_find_from_session(fs, (*t).s, flags);
        if cmd_find_valid_state(fs) != 0 {
            return 1 as ::core::ffi::c_int;
        }
    }
    if cmd_find_from_nothing(fs, flags) == 0 as ::core::ffi::c_int {
        return 1 as ::core::ffi::c_int;
    }
    cmd_find_clear_state(fs, flags);
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_set_string(
    mut ep: *mut event_payload,
    mut name: *const ::core::ffi::c_char,
    mut fmt: *const ::core::ffi::c_char,
    mut args: ...
) {
    let mut epi: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    let mut ap: ::core::ffi::VaList;
    ap = args.clone();
    epi = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<event_payload_item>() as size_t,
    ) as *mut event_payload_item;
    (*epi).type_0 = EVENT_PAYLOAD_STRING;
    xvasprintf(&raw mut (*epi).c2rust_unnamed.string, fmt, ap);
    event_payload_set_item(ep, name, epi);
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_set_time(
    mut ep: *mut event_payload,
    mut name: *const ::core::ffi::c_char,
    mut value: time_t,
) {
    let mut epi: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    epi = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<event_payload_item>() as size_t,
    ) as *mut event_payload_item;
    (*epi).type_0 = EVENT_PAYLOAD_TIME;
    (*epi).c2rust_unnamed.time = value;
    event_payload_set_item(ep, name, epi);
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_set_int(
    mut ep: *mut event_payload,
    mut name: *const ::core::ffi::c_char,
    mut value: ::core::ffi::c_int,
) {
    let mut epi: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    epi = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<event_payload_item>() as size_t,
    ) as *mut event_payload_item;
    (*epi).type_0 = EVENT_PAYLOAD_INT;
    (*epi).c2rust_unnamed.number = value;
    event_payload_set_item(ep, name, epi);
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_set_uint(
    mut ep: *mut event_payload,
    mut name: *const ::core::ffi::c_char,
    mut value: u_int,
) {
    let mut epi: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    epi = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<event_payload_item>() as size_t,
    ) as *mut event_payload_item;
    (*epi).type_0 = EVENT_PAYLOAD_UINT;
    (*epi).c2rust_unnamed.unsigned_number = value;
    event_payload_set_item(ep, name, epi);
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_set_client(
    mut ep: *mut event_payload,
    mut name: *const ::core::ffi::c_char,
    mut c: *mut client,
) {
    let mut epi: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    (*c).references += 1;
    epi = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<event_payload_item>() as size_t,
    ) as *mut event_payload_item;
    (*epi).type_0 = EVENT_PAYLOAD_CLIENT;
    (*epi).c2rust_unnamed.client = c;
    event_payload_set_item(ep, name, epi);
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_set_session(
    mut ep: *mut event_payload,
    mut name: *const ::core::ffi::c_char,
    mut s: *mut session,
) {
    let mut epi: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    session_add_ref(
        s,
        b"event_payload_set_session\0" as *const u8 as *const ::core::ffi::c_char,
    );
    epi = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<event_payload_item>() as size_t,
    ) as *mut event_payload_item;
    (*epi).type_0 = EVENT_PAYLOAD_SESSION;
    (*epi).c2rust_unnamed.session = s;
    event_payload_set_item(ep, name, epi);
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_set_window(
    mut ep: *mut event_payload,
    mut name: *const ::core::ffi::c_char,
    mut w: *mut window,
) {
    let mut epi: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    window_add_ref(
        w,
        b"event_payload_set_window\0" as *const u8 as *const ::core::ffi::c_char,
    );
    epi = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<event_payload_item>() as size_t,
    ) as *mut event_payload_item;
    (*epi).type_0 = EVENT_PAYLOAD_WINDOW;
    (*epi).c2rust_unnamed.window = w;
    event_payload_set_item(ep, name, epi);
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_set_pane(
    mut ep: *mut event_payload,
    mut name: *const ::core::ffi::c_char,
    mut wp: *mut window_pane,
) {
    let mut epi: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    window_pane_add_ref(
        wp,
        b"event_payload_set_pane\0" as *const u8 as *const ::core::ffi::c_char,
    );
    epi = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<event_payload_item>() as size_t,
    ) as *mut event_payload_item;
    (*epi).type_0 = EVENT_PAYLOAD_PANE;
    (*epi).c2rust_unnamed.pane = wp;
    event_payload_set_item(ep, name, epi);
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_set_pointer(
    mut ep: *mut event_payload,
    mut name: *const ::core::ffi::c_char,
    mut ptr: *mut ::core::ffi::c_void,
    mut free_cb: event_payload_free_cb,
    mut print_cb: event_payload_print_cb,
) {
    let mut epi: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    epi = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<event_payload_item>() as size_t,
    ) as *mut event_payload_item;
    (*epi).type_0 = EVENT_PAYLOAD_POINTER;
    (*epi).c2rust_unnamed.pointer.ptr = ptr;
    (*epi).c2rust_unnamed.pointer.free_cb = free_cb;
    (*epi).c2rust_unnamed.pointer.print_cb = print_cb;
    event_payload_set_item(ep, name, epi);
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_get_string(
    mut ep: *mut event_payload,
    mut name: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    let mut epi: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    epi = event_payload_find(ep, name);
    if epi.is_null()
        || (*epi).type_0 as ::core::ffi::c_uint
            != EVENT_PAYLOAD_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    return (*epi).c2rust_unnamed.string;
}
unsafe extern "C" fn event_payload_add_item(
    mut epi: *mut event_payload_item,
    mut evb: *mut evbuffer,
) {
    match (*epi).type_0 as ::core::ffi::c_uint {
        0 => {
            evbuffer_add_printf(
                evb,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                (*epi).c2rust_unnamed.string,
            );
        }
        1 => {
            evbuffer_add_printf(
                evb,
                b"%lld\0" as *const u8 as *const ::core::ffi::c_char,
                (*epi).c2rust_unnamed.time as ::core::ffi::c_longlong,
            );
        }
        2 => {
            evbuffer_add_printf(
                evb,
                b"%d\0" as *const u8 as *const ::core::ffi::c_char,
                (*epi).c2rust_unnamed.number,
            );
        }
        3 => {
            evbuffer_add_printf(
                evb,
                b"%u\0" as *const u8 as *const ::core::ffi::c_char,
                (*epi).c2rust_unnamed.unsigned_number,
            );
        }
        4 => {
            evbuffer_add_printf(
                evb,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                (*(*epi).c2rust_unnamed.client).name,
            );
        }
        5 => {
            evbuffer_add_printf(
                evb,
                b"$%u\0" as *const u8 as *const ::core::ffi::c_char,
                (*(*epi).c2rust_unnamed.session).id,
            );
        }
        6 => {
            evbuffer_add_printf(
                evb,
                b"@%u\0" as *const u8 as *const ::core::ffi::c_char,
                (*(*epi).c2rust_unnamed.window).id,
            );
        }
        7 => {
            evbuffer_add_printf(
                evb,
                b"%%%u\0" as *const u8 as *const ::core::ffi::c_char,
                (*(*epi).c2rust_unnamed.pane).id,
            );
        }
        8 => {
            if (*epi).c2rust_unnamed.pointer.print_cb.is_some() {
                (*epi)
                    .c2rust_unnamed
                    .pointer
                    .print_cb
                    .expect("non-null function pointer")(
                    (*epi).c2rust_unnamed.pointer.ptr, evb
                );
            } else {
                evbuffer_add_printf(
                    evb,
                    b"%p\0" as *const u8 as *const ::core::ffi::c_char,
                    (*epi).c2rust_unnamed.pointer.ptr,
                );
            }
        }
        _ => {}
    };
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_item_print(
    mut epi: *mut event_payload_item,
) -> *mut ::core::ffi::c_char {
    let mut evb: *mut evbuffer = ::core::ptr::null_mut::<evbuffer>();
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut size: size_t = 0;
    evb = evbuffer_new();
    if evb.is_null() {
        fatalx(b"out of memory\0" as *const u8 as *const ::core::ffi::c_char);
    }
    event_payload_add_item(epi, evb);
    size = evbuffer_get_length(evb);
    if size != 0 as size_t {
        value = xmemdup(
            evbuffer_pullup(evb, -(1 as ::core::ffi::c_int) as ssize_t)
                as *const ::core::ffi::c_void,
            size,
        );
    } else {
        value = xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
    }
    evbuffer_free(evb);
    return value;
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_print(
    mut ep: *mut event_payload,
    mut name: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut epi: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    epi = event_payload_find(ep, name);
    if epi.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    return event_payload_item_print(epi);
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_add_formats(
    mut ep: *mut event_payload,
    mut ft: *mut format_tree,
    mut prefix: *const ::core::ffi::c_char,
) {
    let mut epi: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut key: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if prefix.is_null() {
        prefix = b"\0" as *const u8 as *const ::core::ffi::c_char;
    }
    epi = event_payload_tree_RB_MINMAX(&raw mut (*ep).items, RB_NEGINF);
    while !epi.is_null() {
        key = (*epi).name;
        if !(*key as ::core::ffi::c_int == '_' as i32) {
            value = event_payload_item_print(epi);
            xasprintf(
                &raw mut name,
                b"%s%s\0" as *const u8 as *const ::core::ffi::c_char,
                prefix,
                key,
            );
            format_add(
                ft,
                name,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                value,
            );
            free(name as *mut ::core::ffi::c_void);
            free(value as *mut ::core::ffi::c_void);
            if (*epi).type_0 as ::core::ffi::c_uint
                == EVENT_PAYLOAD_SESSION as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                xasprintf(
                    &raw mut name,
                    b"%s%s_name\0" as *const u8 as *const ::core::ffi::c_char,
                    prefix,
                    key,
                );
                format_add(
                    ft,
                    name,
                    b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                    (*(*epi).c2rust_unnamed.session).name,
                );
                free(name as *mut ::core::ffi::c_void);
            } else if (*epi).type_0 as ::core::ffi::c_uint
                == EVENT_PAYLOAD_WINDOW as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                xasprintf(
                    &raw mut name,
                    b"%s%s_name\0" as *const u8 as *const ::core::ffi::c_char,
                    prefix,
                    key,
                );
                format_add(
                    ft,
                    name,
                    b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                    (*(*epi).c2rust_unnamed.window).name,
                );
                free(name as *mut ::core::ffi::c_void);
            }
        }
        epi = event_payload_tree_RB_NEXT(epi);
    }
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_first(
    mut ep: *mut event_payload,
) -> *mut event_payload_item {
    return event_payload_tree_RB_MINMAX(&raw mut (*ep).items, RB_NEGINF);
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_next(
    mut epi: *mut event_payload_item,
) -> *mut event_payload_item {
    return event_payload_tree_RB_NEXT(epi);
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_item_name(
    mut epi: *mut event_payload_item,
) -> *const ::core::ffi::c_char {
    return (*epi).name;
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_item_type(
    mut epi: *mut event_payload_item,
) -> event_payload_type {
    return (*epi).type_0;
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_log(
    mut ep: *mut event_payload,
    mut fmt: *const ::core::ffi::c_char,
    mut args: ...
) {
    let mut epi: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    let mut evb: *mut evbuffer = ::core::ptr::null_mut::<evbuffer>();
    let mut ap: ::core::ffi::VaList;
    let mut prefix: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    ap = args.clone();
    xvasprintf(&raw mut prefix, fmt, ap);
    evb = evbuffer_new();
    if evb.is_null() {
        fatalx(b"out of memory\0" as *const u8 as *const ::core::ffi::c_char);
    }
    if !ep.is_null() {
        epi = event_payload_tree_RB_MINMAX(&raw mut (*ep).items, RB_NEGINF);
        while !epi.is_null() {
            if evbuffer_get_length(evb) != 0 as size_t {
                evbuffer_add_printf(evb, b", \0" as *const u8 as *const ::core::ffi::c_char);
            }
            evbuffer_add_printf(
                evb,
                b"%s=\0" as *const u8 as *const ::core::ffi::c_char,
                (*epi).name,
            );
            event_payload_add_item(epi, evb);
            epi = event_payload_tree_RB_NEXT(epi);
        }
    }
    log_debug(
        b"%s%.*s\0" as *const u8 as *const ::core::ffi::c_char,
        prefix,
        evbuffer_get_length(evb) as ::core::ffi::c_int,
        evbuffer_pullup(evb, -(1 as ::core::ffi::c_int) as ssize_t) as *mut ::core::ffi::c_char,
    );
    evbuffer_free(evb);
    free(prefix as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_get_time(
    mut ep: *mut event_payload,
    mut name: *const ::core::ffi::c_char,
) -> time_t {
    let mut epi: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    epi = event_payload_find(ep, name);
    if epi.is_null()
        || (*epi).type_0 as ::core::ffi::c_uint
            != EVENT_PAYLOAD_TIME as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return 0 as time_t;
    }
    return (*epi).c2rust_unnamed.time;
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_get_int(
    mut ep: *mut event_payload,
    mut name: *const ::core::ffi::c_char,
    mut value: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut epi: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    epi = event_payload_find(ep, name);
    if epi.is_null()
        || (*epi).type_0 as ::core::ffi::c_uint
            != EVENT_PAYLOAD_INT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return -(1 as ::core::ffi::c_int);
    }
    *value = (*epi).c2rust_unnamed.number;
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_get_uint(
    mut ep: *mut event_payload,
    mut name: *const ::core::ffi::c_char,
    mut value: *mut u_int,
) -> ::core::ffi::c_int {
    let mut epi: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    epi = event_payload_find(ep, name);
    if epi.is_null()
        || (*epi).type_0 as ::core::ffi::c_uint
            != EVENT_PAYLOAD_UINT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return -(1 as ::core::ffi::c_int);
    }
    *value = (*epi).c2rust_unnamed.unsigned_number;
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_get_client(
    mut ep: *mut event_payload,
    mut name: *const ::core::ffi::c_char,
) -> *mut client {
    let mut epi: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    epi = event_payload_find(ep, name);
    if epi.is_null()
        || (*epi).type_0 as ::core::ffi::c_uint
            != EVENT_PAYLOAD_CLIENT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return ::core::ptr::null_mut::<client>();
    }
    return (*epi).c2rust_unnamed.client;
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_get_session(
    mut ep: *mut event_payload,
    mut name: *const ::core::ffi::c_char,
) -> *mut session {
    let mut epi: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    epi = event_payload_find(ep, name);
    if epi.is_null()
        || (*epi).type_0 as ::core::ffi::c_uint
            != EVENT_PAYLOAD_SESSION as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return ::core::ptr::null_mut::<session>();
    }
    return (*epi).c2rust_unnamed.session;
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_get_window(
    mut ep: *mut event_payload,
    mut name: *const ::core::ffi::c_char,
) -> *mut window {
    let mut epi: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    epi = event_payload_find(ep, name);
    if epi.is_null()
        || (*epi).type_0 as ::core::ffi::c_uint
            != EVENT_PAYLOAD_WINDOW as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return ::core::ptr::null_mut::<window>();
    }
    return (*epi).c2rust_unnamed.window;
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_get_pane(
    mut ep: *mut event_payload,
    mut name: *const ::core::ffi::c_char,
) -> *mut window_pane {
    let mut epi: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    epi = event_payload_find(ep, name);
    if epi.is_null()
        || (*epi).type_0 as ::core::ffi::c_uint
            != EVENT_PAYLOAD_PANE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return ::core::ptr::null_mut::<window_pane>();
    }
    return (*epi).c2rust_unnamed.pane;
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_get_pointer(
    mut ep: *mut event_payload,
    mut name: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_void {
    let mut epi: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    epi = event_payload_find(ep, name);
    if epi.is_null()
        || (*epi).type_0 as ::core::ffi::c_uint
            != EVENT_PAYLOAD_POINTER as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    return (*epi).c2rust_unnamed.pointer.ptr;
}
