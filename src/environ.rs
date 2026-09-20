use crate::src::ffi::libc::{
    environ, fnmatch, free, getpid, setenv, strchr, strcmp, strcspn, vasprintf,
};
use crate::src::log::log_debug;
use crate::src::options::{
    options_array_first, options_array_item_value, options_array_next, options_get,
    options_get_string,
};
use crate::src::tmux::{getversion, global_environ, global_options, socket_path};
use crate::src::xmalloc::{xcalloc, xmalloc, xstrdup, xvasprintf};
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
pub use crate::src::shared::options::{
    options, options_array, options_array_item, options_entry, options_value,
};
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
pub use crate::src::shared::environment::{environ, environ_entry, environ_entry_entry};
pub use crate::src::shared::environment::{ENVIRON_HIDDEN};
pub use crate::src::shared::variadic::{__builtin_va_list, __va_list_tag, va_list};
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

pub use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_12;
pub use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_13;

unsafe extern "C" fn environ_RB_INSERT(
    mut head: *mut environ,
    mut elm: *mut environ_entry,
) -> *mut environ_entry {
    let mut tmp: *mut environ_entry = ::core::ptr::null_mut::<environ_entry>();
    let mut parent: *mut environ_entry = ::core::ptr::null_mut::<environ_entry>();
    let mut comp: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    tmp = (*head).rbh_root;
    while !tmp.is_null() {
        parent = tmp;
        comp = environ_cmp(elm, parent);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_right;
        } else {
            return tmp;
        }
    }
    (*elm).entry.rbe_parent = parent;
    (*elm).entry.rbe_right = ::core::ptr::null_mut::<environ_entry>();
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
    environ_RB_INSERT_COLOR(head, elm);
    return ::core::ptr::null_mut::<environ_entry>();
}
unsafe extern "C" fn environ_RB_REMOVE(
    mut head: *mut environ,
    mut elm: *mut environ_entry,
) -> *mut environ_entry {
    let mut current_block: u64;
    let mut child: *mut environ_entry = ::core::ptr::null_mut::<environ_entry>();
    let mut parent: *mut environ_entry = ::core::ptr::null_mut::<environ_entry>();
    let mut old: *mut environ_entry = elm;
    let mut color: ::core::ffi::c_int = 0;
    if (*elm).entry.rbe_left.is_null() {
        child = (*elm).entry.rbe_right;
        current_block = 7245201122033322888;
    } else if (*elm).entry.rbe_right.is_null() {
        child = (*elm).entry.rbe_left;
        current_block = 7245201122033322888;
    } else {
        let mut left: *mut environ_entry = ::core::ptr::null_mut::<environ_entry>();
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
        current_block = 17095038238854506463;
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
        environ_RB_REMOVE_COLOR(head, parent, child);
    }
    return old;
}
unsafe extern "C" fn environ_RB_REMOVE_COLOR(
    mut head: *mut environ,
    mut parent: *mut environ_entry,
    mut elm: *mut environ_entry,
) {
    let mut tmp: *mut environ_entry = ::core::ptr::null_mut::<environ_entry>();
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
                    let mut oleft: *mut environ_entry = ::core::ptr::null_mut::<environ_entry>();
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
                    let mut oright: *mut environ_entry = ::core::ptr::null_mut::<environ_entry>();
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
unsafe extern "C" fn environ_RB_NEXT(mut elm: *mut environ_entry) -> *mut environ_entry {
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
unsafe extern "C" fn environ_RB_MINMAX(
    mut head: *mut environ,
    mut val: ::core::ffi::c_int,
) -> *mut environ_entry {
    let mut tmp: *mut environ_entry = (*head).rbh_root;
    let mut parent: *mut environ_entry = ::core::ptr::null_mut::<environ_entry>();
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
unsafe extern "C" fn environ_RB_FIND(
    mut head: *mut environ,
    mut elm: *mut environ_entry,
) -> *mut environ_entry {
    let mut tmp: *mut environ_entry = (*head).rbh_root;
    let mut comp: ::core::ffi::c_int = 0;
    while !tmp.is_null() {
        comp = environ_cmp(elm, tmp);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_right;
        } else {
            return tmp;
        }
    }
    return ::core::ptr::null_mut::<environ_entry>();
}
unsafe extern "C" fn environ_RB_INSERT_COLOR(mut head: *mut environ, mut elm: *mut environ_entry) {
    let mut parent: *mut environ_entry = ::core::ptr::null_mut::<environ_entry>();
    let mut gparent: *mut environ_entry = ::core::ptr::null_mut::<environ_entry>();
    let mut tmp: *mut environ_entry = ::core::ptr::null_mut::<environ_entry>();
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
unsafe extern "C" fn environ_cmp(
    mut envent1: *mut environ_entry,
    mut envent2: *mut environ_entry,
) -> ::core::ffi::c_int {
    return strcmp((*envent1).name, (*envent2).name);
}
#[no_mangle]
pub unsafe extern "C" fn environ_create() -> *mut environ {
    let mut env: *mut environ = ::core::ptr::null_mut::<environ>();
    env = xcalloc(1 as size_t, ::core::mem::size_of::<environ>() as size_t) as *mut environ;
    (*env).rbh_root = ::core::ptr::null_mut::<environ_entry>();
    return env;
}
#[no_mangle]
pub unsafe extern "C" fn environ_free(mut env: *mut environ) {
    let mut envent: *mut environ_entry = ::core::ptr::null_mut::<environ_entry>();
    let mut envent1: *mut environ_entry = ::core::ptr::null_mut::<environ_entry>();
    if env.is_null() {
        return;
    }
    envent = environ_RB_MINMAX(env, RB_NEGINF);
    while !envent.is_null() && {
        envent1 = environ_RB_NEXT(envent);
        1 as ::core::ffi::c_int != 0
    } {
        environ_RB_REMOVE(env, envent);
        free((*envent).name as *mut ::core::ffi::c_void);
        free((*envent).value as *mut ::core::ffi::c_void);
        free(envent as *mut ::core::ffi::c_void);
        envent = envent1;
    }
    free(env as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn environ_first(mut env: *mut environ) -> *mut environ_entry {
    return environ_RB_MINMAX(env, RB_NEGINF);
}
#[no_mangle]
pub unsafe extern "C" fn environ_next(mut envent: *mut environ_entry) -> *mut environ_entry {
    return environ_RB_NEXT(envent);
}
#[no_mangle]
pub unsafe extern "C" fn environ_copy(mut srcenv: *mut environ, mut dstenv: *mut environ) {
    let mut envent: *mut environ_entry = ::core::ptr::null_mut::<environ_entry>();
    envent = environ_RB_MINMAX(srcenv, RB_NEGINF);
    while !envent.is_null() {
        if (*envent).value.is_null() {
            environ_clear(dstenv, (*envent).name);
        } else {
            environ_set(
                dstenv,
                (*envent).name,
                (*envent).flags,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                (*envent).value,
            );
        }
        envent = environ_RB_NEXT(envent);
    }
}
#[no_mangle]
pub unsafe extern "C" fn environ_find(
    mut env: *mut environ,
    mut name: *const ::core::ffi::c_char,
) -> *mut environ_entry {
    let mut envent: environ_entry = environ_entry {
        name: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        value: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        flags: 0,
        entry: environ_entry_entry {
            rbe_left: ::core::ptr::null_mut::<environ_entry>(),
            rbe_right: ::core::ptr::null_mut::<environ_entry>(),
            rbe_parent: ::core::ptr::null_mut::<environ_entry>(),
            rbe_color: 0,
        },
    };
    envent.name = name as *mut ::core::ffi::c_char;
    return environ_RB_FIND(env, &raw mut envent);
}
#[no_mangle]
pub unsafe extern "C" fn environ_set(
    mut env: *mut environ,
    mut name: *const ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
    mut fmt: *const ::core::ffi::c_char,
    mut args: ...
) {
    let mut envent: *mut environ_entry = ::core::ptr::null_mut::<environ_entry>();
    let mut ap: ::core::ffi::VaList;
    ap = args.clone();
    envent = environ_find(env, name);
    if !envent.is_null() {
        (*envent).flags = flags;
        free((*envent).value as *mut ::core::ffi::c_void);
        xvasprintf(&raw mut (*envent).value, fmt, ap);
    } else {
        envent = xmalloc(::core::mem::size_of::<environ_entry>() as size_t) as *mut environ_entry;
        (*envent).name = xstrdup(name);
        (*envent).flags = flags;
        xvasprintf(&raw mut (*envent).value, fmt, ap);
        environ_RB_INSERT(env, envent);
    };
}
#[no_mangle]
pub unsafe extern "C" fn environ_clear(
    mut env: *mut environ,
    mut name: *const ::core::ffi::c_char,
) {
    let mut envent: *mut environ_entry = ::core::ptr::null_mut::<environ_entry>();
    envent = environ_find(env, name);
    if !envent.is_null() {
        free((*envent).value as *mut ::core::ffi::c_void);
        (*envent).value = ::core::ptr::null_mut::<::core::ffi::c_char>();
    } else {
        envent = xmalloc(::core::mem::size_of::<environ_entry>() as size_t) as *mut environ_entry;
        (*envent).name = xstrdup(name);
        (*envent).flags = 0 as ::core::ffi::c_int;
        (*envent).value = ::core::ptr::null_mut::<::core::ffi::c_char>();
        environ_RB_INSERT(env, envent);
    };
}
#[no_mangle]
pub unsafe extern "C" fn environ_put(
    mut env: *mut environ,
    mut var: *const ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
) {
    let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut value: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    value = strchr(var, '=' as i32);
    if value.is_null() {
        return;
    }
    value = value.offset(1);
    name = xstrdup(var);
    *name.offset(strcspn(name, b"=\0" as *const u8 as *const ::core::ffi::c_char) as isize) =
        '\0' as i32 as ::core::ffi::c_char;
    environ_set(
        env,
        name,
        flags,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        value,
    );
    free(name as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn environ_unset(
    mut env: *mut environ,
    mut name: *const ::core::ffi::c_char,
) {
    let mut envent: *mut environ_entry = ::core::ptr::null_mut::<environ_entry>();
    envent = environ_find(env, name);
    if envent.is_null() {
        return;
    }
    environ_RB_REMOVE(env, envent);
    free((*envent).name as *mut ::core::ffi::c_void);
    free((*envent).value as *mut ::core::ffi::c_void);
    free(envent as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn environ_update(
    mut oo: *mut options,
    mut src: *mut environ,
    mut dst: *mut environ,
) {
    let mut envent: *mut environ_entry = ::core::ptr::null_mut::<environ_entry>();
    let mut envent1: *mut environ_entry = ::core::ptr::null_mut::<environ_entry>();
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut a: *mut options_array_item = ::core::ptr::null_mut::<options_array_item>();
    let mut ov: *mut options_value = ::core::ptr::null_mut::<options_value>();
    let mut found: ::core::ffi::c_int = 0;
    o = options_get(
        oo,
        b"update-environment\0" as *const u8 as *const ::core::ffi::c_char,
    );
    if o.is_null() {
        return;
    }
    a = options_array_first(o);
    while !a.is_null() {
        ov = options_array_item_value(a);
        found = 0 as ::core::ffi::c_int;
        envent = environ_RB_MINMAX(src, RB_NEGINF);
        while !envent.is_null() && {
            envent1 = environ_RB_NEXT(envent);
            1 as ::core::ffi::c_int != 0
        } {
            if fnmatch((*ov).string, (*envent).name, 0 as ::core::ffi::c_int)
                == 0 as ::core::ffi::c_int
            {
                environ_set(
                    dst,
                    (*envent).name,
                    0 as ::core::ffi::c_int,
                    b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                    (*envent).value,
                );
                found = 1 as ::core::ffi::c_int;
            }
            envent = envent1;
        }
        if found == 0 {
            environ_clear(dst, (*ov).string);
        }
        a = options_array_next(a);
    }
}
#[no_mangle]
pub unsafe extern "C" fn environ_push(mut env: *mut environ) {
    let mut envent: *mut environ_entry = ::core::ptr::null_mut::<environ_entry>();
    let mut new_environ: *mut *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    environ = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<*mut ::core::ffi::c_char>() as size_t,
    ) as *mut *mut ::core::ffi::c_char;
    new_environ = environ;
    envent = environ_RB_MINMAX(env, RB_NEGINF);
    while !envent.is_null() {
        if !(*envent).value.is_null()
            && *(*envent).name as ::core::ffi::c_int != '\0' as i32
            && !(*envent).flags & ENVIRON_HIDDEN != 0
        {
            setenv((*envent).name, (*envent).value, 1 as ::core::ffi::c_int);
        }
        envent = environ_RB_NEXT(envent);
    }
    if environ != new_environ {
        free(new_environ as *mut ::core::ffi::c_void);
    }
}
#[no_mangle]
pub unsafe extern "C" fn environ_log(
    mut env: *mut environ,
    mut fmt: *const ::core::ffi::c_char,
    mut args: ...
) {
    let mut envent: *mut environ_entry = ::core::ptr::null_mut::<environ_entry>();
    let mut ap: ::core::ffi::VaList;
    let mut prefix: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    ap = args.clone();
    vasprintf(&raw mut prefix, fmt, ap);
    envent = environ_RB_MINMAX(env, RB_NEGINF);
    while !envent.is_null() {
        if !(*envent).value.is_null() && *(*envent).name as ::core::ffi::c_int != '\0' as i32 {
            log_debug(
                b"%s%s=%s\0" as *const u8 as *const ::core::ffi::c_char,
                prefix,
                (*envent).name,
                (*envent).value,
            );
        }
        envent = environ_RB_NEXT(envent);
    }
    free(prefix as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn environ_for_session(
    mut s: *mut session,
    mut no_TERM: ::core::ffi::c_int,
) -> *mut environ {
    let mut env: *mut environ = ::core::ptr::null_mut::<environ>();
    let mut value: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut idx: ::core::ffi::c_int = 0;
    env = environ_create();
    environ_copy(global_environ, env);
    if !s.is_null() {
        environ_copy((*s).environ, env);
    }
    if no_TERM == 0 {
        value = options_get_string(
            global_options,
            b"default-terminal\0" as *const u8 as *const ::core::ffi::c_char,
        );
        environ_set(
            env,
            b"TERM\0" as *const u8 as *const ::core::ffi::c_char,
            0 as ::core::ffi::c_int,
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            value,
        );
        environ_set(
            env,
            b"TERM_PROGRAM\0" as *const u8 as *const ::core::ffi::c_char,
            0 as ::core::ffi::c_int,
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            b"tmux\0" as *const u8 as *const ::core::ffi::c_char,
        );
        environ_set(
            env,
            b"TERM_PROGRAM_VERSION\0" as *const u8 as *const ::core::ffi::c_char,
            0 as ::core::ffi::c_int,
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            getversion(),
        );
        environ_set(
            env,
            b"COLORTERM\0" as *const u8 as *const ::core::ffi::c_char,
            0 as ::core::ffi::c_int,
            b"truecolor\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    environ_clear(
        env,
        b"LISTEN_PID\0" as *const u8 as *const ::core::ffi::c_char,
    );
    environ_clear(
        env,
        b"LISTEN_FDS\0" as *const u8 as *const ::core::ffi::c_char,
    );
    environ_clear(
        env,
        b"LISTEN_FDNAMES\0" as *const u8 as *const ::core::ffi::c_char,
    );
    if !s.is_null() {
        idx = (*s).id as ::core::ffi::c_int;
    } else {
        idx = -(1 as ::core::ffi::c_int);
    }
    environ_set(
        env,
        b"TMUX\0" as *const u8 as *const ::core::ffi::c_char,
        0 as ::core::ffi::c_int,
        b"%s,%ld,%d\0" as *const u8 as *const ::core::ffi::c_char,
        socket_path,
        getpid() as ::core::ffi::c_long,
        idx,
    );
    return env;
}
