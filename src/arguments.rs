use crate::src::cmd::{
    cmd_append_argv, cmd_get_args, cmd_get_entry, cmd_get_source, cmd_list_copy, cmd_list_first,
    cmd_list_free, cmd_list_print, cmd_log_argv, cmd_template_replace,
};
use crate::src::cmd_find::cmd_find_copy_state;
use crate::src::cmd_parse::cmd_parse_from_string;
use crate::src::cmd_queue::{cmdq_error, cmdq_get_target, cmdq_get_target_client};
use crate::src::compat::strtonum::strtonum;
use crate::src::ffi::libc::{__ctype_b_loc, free, strchr, strcspn, strlcat};
use crate::src::format::format_single_from_target;
use crate::src::log::{fatalx, log_debug};
use crate::src::server_client::server_client_unref;
use crate::src::shared::abi::*;
pub use crate::src::shared::arguments::args_command_state;
use crate::src::shared::arguments::*;
pub use crate::src::shared::arguments::{
    args, args_entry, args_entry_entry, args_parse, args_parse_cb, args_tree, args_value,
    args_value_c2rust_unnamed, args_value_entry, args_values,
};
use crate::src::shared::client::*;
pub use crate::src::shared::client::{
    client, client_entry, client_file, client_file_cb, client_file_entry, client_files,
    overlay_check_cb, overlay_draw_cb, overlay_free_cb, overlay_key_cb, overlay_mode_cb,
    overlay_resize_cb,
};
use crate::src::shared::colour::*;
use crate::src::shared::command::*;
use crate::src::shared::command::*;
pub use crate::src::shared::command::{
    cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmd_list, cmdq_item, cmdq_list, cmds,
};
pub use crate::src::shared::command::{cmd_parse_input, cmd_parse_result};
pub use crate::src::shared::control::control_state;
pub use crate::src::shared::ctype::{
    _ISalnum, _ISalpha, _ISblank, _IScntrl, _ISdigit, _ISgraph, _ISlower, _ISprint, _ISpunct,
    _ISspace, _ISupper, _ISxdigit, ctype_code,
};
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
pub use crate::src::shared::session::{session, session_entry, session_gentry};
pub use crate::src::shared::spawn::spawn_editor_state;
pub use crate::src::shared::status::status_line;
use crate::src::shared::style::*;
use crate::src::shared::terminal::*;
pub use crate::src::shared::tree::{RB_BLACK, RB_NEGINF, RB_RED};
pub use crate::src::shared::tty::{tty, tty_code, tty_key, tty_term, tty_term_entry};
pub use crate::src::shared::variadic::{__builtin_va_list, __va_list_tag, va_list};
pub use crate::src::shared::vis::{VIS_CSTYLE, VIS_DQ, VIS_NL, VIS_OCTAL, VIS_TAB};
pub use crate::src::shared::window::{
    window, window_alerts_entry, window_entry, window_mode, window_mode_entry,
    window_mode_entry_entry, window_winlinks, winlink, winlink_entry, winlink_sentry,
    winlink_stack, winlink_wentry, winlinks,
};
use crate::src::utf8::utf8_stravis;
use crate::src::xmalloc::{xasprintf, xcalloc, xrealloc, xrecallocarray, xstrdup, xvasprintf};
use std::ffi::{CStr, CString};

pub use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_21;
pub use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_20;

pub const ARGS_ENTRY_OPTIONAL_VALUE: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ArgumentValueError {
    Missing,
    Empty,
    Invalid,
    TooSmall,
    TooLarge,
}

