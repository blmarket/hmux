use crate::src::ffi::libc::{free, sscanf, strchr, strcmp};
use crate::src::reactor::{event_add, event_del, event_initialized, event_pending, event_set};
use crate::src::format::{format_create, format_defaults, format_expand, format_free, format_true};
use crate::src::log::log_debug;
use crate::src::server::current_time;
use crate::src::session::{
    session_add_ref, session_find_by_id, session_remove_ref, sessions_RB_MINMAX,
};
pub use crate::src::session::sessions;
use crate::src::window::{
    window_find_by_id, window_pane_find_by_id, winlinks_RB_MINMAX, winlinks_RB_NEXT,
};
use crate::src::xmalloc::{xcalloc, xstrdup};
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
pub use crate::src::shared::monitor::{
    monitor_cb, monitor_change, monitor_item, monitor_item_entry, monitor_items, monitor_pane,
    monitor_pane_entry, monitor_panes, monitor_set, monitor_window, monitor_window_entry,
    monitor_windows,
};
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
pub use crate::src::shared::format::{FORMAT_NOJOBS};
pub use crate::src::shared::monitor::{
    MONITOR_ALL_PANES, MONITOR_ALL_WINDOWS, MONITOR_NOTIFY_INITIAL, MONITOR_NOTIFY_TRUE,
    MONITOR_PANE, MONITOR_SESSION, MONITOR_WINDOW, monitor_type,
};
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::{screen_write_cline};
pub use crate::src::shared::hyperlinks::{hyperlinks};
pub use crate::src::shared::pane::{
    window_pane_offset, window_pane_resize, window_pane_resize_entry, window_pane_resizes,
};
pub use crate::src::shared::tree::{RB_BLACK, RB_NEGINF, RB_RED};
pub use crate::src::shared::event::{EV_TIMEOUT};
pub use crate::src::shared::display::{visible_range, visible_ranges};
pub use crate::src::shared::layout::{layout_geometry};
pub use crate::src::shared::mouse::{mouse_event};
use crate::src::shared::client::*;
use crate::src::shared::terminal::*;
use crate::src::shared::display::*;
use crate::src::shared::layout::*;
use crate::src::shared::message::*;
use crate::src::shared::abi::*;
use crate::src::shared::colour::*;
use crate::src::shared::grid::*;
use crate::src::shared::key::*;
use crate::src::shared::style::*;

pub use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_12;
pub use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_13;

