use crate::src::alerts::alerts_queue;
use crate::src::arguments::args_has;
use crate::src::cmd::{cmd_free_argv, cmd_mouse_at};
use crate::src::cmd_find::{cmd_find_from_pane, cmd_find_from_window};
use crate::src::cmd_queue::{cmdq_continue, cmdq_get_client};
use crate::src::colour::{
    colour_palette_free, colour_palette_from_option, colour_palette_get, colour_palette_init,
    colour_totheme,
};
use crate::src::compat::strtonum::strtonum;
use crate::src::control::control_write_output;
use crate::src::events::{events_fire, events_fire_pane, events_fire_window};
use crate::src::events_payload::{
    event_payload_create, event_payload_set_int, event_payload_set_pane,
    event_payload_set_string, event_payload_set_target, event_payload_set_uint,
    event_payload_set_window,
};
use crate::src::ffi::libc::{
    __ctype_b_loc, close, fnmatch, free, gethostname, getpid, gettimeofday, ioctl, kill, memcpy,
    memset, regcomp, regexec, regfree, strcasecmp, strlen,
};
use crate::src::reactor::{
    bufferevent_disable, bufferevent_enable, bufferevent_free, bufferevent_new,
    bufferevent_write, evbuffer_drain, evbuffer_get_length, evbuffer_pullup, event_add,
    event_del, event_initialized, event_set,
};
use crate::src::ffi::utempter::utempter_remove_record;
use crate::src::file::{file_cancel, file_read};
use crate::src::grid::grid_cells_look_equal;
use crate::src::grid_view::grid_view_string_cells;
use crate::src::input::{input_free, input_init, input_parse_buffer, input_parse_pane};
use crate::src::input_keys::input_key_pane;
use crate::src::layout::{
    layout_assign_pane, layout_fix_panes, layout_floating_pane, layout_free, layout_free_cell,
    layout_init,
};
use crate::src::log::{fatal, fatalx, log_debug};
use crate::src::menu::{menu_destroy, menu_resize};
use crate::src::options::{options_create, options_free, options_get_number};
use crate::src::prompt::{
    prompt_closed, prompt_create, prompt_free, prompt_incremental_start, prompt_key,
    prompt_mouse, prompt_set_options, prompt_type_string, prompt_update,
};
use crate::src::screen::{
    screen_free, screen_init, screen_resize, screen_set_default_cursor, screen_set_title,
};
use crate::src::screen_redraw::redraw_invalidate_scene;
use crate::src::screen_write::{screen_write_clear_dirty, screen_write_stop_sync};
use crate::src::server::{marked_pane, server_check_marked, server_clear_marked};
pub use crate::src::server::clients;
use crate::src::server_client::server_client_unref;
use crate::src::server_fn::{
    server_destroy_pane, server_kill_pane, server_redraw_window, server_redraw_window_borders,
    server_status_session, server_status_window,
};
use crate::src::session::session_has;
use crate::src::spawn::spawn_editor_finish;
use crate::src::status::status_at_line;
use crate::src::style::{
    style_ranges_free, style_ranges_get_range, style_ranges_init,
    style_set_scrollbar_style_from_option,
};
use crate::src::tmux::{clean_name, global_options, global_w_options, setblocking};
use crate::src::tty::{tty_default_colours, tty_update_window_offset};
use crate::src::window_copy::{window_copy_mode, window_view_mode};
use crate::src::xmalloc::{xasprintf, xcalloc, xmalloc, xreallocarray, xstrdup};
pub use crate::src::shared::prompt::{prompt_create_data};
pub use crate::src::shared::pane::{window_pane_tree, window_panes_zindex};
pub use crate::src::shared::events::{event_payload};

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
pub use crate::src::shared::status::{status_line, status_prompt_input_cb};
pub use crate::src::shared::tty::{tty, tty_code, tty_key, tty_term, tty_term_entry};
pub use crate::src::shared::window::{
    window, window_alerts_entry, window_entry, window_mode, window_mode_entry,
    window_mode_entry_entry, window_winlinks, windows, winlink, winlink_entry, winlink_sentry,
    winlink_stack, winlink_wentry, winlinks,
};
pub use crate::src::shared::environment::{environ};
pub use crate::src::shared::ctype::{
    _ISalnum, _ISalpha, _ISblank, _IScntrl, _ISdigit, _ISgraph, _ISlower, _ISprint, _ISpunct,
    _ISspace, _ISupper, _ISxdigit, ctype_code,
};
pub use crate::src::shared::posix_io::{FNM_CASEFOLD};
pub use crate::src::shared::limits::{__INT_MAX__, INT_MAX, UINT_MAX};
pub use crate::src::shared::posix_terminal::{winsize, TIOCSWINSZ};
pub use crate::src::shared::regex::{
    __re_long_size_t, re_dfa_t, re_pattern_buffer, reg_syntax_t, regex_t, regmatch_t, regoff_t,
    REG_EXTENDED, REG_ICASE,
};
pub use crate::src::shared::signal::SIGCHLD;
pub use crate::src::shared::spawn::{SPAWN_BEFORE, SPAWN_FLOATING, SPAWN_FULLSIZE};
pub use crate::src::shared::window::{
    WINDOW_ACTIVITY, WINDOW_ALERTFLAGS, WINDOW_BELL, WINDOW_MODE_HIDE_PANE_STATUS,
    WINDOW_MODE_HIDE_SCROLLBARS, WINDOW_MODE_NO_STACK, WINDOW_PANE_NO_MODE, WINDOW_SILENCE,
    WINDOW_ZOOMED, WINLINK_ACTIVITY, WINLINK_ALERTFLAGS, WINLINK_BELL, WINLINK_SILENCE,
    WINLINK_VISITED,
};
pub use crate::src::shared::pane::{
    PANE_CHANGED, PANE_DESTROYED, PANE_EMPTY, PANE_EXITED, PANE_FLOATOVERZOOM, PANE_FOCUSED,
    PANE_INPUTOFF, PANE_REDRAW, PANE_REDRAWSCROLLBAR, PANE_SCROLLBARS_ALWAYS,
    PANE_SCROLLBARS_AUTOHIDE, PANE_SCROLLBARS_LEFT, PANE_SCROLLBARS_MODAL, PANE_STATUSREADY,
    PANE_STATUS_BOTTOM, PANE_STATUS_BOTTOM_FLOATING, PANE_STATUS_OFF, PANE_STATUS_TOP,
    PANE_STATUS_TOP_FLOATING, PANE_STYLECHANGED, PANE_THEMECHANGED, PANE_UNSEENCHANGES,
    PANE_VISITED, PANE_ZOOMED, window_pane_offset, window_pane_resize, window_pane_resize_entry,
    window_pane_resizes,
};
pub use crate::src::shared::screen::{
    MODE_BRACKETPASTE, MODE_FOCUSON, MODE_THEME_UPDATES, screen, screen_sel, screen_titles,
};
pub use crate::src::shared::prompt::{
    PROMPT_CLOSE, PROMPT_CONTINUE, prompt_free_cb, prompt_input_cb, prompt_result,
};
pub use crate::src::shared::abi::{ssize_t};
pub use crate::src::shared::screen_write::{screen_write_cline};
pub use crate::src::shared::hyperlinks::{hyperlinks};
pub use crate::src::shared::tree::{RB_BLACK, RB_INF, RB_NEGINF, RB_RED};
pub use crate::src::shared::event::{EV_READ, EV_WRITE};
pub use crate::src::shared::client::{
    CLIENT_CONTROL, CLIENT_DEAD, CLIENT_EXIT, CLIENT_EXITED, CLIENT_FOCUSED, CLIENT_SUSPENDED,
    CLIENT_UNATTACHEDFLAGS,
};
pub use crate::src::shared::display::{visible_range, visible_ranges};
pub use crate::src::shared::layout::{layout_geometry};
pub use crate::src::shared::mouse::{
    MOUSE_BUTTON_1, MOUSE_MASK_BUTTONS, MOUSE_MASK_DRAG, mouse_event,
};
use crate::src::shared::client::*;
use crate::src::shared::layout::*;
use crate::src::shared::prompt::*;
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

pub use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_13;
pub use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_14;

pub use crate::src::shared::key::key_code_enum as C2RustUnnamed_36;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct window_pane_input_data {
    pub item: *mut cmdq_item,
    pub wp: u_int,
    pub file: *mut client_file,
}

pub const FIONREAD: ::core::ffi::c_int = 0x541b as ::core::ffi::c_int;