impl ArgumentValueError {
    pub fn message(self) -> &'static CStr {
        unsafe {
            CStr::from_bytes_with_nul_unchecked(match self {
                Self::Missing => b"missing\0",
                Self::Empty => b"empty\0",
                Self::Invalid => b"invalid\0",
                Self::TooSmall => b"too small\0",
                Self::TooLarge => b"too large\0",
            })
        }
    }
}
unsafe extern "C" fn args_tree_RB_INSERT(
    mut head: *mut args_tree,
    mut elm: *mut args_entry,
) -> *mut args_entry {
    let mut tmp: *mut args_entry = ::core::ptr::null_mut::<args_entry>();
    let mut parent: *mut args_entry = ::core::ptr::null_mut::<args_entry>();
    let mut comp: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    tmp = (*head).rbh_root;
    while !tmp.is_null() {
        parent = tmp;
        comp = args_cmp(elm, parent);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_right;
        } else {
            return tmp;
        }
    }
    (*elm).entry.rbe_parent = parent;
    (*elm).entry.rbe_right = ::core::ptr::null_mut::<args_entry>();
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
    args_tree_RB_INSERT_COLOR(head, elm);
    return ::core::ptr::null_mut::<args_entry>();
}
unsafe extern "C" fn args_tree_RB_FIND(
    mut head: *mut args_tree,
    mut elm: *mut args_entry,
) -> *mut args_entry {
    let mut tmp: *mut args_entry = (*head).rbh_root;
    let mut comp: ::core::ffi::c_int = 0;
    while !tmp.is_null() {
        comp = args_cmp(elm, tmp);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_right;
        } else {
            return tmp;
        }
    }
    return ::core::ptr::null_mut::<args_entry>();
}
unsafe extern "C" fn args_tree_RB_INSERT_COLOR(mut head: *mut args_tree, mut elm: *mut args_entry) {
    let mut parent: *mut args_entry = ::core::ptr::null_mut::<args_entry>();
    let mut gparent: *mut args_entry = ::core::ptr::null_mut::<args_entry>();
    let mut tmp: *mut args_entry = ::core::ptr::null_mut::<args_entry>();
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
unsafe extern "C" fn args_tree_RB_NEXT(mut elm: *mut args_entry) -> *mut args_entry {
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
unsafe extern "C" fn args_tree_RB_REMOVE(
    mut head: *mut args_tree,
    mut elm: *mut args_entry,
) -> *mut args_entry {
    let mut current_block: u64;
    let mut child: *mut args_entry = ::core::ptr::null_mut::<args_entry>();
    let mut parent: *mut args_entry = ::core::ptr::null_mut::<args_entry>();
    let mut old: *mut args_entry = elm;
    let mut color: ::core::ffi::c_int = 0;
    if (*elm).entry.rbe_left.is_null() {
        child = (*elm).entry.rbe_right;
        current_block = 7245201122033322888;
    } else if (*elm).entry.rbe_right.is_null() {
        child = (*elm).entry.rbe_left;
        current_block = 7245201122033322888;
    } else {
        let mut left: *mut args_entry = ::core::ptr::null_mut::<args_entry>();
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
        current_block = 10138408787860760292;
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
        args_tree_RB_REMOVE_COLOR(head, parent, child);
    }
    return old;
}
unsafe extern "C" fn args_tree_RB_REMOVE_COLOR(
    mut head: *mut args_tree,
    mut parent: *mut args_entry,
    mut elm: *mut args_entry,
) {
    let mut tmp: *mut args_entry = ::core::ptr::null_mut::<args_entry>();
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
                    let mut oleft: *mut args_entry = ::core::ptr::null_mut::<args_entry>();
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
                    let mut oright: *mut args_entry = ::core::ptr::null_mut::<args_entry>();
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
unsafe extern "C" fn args_tree_RB_MINMAX(
    mut head: *mut args_tree,
    mut val: ::core::ffi::c_int,
) -> *mut args_entry {
    let mut tmp: *mut args_entry = (*head).rbh_root;
    let mut parent: *mut args_entry = ::core::ptr::null_mut::<args_entry>();
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
unsafe extern "C" fn args_cmp(
    mut a1: *mut args_entry,
    mut a2: *mut args_entry,
) -> ::core::ffi::c_int {
    return (*a1).flag as ::core::ffi::c_int - (*a2).flag as ::core::ffi::c_int;
}
unsafe extern "C" fn args_find(mut args: *mut args, mut flag: u_char) -> *mut args_entry {
    let mut entry: args_entry = args_entry {
        flag: 0,
        values: args_values {
            tqh_first: ::core::ptr::null_mut::<args_value>(),
            tqh_last: ::core::ptr::null_mut::<*mut args_value>(),
        },
        count: 0,
        flags: 0,
        entry: args_entry_entry {
            rbe_left: ::core::ptr::null_mut::<args_entry>(),
            rbe_right: ::core::ptr::null_mut::<args_entry>(),
            rbe_parent: ::core::ptr::null_mut::<args_entry>(),
            rbe_color: 0,
        },
    };
    entry.flag = flag;
    return args_tree_RB_FIND(&raw mut (*args).tree, &raw mut entry);
}

unsafe fn args_last_value(args: *mut args, flag: u_char) -> Option<*mut args_value> {
    let entry = args_find(args, flag);
    if entry.is_null() {
        return None;
    }
    let mut value = (*entry).values.tqh_first;
    let mut last = None;
    while !value.is_null() {
        last = Some(value);
        value = (*value).entry.tqe_next;
    }
    last
}

unsafe fn args_last_string(args: *mut args, flag: u_char) -> Option<*const ::core::ffi::c_char> {
    let value = args_last_value(args, flag)?;
    if (*value).type_0 as ::core::ffi::c_uint
        != ARGS_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*value).c2rust_unnamed.string.is_null()
    {
        return None;
    }
    Some((*value).c2rust_unnamed.string)
}
unsafe extern "C" fn args_copy_value(mut to: *mut args_value, mut from: *mut args_value) {
    (*to).type_0 = (*from).type_0;
    match (*from).type_0 as ::core::ffi::c_uint {
        2 => {
            (*to).c2rust_unnamed.cmdlist = (*from).c2rust_unnamed.cmdlist;
            (*(*to).c2rust_unnamed.cmdlist).references += 1;
        }
        1 => {
            (*to).c2rust_unnamed.string = xstrdup((*from).c2rust_unnamed.string);
        }
        0 | _ => {}
    };
}
unsafe extern "C" fn args_type_to_string(mut type_0: args_type) -> *const ::core::ffi::c_char {
    match type_0 as ::core::ffi::c_uint {
        0 => return b"NONE\0" as *const u8 as *const ::core::ffi::c_char,
        1 => return b"STRING\0" as *const u8 as *const ::core::ffi::c_char,
        2 => return b"COMMANDS\0" as *const u8 as *const ::core::ffi::c_char,
        _ => {}
    }
    return b"INVALID\0" as *const u8 as *const ::core::ffi::c_char;
}
unsafe extern "C" fn args_value_as_string(
    mut value: *mut args_value,
) -> *const ::core::ffi::c_char {
    match (*value).type_0 as ::core::ffi::c_uint {
        0 => return b"\0" as *const u8 as *const ::core::ffi::c_char,
        2 => {
            if (*value).cached.is_null() {
                (*value).cached =
                    cmd_list_print((*value).c2rust_unnamed.cmdlist, 0 as ::core::ffi::c_int);
            }
            return (*value).cached;
        }
        1 => return (*value).c2rust_unnamed.string,
        _ => {}
    }
    fatalx(b"unexpected argument type\0" as *const u8 as *const ::core::ffi::c_char);
}
#[no_mangle]
pub unsafe extern "C" fn args_create() -> *mut args {
    let mut args: *mut args = ::core::ptr::null_mut::<args>();
    args = xcalloc(1 as size_t, ::core::mem::size_of::<args>() as size_t) as *mut args;
    (*args).tree.rbh_root = ::core::ptr::null_mut::<args_entry>();
    return args;
}
unsafe extern "C" fn args_parse_flag_argument(
    mut values: *mut args_value,
    mut count: u_int,
    mut cause: *mut *mut ::core::ffi::c_char,
    mut args: *mut args,
    mut i: *mut u_int,
    mut string: *const ::core::ffi::c_char,
    mut flag: ::core::ffi::c_int,
    mut optional_argument: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut argument: *mut args_value = ::core::ptr::null_mut::<args_value>();
    let mut new: *mut args_value = ::core::ptr::null_mut::<args_value>();
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut as_0: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    new = xcalloc(1 as size_t, ::core::mem::size_of::<args_value>() as size_t) as *mut args_value;
    if *string as ::core::ffi::c_int != '\0' as i32 {
        (*new).type_0 = ARGS_STRING;
        (*new).c2rust_unnamed.string = xstrdup(string);
    } else {
        if *i == count {
            argument = ::core::ptr::null_mut::<args_value>();
        } else {
            argument = values.offset(*i as isize) as *mut args_value;
            if (*argument).type_0 as ::core::ffi::c_uint
                != ARGS_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                xasprintf(
                    cause,
                    b"-%c argument must be a string\0" as *const u8 as *const ::core::ffi::c_char,
                    flag,
                );
                args_free_value(new);
                free(new as *mut ::core::ffi::c_void);
                return -(1 as ::core::ffi::c_int);
            }
        }
        if argument.is_null() {
            args_free_value(new);
            free(new as *mut ::core::ffi::c_void);
            if optional_argument != 0 {
                log_debug(
                    b"%s: -%c (optional)\0" as *const u8 as *const ::core::ffi::c_char,
                    b"args_parse_flag_argument\0" as *const u8 as *const ::core::ffi::c_char,
                    flag,
                );
                args_set(
                    args,
                    flag as u_char,
                    ::core::ptr::null_mut::<args_value>(),
                    ARGS_ENTRY_OPTIONAL_VALUE,
                );
                return 0 as ::core::ffi::c_int;
            }
            xasprintf(
                cause,
                b"-%c expects an argument\0" as *const u8 as *const ::core::ffi::c_char,
                flag,
            );
            return -(1 as ::core::ffi::c_int);
        }
        if optional_argument != 0
            && (*argument).type_0 as ::core::ffi::c_uint
                == ARGS_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            as_0 = (*argument).c2rust_unnamed.string;
            if *as_0.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '-' as i32
                && (*as_0.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '-' as i32
                    || *(*__ctype_b_loc())
                        .offset(*as_0.offset(1 as ::core::ffi::c_int as isize) as u_char
                            as ::core::ffi::c_int as isize)
                        as ::core::ffi::c_int
                        & _ISalpha as ::core::ffi::c_int as ::core::ffi::c_ushort
                            as ::core::ffi::c_int
                        != 0)
            {
                args_free_value(new);
                free(new as *mut ::core::ffi::c_void);
                log_debug(
                    b"%s: -%c (optional)\0" as *const u8 as *const ::core::ffi::c_char,
                    b"args_parse_flag_argument\0" as *const u8 as *const ::core::ffi::c_char,
                    flag,
                );
                args_set(
                    args,
                    flag as u_char,
                    ::core::ptr::null_mut::<args_value>(),
                    ARGS_ENTRY_OPTIONAL_VALUE,
                );
                return 0 as ::core::ffi::c_int;
            }
        }
        args_copy_value(new, argument);
        *i = (*i).wrapping_add(1);
    }
    s = args_value_as_string(new);
    log_debug(
        b"%s: -%c = %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"args_parse_flag_argument\0" as *const u8 as *const ::core::ffi::c_char,
        flag,
        s,
    );
    args_set(args, flag as u_char, new, 0 as ::core::ffi::c_int);
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn args_parse_flags(
    mut parse: *const args_parse,
    mut values: *mut args_value,
    mut count: u_int,
    mut cause: *mut *mut ::core::ffi::c_char,
    mut args: *mut args,
    mut i: *mut u_int,
) -> ::core::ffi::c_int {
    let mut value: *mut args_value = ::core::ptr::null_mut::<args_value>();
    let mut flag: u_char = 0;
    let mut found: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut string: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut optional_argument: ::core::ffi::c_int = 0;
    value = values.offset(*i as isize) as *mut args_value;
    if (*value).type_0 as ::core::ffi::c_uint
        != ARGS_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return 1 as ::core::ffi::c_int;
    }
    string = (*value).c2rust_unnamed.string;
    log_debug(
        b"%s: next %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"args_parse_flags\0" as *const u8 as *const ::core::ffi::c_char,
        string,
    );
    let fresh1 = string;
    string = string.offset(1);
    if *fresh1 as ::core::ffi::c_int != '-' as i32 || *string as ::core::ffi::c_int == '\0' as i32 {
        return 1 as ::core::ffi::c_int;
    }
    *i = (*i).wrapping_add(1);
    if *string.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '-' as i32
        && *string.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '\0' as i32
    {
        return 1 as ::core::ffi::c_int;
    }
    loop {
        let fresh2 = string;
        string = string.offset(1);
        flag = *fresh2 as u_char;
        if flag as ::core::ffi::c_int == '\0' as i32 {
            return 0 as ::core::ffi::c_int;
        }
        if flag as ::core::ffi::c_int == '?' as i32 {
            return -(1 as ::core::ffi::c_int);
        }
        if *(*__ctype_b_loc()).offset(flag as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            & _ISalnum as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
            == 0
        {
            xasprintf(
                cause,
                b"invalid flag -%c\0" as *const u8 as *const ::core::ffi::c_char,
                flag as ::core::ffi::c_int,
            );
            return -(1 as ::core::ffi::c_int);
        }
        found = strchr((*parse).template, flag as ::core::ffi::c_int);
        if found.is_null() {
            xasprintf(
                cause,
                b"unknown flag -%c\0" as *const u8 as *const ::core::ffi::c_char,
                flag as ::core::ffi::c_int,
            );
            return -(1 as ::core::ffi::c_int);
        }
        if *found.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != ':' as i32 {
            log_debug(
                b"%s: -%c\0" as *const u8 as *const ::core::ffi::c_char,
                b"args_parse_flags\0" as *const u8 as *const ::core::ffi::c_char,
                flag as ::core::ffi::c_int,
            );
            args_set(
                args,
                flag,
                ::core::ptr::null_mut::<args_value>(),
                0 as ::core::ffi::c_int,
            );
        } else {
            optional_argument = (*found.offset(2 as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int
                == ':' as i32) as ::core::ffi::c_int;
            return args_parse_flag_argument(
                values,
                count,
                cause,
                args,
                i,
                string,
                flag as ::core::ffi::c_int,
                optional_argument,
            );
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn args_parse(
    mut parse: *const args_parse,
    mut values: *mut args_value,
    mut count: u_int,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> *mut args {
    let mut args: *mut args = ::core::ptr::null_mut::<args>();
    let mut i: u_int = 0;
    let mut type_0: args_parse_type = ARGS_PARSE_INVALID;
    let mut value: *mut args_value = ::core::ptr::null_mut::<args_value>();
    let mut new: *mut args_value = ::core::ptr::null_mut::<args_value>();
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut stop: ::core::ffi::c_int = 0;
    if count == 0 as u_int {
        return args_create();
    }
    args = args_create();
    i = 1 as u_int;
    while i < count {
        stop = args_parse_flags(parse, values, count, cause, args, &raw mut i);
        if stop == -(1 as ::core::ffi::c_int) {
            args_free(args);
            return ::core::ptr::null_mut::<args>();
        }
        if stop == 1 as ::core::ffi::c_int {
            break;
        }
    }
    log_debug(
        b"%s: flags end at %u of %u\0" as *const u8 as *const ::core::ffi::c_char,
        b"args_parse\0" as *const u8 as *const ::core::ffi::c_char,
        i,
        count,
    );
    if i != count {
        while i < count {
            value = values.offset(i as isize) as *mut args_value;
            s = args_value_as_string(value);
            log_debug(
                b"%s: %u = %s (type %s)\0" as *const u8 as *const ::core::ffi::c_char,
                b"args_parse\0" as *const u8 as *const ::core::ffi::c_char,
                i,
                s,
                args_type_to_string((*value).type_0),
            );
            if (*parse).cb.is_some() {
                type_0 =
                    (*parse).cb.expect("non-null function pointer")(args, (*args).count, cause);
                if type_0 as ::core::ffi::c_uint
                    == ARGS_PARSE_INVALID as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    args_free(args);
                    return ::core::ptr::null_mut::<args>();
                }
            } else {
                type_0 = ARGS_PARSE_STRING;
            }
            (*args).values = xrecallocarray(
                (*args).values as *mut ::core::ffi::c_void,
                (*args).count as size_t,
                (*args).count.wrapping_add(1 as u_int) as size_t,
                ::core::mem::size_of::<args_value>() as size_t,
            ) as *mut args_value;
            let fresh0 = (*args).count;
            (*args).count = (*args).count.wrapping_add(1);
            new = (*args).values.offset(fresh0 as isize) as *mut args_value;
            match type_0 as ::core::ffi::c_uint {
                0 => {
                    fatalx(
                        b"unexpected argument type\0" as *const u8 as *const ::core::ffi::c_char,
                    );
                }
                1 => {
                    if (*value).type_0 as ::core::ffi::c_uint
                        != ARGS_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        xasprintf(
                            cause,
                            b"argument %u must be \"string\"\0" as *const u8
                                as *const ::core::ffi::c_char,
                            (*args).count,
                        );
                        args_free(args);
                        return ::core::ptr::null_mut::<args>();
                    }
                    args_copy_value(new, value);
                }
                2 => {
                    args_copy_value(new, value);
                }
                3 => {
                    if (*value).type_0 as ::core::ffi::c_uint
                        != ARGS_COMMANDS as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        xasprintf(
                            cause,
                            b"argument %u must be { commands }\0" as *const u8
                                as *const ::core::ffi::c_char,
                            (*args).count,
                        );
                        args_free(args);
                        return ::core::ptr::null_mut::<args>();
                    }
                    args_copy_value(new, value);
                }
                _ => {}
            }
            i = i.wrapping_add(1);
        }
    }
    if (*parse).lower != -(1 as ::core::ffi::c_int) && (*args).count < (*parse).lower as u_int {
        xasprintf(
            cause,
            b"too few arguments (need at least %u)\0" as *const u8 as *const ::core::ffi::c_char,
            (*parse).lower,
        );
        args_free(args);
        return ::core::ptr::null_mut::<args>();
    }
    if (*parse).upper != -(1 as ::core::ffi::c_int) && (*args).count > (*parse).upper as u_int {
        xasprintf(
            cause,
            b"too many arguments (need at most %u)\0" as *const u8 as *const ::core::ffi::c_char,
            (*parse).upper,
        );
        args_free(args);
        return ::core::ptr::null_mut::<args>();
    }
    return args;
}
unsafe extern "C" fn args_copy_copy_value(
    mut to: *mut args_value,
    mut from: *mut args_value,
    mut argc: ::core::ffi::c_int,
    mut argv: *mut *mut ::core::ffi::c_char,
) {
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut expanded: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut i: ::core::ffi::c_int = 0;
    (*to).type_0 = (*from).type_0;
    match (*from).type_0 as ::core::ffi::c_uint {
        1 => {
            expanded = xstrdup((*from).c2rust_unnamed.string);
            i = 0 as ::core::ffi::c_int;
            while i < argc {
                s = cmd_template_replace(
                    expanded,
                    *argv.offset(i as isize),
                    i + 1 as ::core::ffi::c_int,
                );
                free(expanded as *mut ::core::ffi::c_void);
                expanded = s;
                i += 1;
            }
            (*to).c2rust_unnamed.string = expanded;
        }
        2 => {
            (*to).c2rust_unnamed.cmdlist =
                cmd_list_copy((*from).c2rust_unnamed.cmdlist, argc, argv) as *mut cmd_list;
        }
        0 | _ => {}
    };
}
#[no_mangle]
pub unsafe extern "C" fn args_copy(
    mut args: *mut args,
    mut argc: ::core::ffi::c_int,
    mut argv: *mut *mut ::core::ffi::c_char,
) -> *mut args {
    let mut new_args: *mut args = ::core::ptr::null_mut::<args>();
    let mut entry: *mut args_entry = ::core::ptr::null_mut::<args_entry>();
    let mut value: *mut args_value = ::core::ptr::null_mut::<args_value>();
    let mut new_value: *mut args_value = ::core::ptr::null_mut::<args_value>();
    let mut i: u_int = 0;
    cmd_log_argv(
        argc,
        argv,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        b"args_copy\0" as *const u8 as *const ::core::ffi::c_char,
    );
    new_args = args_create();
    entry = args_tree_RB_MINMAX(&raw mut (*args).tree, RB_NEGINF);
    while !entry.is_null() {
        if (*entry).values.tqh_first.is_null() {
            i = 0 as u_int;
            while i < (*entry).count {
                args_set(
                    new_args,
                    (*entry).flag,
                    ::core::ptr::null_mut::<args_value>(),
                    0 as ::core::ffi::c_int,
                );
                i = i.wrapping_add(1);
            }
        } else {
            value = (*entry).values.tqh_first;
            while !value.is_null() {
                new_value = xcalloc(1 as size_t, ::core::mem::size_of::<args_value>() as size_t)
                    as *mut args_value;
                args_copy_copy_value(new_value, value, argc, argv);
                args_set(new_args, (*entry).flag, new_value, 0 as ::core::ffi::c_int);
                value = (*value).entry.tqe_next;
            }
        }
        entry = args_tree_RB_NEXT(entry);
    }
    if (*args).count == 0 as u_int {
        return new_args;
    }
    (*new_args).count = (*args).count;
    (*new_args).values = xcalloc(
        (*args).count as size_t,
        ::core::mem::size_of::<args_value>() as size_t,
    ) as *mut args_value;
    i = 0 as u_int;
    while i < (*args).count {
        new_value = (*new_args).values.offset(i as isize) as *mut args_value;
        args_copy_copy_value(
            new_value,
            (*args).values.offset(i as isize) as *mut args_value,
            argc,
            argv,
        );
        i = i.wrapping_add(1);
    }
    return new_args;
}
#[no_mangle]
pub unsafe extern "C" fn args_free_value(mut value: *mut args_value) {
    match (*value).type_0 as ::core::ffi::c_uint {
        1 => {
            free((*value).c2rust_unnamed.string as *mut ::core::ffi::c_void);
        }
        2 => {
            cmd_list_free((*value).c2rust_unnamed.cmdlist as *mut cmd_list);
        }
        0 | _ => {}
    }
    free((*value).cached as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn args_free_values(mut values: *mut args_value, mut count: u_int) {
    let mut i: u_int = 0;
    i = 0 as u_int;
    while i < count {
        args_free_value(values.offset(i as isize) as *mut args_value);
        i = i.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn args_free(mut args: *mut args) {
    let mut entry: *mut args_entry = ::core::ptr::null_mut::<args_entry>();
    let mut entry1: *mut args_entry = ::core::ptr::null_mut::<args_entry>();
    let mut value: *mut args_value = ::core::ptr::null_mut::<args_value>();
    let mut value1: *mut args_value = ::core::ptr::null_mut::<args_value>();
    args_free_values((*args).values, (*args).count);
    free((*args).values as *mut ::core::ffi::c_void);
    entry = args_tree_RB_MINMAX(&raw mut (*args).tree, RB_NEGINF);
    while !entry.is_null() && {
        entry1 = args_tree_RB_NEXT(entry);
        1 as ::core::ffi::c_int != 0
    } {
        args_tree_RB_REMOVE(&raw mut (*args).tree, entry);
        value = (*entry).values.tqh_first;
        while !value.is_null() && {
            value1 = (*value).entry.tqe_next;
            1 as ::core::ffi::c_int != 0
        } {
            if !(*value).entry.tqe_next.is_null() {
                (*(*value).entry.tqe_next).entry.tqe_prev = (*value).entry.tqe_prev;
            } else {
                (*entry).values.tqh_last = (*value).entry.tqe_prev;
            }
            *(*value).entry.tqe_prev = (*value).entry.tqe_next;
            args_free_value(value);
            free(value as *mut ::core::ffi::c_void);
            value = value1;
        }
        free(entry as *mut ::core::ffi::c_void);
        entry = entry1;
    }
    free(args as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn args_to_vector(
    mut args: *mut args,
    mut argc: *mut ::core::ffi::c_int,
    mut argv: *mut *mut *mut ::core::ffi::c_char,
) {
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut i: u_int = 0;
    *argc = 0 as ::core::ffi::c_int;
    *argv = ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    i = 0 as u_int;
    while i < (*args).count {
        match (*(*args).values.offset(i as isize)).type_0 as ::core::ffi::c_uint {
            1 => {
                cmd_append_argv(
                    argc,
                    argv,
                    (*(*args).values.offset(i as isize)).c2rust_unnamed.string,
                );
            }
            2 => {
                s = cmd_list_print(
                    (*(*args).values.offset(i as isize)).c2rust_unnamed.cmdlist,
                    0 as ::core::ffi::c_int,
                );
                cmd_append_argv(argc, argv, s);
                free(s as *mut ::core::ffi::c_void);
            }
            0 | _ => {}
        }
        i = i.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn args_from_vector(
    mut argc: ::core::ffi::c_int,
    mut argv: *mut *mut ::core::ffi::c_char,
) -> *mut args_value {
    let mut values: *mut args_value = ::core::ptr::null_mut::<args_value>();
    let mut i: ::core::ffi::c_int = 0;
    values = xcalloc(
        argc as size_t,
        ::core::mem::size_of::<args_value>() as size_t,
    ) as *mut args_value;
    i = 0 as ::core::ffi::c_int;
    while i < argc {
        (*values.offset(i as isize)).type_0 = ARGS_STRING;
        let ref mut fresh3 = (*values.offset(i as isize)).c2rust_unnamed.string;
        *fresh3 = xstrdup(*argv.offset(i as isize));
        i += 1;
    }
    return values;
}
unsafe extern "C" fn args_print_add(
    mut buf: *mut *mut ::core::ffi::c_char,
    mut len: *mut size_t,
    mut fmt: *const ::core::ffi::c_char,
    mut args: ...
) {
    let mut ap: ::core::ffi::VaList;
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut slen: size_t = 0;
    ap = args.clone();
    slen = xvasprintf(&raw mut s, fmt, ap) as size_t;
    *len = (*len).wrapping_add(slen);
    *buf = xrealloc(*buf as *mut ::core::ffi::c_void, *len) as *mut ::core::ffi::c_char;
    strlcat(*buf, s, *len);
    free(s as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn args_print_add_value(
    mut buf: *mut *mut ::core::ffi::c_char,
    mut len: *mut size_t,
    mut value: *mut args_value,
) {
    let mut expanded: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if **buf as ::core::ffi::c_int != '\0' as i32 {
        args_print_add(buf, len, b" \0" as *const u8 as *const ::core::ffi::c_char);
    }
    match (*value).type_0 as ::core::ffi::c_uint {
        2 => {
            expanded = cmd_list_print((*value).c2rust_unnamed.cmdlist, 0 as ::core::ffi::c_int);
            args_print_add(
                buf,
                len,
                b"{ %s }\0" as *const u8 as *const ::core::ffi::c_char,
                expanded,
            );
        }
        1 => {
            expanded = args_escape((*value).c2rust_unnamed.string);
            args_print_add(
                buf,
                len,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                expanded,
            );
        }
        0 | _ => {}
    }
    free(expanded as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn args_print(mut args: *mut args) -> *mut ::core::ffi::c_char {
    let mut len: size_t = 0;
    let mut buf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut i: u_int = 0;
    let mut j: u_int = 0;
    let mut entry: *mut args_entry = ::core::ptr::null_mut::<args_entry>();
    let mut last: *mut args_entry = ::core::ptr::null_mut::<args_entry>();
    let mut value: *mut args_value = ::core::ptr::null_mut::<args_value>();
    len = 1 as size_t;
    buf = xcalloc(1 as size_t, len) as *mut ::core::ffi::c_char;
    entry = args_tree_RB_MINMAX(&raw mut (*args).tree, RB_NEGINF);
    while !entry.is_null() {
        if !((*entry).flags & ARGS_ENTRY_OPTIONAL_VALUE != 0) {
            if (*entry).values.tqh_first.is_null() {
                if *buf as ::core::ffi::c_int == '\0' as i32 {
                    args_print_add(
                        &raw mut buf,
                        &raw mut len,
                        b"-\0" as *const u8 as *const ::core::ffi::c_char,
                    );
                }
                j = 0 as u_int;
                while j < (*entry).count {
                    args_print_add(
                        &raw mut buf,
                        &raw mut len,
                        b"%c\0" as *const u8 as *const ::core::ffi::c_char,
                        (*entry).flag as ::core::ffi::c_int,
                    );
                    j = j.wrapping_add(1);
                }
            }
        }
        entry = args_tree_RB_NEXT(entry);
    }
    entry = args_tree_RB_MINMAX(&raw mut (*args).tree, RB_NEGINF);
    while !entry.is_null() {
        if (*entry).flags & ARGS_ENTRY_OPTIONAL_VALUE != 0 {
            if *buf as ::core::ffi::c_int != '\0' as i32 {
                args_print_add(
                    &raw mut buf,
                    &raw mut len,
                    b" -%c\0" as *const u8 as *const ::core::ffi::c_char,
                    (*entry).flag as ::core::ffi::c_int,
                );
            } else {
                args_print_add(
                    &raw mut buf,
                    &raw mut len,
                    b"-%c\0" as *const u8 as *const ::core::ffi::c_char,
                    (*entry).flag as ::core::ffi::c_int,
                );
            }
            last = entry;
        } else if !(*entry).values.tqh_first.is_null() {
            value = (*entry).values.tqh_first;
            while !value.is_null() {
                if *buf as ::core::ffi::c_int != '\0' as i32 {
                    args_print_add(
                        &raw mut buf,
                        &raw mut len,
                        b" -%c\0" as *const u8 as *const ::core::ffi::c_char,
                        (*entry).flag as ::core::ffi::c_int,
                    );
                } else {
                    args_print_add(
                        &raw mut buf,
                        &raw mut len,
                        b"-%c\0" as *const u8 as *const ::core::ffi::c_char,
                        (*entry).flag as ::core::ffi::c_int,
                    );
                }
                args_print_add_value(&raw mut buf, &raw mut len, value);
                value = (*value).entry.tqe_next;
            }
            last = entry;
        }
        entry = args_tree_RB_NEXT(entry);
    }
    if !last.is_null() && (*last).flags & ARGS_ENTRY_OPTIONAL_VALUE != 0 {
        args_print_add(
            &raw mut buf,
            &raw mut len,
            b" --\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    i = 0 as u_int;
    while i < (*args).count {
        args_print_add_value(
            &raw mut buf,
            &raw mut len,
            (*args).values.offset(i as isize) as *mut args_value,
        );
        i = i.wrapping_add(1);
    }
    return buf;
}
#[no_mangle]
pub unsafe extern "C" fn args_escape(
    mut s: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    static mut dquoted: [::core::ffi::c_char; 9] =
        unsafe { ::core::mem::transmute::<[u8; 9], [::core::ffi::c_char; 9]>(*b" #';${}%\0") };
    static mut squoted: [::core::ffi::c_char; 3] =
        unsafe { ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b" \"\0") };
    let mut escaped: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut result: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut flags: ::core::ffi::c_int = 0;
    let mut quotes: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if *s as ::core::ffi::c_int == '\0' as i32 {
        xasprintf(
            &raw mut result,
            b"''\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return result;
    }
    if *s.offset(strcspn(s, &raw const dquoted as *const ::core::ffi::c_char) as isize)
        as ::core::ffi::c_int
        != '\0' as i32
    {
        quotes = '"' as i32;
    } else if *s.offset(strcspn(s, &raw const squoted as *const ::core::ffi::c_char) as isize)
        as ::core::ffi::c_int
        != '\0' as i32
    {
        quotes = '\'' as i32;
    }
    if *s.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != ' ' as i32
        && *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '\0' as i32
        && (quotes != 0 as ::core::ffi::c_int
            || *s.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '~' as i32)
    {
        xasprintf(
            &raw mut escaped,
            b"\\%c\0" as *const u8 as *const ::core::ffi::c_char,
            *s.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int,
        );
        return escaped;
    }
    flags = VIS_OCTAL | VIS_CSTYLE | VIS_TAB | VIS_NL;
    if quotes == '"' as i32 {
        flags |= VIS_DQ;
    }
    utf8_stravis(&raw mut escaped, s, flags);
    if quotes == '\'' as i32 {
        xasprintf(
            &raw mut result,
            b"'%s'\0" as *const u8 as *const ::core::ffi::c_char,
            escaped,
        );
    } else if quotes == '"' as i32 {
        if *escaped as ::core::ffi::c_int == '~' as i32 {
            xasprintf(
                &raw mut result,
                b"\"\\%s\"\0" as *const u8 as *const ::core::ffi::c_char,
                escaped,
            );
        } else {
            xasprintf(
                &raw mut result,
                b"\"%s\"\0" as *const u8 as *const ::core::ffi::c_char,
                escaped,
            );
        }
    } else if *escaped as ::core::ffi::c_int == '~' as i32 {
        xasprintf(
            &raw mut result,
            b"\\%s\0" as *const u8 as *const ::core::ffi::c_char,
            escaped,
        );
    } else {
        result = xstrdup(escaped);
    }
    free(escaped as *mut ::core::ffi::c_void);
    return result;
}
#[no_mangle]
pub unsafe extern "C" fn args_has(mut args: *mut args, mut flag: u_char) -> ::core::ffi::c_int {
    let mut entry: *mut args_entry = ::core::ptr::null_mut::<args_entry>();
    entry = args_find(args, flag);
    if entry.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    return (*entry).count as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn args_set(
    mut args: *mut args,
    mut flag: u_char,
    mut value: *mut args_value,
    mut flags: ::core::ffi::c_int,
) {
    let mut entry: *mut args_entry = ::core::ptr::null_mut::<args_entry>();
    entry = args_find(args, flag);
    if entry.is_null() {
        entry =
            xcalloc(1 as size_t, ::core::mem::size_of::<args_entry>() as size_t) as *mut args_entry;
        (*entry).flag = flag;
        (*entry).count = 1 as u_int;
        (*entry).flags = flags;
        (*entry).values.tqh_first = ::core::ptr::null_mut::<args_value>();
        (*entry).values.tqh_last = &raw mut (*entry).values.tqh_first;
        args_tree_RB_INSERT(&raw mut (*args).tree, entry);
    } else {
        (*entry).count = (*entry).count.wrapping_add(1);
    }
    if !value.is_null()
        && (*value).type_0 as ::core::ffi::c_uint
            != ARGS_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*value).entry.tqe_next = ::core::ptr::null_mut::<args_value>();
        (*value).entry.tqe_prev = (*entry).values.tqh_last;
        *(*entry).values.tqh_last = value;
        (*entry).values.tqh_last = &raw mut (*value).entry.tqe_next;
    } else {
        free(value as *mut ::core::ffi::c_void);
    };
}
#[no_mangle]
pub unsafe extern "C" fn args_get(
    mut args: *mut args,
    mut flag: u_char,
) -> *const ::core::ffi::c_char {
    let mut entry: *mut args_entry = ::core::ptr::null_mut::<args_entry>();
    entry = args_find(args, flag);
    if entry.is_null() {
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    if (*entry).values.tqh_first.is_null() {
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    return (**(*((*entry).values.tqh_last as *mut args_values)).tqh_last)
        .c2rust_unnamed
        .string;
}
#[no_mangle]
pub unsafe extern "C" fn args_first(
    mut args: *mut args,
    mut entry: *mut *mut args_entry,
) -> u_char {
    *entry = args_tree_RB_MINMAX(&raw mut (*args).tree, RB_NEGINF);
    if (*entry).is_null() {
        return 0 as u_char;
    }
    return (**entry).flag;
}
#[no_mangle]
pub unsafe extern "C" fn args_next(mut entry: *mut *mut args_entry) -> u_char {
    *entry = args_tree_RB_NEXT(*entry);
    if (*entry).is_null() {
        return 0 as u_char;
    }
    return (**entry).flag;
}
#[no_mangle]
pub unsafe extern "C" fn args_count(mut args: *mut args) -> u_int {
    return (*args).count;
}
#[no_mangle]
pub unsafe extern "C" fn args_values(mut args: *mut args) -> *mut args_value {
    return (*args).values;
}
#[no_mangle]
pub unsafe extern "C" fn args_value(mut args: *mut args, mut idx: u_int) -> *mut args_value {
    if idx >= (*args).count {
        return ::core::ptr::null_mut::<args_value>();
    }
    return (*args).values.offset(idx as isize) as *mut args_value;
}
#[no_mangle]
pub unsafe extern "C" fn args_string(
    mut args: *mut args,
    mut idx: u_int,
) -> *const ::core::ffi::c_char {
    if idx >= (*args).count {
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    return args_value_as_string((*args).values.offset(idx as isize) as *mut args_value);
}
#[no_mangle]
pub unsafe extern "C" fn args_make_commands_now(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
    mut idx: u_int,
    mut expand: ::core::ffi::c_int,
) -> *mut cmd_list {
    let mut state: *mut args_command_state = ::core::ptr::null_mut::<args_command_state>();
    let mut error: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut cmdlist: *mut cmd_list = ::core::ptr::null_mut::<cmd_list>();
    state = args_make_commands_prepare(
        self_0,
        item,
        idx,
        ::core::ptr::null::<::core::ffi::c_char>(),
        0 as ::core::ffi::c_int,
        expand,
    );
    cmdlist = args_make_commands(
        state,
        0 as ::core::ffi::c_int,
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        &raw mut error,
    );
    if cmdlist.is_null() {
        cmdq_error(
            item,
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            error,
        );
        free(error as *mut ::core::ffi::c_void);
    }
    args_make_commands_free(state);
    return cmdlist;
}
#[no_mangle]
pub unsafe extern "C" fn args_make_commands_prepare(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
    mut idx: u_int,
    mut default_command: *const ::core::ffi::c_char,
    mut wait: ::core::ffi::c_int,
    mut expand: ::core::ffi::c_int,
) -> *mut args_command_state {
    let mut args: *mut args = cmd_get_args(self_0);
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    let mut tc: *mut client = cmdq_get_target_client(item);
    let mut value: *mut args_value = ::core::ptr::null_mut::<args_value>();
    let mut state: *mut args_command_state = ::core::ptr::null_mut::<args_command_state>();
    let mut cmd: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut file: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    state = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<args_command_state>() as size_t,
    ) as *mut args_command_state;
    if idx < (*args).count {
        value = (*args).values.offset(idx as isize) as *mut args_value;
        if (*value).type_0 as ::core::ffi::c_uint
            == ARGS_COMMANDS as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            (*state).cmdlist = (*value).c2rust_unnamed.cmdlist as *mut cmd_list;
            (*(*state).cmdlist).references += 1;
            return state;
        }
        cmd = (*value).c2rust_unnamed.string;
    } else {
        if default_command.is_null() {
            fatalx(b"argument out of range\0" as *const u8 as *const ::core::ffi::c_char);
        }
        cmd = default_command;
    }
    if expand != 0 {
        (*state).cmd = format_single_from_target(item, cmd);
    } else {
        (*state).cmd = xstrdup(cmd);
    }
    log_debug(
        b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"args_make_commands_prepare\0" as *const u8 as *const ::core::ffi::c_char,
        (*state).cmd,
    );
    if wait != 0 {
        (*state).pi.item = item;
    }
    cmd_get_source(self_0, &raw mut file, &raw mut (*state).pi.line);
    if !file.is_null() {
        (*state).pi.file = xstrdup(file);
    }
    (*state).pi.c = tc;
    if !(*state).pi.c.is_null() {
        (*(*state).pi.c).references += 1;
    }
    cmd_find_copy_state(&raw mut (*state).pi.fs, target);
    return state;
}
#[no_mangle]
pub unsafe extern "C" fn args_make_commands(
    mut state: *mut args_command_state,
    mut argc: ::core::ffi::c_int,
    mut argv: *mut *mut ::core::ffi::c_char,
    mut error: *mut *mut ::core::ffi::c_char,
) -> *mut cmd_list {
    let mut pr: *mut cmd_parse_result = ::core::ptr::null_mut::<cmd_parse_result>();
    let mut cmd: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut new_cmd: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut i: ::core::ffi::c_int = 0;
    if !(*state).cmdlist.is_null() {
        if argc == 0 as ::core::ffi::c_int {
            (*(*state).cmdlist).references += 1;
            return (*state).cmdlist;
        }
        return cmd_list_copy((*state).cmdlist, argc, argv);
    }
    cmd = xstrdup((*state).cmd);
    log_debug(
        b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"args_make_commands\0" as *const u8 as *const ::core::ffi::c_char,
        cmd,
    );
    cmd_log_argv(
        argc,
        argv,
        b"args_make_commands\0" as *const u8 as *const ::core::ffi::c_char,
    );
    i = 0 as ::core::ffi::c_int;
    while i < argc {
        new_cmd = cmd_template_replace(cmd, *argv.offset(i as isize), i + 1 as ::core::ffi::c_int);
        log_debug(
            b"%s: %%%u %s: %s\0" as *const u8 as *const ::core::ffi::c_char,
            b"args_make_commands\0" as *const u8 as *const ::core::ffi::c_char,
            i + 1 as ::core::ffi::c_int,
            *argv.offset(i as isize),
            new_cmd,
        );
        free(cmd as *mut ::core::ffi::c_void);
        cmd = new_cmd;
        i += 1;
    }
    log_debug(
        b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"args_make_commands\0" as *const u8 as *const ::core::ffi::c_char,
        cmd,
    );
    pr = cmd_parse_from_string(cmd, &raw mut (*state).pi);
    free(cmd as *mut ::core::ffi::c_void);
    match (*pr).status as ::core::ffi::c_uint {
        0 => {
            *error = (*pr).error;
            return ::core::ptr::null_mut::<cmd_list>();
        }
        1 => return (*pr).cmdlist,
        _ => {}
    }
    fatalx(b"invalid parse return state\0" as *const u8 as *const ::core::ffi::c_char);
}
#[no_mangle]
pub unsafe extern "C" fn args_make_commands_free(mut state: *mut args_command_state) {
    if !(*state).cmdlist.is_null() {
        cmd_list_free((*state).cmdlist);
    }
    if !(*state).pi.c.is_null() {
        server_client_unref((*state).pi.c);
    }
    free((*state).pi.file as *mut ::core::ffi::c_void);
    free((*state).cmd as *mut ::core::ffi::c_void);
    free(state as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn args_make_commands_get_command(
    mut state: *mut args_command_state,
) -> *mut ::core::ffi::c_char {
    let mut first: *mut cmd = ::core::ptr::null_mut::<cmd>();
    let mut n: ::core::ffi::c_int = 0;
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if !(*state).cmdlist.is_null() {
        first = cmd_list_first((*state).cmdlist);
        if first.is_null() {
            return xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
        }
        return xstrdup((*cmd_get_entry(first)).name);
    }
    n = strcspn(
        (*state).cmd,
        b" ,\0" as *const u8 as *const ::core::ffi::c_char,
    ) as ::core::ffi::c_int;
    xasprintf(
        &raw mut s,
        b"%.*s\0" as *const u8 as *const ::core::ffi::c_char,
        n,
        (*state).cmd,
    );
    return s;
}
#[no_mangle]
pub unsafe extern "C" fn args_first_value(
    mut args: *mut args,
    mut flag: u_char,
) -> *mut args_value {
    let mut entry: *mut args_entry = ::core::ptr::null_mut::<args_entry>();
    entry = args_find(args, flag);
    if entry.is_null() {
        return ::core::ptr::null_mut::<args_value>();
    }
    return (*entry).values.tqh_first;
}
#[no_mangle]
pub unsafe extern "C" fn args_next_value(mut value: *mut args_value) -> *mut args_value {
    return (*value).entry.tqe_next;
}

fn strtonum_error(errstr: *const ::core::ffi::c_char) -> ArgumentValueError {
    match unsafe { CStr::from_ptr(errstr).to_bytes() } {
        b"invalid" => ArgumentValueError::Invalid,
        b"too small" => ArgumentValueError::TooSmall,
        b"too large" => ArgumentValueError::TooLarge,
        _ => ArgumentValueError::Invalid,
    }
}

pub fn parse_number(value: &CStr, minval: i64, maxval: i64) -> Result<i64, ArgumentValueError> {
    let mut errstr = ::core::ptr::null::<::core::ffi::c_char>();
    let number = unsafe {
        strtonum(
            value.as_ptr(),
            minval as ::core::ffi::c_longlong,
            maxval as ::core::ffi::c_longlong,
            &raw mut errstr,
        )
    };
    if errstr.is_null() {
        Ok(number as i64)
    } else {
        Err(strtonum_error(errstr))
    }
}

fn percentage_share(
    curval: i64,
    percentage: i64,
    minval: i64,
    maxval: i64,
) -> Result<i64, ArgumentValueError> {
    let value = (curval as i128 * percentage as i128) / 100;
    if value < minval as i128 {
        return Err(ArgumentValueError::TooSmall);
    }
    if value > maxval as i128 {
        return Err(ArgumentValueError::TooLarge);
    }
    Ok(value as i64)
}

pub fn parse_percentage(
    value: &CStr,
    minval: i64,
    maxval: i64,
    curval: i64,
) -> Result<i64, ArgumentValueError> {
    let bytes = value.to_bytes();
    if bytes.is_empty() {
        return Err(ArgumentValueError::Empty);
    }
    if let Some(percentage) = bytes.strip_suffix(b"%") {
        let percentage = CString::new(percentage).map_err(|_| ArgumentValueError::Invalid)?;
        let percentage = parse_number(percentage.as_c_str(), 0, 1000)?;
        return percentage_share(curval, percentage, minval, maxval);
    }
    parse_number(value, minval, maxval)
}

/// Converts a percentage after expanding format expressions against `item`.
///
/// # Safety
/// `item` must be a valid command-queue item whenever a format expression is
/// expanded.
pub unsafe fn parse_percentage_and_expand(
    value: &CStr,
    minval: i64,
    maxval: i64,
    curval: i64,
    item: *mut cmdq_item,
) -> Result<i64, ArgumentValueError> {
    let bytes = value.to_bytes();
    if let Some(percentage) = bytes.strip_suffix(b"%") {
        let percentage = CString::new(percentage).map_err(|_| ArgumentValueError::Invalid)?;
        let formatted = format_single_from_target(item, percentage.as_ptr());
        let result = parse_number(CStr::from_ptr(formatted), 0, 1000)
            .and_then(|percentage| percentage_share(curval, percentage, minval, maxval));
        free(formatted as *mut ::core::ffi::c_void);
        return result;
    }
    let formatted = format_single_from_target(item, value.as_ptr());
    let result = parse_number(CStr::from_ptr(formatted), minval, maxval);
    free(formatted as *mut ::core::ffi::c_void);
    result
}

/// Converts the last string value stored for `flag` to a bounded integer.
///
/// # Safety
/// `args` must point to a valid argument store.
pub unsafe fn args_strtonum_result(
    args: *mut args,
    flag: u_char,
    minval: ::core::ffi::c_longlong,
    maxval: ::core::ffi::c_longlong,
) -> Result<i64, ArgumentValueError> {
    let value = args_last_string(args, flag).ok_or(ArgumentValueError::Missing)?;
    parse_number(CStr::from_ptr(value), minval, maxval)
}

/// Converts the last string value after format expansion to a bounded integer.
///
/// # Safety
/// `args` must point to a valid argument store and `item` must be valid for
/// format expansion.
pub unsafe fn args_strtonum_and_expand_result(
    args: *mut args,
    flag: u_char,
    minval: ::core::ffi::c_longlong,
    maxval: ::core::ffi::c_longlong,
    item: *mut cmdq_item,
) -> Result<i64, ArgumentValueError> {
    let value = args_last_string(args, flag).ok_or(ArgumentValueError::Missing)?;
    let value = CStr::from_ptr(value);
    let formatted = format_single_from_target(item, value.as_ptr());
    let result = parse_number(CStr::from_ptr(formatted), minval, maxval);
    free(formatted as *mut ::core::ffi::c_void);
    result
}

/// Converts the last stored string as an integer or percentage.
///
/// # Safety
/// `args` must point to a valid argument store.
pub unsafe fn args_percentage_result(
    args: *mut args,
    flag: u_char,
    minval: ::core::ffi::c_longlong,
    maxval: ::core::ffi::c_longlong,
    curval: ::core::ffi::c_longlong,
) -> Result<i64, ArgumentValueError> {
    let entry = args_find(args, flag);
    if entry.is_null() {
        return Err(ArgumentValueError::Missing);
    }
    let value = args_last_value(args, flag).ok_or(ArgumentValueError::Empty)?;
    if (*value).type_0 as ::core::ffi::c_uint
        != ARGS_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*value).c2rust_unnamed.string.is_null()
    {
        return Err(ArgumentValueError::Missing);
    }
    parse_percentage(
        CStr::from_ptr((*value).c2rust_unnamed.string),
        minval,
        maxval,
        curval,
    )
}

/// Converts the last stored string after format expansion as an integer or percentage.
///
/// # Safety
/// `args` must point to a valid argument store and `item` must be valid for
/// format expansion.
pub unsafe fn args_percentage_and_expand_result(
    args: *mut args,
    flag: u_char,
    minval: ::core::ffi::c_longlong,
    maxval: ::core::ffi::c_longlong,
    curval: ::core::ffi::c_longlong,
    item: *mut cmdq_item,
) -> Result<i64, ArgumentValueError> {
    let entry = args_find(args, flag);
    if entry.is_null() {
        return Err(ArgumentValueError::Missing);
    }
    let value = args_last_value(args, flag).ok_or(ArgumentValueError::Empty)?;
    if (*value).type_0 as ::core::ffi::c_uint
        != ARGS_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*value).c2rust_unnamed.string.is_null()
    {
        return Err(ArgumentValueError::Missing);
    }
    parse_percentage_and_expand(
        CStr::from_ptr((*value).c2rust_unnamed.string),
        minval,
        maxval,
        curval,
        item,
    )
}

/// Converts a C string as an integer or percentage.
///
/// # Safety
/// If non-null, `value` must point to a valid NUL-terminated C string.
pub unsafe fn args_string_percentage_result(
    value: *const ::core::ffi::c_char,
    minval: ::core::ffi::c_longlong,
    maxval: ::core::ffi::c_longlong,
    curval: ::core::ffi::c_longlong,
) -> Result<i64, ArgumentValueError> {
    if value.is_null() {
        return Err(ArgumentValueError::Missing);
    }
    parse_percentage(CStr::from_ptr(value), minval, maxval, curval)
}

/// Converts a C string after format expansion as an integer or percentage.
///
/// # Safety
/// If non-null, `value` must point to a valid NUL-terminated C string and
/// `item` must be valid for format expansion.
pub unsafe fn args_string_percentage_and_expand_result(
    value: *const ::core::ffi::c_char,
    minval: ::core::ffi::c_longlong,
    maxval: ::core::ffi::c_longlong,
    curval: ::core::ffi::c_longlong,
    item: *mut cmdq_item,
) -> Result<i64, ArgumentValueError> {
    if value.is_null() {
        return Err(ArgumentValueError::Missing);
    }
    parse_percentage_and_expand(CStr::from_ptr(value), minval, maxval, curval, item)
}

unsafe fn args_result_to_c(
    result: Result<i64, ArgumentValueError>,
    cause: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_longlong {
    match result {
        Ok(value) => {
            *cause = ::core::ptr::null_mut::<::core::ffi::c_char>();
            value as ::core::ffi::c_longlong
        }
        Err(error) => {
            *cause = xstrdup(error.message().as_ptr());
            0 as ::core::ffi::c_longlong
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn args_strtonum(
    mut args: *mut args,
    mut flag: u_char,
    mut minval: ::core::ffi::c_longlong,
    mut maxval: ::core::ffi::c_longlong,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_longlong {
    args_result_to_c(args_strtonum_result(args, flag, minval, maxval), cause)
}
#[no_mangle]
pub unsafe extern "C" fn args_strtonum_and_expand(
    mut args: *mut args,
    mut flag: u_char,
    mut minval: ::core::ffi::c_longlong,
    mut maxval: ::core::ffi::c_longlong,
    mut item: *mut cmdq_item,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_longlong {
    args_result_to_c(
        args_strtonum_and_expand_result(args, flag, minval, maxval, item),
        cause,
    )
}
#[no_mangle]
pub unsafe extern "C" fn args_percentage(
    mut args: *mut args,
    mut flag: u_char,
    mut minval: ::core::ffi::c_longlong,
    mut maxval: ::core::ffi::c_longlong,
    mut curval: ::core::ffi::c_longlong,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_longlong {
    args_result_to_c(
        args_percentage_result(args, flag, minval, maxval, curval),
        cause,
    )
}
#[no_mangle]
pub unsafe extern "C" fn args_string_percentage(
    mut value: *const ::core::ffi::c_char,
    mut minval: ::core::ffi::c_longlong,
    mut maxval: ::core::ffi::c_longlong,
    mut curval: ::core::ffi::c_longlong,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_longlong {
    args_result_to_c(
        args_string_percentage_result(value, minval, maxval, curval),
        cause,
    )
}
#[no_mangle]
pub unsafe extern "C" fn args_percentage_and_expand(
    mut args: *mut args,
    mut flag: u_char,
    mut minval: ::core::ffi::c_longlong,
    mut maxval: ::core::ffi::c_longlong,
    mut curval: ::core::ffi::c_longlong,
    mut item: *mut cmdq_item,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_longlong {
    args_result_to_c(
        args_percentage_and_expand_result(args, flag, minval, maxval, curval, item),
        cause,
    )
}
#[no_mangle]
pub unsafe extern "C" fn args_string_percentage_and_expand(
    mut value: *const ::core::ffi::c_char,
    mut minval: ::core::ffi::c_longlong,
    mut maxval: ::core::ffi::c_longlong,
    mut curval: ::core::ffi::c_longlong,
    mut item: *mut cmdq_item,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_longlong {
    args_result_to_c(
        args_string_percentage_and_expand_result(value, minval, maxval, curval, item),
        cause,
    )
}