unsafe extern "C" fn monitor_get_session(mut ms: *mut monitor_set) -> *mut session {
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    if !(*ms).client.is_null() {
        return (*(*ms).client).session;
    }
    s = (*ms).session;
    if s.is_null() {
        return sessions_RB_MINMAX(&raw mut sessions, RB_NEGINF);
    }
    if session_find_by_id((*s).id) != s {
        return ::core::ptr::null_mut::<session>();
    }
    return s;
}
unsafe extern "C" fn monitor_create_formats(
    mut c: *mut client,
    mut s: *mut session,
    mut wl: *mut winlink,
    mut wp: *mut window_pane,
) -> *mut format_tree {
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    ft = format_create(
        ::core::ptr::null_mut::<client>(),
        ::core::ptr::null_mut::<cmdq_item>(),
        0 as ::core::ffi::c_int,
        FORMAT_NOJOBS,
    );
    format_defaults(ft, c, s, wl, wp);
    return ft;
}
unsafe extern "C" fn monitor_item_cmp(
    mut m1: *mut monitor_item,
    mut m2: *mut monitor_item,
) -> ::core::ffi::c_int {
    return strcmp((*m1).name, (*m2).name);
}
unsafe extern "C" fn monitor_items_RB_MINMAX(
    mut head: *mut monitor_items,
    mut val: ::core::ffi::c_int,
) -> *mut monitor_item {
    let mut tmp: *mut monitor_item = (*head).rbh_root;
    let mut parent: *mut monitor_item = ::core::ptr::null_mut::<monitor_item>();
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
unsafe extern "C" fn monitor_items_RB_NEXT(mut elm: *mut monitor_item) -> *mut monitor_item {
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
unsafe extern "C" fn monitor_items_RB_REMOVE_COLOR(
    mut head: *mut monitor_items,
    mut parent: *mut monitor_item,
    mut elm: *mut monitor_item,
) {
    let mut tmp: *mut monitor_item = ::core::ptr::null_mut::<monitor_item>();
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
                    let mut oleft: *mut monitor_item = ::core::ptr::null_mut::<monitor_item>();
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
                    let mut oright: *mut monitor_item = ::core::ptr::null_mut::<monitor_item>();
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
unsafe extern "C" fn monitor_items_RB_INSERT_COLOR(
    mut head: *mut monitor_items,
    mut elm: *mut monitor_item,
) {
    let mut parent: *mut monitor_item = ::core::ptr::null_mut::<monitor_item>();
    let mut gparent: *mut monitor_item = ::core::ptr::null_mut::<monitor_item>();
    let mut tmp: *mut monitor_item = ::core::ptr::null_mut::<monitor_item>();
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
unsafe extern "C" fn monitor_items_RB_INSERT(
    mut head: *mut monitor_items,
    mut elm: *mut monitor_item,
) -> *mut monitor_item {
    let mut tmp: *mut monitor_item = ::core::ptr::null_mut::<monitor_item>();
    let mut parent: *mut monitor_item = ::core::ptr::null_mut::<monitor_item>();
    let mut comp: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    tmp = (*head).rbh_root;
    while !tmp.is_null() {
        parent = tmp;
        comp = monitor_item_cmp(elm, parent);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_right;
        } else {
            return tmp;
        }
    }
    (*elm).entry.rbe_parent = parent;
    (*elm).entry.rbe_right = ::core::ptr::null_mut::<monitor_item>();
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
    monitor_items_RB_INSERT_COLOR(head, elm);
    return ::core::ptr::null_mut::<monitor_item>();
}
unsafe extern "C" fn monitor_items_RB_REMOVE(
    mut head: *mut monitor_items,
    mut elm: *mut monitor_item,
) -> *mut monitor_item {
    let mut current_block: u64;
    let mut child: *mut monitor_item = ::core::ptr::null_mut::<monitor_item>();
    let mut parent: *mut monitor_item = ::core::ptr::null_mut::<monitor_item>();
    let mut old: *mut monitor_item = elm;
    let mut color: ::core::ffi::c_int = 0;
    if (*elm).entry.rbe_left.is_null() {
        child = (*elm).entry.rbe_right;
        current_block = 7245201122033322888;
    } else if (*elm).entry.rbe_right.is_null() {
        child = (*elm).entry.rbe_left;
        current_block = 7245201122033322888;
    } else {
        let mut left: *mut monitor_item = ::core::ptr::null_mut::<monitor_item>();
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
        current_block = 12669146338688518024;
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
        monitor_items_RB_REMOVE_COLOR(head, parent, child);
    }
    return old;
}
unsafe extern "C" fn monitor_items_RB_FIND(
    mut head: *mut monitor_items,
    mut elm: *mut monitor_item,
) -> *mut monitor_item {
    let mut tmp: *mut monitor_item = (*head).rbh_root;
    let mut comp: ::core::ffi::c_int = 0;
    while !tmp.is_null() {
        comp = monitor_item_cmp(elm, tmp);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_right;
        } else {
            return tmp;
        }
    }
    return ::core::ptr::null_mut::<monitor_item>();
}
unsafe extern "C" fn monitor_pane_cmp(
    mut mp1: *mut monitor_pane,
    mut mp2: *mut monitor_pane,
) -> ::core::ffi::c_int {
    if (*mp1).pane < (*mp2).pane {
        return -(1 as ::core::ffi::c_int);
    }
    if (*mp1).pane > (*mp2).pane {
        return 1 as ::core::ffi::c_int;
    }
    if (*mp1).idx < (*mp2).idx {
        return -(1 as ::core::ffi::c_int);
    }
    if (*mp1).idx > (*mp2).idx {
        return 1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn monitor_panes_RB_REMOVE_COLOR(
    mut head: *mut monitor_panes,
    mut parent: *mut monitor_pane,
    mut elm: *mut monitor_pane,
) {
    let mut tmp: *mut monitor_pane = ::core::ptr::null_mut::<monitor_pane>();
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
                    let mut oleft: *mut monitor_pane = ::core::ptr::null_mut::<monitor_pane>();
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
                    let mut oright: *mut monitor_pane = ::core::ptr::null_mut::<monitor_pane>();
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
unsafe extern "C" fn monitor_panes_RB_REMOVE(
    mut head: *mut monitor_panes,
    mut elm: *mut monitor_pane,
) -> *mut monitor_pane {
    let mut current_block: u64;
    let mut child: *mut monitor_pane = ::core::ptr::null_mut::<monitor_pane>();
    let mut parent: *mut monitor_pane = ::core::ptr::null_mut::<monitor_pane>();
    let mut old: *mut monitor_pane = elm;
    let mut color: ::core::ffi::c_int = 0;
    if (*elm).entry.rbe_left.is_null() {
        child = (*elm).entry.rbe_right;
        current_block = 7245201122033322888;
    } else if (*elm).entry.rbe_right.is_null() {
        child = (*elm).entry.rbe_left;
        current_block = 7245201122033322888;
    } else {
        let mut left: *mut monitor_pane = ::core::ptr::null_mut::<monitor_pane>();
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
        current_block = 11011803464182723597;
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
        monitor_panes_RB_REMOVE_COLOR(head, parent, child);
    }
    return old;
}
unsafe extern "C" fn monitor_panes_RB_NEXT(mut elm: *mut monitor_pane) -> *mut monitor_pane {
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
unsafe extern "C" fn monitor_panes_RB_MINMAX(
    mut head: *mut monitor_panes,
    mut val: ::core::ffi::c_int,
) -> *mut monitor_pane {
    let mut tmp: *mut monitor_pane = (*head).rbh_root;
    let mut parent: *mut monitor_pane = ::core::ptr::null_mut::<monitor_pane>();
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
unsafe extern "C" fn monitor_panes_RB_FIND(
    mut head: *mut monitor_panes,
    mut elm: *mut monitor_pane,
) -> *mut monitor_pane {
    let mut tmp: *mut monitor_pane = (*head).rbh_root;
    let mut comp: ::core::ffi::c_int = 0;
    while !tmp.is_null() {
        comp = monitor_pane_cmp(elm, tmp);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_right;
        } else {
            return tmp;
        }
    }
    return ::core::ptr::null_mut::<monitor_pane>();
}
unsafe extern "C" fn monitor_panes_RB_INSERT(
    mut head: *mut monitor_panes,
    mut elm: *mut monitor_pane,
) -> *mut monitor_pane {
    let mut tmp: *mut monitor_pane = ::core::ptr::null_mut::<monitor_pane>();
    let mut parent: *mut monitor_pane = ::core::ptr::null_mut::<monitor_pane>();
    let mut comp: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    tmp = (*head).rbh_root;
    while !tmp.is_null() {
        parent = tmp;
        comp = monitor_pane_cmp(elm, parent);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_right;
        } else {
            return tmp;
        }
    }
    (*elm).entry.rbe_parent = parent;
    (*elm).entry.rbe_right = ::core::ptr::null_mut::<monitor_pane>();
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
    monitor_panes_RB_INSERT_COLOR(head, elm);
    return ::core::ptr::null_mut::<monitor_pane>();
}
unsafe extern "C" fn monitor_panes_RB_INSERT_COLOR(
    mut head: *mut monitor_panes,
    mut elm: *mut monitor_pane,
) {
    let mut parent: *mut monitor_pane = ::core::ptr::null_mut::<monitor_pane>();
    let mut gparent: *mut monitor_pane = ::core::ptr::null_mut::<monitor_pane>();
    let mut tmp: *mut monitor_pane = ::core::ptr::null_mut::<monitor_pane>();
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
unsafe extern "C" fn monitor_window_cmp(
    mut mw1: *mut monitor_window,
    mut mw2: *mut monitor_window,
) -> ::core::ffi::c_int {
    if (*mw1).window < (*mw2).window {
        return -(1 as ::core::ffi::c_int);
    }
    if (*mw1).window > (*mw2).window {
        return 1 as ::core::ffi::c_int;
    }
    if (*mw1).idx < (*mw2).idx {
        return -(1 as ::core::ffi::c_int);
    }
    if (*mw1).idx > (*mw2).idx {
        return 1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn monitor_windows_RB_INSERT_COLOR(
    mut head: *mut monitor_windows,
    mut elm: *mut monitor_window,
) {
    let mut parent: *mut monitor_window = ::core::ptr::null_mut::<monitor_window>();
    let mut gparent: *mut monitor_window = ::core::ptr::null_mut::<monitor_window>();
    let mut tmp: *mut monitor_window = ::core::ptr::null_mut::<monitor_window>();
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
unsafe extern "C" fn monitor_windows_RB_INSERT(
    mut head: *mut monitor_windows,
    mut elm: *mut monitor_window,
) -> *mut monitor_window {
    let mut tmp: *mut monitor_window = ::core::ptr::null_mut::<monitor_window>();
    let mut parent: *mut monitor_window = ::core::ptr::null_mut::<monitor_window>();
    let mut comp: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    tmp = (*head).rbh_root;
    while !tmp.is_null() {
        parent = tmp;
        comp = monitor_window_cmp(elm, parent);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_right;
        } else {
            return tmp;
        }
    }
    (*elm).entry.rbe_parent = parent;
    (*elm).entry.rbe_right = ::core::ptr::null_mut::<monitor_window>();
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
    monitor_windows_RB_INSERT_COLOR(head, elm);
    return ::core::ptr::null_mut::<monitor_window>();
}
unsafe extern "C" fn monitor_windows_RB_FIND(
    mut head: *mut monitor_windows,
    mut elm: *mut monitor_window,
) -> *mut monitor_window {
    let mut tmp: *mut monitor_window = (*head).rbh_root;
    let mut comp: ::core::ffi::c_int = 0;
    while !tmp.is_null() {
        comp = monitor_window_cmp(elm, tmp);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_right;
        } else {
            return tmp;
        }
    }
    return ::core::ptr::null_mut::<monitor_window>();
}
unsafe extern "C" fn monitor_windows_RB_NEXT(mut elm: *mut monitor_window) -> *mut monitor_window {
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
unsafe extern "C" fn monitor_windows_RB_REMOVE_COLOR(
    mut head: *mut monitor_windows,
    mut parent: *mut monitor_window,
    mut elm: *mut monitor_window,
) {
    let mut tmp: *mut monitor_window = ::core::ptr::null_mut::<monitor_window>();
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
                    let mut oleft: *mut monitor_window = ::core::ptr::null_mut::<monitor_window>();
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
                    let mut oright: *mut monitor_window = ::core::ptr::null_mut::<monitor_window>();
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
unsafe extern "C" fn monitor_windows_RB_REMOVE(
    mut head: *mut monitor_windows,
    mut elm: *mut monitor_window,
) -> *mut monitor_window {
    let mut current_block: u64;
    let mut child: *mut monitor_window = ::core::ptr::null_mut::<monitor_window>();
    let mut parent: *mut monitor_window = ::core::ptr::null_mut::<monitor_window>();
    let mut old: *mut monitor_window = elm;
    let mut color: ::core::ffi::c_int = 0;
    if (*elm).entry.rbe_left.is_null() {
        child = (*elm).entry.rbe_right;
        current_block = 7245201122033322888;
    } else if (*elm).entry.rbe_right.is_null() {
        child = (*elm).entry.rbe_left;
        current_block = 7245201122033322888;
    } else {
        let mut left: *mut monitor_window = ::core::ptr::null_mut::<monitor_window>();
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
        current_block = 9601630785265386486;
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
        monitor_windows_RB_REMOVE_COLOR(head, parent, child);
    }
    return old;
}
unsafe extern "C" fn monitor_windows_RB_MINMAX(
    mut head: *mut monitor_windows,
    mut val: ::core::ffi::c_int,
) -> *mut monitor_window {
    let mut tmp: *mut monitor_window = (*head).rbh_root;
    let mut parent: *mut monitor_window = ::core::ptr::null_mut::<monitor_window>();
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
unsafe extern "C" fn monitor_free_item(mut ms: *mut monitor_set, mut me: *mut monitor_item) {
    let mut mp: *mut monitor_pane = ::core::ptr::null_mut::<monitor_pane>();
    let mut mp1: *mut monitor_pane = ::core::ptr::null_mut::<monitor_pane>();
    let mut mw: *mut monitor_window = ::core::ptr::null_mut::<monitor_window>();
    let mut mw1: *mut monitor_window = ::core::ptr::null_mut::<monitor_window>();
    mp = monitor_panes_RB_MINMAX(&raw mut (*me).panes, RB_NEGINF);
    while !mp.is_null() && {
        mp1 = monitor_panes_RB_NEXT(mp);
        1 as ::core::ffi::c_int != 0
    } {
        monitor_panes_RB_REMOVE(&raw mut (*me).panes, mp);
        free((*mp).last as *mut ::core::ffi::c_void);
        free(mp as *mut ::core::ffi::c_void);
        mp = mp1;
    }
    mw = monitor_windows_RB_MINMAX(&raw mut (*me).windows, RB_NEGINF);
    while !mw.is_null() && {
        mw1 = monitor_windows_RB_NEXT(mw);
        1 as ::core::ffi::c_int != 0
    } {
        monitor_windows_RB_REMOVE(&raw mut (*me).windows, mw);
        free((*mw).last as *mut ::core::ffi::c_void);
        free(mw as *mut ::core::ffi::c_void);
        mw = mw1;
    }
    free((*me).last as *mut ::core::ffi::c_void);
    monitor_items_RB_REMOVE(&raw mut (*ms).items, me);
    free((*me).name as *mut ::core::ffi::c_void);
    free((*me).format as *mut ::core::ffi::c_void);
    free(me as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn monitor_report(
    mut ms: *mut monitor_set,
    mut me: *mut monitor_item,
    mut s: *mut session,
    mut wl: *mut winlink,
    mut wp: *mut window_pane,
    mut value: *const ::core::ffi::c_char,
    mut last: *const ::core::ffi::c_char,
) {
    let mut change: monitor_change = monitor_change {
        name: ::core::ptr::null::<::core::ffi::c_char>(),
        value: ::core::ptr::null::<::core::ffi::c_char>(),
        last: ::core::ptr::null::<::core::ffi::c_char>(),
        c: ::core::ptr::null_mut::<client>(),
        s: ::core::ptr::null_mut::<session>(),
        wl: ::core::ptr::null_mut::<winlink>(),
        wp: ::core::ptr::null_mut::<window_pane>(),
    };
    log_debug(
        b"%s: %s changed to %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"monitor_report\0" as *const u8 as *const ::core::ffi::c_char,
        (*me).name,
        value,
    );
    (*me).fire_count = (*me).fire_count.wrapping_add(1);
    (*me).fire_time = current_time;
    change.name = (*me).name;
    change.value = value;
    change.last = last;
    change.c = (*ms).client;
    change.s = s;
    change.wl = wl;
    change.wp = wp;
    (*ms).cb.expect("non-null function pointer")(&raw mut change, (*ms).data);
}
unsafe extern "C" fn monitor_check_value(
    mut ms: *mut monitor_set,
    mut me: *mut monitor_item,
    mut s: *mut session,
    mut wl: *mut winlink,
    mut wp: *mut window_pane,
    mut value: *mut ::core::ffi::c_char,
    mut last: *mut *mut ::core::ffi::c_char,
) {
    if (*last).is_null() {
        *last = value;
        if (*me).flags & MONITOR_NOTIFY_INITIAL != 0
            && (!(*me).flags & MONITOR_NOTIFY_TRUE != 0 || format_true(value) != 0)
        {
            monitor_report(
                ms,
                me,
                s,
                wl,
                wp,
                value,
                ::core::ptr::null::<::core::ffi::c_char>(),
            );
        }
        return;
    }
    if strcmp(value, *last) == 0 as ::core::ffi::c_int {
        free(value as *mut ::core::ffi::c_void);
        return;
    }
    if !(*me).flags & MONITOR_NOTIFY_TRUE != 0 || format_true(value) != 0 {
        monitor_report(ms, me, s, wl, wp, value, *last);
    }
    free(*last as *mut ::core::ffi::c_void);
    *last = value;
}
unsafe extern "C" fn monitor_check_session(
    mut ms: *mut monitor_set,
    mut me: *mut monitor_item,
    mut ft: *mut format_tree,
) {
    let mut s: *mut session = monitor_get_session(ms);
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    value = format_expand(ft, (*me).format);
    monitor_check_value(
        ms,
        me,
        s,
        ::core::ptr::null_mut::<winlink>(),
        ::core::ptr::null_mut::<window_pane>(),
        value,
        &raw mut (*me).last,
    );
}
unsafe extern "C" fn monitor_check_pane(mut ms: *mut monitor_set, mut me: *mut monitor_item) {
    let mut c: *mut client = (*ms).client;
    let mut s: *mut session = monitor_get_session(ms);
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut mp: *mut monitor_pane = ::core::ptr::null_mut::<monitor_pane>();
    let mut find: monitor_pane = monitor_pane {
        pane: 0,
        idx: 0,
        last: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        generation: 0,
        entry: monitor_pane_entry {
            rbe_left: ::core::ptr::null_mut::<monitor_pane>(),
            rbe_right: ::core::ptr::null_mut::<monitor_pane>(),
            rbe_parent: ::core::ptr::null_mut::<monitor_pane>(),
            rbe_color: 0,
        },
    };
    wp = window_pane_find_by_id((*me).id);
    if wp.is_null() || (*wp).fd == -(1 as ::core::ffi::c_int) {
        return;
    }
    w = (*wp).window as *mut window;
    wl = (*w).winlinks.tqh_first;
    while !wl.is_null() {
        if !((*wl).session != s) {
            ft = monitor_create_formats(c, s, wl, wp);
            value = format_expand(ft, (*me).format);
            format_free(ft);
            find.pane = (*wp).id;
            find.idx = (*wl).idx as u_int;
            mp = monitor_panes_RB_FIND(&raw mut (*me).panes, &raw mut find);
            if mp.is_null() {
                mp = xcalloc(
                    1 as size_t,
                    ::core::mem::size_of::<monitor_pane>() as size_t,
                ) as *mut monitor_pane;
                (*mp).pane = (*wp).id;
                (*mp).idx = (*wl).idx as u_int;
                monitor_panes_RB_INSERT(&raw mut (*me).panes, mp);
            }
            monitor_check_value(ms, me, s, wl, wp, value, &raw mut (*mp).last);
        }
        wl = (*wl).wentry.tqe_next;
    }
}
unsafe extern "C" fn monitor_check_all_panes_one(
    mut ms: *mut monitor_set,
    mut me: *mut monitor_item,
    mut ft: *mut format_tree,
    mut wl: *mut winlink,
    mut wp: *mut window_pane,
) {
    let mut s: *mut session = monitor_get_session(ms);
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut mp: *mut monitor_pane = ::core::ptr::null_mut::<monitor_pane>();
    let mut find: monitor_pane = monitor_pane {
        pane: 0,
        idx: 0,
        last: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        generation: 0,
        entry: monitor_pane_entry {
            rbe_left: ::core::ptr::null_mut::<monitor_pane>(),
            rbe_right: ::core::ptr::null_mut::<monitor_pane>(),
            rbe_parent: ::core::ptr::null_mut::<monitor_pane>(),
            rbe_color: 0,
        },
    };
    value = format_expand(ft, (*me).format);
    find.pane = (*wp).id;
    find.idx = (*wl).idx as u_int;
    mp = monitor_panes_RB_FIND(&raw mut (*me).panes, &raw mut find);
    if mp.is_null() {
        mp = xcalloc(
            1 as size_t,
            ::core::mem::size_of::<monitor_pane>() as size_t,
        ) as *mut monitor_pane;
        (*mp).pane = (*wp).id;
        (*mp).idx = (*wl).idx as u_int;
        monitor_panes_RB_INSERT(&raw mut (*me).panes, mp);
    }
    (*mp).generation = (*ms).generation;
    monitor_check_value(ms, me, s, wl, wp, value, &raw mut (*mp).last);
}
unsafe extern "C" fn monitor_sweep_all_panes(mut me: *mut monitor_item, mut generation: u_int) {
    let mut mp: *mut monitor_pane = ::core::ptr::null_mut::<monitor_pane>();
    let mut mp1: *mut monitor_pane = ::core::ptr::null_mut::<monitor_pane>();
    mp = monitor_panes_RB_MINMAX(&raw mut (*me).panes, RB_NEGINF);
    while !mp.is_null() && {
        mp1 = monitor_panes_RB_NEXT(mp);
        1 as ::core::ffi::c_int != 0
    } {
        if !((*mp).generation == generation) {
            monitor_panes_RB_REMOVE(&raw mut (*me).panes, mp);
            free((*mp).last as *mut ::core::ffi::c_void);
            free(mp as *mut ::core::ffi::c_void);
        }
        mp = mp1;
    }
}
unsafe extern "C" fn monitor_check_window(mut ms: *mut monitor_set, mut me: *mut monitor_item) {
    let mut c: *mut client = (*ms).client;
    let mut s: *mut session = monitor_get_session(ms);
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut mw: *mut monitor_window = ::core::ptr::null_mut::<monitor_window>();
    let mut find: monitor_window = monitor_window {
        window: 0,
        idx: 0,
        last: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        generation: 0,
        entry: monitor_window_entry {
            rbe_left: ::core::ptr::null_mut::<monitor_window>(),
            rbe_right: ::core::ptr::null_mut::<monitor_window>(),
            rbe_parent: ::core::ptr::null_mut::<monitor_window>(),
            rbe_color: 0,
        },
    };
    w = window_find_by_id((*me).id);
    if w.is_null() {
        return;
    }
    wl = (*w).winlinks.tqh_first;
    while !wl.is_null() {
        if !((*wl).session != s) {
            ft = monitor_create_formats(c, s, wl, ::core::ptr::null_mut::<window_pane>());
            value = format_expand(ft, (*me).format);
            format_free(ft);
            find.window = (*w).id;
            find.idx = (*wl).idx as u_int;
            mw = monitor_windows_RB_FIND(&raw mut (*me).windows, &raw mut find);
            if mw.is_null() {
                mw = xcalloc(
                    1 as size_t,
                    ::core::mem::size_of::<monitor_window>() as size_t,
                ) as *mut monitor_window;
                (*mw).window = (*w).id;
                (*mw).idx = (*wl).idx as u_int;
                monitor_windows_RB_INSERT(&raw mut (*me).windows, mw);
            }
            monitor_check_value(
                ms,
                me,
                s,
                wl,
                ::core::ptr::null_mut::<window_pane>(),
                value,
                &raw mut (*mw).last,
            );
        }
        wl = (*wl).wentry.tqe_next;
    }
}
unsafe extern "C" fn monitor_check_all_windows_one(
    mut ms: *mut monitor_set,
    mut me: *mut monitor_item,
    mut ft: *mut format_tree,
    mut wl: *mut winlink,
) {
    let mut s: *mut session = monitor_get_session(ms);
    let mut w: *mut window = (*wl).window;
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut mw: *mut monitor_window = ::core::ptr::null_mut::<monitor_window>();
    let mut find: monitor_window = monitor_window {
        window: 0,
        idx: 0,
        last: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        generation: 0,
        entry: monitor_window_entry {
            rbe_left: ::core::ptr::null_mut::<monitor_window>(),
            rbe_right: ::core::ptr::null_mut::<monitor_window>(),
            rbe_parent: ::core::ptr::null_mut::<monitor_window>(),
            rbe_color: 0,
        },
    };
    value = format_expand(ft, (*me).format);
    find.window = (*w).id;
    find.idx = (*wl).idx as u_int;
    mw = monitor_windows_RB_FIND(&raw mut (*me).windows, &raw mut find);
    if mw.is_null() {
        mw = xcalloc(
            1 as size_t,
            ::core::mem::size_of::<monitor_window>() as size_t,
        ) as *mut monitor_window;
        (*mw).window = (*w).id;
        (*mw).idx = (*wl).idx as u_int;
        monitor_windows_RB_INSERT(&raw mut (*me).windows, mw);
    }
    (*mw).generation = (*ms).generation;
    monitor_check_value(
        ms,
        me,
        s,
        wl,
        ::core::ptr::null_mut::<window_pane>(),
        value,
        &raw mut (*mw).last,
    );
}
unsafe extern "C" fn monitor_sweep_all_windows(mut me: *mut monitor_item, mut generation: u_int) {
    let mut mw: *mut monitor_window = ::core::ptr::null_mut::<monitor_window>();
    let mut mw1: *mut monitor_window = ::core::ptr::null_mut::<monitor_window>();
    mw = monitor_windows_RB_MINMAX(&raw mut (*me).windows, RB_NEGINF);
    while !mw.is_null() && {
        mw1 = monitor_windows_RB_NEXT(mw);
        1 as ::core::ffi::c_int != 0
    } {
        if !((*mw).generation == generation) {
            monitor_windows_RB_REMOVE(&raw mut (*me).windows, mw);
            free((*mw).last as *mut ::core::ffi::c_void);
            free(mw as *mut ::core::ffi::c_void);
        }
        mw = mw1;
    }
}
unsafe extern "C" fn monitor_check_sessions(mut ms: *mut monitor_set) {
    let mut c: *mut client = (*ms).client;
    let mut s: *mut session = monitor_get_session(ms);
    let mut me: *mut monitor_item = ::core::ptr::null_mut::<monitor_item>();
    let mut me1: *mut monitor_item = ::core::ptr::null_mut::<monitor_item>();
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    ft = monitor_create_formats(
        c,
        s,
        ::core::ptr::null_mut::<winlink>(),
        ::core::ptr::null_mut::<window_pane>(),
    );
    me = monitor_items_RB_MINMAX(&raw mut (*ms).items, RB_NEGINF);
    while !me.is_null() && {
        me1 = monitor_items_RB_NEXT(me);
        1 as ::core::ffi::c_int != 0
    } {
        if (*me).type_0 as ::core::ffi::c_uint
            == MONITOR_SESSION as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            monitor_check_session(ms, me, ft);
        }
        me = me1;
    }
    format_free(ft);
}
unsafe extern "C" fn monitor_check_panes_windows(mut ms: *mut monitor_set) {
    let mut me: *mut monitor_item = ::core::ptr::null_mut::<monitor_item>();
    let mut me1: *mut monitor_item = ::core::ptr::null_mut::<monitor_item>();
    me = monitor_items_RB_MINMAX(&raw mut (*ms).items, RB_NEGINF);
    while !me.is_null() && {
        me1 = monitor_items_RB_NEXT(me);
        1 as ::core::ffi::c_int != 0
    } {
        match (*me).type_0 as ::core::ffi::c_uint {
            1 => {
                monitor_check_pane(ms, me);
            }
            3 => {
                monitor_check_window(ms, me);
            }
            0 | 2 | 4 | _ => {}
        }
        me = me1;
    }
}
unsafe extern "C" fn monitor_check_all_panes(mut ms: *mut monitor_set) {
    let mut c: *mut client = (*ms).client;
    let mut s: *mut session = monitor_get_session(ms);
    let mut me: *mut monitor_item = ::core::ptr::null_mut::<monitor_item>();
    let mut me1: *mut monitor_item = ::core::ptr::null_mut::<monitor_item>();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    (*ms).generation = (*ms).generation.wrapping_add(1);
    if (*ms).generation == 0 as u_int {
        (*ms).generation = 1 as u_int;
    }
    wl = winlinks_RB_MINMAX(&raw mut (*s).windows, RB_NEGINF);
    while !wl.is_null() {
        wp = (*(*wl).window).panes.tqh_first;
        while !wp.is_null() {
            ft = monitor_create_formats(c, s, wl, wp);
            me = monitor_items_RB_MINMAX(&raw mut (*ms).items, RB_NEGINF);
            while !me.is_null() && {
                me1 = monitor_items_RB_NEXT(me);
                1 as ::core::ffi::c_int != 0
            } {
                if !((*me).type_0 as ::core::ffi::c_uint
                    != MONITOR_ALL_PANES as ::core::ffi::c_int as ::core::ffi::c_uint)
                {
                    monitor_check_all_panes_one(ms, me, ft, wl, wp);
                }
                me = me1;
            }
            format_free(ft);
            wp = (*wp).entry.tqe_next;
        }
        wl = winlinks_RB_NEXT(wl);
    }
    me = monitor_items_RB_MINMAX(&raw mut (*ms).items, RB_NEGINF);
    while !me.is_null() && {
        me1 = monitor_items_RB_NEXT(me);
        1 as ::core::ffi::c_int != 0
    } {
        if (*me).type_0 as ::core::ffi::c_uint
            == MONITOR_ALL_PANES as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            monitor_sweep_all_panes(me, (*ms).generation);
        }
        me = me1;
    }
}
unsafe extern "C" fn monitor_check_all_windows(mut ms: *mut monitor_set) {
    let mut c: *mut client = (*ms).client;
    let mut s: *mut session = monitor_get_session(ms);
    let mut me: *mut monitor_item = ::core::ptr::null_mut::<monitor_item>();
    let mut me1: *mut monitor_item = ::core::ptr::null_mut::<monitor_item>();
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    (*ms).generation = (*ms).generation.wrapping_add(1);
    if (*ms).generation == 0 as u_int {
        (*ms).generation = 1 as u_int;
    }
    wl = winlinks_RB_MINMAX(&raw mut (*s).windows, RB_NEGINF);
    while !wl.is_null() {
        ft = monitor_create_formats(c, s, wl, ::core::ptr::null_mut::<window_pane>());
        me = monitor_items_RB_MINMAX(&raw mut (*ms).items, RB_NEGINF);
        while !me.is_null() && {
            me1 = monitor_items_RB_NEXT(me);
            1 as ::core::ffi::c_int != 0
        } {
            if !((*me).type_0 as ::core::ffi::c_uint
                != MONITOR_ALL_WINDOWS as ::core::ffi::c_int as ::core::ffi::c_uint)
            {
                monitor_check_all_windows_one(ms, me, ft, wl);
            }
            me = me1;
        }
        format_free(ft);
        wl = winlinks_RB_NEXT(wl);
    }
    me = monitor_items_RB_MINMAX(&raw mut (*ms).items, RB_NEGINF);
    while !me.is_null() && {
        me1 = monitor_items_RB_NEXT(me);
        1 as ::core::ffi::c_int != 0
    } {
        if (*me).type_0 as ::core::ffi::c_uint
            == MONITOR_ALL_WINDOWS as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            monitor_sweep_all_windows(me, (*ms).generation);
        }
        me = me1;
    }
}
unsafe extern "C" fn monitor_timer(
    mut fd: ::core::ffi::c_int,
    mut events: ::core::ffi::c_short,
    mut data: *mut ::core::ffi::c_void,
) {
    let mut ms: *mut monitor_set = data as *mut monitor_set;
    let mut me: *mut monitor_item = ::core::ptr::null_mut::<monitor_item>();
    let mut tv: timeval = timeval {
        tv_sec: 1 as __time_t,
        tv_usec: 0,
    };
    let mut have_session: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut have_all_panes: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut have_all_windows: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    log_debug(
        b"%s: timer fired\0" as *const u8 as *const ::core::ffi::c_char,
        b"monitor_timer\0" as *const u8 as *const ::core::ffi::c_char,
    );
    event_add(&raw mut (*ms).timer, &raw mut tv);
    if monitor_get_session(ms).is_null() {
        return;
    }
    me = monitor_items_RB_MINMAX(&raw mut (*ms).items, RB_NEGINF);
    while !me.is_null() {
        match (*me).type_0 as ::core::ffi::c_uint {
            0 => {
                have_session = 1 as ::core::ffi::c_int;
            }
            2 => {
                have_all_panes = 1 as ::core::ffi::c_int;
            }
            4 => {
                have_all_windows = 1 as ::core::ffi::c_int;
            }
            1 | 3 | _ => {}
        }
        me = monitor_items_RB_NEXT(me);
    }
    if have_session != 0 {
        monitor_check_sessions(ms);
    }
    monitor_check_panes_windows(ms);
    if have_all_panes != 0 {
        monitor_check_all_panes(ms);
    }
    if have_all_windows != 0 {
        monitor_check_all_windows(ms);
    }
}
unsafe extern "C" fn monitor_create(
    mut cb: monitor_cb,
    mut data: *mut ::core::ffi::c_void,
) -> *mut monitor_set {
    let mut ms: *mut monitor_set = ::core::ptr::null_mut::<monitor_set>();
    ms = xcalloc(1 as size_t, ::core::mem::size_of::<monitor_set>() as size_t) as *mut monitor_set;
    (*ms).cb = cb;
    (*ms).data = data;
    (*ms).items.rbh_root = ::core::ptr::null_mut::<monitor_item>();
    return ms;
}
#[no_mangle]
pub unsafe extern "C" fn monitor_create_client(
    mut c: *mut client,
    mut cb: monitor_cb,
    mut data: *mut ::core::ffi::c_void,
) -> *mut monitor_set {
    let mut ms: *mut monitor_set = ::core::ptr::null_mut::<monitor_set>();
    ms = monitor_create(cb, data);
    (*ms).client = c;
    return ms as *mut monitor_set;
}
#[no_mangle]
pub unsafe extern "C" fn monitor_create_session(
    mut s: *mut session,
    mut cb: monitor_cb,
    mut data: *mut ::core::ffi::c_void,
) -> *mut monitor_set {
    let mut ms: *mut monitor_set = ::core::ptr::null_mut::<monitor_set>();
    ms = monitor_create(cb, data);
    (*ms).session = s;
    if !s.is_null() {
        session_add_ref(
            s,
            b"monitor_create_session\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    return ms;
}
#[no_mangle]
pub unsafe extern "C" fn monitor_destroy(mut ms: *mut monitor_set) {
    let mut me: *mut monitor_item = ::core::ptr::null_mut::<monitor_item>();
    let mut me1: *mut monitor_item = ::core::ptr::null_mut::<monitor_item>();
    if !ms.is_null() {
        if event_initialized(&raw mut (*ms).timer) != 0 {
            event_del(&raw mut (*ms).timer);
        }
        me = monitor_items_RB_MINMAX(&raw mut (*ms).items, RB_NEGINF);
        while !me.is_null() && {
            me1 = monitor_items_RB_NEXT(me);
            1 as ::core::ffi::c_int != 0
        } {
            monitor_free_item(ms, me);
            me = me1;
        }
        if !(*ms).session.is_null() {
            session_remove_ref(
                (*ms).session,
                b"monitor_destroy\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        free(ms as *mut ::core::ffi::c_void);
    }
}
#[no_mangle]
pub unsafe extern "C" fn monitor_parse(
    mut value: *const ::core::ffi::c_char,
    mut name: *mut *mut ::core::ffi::c_char,
    mut type_0: *mut monitor_type,
    mut id: *mut ::core::ffi::c_int,
    mut format: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut current_block: u64;
    let mut copy: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut what: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut split: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    copy = xstrdup(value);
    *id = -(1 as ::core::ffi::c_int);
    what = strchr(copy, ':' as i32);
    if !what.is_null() {
        let fresh0 = what;
        what = what.offset(1);
        *fresh0 = '\0' as i32 as ::core::ffi::c_char;
        split = strchr(what, ':' as i32);
        if !split.is_null() {
            let fresh1 = split;
            split = split.offset(1);
            *fresh1 = '\0' as i32 as ::core::ffi::c_char;
            if strcmp(what, b"%*\0" as *const u8 as *const ::core::ffi::c_char)
                == 0 as ::core::ffi::c_int
            {
                *type_0 = MONITOR_ALL_PANES;
                current_block = 3512920355445576850;
            } else if sscanf(
                what,
                b"%%%d\0" as *const u8 as *const ::core::ffi::c_char,
                id,
            ) == 1 as ::core::ffi::c_int
                && *id >= 0 as ::core::ffi::c_int
            {
                *type_0 = MONITOR_PANE;
                current_block = 3512920355445576850;
            } else if strcmp(what, b"@*\0" as *const u8 as *const ::core::ffi::c_char)
                == 0 as ::core::ffi::c_int
            {
                *type_0 = MONITOR_ALL_WINDOWS;
                current_block = 3512920355445576850;
            } else if sscanf(
                what,
                b"@%d\0" as *const u8 as *const ::core::ffi::c_char,
                id,
            ) == 1 as ::core::ffi::c_int
                && *id >= 0 as ::core::ffi::c_int
            {
                *type_0 = MONITOR_WINDOW;
                current_block = 3512920355445576850;
            } else if *what as ::core::ffi::c_int == '\0' as i32 {
                *type_0 = MONITOR_SESSION;
                current_block = 3512920355445576850;
            } else {
                current_block = 7799373935801088419;
            }
            match current_block {
                7799373935801088419 => {}
                _ => {
                    *name = xstrdup(copy);
                    *format = xstrdup(split);
                    free(copy as *mut ::core::ffi::c_void);
                    return 0 as ::core::ffi::c_int;
                }
            }
        }
    }
    free(copy as *mut ::core::ffi::c_void);
    return -(1 as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn monitor_add(
    mut ms: *mut monitor_set,
    mut name: *const ::core::ffi::c_char,
    mut type_0: monitor_type,
    mut id: ::core::ffi::c_int,
    mut format: *const ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
) {
    let mut me: *mut monitor_item = ::core::ptr::null_mut::<monitor_item>();
    let mut find: monitor_item = monitor_item {
        name: name as *mut ::core::ffi::c_char,
        format: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        type_0: MONITOR_SESSION,
        id: 0,
        flags: 0,
        last: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        panes: monitor_panes {
            rbh_root: ::core::ptr::null_mut::<monitor_pane>(),
        },
        windows: monitor_windows {
            rbh_root: ::core::ptr::null_mut::<monitor_window>(),
        },
        fire_count: 0,
        fire_time: 0,
        entry: monitor_item_entry {
            rbe_left: ::core::ptr::null_mut::<monitor_item>(),
            rbe_right: ::core::ptr::null_mut::<monitor_item>(),
            rbe_parent: ::core::ptr::null_mut::<monitor_item>(),
            rbe_color: 0,
        },
    };
    let mut tv: timeval = timeval {
        tv_sec: 1 as __time_t,
        tv_usec: 0,
    };
    me = monitor_items_RB_FIND(&raw mut (*ms).items, &raw mut find);
    if !me.is_null() {
        monitor_free_item(ms, me);
    }
    me = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<monitor_item>() as size_t,
    ) as *mut monitor_item;
    (*me).name = xstrdup(name);
    (*me).format = xstrdup(format);
    (*me).type_0 = type_0;
    (*me).id = id as u_int;
    (*me).flags = flags;
    (*me).panes.rbh_root = ::core::ptr::null_mut::<monitor_pane>();
    (*me).windows.rbh_root = ::core::ptr::null_mut::<monitor_window>();
    monitor_items_RB_INSERT(&raw mut (*ms).items, me);
    if event_initialized(&raw mut (*ms).timer) == 0 {
        event_set(
            &raw mut (*ms).timer,
            -(1 as ::core::ffi::c_int),
            0 as ::core::ffi::c_short,
            Some(
                monitor_timer
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        ::core::ffi::c_short,
                        *mut ::core::ffi::c_void,
                    ) -> (),
            ),
            ms as *mut ::core::ffi::c_void,
        );
    }
    if event_pending(
        &raw mut (*ms).timer,
        EV_TIMEOUT as ::core::ffi::c_short,
        ::core::ptr::null_mut::<timeval>(),
    ) == 0
    {
        event_add(&raw mut (*ms).timer, &raw mut tv);
    }
}
#[no_mangle]
pub unsafe extern "C" fn monitor_remove(
    mut ms: *mut monitor_set,
    mut name: *const ::core::ffi::c_char,
) {
    let mut me: *mut monitor_item = ::core::ptr::null_mut::<monitor_item>();
    let mut find: monitor_item = monitor_item {
        name: name as *mut ::core::ffi::c_char,
        format: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        type_0: MONITOR_SESSION,
        id: 0,
        flags: 0,
        last: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        panes: monitor_panes {
            rbh_root: ::core::ptr::null_mut::<monitor_pane>(),
        },
        windows: monitor_windows {
            rbh_root: ::core::ptr::null_mut::<monitor_window>(),
        },
        fire_count: 0,
        fire_time: 0,
        entry: monitor_item_entry {
            rbe_left: ::core::ptr::null_mut::<monitor_item>(),
            rbe_right: ::core::ptr::null_mut::<monitor_item>(),
            rbe_parent: ::core::ptr::null_mut::<monitor_item>(),
            rbe_color: 0,
        },
    };
    me = monitor_items_RB_FIND(&raw mut (*ms).items, &raw mut find);
    if !me.is_null() {
        monitor_free_item(ms, me);
    }
    if (*ms).items.rbh_root.is_null() && event_initialized(&raw mut (*ms).timer) != 0 {
        event_del(&raw mut (*ms).timer);
    }
}
#[no_mangle]
pub unsafe extern "C" fn monitor_get_fire_count(
    mut ms: *mut monitor_set,
    mut name: *const ::core::ffi::c_char,
) -> u_int {
    let mut me: *mut monitor_item = ::core::ptr::null_mut::<monitor_item>();
    let mut find: monitor_item = monitor_item {
        name: name as *mut ::core::ffi::c_char,
        format: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        type_0: MONITOR_SESSION,
        id: 0,
        flags: 0,
        last: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        panes: monitor_panes {
            rbh_root: ::core::ptr::null_mut::<monitor_pane>(),
        },
        windows: monitor_windows {
            rbh_root: ::core::ptr::null_mut::<monitor_window>(),
        },
        fire_count: 0,
        fire_time: 0,
        entry: monitor_item_entry {
            rbe_left: ::core::ptr::null_mut::<monitor_item>(),
            rbe_right: ::core::ptr::null_mut::<monitor_item>(),
            rbe_parent: ::core::ptr::null_mut::<monitor_item>(),
            rbe_color: 0,
        },
    };
    me = monitor_items_RB_FIND(&raw mut (*ms).items, &raw mut find);
    if me.is_null() {
        return 0 as u_int;
    }
    return (*me).fire_count;
}
#[no_mangle]
pub unsafe extern "C" fn monitor_get_fire_time(
    mut ms: *mut monitor_set,
    mut name: *const ::core::ffi::c_char,
) -> time_t {
    let mut me: *mut monitor_item = ::core::ptr::null_mut::<monitor_item>();
    let mut find: monitor_item = monitor_item {
        name: name as *mut ::core::ffi::c_char,
        format: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        type_0: MONITOR_SESSION,
        id: 0,
        flags: 0,
        last: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        panes: monitor_panes {
            rbh_root: ::core::ptr::null_mut::<monitor_pane>(),
        },
        windows: monitor_windows {
            rbh_root: ::core::ptr::null_mut::<monitor_window>(),
        },
        fire_count: 0,
        fire_time: 0,
        entry: monitor_item_entry {
            rbe_left: ::core::ptr::null_mut::<monitor_item>(),
            rbe_right: ::core::ptr::null_mut::<monitor_item>(),
            rbe_parent: ::core::ptr::null_mut::<monitor_item>(),
            rbe_color: 0,
        },
    };
    me = monitor_items_RB_FIND(&raw mut (*ms).items, &raw mut find);
    if me.is_null() {
        return 0 as time_t;
    }
    return (*me).fire_time;
}