pub const DEFAULT_XPIXEL: ::core::ffi::c_int = 16 as ::core::ffi::c_int;
pub const DEFAULT_YPIXEL: ::core::ffi::c_int = 32 as ::core::ffi::c_int;
pub const WINDOW_PANE_COPY_MODE: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const WINDOW_PANE_VIEW_MODE: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const WINDOW_WASZOOMED: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
#[no_mangle]
pub static mut windows: windows = windows {
    rbh_root: ::core::ptr::null::<window>() as *mut window,
};
#[no_mangle]
pub static mut all_window_panes: window_pane_tree = window_pane_tree {
    rbh_root: ::core::ptr::null::<window_pane>() as *mut window_pane,
};
static mut next_window_pane_id: u_int = 0;
static mut next_window_id: u_int = 0;
static mut next_active_point: u_int = 0;
#[no_mangle]
pub unsafe extern "C" fn windows_RB_NEXT(mut elm: *mut window) -> *mut window {
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
#[no_mangle]
pub unsafe extern "C" fn windows_RB_NFIND(
    mut head: *mut windows,
    mut elm: *mut window,
) -> *mut window {
    let mut tmp: *mut window = (*head).rbh_root;
    let mut res: *mut window = ::core::ptr::null_mut::<window>();
    let mut comp: ::core::ffi::c_int = 0;
    while !tmp.is_null() {
        comp = window_cmp(elm, tmp);
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
pub unsafe extern "C" fn windows_RB_MINMAX(
    mut head: *mut windows,
    mut val: ::core::ffi::c_int,
) -> *mut window {
    let mut tmp: *mut window = (*head).rbh_root;
    let mut parent: *mut window = ::core::ptr::null_mut::<window>();
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
pub unsafe extern "C" fn windows_RB_INSERT(
    mut head: *mut windows,
    mut elm: *mut window,
) -> *mut window {
    let mut tmp: *mut window = ::core::ptr::null_mut::<window>();
    let mut parent: *mut window = ::core::ptr::null_mut::<window>();
    let mut comp: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    tmp = (*head).rbh_root;
    while !tmp.is_null() {
        parent = tmp;
        comp = window_cmp(elm, parent);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_right;
        } else {
            return tmp;
        }
    }
    (*elm).entry.rbe_parent = parent;
    (*elm).entry.rbe_right = ::core::ptr::null_mut::<window>();
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
    windows_RB_INSERT_COLOR(head, elm);
    return ::core::ptr::null_mut::<window>();
}
#[no_mangle]
pub unsafe extern "C" fn windows_RB_PREV(mut elm: *mut window) -> *mut window {
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
pub unsafe extern "C" fn windows_RB_FIND(
    mut head: *mut windows,
    mut elm: *mut window,
) -> *mut window {
    let mut tmp: *mut window = (*head).rbh_root;
    let mut comp: ::core::ffi::c_int = 0;
    while !tmp.is_null() {
        comp = window_cmp(elm, tmp);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_right;
        } else {
            return tmp;
        }
    }
    return ::core::ptr::null_mut::<window>();
}
#[no_mangle]
pub unsafe extern "C" fn windows_RB_REMOVE(
    mut head: *mut windows,
    mut elm: *mut window,
) -> *mut window {
    let mut current_block: u64;
    let mut child: *mut window = ::core::ptr::null_mut::<window>();
    let mut parent: *mut window = ::core::ptr::null_mut::<window>();
    let mut old: *mut window = elm;
    let mut color: ::core::ffi::c_int = 0;
    if (*elm).entry.rbe_left.is_null() {
        child = (*elm).entry.rbe_right;
        current_block = 7245201122033322888;
    } else if (*elm).entry.rbe_right.is_null() {
        child = (*elm).entry.rbe_left;
        current_block = 7245201122033322888;
    } else {
        let mut left: *mut window = ::core::ptr::null_mut::<window>();
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
        current_block = 11459827966151058445;
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
        windows_RB_REMOVE_COLOR(head, parent, child);
    }
    return old;
}
#[no_mangle]
pub unsafe extern "C" fn windows_RB_REMOVE_COLOR(
    mut head: *mut windows,
    mut parent: *mut window,
    mut elm: *mut window,
) {
    let mut tmp: *mut window = ::core::ptr::null_mut::<window>();
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
                    let mut oleft: *mut window = ::core::ptr::null_mut::<window>();
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
                    let mut oright: *mut window = ::core::ptr::null_mut::<window>();
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
pub unsafe extern "C" fn windows_RB_INSERT_COLOR(mut head: *mut windows, mut elm: *mut window) {
    let mut parent: *mut window = ::core::ptr::null_mut::<window>();
    let mut gparent: *mut window = ::core::ptr::null_mut::<window>();
    let mut tmp: *mut window = ::core::ptr::null_mut::<window>();
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
pub unsafe extern "C" fn winlinks_RB_INSERT_COLOR(mut head: *mut winlinks, mut elm: *mut winlink) {
    let mut parent: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut gparent: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut tmp: *mut winlink = ::core::ptr::null_mut::<winlink>();
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
pub unsafe extern "C" fn winlinks_RB_REMOVE_COLOR(
    mut head: *mut winlinks,
    mut parent: *mut winlink,
    mut elm: *mut winlink,
) {
    let mut tmp: *mut winlink = ::core::ptr::null_mut::<winlink>();
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
                    let mut oleft: *mut winlink = ::core::ptr::null_mut::<winlink>();
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
                    let mut oright: *mut winlink = ::core::ptr::null_mut::<winlink>();
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
pub unsafe extern "C" fn winlinks_RB_REMOVE(
    mut head: *mut winlinks,
    mut elm: *mut winlink,
) -> *mut winlink {
    let mut current_block: u64;
    let mut child: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut parent: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut old: *mut winlink = elm;
    let mut color: ::core::ffi::c_int = 0;
    if (*elm).entry.rbe_left.is_null() {
        child = (*elm).entry.rbe_right;
        current_block = 7245201122033322888;
    } else if (*elm).entry.rbe_right.is_null() {
        child = (*elm).entry.rbe_left;
        current_block = 7245201122033322888;
    } else {
        let mut left: *mut winlink = ::core::ptr::null_mut::<winlink>();
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
        current_block = 11350272964138407771;
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
        winlinks_RB_REMOVE_COLOR(head, parent, child);
    }
    return old;
}
#[no_mangle]
pub unsafe extern "C" fn winlinks_RB_INSERT(
    mut head: *mut winlinks,
    mut elm: *mut winlink,
) -> *mut winlink {
    let mut tmp: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut parent: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut comp: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    tmp = (*head).rbh_root;
    while !tmp.is_null() {
        parent = tmp;
        comp = winlink_cmp(elm, parent);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_right;
        } else {
            return tmp;
        }
    }
    (*elm).entry.rbe_parent = parent;
    (*elm).entry.rbe_right = ::core::ptr::null_mut::<winlink>();
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
    winlinks_RB_INSERT_COLOR(head, elm);
    return ::core::ptr::null_mut::<winlink>();
}
#[no_mangle]
pub unsafe extern "C" fn winlinks_RB_NFIND(
    mut head: *mut winlinks,
    mut elm: *mut winlink,
) -> *mut winlink {
    let mut tmp: *mut winlink = (*head).rbh_root;
    let mut res: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut comp: ::core::ffi::c_int = 0;
    while !tmp.is_null() {
        comp = winlink_cmp(elm, tmp);
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
pub unsafe extern "C" fn winlinks_RB_MINMAX(
    mut head: *mut winlinks,
    mut val: ::core::ffi::c_int,
) -> *mut winlink {
    let mut tmp: *mut winlink = (*head).rbh_root;
    let mut parent: *mut winlink = ::core::ptr::null_mut::<winlink>();
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
pub unsafe extern "C" fn winlinks_RB_PREV(mut elm: *mut winlink) -> *mut winlink {
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
pub unsafe extern "C" fn winlinks_RB_NEXT(mut elm: *mut winlink) -> *mut winlink {
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
#[no_mangle]
pub unsafe extern "C" fn winlinks_RB_FIND(
    mut head: *mut winlinks,
    mut elm: *mut winlink,
) -> *mut winlink {
    let mut tmp: *mut winlink = (*head).rbh_root;
    let mut comp: ::core::ffi::c_int = 0;
    while !tmp.is_null() {
        comp = winlink_cmp(elm, tmp);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_right;
        } else {
            return tmp;
        }
    }
    return ::core::ptr::null_mut::<winlink>();
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_tree_RB_FIND(
    mut head: *mut window_pane_tree,
    mut elm: *mut window_pane,
) -> *mut window_pane {
    let mut tmp: *mut window_pane = (*head).rbh_root;
    let mut comp: ::core::ffi::c_int = 0;
    while !tmp.is_null() {
        comp = window_pane_cmp(elm, tmp);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).tree_entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).tree_entry.rbe_right;
        } else {
            return tmp;
        }
    }
    return ::core::ptr::null_mut::<window_pane>();
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_tree_RB_NFIND(
    mut head: *mut window_pane_tree,
    mut elm: *mut window_pane,
) -> *mut window_pane {
    let mut tmp: *mut window_pane = (*head).rbh_root;
    let mut res: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut comp: ::core::ffi::c_int = 0;
    while !tmp.is_null() {
        comp = window_pane_cmp(elm, tmp);
        if comp < 0 as ::core::ffi::c_int {
            res = tmp;
            tmp = (*tmp).tree_entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).tree_entry.rbe_right;
        } else {
            return tmp;
        }
    }
    return res;
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_tree_RB_PREV(mut elm: *mut window_pane) -> *mut window_pane {
    if !(*elm).tree_entry.rbe_left.is_null() {
        elm = (*elm).tree_entry.rbe_left;
        while !(*elm).tree_entry.rbe_right.is_null() {
            elm = (*elm).tree_entry.rbe_right;
        }
    } else if !(*elm).tree_entry.rbe_parent.is_null()
        && elm == (*(*elm).tree_entry.rbe_parent).tree_entry.rbe_right
    {
        elm = (*elm).tree_entry.rbe_parent;
    } else {
        while !(*elm).tree_entry.rbe_parent.is_null()
            && elm == (*(*elm).tree_entry.rbe_parent).tree_entry.rbe_left
        {
            elm = (*elm).tree_entry.rbe_parent;
        }
        elm = (*elm).tree_entry.rbe_parent;
    }
    return elm;
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_tree_RB_MINMAX(
    mut head: *mut window_pane_tree,
    mut val: ::core::ffi::c_int,
) -> *mut window_pane {
    let mut tmp: *mut window_pane = (*head).rbh_root;
    let mut parent: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    while !tmp.is_null() {
        parent = tmp;
        if val < 0 as ::core::ffi::c_int {
            tmp = (*tmp).tree_entry.rbe_left;
        } else {
            tmp = (*tmp).tree_entry.rbe_right;
        }
    }
    return parent;
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_tree_RB_REMOVE_COLOR(
    mut head: *mut window_pane_tree,
    mut parent: *mut window_pane,
    mut elm: *mut window_pane,
) {
    let mut tmp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    while (elm.is_null() || (*elm).tree_entry.rbe_color == RB_BLACK) && elm != (*head).rbh_root {
        if (*parent).tree_entry.rbe_left == elm {
            tmp = (*parent).tree_entry.rbe_right;
            if (*tmp).tree_entry.rbe_color == RB_RED {
                (*tmp).tree_entry.rbe_color = RB_BLACK;
                (*parent).tree_entry.rbe_color = RB_RED;
                tmp = (*parent).tree_entry.rbe_right;
                (*parent).tree_entry.rbe_right = (*tmp).tree_entry.rbe_left;
                if !(*parent).tree_entry.rbe_right.is_null() {
                    (*(*tmp).tree_entry.rbe_left).tree_entry.rbe_parent = parent;
                }
                (*tmp).tree_entry.rbe_parent = (*parent).tree_entry.rbe_parent;
                if !(*tmp).tree_entry.rbe_parent.is_null() {
                    if parent == (*(*parent).tree_entry.rbe_parent).tree_entry.rbe_left {
                        (*(*parent).tree_entry.rbe_parent).tree_entry.rbe_left = tmp;
                    } else {
                        (*(*parent).tree_entry.rbe_parent).tree_entry.rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).tree_entry.rbe_left = parent;
                (*parent).tree_entry.rbe_parent = tmp;
                !(*tmp).tree_entry.rbe_parent.is_null();
                tmp = (*parent).tree_entry.rbe_right;
            }
            if ((*tmp).tree_entry.rbe_left.is_null()
                || (*(*tmp).tree_entry.rbe_left).tree_entry.rbe_color == RB_BLACK)
                && ((*tmp).tree_entry.rbe_right.is_null()
                    || (*(*tmp).tree_entry.rbe_right).tree_entry.rbe_color == RB_BLACK)
            {
                (*tmp).tree_entry.rbe_color = RB_RED;
                elm = parent;
                parent = (*elm).tree_entry.rbe_parent;
            } else {
                if (*tmp).tree_entry.rbe_right.is_null()
                    || (*(*tmp).tree_entry.rbe_right).tree_entry.rbe_color == RB_BLACK
                {
                    let mut oleft: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
                    oleft = (*tmp).tree_entry.rbe_left;
                    if !oleft.is_null() {
                        (*oleft).tree_entry.rbe_color = RB_BLACK;
                    }
                    (*tmp).tree_entry.rbe_color = RB_RED;
                    oleft = (*tmp).tree_entry.rbe_left;
                    (*tmp).tree_entry.rbe_left = (*oleft).tree_entry.rbe_right;
                    if !(*tmp).tree_entry.rbe_left.is_null() {
                        (*(*oleft).tree_entry.rbe_right).tree_entry.rbe_parent = tmp;
                    }
                    (*oleft).tree_entry.rbe_parent = (*tmp).tree_entry.rbe_parent;
                    if !(*oleft).tree_entry.rbe_parent.is_null() {
                        if tmp == (*(*tmp).tree_entry.rbe_parent).tree_entry.rbe_left {
                            (*(*tmp).tree_entry.rbe_parent).tree_entry.rbe_left = oleft;
                        } else {
                            (*(*tmp).tree_entry.rbe_parent).tree_entry.rbe_right = oleft;
                        }
                    } else {
                        (*head).rbh_root = oleft;
                    }
                    (*oleft).tree_entry.rbe_right = tmp;
                    (*tmp).tree_entry.rbe_parent = oleft;
                    !(*oleft).tree_entry.rbe_parent.is_null();
                    tmp = (*parent).tree_entry.rbe_right;
                }
                (*tmp).tree_entry.rbe_color = (*parent).tree_entry.rbe_color;
                (*parent).tree_entry.rbe_color = RB_BLACK;
                if !(*tmp).tree_entry.rbe_right.is_null() {
                    (*(*tmp).tree_entry.rbe_right).tree_entry.rbe_color = RB_BLACK;
                }
                tmp = (*parent).tree_entry.rbe_right;
                (*parent).tree_entry.rbe_right = (*tmp).tree_entry.rbe_left;
                if !(*parent).tree_entry.rbe_right.is_null() {
                    (*(*tmp).tree_entry.rbe_left).tree_entry.rbe_parent = parent;
                }
                (*tmp).tree_entry.rbe_parent = (*parent).tree_entry.rbe_parent;
                if !(*tmp).tree_entry.rbe_parent.is_null() {
                    if parent == (*(*parent).tree_entry.rbe_parent).tree_entry.rbe_left {
                        (*(*parent).tree_entry.rbe_parent).tree_entry.rbe_left = tmp;
                    } else {
                        (*(*parent).tree_entry.rbe_parent).tree_entry.rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).tree_entry.rbe_left = parent;
                (*parent).tree_entry.rbe_parent = tmp;
                !(*tmp).tree_entry.rbe_parent.is_null();
                elm = (*head).rbh_root;
                break;
            }
        } else {
            tmp = (*parent).tree_entry.rbe_left;
            if (*tmp).tree_entry.rbe_color == RB_RED {
                (*tmp).tree_entry.rbe_color = RB_BLACK;
                (*parent).tree_entry.rbe_color = RB_RED;
                tmp = (*parent).tree_entry.rbe_left;
                (*parent).tree_entry.rbe_left = (*tmp).tree_entry.rbe_right;
                if !(*parent).tree_entry.rbe_left.is_null() {
                    (*(*tmp).tree_entry.rbe_right).tree_entry.rbe_parent = parent;
                }
                (*tmp).tree_entry.rbe_parent = (*parent).tree_entry.rbe_parent;
                if !(*tmp).tree_entry.rbe_parent.is_null() {
                    if parent == (*(*parent).tree_entry.rbe_parent).tree_entry.rbe_left {
                        (*(*parent).tree_entry.rbe_parent).tree_entry.rbe_left = tmp;
                    } else {
                        (*(*parent).tree_entry.rbe_parent).tree_entry.rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).tree_entry.rbe_right = parent;
                (*parent).tree_entry.rbe_parent = tmp;
                !(*tmp).tree_entry.rbe_parent.is_null();
                tmp = (*parent).tree_entry.rbe_left;
            }
            if ((*tmp).tree_entry.rbe_left.is_null()
                || (*(*tmp).tree_entry.rbe_left).tree_entry.rbe_color == RB_BLACK)
                && ((*tmp).tree_entry.rbe_right.is_null()
                    || (*(*tmp).tree_entry.rbe_right).tree_entry.rbe_color == RB_BLACK)
            {
                (*tmp).tree_entry.rbe_color = RB_RED;
                elm = parent;
                parent = (*elm).tree_entry.rbe_parent;
            } else {
                if (*tmp).tree_entry.rbe_left.is_null()
                    || (*(*tmp).tree_entry.rbe_left).tree_entry.rbe_color == RB_BLACK
                {
                    let mut oright: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
                    oright = (*tmp).tree_entry.rbe_right;
                    if !oright.is_null() {
                        (*oright).tree_entry.rbe_color = RB_BLACK;
                    }
                    (*tmp).tree_entry.rbe_color = RB_RED;
                    oright = (*tmp).tree_entry.rbe_right;
                    (*tmp).tree_entry.rbe_right = (*oright).tree_entry.rbe_left;
                    if !(*tmp).tree_entry.rbe_right.is_null() {
                        (*(*oright).tree_entry.rbe_left).tree_entry.rbe_parent = tmp;
                    }
                    (*oright).tree_entry.rbe_parent = (*tmp).tree_entry.rbe_parent;
                    if !(*oright).tree_entry.rbe_parent.is_null() {
                        if tmp == (*(*tmp).tree_entry.rbe_parent).tree_entry.rbe_left {
                            (*(*tmp).tree_entry.rbe_parent).tree_entry.rbe_left = oright;
                        } else {
                            (*(*tmp).tree_entry.rbe_parent).tree_entry.rbe_right = oright;
                        }
                    } else {
                        (*head).rbh_root = oright;
                    }
                    (*oright).tree_entry.rbe_left = tmp;
                    (*tmp).tree_entry.rbe_parent = oright;
                    !(*oright).tree_entry.rbe_parent.is_null();
                    tmp = (*parent).tree_entry.rbe_left;
                }
                (*tmp).tree_entry.rbe_color = (*parent).tree_entry.rbe_color;
                (*parent).tree_entry.rbe_color = RB_BLACK;
                if !(*tmp).tree_entry.rbe_left.is_null() {
                    (*(*tmp).tree_entry.rbe_left).tree_entry.rbe_color = RB_BLACK;
                }
                tmp = (*parent).tree_entry.rbe_left;
                (*parent).tree_entry.rbe_left = (*tmp).tree_entry.rbe_right;
                if !(*parent).tree_entry.rbe_left.is_null() {
                    (*(*tmp).tree_entry.rbe_right).tree_entry.rbe_parent = parent;
                }
                (*tmp).tree_entry.rbe_parent = (*parent).tree_entry.rbe_parent;
                if !(*tmp).tree_entry.rbe_parent.is_null() {
                    if parent == (*(*parent).tree_entry.rbe_parent).tree_entry.rbe_left {
                        (*(*parent).tree_entry.rbe_parent).tree_entry.rbe_left = tmp;
                    } else {
                        (*(*parent).tree_entry.rbe_parent).tree_entry.rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).tree_entry.rbe_right = parent;
                (*parent).tree_entry.rbe_parent = tmp;
                !(*tmp).tree_entry.rbe_parent.is_null();
                elm = (*head).rbh_root;
                break;
            }
        }
    }
    if !elm.is_null() {
        (*elm).tree_entry.rbe_color = RB_BLACK;
    }
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_tree_RB_REMOVE(
    mut head: *mut window_pane_tree,
    mut elm: *mut window_pane,
) -> *mut window_pane {
    let mut current_block: u64;
    let mut child: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut parent: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut old: *mut window_pane = elm;
    let mut color: ::core::ffi::c_int = 0;
    if (*elm).tree_entry.rbe_left.is_null() {
        child = (*elm).tree_entry.rbe_right;
        current_block = 7245201122033322888;
    } else if (*elm).tree_entry.rbe_right.is_null() {
        child = (*elm).tree_entry.rbe_left;
        current_block = 7245201122033322888;
    } else {
        let mut left: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
        elm = (*elm).tree_entry.rbe_right;
        loop {
            left = (*elm).tree_entry.rbe_left;
            if left.is_null() {
                break;
            }
            elm = left;
        }
        child = (*elm).tree_entry.rbe_right;
        parent = (*elm).tree_entry.rbe_parent;
        color = (*elm).tree_entry.rbe_color;
        if !child.is_null() {
            (*child).tree_entry.rbe_parent = parent;
        }
        if !parent.is_null() {
            if (*parent).tree_entry.rbe_left == elm {
                (*parent).tree_entry.rbe_left = child;
            } else {
                (*parent).tree_entry.rbe_right = child;
            }
        } else {
            (*head).rbh_root = child;
        }
        if (*elm).tree_entry.rbe_parent == old {
            parent = elm;
        }
        (*elm).tree_entry = (*old).tree_entry;
        if !(*old).tree_entry.rbe_parent.is_null() {
            if (*(*old).tree_entry.rbe_parent).tree_entry.rbe_left == old {
                (*(*old).tree_entry.rbe_parent).tree_entry.rbe_left = elm;
            } else {
                (*(*old).tree_entry.rbe_parent).tree_entry.rbe_right = elm;
            }
        } else {
            (*head).rbh_root = elm;
        }
        (*(*old).tree_entry.rbe_left).tree_entry.rbe_parent = elm;
        if !(*old).tree_entry.rbe_right.is_null() {
            (*(*old).tree_entry.rbe_right).tree_entry.rbe_parent = elm;
        }
        if !parent.is_null() {
            left = parent;
            loop {
                left = (*left).tree_entry.rbe_parent;
                if left.is_null() {
                    break;
                }
            }
        }
        current_block = 6103297587398279902;
    }
    match current_block {
        7245201122033322888 => {
            parent = (*elm).tree_entry.rbe_parent;
            color = (*elm).tree_entry.rbe_color;
            if !child.is_null() {
                (*child).tree_entry.rbe_parent = parent;
            }
            if !parent.is_null() {
                if (*parent).tree_entry.rbe_left == elm {
                    (*parent).tree_entry.rbe_left = child;
                } else {
                    (*parent).tree_entry.rbe_right = child;
                }
            } else {
                (*head).rbh_root = child;
            }
        }
        _ => {}
    }
    if color == RB_BLACK {
        window_pane_tree_RB_REMOVE_COLOR(head, parent, child);
    }
    return old;
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_tree_RB_INSERT_COLOR(
    mut head: *mut window_pane_tree,
    mut elm: *mut window_pane,
) {
    let mut parent: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut gparent: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut tmp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    loop {
        parent = (*elm).tree_entry.rbe_parent;
        if !(!parent.is_null() && (*parent).tree_entry.rbe_color == RB_RED) {
            break;
        }
        gparent = (*parent).tree_entry.rbe_parent;
        if parent == (*gparent).tree_entry.rbe_left {
            tmp = (*gparent).tree_entry.rbe_right;
            if !tmp.is_null() && (*tmp).tree_entry.rbe_color == RB_RED {
                (*tmp).tree_entry.rbe_color = RB_BLACK;
                (*parent).tree_entry.rbe_color = RB_BLACK;
                (*gparent).tree_entry.rbe_color = RB_RED;
                elm = gparent;
            } else {
                if (*parent).tree_entry.rbe_right == elm {
                    tmp = (*parent).tree_entry.rbe_right;
                    (*parent).tree_entry.rbe_right = (*tmp).tree_entry.rbe_left;
                    if !(*parent).tree_entry.rbe_right.is_null() {
                        (*(*tmp).tree_entry.rbe_left).tree_entry.rbe_parent = parent;
                    }
                    (*tmp).tree_entry.rbe_parent = (*parent).tree_entry.rbe_parent;
                    if !(*tmp).tree_entry.rbe_parent.is_null() {
                        if parent == (*(*parent).tree_entry.rbe_parent).tree_entry.rbe_left {
                            (*(*parent).tree_entry.rbe_parent).tree_entry.rbe_left = tmp;
                        } else {
                            (*(*parent).tree_entry.rbe_parent).tree_entry.rbe_right = tmp;
                        }
                    } else {
                        (*head).rbh_root = tmp;
                    }
                    (*tmp).tree_entry.rbe_left = parent;
                    (*parent).tree_entry.rbe_parent = tmp;
                    !(*tmp).tree_entry.rbe_parent.is_null();
                    tmp = parent;
                    parent = elm;
                    elm = tmp;
                }
                (*parent).tree_entry.rbe_color = RB_BLACK;
                (*gparent).tree_entry.rbe_color = RB_RED;
                tmp = (*gparent).tree_entry.rbe_left;
                (*gparent).tree_entry.rbe_left = (*tmp).tree_entry.rbe_right;
                if !(*gparent).tree_entry.rbe_left.is_null() {
                    (*(*tmp).tree_entry.rbe_right).tree_entry.rbe_parent = gparent;
                }
                (*tmp).tree_entry.rbe_parent = (*gparent).tree_entry.rbe_parent;
                if !(*tmp).tree_entry.rbe_parent.is_null() {
                    if gparent == (*(*gparent).tree_entry.rbe_parent).tree_entry.rbe_left {
                        (*(*gparent).tree_entry.rbe_parent).tree_entry.rbe_left = tmp;
                    } else {
                        (*(*gparent).tree_entry.rbe_parent).tree_entry.rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).tree_entry.rbe_right = gparent;
                (*gparent).tree_entry.rbe_parent = tmp;
                !(*tmp).tree_entry.rbe_parent.is_null();
            }
        } else {
            tmp = (*gparent).tree_entry.rbe_left;
            if !tmp.is_null() && (*tmp).tree_entry.rbe_color == RB_RED {
                (*tmp).tree_entry.rbe_color = RB_BLACK;
                (*parent).tree_entry.rbe_color = RB_BLACK;
                (*gparent).tree_entry.rbe_color = RB_RED;
                elm = gparent;
            } else {
                if (*parent).tree_entry.rbe_left == elm {
                    tmp = (*parent).tree_entry.rbe_left;
                    (*parent).tree_entry.rbe_left = (*tmp).tree_entry.rbe_right;
                    if !(*parent).tree_entry.rbe_left.is_null() {
                        (*(*tmp).tree_entry.rbe_right).tree_entry.rbe_parent = parent;
                    }
                    (*tmp).tree_entry.rbe_parent = (*parent).tree_entry.rbe_parent;
                    if !(*tmp).tree_entry.rbe_parent.is_null() {
                        if parent == (*(*parent).tree_entry.rbe_parent).tree_entry.rbe_left {
                            (*(*parent).tree_entry.rbe_parent).tree_entry.rbe_left = tmp;
                        } else {
                            (*(*parent).tree_entry.rbe_parent).tree_entry.rbe_right = tmp;
                        }
                    } else {
                        (*head).rbh_root = tmp;
                    }
                    (*tmp).tree_entry.rbe_right = parent;
                    (*parent).tree_entry.rbe_parent = tmp;
                    !(*tmp).tree_entry.rbe_parent.is_null();
                    tmp = parent;
                    parent = elm;
                    elm = tmp;
                }
                (*parent).tree_entry.rbe_color = RB_BLACK;
                (*gparent).tree_entry.rbe_color = RB_RED;
                tmp = (*gparent).tree_entry.rbe_right;
                (*gparent).tree_entry.rbe_right = (*tmp).tree_entry.rbe_left;
                if !(*gparent).tree_entry.rbe_right.is_null() {
                    (*(*tmp).tree_entry.rbe_left).tree_entry.rbe_parent = gparent;
                }
                (*tmp).tree_entry.rbe_parent = (*gparent).tree_entry.rbe_parent;
                if !(*tmp).tree_entry.rbe_parent.is_null() {
                    if gparent == (*(*gparent).tree_entry.rbe_parent).tree_entry.rbe_left {
                        (*(*gparent).tree_entry.rbe_parent).tree_entry.rbe_left = tmp;
                    } else {
                        (*(*gparent).tree_entry.rbe_parent).tree_entry.rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).tree_entry.rbe_left = gparent;
                (*gparent).tree_entry.rbe_parent = tmp;
                !(*tmp).tree_entry.rbe_parent.is_null();
            }
        }
    }
    (*(*head).rbh_root).tree_entry.rbe_color = RB_BLACK;
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_tree_RB_INSERT(
    mut head: *mut window_pane_tree,
    mut elm: *mut window_pane,
) -> *mut window_pane {
    let mut tmp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut parent: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut comp: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    tmp = (*head).rbh_root;
    while !tmp.is_null() {
        parent = tmp;
        comp = window_pane_cmp(elm, parent);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).tree_entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).tree_entry.rbe_right;
        } else {
            return tmp;
        }
    }
    (*elm).tree_entry.rbe_parent = parent;
    (*elm).tree_entry.rbe_right = ::core::ptr::null_mut::<window_pane>();
    (*elm).tree_entry.rbe_left = (*elm).tree_entry.rbe_right;
    (*elm).tree_entry.rbe_color = RB_RED;
    if !parent.is_null() {
        if comp < 0 as ::core::ffi::c_int {
            (*parent).tree_entry.rbe_left = elm;
        } else {
            (*parent).tree_entry.rbe_right = elm;
        }
    } else {
        (*head).rbh_root = elm;
    }
    window_pane_tree_RB_INSERT_COLOR(head, elm);
    return ::core::ptr::null_mut::<window_pane>();
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_tree_RB_NEXT(mut elm: *mut window_pane) -> *mut window_pane {
    if !(*elm).tree_entry.rbe_right.is_null() {
        elm = (*elm).tree_entry.rbe_right;
        while !(*elm).tree_entry.rbe_left.is_null() {
            elm = (*elm).tree_entry.rbe_left;
        }
    } else if !(*elm).tree_entry.rbe_parent.is_null()
        && elm == (*(*elm).tree_entry.rbe_parent).tree_entry.rbe_left
    {
        elm = (*elm).tree_entry.rbe_parent;
    } else {
        while !(*elm).tree_entry.rbe_parent.is_null()
            && elm == (*(*elm).tree_entry.rbe_parent).tree_entry.rbe_right
        {
            elm = (*elm).tree_entry.rbe_parent;
        }
        elm = (*elm).tree_entry.rbe_parent;
    }
    return elm;
}
#[no_mangle]
pub unsafe extern "C" fn window_cmp(
    mut w1: *mut window,
    mut w2: *mut window,
) -> ::core::ffi::c_int {
    return (*w1).id.wrapping_sub((*w2).id) as ::core::ffi::c_int;
}
unsafe extern "C" fn window_fire_renamed(
    mut w: *mut window,
    mut old_name: *const ::core::ffi::c_char,
) {
    let mut ep: *mut event_payload = ::core::ptr::null_mut::<event_payload>();
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null::<cmd_find_state>() as *mut cmd_find_state,
        s: ::core::ptr::null::<session>() as *mut session,
        wl: ::core::ptr::null::<winlink>() as *mut winlink,
        w: ::core::ptr::null::<window>() as *mut window,
        wp: ::core::ptr::null::<window_pane>() as *mut window_pane,
        idx: 0,
    };
    ep = event_payload_create();
    cmd_find_from_window(&raw mut fs, w, 0 as ::core::ffi::c_int);
    event_payload_set_target(ep, &raw mut fs);
    event_payload_set_window(
        ep,
        b"window\0" as *const u8 as *const ::core::ffi::c_char,
        w,
    );
    event_payload_set_string(
        ep,
        b"old_name\0" as *const u8 as *const ::core::ffi::c_char,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        old_name,
    );
    event_payload_set_string(
        ep,
        b"new_name\0" as *const u8 as *const ::core::ffi::c_char,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        (*w).name,
    );
    events_fire(
        b"window-renamed\0" as *const u8 as *const ::core::ffi::c_char,
        ep,
    );
}
unsafe extern "C" fn window_fire_pane_changed(
    mut w: *mut window,
    mut wp: *mut window_pane,
    mut lastwp: *mut window_pane,
) {
    let mut ep: *mut event_payload = ::core::ptr::null_mut::<event_payload>();
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null::<cmd_find_state>() as *mut cmd_find_state,
        s: ::core::ptr::null::<session>() as *mut session,
        wl: ::core::ptr::null::<winlink>() as *mut winlink,
        w: ::core::ptr::null::<window>() as *mut window,
        wp: ::core::ptr::null::<window_pane>() as *mut window_pane,
        idx: 0,
    };
    ep = event_payload_create();
    cmd_find_from_pane(&raw mut fs, wp, 0 as ::core::ffi::c_int);
    event_payload_set_target(ep, &raw mut fs);
    event_payload_set_window(
        ep,
        b"window\0" as *const u8 as *const ::core::ffi::c_char,
        w,
    );
    event_payload_set_pane(ep, b"pane\0" as *const u8 as *const ::core::ffi::c_char, wp);
    event_payload_set_pane(
        ep,
        b"new_pane\0" as *const u8 as *const ::core::ffi::c_char,
        wp,
    );
    if !lastwp.is_null() {
        event_payload_set_pane(
            ep,
            b"old_pane\0" as *const u8 as *const ::core::ffi::c_char,
            lastwp,
        );
    }
    events_fire(
        b"window-pane-changed\0" as *const u8 as *const ::core::ffi::c_char,
        ep,
    );
}
#[no_mangle]
pub unsafe extern "C" fn window_fire_pane_moved(
    mut wp: *mut window_pane,
    mut old_w: *mut window,
    mut old_idx: ::core::ffi::c_int,
    mut new_w: *mut window,
    mut new_idx: ::core::ffi::c_int,
) {
    let mut ep: *mut event_payload = ::core::ptr::null_mut::<event_payload>();
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null::<cmd_find_state>() as *mut cmd_find_state,
        s: ::core::ptr::null::<session>() as *mut session,
        wl: ::core::ptr::null::<winlink>() as *mut winlink,
        w: ::core::ptr::null::<window>() as *mut window,
        wp: ::core::ptr::null::<window_pane>() as *mut window_pane,
        idx: 0,
    };
    ep = event_payload_create();
    cmd_find_from_pane(&raw mut fs, wp, 0 as ::core::ffi::c_int);
    event_payload_set_target(ep, &raw mut fs);
    event_payload_set_pane(ep, b"pane\0" as *const u8 as *const ::core::ffi::c_char, wp);
    event_payload_set_window(
        ep,
        b"window\0" as *const u8 as *const ::core::ffi::c_char,
        new_w,
    );
    event_payload_set_window(
        ep,
        b"old_window\0" as *const u8 as *const ::core::ffi::c_char,
        old_w,
    );
    event_payload_set_window(
        ep,
        b"new_window\0" as *const u8 as *const ::core::ffi::c_char,
        new_w,
    );
    if old_idx != -(1 as ::core::ffi::c_int) {
        event_payload_set_int(
            ep,
            b"old_window_index\0" as *const u8 as *const ::core::ffi::c_char,
            old_idx,
        );
    }
    if new_idx != -(1 as ::core::ffi::c_int) {
        event_payload_set_int(
            ep,
            b"window_index\0" as *const u8 as *const ::core::ffi::c_char,
            new_idx,
        );
        event_payload_set_int(
            ep,
            b"new_window_index\0" as *const u8 as *const ::core::ffi::c_char,
            new_idx,
        );
    }
    events_fire(
        b"pane-moved\0" as *const u8 as *const ::core::ffi::c_char,
        ep,
    );
}
unsafe extern "C" fn window_fire_pane_mode_changed(
    mut name: *const ::core::ffi::c_char,
    mut wp: *mut window_pane,
    mut previous: *const ::core::ffi::c_char,
    mut current: *const ::core::ffi::c_char,
    mut entered: ::core::ffi::c_int,
) {
    let mut ep: *mut event_payload = ::core::ptr::null_mut::<event_payload>();
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null::<cmd_find_state>() as *mut cmd_find_state,
        s: ::core::ptr::null::<session>() as *mut session,
        wl: ::core::ptr::null::<winlink>() as *mut winlink,
        w: ::core::ptr::null::<window>() as *mut window,
        wp: ::core::ptr::null::<window_pane>() as *mut window_pane,
        idx: 0,
    };
    ep = event_payload_create();
    cmd_find_from_pane(&raw mut fs, wp, 0 as ::core::ffi::c_int);
    event_payload_set_target(ep, &raw mut fs);
    event_payload_set_pane(ep, b"pane\0" as *const u8 as *const ::core::ffi::c_char, wp);
    event_payload_set_window(
        ep,
        b"window\0" as *const u8 as *const ::core::ffi::c_char,
        (*wp).window as *mut window,
    );
    if !current.is_null() {
        event_payload_set_string(
            ep,
            b"current_mode\0" as *const u8 as *const ::core::ffi::c_char,
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            current,
        );
    }
    if !previous.is_null() {
        event_payload_set_string(
            ep,
            b"previous_mode\0" as *const u8 as *const ::core::ffi::c_char,
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            previous,
        );
    }
    event_payload_set_int(
        ep,
        b"mode_entered\0" as *const u8 as *const ::core::ffi::c_char,
        entered,
    );
    events_fire(name, ep);
}
unsafe extern "C" fn window_fire_pane_prompt(
    mut name: *const ::core::ffi::c_char,
    mut wp: *mut window_pane,
    mut type_0: prompt_type,
) {
    let mut ep: *mut event_payload = ::core::ptr::null_mut::<event_payload>();
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null::<cmd_find_state>() as *mut cmd_find_state,
        s: ::core::ptr::null::<session>() as *mut session,
        wl: ::core::ptr::null::<winlink>() as *mut winlink,
        w: ::core::ptr::null::<window>() as *mut window,
        wp: ::core::ptr::null::<window_pane>() as *mut window_pane,
        idx: 0,
    };
    let mut type_string: *const ::core::ffi::c_char = prompt_type_string(type_0);
    ep = event_payload_create();
    cmd_find_from_pane(&raw mut fs, wp, 0 as ::core::ffi::c_int);
    event_payload_set_target(ep, &raw mut fs);
    event_payload_set_pane(ep, b"pane\0" as *const u8 as *const ::core::ffi::c_char, wp);
    event_payload_set_window(
        ep,
        b"window\0" as *const u8 as *const ::core::ffi::c_char,
        (*wp).window as *mut window,
    );
    event_payload_set_string(
        ep,
        b"prompt_type\0" as *const u8 as *const ::core::ffi::c_char,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        type_string,
    );
    events_fire(name, ep);
}
#[no_mangle]
pub unsafe extern "C" fn winlink_cmp(
    mut wl1: *mut winlink,
    mut wl2: *mut winlink,
) -> ::core::ffi::c_int {
    return (*wl1).idx - (*wl2).idx;
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_cmp(
    mut wp1: *mut window_pane,
    mut wp2: *mut window_pane,
) -> ::core::ffi::c_int {
    return (*wp1).id.wrapping_sub((*wp2).id) as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn winlink_find_by_window(
    mut wwl: *mut winlinks,
    mut w: *mut window,
) -> *mut winlink {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    wl = winlinks_RB_MINMAX(wwl, RB_NEGINF);
    while !wl.is_null() {
        if (*wl).window == w {
            return wl;
        }
        wl = winlinks_RB_NEXT(wl);
    }
    return ::core::ptr::null_mut::<winlink>();
}
#[no_mangle]
pub unsafe extern "C" fn winlink_find_by_index(
    mut wwl: *mut winlinks,
    mut idx: ::core::ffi::c_int,
) -> *mut winlink {
    let mut wl: winlink = winlink {
        idx: 0,
        session: ::core::ptr::null_mut::<session>(),
        window: ::core::ptr::null_mut::<window>(),
        flags: 0,
        entry: winlink_entry {
            rbe_left: ::core::ptr::null_mut::<winlink>(),
            rbe_right: ::core::ptr::null_mut::<winlink>(),
            rbe_parent: ::core::ptr::null_mut::<winlink>(),
            rbe_color: 0,
        },
        wentry: winlink_wentry {
            tqe_next: ::core::ptr::null_mut::<winlink>(),
            tqe_prev: ::core::ptr::null_mut::<*mut winlink>(),
        },
        sentry: winlink_sentry {
            tqe_next: ::core::ptr::null_mut::<winlink>(),
            tqe_prev: ::core::ptr::null_mut::<*mut winlink>(),
        },
    };
    if idx < 0 as ::core::ffi::c_int {
        fatalx(b"bad index\0" as *const u8 as *const ::core::ffi::c_char);
    }
    wl.idx = idx;
    return winlinks_RB_FIND(wwl, &raw mut wl);
}
#[no_mangle]
pub unsafe extern "C" fn winlink_find_by_window_id(
    mut wwl: *mut winlinks,
    mut id: u_int,
) -> *mut winlink {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    wl = winlinks_RB_MINMAX(wwl, RB_NEGINF);
    while !wl.is_null() {
        if (*(*wl).window).id == id {
            return wl;
        }
        wl = winlinks_RB_NEXT(wl);
    }
    return ::core::ptr::null_mut::<winlink>();
}
unsafe extern "C" fn winlink_next_index(
    mut wwl: *mut winlinks,
    mut idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut i: ::core::ffi::c_int = 0;
    i = idx;
    loop {
        if winlink_find_by_index(wwl, i).is_null() {
            return i;
        }
        if i == INT_MAX {
            i = 0 as ::core::ffi::c_int;
        } else {
            i += 1;
        }
        if !(i != idx) {
            break;
        }
    }
    return -(1 as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn winlink_count(mut wwl: *mut winlinks) -> u_int {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut n: u_int = 0;
    n = 0 as u_int;
    wl = winlinks_RB_MINMAX(wwl, RB_NEGINF);
    while !wl.is_null() {
        n = n.wrapping_add(1);
        wl = winlinks_RB_NEXT(wl);
    }
    return n;
}
#[no_mangle]
pub unsafe extern "C" fn winlink_add(
    mut wwl: *mut winlinks,
    mut idx: ::core::ffi::c_int,
) -> *mut winlink {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    if idx < 0 as ::core::ffi::c_int {
        idx = winlink_next_index(wwl, -idx - 1 as ::core::ffi::c_int);
        if idx == -(1 as ::core::ffi::c_int) {
            return ::core::ptr::null_mut::<winlink>();
        }
    } else if !winlink_find_by_index(wwl, idx).is_null() {
        return ::core::ptr::null_mut::<winlink>();
    }
    wl = xcalloc(1 as size_t, ::core::mem::size_of::<winlink>() as size_t) as *mut winlink;
    (*wl).idx = idx;
    winlinks_RB_INSERT(wwl, wl);
    return wl;
}
#[no_mangle]
pub unsafe extern "C" fn winlink_set_window(mut wl: *mut winlink, mut w: *mut window) {
    if !(*wl).window.is_null() {
        if !(*wl).wentry.tqe_next.is_null() {
            (*(*wl).wentry.tqe_next).wentry.tqe_prev = (*wl).wentry.tqe_prev;
        } else {
            (*(*wl).window).winlinks.tqh_last = (*wl).wentry.tqe_prev;
        }
        *(*wl).wentry.tqe_prev = (*wl).wentry.tqe_next;
        window_remove_ref(
            (*wl).window,
            b"winlink_set_window\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    (*wl).wentry.tqe_next = ::core::ptr::null_mut::<winlink>();
    (*wl).wentry.tqe_prev = (*w).winlinks.tqh_last;
    *(*w).winlinks.tqh_last = wl;
    (*w).winlinks.tqh_last = &raw mut (*wl).wentry.tqe_next;
    (*wl).window = w;
    window_add_ref(
        w,
        b"winlink_set_window\0" as *const u8 as *const ::core::ffi::c_char,
    );
}
#[no_mangle]
pub unsafe extern "C" fn winlink_remove(mut wwl: *mut winlinks, mut wl: *mut winlink) {
    let mut w: *mut window = (*wl).window;
    if !w.is_null() {
        if !(*wl).wentry.tqe_next.is_null() {
            (*(*wl).wentry.tqe_next).wentry.tqe_prev = (*wl).wentry.tqe_prev;
        } else {
            (*w).winlinks.tqh_last = (*wl).wentry.tqe_prev;
        }
        *(*wl).wentry.tqe_prev = (*wl).wentry.tqe_next;
        window_remove_ref(
            w,
            b"winlink_remove\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    winlinks_RB_REMOVE(wwl, wl);
    free(wl as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn winlink_next(mut wl: *mut winlink) -> *mut winlink {
    return winlinks_RB_NEXT(wl);
}
#[no_mangle]
pub unsafe extern "C" fn winlink_previous(mut wl: *mut winlink) -> *mut winlink {
    return winlinks_RB_PREV(wl);
}
#[no_mangle]
pub unsafe extern "C" fn winlink_next_by_number(
    mut wl: *mut winlink,
    mut s: *mut session,
    mut n: ::core::ffi::c_int,
) -> *mut winlink {
    while n > 0 as ::core::ffi::c_int {
        wl = winlinks_RB_NEXT(wl);
        if wl.is_null() {
            wl = winlinks_RB_MINMAX(&raw mut (*s).windows, RB_NEGINF);
        }
        n -= 1;
    }
    return wl;
}
#[no_mangle]
pub unsafe extern "C" fn winlink_previous_by_number(
    mut wl: *mut winlink,
    mut s: *mut session,
    mut n: ::core::ffi::c_int,
) -> *mut winlink {
    while n > 0 as ::core::ffi::c_int {
        wl = winlinks_RB_PREV(wl);
        if wl.is_null() {
            wl = winlinks_RB_MINMAX(&raw mut (*s).windows, RB_INF);
        }
        n -= 1;
    }
    return wl;
}
#[no_mangle]
pub unsafe extern "C" fn winlink_stack_push(mut stack: *mut winlink_stack, mut wl: *mut winlink) {
    if wl.is_null() {
        return;
    }
    winlink_stack_remove(stack, wl);
    (*wl).sentry.tqe_next = (*stack).tqh_first;
    if !(*wl).sentry.tqe_next.is_null() {
        (*(*stack).tqh_first).sentry.tqe_prev = &raw mut (*wl).sentry.tqe_next;
    } else {
        (*stack).tqh_last = &raw mut (*wl).sentry.tqe_next;
    }
    (*stack).tqh_first = wl;
    (*wl).sentry.tqe_prev = &raw mut (*stack).tqh_first;
    (*wl).flags |= WINLINK_VISITED;
}
#[no_mangle]
pub unsafe extern "C" fn winlink_stack_remove(mut stack: *mut winlink_stack, mut wl: *mut winlink) {
    if !wl.is_null() && (*wl).flags & WINLINK_VISITED != 0 {
        if !(*wl).sentry.tqe_next.is_null() {
            (*(*wl).sentry.tqe_next).sentry.tqe_prev = (*wl).sentry.tqe_prev;
        } else {
            (*stack).tqh_last = (*wl).sentry.tqe_prev;
        }
        *(*wl).sentry.tqe_prev = (*wl).sentry.tqe_next;
        (*wl).flags &= !WINLINK_VISITED;
    }
}
#[no_mangle]
pub unsafe extern "C" fn window_find_by_id_str(mut s: *const ::core::ffi::c_char) -> *mut window {
    let mut errstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut id: u_int = 0;
    if *s as ::core::ffi::c_int != '@' as i32 {
        return ::core::ptr::null_mut::<window>();
    }
    id = strtonum(
        s.offset(1 as ::core::ffi::c_int as isize),
        0 as ::core::ffi::c_longlong,
        UINT_MAX as ::core::ffi::c_longlong,
        &raw mut errstr,
    ) as u_int;
    if !errstr.is_null() {
        return ::core::ptr::null_mut::<window>();
    }
    return window_find_by_id(id);
}
#[no_mangle]
pub unsafe extern "C" fn window_find_by_id(mut id: u_int) -> *mut window {
    let mut w: window = window {
        id: 0,
        latest: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        name: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        name_event: event::ZERO,
        name_time: timeval {
            tv_sec: 0,
            tv_usec: 0,
        },
        alerts_timer: event::ZERO,
        offset_timer: event::ZERO,
        activity_time: timeval {
            tv_sec: 0,
            tv_usec: 0,
        },
        creation_time: timeval {
            tv_sec: 0,
            tv_usec: 0,
        },
        active: ::core::ptr::null_mut::<window_pane>(),
        modal: ::core::ptr::null_mut::<window_pane>(),
        modal_last: ::core::ptr::null_mut::<window_pane>(),
        was_zoomed: ::core::ptr::null_mut::<window_pane>(),
        last_panes: window_panes {
            tqh_first: ::core::ptr::null_mut::<window_pane>(),
            tqh_last: ::core::ptr::null_mut::<*mut window_pane>(),
        },
        z_index: window_panes {
            tqh_first: ::core::ptr::null_mut::<window_pane>(),
            tqh_last: ::core::ptr::null_mut::<*mut window_pane>(),
        },
        panes: window_panes {
            tqh_first: ::core::ptr::null_mut::<window_pane>(),
            tqh_last: ::core::ptr::null_mut::<*mut window_pane>(),
        },
        lastlayout: 0,
        layout_root: ::core::ptr::null_mut::<layout_cell>(),
        saved_layout_root: ::core::ptr::null_mut::<layout_cell>(),
        old_layout: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        sx: 0,
        sy: 0,
        manual_sx: 0,
        manual_sy: 0,
        xpixel: 0,
        ypixel: 0,
        new_sx: 0,
        new_sy: 0,
        new_xpixel: 0,
        new_ypixel: 0,
        redraw_scene_generation: 0,
        menu: ::core::ptr::null_mut::<menu_data>(),
        menu_last_px: 0,
        menu_last_py: 0,
        last_new_pane_x: 0,
        last_new_pane_y: 0,
        sb: 0,
        sb_pos: 0,
        inside_cell: grid_cell {
            data: utf8_data {
                data: [0; 32],
                have: 0,
                size: 0,
                width: 0,
            },
            attr: 0,
            flags: 0,
            fg: 0,
            bg: 0,
            us: 0,
            link: 0,
        },
        outside_cell: grid_cell {
            data: utf8_data {
                data: [0; 32],
                have: 0,
                size: 0,
                width: 0,
            },
            attr: 0,
            flags: 0,
            fg: 0,
            bg: 0,
            us: 0,
            link: 0,
        },
        flags: 0,
        alerts_queued: 0,
        alerts_entry: window_alerts_entry {
            tqe_next: ::core::ptr::null_mut::<window>(),
            tqe_prev: ::core::ptr::null_mut::<*mut window>(),
        },
        options: ::core::ptr::null_mut::<options>(),
        references: 0,
        winlinks: window_winlinks {
            tqh_first: ::core::ptr::null_mut::<winlink>(),
            tqh_last: ::core::ptr::null_mut::<*mut winlink>(),
        },
        entry: window_entry {
            rbe_left: ::core::ptr::null_mut::<window>(),
            rbe_right: ::core::ptr::null_mut::<window>(),
            rbe_parent: ::core::ptr::null_mut::<window>(),
            rbe_color: 0,
        },
    };
    w.id = id;
    return windows_RB_FIND(&raw mut windows, &raw mut w);
}
#[no_mangle]
pub unsafe extern "C" fn window_update_activity(mut w: *mut window) {
    gettimeofday(&raw mut (*w).activity_time, NULL);
    alerts_queue(w, WINDOW_ACTIVITY);
}
#[no_mangle]
pub unsafe extern "C" fn window_create(
    mut sx: u_int,
    mut sy: u_int,
    mut xpixel: u_int,
    mut ypixel: u_int,
) -> *mut window {
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    if xpixel == 0 as u_int {
        xpixel = DEFAULT_XPIXEL as u_int;
    }
    if ypixel == 0 as u_int {
        ypixel = DEFAULT_YPIXEL as u_int;
    }
    w = xcalloc(1 as size_t, ::core::mem::size_of::<window>() as size_t) as *mut window;
    (*w).name = xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
    (*w).flags = 0 as ::core::ffi::c_int;
    (*w).panes.tqh_first = ::core::ptr::null_mut::<window_pane>();
    (*w).panes.tqh_last = &raw mut (*w).panes.tqh_first;
    (*w).z_index.tqh_first = ::core::ptr::null_mut::<window_pane>();
    (*w).z_index.tqh_last = &raw mut (*w).z_index.tqh_first;
    (*w).last_panes.tqh_first = ::core::ptr::null_mut::<window_pane>();
    (*w).last_panes.tqh_last = &raw mut (*w).last_panes.tqh_first;
    (*w).active = ::core::ptr::null_mut::<window_pane>();
    (*w).lastlayout = -(1 as ::core::ffi::c_int);
    (*w).layout_root = ::core::ptr::null_mut::<layout_cell>();
    (*w).sx = sx;
    (*w).sy = sy;
    (*w).manual_sx = sx;
    (*w).manual_sy = sy;
    (*w).xpixel = xpixel;
    (*w).ypixel = ypixel;
    (*w).options = options_create(global_w_options);
    (*w).sb = options_get_number(
        (*w).options,
        b"pane-scrollbars\0" as *const u8 as *const ::core::ffi::c_char,
    ) as ::core::ffi::c_int;
    (*w).sb_pos = options_get_number(
        (*w).options,
        b"pane-scrollbars-position\0" as *const u8 as *const ::core::ffi::c_char,
    ) as ::core::ffi::c_int;
    (*w).references = 0 as u_int;
    (*w).winlinks.tqh_first = ::core::ptr::null_mut::<winlink>();
    (*w).winlinks.tqh_last = &raw mut (*w).winlinks.tqh_first;
    let fresh0 = next_window_id;
    next_window_id = next_window_id.wrapping_add(1);
    (*w).id = fresh0;
    windows_RB_INSERT(&raw mut windows, w);
    if gettimeofday(&raw mut (*w).creation_time, NULL) != 0 as ::core::ffi::c_int {
        fatal(b"gettimeofday failed\0" as *const u8 as *const ::core::ffi::c_char);
    }
    window_update_activity(w);
    log_debug(
        b"%s: @%u create %ux%u (%ux%u)\0" as *const u8 as *const ::core::ffi::c_char,
        b"window_create\0" as *const u8 as *const ::core::ffi::c_char,
        (*w).id,
        sx,
        sy,
        (*w).xpixel,
        (*w).ypixel,
    );
    return w;
}
unsafe extern "C" fn window_destroy(mut w: *mut window) {
    log_debug(
        b"window @%u destroyed (%d references)\0" as *const u8 as *const ::core::ffi::c_char,
        (*w).id,
        (*w).references,
    );
    window_unzoom(w, 0 as ::core::ffi::c_int);
    windows_RB_REMOVE(&raw mut windows, w);
    layout_free_cell((*w).layout_root, 0 as ::core::ffi::c_int);
    layout_free_cell((*w).saved_layout_root, 0 as ::core::ffi::c_int);
    free((*w).old_layout as *mut ::core::ffi::c_void);
    menu_destroy(w);
    window_destroy_panes(w);
    if event_initialized(&raw mut (*w).name_event) != 0 {
        event_del(&raw mut (*w).name_event);
    }
    if event_initialized(&raw mut (*w).alerts_timer) != 0 {
        event_del(&raw mut (*w).alerts_timer);
    }
    if event_initialized(&raw mut (*w).offset_timer) != 0 {
        event_del(&raw mut (*w).offset_timer);
    }
    options_free((*w).options);
    free((*w).name as *mut ::core::ffi::c_void);
    free(w as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_destroy_ready(mut wp: *mut window_pane) -> ::core::ffi::c_int {
    let mut n: ::core::ffi::c_int = 0;
    if (*wp).pipe_fd != -(1 as ::core::ffi::c_int)
        && evbuffer_get_length(crate::src::reactor::stream_output((*wp).pipe_event)) != 0 as size_t
    {
        return 0 as ::core::ffi::c_int;
    }
    if ioctl((*wp).fd, FIONREAD as ::core::ffi::c_ulong, &raw mut n) != -(1 as ::core::ffi::c_int)
        && n > 0 as ::core::ffi::c_int
    {
        return 0 as ::core::ffi::c_int;
    }
    if !(*wp).flags & PANE_EXITED != 0 {
        return 0 as ::core::ffi::c_int;
    }
    if !(*wp).wait_item.is_null() && !(*wp).flags & PANE_STATUSREADY != 0 {
        return 0 as ::core::ffi::c_int;
    }
    if !(*wp).editor.is_null() && !(*wp).flags & PANE_STATUSREADY != 0 {
        return 0 as ::core::ffi::c_int;
    }
    return 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn window_add_ref(mut w: *mut window, mut from: *const ::core::ffi::c_char) {
    (*w).references = (*w).references.wrapping_add(1);
    log_debug(
        b"%s: @%u %s, now %d\0" as *const u8 as *const ::core::ffi::c_char,
        b"window_add_ref\0" as *const u8 as *const ::core::ffi::c_char,
        (*w).id,
        from,
        (*w).references,
    );
}
#[no_mangle]
pub unsafe extern "C" fn window_remove_ref(
    mut w: *mut window,
    mut from: *const ::core::ffi::c_char,
) {
    if (*w).references == 1 as u_int {
        events_fire_window(
            b"window-closed\0" as *const u8 as *const ::core::ffi::c_char,
            w,
        );
    }
    (*w).references = (*w).references.wrapping_sub(1);
    log_debug(
        b"%s: @%u %s, now %d\0" as *const u8 as *const ::core::ffi::c_char,
        b"window_remove_ref\0" as *const u8 as *const ::core::ffi::c_char,
        (*w).id,
        from,
        (*w).references,
    );
    if (*w).references == 0 as u_int {
        window_destroy(w);
    }
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_add_ref(
    mut wp: *mut window_pane,
    mut from: *const ::core::ffi::c_char,
) {
    (*wp).references += 1;
    log_debug(
        b"%s: %%%u %s, now %d\0" as *const u8 as *const ::core::ffi::c_char,
        b"window_pane_add_ref\0" as *const u8 as *const ::core::ffi::c_char,
        (*wp).id,
        from,
        (*wp).references,
    );
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_remove_ref(
    mut wp: *mut window_pane,
    mut from: *const ::core::ffi::c_char,
) {
    (*wp).references -= 1;
    log_debug(
        b"%s: %%%u %s, now %d\0" as *const u8 as *const ::core::ffi::c_char,
        b"window_pane_remove_ref\0" as *const u8 as *const ::core::ffi::c_char,
        (*wp).id,
        from,
        (*wp).references,
    );
    if (*wp).references == 0 as ::core::ffi::c_int {
        window_pane_free(wp);
    }
}
#[no_mangle]
pub unsafe extern "C" fn window_set_name(
    mut w: *mut window,
    mut new_name: *const ::core::ffi::c_char,
    mut untrusted: ::core::ffi::c_int,
) {
    let mut last: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    name = clean_name(new_name, untrusted);
    if !name.is_null() {
        last = xstrdup((*w).name);
        free((*w).name as *mut ::core::ffi::c_void);
        (*w).name = name;
        window_fire_renamed(w, last);
        free(last as *mut ::core::ffi::c_void);
    }
}
#[no_mangle]
pub unsafe extern "C" fn window_resize(
    mut w: *mut window,
    mut sx: u_int,
    mut sy: u_int,
    mut xpixel: ::core::ffi::c_int,
    mut ypixel: ::core::ffi::c_int,
) {
    if xpixel == 0 as ::core::ffi::c_int {
        xpixel = DEFAULT_XPIXEL;
    }
    if ypixel == 0 as ::core::ffi::c_int {
        ypixel = DEFAULT_YPIXEL;
    }
    log_debug(
        b"%s: @%u resize %ux%u (%ux%u)\0" as *const u8 as *const ::core::ffi::c_char,
        b"window_resize\0" as *const u8 as *const ::core::ffi::c_char,
        (*w).id,
        sx,
        sy,
        if xpixel == -(1 as ::core::ffi::c_int) {
            (*w).xpixel
        } else {
            xpixel as u_int
        },
        if ypixel == -(1 as ::core::ffi::c_int) {
            (*w).ypixel
        } else {
            ypixel as u_int
        },
    );
    (*w).sx = sx;
    (*w).sy = sy;
    if !(*w).menu.is_null() {
        menu_resize((*w).menu, w);
        server_redraw_window(w);
    }
    if xpixel != -(1 as ::core::ffi::c_int) {
        (*w).xpixel = xpixel as u_int;
    }
    if ypixel != -(1 as ::core::ffi::c_int) {
        (*w).ypixel = ypixel as u_int;
    }
    redraw_invalidate_scene(w);
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_send_resize(
    mut wp: *mut window_pane,
    mut sx: u_int,
    mut sy: u_int,
) {
    let mut w: *mut window = (*wp).window as *mut window;
    let mut ws: winsize = winsize {
        ws_row: 0,
        ws_col: 0,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };
    if (*wp).fd == -(1 as ::core::ffi::c_int) {
        return;
    }
    log_debug(
        b"%s: %%%u resize to %u,%u\0" as *const u8 as *const ::core::ffi::c_char,
        b"window_pane_send_resize\0" as *const u8 as *const ::core::ffi::c_char,
        (*wp).id,
        sx,
        sy,
    );
    memset(
        &raw mut ws as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<winsize>() as size_t,
    );
    ws.ws_col = sx as ::core::ffi::c_ushort;
    ws.ws_row = sy as ::core::ffi::c_ushort;
    ws.ws_xpixel = (*w).xpixel.wrapping_mul(ws.ws_col as u_int) as ::core::ffi::c_ushort;
    ws.ws_ypixel = (*w).ypixel.wrapping_mul(ws.ws_row as u_int) as ::core::ffi::c_ushort;
    if ioctl((*wp).fd, TIOCSWINSZ as ::core::ffi::c_ulong, &raw mut ws)
        == -(1 as ::core::ffi::c_int)
    {
        fatal(b"ioctl failed\0" as *const u8 as *const ::core::ffi::c_char);
    }
}
#[no_mangle]
pub unsafe extern "C" fn window_has_floating_panes(mut w: *mut window) -> ::core::ffi::c_int {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    wp = (*w).panes.tqh_first;
    while !wp.is_null() {
        if window_pane_is_floating(wp) != 0 {
            return 1 as ::core::ffi::c_int;
        }
        wp = (*wp).entry.tqe_next;
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn window_has_pane(
    mut w: *mut window,
    mut wp: *mut window_pane,
) -> ::core::ffi::c_int {
    let mut wp1: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    wp1 = (*w).panes.tqh_first;
    while !wp1.is_null() {
        if wp1 == wp {
            return 1 as ::core::ffi::c_int;
        }
        wp1 = (*wp1).entry.tqe_next;
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_contains(
    mut wp: *mut window_pane,
    mut x: u_int,
    mut y: u_int,
) -> ::core::ffi::c_int {
    let mut xoff: ::core::ffi::c_int = 0;
    let mut yoff: ::core::ffi::c_int = 0;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    if window_pane_is_visible(wp) == 0 {
        return 0 as ::core::ffi::c_int;
    }
    window_pane_full_size_offset(wp, &raw mut xoff, &raw mut yoff, &raw mut sx, &raw mut sy);
    if window_pane_is_floating(wp) == 0 {
        if (x as ::core::ffi::c_int) < xoff || x > (xoff as u_int).wrapping_add(sx) {
            return 0 as ::core::ffi::c_int;
        }
        if (y as ::core::ffi::c_int) < yoff || y > (yoff as u_int).wrapping_add(sy) {
            return 0 as ::core::ffi::c_int;
        }
    } else if window_pane_get_pane_lines(wp) as ::core::ffi::c_uint
        == PANE_LINES_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if (x as ::core::ffi::c_int) < xoff
            || x as ::core::ffi::c_int >= xoff + sx as ::core::ffi::c_int
        {
            return 0 as ::core::ffi::c_int;
        }
        if (y as ::core::ffi::c_int) < yoff
            || y as ::core::ffi::c_int >= yoff + sy as ::core::ffi::c_int
        {
            return 0 as ::core::ffi::c_int;
        }
    } else {
        if (x as ::core::ffi::c_int) < xoff - 1 as ::core::ffi::c_int
            || x > (xoff as u_int).wrapping_add(sx)
        {
            return 0 as ::core::ffi::c_int;
        }
        if (y as ::core::ffi::c_int) < yoff - 1 as ::core::ffi::c_int
            || y > (yoff as u_int).wrapping_add(sy)
        {
            return 0 as ::core::ffi::c_int;
        }
    }
    return 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn window_update_focus(mut w: *mut window) {
    if !w.is_null() {
        log_debug(
            b"%s: @%u\0" as *const u8 as *const ::core::ffi::c_char,
            b"window_update_focus\0" as *const u8 as *const ::core::ffi::c_char,
            (*w).id,
        );
        window_pane_update_focus((*w).active);
    }
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_update_focus(mut wp: *mut window_pane) {
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut focused: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if !wp.is_null() && !(*wp).flags & PANE_EXITED != 0 {
        if wp != (*(*wp).window).active {
            focused = 0 as ::core::ffi::c_int;
        } else {
            c = clients.tqh_first;
            while !c.is_null() {
                if !(*c).session.is_null()
                    && (*(*c).session).attached != 0 as u_int
                    && (*c).flags & CLIENT_FOCUSED as uint64_t != 0
                    && (*(*(*c).session).curw).window == (*wp).window
                    && (*c).overlay_draw.is_none()
                    && (*(*wp).window).menu.is_null()
                {
                    focused = 1 as ::core::ffi::c_int;
                    break;
                } else {
                    c = (*c).entry.tqe_next;
                }
            }
        }
        if focused == 0 && (*wp).flags & PANE_FOCUSED != 0 {
            log_debug(
                b"%s: %%%u focus out\0" as *const u8 as *const ::core::ffi::c_char,
                b"window_pane_update_focus\0" as *const u8 as *const ::core::ffi::c_char,
                (*wp).id,
            );
            if (*wp).base.mode & MODE_FOCUSON != 0 {
                bufferevent_write(
                    (*wp).event,
                    b"\x1B[O\0" as *const u8 as *const ::core::ffi::c_char
                        as *const ::core::ffi::c_void,
                    3 as size_t,
                );
            }
            events_fire_pane(
                b"pane-focus-out\0" as *const u8 as *const ::core::ffi::c_char,
                wp,
            );
            (*wp).flags &= !PANE_FOCUSED;
        } else if focused != 0 && !(*wp).flags & PANE_FOCUSED != 0 {
            log_debug(
                b"%s: %%%u focus in\0" as *const u8 as *const ::core::ffi::c_char,
                b"window_pane_update_focus\0" as *const u8 as *const ::core::ffi::c_char,
                (*wp).id,
            );
            if (*wp).base.mode & MODE_FOCUSON != 0 {
                bufferevent_write(
                    (*wp).event,
                    b"\x1B[I\0" as *const u8 as *const ::core::ffi::c_char
                        as *const ::core::ffi::c_void,
                    3 as size_t,
                );
            }
            events_fire_pane(
                b"pane-focus-in\0" as *const u8 as *const ::core::ffi::c_char,
                wp,
            );
            (*wp).flags |= PANE_FOCUSED;
        } else {
            log_debug(
                b"%s: %%%u focus unchanged\0" as *const u8 as *const ::core::ffi::c_char,
                b"window_pane_update_focus\0" as *const u8 as *const ::core::ffi::c_char,
                (*wp).id,
            );
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn window_set_active_pane(
    mut w: *mut window,
    mut wp: *mut window_pane,
    mut notify: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut lastwp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    log_debug(
        b"%s: pane %%%u\0" as *const u8 as *const ::core::ffi::c_char,
        b"window_set_active_pane\0" as *const u8 as *const ::core::ffi::c_char,
        (*wp).id,
    );
    if wp == (*w).active {
        return 0 as ::core::ffi::c_int;
    }
    if !(*w).modal.is_null() && wp != (*w).modal {
        return 0 as ::core::ffi::c_int;
    }
    if (*w).flags & WINDOW_ZOOMED != 0 && window_pane_is_visible(wp) == 0 {
        window_unzoom(w, 1 as ::core::ffi::c_int);
    }
    lastwp = (*w).active;
    window_pane_stack_remove(&raw mut (*w).last_panes, wp);
    window_pane_stack_push(&raw mut (*w).last_panes, lastwp);
    (*w).active = wp;
    let fresh1 = next_active_point;
    next_active_point = next_active_point.wrapping_add(1);
    (*(*w).active).active_point = fresh1;
    (*(*w).active).flags |= PANE_CHANGED;
    if options_get_number(
        global_options,
        b"focus-events\0" as *const u8 as *const ::core::ffi::c_char,
    ) != 0
    {
        window_pane_update_focus(lastwp);
        window_pane_update_focus((*w).active);
    }
    tty_update_window_offset(w);
    server_redraw_window(w);
    if notify != 0 {
        window_fire_pane_changed(w, (*w).active, lastwp);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn window_pane_get_palette(
    mut wp: *mut window_pane,
    mut c: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if wp.is_null() {
        return -(1 as ::core::ffi::c_int);
    }
    return colour_palette_get(&raw mut (*wp).palette, c);
}
#[no_mangle]
pub unsafe extern "C" fn window_redraw_active_switch(mut w: *mut window, mut wp: *mut window_pane) {
    let mut gc1: *mut grid_cell = ::core::ptr::null_mut::<grid_cell>();
    let mut gc2: *mut grid_cell = ::core::ptr::null_mut::<grid_cell>();
    let mut c1: ::core::ffi::c_int = 0;
    let mut c2: ::core::ffi::c_int = 0;
    if !(*w).modal.is_null() && wp != (*w).modal {
        return;
    }
    if wp == (*w).active {
        return;
    }
    loop {
        gc1 = &raw mut (*wp).cached_gc;
        gc2 = &raw mut (*wp).cached_active_gc;
        if grid_cells_look_equal(gc1, gc2) == 0 {
            (*wp).flags |= PANE_REDRAW;
        } else if (*wp).cached_dim != (*wp).cached_active_dim {
            (*wp).flags |= PANE_REDRAW;
        } else {
            c1 = window_pane_get_palette(wp, (*gc1).fg);
            c2 = window_pane_get_palette(wp, (*gc2).fg);
            if c1 != c2 {
                (*wp).flags |= PANE_REDRAW;
            } else {
                c1 = window_pane_get_palette(wp, (*gc1).bg);
                c2 = window_pane_get_palette(wp, (*gc2).bg);
                if c1 != c2 {
                    (*wp).flags |= PANE_REDRAW;
                }
            }
        }
        if wp == (*w).active {
            break;
        }
        if window_pane_is_floating(wp) != 0 {
            if !(*wp).zentry.tqe_next.is_null() {
                (*(*wp).zentry.tqe_next).zentry.tqe_prev = (*wp).zentry.tqe_prev;
            } else {
                (*w).z_index.tqh_last = (*wp).zentry.tqe_prev;
            }
            *(*wp).zentry.tqe_prev = (*wp).zentry.tqe_next;
            (*wp).zentry.tqe_next = (*w).z_index.tqh_first;
            if !(*wp).zentry.tqe_next.is_null() {
                (*(*w).z_index.tqh_first).zentry.tqe_prev = &raw mut (*wp).zentry.tqe_next;
            } else {
                (*w).z_index.tqh_last = &raw mut (*wp).zentry.tqe_next;
            }
            (*w).z_index.tqh_first = wp;
            (*wp).zentry.tqe_prev = &raw mut (*w).z_index.tqh_first;
            (*wp).flags |= PANE_REDRAW;
            redraw_invalidate_scene(w);
        }
        wp = (*w).active;
        if wp.is_null() {
            break;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn window_get_active_at(
    mut w: *mut window,
    mut x: u_int,
    mut y: u_int,
) -> *mut window_pane {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut pane_status: ::core::ffi::c_int = 0;
    let mut xoff: ::core::ffi::c_int = 0;
    let mut yoff: ::core::ffi::c_int = 0;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    pane_status = window_get_pane_status(w);
    if !(*w).modal.is_null() {
        if window_pane_contains((*w).modal, x, y) != 0 {
            return (*w).modal;
        }
        return ::core::ptr::null_mut::<window_pane>();
    }
    if pane_status == PANE_STATUS_TOP {
        wp = (*w).z_index.tqh_first;
        while !wp.is_null() {
            if !(window_pane_is_visible(wp) == 0 || window_pane_is_floating(wp) != 0) {
                window_pane_full_size_offset(
                    wp,
                    &raw mut xoff,
                    &raw mut yoff,
                    &raw mut sx,
                    &raw mut sy,
                );
                if !((x as ::core::ffi::c_int) < xoff || x > (xoff as u_int).wrapping_add(sx)) {
                    if y as ::core::ffi::c_int == yoff - 1 as ::core::ffi::c_int {
                        return wp;
                    }
                }
            }
            wp = (*wp).zentry.tqe_next;
        }
    }
    let mut current_block_15: u64;
    wp = (*w).z_index.tqh_first;
    while !wp.is_null() {
        if !(window_pane_is_visible(wp) == 0) {
            window_pane_full_size_offset(
                wp,
                &raw mut xoff,
                &raw mut yoff,
                &raw mut sx,
                &raw mut sy,
            );
            if window_pane_is_floating(wp) == 0 {
                if (x as ::core::ffi::c_int) < xoff || x > (xoff as u_int).wrapping_add(sx) {
                    current_block_15 = 12349973810996921269;
                } else if pane_status == PANE_STATUS_TOP {
                    if (y as ::core::ffi::c_int) < yoff - 1 as ::core::ffi::c_int
                        || y > (yoff as u_int).wrapping_add(sy)
                    {
                        current_block_15 = 12349973810996921269;
                    } else {
                        current_block_15 = 8693738493027456495;
                    }
                } else if (y as ::core::ffi::c_int) < yoff || y > (yoff as u_int).wrapping_add(sy) {
                    current_block_15 = 12349973810996921269;
                } else {
                    current_block_15 = 8693738493027456495;
                }
            } else if window_pane_get_pane_lines(wp) as ::core::ffi::c_uint
                == PANE_LINES_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                if (x as ::core::ffi::c_int) < xoff
                    || x as ::core::ffi::c_int >= xoff + sx as ::core::ffi::c_int
                {
                    current_block_15 = 12349973810996921269;
                } else if (y as ::core::ffi::c_int) < yoff
                    || y as ::core::ffi::c_int >= yoff + sy as ::core::ffi::c_int
                {
                    current_block_15 = 12349973810996921269;
                } else {
                    current_block_15 = 8693738493027456495;
                }
            } else if (x as ::core::ffi::c_int) < xoff - 1 as ::core::ffi::c_int
                || x > (xoff as u_int).wrapping_add(sx)
            {
                current_block_15 = 12349973810996921269;
            } else if (y as ::core::ffi::c_int) < yoff - 1 as ::core::ffi::c_int
                || y > (yoff as u_int).wrapping_add(sy)
            {
                current_block_15 = 12349973810996921269;
            } else {
                current_block_15 = 8693738493027456495;
            }
            match current_block_15 {
                12349973810996921269 => {}
                _ => return wp,
            }
        }
        wp = (*wp).zentry.tqe_next;
    }
    return ::core::ptr::null_mut::<window_pane>();
}
#[no_mangle]
pub unsafe extern "C" fn window_find_string(
    mut w: *mut window,
    mut s: *const ::core::ffi::c_char,
) -> *mut window_pane {
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    let mut top: u_int = 0 as u_int;
    let mut bottom: u_int = (*w).sy.wrapping_sub(1 as u_int);
    let mut status: ::core::ffi::c_int = 0;
    x = (*w).sx.wrapping_div(2 as u_int);
    y = (*w).sy.wrapping_div(2 as u_int);
    status = window_get_pane_status(w);
    if status == PANE_STATUS_TOP {
        top = top.wrapping_add(1);
    } else if status == PANE_STATUS_BOTTOM {
        bottom = bottom.wrapping_sub(1);
    }
    if strcasecmp(s, b"top\0" as *const u8 as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int
    {
        y = top;
    } else if strcasecmp(s, b"bottom\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        y = bottom;
    } else if strcasecmp(s, b"left\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        x = 0 as u_int;
    } else if strcasecmp(s, b"right\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        x = (*w).sx.wrapping_sub(1 as u_int);
    } else if strcasecmp(s, b"top-left\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        x = 0 as u_int;
        y = top;
    } else if strcasecmp(s, b"top-right\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        x = (*w).sx.wrapping_sub(1 as u_int);
        y = top;
    } else if strcasecmp(
        s,
        b"bottom-left\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        x = 0 as u_int;
        y = bottom;
    } else if strcasecmp(
        s,
        b"bottom-right\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        x = (*w).sx.wrapping_sub(1 as u_int);
        y = bottom;
    } else {
        return ::core::ptr::null_mut::<window_pane>();
    }
    return window_get_active_at(w, x, y);
}
#[no_mangle]
pub unsafe extern "C" fn window_zoom(mut wp: *mut window_pane) -> ::core::ffi::c_int {
    let mut w: *mut window = (*wp).window as *mut window;
    let mut wp1: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut lc: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lg: layout_geometry = layout_geometry {
        sx: 0,
        sy: 0,
        xoff: 0,
        yoff: 0,
    };
    if (*w).flags & WINDOW_ZOOMED != 0 {
        return -(1 as ::core::ffi::c_int);
    }
    if window_count_panes(w, 1 as ::core::ffi::c_int) == 1 as u_int {
        return -(1 as ::core::ffi::c_int);
    }
    if (*w).active != wp
        && ((*w).active.is_null()
            || !(*(*w).active).flags & PANE_FLOATOVERZOOM != 0
            || window_pane_is_floating((*w).active) == 0)
    {
        window_set_active_pane(w, wp, 1 as ::core::ffi::c_int);
    }
    (*wp).flags |= PANE_ZOOMED;
    wp1 = (*w).panes.tqh_first;
    while !wp1.is_null() {
        (*wp1).saved_layout_cell = (*wp1).layout_cell as *mut layout_cell;
        (*wp1).layout_cell = ::core::ptr::null_mut::<layout_cell>();
        wp1 = (*wp1).entry.tqe_next;
    }
    (*w).saved_layout_root = (*w).layout_root;
    layout_init(w, wp);
    wp1 = (*w).panes.tqh_first;
    while !wp1.is_null() {
        lc = (*wp1).saved_layout_cell;
        if !(wp1 == wp
            || !(*wp1).flags & PANE_FLOATOVERZOOM != 0
            || lc.is_null()
            || !(*lc).flags & LAYOUT_CELL_FLOATING != 0)
        {
            memcpy(
                &raw mut lg as *mut ::core::ffi::c_void,
                &raw mut (*lc).g as *const ::core::ffi::c_void,
                ::core::mem::size_of::<layout_geometry>() as size_t,
            );
            lc = layout_floating_pane(w, wp, &raw mut lg);
            layout_assign_pane(lc, wp1, 0 as ::core::ffi::c_int);
        }
        wp1 = (*wp1).entry.tqe_next;
    }
    if (*(*wp).saved_layout_cell).flags & LAYOUT_CELL_FLOATING != 0 {
        if !(*wp).zentry.tqe_next.is_null() {
            (*(*wp).zentry.tqe_next).zentry.tqe_prev = (*wp).zentry.tqe_prev;
        } else {
            (*w).z_index.tqh_last = (*wp).zentry.tqe_prev;
        }
        *(*wp).zentry.tqe_prev = (*wp).zentry.tqe_next;
        (*wp).zentry.tqe_next = ::core::ptr::null_mut::<window_pane>();
        (*wp).zentry.tqe_prev = (*w).z_index.tqh_last;
        *(*w).z_index.tqh_last = wp;
        (*w).z_index.tqh_last = &raw mut (*wp).zentry.tqe_next;
    }
    (*w).flags |= WINDOW_ZOOMED;
    events_fire_window(
        b"window-zoomed\0" as *const u8 as *const ::core::ffi::c_char,
        w,
    );
    events_fire_window(
        b"window-layout-changed\0" as *const u8 as *const ::core::ffi::c_char,
        w,
    );
    redraw_invalidate_scene(w);
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn window_unzoom(
    mut w: *mut window,
    mut notify: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut zoomed: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut slc: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    if !(*w).flags & WINDOW_ZOOMED != 0 {
        return -(1 as ::core::ffi::c_int);
    }
    wp = (*w).panes.tqh_first;
    while !wp.is_null() {
        if (*wp).flags & PANE_ZOOMED != 0 {
            zoomed = wp;
        }
        if !(!(*wp).flags & PANE_FLOATOVERZOOM != 0) {
            if !((*wp).flags & PANE_ZOOMED != 0) {
                slc = (*wp).saved_layout_cell;
                if !(slc.is_null() || (*wp).layout_cell.is_null()) {
                    memcpy(
                        &raw mut (*slc).g as *mut ::core::ffi::c_void,
                        &raw mut (*(*wp).layout_cell).g as *const ::core::ffi::c_void,
                        ::core::mem::size_of::<layout_geometry>() as size_t,
                    );
                    memcpy(
                        &raw mut (*slc).fg as *mut ::core::ffi::c_void,
                        &raw mut (*(*wp).layout_cell).fg as *const ::core::ffi::c_void,
                        ::core::mem::size_of::<layout_geometry>() as size_t,
                    );
                }
            }
        }
        wp = (*wp).entry.tqe_next;
    }
    (*w).flags &= !WINDOW_ZOOMED;
    layout_free(w, 0 as ::core::ffi::c_int);
    (*w).layout_root = (*w).saved_layout_root;
    (*w).saved_layout_root = ::core::ptr::null_mut::<layout_cell>();
    wp = (*w).panes.tqh_first;
    while !wp.is_null() {
        (*wp).layout_cell = (*wp).saved_layout_cell as *mut layout_cell;
        (*wp).saved_layout_cell = ::core::ptr::null_mut::<layout_cell>();
        (*wp).flags &= !PANE_ZOOMED;
        wp = (*wp).entry.tqe_next;
    }
    if !zoomed.is_null() && window_pane_is_floating(zoomed) != 0 {
        if !(*zoomed).zentry.tqe_next.is_null() {
            (*(*zoomed).zentry.tqe_next).zentry.tqe_prev = (*zoomed).zentry.tqe_prev;
        } else {
            (*w).z_index.tqh_last = (*zoomed).zentry.tqe_prev;
        }
        *(*zoomed).zentry.tqe_prev = (*zoomed).zentry.tqe_next;
        if zoomed == (*w).active {
            (*zoomed).zentry.tqe_next = (*w).z_index.tqh_first;
            if !(*zoomed).zentry.tqe_next.is_null() {
                (*(*w).z_index.tqh_first).zentry.tqe_prev = &raw mut (*zoomed).zentry.tqe_next;
            } else {
                (*w).z_index.tqh_last = &raw mut (*zoomed).zentry.tqe_next;
            }
            (*w).z_index.tqh_first = zoomed;
            (*zoomed).zentry.tqe_prev = &raw mut (*w).z_index.tqh_first;
        } else {
            wp = (*w).z_index.tqh_first;
            while !wp.is_null() {
                if window_pane_is_floating(wp) == 0 {
                    break;
                }
                wp = (*wp).zentry.tqe_next;
            }
            if wp.is_null() {
                (*zoomed).zentry.tqe_next = ::core::ptr::null_mut::<window_pane>();
                (*zoomed).zentry.tqe_prev = (*w).z_index.tqh_last;
                *(*w).z_index.tqh_last = zoomed;
                (*w).z_index.tqh_last = &raw mut (*zoomed).zentry.tqe_next;
            } else {
                (*zoomed).zentry.tqe_prev = (*wp).zentry.tqe_prev;
                (*zoomed).zentry.tqe_next = wp;
                *(*wp).zentry.tqe_prev = zoomed;
                (*wp).zentry.tqe_prev = &raw mut (*zoomed).zentry.tqe_next;
            }
        }
    }
    layout_fix_panes(w, ::core::ptr::null_mut::<window_pane>());
    if notify != 0 {
        events_fire_window(
            b"window-unzoomed\0" as *const u8 as *const ::core::ffi::c_char,
            w,
        );
        events_fire_window(
            b"window-layout-changed\0" as *const u8 as *const ::core::ffi::c_char,
            w,
        );
    }
    redraw_invalidate_scene(w);
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn window_zoomed_pane(mut w: *mut window) -> *mut window_pane {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    if !(*w).flags & WINDOW_ZOOMED != 0 {
        return ::core::ptr::null_mut::<window_pane>();
    }
    wp = *(*((*w).z_index.tqh_last as *mut window_panes_zindex)).tqh_last;
    while !wp.is_null() {
        if !(*wp).layout_cell.is_null() && window_pane_is_floating(wp) == 0 {
            return wp;
        }
        wp = *(*((*wp).zentry.tqe_prev as *mut window_panes_zindex)).tqh_last;
    }
    return ::core::ptr::null_mut::<window_pane>();
}
#[no_mangle]
pub unsafe extern "C" fn window_active_pane_is_over_zoom(mut w: *mut window) -> ::core::ffi::c_int {
    if !(*w).flags & WINDOW_ZOOMED != 0 {
        return 0 as ::core::ffi::c_int;
    }
    if (*w).active.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    if !(*(*w).active).flags & PANE_FLOATOVERZOOM != 0 {
        return 0 as ::core::ffi::c_int;
    }
    return window_pane_is_floating((*w).active);
}
#[no_mangle]
pub unsafe extern "C" fn window_push_zoom(
    mut w: *mut window,
    mut always: ::core::ffi::c_int,
    mut flag: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut wp: *mut window_pane = window_zoomed_pane(w);
    log_debug(
        b"%s: @%u %d\0" as *const u8 as *const ::core::ffi::c_char,
        b"window_push_zoom\0" as *const u8 as *const ::core::ffi::c_char,
        (*w).id,
        (flag != 0 && (*w).flags & WINDOW_ZOOMED != 0) as ::core::ffi::c_int,
    );
    if flag != 0 && (always != 0 || (*w).flags & WINDOW_ZOOMED != 0) {
        (*w).flags |= WINDOW_WASZOOMED;
    } else {
        (*w).flags &= !WINDOW_WASZOOMED;
    }
    if (*w).flags & WINDOW_WASZOOMED != 0 {
        (*w).was_zoomed = wp;
    } else {
        (*w).was_zoomed = ::core::ptr::null_mut::<window_pane>();
    }
    return (window_unzoom(w, 1 as ::core::ffi::c_int) == 0 as ::core::ffi::c_int)
        as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn window_pop_zoom(mut w: *mut window) -> ::core::ffi::c_int {
    let mut wp: *mut window_pane = (*w).was_zoomed;
    log_debug(
        b"%s: @%u %d\0" as *const u8 as *const ::core::ffi::c_char,
        b"window_pop_zoom\0" as *const u8 as *const ::core::ffi::c_char,
        (*w).id,
        ((*w).flags & WINDOW_WASZOOMED != 0) as ::core::ffi::c_int,
    );
    if (*w).flags & WINDOW_WASZOOMED != 0 {
        (*w).flags &= !WINDOW_WASZOOMED;
        (*w).was_zoomed = ::core::ptr::null_mut::<window_pane>();
        if !(*w).active.is_null()
            && (!(*(*w).active).flags & PANE_FLOATOVERZOOM != 0
                || window_pane_is_floating((*w).active) == 0)
        {
            wp = (*w).active;
        }
        if wp.is_null() || window_has_pane(w, wp) == 0 {
            wp = (*w).active;
        }
        if !wp.is_null() {
            return (window_zoom(wp) == 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
        }
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn window_add_pane(
    mut w: *mut window,
    mut other: *mut window_pane,
    mut hlimit: u_int,
    mut flags: ::core::ffi::c_int,
) -> *mut window_pane {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    if other.is_null() {
        other = (*w).active;
    }
    wp = window_pane_create(w, (*w).sx, (*w).sy, hlimit);
    if (*w).panes.tqh_first.is_null() {
        log_debug(
            b"%s: @%u at start\0" as *const u8 as *const ::core::ffi::c_char,
            b"window_add_pane\0" as *const u8 as *const ::core::ffi::c_char,
            (*w).id,
        );
        (*wp).entry.tqe_next = (*w).panes.tqh_first;
        if !(*wp).entry.tqe_next.is_null() {
            (*(*w).panes.tqh_first).entry.tqe_prev = &raw mut (*wp).entry.tqe_next;
        } else {
            (*w).panes.tqh_last = &raw mut (*wp).entry.tqe_next;
        }
        (*w).panes.tqh_first = wp;
        (*wp).entry.tqe_prev = &raw mut (*w).panes.tqh_first;
    } else if flags & SPAWN_BEFORE != 0 {
        log_debug(
            b"%s: @%u before %%%u\0" as *const u8 as *const ::core::ffi::c_char,
            b"window_add_pane\0" as *const u8 as *const ::core::ffi::c_char,
            (*w).id,
            (*wp).id,
        );
        if flags & SPAWN_FULLSIZE != 0 {
            (*wp).entry.tqe_next = (*w).panes.tqh_first;
            if !(*wp).entry.tqe_next.is_null() {
                (*(*w).panes.tqh_first).entry.tqe_prev = &raw mut (*wp).entry.tqe_next;
            } else {
                (*w).panes.tqh_last = &raw mut (*wp).entry.tqe_next;
            }
            (*w).panes.tqh_first = wp;
            (*wp).entry.tqe_prev = &raw mut (*w).panes.tqh_first;
        } else {
            (*wp).entry.tqe_prev = (*other).entry.tqe_prev;
            (*wp).entry.tqe_next = other;
            *(*other).entry.tqe_prev = wp;
            (*other).entry.tqe_prev = &raw mut (*wp).entry.tqe_next;
        }
    } else {
        log_debug(
            b"%s: @%u after %%%u\0" as *const u8 as *const ::core::ffi::c_char,
            b"window_add_pane\0" as *const u8 as *const ::core::ffi::c_char,
            (*w).id,
            (*wp).id,
        );
        if flags & (SPAWN_FULLSIZE | SPAWN_FLOATING) != 0 {
            (*wp).entry.tqe_next = ::core::ptr::null_mut::<window_pane>();
            (*wp).entry.tqe_prev = (*w).panes.tqh_last;
            *(*w).panes.tqh_last = wp;
            (*w).panes.tqh_last = &raw mut (*wp).entry.tqe_next;
        } else {
            (*wp).entry.tqe_next = (*other).entry.tqe_next;
            if !(*wp).entry.tqe_next.is_null() {
                (*(*wp).entry.tqe_next).entry.tqe_prev = &raw mut (*wp).entry.tqe_next;
            } else {
                (*w).panes.tqh_last = &raw mut (*wp).entry.tqe_next;
            }
            (*other).entry.tqe_next = wp;
            (*wp).entry.tqe_prev = &raw mut (*other).entry.tqe_next;
        }
    }
    if !flags & SPAWN_FLOATING != 0 {
        (*wp).zentry.tqe_next = ::core::ptr::null_mut::<window_pane>();
        (*wp).zentry.tqe_prev = (*w).z_index.tqh_last;
        *(*w).z_index.tqh_last = wp;
        (*w).z_index.tqh_last = &raw mut (*wp).zentry.tqe_next;
    } else if !(*w).modal.is_null() {
        (*wp).zentry.tqe_next = (*(*w).modal).zentry.tqe_next;
        if !(*wp).zentry.tqe_next.is_null() {
            (*(*wp).zentry.tqe_next).zentry.tqe_prev = &raw mut (*wp).zentry.tqe_next;
        } else {
            (*w).z_index.tqh_last = &raw mut (*wp).zentry.tqe_next;
        }
        (*(*w).modal).zentry.tqe_next = wp;
        (*wp).zentry.tqe_prev = &raw mut (*(*w).modal).zentry.tqe_next;
    } else {
        (*wp).zentry.tqe_next = (*w).z_index.tqh_first;
        if !(*wp).zentry.tqe_next.is_null() {
            (*(*w).z_index.tqh_first).zentry.tqe_prev = &raw mut (*wp).zentry.tqe_next;
        } else {
            (*w).z_index.tqh_last = &raw mut (*wp).zentry.tqe_next;
        }
        (*w).z_index.tqh_first = wp;
        (*wp).zentry.tqe_prev = &raw mut (*w).z_index.tqh_first;
    }
    redraw_invalidate_scene(w);
    return wp;
}
#[no_mangle]
pub unsafe extern "C" fn window_lost_pane(mut w: *mut window, mut wp: *mut window_pane) {
    let mut lastwp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    log_debug(
        b"%s: @%u pane %%%u\0" as *const u8 as *const ::core::ffi::c_char,
        b"window_lost_pane\0" as *const u8 as *const ::core::ffi::c_char,
        (*w).id,
        (*wp).id,
    );
    if wp == marked_pane.wp {
        server_clear_marked();
    }
    if wp == (*w).modal_last {
        (*w).modal_last = ::core::ptr::null_mut::<window_pane>();
    }
    if wp == (*w).was_zoomed {
        (*w).was_zoomed = ::core::ptr::null_mut::<window_pane>();
    }
    window_pane_stack_remove(&raw mut (*w).last_panes, wp);
    if wp == (*w).active {
        lastwp = ::core::ptr::null_mut::<window_pane>();
        if wp == (*w).modal {
            lastwp = (*w).modal_last;
            (*w).modal = ::core::ptr::null_mut::<window_pane>();
            (*w).modal_last = ::core::ptr::null_mut::<window_pane>();
        }
        if !lastwp.is_null() && window_has_pane(w, lastwp) != 0 {
            (*w).active = lastwp;
        } else {
            (*w).active = (*w).last_panes.tqh_first;
        }
        if (*w).active.is_null() {
            (*w).active = *(*((*wp).entry.tqe_prev as *mut window_panes)).tqh_last;
            if (*w).active.is_null() {
                (*w).active = (*wp).entry.tqe_next;
            }
        }
        if !(*w).active.is_null() {
            window_pane_stack_remove(&raw mut (*w).last_panes, (*w).active);
            (*(*w).active).flags |= PANE_CHANGED;
            window_fire_pane_changed(w, (*w).active, wp);
            window_update_focus(w);
        }
    } else if wp == (*w).modal {
        (*w).modal_last = ::core::ptr::null_mut::<window_pane>();
        (*w).modal = (*w).modal_last;
    }
    redraw_invalidate_scene(w);
}
#[no_mangle]
pub unsafe extern "C" fn window_remove_pane(mut w: *mut window, mut wp: *mut window_pane) {
    window_lost_pane(w, wp);
    if !(*wp).entry.tqe_next.is_null() {
        (*(*wp).entry.tqe_next).entry.tqe_prev = (*wp).entry.tqe_prev;
    } else {
        (*w).panes.tqh_last = (*wp).entry.tqe_prev;
    }
    *(*wp).entry.tqe_prev = (*wp).entry.tqe_next;
    if !(*wp).zentry.tqe_next.is_null() {
        (*(*wp).zentry.tqe_next).zentry.tqe_prev = (*wp).zentry.tqe_prev;
    } else {
        (*w).z_index.tqh_last = (*wp).zentry.tqe_prev;
    }
    *(*wp).zentry.tqe_prev = (*wp).zentry.tqe_next;
    redraw_invalidate_scene(w);
    window_pane_destroy(wp);
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_at_index(
    mut w: *mut window,
    mut idx: u_int,
) -> *mut window_pane {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut n: u_int = 0;
    n = options_get_number(
        (*w).options,
        b"pane-base-index\0" as *const u8 as *const ::core::ffi::c_char,
    ) as u_int;
    wp = (*w).panes.tqh_first;
    while !wp.is_null() {
        if n == idx {
            return wp;
        }
        n = n.wrapping_add(1);
        wp = (*wp).entry.tqe_next;
    }
    return ::core::ptr::null_mut::<window_pane>();
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_next_by_number(
    mut w: *mut window,
    mut wp: *mut window_pane,
    mut n: u_int,
) -> *mut window_pane {
    while n > 0 as u_int {
        wp = (*wp).entry.tqe_next;
        if wp.is_null() {
            wp = (*w).panes.tqh_first;
        }
        n = n.wrapping_sub(1);
    }
    return wp;
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_previous_by_number(
    mut w: *mut window,
    mut wp: *mut window_pane,
    mut n: u_int,
) -> *mut window_pane {
    while n > 0 as u_int {
        wp = *(*((*wp).entry.tqe_prev as *mut window_panes)).tqh_last;
        if wp.is_null() {
            wp = *(*((*w).panes.tqh_last as *mut window_panes)).tqh_last;
        }
        n = n.wrapping_sub(1);
    }
    return wp;
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_index(
    mut wp: *mut window_pane,
    mut i: *mut u_int,
) -> ::core::ffi::c_int {
    let mut w: *mut window = (*wp).window as *mut window;
    let mut wq: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    *i = options_get_number(
        (*w).options,
        b"pane-base-index\0" as *const u8 as *const ::core::ffi::c_char,
    ) as u_int;
    wq = (*w).panes.tqh_first;
    while !wq.is_null() {
        if wp == wq {
            return 0 as ::core::ffi::c_int;
        }
        *i = (*i).wrapping_add(1);
        wq = (*wq).entry.tqe_next;
    }
    return -(1 as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_zindex(
    mut wp: *mut window_pane,
    mut i: *mut u_int,
) -> ::core::ffi::c_int {
    let mut w: *mut window = (*wp).window as *mut window;
    let mut wq: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    *i = 0 as u_int;
    wq = (*w).z_index.tqh_first;
    while !wq.is_null() {
        if wq == wp {
            if window_pane_is_floating(wp) == 0 {
                *i = (*i).wrapping_add(1);
            }
            return 0 as ::core::ffi::c_int;
        }
        if window_pane_is_floating(wq) != 0 {
            *i = (*i).wrapping_add(1);
        }
        wq = (*wq).zentry.tqe_next;
    }
    return -(1 as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_last_index(
    mut wp: *mut window_pane,
    mut i: *mut u_int,
) -> ::core::ffi::c_int {
    let mut w: *mut window = (*wp).window as *mut window;
    let mut wq: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    *i = 0 as u_int;
    wq = (*w).last_panes.tqh_first;
    while !wq.is_null() {
        if wq == wp {
            return 0 as ::core::ffi::c_int;
        }
        *i = (*i).wrapping_add(1);
        wq = (*wq).sentry.tqe_next;
    }
    return -(1 as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn window_count_panes(
    mut w: *mut window,
    mut with_floating: ::core::ffi::c_int,
) -> u_int {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut n: u_int = 0 as u_int;
    wp = (*w).panes.tqh_first;
    while !wp.is_null() {
        if with_floating != 0 || window_pane_is_floating(wp) == 0 {
            n = n.wrapping_add(1);
        }
        wp = (*wp).entry.tqe_next;
    }
    return n;
}
#[no_mangle]
pub unsafe extern "C" fn window_destroy_panes(mut w: *mut window) {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    while !(*w).last_panes.tqh_first.is_null() {
        wp = (*w).last_panes.tqh_first;
        window_pane_stack_remove(&raw mut (*w).last_panes, wp);
    }
    while !(*w).panes.tqh_first.is_null() {
        wp = (*w).panes.tqh_first;
        if !(*wp).entry.tqe_next.is_null() {
            (*(*wp).entry.tqe_next).entry.tqe_prev = (*wp).entry.tqe_prev;
        } else {
            (*w).panes.tqh_last = (*wp).entry.tqe_prev;
        }
        *(*wp).entry.tqe_prev = (*wp).entry.tqe_next;
        if !(*wp).zentry.tqe_next.is_null() {
            (*(*wp).zentry.tqe_next).zentry.tqe_prev = (*wp).zentry.tqe_prev;
        } else {
            (*w).z_index.tqh_last = (*wp).zentry.tqe_prev;
        }
        *(*wp).zentry.tqe_prev = (*wp).zentry.tqe_next;
        window_pane_destroy(wp);
    }
}
#[no_mangle]
pub unsafe extern "C" fn window_printable_flags(
    mut wl: *mut winlink,
    mut escape: ::core::ffi::c_int,
) -> *const ::core::ffi::c_char {
    let mut s: *mut session = (*wl).session;
    static mut flags: [::core::ffi::c_char; 32] = [0; 32];
    let mut pos: u_int = 0 as u_int;
    if (*wl).flags & WINLINK_ACTIVITY != 0 {
        let fresh3 = pos;
        pos = pos.wrapping_add(1);
        flags[fresh3 as usize] = '#' as i32 as ::core::ffi::c_char;
        if escape != 0 {
            let fresh4 = pos;
            pos = pos.wrapping_add(1);
            flags[fresh4 as usize] = '#' as i32 as ::core::ffi::c_char;
        }
    }
    if (*wl).flags & WINLINK_BELL != 0 {
        let fresh5 = pos;
        pos = pos.wrapping_add(1);
        flags[fresh5 as usize] = '!' as i32 as ::core::ffi::c_char;
    }
    if (*wl).flags & WINLINK_SILENCE != 0 {
        let fresh6 = pos;
        pos = pos.wrapping_add(1);
        flags[fresh6 as usize] = '~' as i32 as ::core::ffi::c_char;
    }
    if wl == (*s).curw {
        let fresh7 = pos;
        pos = pos.wrapping_add(1);
        flags[fresh7 as usize] = '*' as i32 as ::core::ffi::c_char;
    }
    if wl == (*s).lastw.tqh_first {
        let fresh8 = pos;
        pos = pos.wrapping_add(1);
        flags[fresh8 as usize] = '-' as i32 as ::core::ffi::c_char;
    }
    if server_check_marked() != 0 && wl == marked_pane.wl {
        let fresh9 = pos;
        pos = pos.wrapping_add(1);
        flags[fresh9 as usize] = 'M' as i32 as ::core::ffi::c_char;
    }
    if !(*(*wl).window).modal.is_null() {
        let fresh10 = pos;
        pos = pos.wrapping_add(1);
        flags[fresh10 as usize] = 'O' as i32 as ::core::ffi::c_char;
    }
    if (*(*wl).window).flags & WINDOW_ZOOMED != 0 {
        let fresh11 = pos;
        pos = pos.wrapping_add(1);
        flags[fresh11 as usize] = 'Z' as i32 as ::core::ffi::c_char;
    }
    flags[pos as usize] = '\0' as i32 as ::core::ffi::c_char;
    return &raw mut flags as *mut ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_printable_flags(
    mut wp: *mut window_pane,
) -> *const ::core::ffi::c_char {
    let mut w: *mut window = (*wp).window as *mut window;
    static mut flags: [::core::ffi::c_char; 32] = [0; 32];
    let mut pos: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if wp == (*w).active {
        let fresh12 = pos;
        pos = pos + 1;
        flags[fresh12 as usize] = '*' as i32 as ::core::ffi::c_char;
    }
    if wp == (*w).last_panes.tqh_first {
        let fresh13 = pos;
        pos = pos + 1;
        flags[fresh13 as usize] = '-' as i32 as ::core::ffi::c_char;
    }
    if (*wp).flags & PANE_ZOOMED != 0 {
        let fresh14 = pos;
        pos = pos + 1;
        flags[fresh14 as usize] = 'Z' as i32 as ::core::ffi::c_char;
    }
    if window_pane_is_floating(wp) != 0 {
        let fresh15 = pos;
        pos = pos + 1;
        flags[fresh15 as usize] = 'F' as i32 as ::core::ffi::c_char;
    }
    if (*wp).flags & PANE_FLOATOVERZOOM != 0 {
        let fresh16 = pos;
        pos = pos + 1;
        flags[fresh16 as usize] = 'A' as i32 as ::core::ffi::c_char;
    }
    if wp == (*w).modal {
        let fresh17 = pos;
        pos = pos + 1;
        flags[fresh17 as usize] = 'O' as i32 as ::core::ffi::c_char;
    }
    flags[pos as usize] = '\0' as i32 as ::core::ffi::c_char;
    return &raw mut flags as *mut ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_find_by_id_str(
    mut s: *const ::core::ffi::c_char,
) -> *mut window_pane {
    let mut errstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut id: u_int = 0;
    if *s as ::core::ffi::c_int != '%' as i32 {
        return ::core::ptr::null_mut::<window_pane>();
    }
    id = strtonum(
        s.offset(1 as ::core::ffi::c_int as isize),
        0 as ::core::ffi::c_longlong,
        UINT_MAX as ::core::ffi::c_longlong,
        &raw mut errstr,
    ) as u_int;
    if !errstr.is_null() {
        return ::core::ptr::null_mut::<window_pane>();
    }
    return window_pane_find_by_id(id);
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_find_by_id(mut id: u_int) -> *mut window_pane {
    let mut wp: window_pane = window_pane {
        id: 0,
        references: 0,
        active_point: 0,
        window: ::core::ptr::null_mut::<window>(),
        options: ::core::ptr::null_mut::<options>(),
        layout_cell: ::core::ptr::null_mut::<layout_cell>(),
        saved_layout_cell: ::core::ptr::null_mut::<layout_cell>(),
        sx: 0,
        sy: 0,
        xoff: 0,
        yoff: 0,
        flags: 0,
        sync_dirty: ::core::ptr::null_mut::<bitstr_t>(),
        sync_dirty_size: 0,
        sb_slider_y: 0,
        sb_slider_h: 0,
        sb_auto_visible: 0,
        sb_auto_hover: 0,
        sb_auto_timer: event::ZERO,
        argc: 0,
        argv: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        shell: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        cwd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        pid: 0,
        tty: [0; 32],
        status: 0,
        dead_time: timeval {
            tv_sec: 0,
            tv_usec: 0,
        },
        wait_item: ::core::ptr::null_mut::<cmdq_item>(),
        editor: ::core::ptr::null_mut::<spawn_editor_state>(),
        output_generation: 0,
        last_output_time: 0,
        last_prompt_time: 0,
        cmd_start_time: 0,
        cmd_end_time: 0,
        cmd_status: 0,
        fd: 0,
        event: ::core::ptr::null_mut::<bufferevent>(),
        offset: window_pane_offset { used: 0 },
        base_offset: 0,
        resize_queue: window_pane_resizes {
            tqh_first: ::core::ptr::null_mut::<window_pane_resize>(),
            tqh_last: ::core::ptr::null_mut::<*mut window_pane_resize>(),
        },
        resize_timer: event::ZERO,
        sync_timer: event::ZERO,
        ictx: ::core::ptr::null_mut::<input_ctx>(),
        cached_gc: grid_cell {
            data: utf8_data {
                data: [0; 32],
                have: 0,
                size: 0,
                width: 0,
            },
            attr: 0,
            flags: 0,
            fg: 0,
            bg: 0,
            us: 0,
            link: 0,
        },
        cached_active_gc: grid_cell {
            data: utf8_data {
                data: [0; 32],
                have: 0,
                size: 0,
                width: 0,
            },
            attr: 0,
            flags: 0,
            fg: 0,
            bg: 0,
            us: 0,
            link: 0,
        },
        cached_dim: 0,
        cached_active_dim: 0,
        palette: colour_palette {
            fg: 0,
            bg: 0,
            palette: ::core::ptr::null_mut::<::core::ffi::c_int>(),
            default_palette: ::core::ptr::null_mut::<::core::ffi::c_int>(),
        },
        last_theme: THEME_UNKNOWN,
        border_status_line: style_line_entry {
            expanded: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ranges: style_ranges {
                tqh_first: ::core::ptr::null_mut::<style_range>(),
                tqh_last: ::core::ptr::null_mut::<*mut style_range>(),
            },
        },
        pipe_fd: 0,
        pipe_pid: 0,
        pipe_event: ::core::ptr::null_mut::<bufferevent>(),
        pipe_offset: window_pane_offset { used: 0 },
        screen: ::core::ptr::null_mut::<screen>(),
        base: screen {
            title: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            path: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            titles: ::core::ptr::null_mut::<screen_titles>(),
            ntitles: 0,
            grid: ::core::ptr::null_mut::<grid>(),
            cx: 0,
            cy: 0,
            cstyle: SCREEN_CURSOR_DEFAULT,
            default_cstyle: SCREEN_CURSOR_DEFAULT,
            ccolour: 0,
            default_ccolour: 0,
            rupper: 0,
            rlower: 0,
            mode: 0,
            default_mode: 0,
            saved_cx: 0,
            saved_cy: 0,
            saved_grid: ::core::ptr::null_mut::<grid>(),
            saved_cell: grid_cell {
                data: utf8_data {
                    data: [0; 32],
                    have: 0,
                    size: 0,
                    width: 0,
                },
                attr: 0,
                flags: 0,
                fg: 0,
                bg: 0,
                us: 0,
                link: 0,
            },
            saved_flags: 0,
            tabs: ::core::ptr::null_mut::<bitstr_t>(),
            sel: ::core::ptr::null_mut::<screen_sel>(),
            write_list: ::core::ptr::null_mut::<screen_write_cline>(),
            hyperlinks: ::core::ptr::null_mut::<hyperlinks>(),
            progress_bar: progress_bar {
                state: PROGRESS_BAR_HIDDEN,
                progress: 0,
            },
        },
        status_screen: screen {
            title: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            path: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            titles: ::core::ptr::null_mut::<screen_titles>(),
            ntitles: 0,
            grid: ::core::ptr::null_mut::<grid>(),
            cx: 0,
            cy: 0,
            cstyle: SCREEN_CURSOR_DEFAULT,
            default_cstyle: SCREEN_CURSOR_DEFAULT,
            ccolour: 0,
            default_ccolour: 0,
            rupper: 0,
            rlower: 0,
            mode: 0,
            default_mode: 0,
            saved_cx: 0,
            saved_cy: 0,
            saved_grid: ::core::ptr::null_mut::<grid>(),
            saved_cell: grid_cell {
                data: utf8_data {
                    data: [0; 32],
                    have: 0,
                    size: 0,
                    width: 0,
                },
                attr: 0,
                flags: 0,
                fg: 0,
                bg: 0,
                us: 0,
                link: 0,
            },
            saved_flags: 0,
            tabs: ::core::ptr::null_mut::<bitstr_t>(),
            sel: ::core::ptr::null_mut::<screen_sel>(),
            write_list: ::core::ptr::null_mut::<screen_write_cline>(),
            hyperlinks: ::core::ptr::null_mut::<hyperlinks>(),
            progress_bar: progress_bar {
                state: PROGRESS_BAR_HIDDEN,
                progress: 0,
            },
        },
        modes: window_pane_modes {
            tqh_first: ::core::ptr::null_mut::<window_mode_entry>(),
            tqh_last: ::core::ptr::null_mut::<*mut window_mode_entry>(),
        },
        searchstr: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        searchregex: 0,
        prompt: ::core::ptr::null_mut::<prompt>(),
        prompt_data: ::core::ptr::null_mut::<window_pane_prompt>(),
        prompt_cx: 0,
        border_gc_set: 0,
        border_gc: grid_cell {
            data: utf8_data {
                data: [0; 32],
                have: 0,
                size: 0,
                width: 0,
            },
            attr: 0,
            flags: 0,
            fg: 0,
            bg: 0,
            us: 0,
            link: 0,
        },
        active_border_gc_set: 0,
        active_border_gc: grid_cell {
            data: utf8_data {
                data: [0; 32],
                have: 0,
                size: 0,
                width: 0,
            },
            attr: 0,
            flags: 0,
            fg: 0,
            bg: 0,
            us: 0,
            link: 0,
        },
        control_bg: 0,
        control_fg: 0,
        scrollbar_style: style {
            gc: grid_cell {
                data: utf8_data {
                    data: [0; 32],
                    have: 0,
                    size: 0,
                    width: 0,
                },
                attr: 0,
                flags: 0,
                fg: 0,
                bg: 0,
                us: 0,
                link: 0,
            },
            ignore: 0,
            dim: 0,
            fill: 0,
            align: STYLE_ALIGN_DEFAULT,
            list: STYLE_LIST_OFF,
            range_type: STYLE_RANGE_NONE,
            range_argument: 0,
            range_string: [0; 16],
            width: 0,
            width_percentage: 0,
            pad: 0,
            default_type: STYLE_DEFAULT_BASE,
            link: 0,
        },
        r: visible_ranges {
            ranges: ::core::ptr::null_mut::<visible_range>(),
            used: 0,
            size: 0,
        },
        entry: window_pane_entry {
            tqe_next: ::core::ptr::null_mut::<window_pane>(),
            tqe_prev: ::core::ptr::null_mut::<*mut window_pane>(),
        },
        sentry: window_pane_sentry {
            tqe_next: ::core::ptr::null_mut::<window_pane>(),
            tqe_prev: ::core::ptr::null_mut::<*mut window_pane>(),
        },
        zentry: window_pane_zentry {
            tqe_next: ::core::ptr::null_mut::<window_pane>(),
            tqe_prev: ::core::ptr::null_mut::<*mut window_pane>(),
        },
        tree_entry: window_pane_tree_entry {
            rbe_left: ::core::ptr::null_mut::<window_pane>(),
            rbe_right: ::core::ptr::null_mut::<window_pane>(),
            rbe_parent: ::core::ptr::null_mut::<window_pane>(),
            rbe_color: 0,
        },
    };
    wp.id = id;
    return window_pane_tree_RB_FIND(&raw mut all_window_panes, &raw mut wp);
}
unsafe extern "C" fn window_pane_create(
    mut w: *mut window,
    mut sx: u_int,
    mut sy: u_int,
    mut hlimit: u_int,
) -> *mut window_pane {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut host: [::core::ffi::c_char; 65] = [0; 65];
    wp = xcalloc(1 as size_t, ::core::mem::size_of::<window_pane>() as size_t) as *mut window_pane;
    (*wp).references = 1 as ::core::ffi::c_int;
    (*wp).window = w as *mut window;
    (*wp).options = options_create((*w).options);
    (*wp).flags = PANE_STYLECHANGED;
    (*wp).cmd_status = -(1 as ::core::ffi::c_int);
    let fresh2 = next_window_pane_id;
    next_window_pane_id = next_window_pane_id.wrapping_add(1);
    (*wp).id = fresh2;
    window_pane_tree_RB_INSERT(&raw mut all_window_panes, wp);
    (*wp).fd = -(1 as ::core::ffi::c_int);
    (*wp).modes.tqh_first = ::core::ptr::null_mut::<window_mode_entry>();
    (*wp).modes.tqh_last = &raw mut (*wp).modes.tqh_first;
    (*wp).resize_queue.tqh_first = ::core::ptr::null_mut::<window_pane_resize>();
    (*wp).resize_queue.tqh_last = &raw mut (*wp).resize_queue.tqh_first;
    (*wp).sx = sx;
    (*wp).sy = sy;
    (*wp).pipe_fd = -(1 as ::core::ffi::c_int);
    (*wp).control_bg = -(1 as ::core::ffi::c_int);
    (*wp).control_fg = -(1 as ::core::ffi::c_int);
    style_set_scrollbar_style_from_option(&raw mut (*wp).scrollbar_style, (*wp).options);
    colour_palette_init(&raw mut (*wp).palette);
    colour_palette_from_option(&raw mut (*wp).palette, (*wp).options);
    screen_init(&raw mut (*wp).base, sx, sy, hlimit);
    (*wp).screen = &raw mut (*wp).base;
    window_pane_default_cursor(wp);
    screen_init(
        &raw mut (*wp).status_screen,
        1 as u_int,
        1 as u_int,
        0 as u_int,
    );
    style_ranges_init(&raw mut (*wp).border_status_line.ranges);
    event_set(
        &raw mut (*wp).sb_auto_timer,
        -(1 as ::core::ffi::c_int),
        0 as ::core::ffi::c_short,
        Some(
            window_pane_scrollbar_timer
                as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    ::core::ffi::c_short,
                    *mut ::core::ffi::c_void,
                ) -> (),
        ),
        wp as *mut ::core::ffi::c_void,
    );
    if gethostname(
        &raw mut host as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 65]>() as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        screen_set_title(
            &raw mut (*wp).base,
            &raw mut host as *mut ::core::ffi::c_char,
            0 as ::core::ffi::c_int,
        );
    }
    return wp;
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_wait_finish(mut wp: *mut window_pane) {
    let mut item: *mut cmdq_item = (*wp).wait_item;
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut retval: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if item.is_null() {
        return;
    }
    (*wp).wait_item = ::core::ptr::null_mut::<cmdq_item>();
    if (*wp).flags & PANE_STATUSREADY != 0 {
        if (*wp).status & 0x7f as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            retval = ((*wp).status & 0xff00 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int;
        } else if (((*wp).status & 0x7f as ::core::ffi::c_int) + 1 as ::core::ffi::c_int)
            as ::core::ffi::c_schar as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int
            > 0 as ::core::ffi::c_int
        {
            retval = ((*wp).status & 0x7f as ::core::ffi::c_int) + 128 as ::core::ffi::c_int;
        }
    }
    c = cmdq_get_client(item);
    if !c.is_null() && (*c).session.is_null() {
        (*c).retval = retval;
    }
    cmdq_continue(item);
}
unsafe extern "C" fn window_pane_free_modes(mut wp: *mut window_pane) {
    let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
    while !(*wp).modes.tqh_first.is_null() {
        wme = (*wp).modes.tqh_first;
        if !(*wme).entry.tqe_next.is_null() {
            (*(*wme).entry.tqe_next).entry.tqe_prev = (*wme).entry.tqe_prev;
        } else {
            (*wp).modes.tqh_last = (*wme).entry.tqe_prev;
        }
        *(*wme).entry.tqe_prev = (*wme).entry.tqe_next;
        (*(*wme).mode).free.expect("non-null function pointer")(wme);
        free(wme as *mut ::core::ffi::c_void);
    }
    (*wp).screen = &raw mut (*wp).base;
}
unsafe extern "C" fn window_pane_scrollbar_timer(
    mut fd: ::core::ffi::c_int,
    mut events: ::core::ffi::c_short,
    mut arg: *mut ::core::ffi::c_void,
) {
    let mut wp: *mut window_pane = arg as *mut window_pane;
    (*wp).sb_auto_hover = 0 as ::core::ffi::c_int;
    window_pane_scrollbar_hide(wp);
}
unsafe extern "C" fn window_pane_scrollbar_auto_hide(
    mut wp: *mut window_pane,
) -> ::core::ffi::c_int {
    return ((*(*wp).window).sb == PANE_SCROLLBARS_MODAL
        || (*(*wp).window).sb == PANE_SCROLLBARS_AUTOHIDE) as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_scrollbar_overlay_visible(
    mut wp: *mut window_pane,
) -> ::core::ffi::c_int {
    return (window_pane_scrollbar_overlay(wp) != 0 && window_pane_scrollbar_visible(wp) != 0)
        as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_scrollbar_redraw(mut wp: *mut window_pane) {
    if window_pane_scrollbar_visible(wp) == 0 {
        return;
    }
    if window_pane_scrollbar_overlay_visible(wp) != 0 {
        (*wp).flags |= PANE_REDRAW;
        return;
    }
    (*wp).flags |= PANE_REDRAWSCROLLBAR;
}
unsafe extern "C" fn window_pane_scrollbar_redraw_visibility(mut wp: *mut window_pane) {
    redraw_invalidate_scene((*wp).window as *mut window);
    (*wp).flags |= PANE_REDRAW;
    server_redraw_window((*wp).window as *mut window);
}
unsafe extern "C" fn window_pane_destroy(mut wp: *mut window_pane) {
    window_pane_wait_finish(wp);
    spawn_editor_finish(wp);
    window_pane_tree_RB_REMOVE(&raw mut all_window_panes, wp);
    (*wp).flags |= PANE_DESTROYED;
    window_pane_clear_prompt(wp);
    window_pane_free_modes(wp);
    screen_write_clear_dirty(wp);
    if (*wp).fd != -(1 as ::core::ffi::c_int) {
        utempter_remove_record((*wp).fd);
        kill(getpid(), SIGCHLD);
        bufferevent_free((*wp).event);
        (*wp).event = ::core::ptr::null_mut::<bufferevent>();
        close((*wp).fd);
        (*wp).fd = -(1 as ::core::ffi::c_int);
    }
    if !(*wp).ictx.is_null() {
        input_free((*wp).ictx);
        (*wp).ictx = ::core::ptr::null_mut::<input_ctx>();
    }
    if (*wp).pipe_fd != -(1 as ::core::ffi::c_int) {
        bufferevent_free((*wp).pipe_event);
        (*wp).pipe_event = ::core::ptr::null_mut::<bufferevent>();
        close((*wp).pipe_fd);
        (*wp).pipe_fd = -(1 as ::core::ffi::c_int);
    }
    if event_initialized(&raw mut (*wp).resize_timer) != 0 {
        event_del(&raw mut (*wp).resize_timer);
    }
    if event_initialized(&raw mut (*wp).sync_timer) != 0 {
        event_del(&raw mut (*wp).sync_timer);
    }
    if event_initialized(&raw mut (*wp).sb_auto_timer) != 0 {
        event_del(&raw mut (*wp).sb_auto_timer);
    }
    window_pane_clear_resizes(wp, ::core::ptr::null_mut::<window_pane_resize>());
    window_pane_remove_ref(
        wp,
        b"window_pane_destroy\0" as *const u8 as *const ::core::ffi::c_char,
    );
}
unsafe extern "C" fn window_pane_free(mut wp: *mut window_pane) {
    log_debug(
        b"pane %%%u freed (%d references)\0" as *const u8 as *const ::core::ffi::c_char,
        (*wp).id,
        (*wp).references,
    );
    free((*wp).searchstr as *mut ::core::ffi::c_void);
    screen_free(&raw mut (*wp).status_screen);
    screen_free(&raw mut (*wp).base);
    free((*wp).r.ranges as *mut ::core::ffi::c_void);
    options_free((*wp).options);
    free((*wp).cwd as *mut ::core::ffi::c_void);
    free((*wp).shell as *mut ::core::ffi::c_void);
    cmd_free_argv((*wp).argc, (*wp).argv);
    colour_palette_free(&raw mut (*wp).palette);
    style_ranges_free(&raw mut (*wp).border_status_line.ranges);
    free((*wp).border_status_line.expanded as *mut ::core::ffi::c_void);
    free(wp as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn window_pane_read_callback(
    mut bufev: *mut bufferevent,
    mut data: *mut ::core::ffi::c_void,
) {
    let mut wp: *mut window_pane = data as *mut window_pane;
    let mut evb: *mut evbuffer = crate::src::reactor::stream_input((*wp).event);
    let mut wpo: *mut window_pane_offset = &raw mut (*wp).pipe_offset;
    let mut size: size_t = evbuffer_get_length(evb);
    let mut new_data: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut new_size: size_t = 0;
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    if (*wp).pipe_fd != -(1 as ::core::ffi::c_int) {
        new_data = window_pane_get_new_data(wp, wpo, &raw mut new_size) as *mut ::core::ffi::c_char;
        if new_size > 0 as size_t {
            bufferevent_write(
                (*wp).pipe_event,
                new_data as *const ::core::ffi::c_void,
                new_size,
            );
            window_pane_update_used_data(wp, wpo, new_size);
        }
    }
    log_debug(
        b"%%%u has %zu bytes\0" as *const u8 as *const ::core::ffi::c_char,
        (*wp).id,
        size,
    );
    c = clients.tqh_first;
    while !c.is_null() {
        if !(*c).session.is_null() && (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
            control_write_output(c, wp);
        }
        c = (*c).entry.tqe_next;
    }
    input_parse_pane(wp);
    bufferevent_disable((*wp).event, EV_READ as ::core::ffi::c_short);
}
unsafe extern "C" fn window_pane_error_callback(
    mut bufev: *mut bufferevent,
    mut what: ::core::ffi::c_short,
    mut data: *mut ::core::ffi::c_void,
) {
    let mut wp: *mut window_pane = data as *mut window_pane;
    log_debug(
        b"%%%u error\0" as *const u8 as *const ::core::ffi::c_char,
        (*wp).id,
    );
    (*wp).flags |= PANE_EXITED;
    if window_pane_destroy_ready(wp) != 0 {
        server_destroy_pane(wp, 1 as ::core::ffi::c_int);
    }
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_set_event(mut wp: *mut window_pane) {
    setblocking((*wp).fd, 0 as ::core::ffi::c_int);
    (*wp).event = bufferevent_new(
        (*wp).fd,
        Some(
            window_pane_read_callback
                as unsafe extern "C" fn(*mut bufferevent, *mut ::core::ffi::c_void) -> (),
        ),
        None,
        Some(
            window_pane_error_callback
                as unsafe extern "C" fn(
                    *mut bufferevent,
                    ::core::ffi::c_short,
                    *mut ::core::ffi::c_void,
                ) -> (),
        ),
        wp as *mut ::core::ffi::c_void,
    );
    if (*wp).event.is_null() {
        fatalx(b"out of memory\0" as *const u8 as *const ::core::ffi::c_char);
    }
    (*wp).ictx = input_init(
        wp,
        (*wp).event,
        &raw mut (*wp).palette,
        ::core::ptr::null_mut::<client>(),
    );
    bufferevent_enable((*wp).event, (EV_READ | EV_WRITE) as ::core::ffi::c_short);
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_clear_resizes(
    mut wp: *mut window_pane,
    mut except: *mut window_pane_resize,
) {
    let mut r: *mut window_pane_resize = ::core::ptr::null_mut::<window_pane_resize>();
    let mut r1: *mut window_pane_resize = ::core::ptr::null_mut::<window_pane_resize>();
    r = (*wp).resize_queue.tqh_first;
    while !r.is_null() && {
        r1 = (*r).entry.tqe_next;
        1 as ::core::ffi::c_int != 0
    } {
        if !(r == except) {
            if !(*r).entry.tqe_next.is_null() {
                (*(*r).entry.tqe_next).entry.tqe_prev = (*r).entry.tqe_prev;
            } else {
                (*wp).resize_queue.tqh_last = (*r).entry.tqe_prev;
            }
            *(*r).entry.tqe_prev = (*r).entry.tqe_next;
            free(r as *mut ::core::ffi::c_void);
        }
        r = r1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_resize(
    mut wp: *mut window_pane,
    mut sx: u_int,
    mut sy: u_int,
) {
    let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
    let mut r: *mut window_pane_resize = ::core::ptr::null_mut::<window_pane_resize>();
    let mut ep: *mut event_payload = ::core::ptr::null_mut::<event_payload>();
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null::<cmd_find_state>() as *mut cmd_find_state,
        s: ::core::ptr::null::<session>() as *mut session,
        wl: ::core::ptr::null::<winlink>() as *mut winlink,
        w: ::core::ptr::null::<window>() as *mut window,
        wp: ::core::ptr::null::<window_pane>() as *mut window_pane,
        idx: 0,
    };
    if sx == (*wp).sx && sy == (*wp).sy {
        return;
    }
    screen_write_stop_sync(wp);
    r = xmalloc(::core::mem::size_of::<window_pane_resize>() as size_t) as *mut window_pane_resize;
    (*r).sx = sx;
    (*r).sy = sy;
    (*r).osx = (*wp).sx;
    (*r).osy = (*wp).sy;
    (*r).entry.tqe_next = ::core::ptr::null_mut::<window_pane_resize>();
    (*r).entry.tqe_prev = (*wp).resize_queue.tqh_last;
    *(*wp).resize_queue.tqh_last = r;
    (*wp).resize_queue.tqh_last = &raw mut (*r).entry.tqe_next;
    (*wp).sx = sx;
    (*wp).sy = sy;
    log_debug(
        b"%s: %%%u resize %ux%u\0" as *const u8 as *const ::core::ffi::c_char,
        b"window_pane_resize\0" as *const u8 as *const ::core::ffi::c_char,
        (*wp).id,
        sx,
        sy,
    );
    screen_resize(
        &raw mut (*wp).base,
        sx,
        sy,
        ((*wp).base.saved_grid == NULL as *mut grid) as ::core::ffi::c_int,
    );
    wme = (*wp).modes.tqh_first;
    if !wme.is_null() && (*(*wme).mode).resize.is_some() {
        (*(*wme).mode).resize.expect("non-null function pointer")(wme, sx, sy);
    }
    ep = event_payload_create();
    cmd_find_from_pane(&raw mut fs, wp, 0 as ::core::ffi::c_int);
    event_payload_set_target(ep, &raw mut fs);
    event_payload_set_pane(ep, b"pane\0" as *const u8 as *const ::core::ffi::c_char, wp);
    event_payload_set_window(
        ep,
        b"window\0" as *const u8 as *const ::core::ffi::c_char,
        (*wp).window as *mut window,
    );
    event_payload_set_uint(
        ep,
        b"width\0" as *const u8 as *const ::core::ffi::c_char,
        sx,
    );
    event_payload_set_uint(
        ep,
        b"height\0" as *const u8 as *const ::core::ffi::c_char,
        sy,
    );
    event_payload_set_uint(
        ep,
        b"old_width\0" as *const u8 as *const ::core::ffi::c_char,
        (*r).osx,
    );
    event_payload_set_uint(
        ep,
        b"old_height\0" as *const u8 as *const ::core::ffi::c_char,
        (*r).osy,
    );
    events_fire(
        b"pane-resized\0" as *const u8 as *const ::core::ffi::c_char,
        ep,
    );
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_set_mode(
    mut wp: *mut window_pane,
    mut swp: *mut window_pane,
    mut mode: *const window_mode,
    mut item: *mut cmdq_item,
    mut fs: *mut cmd_find_state,
    mut args: *mut args,
) -> ::core::ffi::c_int {
    let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
    let mut w: *mut window = (*wp).window as *mut window;
    let mut name: *const ::core::ffi::c_char = (*mode).name;
    let mut oname: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if !(*wp).modes.tqh_first.is_null() {
        if (*(*wp).modes.tqh_first).mode == mode {
            return 1 as ::core::ffi::c_int;
        }
        if (*(*(*wp).modes.tqh_first).mode).flags & WINDOW_MODE_NO_STACK != 0 {
            window_pane_reset_mode(wp);
        }
    }
    if !(*wp).modes.tqh_first.is_null() {
        oname = (*(*(*wp).modes.tqh_first).mode).name;
    }
    wme = (*wp).modes.tqh_first;
    while !wme.is_null() {
        if (*wme).mode == mode {
            break;
        }
        wme = (*wme).entry.tqe_next;
    }
    if !wme.is_null() {
        if !(*wme).entry.tqe_next.is_null() {
            (*(*wme).entry.tqe_next).entry.tqe_prev = (*wme).entry.tqe_prev;
        } else {
            (*wp).modes.tqh_last = (*wme).entry.tqe_prev;
        }
        *(*wme).entry.tqe_prev = (*wme).entry.tqe_next;
        (*wme).entry.tqe_next = (*wp).modes.tqh_first;
        if !(*wme).entry.tqe_next.is_null() {
            (*(*wp).modes.tqh_first).entry.tqe_prev = &raw mut (*wme).entry.tqe_next;
        } else {
            (*wp).modes.tqh_last = &raw mut (*wme).entry.tqe_next;
        }
        (*wp).modes.tqh_first = wme;
        (*wme).entry.tqe_prev = &raw mut (*wp).modes.tqh_first;
    } else {
        wme = xcalloc(
            1 as size_t,
            ::core::mem::size_of::<window_mode_entry>() as size_t,
        ) as *mut window_mode_entry;
        (*wme).wp = wp;
        (*wme).swp = swp;
        (*wme).mode = mode;
        (*wme).prefix = 1 as u_int;
        (*wme).entry.tqe_next = (*wp).modes.tqh_first;
        if !(*wme).entry.tqe_next.is_null() {
            (*(*wp).modes.tqh_first).entry.tqe_prev = &raw mut (*wme).entry.tqe_next;
        } else {
            (*wp).modes.tqh_last = &raw mut (*wme).entry.tqe_next;
        }
        (*wp).modes.tqh_first = wme;
        (*wme).entry.tqe_prev = &raw mut (*wp).modes.tqh_first;
        (*wme).screen =
            (*(*wme).mode).init.expect("non-null function pointer")(wme, item, fs, args);
        if (*wme).screen.is_null() {
            if !(*wme).entry.tqe_next.is_null() {
                (*(*wme).entry.tqe_next).entry.tqe_prev = (*wme).entry.tqe_prev;
            } else {
                (*wp).modes.tqh_last = (*wme).entry.tqe_prev;
            }
            *(*wme).entry.tqe_prev = (*wme).entry.tqe_next;
            free(wme as *mut ::core::ffi::c_void);
            return 1 as ::core::ffi::c_int;
        }
    }
    (*wme).kill = if !args.is_null() {
        args_has(args, 'k' as i32 as u_char)
    } else {
        0 as ::core::ffi::c_int
    };
    (*wp).screen = (*wme).screen;
    (*wp).flags |= PANE_REDRAW | PANE_REDRAWSCROLLBAR | PANE_CHANGED;
    layout_fix_panes(w, ::core::ptr::null_mut::<window_pane>());
    server_redraw_window_borders((*wp).window as *mut window);
    server_status_window((*wp).window as *mut window);
    window_fire_pane_mode_changed(
        b"pane-mode-entered\0" as *const u8 as *const ::core::ffi::c_char,
        wp,
        oname,
        name,
        1 as ::core::ffi::c_int,
    );
    window_fire_pane_mode_changed(
        b"pane-mode-changed\0" as *const u8 as *const ::core::ffi::c_char,
        wp,
        oname,
        name,
        1 as ::core::ffi::c_int,
    );
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_reset_mode(mut wp: *mut window_pane) {
    let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
    let mut next: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
    let mut w: *mut window = (*wp).window as *mut window;
    let mut kill_0: ::core::ffi::c_int = 0;
    let mut name: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut p: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if (*wp).modes.tqh_first.is_null() {
        return;
    }
    wme = (*wp).modes.tqh_first;
    p = (*(*wme).mode).name;
    kill_0 = (*wme).kill;
    if !(*wme).entry.tqe_next.is_null() {
        (*(*wme).entry.tqe_next).entry.tqe_prev = (*wme).entry.tqe_prev;
    } else {
        (*wp).modes.tqh_last = (*wme).entry.tqe_prev;
    }
    *(*wme).entry.tqe_prev = (*wme).entry.tqe_next;
    (*(*wme).mode).free.expect("non-null function pointer")(wme);
    free(wme as *mut ::core::ffi::c_void);
    next = (*wp).modes.tqh_first;
    if next.is_null() {
        (*wp).flags &= !PANE_UNSEENCHANGES;
        log_debug(
            b"%s: no next mode\0" as *const u8 as *const ::core::ffi::c_char,
            b"window_pane_reset_mode\0" as *const u8 as *const ::core::ffi::c_char,
        );
        (*wp).screen = &raw mut (*wp).base;
    } else {
        log_debug(
            b"%s: next mode is %s\0" as *const u8 as *const ::core::ffi::c_char,
            b"window_pane_reset_mode\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*next).mode).name,
        );
        (*wp).screen = (*next).screen;
        if (*(*next).mode).resize.is_some() {
            (*(*next).mode).resize.expect("non-null function pointer")(next, (*wp).sx, (*wp).sy);
        }
    }
    name = if next.is_null() {
        ::core::ptr::null::<::core::ffi::c_char>()
    } else {
        (*(*next).mode).name
    };
    (*wp).flags |= PANE_REDRAW | PANE_REDRAWSCROLLBAR | PANE_CHANGED;
    layout_fix_panes(w, ::core::ptr::null_mut::<window_pane>());
    server_redraw_window_borders((*wp).window as *mut window);
    server_status_window((*wp).window as *mut window);
    window_fire_pane_mode_changed(
        b"pane-mode-exited\0" as *const u8 as *const ::core::ffi::c_char,
        wp,
        p,
        name,
        0 as ::core::ffi::c_int,
    );
    window_fire_pane_mode_changed(
        b"pane-mode-changed\0" as *const u8 as *const ::core::ffi::c_char,
        wp,
        p,
        name,
        0 as ::core::ffi::c_int,
    );
    if kill_0 != 0 {
        server_kill_pane(wp);
    }
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_reset_mode_all(mut wp: *mut window_pane) {
    while !(*wp).modes.tqh_first.is_null() {
        window_pane_reset_mode(wp);
    }
}
unsafe extern "C" fn window_pane_prompt_input_callback(
    mut data: *mut ::core::ffi::c_void,
    mut s: *const ::core::ffi::c_char,
    mut key: prompt_key_result,
) -> prompt_result {
    let mut wpp: *mut window_pane_prompt = data as *mut window_pane_prompt;
    if (*wpp).inputcb.is_some() {
        return (*wpp).inputcb.expect("non-null function pointer")((*wpp).c, (*wpp).data, s, key);
    }
    return PROMPT_CLOSE;
}
unsafe extern "C" fn window_pane_prompt_free_callback(mut data: *mut ::core::ffi::c_void) {
    let mut wpp: *mut window_pane_prompt = data as *mut window_pane_prompt;
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    wp = window_pane_find_by_id((*wpp).wp_id);
    if !wp.is_null() && (*wp).prompt_data == wpp {
        (*wp).prompt_data = ::core::ptr::null_mut::<window_pane_prompt>();
    }
    if (*wpp).freecb.is_some() {
        (*wpp).freecb.expect("non-null function pointer")((*wpp).data);
    }
    free(wpp as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_set_prompt(
    mut wp: *mut window_pane,
    mut c: *mut client,
    mut fs: *mut cmd_find_state,
    mut msg: *const ::core::ffi::c_char,
    mut input: *const ::core::ffi::c_char,
    mut inputcb: status_prompt_input_cb,
    mut freecb: prompt_free_cb,
    mut data: *mut ::core::ffi::c_void,
    mut flags: ::core::ffi::c_int,
    mut type_0: prompt_type,
) {
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut pd: prompt_create_data = prompt_create_data {
        fs: ::core::ptr::null_mut::<cmd_find_state>(),
        prompt: ::core::ptr::null::<::core::ffi::c_char>(),
        input: ::core::ptr::null::<::core::ffi::c_char>(),
        type_0: PROMPT_TYPE_COMMAND,
        flags: 0,
        style: grid_cell {
            data: utf8_data {
                data: [0; 32],
                have: 0,
                size: 0,
                width: 0,
            },
            attr: 0,
            flags: 0,
            fg: 0,
            bg: 0,
            us: 0,
            link: 0,
        },
        command_style: grid_cell {
            data: utf8_data {
                data: [0; 32],
                have: 0,
                size: 0,
                width: 0,
            },
            attr: 0,
            flags: 0,
            fg: 0,
            bg: 0,
            us: 0,
            link: 0,
        },
        style_str: ::core::ptr::null::<::core::ffi::c_char>(),
        command_style_str: ::core::ptr::null::<::core::ffi::c_char>(),
        cstyle: SCREEN_CURSOR_DEFAULT,
        command_cstyle: SCREEN_CURSOR_DEFAULT,
        ccolour: 0,
        command_ccolour: 0,
        cmode: 0,
        command_cmode: 0,
        message_format: ::core::ptr::null::<::core::ffi::c_char>(),
        keys: 0,
        word_separators: ::core::ptr::null::<::core::ffi::c_char>(),
        inputcb: None,
        freecb: None,
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    let mut wpp: *mut window_pane_prompt = ::core::ptr::null_mut::<window_pane_prompt>();
    if !c.is_null() {
        s = (*c).session;
    }
    window_pane_clear_prompt(wp);
    wpp = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<window_pane_prompt>() as size_t,
    ) as *mut window_pane_prompt;
    (*wpp).wp_id = (*wp).id;
    (*wpp).c = c;
    (*wpp).inputcb = inputcb;
    (*wpp).freecb = freecb;
    (*wpp).data = data;
    (*wpp).type_0 = type_0;
    memset(
        &raw mut pd as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<prompt_create_data>() as size_t,
    );
    prompt_set_options(&raw mut pd, s);
    pd.fs = fs;
    pd.prompt = msg;
    pd.input = input;
    pd.type_0 = type_0;
    pd.flags = flags;
    pd.inputcb = Some(
        window_pane_prompt_input_callback
            as unsafe extern "C" fn(
                *mut ::core::ffi::c_void,
                *const ::core::ffi::c_char,
                prompt_key_result,
            ) -> prompt_result,
    ) as prompt_input_cb;
    pd.freecb = Some(
        window_pane_prompt_free_callback as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> (),
    ) as prompt_free_cb;
    pd.data = wpp as *mut ::core::ffi::c_void;
    (*wp).prompt = prompt_create(&raw mut pd);
    (*wp).prompt_data = wpp;
    (*wp).flags |= PANE_REDRAW;
    prompt_incremental_start((*wp).prompt);
    window_fire_pane_prompt(
        b"pane-prompt-opened\0" as *const u8 as *const ::core::ffi::c_char,
        wp,
        type_0,
    );
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_clear_prompt(mut wp: *mut window_pane) {
    let mut prompt: *mut prompt = (*wp).prompt;
    let mut wpp: *mut window_pane_prompt = (*wp).prompt_data;
    let mut type_0: prompt_type = PROMPT_TYPE_INVALID;
    if !prompt.is_null() {
        if !wpp.is_null() {
            type_0 = (*wpp).type_0;
        }
        (*wp).prompt = ::core::ptr::null_mut::<prompt>();
        prompt_free(prompt);
        (*wp).flags |= PANE_REDRAW;
        if !(*wp).flags & PANE_DESTROYED != 0 {
            window_fire_pane_prompt(
                b"pane-prompt-closed\0" as *const u8 as *const ::core::ffi::c_char,
                wp,
                type_0,
            );
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_has_prompt(mut wp: *mut window_pane) -> ::core::ffi::c_int {
    return ((*wp).prompt != NULL as *mut prompt) as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_update_prompt(
    mut wp: *mut window_pane,
    mut msg: *const ::core::ffi::c_char,
    mut input: *const ::core::ffi::c_char,
) {
    if !(*wp).prompt.is_null() {
        prompt_update((*wp).prompt, msg, input);
        (*wp).flags |= PANE_REDRAW;
    }
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_prompt_key(
    mut wp: *mut window_pane,
    mut c: *mut client,
    mut key: key_code,
    mut m: *mut mouse_event,
) -> prompt_key_result {
    let mut prompt: *mut prompt = (*wp).prompt;
    let mut wpp: *mut window_pane_prompt = (*wp).prompt_data;
    let mut result: prompt_key_result = PROMPT_KEY_NOT_HANDLED;
    let mut wp_id: u_int = (*wp).id;
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    let mut py: u_int = 0;
    let mut redraw: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if prompt.is_null() {
        return PROMPT_KEY_NOT_HANDLED;
    }
    if !wpp.is_null() {
        (*wpp).c = c;
    }
    if key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
        == KEYC_MOUSE as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
        || key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
            >= (KEYC_TYPE_MOUSEMOVE as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                << 32 as ::core::ffi::c_int
            && key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                <= (KEYC_TYPE_TRIPLECLICK as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                    << 32 as ::core::ffi::c_int
    {
        if m.is_null()
            || (*m).b & MOUSE_MASK_BUTTONS as u_int != MOUSE_BUTTON_1 as u_int
            || (*m).b & MOUSE_MASK_DRAG as u_int != 0
            || (*m).b & MOUSE_MASK_BUTTONS as u_int == 3 as u_int
            || cmd_mouse_at(wp, m, &raw mut x, &raw mut y, 0 as ::core::ffi::c_int)
                != 0 as ::core::ffi::c_int
        {
            result = PROMPT_KEY_NOT_HANDLED;
        } else {
            if !c.is_null() && status_at_line(c) == 0 as ::core::ffi::c_int {
                py = 0 as u_int;
            } else {
                py = (*wp).sy.wrapping_sub(1 as u_int);
            }
            if y == py {
                result = prompt_mouse(prompt, x, 0 as u_int, (*wp).sx, &raw mut redraw);
            } else {
                result = PROMPT_KEY_NOT_HANDLED;
            }
        }
    } else {
        result = prompt_key(prompt, key, &raw mut redraw);
    }
    wp = window_pane_find_by_id(wp_id);
    if wp.is_null() {
        return result;
    }
    if !wpp.is_null() && (*wp).prompt_data == wpp {
        (*wpp).c = ::core::ptr::null_mut::<client>();
    }
    if (*wp).prompt == prompt
        && (result as ::core::ffi::c_uint
            == PROMPT_KEY_CLOSE as ::core::ffi::c_int as ::core::ffi::c_uint
            || prompt_closed(prompt) != 0)
    {
        window_pane_clear_prompt(wp);
    }
    if redraw != 0 || (*wp).prompt != prompt {
        (*wp).flags |= PANE_REDRAW;
    }
    return result;
}
unsafe extern "C" fn window_pane_copy_paste(
    mut wp: *mut window_pane,
    mut buf: *mut ::core::ffi::c_char,
    mut len: size_t,
) {
    let mut loop_0: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    loop_0 = (*(*wp).window).panes.tqh_first;
    while !loop_0.is_null() {
        if loop_0 != wp
            && (*loop_0).modes.tqh_first.is_null()
            && (*loop_0).fd != -(1 as ::core::ffi::c_int)
            && !(*loop_0).flags & PANE_INPUTOFF != 0
            && window_pane_is_visible(loop_0) != 0
            && options_get_number(
                (*loop_0).options,
                b"synchronize-panes\0" as *const u8 as *const ::core::ffi::c_char,
            ) != 0
        {
            log_debug(
                b"%s: %.*s\0" as *const u8 as *const ::core::ffi::c_char,
                b"window_pane_copy_paste\0" as *const u8 as *const ::core::ffi::c_char,
                len as ::core::ffi::c_int,
                buf,
            );
            bufferevent_write((*loop_0).event, buf as *const ::core::ffi::c_void, len);
        }
        loop_0 = (*loop_0).entry.tqe_next;
    }
}
unsafe extern "C" fn window_pane_copy_key(mut wp: *mut window_pane, mut key: key_code) {
    let mut loop_0: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    loop_0 = (*(*wp).window).panes.tqh_first;
    while !loop_0.is_null() {
        if loop_0 != wp
            && (*loop_0).modes.tqh_first.is_null()
            && (*loop_0).fd != -(1 as ::core::ffi::c_int)
            && !(*loop_0).flags & PANE_INPUTOFF != 0
            && window_pane_is_visible(loop_0) != 0
            && options_get_number(
                (*loop_0).options,
                b"synchronize-panes\0" as *const u8 as *const ::core::ffi::c_char,
            ) != 0
        {
            input_key_pane(loop_0, key, ::core::ptr::null_mut::<mouse_event>());
        }
        loop_0 = (*loop_0).entry.tqe_next;
    }
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_paste(
    mut wp: *mut window_pane,
    mut key: key_code,
    mut buf: *mut ::core::ffi::c_char,
    mut len: size_t,
) {
    if !(*wp).modes.tqh_first.is_null() {
        return;
    }
    if (*wp).fd == -(1 as ::core::ffi::c_int) || (*wp).flags & PANE_INPUTOFF != 0 {
        return;
    }
    if key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
        == (KEYC_TYPE_FUNCTION as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
            << 32 as ::core::ffi::c_int
        && (key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
            == KEYC_PASTE_START as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
            || key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
                == KEYC_PASTE_END as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong)
        && !(*(*wp).screen).mode & MODE_BRACKETPASTE != 0
    {
        return;
    }
    log_debug(
        b"%s: %.*s\0" as *const u8 as *const ::core::ffi::c_char,
        b"window_pane_paste\0" as *const u8 as *const ::core::ffi::c_char,
        len as ::core::ffi::c_int,
        buf,
    );
    bufferevent_write((*wp).event, buf as *const ::core::ffi::c_void, len);
    if options_get_number(
        (*wp).options,
        b"synchronize-panes\0" as *const u8 as *const ::core::ffi::c_char,
    ) != 0
    {
        window_pane_copy_paste(wp, buf, len);
    }
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_key(
    mut wp: *mut window_pane,
    mut c: *mut client,
    mut s: *mut session,
    mut wl: *mut winlink,
    mut key: key_code,
    mut m: *mut mouse_event,
) -> ::core::ffi::c_int {
    let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
    if (key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
        == KEYC_MOUSE as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
        || key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
            >= (KEYC_TYPE_MOUSEMOVE as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                << 32 as ::core::ffi::c_int
            && key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                <= (KEYC_TYPE_TRIPLECLICK as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                    << 32 as ::core::ffi::c_int)
        && m.is_null()
    {
        return -(1 as ::core::ffi::c_int);
    }
    wme = (*wp).modes.tqh_first;
    if !wme.is_null() {
        if key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
            == (KEYC_TYPE_MOUSEMOVE as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                << 32 as ::core::ffi::c_int
        {
            return 0 as ::core::ffi::c_int;
        }
        if (*(*wme).mode).key.is_some() && !c.is_null() {
            key &= !KEYC_MASK_FLAGS;
            (*(*wme).mode).key.expect("non-null function pointer")(wme, c, s, wl, key, m);
        }
        return 0 as ::core::ffi::c_int;
    }
    if (*wp).fd == -(1 as ::core::ffi::c_int) || (*wp).flags & PANE_INPUTOFF != 0 {
        return 0 as ::core::ffi::c_int;
    }
    if input_key_pane(wp, key, m) != 0 as ::core::ffi::c_int {
        return -(1 as ::core::ffi::c_int);
    }
    if key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
        == KEYC_MOUSE as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
        || key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
            >= (KEYC_TYPE_MOUSEMOVE as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                << 32 as ::core::ffi::c_int
            && key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                <= (KEYC_TYPE_TRIPLECLICK as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                    << 32 as ::core::ffi::c_int
    {
        return 0 as ::core::ffi::c_int;
    }
    if options_get_number(
        (*wp).options,
        b"synchronize-panes\0" as *const u8 as *const ::core::ffi::c_char,
    ) != 0
    {
        window_pane_copy_key(wp, key);
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_is_visible(mut wp: *mut window_pane) -> ::core::ffi::c_int {
    if !(*(*wp).window).flags & WINDOW_ZOOMED != 0 {
        return 1 as ::core::ffi::c_int;
    }
    return ((*wp).layout_cell != NULL as *mut layout_cell) as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_exited(mut wp: *mut window_pane) -> ::core::ffi::c_int {
    return ((*wp).fd == -(1 as ::core::ffi::c_int) || (*wp).flags & PANE_EXITED != 0)
        as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_search(
    mut wp: *mut window_pane,
    mut term: *const ::core::ffi::c_char,
    mut regex: ::core::ffi::c_int,
    mut ignore: ::core::ffi::c_int,
) -> u_int {
    let mut s: *mut screen = &raw mut (*wp).base;
    let mut r: regex_t = re_pattern_buffer {
        buffer: ::core::ptr::null_mut::<re_dfa_t>(),
        allocated: 0,
        used: 0,
        syntax: 0,
        fastmap: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        translate: ::core::ptr::null_mut::<::core::ffi::c_uchar>(),
        re_nsub: 0,
        can_be_null_regs_allocated_fastmap_accurate_no_sub_not_bol_not_eol_newline_anchor: [0; 1],
        c2rust_padding: [0; 7],
    };
    let mut new: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut line: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut i: u_int = 0;
    let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut found: ::core::ffi::c_int = 0;
    let mut n: size_t = 0;
    if regex == 0 {
        if ignore != 0 {
            flags |= FNM_CASEFOLD;
        }
        xasprintf(
            &raw mut new,
            b"*%s*\0" as *const u8 as *const ::core::ffi::c_char,
            term,
        );
    } else {
        if ignore != 0 {
            flags |= REG_ICASE;
        }
        if regcomp(&raw mut r, term, flags | REG_EXTENDED) != 0 as ::core::ffi::c_int {
            return 0 as u_int;
        }
    }
    i = 0 as u_int;
    while i < (*(*s).grid).sy {
        line = grid_view_string_cells((*s).grid, 0 as u_int, i, (*(*s).grid).sx);
        n = strlen(line);
        while n > 0 as size_t {
            if *(*__ctype_b_loc()).offset(*line.offset(n.wrapping_sub(1 as size_t) as isize)
                as u_char as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                & _ISspace as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
                == 0
            {
                break;
            }
            *line.offset(n.wrapping_sub(1 as size_t) as isize) = '\0' as i32 as ::core::ffi::c_char;
            n = n.wrapping_sub(1);
        }
        log_debug(
            b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
            b"window_pane_search\0" as *const u8 as *const ::core::ffi::c_char,
            line,
        );
        if regex == 0 {
            found = (fnmatch(new, line, flags) == 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
        } else {
            found = (regexec(
                &raw mut r,
                line,
                0 as size_t,
                ::core::ptr::null_mut::<regmatch_t>(),
                0 as ::core::ffi::c_int,
            ) == 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
        }
        free(line as *mut ::core::ffi::c_void);
        if found != 0 {
            break;
        }
        i = i.wrapping_add(1);
    }
    if regex == 0 {
        free(new as *mut ::core::ffi::c_void);
    } else {
        regfree(&raw mut r);
    }
    if i == (*(*s).grid).sy {
        return 0 as u_int;
    }
    return i.wrapping_add(1 as u_int);
}
unsafe extern "C" fn window_pane_choose_best(
    mut list: *mut *mut window_pane,
    mut size: u_int,
) -> *mut window_pane {
    let mut next: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut best: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut i: u_int = 0;
    if size == 0 as u_int {
        return ::core::ptr::null_mut::<window_pane>();
    }
    best = *list.offset(0 as ::core::ffi::c_int as isize);
    i = 1 as u_int;
    while i < size {
        next = *list.offset(i as isize);
        if (*next).active_point > (*best).active_point {
            best = next;
        }
        i = i.wrapping_add(1);
    }
    return best;
}
unsafe extern "C" fn window_pane_full_size_offset(
    mut wp: *mut window_pane,
    mut xoff: *mut ::core::ffi::c_int,
    mut yoff: *mut ::core::ffi::c_int,
    mut sx: *mut u_int,
    mut sy: *mut u_int,
) {
    let mut w: *mut window = (*wp).window as *mut window;
    let mut sb_w: u_int = 0;
    if window_pane_scrollbar_reserve(wp) != 0 {
        sb_w = ((*wp).scrollbar_style.width + (*wp).scrollbar_style.pad) as u_int;
    } else {
        sb_w = 0 as u_int;
    }
    if (*w).sb_pos == PANE_SCROLLBARS_LEFT {
        *xoff = ((*wp).xoff as u_int).wrapping_sub(sb_w) as ::core::ffi::c_int;
        *sx = (*wp).sx.wrapping_add(sb_w);
    } else {
        *xoff = (*wp).xoff;
        *sx = (*wp).sx.wrapping_add(sb_w);
    }
    *yoff = (*wp).yoff;
    *sy = (*wp).sy;
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_find_up(mut wp: *mut window_pane) -> *mut window_pane {
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut next: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut best: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut list: *mut *mut window_pane = ::core::ptr::null_mut::<*mut window_pane>();
    let mut edge: ::core::ffi::c_int = 0;
    let mut left: ::core::ffi::c_int = 0;
    let mut right: ::core::ffi::c_int = 0;
    let mut end: ::core::ffi::c_int = 0;
    let mut status: ::core::ffi::c_int = 0;
    let mut found: ::core::ffi::c_int = 0;
    let mut xoff: ::core::ffi::c_int = 0;
    let mut yoff: ::core::ffi::c_int = 0;
    let mut size: u_int = 0;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    if wp.is_null() {
        return ::core::ptr::null_mut::<window_pane>();
    }
    w = (*wp).window as *mut window;
    status = window_get_pane_status(w);
    list = ::core::ptr::null_mut::<*mut window_pane>();
    size = 0 as u_int;
    window_pane_full_size_offset(wp, &raw mut xoff, &raw mut yoff, &raw mut sx, &raw mut sy);
    edge = yoff;
    if status == PANE_STATUS_TOP {
        if edge == 1 as ::core::ffi::c_int {
            edge = (*w).sy as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
        }
    } else if status == PANE_STATUS_BOTTOM {
        if edge == 0 as ::core::ffi::c_int {
            edge = (*w).sy as ::core::ffi::c_int;
        }
    } else if edge == 0 as ::core::ffi::c_int {
        edge = (*w).sy as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
    }
    left = xoff;
    right = xoff + sx as ::core::ffi::c_int;
    next = (*w).panes.tqh_first;
    while !next.is_null() {
        window_pane_full_size_offset(next, &raw mut xoff, &raw mut yoff, &raw mut sx, &raw mut sy);
        if !(next == wp) {
            if !(yoff + sy as ::core::ffi::c_int + 1 as ::core::ffi::c_int != edge) {
                end = xoff + sx as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
                found = 0 as ::core::ffi::c_int;
                if xoff < left && end > right {
                    found = 1 as ::core::ffi::c_int;
                } else if xoff >= left && xoff <= right {
                    found = 1 as ::core::ffi::c_int;
                } else if end >= left && end <= right {
                    found = 1 as ::core::ffi::c_int;
                }
                if !(found == 0) {
                    list = xreallocarray(
                        list as *mut ::core::ffi::c_void,
                        size.wrapping_add(1 as u_int) as size_t,
                        ::core::mem::size_of::<*mut window_pane>() as size_t,
                    ) as *mut *mut window_pane;
                    let fresh18 = size;
                    size = size.wrapping_add(1);
                    let ref mut fresh19 = *list.offset(fresh18 as isize);
                    *fresh19 = next;
                }
            }
        }
        next = (*next).entry.tqe_next;
    }
    best = window_pane_choose_best(list, size);
    free(list as *mut ::core::ffi::c_void);
    return best;
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_find_down(mut wp: *mut window_pane) -> *mut window_pane {
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut next: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut best: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut list: *mut *mut window_pane = ::core::ptr::null_mut::<*mut window_pane>();
    let mut edge: ::core::ffi::c_int = 0;
    let mut left: ::core::ffi::c_int = 0;
    let mut right: ::core::ffi::c_int = 0;
    let mut end: ::core::ffi::c_int = 0;
    let mut status: ::core::ffi::c_int = 0;
    let mut found: ::core::ffi::c_int = 0;
    let mut xoff: ::core::ffi::c_int = 0;
    let mut yoff: ::core::ffi::c_int = 0;
    let mut size: u_int = 0;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    if wp.is_null() {
        return ::core::ptr::null_mut::<window_pane>();
    }
    w = (*wp).window as *mut window;
    status = window_get_pane_status(w);
    list = ::core::ptr::null_mut::<*mut window_pane>();
    size = 0 as u_int;
    window_pane_full_size_offset(wp, &raw mut xoff, &raw mut yoff, &raw mut sx, &raw mut sy);
    edge = yoff + sy as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
    if status == PANE_STATUS_TOP {
        if edge >= (*w).sy as ::core::ffi::c_int {
            edge = 1 as ::core::ffi::c_int;
        }
    } else if status == PANE_STATUS_BOTTOM {
        if edge >= (*w).sy as ::core::ffi::c_int - 1 as ::core::ffi::c_int {
            edge = 0 as ::core::ffi::c_int;
        }
    } else if edge >= (*w).sy as ::core::ffi::c_int {
        edge = 0 as ::core::ffi::c_int;
    }
    left = (*wp).xoff;
    right = (*wp).xoff + (*wp).sx as ::core::ffi::c_int;
    next = (*w).panes.tqh_first;
    while !next.is_null() {
        window_pane_full_size_offset(next, &raw mut xoff, &raw mut yoff, &raw mut sx, &raw mut sy);
        if !(next == wp) {
            if !(yoff != edge) {
                end = xoff + sx as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
                found = 0 as ::core::ffi::c_int;
                if xoff < left && end > right {
                    found = 1 as ::core::ffi::c_int;
                } else if xoff >= left && xoff <= right {
                    found = 1 as ::core::ffi::c_int;
                } else if end >= left && end <= right {
                    found = 1 as ::core::ffi::c_int;
                }
                if !(found == 0) {
                    list = xreallocarray(
                        list as *mut ::core::ffi::c_void,
                        size.wrapping_add(1 as u_int) as size_t,
                        ::core::mem::size_of::<*mut window_pane>() as size_t,
                    ) as *mut *mut window_pane;
                    let fresh20 = size;
                    size = size.wrapping_add(1);
                    let ref mut fresh21 = *list.offset(fresh20 as isize);
                    *fresh21 = next;
                }
            }
        }
        next = (*next).entry.tqe_next;
    }
    best = window_pane_choose_best(list, size);
    free(list as *mut ::core::ffi::c_void);
    return best;
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_find_left(mut wp: *mut window_pane) -> *mut window_pane {
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut next: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut best: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut list: *mut *mut window_pane = ::core::ptr::null_mut::<*mut window_pane>();
    let mut edge: ::core::ffi::c_int = 0;
    let mut top: ::core::ffi::c_int = 0;
    let mut bottom: ::core::ffi::c_int = 0;
    let mut end: ::core::ffi::c_int = 0;
    let mut found: ::core::ffi::c_int = 0;
    let mut xoff: ::core::ffi::c_int = 0;
    let mut yoff: ::core::ffi::c_int = 0;
    let mut size: u_int = 0;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    if wp.is_null() {
        return ::core::ptr::null_mut::<window_pane>();
    }
    w = (*wp).window as *mut window;
    list = ::core::ptr::null_mut::<*mut window_pane>();
    size = 0 as u_int;
    window_pane_full_size_offset(wp, &raw mut xoff, &raw mut yoff, &raw mut sx, &raw mut sy);
    edge = xoff;
    if edge == 0 as ::core::ffi::c_int {
        edge = (*w).sx as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
    }
    top = yoff;
    bottom = yoff + sy as ::core::ffi::c_int;
    next = (*w).panes.tqh_first;
    while !next.is_null() {
        window_pane_full_size_offset(next, &raw mut xoff, &raw mut yoff, &raw mut sx, &raw mut sy);
        if !(next == wp) {
            if !(xoff + sx as ::core::ffi::c_int + 1 as ::core::ffi::c_int != edge) {
                end = yoff + sy as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
                found = 0 as ::core::ffi::c_int;
                if yoff < top && end > bottom {
                    found = 1 as ::core::ffi::c_int;
                } else if yoff >= top && yoff <= bottom {
                    found = 1 as ::core::ffi::c_int;
                } else if end >= top && end <= bottom {
                    found = 1 as ::core::ffi::c_int;
                }
                if !(found == 0) {
                    list = xreallocarray(
                        list as *mut ::core::ffi::c_void,
                        size.wrapping_add(1 as u_int) as size_t,
                        ::core::mem::size_of::<*mut window_pane>() as size_t,
                    ) as *mut *mut window_pane;
                    let fresh22 = size;
                    size = size.wrapping_add(1);
                    let ref mut fresh23 = *list.offset(fresh22 as isize);
                    *fresh23 = next;
                }
            }
        }
        next = (*next).entry.tqe_next;
    }
    best = window_pane_choose_best(list, size);
    free(list as *mut ::core::ffi::c_void);
    return best;
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_find_right(mut wp: *mut window_pane) -> *mut window_pane {
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut next: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut best: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut list: *mut *mut window_pane = ::core::ptr::null_mut::<*mut window_pane>();
    let mut edge: ::core::ffi::c_int = 0;
    let mut top: ::core::ffi::c_int = 0;
    let mut bottom: ::core::ffi::c_int = 0;
    let mut end: ::core::ffi::c_int = 0;
    let mut found: ::core::ffi::c_int = 0;
    let mut xoff: ::core::ffi::c_int = 0;
    let mut yoff: ::core::ffi::c_int = 0;
    let mut size: u_int = 0;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    if wp.is_null() {
        return ::core::ptr::null_mut::<window_pane>();
    }
    w = (*wp).window as *mut window;
    list = ::core::ptr::null_mut::<*mut window_pane>();
    size = 0 as u_int;
    window_pane_full_size_offset(wp, &raw mut xoff, &raw mut yoff, &raw mut sx, &raw mut sy);
    edge = xoff + sx as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
    if edge >= (*w).sx as ::core::ffi::c_int {
        edge = 0 as ::core::ffi::c_int;
    }
    top = (*wp).yoff;
    bottom = (*wp).yoff + (*wp).sy as ::core::ffi::c_int;
    next = (*w).panes.tqh_first;
    while !next.is_null() {
        window_pane_full_size_offset(next, &raw mut xoff, &raw mut yoff, &raw mut sx, &raw mut sy);
        if !(next == wp) {
            if !(xoff != edge) {
                end = yoff + sy as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
                found = 0 as ::core::ffi::c_int;
                if yoff < top && end > bottom {
                    found = 1 as ::core::ffi::c_int;
                } else if yoff >= top && yoff <= bottom {
                    found = 1 as ::core::ffi::c_int;
                } else if end >= top && end <= bottom {
                    found = 1 as ::core::ffi::c_int;
                }
                if !(found == 0) {
                    list = xreallocarray(
                        list as *mut ::core::ffi::c_void,
                        size.wrapping_add(1 as u_int) as size_t,
                        ::core::mem::size_of::<*mut window_pane>() as size_t,
                    ) as *mut *mut window_pane;
                    let fresh24 = size;
                    size = size.wrapping_add(1);
                    let ref mut fresh25 = *list.offset(fresh24 as isize);
                    *fresh25 = next;
                }
            }
        }
        next = (*next).entry.tqe_next;
    }
    best = window_pane_choose_best(list, size);
    free(list as *mut ::core::ffi::c_void);
    return best;
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_stack_push(
    mut stack: *mut window_panes,
    mut wp: *mut window_pane,
) {
    if !wp.is_null() {
        window_pane_stack_remove(stack, wp);
        (*wp).sentry.tqe_next = (*stack).tqh_first;
        if !(*wp).sentry.tqe_next.is_null() {
            (*(*stack).tqh_first).sentry.tqe_prev = &raw mut (*wp).sentry.tqe_next;
        } else {
            (*stack).tqh_last = &raw mut (*wp).sentry.tqe_next;
        }
        (*stack).tqh_first = wp;
        (*wp).sentry.tqe_prev = &raw mut (*stack).tqh_first;
        (*wp).flags |= PANE_VISITED;
    }
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_stack_remove(
    mut stack: *mut window_panes,
    mut wp: *mut window_pane,
) {
    if !wp.is_null() && (*wp).flags & PANE_VISITED != 0 {
        if !(*wp).sentry.tqe_next.is_null() {
            (*(*wp).sentry.tqe_next).sentry.tqe_prev = (*wp).sentry.tqe_prev;
        } else {
            (*stack).tqh_last = (*wp).sentry.tqe_prev;
        }
        *(*wp).sentry.tqe_prev = (*wp).sentry.tqe_next;
        (*wp).flags &= !PANE_VISITED;
    }
}
#[no_mangle]
pub unsafe extern "C" fn winlink_clear_flags(mut wl: *mut winlink) {
    let mut loop_0: *mut winlink = ::core::ptr::null_mut::<winlink>();
    (*(*wl).window).flags &= !WINDOW_ALERTFLAGS;
    loop_0 = (*(*wl).window).winlinks.tqh_first;
    while !loop_0.is_null() {
        if (*loop_0).flags & WINLINK_ALERTFLAGS != 0 as ::core::ffi::c_int {
            (*loop_0).flags &= !WINLINK_ALERTFLAGS;
            server_status_session((*loop_0).session);
        }
        loop_0 = (*loop_0).wentry.tqe_next;
    }
}
#[no_mangle]
pub unsafe extern "C" fn winlink_shuffle_up(
    mut s: *mut session,
    mut wl: *mut winlink,
    mut before: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut idx: ::core::ffi::c_int = 0;
    let mut last: ::core::ffi::c_int = 0;
    if wl.is_null() {
        return -(1 as ::core::ffi::c_int);
    }
    if before != 0 {
        idx = (*wl).idx;
    } else {
        idx = (*wl).idx + 1 as ::core::ffi::c_int;
    }
    last = idx;
    while last < INT_MAX {
        if winlink_find_by_index(&raw mut (*s).windows, last).is_null() {
            break;
        }
        last += 1;
    }
    if last == INT_MAX {
        return -(1 as ::core::ffi::c_int);
    }
    while last > idx {
        wl = winlink_find_by_index(&raw mut (*s).windows, last - 1 as ::core::ffi::c_int);
        winlinks_RB_REMOVE(&raw mut (*s).windows, wl);
        (*wl).idx += 1;
        winlinks_RB_INSERT(&raw mut (*s).windows, wl);
        last -= 1;
    }
    return idx;
}
unsafe extern "C" fn window_pane_input_callback(
    mut c: *mut client,
    mut path: *const ::core::ffi::c_char,
    mut error: ::core::ffi::c_int,
    mut closed: ::core::ffi::c_int,
    mut buffer: *mut evbuffer,
    mut data: *mut ::core::ffi::c_void,
) {
    let mut cdata: *mut window_pane_input_data = data as *mut window_pane_input_data;
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut buf: *mut u_char =
        evbuffer_pullup(buffer, -(1 as ::core::ffi::c_int) as ssize_t) as *mut u_char;
    let mut len: size_t = evbuffer_get_length(buffer);
    wp = window_pane_find_by_id((*cdata).wp);
    if !(*cdata).file.is_null() && (wp.is_null() || (*c).flags & CLIENT_DEAD as uint64_t != 0) {
        if wp.is_null() {
            (*c).retval = 1 as ::core::ffi::c_int;
            (*c).flags |= CLIENT_EXIT as uint64_t;
        }
        file_cancel((*cdata).file);
    } else if (*cdata).file.is_null() || closed != 0 || error != 0 as ::core::ffi::c_int {
        cmdq_continue((*cdata).item);
        server_client_unref(c);
        free(cdata as *mut ::core::ffi::c_void);
    } else {
        input_parse_buffer(wp, buf, len);
    }
    evbuffer_drain(buffer, len);
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_start_input(
    mut wp: *mut window_pane,
    mut item: *mut cmdq_item,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut c: *mut client = cmdq_get_client(item);
    let mut cdata: *mut window_pane_input_data = ::core::ptr::null_mut::<window_pane_input_data>();
    if !(*wp).flags & PANE_EMPTY != 0 {
        *cause = xstrdup(b"pane is not empty\0" as *const u8 as *const ::core::ffi::c_char);
        return -(1 as ::core::ffi::c_int);
    }
    if (*c).flags & (CLIENT_DEAD | CLIENT_EXITED) as uint64_t != 0 {
        return 1 as ::core::ffi::c_int;
    }
    if !(*c).session.is_null() {
        return 1 as ::core::ffi::c_int;
    }
    cdata = xmalloc(::core::mem::size_of::<window_pane_input_data>() as size_t)
        as *mut window_pane_input_data;
    (*cdata).item = item;
    (*cdata).wp = (*wp).id;
    (*cdata).file = file_read(
        c,
        b"-\0" as *const u8 as *const ::core::ffi::c_char,
        Some(
            window_pane_input_callback
                as unsafe extern "C" fn(
                    *mut client,
                    *const ::core::ffi::c_char,
                    ::core::ffi::c_int,
                    ::core::ffi::c_int,
                    *mut evbuffer,
                    *mut ::core::ffi::c_void,
                ) -> (),
        ),
        cdata as *mut ::core::ffi::c_void,
    );
    (*c).references += 1;
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_get_new_data(
    mut wp: *mut window_pane,
    mut wpo: *mut window_pane_offset,
    mut size: *mut size_t,
) -> *mut ::core::ffi::c_void {
    let mut used: size_t = (*wpo).used.wrapping_sub((*wp).base_offset);
    *size = evbuffer_get_length(crate::src::reactor::stream_input((*wp).event)).wrapping_sub(used);
    return evbuffer_pullup(crate::src::reactor::stream_input((*wp).event), -(1 as ::core::ffi::c_int) as ssize_t)
        .offset(used as isize) as *mut ::core::ffi::c_void;
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_update_used_data(
    mut wp: *mut window_pane,
    mut wpo: *mut window_pane_offset,
    mut size: size_t,
) {
    let mut used: size_t = (*wpo).used.wrapping_sub((*wp).base_offset);
    if size > evbuffer_get_length(crate::src::reactor::stream_input((*wp).event)).wrapping_sub(used) {
        size = evbuffer_get_length(crate::src::reactor::stream_input((*wp).event)).wrapping_sub(used);
    }
    (*wpo).used = (*wpo).used.wrapping_add(size);
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_default_cursor(mut wp: *mut window_pane) {
    screen_set_default_cursor((*wp).screen, (*wp).options);
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_mode(mut wp: *mut window_pane) -> ::core::ffi::c_int {
    if !(*wp).modes.tqh_first.is_null() {
        if (*(*wp).modes.tqh_first).mode == &raw const window_copy_mode {
            return 1 as ::core::ffi::c_int;
        }
        if (*(*wp).modes.tqh_first).mode == &raw const window_view_mode {
            return 2 as ::core::ffi::c_int;
        }
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_show_scrollbar(
    mut wp: *mut window_pane,
) -> ::core::ffi::c_int {
    let mut w: *mut window = (*wp).window as *mut window;
    let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
    if !(*wp).base.saved_grid.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    if (*w).flags & WINDOW_ZOOMED != 0 && !(*w).active.is_null() {
        wme = (*(*w).active).modes.tqh_first;
        if !wme.is_null() && (*(*wme).mode).flags & WINDOW_MODE_HIDE_SCROLLBARS != 0 {
            return 0 as ::core::ffi::c_int;
        }
    }
    if (*w).sb == PANE_SCROLLBARS_ALWAYS
        || (*w).sb == PANE_SCROLLBARS_AUTOHIDE
        || (*w).sb == PANE_SCROLLBARS_MODAL && window_pane_mode(wp) != WINDOW_PANE_NO_MODE
    {
        return 1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_scrollbar_reserve(
    mut wp: *mut window_pane,
) -> ::core::ffi::c_int {
    if window_pane_show_scrollbar(wp) == 0 {
        return 0 as ::core::ffi::c_int;
    }
    return ((*(*wp).window).sb == PANE_SCROLLBARS_ALWAYS) as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_scrollbar_overlay(
    mut wp: *mut window_pane,
) -> ::core::ffi::c_int {
    if window_pane_show_scrollbar(wp) == 0 {
        return 0 as ::core::ffi::c_int;
    }
    return window_pane_scrollbar_auto_hide(wp);
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_scrollbar_visible(
    mut wp: *mut window_pane,
) -> ::core::ffi::c_int {
    if window_pane_show_scrollbar(wp) == 0 {
        return 0 as ::core::ffi::c_int;
    }
    if window_pane_scrollbar_auto_hide(wp) == 0 {
        return 1 as ::core::ffi::c_int;
    }
    return (*wp).sb_auto_visible;
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_scrollbar_start_timer(mut wp: *mut window_pane) {
    let mut tv: timeval = timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    let mut delay: u_int = 0;
    if window_pane_scrollbar_auto_hide(wp) == 0 || (*wp).sb_auto_visible == 0 {
        return;
    }
    delay = options_get_number(
        (*(*wp).window).options,
        b"pane-scrollbars-timeout\0" as *const u8 as *const ::core::ffi::c_char,
    ) as u_int;
    tv.tv_sec = delay.wrapping_div(1000 as u_int) as __time_t;
    tv.tv_usec = (delay.wrapping_rem(1000 as u_int) as ::core::ffi::c_long
        * 1000 as ::core::ffi::c_long) as __suseconds_t;
    event_del(&raw mut (*wp).sb_auto_timer);
    event_add(&raw mut (*wp).sb_auto_timer, &raw mut tv);
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_scrollbar_show(
    mut wp: *mut window_pane,
    mut start_timer: ::core::ffi::c_int,
) {
    let mut changed: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if window_pane_scrollbar_auto_hide(wp) == 0 {
        return;
    }
    if window_pane_show_scrollbar(wp) == 0 {
        return;
    }
    if (*wp).sb_auto_visible == 0 {
        (*wp).sb_auto_visible = 1 as ::core::ffi::c_int;
        changed = 1 as ::core::ffi::c_int;
    }
    event_del(&raw mut (*wp).sb_auto_timer);
    if start_timer != 0 {
        window_pane_scrollbar_start_timer(wp);
    }
    if changed != 0 {
        window_pane_scrollbar_redraw_visibility(wp);
    }
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_scrollbar_hide(mut wp: *mut window_pane) {
    if event_initialized(&raw mut (*wp).sb_auto_timer) != 0 {
        event_del(&raw mut (*wp).sb_auto_timer);
    }
    (*wp).sb_auto_hover = 0 as ::core::ffi::c_int;
    if (*wp).sb_auto_visible == 0 {
        return;
    }
    (*wp).sb_auto_visible = 0 as ::core::ffi::c_int;
    window_pane_scrollbar_redraw_visibility(wp);
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_get_bg(mut wp: *mut window_pane) -> ::core::ffi::c_int {
    let mut c: ::core::ffi::c_int = 0;
    let mut defaults: grid_cell = grid_cell {
        data: utf8_data {
            data: [0; 32],
            have: 0,
            size: 0,
            width: 0,
        },
        attr: 0,
        flags: 0,
        fg: 0,
        bg: 0,
        us: 0,
        link: 0,
    };
    c = window_pane_get_bg_control_client(wp);
    if c == -(1 as ::core::ffi::c_int) {
        tty_default_colours(&raw mut defaults, wp, ::core::ptr::null_mut::<u_int>());
        if defaults.bg == 8 as ::core::ffi::c_int || defaults.bg == 9 as ::core::ffi::c_int {
            c = window_get_bg_client(wp);
        } else {
            c = defaults.bg;
        }
    }
    return c;
}
#[no_mangle]
pub unsafe extern "C" fn window_get_bg_client(mut wp: *mut window_pane) -> ::core::ffi::c_int {
    let mut w: *mut window = (*wp).window as *mut window;
    let mut loop_0: *mut client = ::core::ptr::null_mut::<client>();
    loop_0 = clients.tqh_first;
    while !loop_0.is_null() {
        if !((*loop_0).flags & CLIENT_UNATTACHEDFLAGS as uint64_t != 0) {
            if !((*loop_0).session.is_null() || session_has((*loop_0).session, w) == 0) {
                if !((*loop_0).tty.bg == -(1 as ::core::ffi::c_int)) {
                    return (*loop_0).tty.bg;
                }
            }
        }
        loop_0 = (*loop_0).entry.tqe_next;
    }
    return -(1 as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_get_bg_control_client(
    mut wp: *mut window_pane,
) -> ::core::ffi::c_int {
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    if (*wp).control_bg == -(1 as ::core::ffi::c_int) {
        return -(1 as ::core::ffi::c_int);
    }
    c = clients.tqh_first;
    while !c.is_null() {
        if (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
            return (*wp).control_bg;
        }
        c = (*c).entry.tqe_next;
    }
    return -(1 as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_get_fg(mut wp: *mut window_pane) -> ::core::ffi::c_int {
    let mut w: *mut window = (*wp).window as *mut window;
    let mut loop_0: *mut client = ::core::ptr::null_mut::<client>();
    loop_0 = clients.tqh_first;
    while !loop_0.is_null() {
        if !((*loop_0).flags & CLIENT_UNATTACHEDFLAGS as uint64_t != 0) {
            if !((*loop_0).session.is_null() || session_has((*loop_0).session, w) == 0) {
                if !((*loop_0).tty.fg == -(1 as ::core::ffi::c_int)) {
                    return (*loop_0).tty.fg;
                }
            }
        }
        loop_0 = (*loop_0).entry.tqe_next;
    }
    return -(1 as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_get_fg_control_client(
    mut wp: *mut window_pane,
) -> ::core::ffi::c_int {
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    if (*wp).control_fg == -(1 as ::core::ffi::c_int) {
        return -(1 as ::core::ffi::c_int);
    }
    c = clients.tqh_first;
    while !c.is_null() {
        if (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
            return (*wp).control_fg;
        }
        c = (*c).entry.tqe_next;
    }
    return -(1 as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_get_theme(mut wp: *mut window_pane) -> client_theme {
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut loop_0: *mut client = ::core::ptr::null_mut::<client>();
    let mut found_light: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut found_dark: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if wp.is_null() {
        return THEME_UNKNOWN;
    }
    w = (*wp).window as *mut window;
    loop_0 = clients.tqh_first;
    while !loop_0.is_null() {
        if !((*loop_0).flags & CLIENT_UNATTACHEDFLAGS as uint64_t != 0) {
            if !((*loop_0).session.is_null() || session_has((*loop_0).session, w) == 0) {
                match (*loop_0).theme as ::core::ffi::c_uint {
                    1 => {
                        found_light = 1 as ::core::ffi::c_int;
                    }
                    2 => {
                        found_dark = 1 as ::core::ffi::c_int;
                    }
                    0 | _ => {}
                }
            }
        }
        loop_0 = (*loop_0).entry.tqe_next;
    }
    if found_dark != 0 && found_light == 0 {
        return THEME_DARK;
    }
    if found_light != 0 && found_dark == 0 {
        return THEME_LIGHT;
    }
    return colour_totheme(window_pane_get_bg(wp));
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_send_theme_update(mut wp: *mut window_pane) {
    let mut theme: client_theme = THEME_UNKNOWN;
    if wp.is_null() || window_pane_exited(wp) != 0 {
        return;
    }
    if !(*wp).flags & PANE_THEMECHANGED != 0 {
        return;
    }
    if !(*(*wp).screen).mode & MODE_THEME_UPDATES != 0 {
        return;
    }
    theme = window_pane_get_theme(wp);
    if theme as ::core::ffi::c_uint == (*wp).last_theme as ::core::ffi::c_uint {
        return;
    }
    (*wp).last_theme = theme;
    (*wp).flags &= !PANE_THEMECHANGED;
    match theme as ::core::ffi::c_uint {
        1 => {
            log_debug(
                b"%s: %%%u light theme\0" as *const u8 as *const ::core::ffi::c_char,
                b"window_pane_send_theme_update\0" as *const u8 as *const ::core::ffi::c_char,
                (*wp).id,
            );
            bufferevent_write(
                (*wp).event,
                b"\x1B[?997;2n\0" as *const u8 as *const ::core::ffi::c_char
                    as *const ::core::ffi::c_void,
                9 as size_t,
            );
        }
        2 => {
            log_debug(
                b"%s: %%%u dark theme\0" as *const u8 as *const ::core::ffi::c_char,
                b"window_pane_send_theme_update\0" as *const u8 as *const ::core::ffi::c_char,
                (*wp).id,
            );
            bufferevent_write(
                (*wp).event,
                b"\x1B[?997;1n\0" as *const u8 as *const ::core::ffi::c_char
                    as *const ::core::ffi::c_void,
                9 as size_t,
            );
        }
        0 => {
            log_debug(
                b"%s: %%%u unknown theme\0" as *const u8 as *const ::core::ffi::c_char,
                b"window_pane_send_theme_update\0" as *const u8 as *const ::core::ffi::c_char,
                (*wp).id,
            );
        }
        _ => {}
    };
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_status_get_range(
    mut wp: *mut window_pane,
    mut x: u_int,
    mut y: u_int,
) -> *mut style_range {
    let mut srs: *mut style_ranges = ::core::ptr::null_mut::<style_ranges>();
    let mut line: u_int = 0;
    let mut pane_status: ::core::ffi::c_int = 0;
    if wp.is_null() {
        return ::core::ptr::null_mut::<style_range>();
    }
    srs = &raw mut (*wp).border_status_line.ranges;
    pane_status = window_pane_get_pane_status(wp);
    if pane_status == PANE_STATUS_TOP {
        line = ((*wp).yoff - 1 as ::core::ffi::c_int) as u_int;
    } else if pane_status == PANE_STATUS_BOTTOM {
        line = ((*wp).yoff as u_int).wrapping_add((*wp).sy);
    }
    if pane_status == PANE_STATUS_OFF || line != y {
        return ::core::ptr::null_mut::<style_range>();
    }
    return style_ranges_get_range(
        srs,
        x.wrapping_sub((*wp).xoff as u_int).wrapping_sub(2 as u_int),
    );
}
#[no_mangle]
pub unsafe extern "C" fn window_get_pane_lines(mut w: *mut window) -> pane_lines {
    let mut oo: *mut options = ::core::ptr::null_mut::<options>();
    oo = (*w).options;
    return options_get_number(
        oo,
        b"pane-border-lines\0" as *const u8 as *const ::core::ffi::c_char,
    ) as pane_lines;
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_get_pane_lines(mut wp: *mut window_pane) -> pane_lines {
    let mut oo: *mut options = ::core::ptr::null_mut::<options>();
    if window_pane_is_floating(wp) == 0 {
        oo = (*(*wp).window).options;
    } else {
        oo = (*wp).options;
    }
    return options_get_number(
        oo,
        b"pane-border-lines\0" as *const u8 as *const ::core::ffi::c_char,
    ) as pane_lines;
}
#[no_mangle]
pub unsafe extern "C" fn window_get_pane_status(mut w: *mut window) -> ::core::ffi::c_int {
    let mut status: ::core::ffi::c_int = 0;
    status = options_get_number(
        (*w).options,
        b"pane-border-status\0" as *const u8 as *const ::core::ffi::c_char,
    ) as ::core::ffi::c_int;
    if status == PANE_STATUS_TOP_FLOATING || status == PANE_STATUS_BOTTOM_FLOATING {
        return 0 as ::core::ffi::c_int;
    }
    return status;
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_get_pane_status(
    mut wp: *mut window_pane,
) -> ::core::ffi::c_int {
    let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
    let mut status: ::core::ffi::c_int = 0;
    wme = (*wp).modes.tqh_first;
    if !wme.is_null()
        && (*(*wme).mode).flags & WINDOW_MODE_HIDE_PANE_STATUS != 0
        && (*wp).flags & PANE_ZOOMED != 0
    {
        return 0 as ::core::ffi::c_int;
    }
    if window_pane_is_floating(wp) == 0 {
        return window_get_pane_status((*wp).window as *mut window);
    }
    if window_pane_get_pane_lines(wp) as ::core::ffi::c_uint
        == PANE_LINES_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return 0 as ::core::ffi::c_int;
    }
    status = options_get_number(
        (*wp).options,
        b"pane-border-status\0" as *const u8 as *const ::core::ffi::c_char,
    ) as ::core::ffi::c_int;
    if status == PANE_STATUS_TOP_FLOATING {
        return 1 as ::core::ffi::c_int;
    }
    if status == PANE_STATUS_BOTTOM_FLOATING {
        return 2 as ::core::ffi::c_int;
    }
    return status;
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_is_floating(mut wp: *mut window_pane) -> ::core::ffi::c_int {
    let mut lc: *mut layout_cell = (*wp).layout_cell as *mut layout_cell;
    if lc.is_null() || (*lc).flags & LAYOUT_CELL_FLOATING == 0 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    return 1 as ::core::ffi::c_int;
}
