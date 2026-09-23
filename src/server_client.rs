use crate::src::alerts::alerts_check_session;
use crate::src::arguments::OwnedArgumentVector;
use crate::src::cfg::{cfg_client, cfg_finished, start_cfg};
use crate::src::cmd::{cmd_free_argv, cmd_list_all_have, cmd_list_free, cmd_unpack_argv};
use crate::src::cmd_find::{cmd_find_from_client, cmd_find_from_mouse};
use crate::src::cmd_parse::cmd_parse_from_arguments;
use crate::src::cmd_queue::{
    cmdq_append, cmdq_error, cmdq_free, cmdq_get_callback1, cmdq_get_client, cmdq_get_command,
    cmdq_get_error, cmdq_insert_after, cmdq_new,
};
use crate::src::colour::{
    colour_parse_cstr, colour_theme_option, colour_theme_terminal_colour, colour_totheme,
};
use crate::src::compat::imsg::imsg_get_fd;
use crate::src::control::{
    control_all_done, control_discard, control_discard_all, control_pane_offset, control_ready,
    control_reset_offsets, control_start, control_stop, control_write,
};
use crate::src::environ::{environ_create, environ_find, environ_free, environ_put};
use crate::src::events::{events_fire, events_fire_client};
use crate::src::events_payload::{
    event_payload_create, event_payload_set_client, event_payload_set_int, event_payload_set_pane,
    event_payload_set_session, event_payload_set_target, event_payload_set_uint,
    event_payload_set_window,
};
use crate::src::ffi::libc::{
    access, close, free, gettimeofday, isatty, memcpy, sscanf, strchr, strcmp, strlcat, strlen,
    strsep, ttyname,
};
use crate::src::file::{
    client_files_minmax, client_files_next, file_fire_done, file_print, file_read_data,
    file_read_done, file_write_done, file_write_ready,
};
use crate::src::format::{
    format_create, format_defaults, format_expand, format_expand_time, format_free,
    format_lost_client,
};
use crate::src::input::input_cancel_requests;
use crate::src::key_bindings::{
    key_bindings_dispatch, key_bindings_get, key_bindings_get_table, key_bindings_unref_table,
};
use crate::src::key_string::key_string_format;
use crate::src::log::{fatal, log_debug, log_get_level};
use crate::src::menu::{menu_close, menu_get_cursor, menu_key, menu_screen};
use crate::src::names::check_window_name;
use crate::src::options::{
    options_get_command, options_get_number, options_get_string, options_set_number,
};
use crate::src::proc::{proc_add_peer, proc_kill_peer, proc_remove_peer, proc_send};
use crate::src::prompt::prompt_free;
use crate::src::reactor::{
    bufferevent_disable, bufferevent_enable, evbuffer_add, evbuffer_drain, evbuffer_get_length,
    evbuffer_pullup, evbuffer_readln, event_add, event_del, event_initialized, event_once,
    event_pending, event_set,
};
use crate::src::resize::{recalculate_size, recalculate_sizes, resize_window};
use crate::src::screen::screen_mode_to_string;
use crate::src::screen_redraw::{
    redraw_free_scene, redraw_pane, redraw_pane_scrollbar, redraw_screen,
};
pub use crate::src::server::clients;
use crate::src::server::{current_time, server_add_accept, server_proc, server_update_socket};
use crate::src::server_fn::{
    server_check_unattached, server_destroy_pane, server_kill_pane, server_redraw_client,
    server_redraw_window_borders, server_status_client, server_status_window,
};
use crate::src::session::{session_find_by_id, session_theme_changed, session_update_activity};
pub use crate::src::shared::events::event_payload;
pub use crate::src::shared::message::msg_command;
pub use crate::src::shared::pane::window_pane_tree;
use crate::src::status::{
    status_at_line, status_free, status_get_range, status_init, status_line_size,
    status_message_clear, status_prompt_clear, status_prompt_cursor, status_prompt_key,
    status_timer_start,
};
use crate::src::tmux::{checkshell, find_home, global_options, global_s_options, setblocking};
use crate::src::tty::{
    tty_close, tty_cursor, tty_free, tty_init, tty_invalidate, tty_margin_off, tty_open,
    tty_region_off, tty_repeat_requests, tty_reset, tty_resize, tty_send_requests, tty_set_path,
    tty_set_progress_bar, tty_set_title, tty_start_tty, tty_stop_tty, tty_sync_end,
    tty_update_client_offset, tty_update_mode, tty_window_offset,
};
use crate::src::tty_features::tty_get_features;
use crate::src::tty_term::tty_term_has;
use crate::src::utf8::{utf8_sanitize, utf8_stravisx};
pub use crate::src::window::windows;
use crate::src::window::{
    all_window_panes, window_get_active_at, window_pane_clear_resizes, window_pane_contains,
    window_pane_find_by_id, window_pane_get_new_data, window_pane_get_pane_lines,
    window_pane_get_pane_status, window_pane_has_prompt, window_pane_is_floating,
    window_pane_is_visible, window_pane_key, window_pane_paste, window_pane_prompt_key,
    window_pane_scrollbar_overlay, window_pane_scrollbar_overlay_visible,
    window_pane_scrollbar_reserve, window_pane_scrollbar_show, window_pane_scrollbar_start_timer,
    window_pane_scrollbar_visible, window_pane_send_resize, window_pane_send_theme_update,
    window_pane_set_mode, window_pane_status_get_range, window_pane_tree_minmax,
    window_pane_tree_next, window_redraw_active_switch, window_set_active_pane,
    window_update_focus, windows_minmax, windows_next, winlink_find_by_index,
};
use crate::src::window_copy::{window_copy_add, window_view_mode};
use crate::src::window_visible::{window_position_is_visible, window_visible_ranges};
use crate::src::xmalloc::{xasprintf, xcalloc, xmalloc, xrecallocarray, xsnprintf, xstrdup};
use std::ffi::{CStr, CString};

use crate::src::shared::abi::*;
pub use crate::src::shared::abi::{__uint32_t, ssize_t, uint32_t};
use crate::src::shared::arguments::*;
pub use crate::src::shared::arguments::{
    args, args_value, args_value_c2rust_unnamed, args_value_entry,
};
use crate::src::shared::client::*;
pub use crate::src::shared::client::{
    client, client_entry, client_file, client_file_cb, client_file_entry, client_files,
    overlay_check_cb, overlay_draw_cb, overlay_free_cb, overlay_key_cb, overlay_mode_cb,
    overlay_resize_cb,
};

// The public client record keeps its C layout. Only server_client_create
// allocates clients, and server_client_free drops this containing owner.
// message_string borrows the CString until the next replacement or clear.
// term_caps borrows term_cap_ptrs, whose entries borrow term_cap_strings.
// Both views are refreshed after every identify capability and cleared after
// tty_free, before the client owner is eventually dropped.
#[repr(C)]
struct ClientOwner {
    node: client,
    message: Option<CString>,
    saved_status_screen: Option<Box<screen>>,
    term_cap_strings: Vec<CString>,
    term_cap_ptrs: Vec<*mut ::core::ffi::c_char>,
}

const _: () = assert!(std::mem::offset_of!(ClientOwner, node) == 0);

pub(crate) unsafe fn server_client_set_message(c: *mut client, message: Option<CString>) {
    let owner = c as *mut ClientOwner;
    // Invalidate the compatibility pointer before releasing the old value.
    (*c).message_string = ::core::ptr::null_mut();
    (*owner).message = message;
    if let Some(message) = (*owner).message.as_ref() {
        (*c).message_string = message.as_ptr().cast_mut();
    }
}

pub(crate) unsafe fn server_client_set_saved_status_screen(c: *mut client, screen: Box<screen>) {
    let owner = c as *mut ClientOwner;
    assert!((*owner).saved_status_screen.is_none());
    (*owner).saved_status_screen = Some(screen);
    (*c).status.active = (*owner).saved_status_screen.as_deref_mut().unwrap();
}

pub(crate) unsafe fn server_client_clear_saved_status_screen(c: *mut client) {
    let owner = c as *mut ClientOwner;
    // Reset the public view before dropping the allocation it pointed into.
    (*c).status.active = &raw mut (*c).status.screen;
    assert!((*owner).saved_status_screen.is_some());
    (*owner).saved_status_screen = None;
}

unsafe fn server_client_add_term_cap(c: *mut client, data: *const ::core::ffi::c_char) {
    let owner = c as *mut ClientOwner;
    // The pointer array can move on growth. Invalidate the public view first.
    (*c).term_caps = ::core::ptr::null_mut();
    (*c).term_ncaps = 0;
    assert!((*owner).term_cap_strings.len() < u_int::MAX as usize);
    (*owner)
        .term_cap_strings
        .push(CStr::from_ptr(data).to_owned());
    let cap = (*owner)
        .term_cap_strings
        .last()
        .unwrap()
        .as_ptr()
        .cast_mut();
    (*owner).term_cap_ptrs.push(cap);
    (*c).term_caps = (*owner).term_cap_ptrs.as_mut_ptr();
    (*c).term_ncaps = (*owner).term_cap_ptrs.len() as u_int;
}

unsafe fn server_client_clear_term_caps(c: *mut client) {
    let owner = c as *mut ClientOwner;
    (*c).term_caps = ::core::ptr::null_mut();
    (*c).term_ncaps = 0;
    // Release on client loss, even when other references delay the final
    // ClientOwner drop.
    (*owner).term_cap_strings = Vec::new();
    (*owner).term_cap_ptrs = Vec::new();
}

#[cfg(test)]
mod client_message_owner_tests {
    use super::{
        client, server_client_add_term_cap, server_client_clear_term_caps,
        server_client_set_message, ClientOwner,
    };
    use crate::src::status::status_message_clear;
    use std::ffi::{CStr, CString};

    #[test]
    fn message_replacement_and_clear_keep_the_client_pointer_stable() {
        unsafe {
            let mut owner = Box::new(ClientOwner {
                node: std::mem::zeroed::<client>(),
                message: None,
                saved_status_screen: None,
                term_cap_strings: Vec::new(),
                term_cap_ptrs: Vec::new(),
            });
            let c = &raw mut owner.node;
            assert!((*c).message_string.is_null());

            server_client_set_message(c, Some(CString::new(vec![b'a', 0xff]).unwrap()));
            assert_eq!(CStr::from_ptr((*c).message_string).to_bytes(), b"a\xff");
            assert_eq!(c, &raw mut owner.node);

            server_client_set_message(c, Some(CString::new(Vec::<u8>::new()).unwrap()));
            assert_eq!(CStr::from_ptr((*c).message_string).to_bytes(), b"");
            assert!(!(*c).message_string.is_null());

            // A live message has pushed a status screen. Keep one extra
            // reference so status_message_clear does not need a full screen.
            (*c).status.references = 2;
            status_message_clear(c);
            assert!((*c).message_string.is_null());
            assert!(owner.message.is_none());
            assert_eq!((*c).status.references, 1);
        }
    }

    #[test]
    fn term_caps_view_survives_growth_and_preserves_order_and_bytes() {
        unsafe {
            let mut owner = Box::new(ClientOwner {
                node: std::mem::zeroed::<client>(),
                message: None,
                saved_status_screen: None,
                term_cap_strings: Vec::new(),
                term_cap_ptrs: Vec::new(),
            });
            let c = &raw mut owner.node;
            let mut expected = Vec::new();
            for i in 0..64 {
                let value = if i % 3 == 0 {
                    CString::new(b"dup=\xff".to_vec()).unwrap()
                } else {
                    CString::new(format!("cap{i}=value")).unwrap()
                };
                server_client_add_term_cap(c, value.as_ptr());
                expected.push(value);
                assert_eq!((*c).term_ncaps as usize, expected.len());
                for (index, cap) in expected.iter().enumerate() {
                    assert_eq!(
                        CStr::from_ptr(*(*c).term_caps.add(index)).to_bytes(),
                        cap.as_bytes()
                    );
                }
            }
            server_client_clear_term_caps(c);
            assert!((*c).term_caps.is_null());
            assert_eq!((*c).term_ncaps, 0);
            assert!(owner.term_cap_strings.is_empty());
            assert!(owner.term_cap_ptrs.is_empty());
        }
    }
}
pub use crate::src::shared::client::{
    CLIENT_ALLREDRAWFLAGS, CLIENT_ASSUMEPASTING, CLIENT_ATTACHED, CLIENT_BRACKETPASTING,
    CLIENT_CONTROL, CLIENT_CONTROL_NEWLAYOUTS, CLIENT_CONTROL_NOOUTPUT, CLIENT_CONTROL_PAUSEAFTER,
    CLIENT_CONTROL_WAITEXIT, CLIENT_DEAD, CLIENT_DOUBLECLICK, CLIENT_EXIT, CLIENT_EXITED,
    CLIENT_FOCUSED, CLIENT_IDENTIFIED, CLIENT_IGNORESIZE, CLIENT_NODETACHFLAGS,
    CLIENT_NO_DETACH_ON_DESTROY, CLIENT_PASTE_TIME_LIMIT, CLIENT_READONLY, CLIENT_REDRAWBORDERS,
    CLIENT_REDRAWMENU, CLIENT_REDRAWOVERLAY, CLIENT_REDRAWSCROLLBARS, CLIENT_REDRAWSTATUS,
    CLIENT_REDRAWSTATUSALWAYS, CLIENT_REDRAWWINDOW, CLIENT_REPEAT, CLIENT_STATUSFORCE,
    CLIENT_SUSPENDED, CLIENT_TERMINAL, CLIENT_TRIPLECLICK, CLIENT_UNATTACHEDFLAGS, CLIENT_UTF8,
};
use crate::src::shared::colour::*;
pub use crate::src::shared::colour::{COLOUR_FLAG_THEME, COLOUR_THEME_COUNT};
pub use crate::src::shared::command::CMD_READONLY;
use crate::src::shared::command::*;
use crate::src::shared::command::*;
pub use crate::src::shared::command::{
    cmd_find_state, cmd_list, cmdq_cb, cmdq_item, cmdq_list, cmdq_state, cmds,
};
pub use crate::src::shared::command::{cmd_parse_input, cmd_parse_result};
pub use crate::src::shared::control::control_state;
use crate::src::shared::display::*;
pub use crate::src::shared::display::{visible_range, visible_ranges};
pub use crate::src::shared::environment::{environ, environ_entry};
pub use crate::src::shared::errno::EINTR;
use crate::src::shared::event::*;
pub use crate::src::shared::event::{
    evbuffer_eol_style, EVBUFFER_EOL_ANY, EVBUFFER_EOL_CRLF, EVBUFFER_EOL_CRLF_STRICT,
    EVBUFFER_EOL_LF, EVBUFFER_EOL_NUL, EV_READ, EV_TIMEOUT,
};
pub use crate::src::shared::format::{format_job_tree, format_tree};
pub use crate::src::shared::format::{FORMAT_NOJOBS, FORMAT_NONE};
use crate::src::shared::grid::*;
pub use crate::src::shared::hyperlinks::hyperlinks;
pub use crate::src::shared::input::{input_ctx, input_request, input_requests};
pub use crate::src::shared::key::KEY_BINDING_REPEAT;
use crate::src::shared::key::*;
pub use crate::src::shared::key::{
    key_binding, key_binding_entry, key_bindings, key_event, key_table, key_table_entry,
};
pub use crate::src::shared::layout::layout_geometry;
use crate::src::shared::layout::*;
use crate::src::shared::layout::*;
pub use crate::src::shared::layout::{layout_cell, layout_cell_entry, layout_cells};
pub use crate::src::shared::limits::{__INT_MAX__, SIZE_MAX, UINT_MAX};
pub use crate::src::shared::menu::menu_data;
use crate::src::shared::message::*;
pub use crate::src::shared::message::{ibuf, ibuf_entry, imsg};
pub use crate::src::shared::message::{imsg_hdr, IMSG_HEADER_SIZE};
pub use crate::src::shared::mouse::{
    mouse_event, MOUSE_BUTTON_1, MOUSE_BUTTON_10, MOUSE_BUTTON_11, MOUSE_BUTTON_2, MOUSE_BUTTON_3,
    MOUSE_BUTTON_6, MOUSE_BUTTON_7, MOUSE_BUTTON_8, MOUSE_BUTTON_9, MOUSE_MASK_BUTTONS,
    MOUSE_MASK_CTRL, MOUSE_MASK_DRAG, MOUSE_MASK_META, MOUSE_MASK_SHIFT, MOUSE_WHEEL_DOWN,
    MOUSE_WHEEL_UP,
};
pub use crate::src::shared::options::{options, options_entry};
pub use crate::src::shared::pane::{
    window_pane, window_pane_entry, window_pane_modes, window_pane_prompt, window_pane_sentry,
    window_pane_tree_entry, window_pane_zentry, window_panes,
};
pub use crate::src::shared::pane::{
    window_pane_offset, window_pane_resize, window_pane_resize_entry, window_pane_resizes,
    PANE_ACTIVITY, PANE_CAPTUREALLKEYS, PANE_CLOSEONCANCEL, PANE_CLOSEONCLICK, PANE_EXITED,
    PANE_REDRAW, PANE_REDRAWSCROLLBAR, PANE_SCROLLBARS_AUTOHIDE, PANE_SCROLLBARS_LEFT,
    PANE_SCROLLBARS_MODAL, PANE_SCROLLBARS_RIGHT, PANE_STATUS_BOTTOM, PANE_STATUS_OFF,
    PANE_STATUS_TOP, PANE_STYLECHANGED,
};
pub use crate::src::shared::posix_io::{
    _PATH_BSHELL, STDERR_FILENO, STDIN_FILENO, STDOUT_FILENO, X_OK,
};
pub use crate::src::shared::process::{tmuxpeer, tmuxproc};
pub use crate::src::shared::prompt::prompt;
pub use crate::src::shared::redraw::redraw_scene;
pub use crate::src::shared::screen::{
    screen, screen_sel, screen_titles, ALL_MOUSE_MODES, CURSOR_MODES, MODE_BRACKETPASTE,
    MODE_CURSOR, MODE_CURSOR_BLINKING, MODE_CURSOR_VERY_VISIBLE, MODE_MOUSE_ALL, MODE_MOUSE_BUTTON,
    MODE_MOUSE_STANDARD, MODE_SYNC,
};
pub use crate::src::shared::screen_write::screen_write_cline;
pub use crate::src::shared::session::{session, session_entry, session_gentry};
pub use crate::src::shared::spawn::spawn_editor_state;
pub use crate::src::shared::status::status_line;
use crate::src::shared::style::*;
use crate::src::shared::terminal::*;
pub use crate::src::shared::tree::RB_NEGINF;
use crate::src::shared::tty::*;
pub use crate::src::shared::tty::{tty, tty_code, tty_key, tty_term, tty_term_entry};
pub use crate::src::shared::tty::{TTY_BLOCK, TTY_FREEZE, TTY_NOCURSOR, TTY_OPENED};
pub use crate::src::shared::vis::{VIS_CSTYLE, VIS_NOSLASH, VIS_OCTAL};
pub use crate::src::shared::window::{
    window, window_alerts_entry, window_entry, window_mode, window_mode_entry,
    window_mode_entry_entry, window_winlinks, winlink, winlink_entry, winlink_sentry,
    winlink_stack, winlink_wentry, winlinks,
};
pub use crate::src::shared::window::{
    WINDOW_RESIZE, WINDOW_SIZE_LATEST, WINLINK_ACTIVITY, WINLINK_ALERTFLAGS, WINLINK_BELL,
    WINLINK_SILENCE,
};

pub use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_14;
pub use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_13;

pub const _PATH_TTY: [::core::ffi::c_char; 9] =
    unsafe { ::core::mem::transmute::<[u8; 9], [::core::ffi::c_char; 9]>(*b"/dev/tty\0") };

#[no_mangle]
pub unsafe extern "C" fn server_client_how_many() -> u_int {
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut n: u_int = 0;
    n = 0 as u_int;
    c = clients.tqh_first;
    while !c.is_null() {
        if !(*c).session.is_null() && !(*c).flags & CLIENT_UNATTACHEDFLAGS as uint64_t != 0 {
            n = n.wrapping_add(1);
        }
        c = (*c).entry.tqe_next;
    }
    return n;
}
unsafe extern "C" fn server_client_overlay_timer(
    mut fd: ::core::ffi::c_int,
    mut events: ::core::ffi::c_short,
    mut data: *mut ::core::ffi::c_void,
) {
    server_client_clear_overlay(data as *mut client);
}
#[no_mangle]
pub unsafe extern "C" fn server_client_set_overlay(
    mut c: *mut client,
    mut delay: u_int,
    mut checkcb: overlay_check_cb,
    mut modecb: overlay_mode_cb,
    mut drawcb: overlay_draw_cb,
    mut keycb: overlay_key_cb,
    mut freecb: overlay_free_cb,
    mut resizecb: overlay_resize_cb,
    mut data: *mut ::core::ffi::c_void,
) {
    let mut tv: timeval = timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    if (*c).overlay_draw.is_some() {
        server_client_clear_overlay(c);
    }
    tv.tv_sec = delay.wrapping_div(1000 as u_int) as __time_t;
    tv.tv_usec = (delay.wrapping_rem(1000 as u_int) as ::core::ffi::c_long
        * 1000 as ::core::ffi::c_long) as __suseconds_t;
    if event_initialized(&raw mut (*c).overlay_timer) != 0 {
        event_del(&raw mut (*c).overlay_timer);
    }
    event_set(
        &raw mut (*c).overlay_timer,
        -(1 as ::core::ffi::c_int),
        0 as ::core::ffi::c_short,
        Some(
            server_client_overlay_timer
                as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    ::core::ffi::c_short,
                    *mut ::core::ffi::c_void,
                ) -> (),
        ),
        c as *mut ::core::ffi::c_void,
    );
    if delay != 0 as u_int {
        event_add(&raw mut (*c).overlay_timer, &raw mut tv);
    }
    (*c).overlay_check = checkcb;
    (*c).overlay_mode = modecb;
    (*c).overlay_draw = drawcb;
    (*c).overlay_key = keycb;
    (*c).overlay_free = freecb;
    (*c).overlay_resize = resizecb;
    (*c).overlay_data = data;
    if (*c).overlay_check.is_none() {
        (*c).tty.flags |= TTY_FREEZE;
    }
    if (*c).overlay_mode.is_none() {
        (*c).tty.flags |= TTY_NOCURSOR;
    }
    window_update_focus((*(*(*c).session).curw).window);
    server_redraw_client(c);
}
#[no_mangle]
pub unsafe extern "C" fn server_client_clear_overlay(mut c: *mut client) {
    if (*c).overlay_draw.is_none() {
        return;
    }
    if event_initialized(&raw mut (*c).overlay_timer) != 0 {
        event_del(&raw mut (*c).overlay_timer);
    }
    if (*c).overlay_free.is_some() {
        (*c).overlay_free.expect("non-null function pointer")(c, (*c).overlay_data);
    }
    (*c).overlay_check = None;
    (*c).overlay_mode = None;
    (*c).overlay_draw = None;
    (*c).overlay_key = None;
    (*c).overlay_free = None;
    (*c).overlay_resize = None;
    (*c).overlay_data = NULL;
    (*c).tty.flags &= !(TTY_FREEZE | TTY_NOCURSOR);
    if !(*c).session.is_null() {
        window_update_focus((*(*(*c).session).curw).window);
    }
    server_redraw_client(c);
}
#[no_mangle]
pub unsafe extern "C" fn server_client_ranges_is_empty(
    mut r: *mut visible_ranges,
) -> ::core::ffi::c_int {
    let mut i: u_int = 0;
    i = 0 as u_int;
    while i < (*r).used {
        if (*(*r).ranges.offset(i as isize)).nx != 0 as u_int {
            return 0 as ::core::ffi::c_int;
        }
        i = i.wrapping_add(1);
    }
    return 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn server_client_ensure_ranges(mut r: *mut visible_ranges, mut n: u_int) {
    if (*r).size >= n {
        return;
    }
    (*r).ranges = xrecallocarray(
        (*r).ranges as *mut ::core::ffi::c_void,
        (*r).size as size_t,
        n as size_t,
        ::core::mem::size_of::<visible_range>() as size_t,
    ) as *mut visible_range;
    (*r).size = n;
}
#[no_mangle]
pub unsafe extern "C" fn server_client_overlay_range(
    mut x: u_int,
    mut y: u_int,
    mut sx: u_int,
    mut sy: u_int,
    mut px: u_int,
    mut py: u_int,
    mut nx: u_int,
    mut r: *mut visible_ranges,
) {
    let mut ox: u_int = 0;
    let mut onx: u_int = 0;
    if py < y || py > y.wrapping_add(sy).wrapping_sub(1 as u_int) {
        server_client_ensure_ranges(r, 1 as u_int);
        (*(*r).ranges.offset(0 as ::core::ffi::c_int as isize)).px = px;
        (*(*r).ranges.offset(0 as ::core::ffi::c_int as isize)).nx = nx;
        (*r).used = 1 as u_int;
        return;
    }
    server_client_ensure_ranges(r, 2 as u_int);
    if px < x {
        (*(*r).ranges.offset(0 as ::core::ffi::c_int as isize)).px = px;
        (*(*r).ranges.offset(0 as ::core::ffi::c_int as isize)).nx = x.wrapping_sub(px);
        if (*(*r).ranges.offset(0 as ::core::ffi::c_int as isize)).nx > nx {
            (*(*r).ranges.offset(0 as ::core::ffi::c_int as isize)).nx = nx;
        }
    } else {
        (*(*r).ranges.offset(0 as ::core::ffi::c_int as isize)).px = 0 as u_int;
        (*(*r).ranges.offset(0 as ::core::ffi::c_int as isize)).nx = 0 as u_int;
    }
    ox = x.wrapping_add(sx);
    if px > ox {
        ox = px;
    }
    onx = px.wrapping_add(nx);
    if onx > ox {
        (*(*r).ranges.offset(1 as ::core::ffi::c_int as isize)).px = ox;
        (*(*r).ranges.offset(1 as ::core::ffi::c_int as isize)).nx = onx.wrapping_sub(ox);
    } else {
        (*(*r).ranges.offset(1 as ::core::ffi::c_int as isize)).px = 0 as u_int;
        (*(*r).ranges.offset(1 as ::core::ffi::c_int as isize)).nx = 0 as u_int;
    }
    (*r).used = 2 as u_int;
}
#[no_mangle]
pub unsafe extern "C" fn server_client_check_nested(mut c: *mut client) -> ::core::ffi::c_int {
    let mut envent: *mut environ_entry = ::core::ptr::null_mut::<environ_entry>();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    envent = environ_find(
        (*c).environ,
        b"TMUX\0" as *const u8 as *const ::core::ffi::c_char,
    );
    if envent.is_null() || *(*envent).value as ::core::ffi::c_int == '\0' as i32 {
        return 0 as ::core::ffi::c_int;
    }
    wp = window_pane_tree_minmax(&raw mut all_window_panes, RB_NEGINF);
    while !wp.is_null() {
        if strcmp(&raw mut (*wp).tty as *mut ::core::ffi::c_char, (*c).ttyname)
            == 0 as ::core::ffi::c_int
        {
            return 1 as ::core::ffi::c_int;
        }
        wp = window_pane_tree_next(wp);
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn server_client_set_key_table(
    mut c: *mut client,
    mut name: *const ::core::ffi::c_char,
) {
    if name.is_null() {
        name = server_client_get_key_table(c);
    }
    key_bindings_unref_table((*c).keytable as *mut key_table);
    (*c).keytable = key_bindings_get_table(name, 1 as ::core::ffi::c_int) as *mut key_table;
    (*(*c).keytable).references = (*(*c).keytable).references.wrapping_add(1);
    if gettimeofday(&raw mut (*(*c).keytable).activity_time, NULL) != 0 as ::core::ffi::c_int {
        fatal(b"gettimeofday failed\0" as *const u8 as *const ::core::ffi::c_char);
    }
}
unsafe extern "C" fn server_client_key_table_activity_diff(mut c: *mut client) -> uint64_t {
    let mut diff: timeval = timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    diff.tv_sec = (*c).activity_time.tv_sec - (*(*c).keytable).activity_time.tv_sec;
    diff.tv_usec = (*c).activity_time.tv_usec - (*(*c).keytable).activity_time.tv_usec;
    if diff.tv_usec < 0 as __suseconds_t {
        diff.tv_sec -= 1;
        diff.tv_usec += 1000000 as __suseconds_t;
    }
    return (diff.tv_sec as ::core::ffi::c_ulonglong)
        .wrapping_mul(1000 as ::core::ffi::c_ulonglong)
        .wrapping_add(
            (diff.tv_usec as ::core::ffi::c_ulonglong)
                .wrapping_div(1000 as ::core::ffi::c_ulonglong),
        ) as uint64_t;
}
#[no_mangle]
pub unsafe extern "C" fn server_client_get_key_table(
    mut c: *mut client,
) -> *const ::core::ffi::c_char {
    let mut s: *mut session = (*c).session;
    let mut name: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if s.is_null() {
        return b"root\0" as *const u8 as *const ::core::ffi::c_char;
    }
    name = options_get_string(
        (*s).options,
        b"key-table\0" as *const u8 as *const ::core::ffi::c_char,
    );
    if *name as ::core::ffi::c_int == '\0' as i32 {
        return b"root\0" as *const u8 as *const ::core::ffi::c_char;
    }
    return name;
}
unsafe extern "C" fn server_client_is_default_key_table(
    mut c: *mut client,
    mut table: *mut key_table,
) -> ::core::ffi::c_int {
    return (strcmp((*table).name, server_client_get_key_table(c)) == 0 as ::core::ffi::c_int)
        as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn server_client_create(mut fd: ::core::ffi::c_int) -> *mut client {
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut i: u_int = 0;
    setblocking(fd, 0 as ::core::ffi::c_int);
    c = &raw mut (*Box::into_raw(Box::new(ClientOwner {
        node: std::mem::zeroed::<client>(),
        message: None,
        saved_status_screen: None,
        term_cap_strings: Vec::new(),
        term_cap_ptrs: Vec::new(),
    })))
    .node;
    (*c).references = 1 as ::core::ffi::c_int;
    (*c).peer = proc_add_peer(
        server_proc,
        fd,
        Some(
            server_client_dispatch
                as unsafe extern "C" fn(*mut imsg, *mut ::core::ffi::c_void) -> (),
        ),
        c as *mut ::core::ffi::c_void,
    );
    if gettimeofday(&raw mut (*c).creation_time, NULL) != 0 as ::core::ffi::c_int {
        fatal(b"gettimeofday failed\0" as *const u8 as *const ::core::ffi::c_char);
    }
    memcpy(
        &raw mut (*c).activity_time as *mut ::core::ffi::c_void,
        &raw mut (*c).creation_time as *const ::core::ffi::c_void,
        ::core::mem::size_of::<timeval>() as size_t,
    );
    (*c).environ = environ_create();
    (*c).fd = -(1 as ::core::ffi::c_int);
    (*c).out_fd = -(1 as ::core::ffi::c_int);
    (*c).queue = cmdq_new();
    (*c).files.storage = None;
    (*c).tty.sx = 80 as u_int;
    (*c).tty.sy = 24 as u_int;
    i = 0 as u_int;
    while i < COLOUR_THEME_COUNT as u_int {
        (*c).theme_colours[i as usize] = 8 as ::core::ffi::c_int;
        i = i.wrapping_add(1);
    }
    (*c).theme = THEME_UNKNOWN;
    status_init(c);
    (*c).flags |= CLIENT_FOCUSED as uint64_t;
    (*c).keytable = key_bindings_get_table(
        b"root\0" as *const u8 as *const ::core::ffi::c_char,
        1 as ::core::ffi::c_int,
    ) as *mut key_table;
    (*(*c).keytable).references = (*(*c).keytable).references.wrapping_add(1);
    event_set(
        &raw mut (*c).repeat_timer,
        -(1 as ::core::ffi::c_int),
        0 as ::core::ffi::c_short,
        Some(
            server_client_repeat_timer
                as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    ::core::ffi::c_short,
                    *mut ::core::ffi::c_void,
                ) -> (),
        ),
        c as *mut ::core::ffi::c_void,
    );
    event_set(
        &raw mut (*c).click_timer,
        -(1 as ::core::ffi::c_int),
        0 as ::core::ffi::c_short,
        Some(
            server_client_click_timer
                as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    ::core::ffi::c_short,
                    *mut ::core::ffi::c_void,
                ) -> (),
        ),
        c as *mut ::core::ffi::c_void,
    );
    event_set(
        &raw mut (*c).exit_timer,
        -(1 as ::core::ffi::c_int),
        0 as ::core::ffi::c_short,
        Some(
            server_client_exit_timer
                as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    ::core::ffi::c_short,
                    *mut ::core::ffi::c_void,
                ) -> (),
        ),
        c as *mut ::core::ffi::c_void,
    );
    (*c).click_wp = -(1 as ::core::ffi::c_int);
    (*c).input_requests.tqh_first = ::core::ptr::null_mut::<input_request>();
    (*c).input_requests.tqh_last = &raw mut (*c).input_requests.tqh_first;
    (*c).entry.tqe_next = ::core::ptr::null_mut::<client>();
    (*c).entry.tqe_prev = clients.tqh_last;
    *clients.tqh_last = c;
    clients.tqh_last = &raw mut (*c).entry.tqe_next;
    log_debug(
        b"new client %p\0" as *const u8 as *const ::core::ffi::c_char,
        c,
    );
    return c;
}
#[no_mangle]
pub unsafe extern "C" fn server_client_open(
    mut c: *mut client,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut ttynam: *const ::core::ffi::c_char = _PATH_TTY.as_ptr();
    if (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
        return 0 as ::core::ffi::c_int;
    }
    if strcmp((*c).ttyname, ttynam) == 0 as ::core::ffi::c_int
        || (isatty(STDIN_FILENO) != 0
            && {
                ttynam = ttyname(STDIN_FILENO);
                !ttynam.is_null()
            }
            && strcmp((*c).ttyname, ttynam) == 0 as ::core::ffi::c_int
            || isatty(STDOUT_FILENO) != 0
                && {
                    ttynam = ttyname(STDOUT_FILENO);
                    !ttynam.is_null()
                }
                && strcmp((*c).ttyname, ttynam) == 0 as ::core::ffi::c_int
            || isatty(STDERR_FILENO) != 0
                && {
                    ttynam = ttyname(STDERR_FILENO);
                    !ttynam.is_null()
                }
                && strcmp((*c).ttyname, ttynam) == 0 as ::core::ffi::c_int)
    {
        xasprintf(
            cause,
            b"can't use %s\0" as *const u8 as *const ::core::ffi::c_char,
            (*c).ttyname,
        );
        return -(1 as ::core::ffi::c_int);
    }
    if (*c).flags & CLIENT_TERMINAL as uint64_t == 0 {
        *cause = xstrdup(b"not a terminal\0" as *const u8 as *const ::core::ffi::c_char);
        return -(1 as ::core::ffi::c_int);
    }
    if tty_open(&raw mut (*c).tty, cause) != 0 as ::core::ffi::c_int {
        return -(1 as ::core::ffi::c_int);
    }
    server_client_update_theme_colours(c);
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn server_client_attached_lost(mut c: *mut client) {
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut loop_0: *mut client = ::core::ptr::null_mut::<client>();
    let mut found: *mut client = ::core::ptr::null_mut::<client>();
    log_debug(
        b"lost attached client %p\0" as *const u8 as *const ::core::ffi::c_char,
        c,
    );
    w = windows_minmax(&raw mut windows, RB_NEGINF);
    while !w.is_null() {
        if !((*w).latest != c as *mut ::core::ffi::c_void) {
            found = ::core::ptr::null_mut::<client>();
            loop_0 = clients.tqh_first;
            while !loop_0.is_null() {
                s = (*loop_0).session;
                if !(loop_0 == c || s.is_null() || (*(*s).curw).window != w) {
                    if found.is_null()
                        || (if (*loop_0).activity_time.tv_sec == (*found).activity_time.tv_sec {
                            ((*loop_0).activity_time.tv_usec > (*found).activity_time.tv_usec)
                                as ::core::ffi::c_int
                        } else {
                            ((*loop_0).activity_time.tv_sec > (*found).activity_time.tv_sec)
                                as ::core::ffi::c_int
                        }) != 0
                    {
                        found = loop_0;
                    }
                }
                loop_0 = (*loop_0).entry.tqe_next;
            }
            if !found.is_null() {
                server_client_update_latest(found);
            }
        }
        w = windows_next(w);
    }
}
unsafe extern "C" fn server_client_fire_session_changed(mut c: *mut client, mut old: *mut session) {
    let mut ep: *mut event_payload = ::core::ptr::null_mut::<event_payload>();
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null_mut::<cmd_find_state>(),
        s: ::core::ptr::null_mut::<session>(),
        wl: ::core::ptr::null_mut::<winlink>(),
        w: ::core::ptr::null_mut::<window>(),
        wp: ::core::ptr::null_mut::<window_pane>(),
        idx: 0,
    };
    ep = event_payload_create();
    cmd_find_from_client(&raw mut fs, c, 0 as ::core::ffi::c_int);
    event_payload_set_target(ep, &raw mut fs);
    event_payload_set_client(
        ep,
        b"client\0" as *const u8 as *const ::core::ffi::c_char,
        c,
    );
    if !fs.s.is_null() {
        event_payload_set_session(
            ep,
            b"session\0" as *const u8 as *const ::core::ffi::c_char,
            fs.s,
        );
        event_payload_set_session(
            ep,
            b"new_session\0" as *const u8 as *const ::core::ffi::c_char,
            fs.s,
        );
    }
    if !old.is_null() {
        event_payload_set_session(
            ep,
            b"old_session\0" as *const u8 as *const ::core::ffi::c_char,
            old,
        );
    }
    if !fs.w.is_null() {
        event_payload_set_window(
            ep,
            b"window\0" as *const u8 as *const ::core::ffi::c_char,
            fs.w,
        );
    }
    if !fs.wl.is_null() {
        event_payload_set_int(
            ep,
            b"window_index\0" as *const u8 as *const ::core::ffi::c_char,
            (*fs.wl).idx,
        );
    } else if fs.idx != -(1 as ::core::ffi::c_int) {
        event_payload_set_int(
            ep,
            b"window_index\0" as *const u8 as *const ::core::ffi::c_char,
            fs.idx,
        );
    }
    if !fs.wp.is_null() {
        event_payload_set_pane(
            ep,
            b"pane\0" as *const u8 as *const ::core::ffi::c_char,
            fs.wp,
        );
    }
    events_fire(
        b"client-session-changed\0" as *const u8 as *const ::core::ffi::c_char,
        ep,
    );
}
unsafe extern "C" fn server_client_fire_resized(
    mut c: *mut client,
    mut old_sx: u_int,
    mut old_sy: u_int,
) {
    let mut ep: *mut event_payload = ::core::ptr::null_mut::<event_payload>();
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null_mut::<cmd_find_state>(),
        s: ::core::ptr::null_mut::<session>(),
        wl: ::core::ptr::null_mut::<winlink>(),
        w: ::core::ptr::null_mut::<window>(),
        wp: ::core::ptr::null_mut::<window_pane>(),
        idx: 0,
    };
    ep = event_payload_create();
    cmd_find_from_client(&raw mut fs, c, 0 as ::core::ffi::c_int);
    event_payload_set_target(ep, &raw mut fs);
    event_payload_set_client(
        ep,
        b"client\0" as *const u8 as *const ::core::ffi::c_char,
        c,
    );
    if !fs.s.is_null() {
        event_payload_set_session(
            ep,
            b"session\0" as *const u8 as *const ::core::ffi::c_char,
            fs.s,
        );
    }
    if !fs.w.is_null() {
        event_payload_set_window(
            ep,
            b"window\0" as *const u8 as *const ::core::ffi::c_char,
            fs.w,
        );
    }
    if !fs.wl.is_null() {
        event_payload_set_int(
            ep,
            b"window_index\0" as *const u8 as *const ::core::ffi::c_char,
            (*fs.wl).idx,
        );
    } else if fs.idx != -(1 as ::core::ffi::c_int) {
        event_payload_set_int(
            ep,
            b"window_index\0" as *const u8 as *const ::core::ffi::c_char,
            fs.idx,
        );
    }
    if !fs.wp.is_null() {
        event_payload_set_pane(
            ep,
            b"pane\0" as *const u8 as *const ::core::ffi::c_char,
            fs.wp,
        );
    }
    event_payload_set_uint(
        ep,
        b"width\0" as *const u8 as *const ::core::ffi::c_char,
        (*c).tty.sx,
    );
    event_payload_set_uint(
        ep,
        b"height\0" as *const u8 as *const ::core::ffi::c_char,
        (*c).tty.sy,
    );
    event_payload_set_uint(
        ep,
        b"old_width\0" as *const u8 as *const ::core::ffi::c_char,
        old_sx,
    );
    event_payload_set_uint(
        ep,
        b"old_height\0" as *const u8 as *const ::core::ffi::c_char,
        old_sy,
    );
    events_fire(
        b"client-resized\0" as *const u8 as *const ::core::ffi::c_char,
        ep,
    );
}
#[no_mangle]
pub unsafe extern "C" fn server_client_set_session(mut c: *mut client, mut s: *mut session) {
    let mut old: *mut session = (*c).session;
    if !s.is_null() && !(*c).session.is_null() && (*c).session != s {
        (*c).last_session = (*c).session;
    } else if s.is_null() {
        (*c).last_session = ::core::ptr::null_mut::<session>();
    }
    (*c).session = s;
    (*c).flags |= CLIENT_FOCUSED as uint64_t;
    if !old.is_null() && !(*old).curw.is_null() {
        window_update_focus((*(*old).curw).window);
    }
    if !s.is_null() {
        (*(*(*s).curw).window).latest = c as *mut ::core::ffi::c_void;
        recalculate_sizes();
        window_update_focus((*(*s).curw).window);
        session_update_activity(s, ::core::ptr::null_mut::<timeval>());
        session_theme_changed(s);
        gettimeofday(&raw mut (*s).last_attached_time, NULL);
        (*(*s).curw).flags &= !WINLINK_ALERTFLAGS;
        alerts_check_session(s);
        tty_update_client_offset(c);
        status_timer_start(c);
        server_client_fire_session_changed(c, old);
        server_redraw_client(c);
    }
    server_check_unattached();
    server_update_socket();
}
#[no_mangle]
pub unsafe extern "C" fn server_client_lost(mut c: *mut client) {
    let mut cf: *mut client_file = ::core::ptr::null_mut::<client_file>();
    let mut cf1: *mut client_file = ::core::ptr::null_mut::<client_file>();
    if cfg_client == c {
        cfg_client = ::core::ptr::null_mut::<client>();
    }
    (*c).flags |= CLIENT_DEAD as uint64_t;
    server_client_clear_overlay(c);
    status_prompt_clear(c);
    status_message_clear(c);
    cf = client_files_minmax(&raw mut (*c).files, RB_NEGINF);
    while !cf.is_null() && {
        cf1 = client_files_next(cf);
        1 as ::core::ffi::c_int != 0
    } {
        (*cf).error = EINTR;
        file_fire_done(cf);
        cf = cf1;
    }
    if !(*c).entry.tqe_next.is_null() {
        (*(*c).entry.tqe_next).entry.tqe_prev = (*c).entry.tqe_prev;
    } else {
        clients.tqh_last = (*c).entry.tqe_prev;
    }
    *(*c).entry.tqe_prev = (*c).entry.tqe_next;
    log_debug(
        b"lost client %p\0" as *const u8 as *const ::core::ffi::c_char,
        c,
    );
    if (*c).flags & CLIENT_ATTACHED as uint64_t != 0 {
        server_client_attached_lost(c);
        events_fire_client(
            b"client-detached\0" as *const u8 as *const ::core::ffi::c_char,
            c,
        );
    }
    if !(*c).name.is_null() && (*c).flags & (CLIENT_CONTROL | CLIENT_TERMINAL) as uint64_t != 0 {
        events_fire_client(
            b"client-closed\0" as *const u8 as *const ::core::ffi::c_char,
            c,
        );
    }
    if (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
        control_stop(c);
    }
    if (*c).flags & CLIENT_TERMINAL as uint64_t != 0 {
        tty_free(&raw mut (*c).tty);
    }
    free((*c).ttyname as *mut ::core::ffi::c_void);
    free((*c).clipboard_panes as *mut ::core::ffi::c_void);
    free((*c).term_name as *mut ::core::ffi::c_void);
    free((*c).term_type as *mut ::core::ffi::c_void);
    server_client_clear_term_caps(c);
    status_free(c);
    input_cancel_requests(c);
    free((*c).title as *mut ::core::ffi::c_void);
    free((*c).path as *mut ::core::ffi::c_void);
    free((*c).cwd as *mut ::core::ffi::c_void);
    free((*c).exit_session as *mut ::core::ffi::c_void);
    free((*c).exit_message as *mut ::core::ffi::c_void);
    event_del(&raw mut (*c).repeat_timer);
    event_del(&raw mut (*c).click_timer);
    event_del(&raw mut (*c).exit_timer);
    if event_initialized(&raw mut (*c).cycle_timer) != 0 {
        event_del(&raw mut (*c).cycle_timer);
    }
    key_bindings_unref_table((*c).keytable as *mut key_table);
    // Callbacks during client loss can set another message after the earlier
    // clear. Preserve the final release point before cancelling its timer.
    server_client_set_message(c, None);
    if event_initialized(&raw mut (*c).message_timer) != 0 {
        event_del(&raw mut (*c).message_timer);
    }
    prompt_free((*c).prompt);
    format_lost_client(c);
    environ_free((*c).environ);
    proc_remove_peer((*c).peer);
    (*c).peer = ::core::ptr::null_mut::<tmuxpeer>();
    if (*c).out_fd != -(1 as ::core::ffi::c_int) {
        close((*c).out_fd);
    }
    if (*c).fd != -(1 as ::core::ffi::c_int) {
        close((*c).fd);
        (*c).fd = -(1 as ::core::ffi::c_int);
    }
    server_client_unref(c);
    server_add_accept(0 as ::core::ffi::c_int);
    recalculate_sizes();
    server_check_unattached();
    server_update_socket();
}
#[no_mangle]
pub unsafe extern "C" fn server_client_unref(mut c: *mut client) {
    log_debug(
        b"unref client %p (%d references)\0" as *const u8 as *const ::core::ffi::c_char,
        c,
        (*c).references,
    );
    (*c).references -= 1;
    if (*c).references == 0 as ::core::ffi::c_int {
        event_once(
            -(1 as ::core::ffi::c_int),
            EV_TIMEOUT as ::core::ffi::c_short,
            Some(
                server_client_free
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        ::core::ffi::c_short,
                        *mut ::core::ffi::c_void,
                    ) -> (),
            ),
            c as *mut ::core::ffi::c_void,
            ::core::ptr::null::<timeval>(),
        );
    }
}
unsafe extern "C" fn server_client_free(
    mut fd: ::core::ffi::c_int,
    mut events: ::core::ffi::c_short,
    mut arg: *mut ::core::ffi::c_void,
) {
    let mut c: *mut client = arg as *mut client;
    log_debug(
        b"free client %p (%d references)\0" as *const u8 as *const ::core::ffi::c_char,
        c,
        (*c).references,
    );
    redraw_free_scene((*c).redraw_scene);
    cmdq_free((*c).queue);
    if (*c).references == 0 as ::core::ffi::c_int {
        free((*c).name as *mut ::core::ffi::c_void);
        free((*c).user as *mut ::core::ffi::c_void);
        assert!(
            (*c).files.storage.is_none(),
            "client file index still contains live records at client teardown"
        );
        drop(Box::from_raw(c as *mut ClientOwner));
    }
}
#[no_mangle]
pub unsafe extern "C" fn server_client_suspend(mut c: *mut client) {
    let mut s: *mut session = (*c).session;
    if s.is_null() || (*c).flags & CLIENT_UNATTACHEDFLAGS as uint64_t != 0 {
        return;
    }
    tty_stop_tty(&raw mut (*c).tty);
    (*c).flags |= CLIENT_SUSPENDED as uint64_t;
    proc_send(
        (*c).peer,
        MSG_SUSPEND,
        -(1 as ::core::ffi::c_int),
        ::core::ptr::null::<::core::ffi::c_void>(),
        0 as size_t,
    );
}
#[no_mangle]
pub unsafe extern "C" fn server_client_detach(mut c: *mut client, mut msgtype: msgtype) {
    let mut s: *mut session = (*c).session;
    if s.is_null() || (*c).flags & CLIENT_NODETACHFLAGS as uint64_t != 0 {
        return;
    }
    (*c).flags |= CLIENT_EXIT as uint64_t;
    (*c).exit_type = CLIENT_EXIT_DETACH;
    (*c).exit_msgtype = msgtype;
    (*c).exit_session = xstrdup((*s).name);
}
#[no_mangle]
pub unsafe extern "C" fn server_client_exec(
    mut c: *mut client,
    mut cmd: *const ::core::ffi::c_char,
) {
    let mut s: *mut session = (*c).session;
    let mut shell: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if *cmd as ::core::ffi::c_int == '\0' as i32 {
        return;
    }
    if !s.is_null() {
        shell = options_get_string(
            (*s).options,
            b"default-shell\0" as *const u8 as *const ::core::ffi::c_char,
        );
    } else {
        shell = options_get_string(
            global_s_options,
            b"default-shell\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    if checkshell(shell) == 0 {
        shell = _PATH_BSHELL.as_ptr();
    }
    let cmd_bytes = CStr::from_ptr(cmd).to_bytes_with_nul();
    let shell_bytes = CStr::from_ptr(shell).to_bytes_with_nul();
    let mut msg = Vec::with_capacity(cmd_bytes.len() + shell_bytes.len());
    msg.extend_from_slice(cmd_bytes);
    msg.extend_from_slice(shell_bytes);
    proc_send(
        (*c).peer,
        MSG_EXEC,
        -(1 as ::core::ffi::c_int),
        msg.as_ptr().cast(),
        msg.len(),
    );
}
unsafe extern "C" fn server_client_in_scrollbar_area(
    mut wp: *mut window_pane,
    mut px: ::core::ffi::c_int,
    mut py: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut w: *mut window = (*wp).window as *mut window;
    let mut width: u_int = 0;
    let mut pad: u_int = 0;
    let mut total: u_int = 0;
    let mut start: ::core::ffi::c_int = 0;
    let mut end: ::core::ffi::c_int = 0;
    if window_pane_scrollbar_overlay(wp) == 0 {
        return 0 as ::core::ffi::c_int;
    }
    if py < (*wp).yoff || py >= (*wp).yoff + (*wp).sy as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    width = (*wp).scrollbar_style.width as u_int;
    pad = (*wp).scrollbar_style.pad as u_int;
    total = width.wrapping_add(pad);
    if total == 0 as u_int || total > (*wp).sx {
        total = (*wp).sx;
    }
    if (*w).sb_pos == PANE_SCROLLBARS_LEFT {
        start = (*wp).xoff;
        end = (*wp).xoff + total as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
    } else {
        end = (*wp).xoff + (*wp).sx as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
        start = end - total as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
    }
    return (px >= start && px <= end) as ::core::ffi::c_int;
}
unsafe extern "C" fn server_client_update_scrollbar_hover(
    mut c: *mut client,
    mut type_0: ::core::ffi::c_int,
    mut px: ::core::ffi::c_int,
    mut py: ::core::ffi::c_int,
) {
    let mut w: *mut window = (*(*(*c).session).curw).window;
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    if type_0 != KEYC_TYPE_MOUSEMOVE as ::core::ffi::c_int {
        return;
    }
    wp = (*w).panes.tqh_first;
    while !wp.is_null() {
        if !(window_pane_is_visible(wp) == 0) {
            if server_client_in_scrollbar_area(wp, px, py) != 0 {
                (*wp).sb_auto_hover = 1 as ::core::ffi::c_int;
                window_pane_scrollbar_show(wp, 1 as ::core::ffi::c_int);
            } else {
                (*wp).sb_auto_hover = 0 as ::core::ffi::c_int;
                window_pane_scrollbar_start_timer(wp);
            }
        }
        wp = (*wp).entry.tqe_next;
    }
}
unsafe extern "C" fn server_client_check_mouse_in_pane(
    mut wp: *mut window_pane,
    mut px: ::core::ffi::c_int,
    mut py: ::core::ffi::c_int,
    mut sl_mpos: *mut u_int,
) -> key_code_mouse_location {
    let mut w: *mut window = (*wp).window as *mut window;
    let mut fwp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut pane_status: ::core::ffi::c_int = 0;
    let mut sb_w: ::core::ffi::c_int = 0;
    let mut sb_pad: ::core::ffi::c_int = 0;
    let mut pane_status_line: ::core::ffi::c_int = 0;
    let mut sl_top: ::core::ffi::c_int = 0;
    let mut sl_bottom: ::core::ffi::c_int = 0;
    let mut bdr_bottom: ::core::ffi::c_int = 0;
    let mut bdr_top: ::core::ffi::c_int = 0;
    let mut bdr_left: ::core::ffi::c_int = 0;
    let mut bdr_right: ::core::ffi::c_int = 0;
    let mut sb_start: ::core::ffi::c_int = 0;
    let mut sb_end: ::core::ffi::c_int = 0;
    let mut sb_overlay: ::core::ffi::c_int = 0;
    pane_status = window_pane_get_pane_status(wp);
    sb_overlay = window_pane_scrollbar_overlay(wp);
    if window_pane_scrollbar_visible(wp) != 0 {
        sb_w = (*wp).scrollbar_style.width;
        sb_pad = (*wp).scrollbar_style.pad;
        if sb_overlay != 0 && sb_w > (*wp).sx as ::core::ffi::c_int {
            sb_w = (*wp).sx as ::core::ffi::c_int;
        }
    } else {
        sb_w = 0 as ::core::ffi::c_int;
        sb_pad = 0 as ::core::ffi::c_int;
    }
    if pane_status == PANE_STATUS_TOP {
        pane_status_line = (*wp).yoff - 1 as ::core::ffi::c_int;
    } else if pane_status == PANE_STATUS_BOTTOM {
        pane_status_line = ((*wp).yoff as u_int).wrapping_add((*wp).sy) as ::core::ffi::c_int;
    } else {
        pane_status_line = -(1 as ::core::ffi::c_int);
    }
    bdr_left = (*wp).xoff - 1 as ::core::ffi::c_int;
    if sb_overlay == 0 && (*w).sb_pos == PANE_SCROLLBARS_LEFT {
        bdr_left -= sb_pad + sb_w;
    }
    if sb_overlay != 0
        && sb_w != 0 as ::core::ffi::c_int
        && py >= (*wp).yoff
        && py < (*wp).yoff + (*wp).sy as ::core::ffi::c_int
        && px >= (*wp).xoff
        && px < (*wp).xoff + (*wp).sx as ::core::ffi::c_int
    {
        if (*w).sb_pos == PANE_SCROLLBARS_LEFT {
            sb_start = (*wp).xoff;
            sb_end = sb_start + sb_w - 1 as ::core::ffi::c_int;
        } else {
            sb_end = (*wp).xoff + (*wp).sx as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
            sb_start = sb_end - sb_w + 1 as ::core::ffi::c_int;
        }
        if px >= sb_start && px <= sb_end {
            sl_top = ((*wp).yoff as u_int).wrapping_add((*wp).sb_slider_y) as ::core::ffi::c_int;
            sl_bottom = ((*wp).yoff as u_int)
                .wrapping_add((*wp).sb_slider_y)
                .wrapping_add((*wp).sb_slider_h)
                .wrapping_sub(1 as u_int) as ::core::ffi::c_int;
            if py < sl_top {
                return KEYC_MOUSE_LOCATION_SCROLLBAR_UP;
            } else if py >= sl_top && py <= sl_bottom {
                *sl_mpos = (py as u_int)
                    .wrapping_sub((*wp).sb_slider_y)
                    .wrapping_sub((*wp).yoff as u_int);
                return KEYC_MOUSE_LOCATION_SCROLLBAR_SLIDER;
            } else {
                return KEYC_MOUSE_LOCATION_SCROLLBAR_DOWN;
            }
        }
        return KEYC_MOUSE_LOCATION_PANE;
    }
    if (pane_status != PANE_STATUS_OFF
        && py != pane_status_line
        && py != (*wp).yoff + (*wp).sy as ::core::ffi::c_int
        || (*wp).yoff == 0 as ::core::ffi::c_int && py < (*wp).sy as ::core::ffi::c_int
        || py >= (*wp).yoff && py < (*wp).yoff + (*wp).sy as ::core::ffi::c_int)
        && ((*w).sb_pos == PANE_SCROLLBARS_RIGHT
            && px < (*wp).xoff + (*wp).sx as ::core::ffi::c_int + sb_pad + sb_w
            || (*w).sb_pos == PANE_SCROLLBARS_LEFT
                && px < (*wp).xoff + (*wp).sx as ::core::ffi::c_int - sb_pad - sb_w)
    {
        if (*w).sb_pos == PANE_SCROLLBARS_RIGHT
            && (px >= (*wp).xoff + (*wp).sx as ::core::ffi::c_int + sb_pad
                && px < (*wp).xoff + (*wp).sx as ::core::ffi::c_int + sb_pad + sb_w)
            || (*w).sb_pos == PANE_SCROLLBARS_LEFT
                && (px >= (*wp).xoff - sb_pad - sb_w && px < (*wp).xoff - sb_pad)
        {
            sl_top = ((*wp).yoff as u_int).wrapping_add((*wp).sb_slider_y) as ::core::ffi::c_int;
            sl_bottom = ((*wp).yoff as u_int)
                .wrapping_add((*wp).sb_slider_y)
                .wrapping_add((*wp).sb_slider_h)
                .wrapping_sub(1 as u_int) as ::core::ffi::c_int;
            if py < sl_top {
                return KEYC_MOUSE_LOCATION_SCROLLBAR_UP;
            } else if py >= sl_top && py <= sl_bottom {
                *sl_mpos = (py as u_int)
                    .wrapping_sub((*wp).sb_slider_y)
                    .wrapping_sub((*wp).yoff as u_int);
                return KEYC_MOUSE_LOCATION_SCROLLBAR_SLIDER;
            } else {
                return KEYC_MOUSE_LOCATION_SCROLLBAR_DOWN;
            }
        } else if window_pane_is_floating(wp) != 0
            && window_pane_get_pane_lines(wp) as ::core::ffi::c_uint
                != PANE_LINES_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
            && (px == bdr_left
                || py == (*wp).yoff - 1 as ::core::ffi::c_int
                || py == (*wp).yoff + (*wp).sy as ::core::ffi::c_int)
        {
            return KEYC_MOUSE_LOCATION_BORDER;
        } else {
            return KEYC_MOUSE_LOCATION_PANE;
        }
    } else {
        fwp = (*w).panes.tqh_first;
        while !fwp.is_null() {
            if !(window_pane_is_visible(fwp) == 0) {
                if !(window_pane_is_floating(fwp) != 0
                    && window_pane_get_pane_lines(fwp) as ::core::ffi::c_uint
                        == PANE_LINES_NONE as ::core::ffi::c_int as ::core::ffi::c_uint)
                {
                    if window_pane_scrollbar_reserve(fwp) != 0 {
                        sb_w = (*fwp).scrollbar_style.width;
                        sb_pad = (*fwp).scrollbar_style.pad;
                    } else {
                        sb_w = 0 as ::core::ffi::c_int;
                        sb_pad = 0 as ::core::ffi::c_int;
                    }
                    bdr_top = (*fwp).yoff - 1 as ::core::ffi::c_int;
                    bdr_bottom =
                        ((*fwp).yoff as u_int).wrapping_add((*fwp).sy) as ::core::ffi::c_int;
                    bdr_left = (*fwp).xoff - 1 as ::core::ffi::c_int;
                    if (*w).sb_pos == PANE_SCROLLBARS_LEFT {
                        bdr_left -= sb_pad + sb_w;
                        bdr_right =
                            ((*fwp).xoff as u_int).wrapping_add((*fwp).sx) as ::core::ffi::c_int;
                    } else {
                        bdr_right = ((*fwp).xoff as u_int)
                            .wrapping_add((*fwp).sx)
                            .wrapping_add(sb_pad as u_int)
                            .wrapping_add(sb_w as u_int)
                            as ::core::ffi::c_int;
                    }
                    if py >= (*fwp).yoff - 1 as ::core::ffi::c_int
                        && py <= (*fwp).yoff + (*fwp).sy as ::core::ffi::c_int
                    {
                        if px == bdr_right {
                            break;
                        }
                        if window_pane_is_floating(wp) != 0 {
                            if px == bdr_left {
                                break;
                            }
                        }
                    }
                    if px >= bdr_left && px <= (*fwp).xoff + (*fwp).sx as ::core::ffi::c_int {
                        bdr_bottom =
                            ((*fwp).yoff as u_int).wrapping_add((*fwp).sy) as ::core::ffi::c_int;
                        if py == bdr_bottom {
                            break;
                        }
                        if py == bdr_top {
                            break;
                        }
                    }
                }
            }
            fwp = (*fwp).entry.tqe_next;
        }
        if !fwp.is_null() {
            return KEYC_MOUSE_LOCATION_BORDER;
        }
    }
    return KEYC_MOUSE_LOCATION_NOWHERE;
}
unsafe extern "C" fn server_client_check_mouse(
    mut c: *mut client,
    mut event: *mut key_event,
) -> key_code {
    let mut current_block: u64;
    let mut m: *mut mouse_event = &raw mut (*event).m;
    let mut s: *mut session = (*c).session;
    let mut fs: *mut session = ::core::ptr::null_mut::<session>();
    let mut w: *mut window = (*(*s).curw).window;
    let mut fwl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut fwp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut lwp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    let mut n: u_int = 0;
    let mut sl_mpos: u_int = 0 as u_int;
    let mut b: u_int = 0;
    let mut bn: u_int = 0;
    let mut ignore: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut modal_drag: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut key: key_code = 0;
    let mut tv: timeval = timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    let mut sr: *mut style_range = ::core::ptr::null_mut::<style_range>();
    let mut type_0: key_code_type = KEYC_TYPE_NOTYPE;
    let mut loc: key_code_mouse_location = KEYC_MOUSE_LOCATION_NOWHERE;
    log_debug(
        b"%s mouse %02x at %u,%u (last %u,%u) (%d)\0" as *const u8 as *const ::core::ffi::c_char,
        (*c).name,
        (*m).b,
        (*m).x,
        (*m).y,
        (*m).lx,
        (*m).ly,
        (*c).tty.mouse_drag_flag,
    );
    if (*c).tty.mouse_last_pane != -(1 as ::core::ffi::c_int) {
        lwp = window_pane_find_by_id((*c).tty.mouse_last_pane as u_int);
        if !lwp.is_null() {
            log_debug(
                b"%s mouse last pane %%%u\0" as *const u8 as *const ::core::ffi::c_char,
                (*c).name,
                (*lwp).id,
            );
        }
    }
    if (*event).key == KEYC_DOUBLECLICK as ::core::ffi::c_ulong as key_code {
        type_0 = KEYC_TYPE_DOUBLECLICK;
        x = (*m).x;
        y = (*m).y;
        b = (*m).b;
        ignore = 1 as ::core::ffi::c_int;
        log_debug(
            b"double-click at %u,%u\0" as *const u8 as *const ::core::ffi::c_char,
            x,
            y,
        );
    } else if (*m).sgr_type != ' ' as i32 as u_int
        && (*m).sgr_b & MOUSE_MASK_DRAG as u_int != 0
        && (*m).sgr_b & MOUSE_MASK_BUTTONS as u_int == 3 as u_int
        || (*m).sgr_type == ' ' as i32 as u_int
            && (*m).b & MOUSE_MASK_DRAG as u_int != 0
            && (*m).b & MOUSE_MASK_BUTTONS as u_int == 3 as u_int
            && (*m).lb & MOUSE_MASK_BUTTONS as u_int == 3 as u_int
    {
        type_0 = KEYC_TYPE_MOUSEMOVE;
        x = (*m).x;
        y = (*m).y;
        b = 0 as u_int;
        log_debug(
            b"move at %u,%u\0" as *const u8 as *const ::core::ffi::c_char,
            x,
            y,
        );
    } else if (*m).b & MOUSE_MASK_DRAG as u_int != 0 {
        type_0 = KEYC_TYPE_MOUSEDRAG;
        if (*c).tty.mouse_drag_flag != 0 {
            x = (*m).x;
            y = (*m).y;
            b = (*m).b;
            if x == (*m).lx && y == (*m).ly {
                return KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code;
            }
            log_debug(
                b"drag update at %u,%u\0" as *const u8 as *const ::core::ffi::c_char,
                x,
                y,
            );
        } else {
            x = (*m).lx;
            y = (*m).ly;
            b = (*m).lb;
            log_debug(
                b"drag start at %u,%u\0" as *const u8 as *const ::core::ffi::c_char,
                x,
                y,
            );
        }
    } else if (*m).b & MOUSE_MASK_BUTTONS as u_int == MOUSE_WHEEL_UP as u_int
        || (*m).b & MOUSE_MASK_BUTTONS as u_int == MOUSE_WHEEL_DOWN as u_int
    {
        if (*m).b & MOUSE_MASK_BUTTONS as u_int == MOUSE_WHEEL_UP as u_int {
            type_0 = KEYC_TYPE_WHEELUP;
        } else {
            type_0 = KEYC_TYPE_WHEELDOWN;
        }
        x = (*m).x;
        y = (*m).y;
        b = (*m).b;
        log_debug(
            b"wheel at %u,%u\0" as *const u8 as *const ::core::ffi::c_char,
            x,
            y,
        );
    } else if (*m).b & MOUSE_MASK_BUTTONS as u_int == 3 as u_int {
        type_0 = KEYC_TYPE_MOUSEUP;
        x = (*m).x;
        y = (*m).y;
        b = (*m).lb;
        if (*m).sgr_type == 'm' as i32 as u_int {
            b = (*m).sgr_b;
        }
        log_debug(
            b"up at %u,%u\0" as *const u8 as *const ::core::ffi::c_char,
            x,
            y,
        );
    } else {
        if (*c).flags & CLIENT_DOUBLECLICK as uint64_t != 0 {
            event_del(&raw mut (*c).click_timer);
            (*c).flags &= !CLIENT_DOUBLECLICK as uint64_t;
            type_0 = KEYC_TYPE_SECONDCLICK;
            x = (*m).x;
            y = (*m).y;
            b = (*m).b;
            log_debug(
                b"second-click at %u,%u\0" as *const u8 as *const ::core::ffi::c_char,
                x,
                y,
            );
            (*c).flags |= CLIENT_TRIPLECLICK as uint64_t;
            current_block = 16799951812150840583;
        } else if (*c).flags & CLIENT_TRIPLECLICK as uint64_t != 0 {
            event_del(&raw mut (*c).click_timer);
            (*c).flags &= !CLIENT_TRIPLECLICK as uint64_t;
            type_0 = KEYC_TYPE_TRIPLECLICK;
            x = (*m).x;
            y = (*m).y;
            b = (*m).b;
            log_debug(
                b"triple-click at %u,%u\0" as *const u8 as *const ::core::ffi::c_char,
                x,
                y,
            );
            current_block = 5614288427414743461;
        } else {
            current_block = 16799951812150840583;
        }
        match current_block {
            5614288427414743461 => {}
            _ => {
                if type_0 as ::core::ffi::c_uint
                    == KEYC_TYPE_NOTYPE as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    type_0 = KEYC_TYPE_MOUSEDOWN;
                    x = (*m).x;
                    y = (*m).y;
                    b = (*m).b;
                    log_debug(
                        b"down at %u,%u\0" as *const u8 as *const ::core::ffi::c_char,
                        x,
                        y,
                    );
                    (*c).flags |= CLIENT_DOUBLECLICK as uint64_t;
                }
            }
        }
    }
    if type_0 as ::core::ffi::c_uint
        == KEYC_TYPE_NOTYPE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code;
    }
    (*m).s = (*s).id as ::core::ffi::c_int;
    (*m).w = -(1 as ::core::ffi::c_int);
    (*m).wp = -(1 as ::core::ffi::c_int);
    (*m).ignore = ignore;
    (*m).statusat = status_at_line(c);
    (*m).statuslines = status_line_size(c);
    if (*m).statusat != -(1 as ::core::ffi::c_int)
        && y >= (*m).statusat as u_int
        && y < ((*m).statusat as u_int).wrapping_add((*m).statuslines)
    {
        sr = status_get_range(c, x, y.wrapping_sub((*m).statusat as u_int));
        if sr.is_null() {
            loc = KEYC_MOUSE_LOCATION_STATUS_DEFAULT;
        } else {
            match (*sr).type_0 as ::core::ffi::c_uint {
                0 => return KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code,
                1 => {
                    log_debug(b"mouse range: left\0" as *const u8 as *const ::core::ffi::c_char);
                    loc = KEYC_MOUSE_LOCATION_STATUS_LEFT;
                }
                2 => {
                    log_debug(b"mouse range: right\0" as *const u8 as *const ::core::ffi::c_char);
                    loc = KEYC_MOUSE_LOCATION_STATUS_RIGHT;
                }
                3 => {
                    fwp = window_pane_find_by_id((*sr).argument);
                    if fwp.is_null() {
                        return KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code;
                    }
                    (*m).wp = (*sr).argument as ::core::ffi::c_int;
                    log_debug(
                        b"mouse range: pane %%%u\0" as *const u8 as *const ::core::ffi::c_char,
                        (*m).wp,
                    );
                    loc = KEYC_MOUSE_LOCATION_STATUS;
                }
                4 => {
                    fwl = winlink_find_by_index(
                        &raw mut (*s).windows,
                        (*sr).argument as ::core::ffi::c_int,
                    );
                    if fwl.is_null() {
                        return KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code;
                    }
                    (*m).w = (*(*fwl).window).id as ::core::ffi::c_int;
                    log_debug(
                        b"mouse range: window @%u\0" as *const u8 as *const ::core::ffi::c_char,
                        (*m).w,
                    );
                    loc = KEYC_MOUSE_LOCATION_STATUS;
                }
                5 => {
                    fs = session_find_by_id((*sr).argument);
                    if fs.is_null() {
                        return KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code;
                    }
                    (*m).s = (*sr).argument as ::core::ffi::c_int;
                    log_debug(
                        b"mouse range: session $%u\0" as *const u8 as *const ::core::ffi::c_char,
                        (*m).s,
                    );
                    loc = KEYC_MOUSE_LOCATION_STATUS;
                }
                6 => {
                    log_debug(b"mouse range: user\0" as *const u8 as *const ::core::ffi::c_char);
                    loc = KEYC_MOUSE_LOCATION_STATUS;
                }
                7 => {
                    n = (*sr).argument;
                    log_debug(
                        b"mouse range: control %u\0" as *const u8 as *const ::core::ffi::c_char,
                        n,
                    );
                    loc = (KEYC_MOUSE_LOCATION_CONTROL0 as ::core::ffi::c_int as u_int)
                        .wrapping_add(n) as key_code_mouse_location;
                }
                _ => {}
            }
        }
    }
    if loc as ::core::ffi::c_uint
        == KEYC_MOUSE_LOCATION_NOWHERE as ::core::ffi::c_int as ::core::ffi::c_uint
        && (*c).tty.mouse_scrolling_flag != 0
    {
        if !lwp.is_null() {
            loc = KEYC_MOUSE_LOCATION_SCROLLBAR_SLIDER;
            (*m).wp = (*lwp).id as ::core::ffi::c_int;
            (*m).w = (*(*lwp).window).id as ::core::ffi::c_int;
        }
    } else if loc as ::core::ffi::c_uint
        == KEYC_MOUSE_LOCATION_NOWHERE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        px = x;
        if (*m).statusat == 0 as ::core::ffi::c_int && y >= (*m).statuslines {
            py = y.wrapping_sub((*m).statuslines);
        } else if (*m).statusat > 0 as ::core::ffi::c_int && y >= (*m).statusat as u_int {
            py = ((*m).statusat - 1 as ::core::ffi::c_int) as u_int;
        } else {
            py = y;
        }
        tty_window_offset(
            &raw mut (*c).tty,
            &raw mut (*m).ox,
            &raw mut (*m).oy,
            &raw mut sx,
            &raw mut sy,
        );
        log_debug(
            b"mouse window @%u at %u,%u (%ux%u)\0" as *const u8 as *const ::core::ffi::c_char,
            (*w).id,
            (*m).ox,
            (*m).oy,
            sx,
            sy,
        );
        if px > sx || py > sy {
            server_client_update_scrollbar_hover(
                c,
                type_0 as ::core::ffi::c_int,
                -(1 as ::core::ffi::c_int),
                -(1 as ::core::ffi::c_int),
            );
            return KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code;
        }
        px = px.wrapping_add((*m).ox);
        py = py.wrapping_add((*m).oy);
        if !(*w).modal.is_null() && window_pane_contains((*w).modal, px, py) == 0 {
            if lwp == (*w).modal
                && (*c).tty.mouse_drag_flag != 0 as ::core::ffi::c_int
                && (type_0 as ::core::ffi::c_uint
                    == KEYC_TYPE_MOUSEDRAG as ::core::ffi::c_int as ::core::ffi::c_uint
                    || type_0 as ::core::ffi::c_uint
                        == KEYC_TYPE_MOUSEUP as ::core::ffi::c_int as ::core::ffi::c_uint)
            {
                modal_drag = 1 as ::core::ffi::c_int;
                wp = lwp;
                loc = KEYC_MOUSE_LOCATION_PANE;
                (*m).wp = (*wp).id as ::core::ffi::c_int;
                (*m).w = (*(*wp).window).id as ::core::ffi::c_int;
            } else {
                server_client_update_scrollbar_hover(
                    c,
                    type_0 as ::core::ffi::c_int,
                    -(1 as ::core::ffi::c_int),
                    -(1 as ::core::ffi::c_int),
                );
                (*c).tty.mouse_drag_update = None;
                (*c).tty.mouse_drag_release = None;
                (*c).tty.mouse_drag_flag = 0 as ::core::ffi::c_int;
                (*c).tty.mouse_scrolling_flag = 0 as ::core::ffi::c_int;
                (*c).tty.mouse_slider_mpos = -(1 as ::core::ffi::c_int);
                (*c).tty.mouse_last_pane = -(1 as ::core::ffi::c_int);
                if (*(*w).modal).flags & PANE_CLOSEONCLICK != 0
                    && (type_0 as ::core::ffi::c_uint
                        == KEYC_TYPE_MOUSEDOWN as ::core::ffi::c_int as ::core::ffi::c_uint
                        || type_0 as ::core::ffi::c_uint
                            == KEYC_TYPE_SECONDCLICK as ::core::ffi::c_int as ::core::ffi::c_uint
                        || type_0 as ::core::ffi::c_uint
                            == KEYC_TYPE_TRIPLECLICK as ::core::ffi::c_int as ::core::ffi::c_uint)
                {
                    server_kill_pane((*w).modal);
                }
                return KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code;
            }
        }
        server_client_update_scrollbar_hover(
            c,
            type_0 as ::core::ffi::c_int,
            px as ::core::ffi::c_int,
            py as ::core::ffi::c_int,
        );
        if !(modal_drag != 0) {
            if type_0 as ::core::ffi::c_uint
                == KEYC_TYPE_MOUSEDRAG as ::core::ffi::c_int as ::core::ffi::c_uint
                && !lwp.is_null()
            {
                wp = lwp;
            } else {
                wp = window_get_active_at(w, px, py);
            }
        }
        if wp.is_null() {
            loc = KEYC_MOUSE_LOCATION_EMPTY;
            (*m).w = (*w).id as ::core::ffi::c_int;
            log_debug(
                b"mouse %u,%u on empty area\0" as *const u8 as *const ::core::ffi::c_char,
                x,
                y,
            );
        } else {
            if modal_drag == 0 {
                loc = server_client_check_mouse_in_pane(
                    wp,
                    px as ::core::ffi::c_int,
                    py as ::core::ffi::c_int,
                    &raw mut sl_mpos,
                );
            }
            if loc as ::core::ffi::c_uint
                == KEYC_MOUSE_LOCATION_PANE as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                log_debug(
                    b"mouse %u,%u on pane %%%u\0" as *const u8 as *const ::core::ffi::c_char,
                    x,
                    y,
                    (*wp).id,
                );
            } else if loc as ::core::ffi::c_uint
                == KEYC_MOUSE_LOCATION_BORDER as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                sr = window_pane_status_get_range(wp, px, py);
                if !sr.is_null() {
                    n = (*sr).argument;
                    loc = (KEYC_MOUSE_LOCATION_CONTROL0 as ::core::ffi::c_int as u_int)
                        .wrapping_add(n) as key_code_mouse_location;
                }
                log_debug(
                    b"mouse on pane %%%u border\0" as *const u8 as *const ::core::ffi::c_char,
                    (*wp).id,
                );
            } else if loc as ::core::ffi::c_uint
                == KEYC_MOUSE_LOCATION_SCROLLBAR_UP as ::core::ffi::c_int as ::core::ffi::c_uint
                || loc as ::core::ffi::c_uint
                    == KEYC_MOUSE_LOCATION_SCROLLBAR_SLIDER as ::core::ffi::c_int
                        as ::core::ffi::c_uint
                || loc as ::core::ffi::c_uint
                    == KEYC_MOUSE_LOCATION_SCROLLBAR_DOWN as ::core::ffi::c_int
                        as ::core::ffi::c_uint
            {
                log_debug(
                    b"mouse on pane %%%u scrollbar\0" as *const u8 as *const ::core::ffi::c_char,
                    (*wp).id,
                );
            }
            (*m).wp = (*wp).id as ::core::ffi::c_int;
            (*m).w = (*(*wp).window).id as ::core::ffi::c_int;
        }
    } else {
        server_client_update_scrollbar_hover(
            c,
            type_0 as ::core::ffi::c_int,
            -(1 as ::core::ffi::c_int),
            -(1 as ::core::ffi::c_int),
        );
    }
    if type_0 as ::core::ffi::c_uint
        == KEYC_TYPE_MOUSEDOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        || type_0 as ::core::ffi::c_uint
            == KEYC_TYPE_SECONDCLICK as ::core::ffi::c_int as ::core::ffi::c_uint
        || type_0 as ::core::ffi::c_uint
            == KEYC_TYPE_TRIPLECLICK as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if type_0 as ::core::ffi::c_uint
            != KEYC_TYPE_MOUSEDOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            && ((*m).b != (*c).click_button
                || loc as ::core::ffi::c_uint
                    != (*c).click_loc as key_code_mouse_location as ::core::ffi::c_uint
                || (*m).wp != (*c).click_wp)
        {
            type_0 = KEYC_TYPE_MOUSEDOWN;
            log_debug(
                b"click sequence reset at %u,%u\0" as *const u8 as *const ::core::ffi::c_char,
                x,
                y,
            );
            (*c).flags &= !CLIENT_TRIPLECLICK as uint64_t;
            (*c).flags |= CLIENT_DOUBLECLICK as uint64_t;
        }
        if type_0 as ::core::ffi::c_uint
            != KEYC_TYPE_TRIPLECLICK as ::core::ffi::c_int as ::core::ffi::c_uint
            && KEYC_CLICK_TIMEOUT != 0 as ::core::ffi::c_int
        {
            memcpy(
                &raw mut (*c).click_event as *mut ::core::ffi::c_void,
                m as *const ::core::ffi::c_void,
                ::core::mem::size_of::<mouse_event>() as size_t,
            );
            (*c).click_button = (*m).b;
            (*c).click_loc = loc as ::core::ffi::c_int;
            (*c).click_wp = (*m).wp;
            log_debug(b"click timer started\0" as *const u8 as *const ::core::ffi::c_char);
            tv.tv_sec = (KEYC_CLICK_TIMEOUT / 1000 as ::core::ffi::c_int) as __time_t;
            tv.tv_usec = ((KEYC_CLICK_TIMEOUT % 1000 as ::core::ffi::c_int) as ::core::ffi::c_long
                * 1000 as ::core::ffi::c_long) as __suseconds_t;
            event_del(&raw mut (*c).click_timer);
            event_add(&raw mut (*c).click_timer, &raw mut tv);
        }
    }
    key = KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code;
    if type_0 as ::core::ffi::c_uint
        != KEYC_TYPE_MOUSEDRAG as ::core::ffi::c_int as ::core::ffi::c_uint
        && type_0 as ::core::ffi::c_uint
            != KEYC_TYPE_WHEELUP as ::core::ffi::c_int as ::core::ffi::c_uint
        && type_0 as ::core::ffi::c_uint
            != KEYC_TYPE_WHEELDOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        && type_0 as ::core::ffi::c_uint
            != KEYC_TYPE_DOUBLECLICK as ::core::ffi::c_int as ::core::ffi::c_uint
        && type_0 as ::core::ffi::c_uint
            != KEYC_TYPE_TRIPLECLICK as ::core::ffi::c_int as ::core::ffi::c_uint
        && (*c).tty.mouse_drag_flag != 0 as ::core::ffi::c_int
    {
        if (*c).tty.mouse_drag_release.is_some() {
            (*c).tty
                .mouse_drag_release
                .expect("non-null function pointer")(c, m);
        }
        (*c).tty.mouse_drag_update = None;
        (*c).tty.mouse_drag_release = None;
        (*c).tty.mouse_scrolling_flag = 0 as ::core::ffi::c_int;
        type_0 = KEYC_TYPE_MOUSEDRAGEND;
        (*c).tty.mouse_drag_flag = 0 as ::core::ffi::c_int;
        (*c).tty.mouse_slider_mpos = -(1 as ::core::ffi::c_int);
        (*c).tty.mouse_last_pane = -(1 as ::core::ffi::c_int);
    }
    if type_0 as ::core::ffi::c_uint
        == KEYC_TYPE_MOUSEMOVE as ::core::ffi::c_int as ::core::ffi::c_uint
        && loc as ::core::ffi::c_uint
            == KEYC_MOUSE_LOCATION_PANE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        key = KEYC_MOUSEMOVE_PANE as ::core::ffi::c_ulong as key_code;
        if !wp.is_null()
            && wp != (*w).active
            && options_get_number(
                (*s).options,
                b"focus-follows-mouse\0" as *const u8 as *const ::core::ffi::c_char,
            ) != 0
        {
            window_redraw_active_switch(w, wp);
            window_set_active_pane(w, wp, 1 as ::core::ffi::c_int);
            server_redraw_window_borders(w);
            server_status_window(w);
        }
    }
    if type_0 as ::core::ffi::c_uint
        == KEYC_TYPE_MOUSEDRAG as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if (*c).tty.mouse_drag_update.is_some() {
            key = KEYC_DRAGGING as ::core::ffi::c_ulong as key_code;
        }
        if (*c).tty.mouse_drag_flag == 0 as ::core::ffi::c_int {
            (*c).tty.mouse_drag_x = px;
            (*c).tty.mouse_drag_y = py;
        }
        (*c).tty.mouse_drag_flag =
            (b & MOUSE_MASK_BUTTONS as u_int).wrapping_add(1 as u_int) as ::core::ffi::c_int;
        if lwp.is_null() {
            wp = window_get_active_at(w, px, py);
            lwp = wp;
            if !wp.is_null() {
                (*c).tty.mouse_last_pane = (*wp).id as ::core::ffi::c_int;
            }
        }
        if (*c).tty.mouse_scrolling_flag == 0 as ::core::ffi::c_int
            && loc as ::core::ffi::c_uint
                == KEYC_MOUSE_LOCATION_SCROLLBAR_SLIDER as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            (*c).tty.mouse_scrolling_flag = 1 as ::core::ffi::c_int;
            if (*m).statusat == 0 as ::core::ffi::c_int {
                (*c).tty.mouse_slider_mpos =
                    sl_mpos.wrapping_add((*m).statuslines) as ::core::ffi::c_int;
            } else {
                (*c).tty.mouse_slider_mpos = sl_mpos as ::core::ffi::c_int;
            }
        }
    }
    if key == KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code {
        if b & MOUSE_MASK_BUTTONS as u_int == MOUSE_BUTTON_1 as u_int {
            bn = 1 as u_int;
        } else if b & MOUSE_MASK_BUTTONS as u_int == MOUSE_BUTTON_2 as u_int {
            bn = 2 as u_int;
        } else if b & MOUSE_MASK_BUTTONS as u_int == MOUSE_BUTTON_3 as u_int {
            bn = 3 as u_int;
        } else if b & MOUSE_MASK_BUTTONS as u_int == MOUSE_BUTTON_6 as u_int {
            bn = 6 as u_int;
        } else if b & MOUSE_MASK_BUTTONS as u_int == MOUSE_BUTTON_7 as u_int {
            bn = 7 as u_int;
        } else if b & MOUSE_MASK_BUTTONS as u_int == MOUSE_BUTTON_8 as u_int {
            bn = 8 as u_int;
        } else if b & MOUSE_MASK_BUTTONS as u_int == MOUSE_BUTTON_9 as u_int {
            bn = 9 as u_int;
        } else if b & MOUSE_MASK_BUTTONS as u_int == MOUSE_BUTTON_10 as u_int {
            bn = 10 as u_int;
        } else if b & MOUSE_MASK_BUTTONS as u_int == MOUSE_BUTTON_11 as u_int {
            bn = 11 as u_int;
        } else {
            bn = 0 as u_int;
        }
        key = ((type_0 as ::core::ffi::c_ulonglong) << 32 as ::core::ffi::c_int
            | (bn as ::core::ffi::c_ulonglong) << KEYC_MOUSE_BUTTON_SHIFT
            | (loc as ::core::ffi::c_ulonglong) << KEYC_MOUSE_LOCATION_SHIFT)
            as key_code;
    }
    if b & MOUSE_MASK_META as u_int != 0 {
        key |= KEYC_META;
    }
    if b & MOUSE_MASK_CTRL as u_int != 0 {
        key |= KEYC_CTRL;
    }
    if b & MOUSE_MASK_SHIFT as u_int != 0 {
        key |= KEYC_SHIFT;
    }
    if log_get_level() != 0 as ::core::ffi::c_int {
        let key_string = key_string_format(key, true);
        log_debug(
            b"mouse key is %s\0" as *const u8 as *const ::core::ffi::c_char,
            key_string.as_ptr(),
        );
    }
    return key;
}
#[no_mangle]
pub unsafe extern "C" fn server_client_update_theme_colours(mut c: *mut client) {
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut name: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut value: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut theme: client_theme = THEME_UNKNOWN;
    let mut expanded: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut i: u_int = 0;
    let mut colour: ::core::ffi::c_int = 0;
    let mut option: ::core::ffi::c_int = 0;
    if c.is_null() {
        return;
    }
    option = options_get_number(
        global_options,
        b"theme\0" as *const u8 as *const ::core::ffi::c_char,
    ) as ::core::ffi::c_int;
    if option == 1 as ::core::ffi::c_int {
        i = 0 as u_int;
        while i < COLOUR_THEME_COUNT as u_int {
            (*c).theme_colours[i as usize] = colour_theme_terminal_colour(i);
            i = i.wrapping_add(1);
        }
        return;
    }
    ft = format_create(
        c,
        ::core::ptr::null_mut::<cmdq_item>(),
        FORMAT_NONE,
        FORMAT_NOJOBS,
    );
    format_defaults(
        ft,
        c,
        ::core::ptr::null_mut::<session>(),
        ::core::ptr::null_mut::<winlink>(),
        ::core::ptr::null_mut::<window_pane>(),
    );
    theme = (*c).theme;
    if theme as ::core::ffi::c_uint == THEME_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint {
        theme = colour_totheme((*c).tty.bg);
    }
    if option == 2 as ::core::ffi::c_int {
        theme = THEME_LIGHT;
    } else if option == 3 as ::core::ffi::c_int {
        theme = THEME_DARK;
    }
    i = 0 as u_int;
    while i < COLOUR_THEME_COUNT as u_int {
        (*c).theme_colours[i as usize] = 8 as ::core::ffi::c_int;
        name = colour_theme_option(i, theme);
        if !name.is_null() {
            value = options_get_string(global_options, name);
            expanded = format_expand(ft, value);
            colour = colour_parse_cstr(std::ffi::CStr::from_ptr(expanded)).unwrap_or(-1);
            free(expanded as *mut ::core::ffi::c_void);
            if !(colour == -(1 as ::core::ffi::c_int) || colour & COLOUR_FLAG_THEME != 0) {
                (*c).theme_colours[i as usize] = colour;
            }
        }
        i = i.wrapping_add(1);
    }
    format_free(ft);
}
unsafe extern "C" fn server_client_is_bracket_paste(
    mut c: *mut client,
    mut key: key_code,
) -> ::core::ffi::c_int {
    if key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
        == KEYC_PASTE_START as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
    {
        (*c).flags = ((*c).flags as ::core::ffi::c_ulonglong | CLIENT_BRACKETPASTING) as uint64_t;
        (*c).paste_time = current_time;
        log_debug(
            b"%s: bracket paste on\0" as *const u8 as *const ::core::ffi::c_char,
            (*c).name,
        );
        return 0 as ::core::ffi::c_int;
    }
    if key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
        == KEYC_PASTE_END as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
    {
        (*c).flags = ((*c).flags as ::core::ffi::c_ulonglong & !CLIENT_BRACKETPASTING) as uint64_t;
        log_debug(
            b"%s: bracket paste off\0" as *const u8 as *const ::core::ffi::c_char,
            (*c).name,
        );
        return 0 as ::core::ffi::c_int;
    }
    return ((*c).flags as ::core::ffi::c_ulonglong & CLIENT_BRACKETPASTING != 0)
        as ::core::ffi::c_int;
}
unsafe extern "C" fn server_client_is_assume_paste(mut c: *mut client) -> ::core::ffi::c_int {
    let mut s: *mut session = (*c).session;
    let mut tv: timeval = timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    let mut t: ::core::ffi::c_int = 0;
    if (*c).flags as ::core::ffi::c_ulonglong & CLIENT_BRACKETPASTING != 0 {
        return 0 as ::core::ffi::c_int;
    }
    t = options_get_number(
        (*s).options,
        b"assume-paste-time\0" as *const u8 as *const ::core::ffi::c_char,
    ) as ::core::ffi::c_int;
    if t == 0 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    if tty_term_has((*c).tty.term, TTYC_ENBP) != 0 {
        return 0 as ::core::ffi::c_int;
    }
    tv.tv_sec = (*c).activity_time.tv_sec - (*c).last_activity_time.tv_sec;
    tv.tv_usec = (*c).activity_time.tv_usec - (*c).last_activity_time.tv_usec;
    if tv.tv_usec < 0 as __suseconds_t {
        tv.tv_sec -= 1;
        tv.tv_usec += 1000000 as __suseconds_t;
    }
    if tv.tv_sec == 0 as __time_t && tv.tv_usec < (t * 1000 as ::core::ffi::c_int) as __suseconds_t
    {
        if (*c).flags as ::core::ffi::c_ulonglong & CLIENT_ASSUMEPASTING != 0 {
            return 1 as ::core::ffi::c_int;
        }
        (*c).flags = ((*c).flags as ::core::ffi::c_ulonglong | CLIENT_ASSUMEPASTING) as uint64_t;
        (*c).paste_time = current_time;
        log_debug(
            b"%s: assume paste on\0" as *const u8 as *const ::core::ffi::c_char,
            (*c).name,
        );
        return 0 as ::core::ffi::c_int;
    }
    if (*c).flags as ::core::ffi::c_ulonglong & CLIENT_ASSUMEPASTING != 0 {
        (*c).flags = ((*c).flags as ::core::ffi::c_ulonglong & !CLIENT_ASSUMEPASTING) as uint64_t;
        log_debug(
            b"%s: assume paste off\0" as *const u8 as *const ::core::ffi::c_char,
            (*c).name,
        );
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn server_client_update_latest(mut c: *mut client) {
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    if (*c).session.is_null() {
        return;
    }
    w = (*(*(*c).session).curw).window;
    if (*w).latest == c as *mut ::core::ffi::c_void {
        return;
    }
    (*w).latest = c as *mut ::core::ffi::c_void;
    if options_get_number(
        (*w).options,
        b"window-size\0" as *const u8 as *const ::core::ffi::c_char,
    ) == WINDOW_SIZE_LATEST as ::core::ffi::c_longlong
    {
        recalculate_size(w, 0 as ::core::ffi::c_int);
    }
    events_fire_client(
        b"client-active\0" as *const u8 as *const ::core::ffi::c_char,
        c,
    );
}
unsafe extern "C" fn server_client_repeat_time(
    mut c: *mut client,
    mut bd: *mut key_binding,
) -> u_int {
    let mut s: *mut session = (*c).session;
    let mut repeat: u_int = 0;
    let mut initial: u_int = 0;
    if !(*bd).flags & KEY_BINDING_REPEAT != 0 {
        return 0 as u_int;
    }
    repeat = options_get_number(
        (*s).options,
        b"repeat-time\0" as *const u8 as *const ::core::ffi::c_char,
    ) as u_int;
    if repeat == 0 as u_int {
        return 0 as u_int;
    }
    if !(*c).flags & CLIENT_REPEAT as uint64_t != 0 || (*bd).key != (*c).last_key {
        initial = options_get_number(
            (*s).options,
            b"initial-repeat-time\0" as *const u8 as *const ::core::ffi::c_char,
        ) as u_int;
        if initial != 0 as u_int {
            repeat = initial;
        }
    }
    return repeat;
}
unsafe extern "C" fn server_client_handle_dead_key(
    mut wp: *mut window_pane,
    mut key: key_code,
) -> ::core::ffi::c_int {
    let mut remain_on_exit: ::core::ffi::c_int = 0;
    if wp.is_null()
        || !(*wp).flags & PANE_EXITED != 0
        || (key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
            == KEYC_MOUSE as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
            || key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                >= (KEYC_TYPE_MOUSEMOVE as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                    << 32 as ::core::ffi::c_int
                && key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                    <= (KEYC_TYPE_TRIPLECLICK as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                        << 32 as ::core::ffi::c_int)
        || key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
            == (KEYC_TYPE_FUNCTION as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                << 32 as ::core::ffi::c_int
            && (key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
                == KEYC_PASTE_START as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
                || key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
                    == KEYC_PASTE_END as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong)
    {
        return 0 as ::core::ffi::c_int;
    }
    remain_on_exit = options_get_number(
        (*wp).options,
        b"remain-on-exit\0" as *const u8 as *const ::core::ffi::c_char,
    ) as ::core::ffi::c_int;
    if remain_on_exit != 3 as ::core::ffi::c_int && remain_on_exit != 4 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    options_set_number(
        (*wp).options,
        b"remain-on-exit\0" as *const u8 as *const ::core::ffi::c_char,
        0 as ::core::ffi::c_longlong,
    );
    server_destroy_pane(wp, 0 as ::core::ffi::c_int);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn server_client_key_callback(
    mut item: *mut cmdq_item,
    mut data: *mut ::core::ffi::c_void,
) -> cmd_retval {
    let mut current_block: u64;
    // The queued callback owns the event and its bytes until this call returns.
    let mut owned = Box::from_raw(data as *mut OwnedKeyEvent);
    let mut event: *mut key_event = &raw mut owned.event;
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut ec: *mut client = (*event).client;
    let mut key: key_code = (*event).key;
    let mut m: *mut mouse_event = &raw mut (*event).m;
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
    let mut tv: timeval = timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    let mut table: *mut key_table = ::core::ptr::null_mut::<key_table>();
    let mut first: *mut key_table = ::core::ptr::null_mut::<key_table>();
    let mut bd: *mut key_binding = ::core::ptr::null_mut::<key_binding>();
    let mut repeat: u_int = 0;
    let mut flags: uint64_t = 0;
    let mut prefix_delay: uint64_t = 0;
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null_mut::<cmd_find_state>(),
        s: ::core::ptr::null_mut::<session>(),
        wl: ::core::ptr::null_mut::<winlink>(),
        w: ::core::ptr::null_mut::<window>(),
        wp: ::core::ptr::null_mut::<window_pane>(),
        idx: 0,
    };
    let mut key0: key_code = 0;
    let mut prefix: key_code = 0;
    let mut prefix2: key_code = 0;
    if !ec.is_null() {
        c = ec;
    } else {
        c = cmdq_get_client(item);
    }
    s = (*c).session;
    if !(s.is_null() || (*c).flags & CLIENT_UNATTACHEDFLAGS as uint64_t != 0) {
        wl = (*s).curw;
        memcpy(
            &raw mut (*c).last_activity_time as *mut ::core::ffi::c_void,
            &raw mut (*c).activity_time as *const ::core::ffi::c_void,
            ::core::mem::size_of::<timeval>() as size_t,
        );
        if gettimeofday(&raw mut (*c).activity_time, NULL) != 0 as ::core::ffi::c_int {
            fatal(b"gettimeofday failed\0" as *const u8 as *const ::core::ffi::c_char);
        }
        session_update_activity(s, &raw mut (*c).activity_time);
        (*m).valid = 0 as ::core::ffi::c_int;
        if key == KEYC_MOUSE as ::core::ffi::c_ulong as key_code
            || key == KEYC_DOUBLECLICK as ::core::ffi::c_ulong as key_code
        {
            if (*c).flags & CLIENT_READONLY as uint64_t != 0 {
                current_block = 1578459965781631232;
            } else {
                key = server_client_check_mouse(c, event);
                if key == KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code {
                    current_block = 1578459965781631232;
                } else {
                    (*m).valid = 1 as ::core::ffi::c_int;
                    (*m).key = key;
                    if key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
                        == KEYC_DRAGGING as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
                    {
                        (*c).tty
                            .mouse_drag_update
                            .expect("non-null function pointer")(c, m);
                        current_block = 1578459965781631232;
                    } else {
                        (*event).key = key;
                        current_block = 4495394744059808450;
                    }
                }
            }
        } else {
            current_block = 4495394744059808450;
        }
        match current_block {
            1578459965781631232 => {}
            _ => {
                if !(key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
                    == KEYC_MOUSE as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
                    || key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                        >= (KEYC_TYPE_MOUSEMOVE as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                            << 32 as ::core::ffi::c_int
                        && key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                            <= (KEYC_TYPE_TRIPLECLICK as ::core::ffi::c_int
                                as ::core::ffi::c_ulonglong)
                                << 32 as ::core::ffi::c_int)
                    || cmd_find_from_mouse(&raw mut fs, m, 0 as ::core::ffi::c_int)
                        != 0 as ::core::ffi::c_int
                {
                    cmd_find_from_client(&raw mut fs, c, 0 as ::core::ffi::c_int);
                }
                wp = fs.wp;
                if (key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
                    == KEYC_MOUSE as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
                    || key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                        >= (KEYC_TYPE_MOUSEMOVE as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                            << 32 as ::core::ffi::c_int
                        && key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                            <= (KEYC_TYPE_TRIPLECLICK as ::core::ffi::c_int
                                as ::core::ffi::c_ulonglong)
                                << 32 as ::core::ffi::c_int)
                    && options_get_number(
                        (*s).options,
                        b"mouse\0" as *const u8 as *const ::core::ffi::c_char,
                    ) == 0
                {
                    current_block = 15469183920764600035;
                } else {
                    if server_client_is_bracket_paste(c, key) != 0 {
                        current_block = 3436715649514806935;
                    } else if !(key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
                        == KEYC_MOUSE as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
                        || key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                            >= (KEYC_TYPE_MOUSEMOVE as ::core::ffi::c_int
                                as ::core::ffi::c_ulonglong)
                                << 32 as ::core::ffi::c_int
                            && key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                                <= (KEYC_TYPE_TRIPLECLICK as ::core::ffi::c_int
                                    as ::core::ffi::c_ulonglong)
                                    << 32 as ::core::ffi::c_int)
                        && key != KEYC_FOCUS_IN as ::core::ffi::c_ulong as key_code
                        && key != KEYC_FOCUS_OUT as ::core::ffi::c_ulong as key_code
                        && !(key as ::core::ffi::c_ulonglong) & KEYC_SENT != 0
                        && server_client_is_assume_paste(c) != 0
                    {
                        current_block = 3436715649514806935;
                    } else if !wp.is_null()
                        && (*wp).flags & PANE_CAPTUREALLKEYS != 0
                        && !(*wp).flags & PANE_EXITED != 0
                        && !(key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
                            == KEYC_MOUSE as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
                            || key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                                >= (KEYC_TYPE_MOUSEMOVE as ::core::ffi::c_int
                                    as ::core::ffi::c_ulonglong)
                                    << 32 as ::core::ffi::c_int
                                && key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                                    <= (KEYC_TYPE_TRIPLECLICK as ::core::ffi::c_int
                                        as ::core::ffi::c_ulonglong)
                                        << 32 as ::core::ffi::c_int)
                        && (*wp).modes.tqh_first.is_null()
                    {
                        current_block = 15469183920764600035;
                    } else if key == KEYC_FOCUS_IN as ::core::ffi::c_ulong as key_code
                        || key == KEYC_FOCUS_OUT as ::core::ffi::c_ulong as key_code
                    {
                        current_block = 15469183920764600035;
                    } else {
                        if server_client_is_default_key_table(c, (*c).keytable as *mut key_table)
                            != 0
                            && !wp.is_null()
                            && {
                                wme = (*wp).modes.tqh_first;
                                !wme.is_null()
                            }
                            && (*(*wme).mode).key_table.is_some()
                        {
                            table = key_bindings_get_table(
                                (*(*wme).mode).key_table.expect("non-null function pointer")(wme),
                                1 as ::core::ffi::c_int,
                            );
                        } else {
                            table = (*c).keytable as *mut key_table;
                        }
                        first = table;
                        '_table_changed: loop {
                            prefix = options_get_number(
                                (*s).options,
                                b"prefix\0" as *const u8 as *const ::core::ffi::c_char,
                            ) as key_code;
                            prefix2 = options_get_number(
                                (*s).options,
                                b"prefix2\0" as *const u8 as *const ::core::ffi::c_char,
                            ) as key_code;
                            key0 = (key as ::core::ffi::c_ulonglong
                                & (KEYC_MASK_KEY | KEYC_MASK_MODIFIERS))
                                as key_code;
                            if (key0
                                == prefix as ::core::ffi::c_ulonglong
                                    & (KEYC_MASK_KEY | KEYC_MASK_MODIFIERS)
                                || key0
                                    == prefix2 as ::core::ffi::c_ulonglong
                                        & (KEYC_MASK_KEY | KEYC_MASK_MODIFIERS))
                                && strcmp(
                                    (*table).name,
                                    b"prefix\0" as *const u8 as *const ::core::ffi::c_char,
                                ) != 0 as ::core::ffi::c_int
                            {
                                server_client_set_key_table(
                                    c,
                                    b"prefix\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                server_status_client(c);
                                current_block = 1578459965781631232;
                                break;
                            } else {
                                flags = (*c).flags;
                                loop {
                                    if wp.is_null() {
                                        log_debug(
                                            b"key table %s (no pane)\0" as *const u8
                                                as *const ::core::ffi::c_char,
                                            (*table).name,
                                        );
                                    } else {
                                        log_debug(
                                            b"key table %s (pane %%%u)\0" as *const u8
                                                as *const ::core::ffi::c_char,
                                            (*table).name,
                                            (*wp).id,
                                        );
                                    }
                                    if (*c).flags & CLIENT_REPEAT as uint64_t != 0 {
                                        log_debug(
                                            b"currently repeating\0" as *const u8
                                                as *const ::core::ffi::c_char,
                                        );
                                    }
                                    bd = key_bindings_get(table, key0);
                                    prefix_delay = options_get_number(
                                        global_options,
                                        b"prefix-timeout\0" as *const u8
                                            as *const ::core::ffi::c_char,
                                    )
                                        as uint64_t;
                                    if prefix_delay > 0 as uint64_t
                                        && strcmp(
                                            (*table).name,
                                            b"prefix\0" as *const u8 as *const ::core::ffi::c_char,
                                        ) == 0 as ::core::ffi::c_int
                                        && server_client_key_table_activity_diff(c) > prefix_delay
                                    {
                                        if !bd.is_null()
                                            && (*c).flags & CLIENT_REPEAT as uint64_t != 0
                                            && (*bd).flags & KEY_BINDING_REPEAT != 0
                                        {
                                            log_debug(
                                                b"prefix timeout ignored, repeat is active\0"
                                                    as *const u8
                                                    as *const ::core::ffi::c_char,
                                            );
                                        } else {
                                            log_debug(
                                                b"prefix timeout exceeded\0" as *const u8
                                                    as *const ::core::ffi::c_char,
                                            );
                                            server_client_set_key_table(
                                                c,
                                                ::core::ptr::null::<::core::ffi::c_char>(),
                                            );
                                            table = (*c).keytable as *mut key_table;
                                            first = table;
                                            server_status_client(c);
                                            continue '_table_changed;
                                        }
                                    }
                                    if !bd.is_null() {
                                        if (*c).flags & CLIENT_REPEAT as uint64_t != 0
                                            && !(*bd).flags & KEY_BINDING_REPEAT != 0
                                        {
                                            current_block = 5891011138178424807;
                                            break;
                                        } else {
                                            current_block = 13763002826403452995;
                                            break;
                                        }
                                    } else if key0 != KEYC_ANY as ::core::ffi::c_ulong as key_code {
                                        key0 = KEYC_ANY as ::core::ffi::c_ulong as key_code;
                                    } else {
                                        if key
                                            == KEYC_MOUSEMOVE_PANE as ::core::ffi::c_ulong
                                                as key_code
                                            || key
                                                == KEYC_MOUSEMOVE_STATUS as ::core::ffi::c_ulong
                                                    as key_code
                                            || key
                                                == KEYC_MOUSEMOVE_STATUS_LEFT
                                                    as ::core::ffi::c_ulong
                                                    as key_code
                                            || key
                                                == KEYC_MOUSEMOVE_STATUS_RIGHT
                                                    as ::core::ffi::c_ulong
                                                    as key_code
                                            || key
                                                == KEYC_MOUSEMOVE_STATUS_DEFAULT
                                                    as ::core::ffi::c_ulong
                                                    as key_code
                                            || key
                                                == KEYC_MOUSEMOVE_BORDER as ::core::ffi::c_ulong
                                                    as key_code
                                        {
                                            current_block = 15469183920764600035;
                                            break '_table_changed;
                                        }
                                        log_debug(
                                            b"not found in key table %s\0" as *const u8
                                                as *const ::core::ffi::c_char,
                                            (*table).name,
                                        );
                                        if server_client_is_default_key_table(c, table) == 0
                                            || (*c).flags & CLIENT_REPEAT as uint64_t != 0
                                        {
                                            current_block = 13484060386966298149;
                                            break;
                                        } else {
                                            current_block = 11796148217846552555;
                                            break;
                                        }
                                    }
                                }
                                match current_block {
                                    11796148217846552555 => {
                                        if first != table && !flags & CLIENT_REPEAT as uint64_t != 0
                                        {
                                            current_block = 10435735846551762309;
                                            break;
                                        } else {
                                            current_block = 15469183920764600035;
                                            break;
                                        }
                                    }
                                    13763002826403452995 => {
                                        log_debug(
                                            b"found in key table %s\0" as *const u8
                                                as *const ::core::ffi::c_char,
                                            (*table).name,
                                        );
                                        (*table).references = (*table).references.wrapping_add(1);
                                        repeat = server_client_repeat_time(c, bd);
                                        if repeat != 0 as u_int {
                                            (*c).flags |= CLIENT_REPEAT as uint64_t;
                                            (*c).last_key = (*bd).key;
                                            tv.tv_sec =
                                                repeat.wrapping_div(1000 as u_int) as __time_t;
                                            tv.tv_usec = (repeat.wrapping_rem(1000 as u_int)
                                                as ::core::ffi::c_long
                                                * 1000 as ::core::ffi::c_long)
                                                as __suseconds_t;
                                            event_del(&raw mut (*c).repeat_timer);
                                            event_add(&raw mut (*c).repeat_timer, &raw mut tv);
                                        } else {
                                            (*c).flags &= !CLIENT_REPEAT as uint64_t;
                                            server_client_set_key_table(
                                                c,
                                                ::core::ptr::null::<::core::ffi::c_char>(),
                                            );
                                        }
                                        server_status_client(c);
                                        key_bindings_dispatch(bd, item, c, event, &raw mut fs);
                                        key_bindings_unref_table(table);
                                        current_block = 1578459965781631232;
                                        break;
                                    }
                                    13484060386966298149 => {
                                        log_debug(
                                            b"trying in root table\0" as *const u8
                                                as *const ::core::ffi::c_char,
                                        );
                                        server_client_set_key_table(
                                            c,
                                            ::core::ptr::null::<::core::ffi::c_char>(),
                                        );
                                        table = (*c).keytable as *mut key_table;
                                        if (*c).flags & CLIENT_REPEAT as uint64_t != 0 {
                                            first = table;
                                        }
                                        (*c).flags &= !CLIENT_REPEAT as uint64_t;
                                        server_status_client(c);
                                    }
                                    _ => {
                                        log_debug(
                                            b"found in key table %s (not repeating)\0" as *const u8
                                                as *const ::core::ffi::c_char,
                                            (*table).name,
                                        );
                                        server_client_set_key_table(
                                            c,
                                            ::core::ptr::null::<::core::ffi::c_char>(),
                                        );
                                        table = (*c).keytable as *mut key_table;
                                        first = table;
                                        (*c).flags &= !CLIENT_REPEAT as uint64_t;
                                        server_status_client(c);
                                    }
                                }
                            }
                        }
                        match current_block {
                            1578459965781631232 => {}
                            15469183920764600035 => {}
                            _ => {
                                server_client_set_key_table(
                                    c,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                );
                                server_status_client(c);
                                current_block = 1578459965781631232;
                            }
                        }
                    }
                    match current_block {
                        1578459965781631232 => {}
                        15469183920764600035 => {}
                        _ => {
                            if (*c).flags & CLIENT_READONLY as uint64_t != 0 {
                                current_block = 1578459965781631232;
                            } else {
                                if !(*event).buf.is_null() {
                                    window_pane_paste(wp, key, (*event).buf, (*event).len);
                                }
                                key = KEYC_NONE as ::core::ffi::c_ulong as key_code;
                                current_block = 1578459965781631232;
                            }
                        }
                    }
                }
                match current_block {
                    1578459965781631232 => {}
                    _ => {
                        if !(server_client_handle_dead_key(wp, key) != 0) {
                            if !((*c).flags & CLIENT_READONLY as uint64_t != 0) {
                                if !wp.is_null() {
                                    window_pane_key(wp, c, s, wl, key, m);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    if !s.is_null() && key != KEYC_FOCUS_OUT as ::core::ffi::c_ulong as key_code {
        server_client_update_latest(c);
    }
    if !ec.is_null() {
        server_client_unref(ec);
    }
    return CMD_RETURN_NORMAL;
}

// key_event remains a plain C layout value because command states and overlay
// callbacks copy or borrow it. Only queued events use this owner. The Vec keeps
// buf alive across the command queue callback; those copies only inspect the
// key and mouse fields after the callback has returned.
pub struct OwnedKeyEvent {
    event: key_event,
    bytes: Option<Vec<u8>>,
}

impl OwnedKeyEvent {
    pub fn new(key: key_code, m: mouse_event, mut bytes: Option<Vec<u8>>) -> Box<Self> {
        let (buf, len) = match bytes.as_mut() {
            Some(bytes) => (bytes.as_mut_ptr().cast(), bytes.len()),
            None => (::core::ptr::null_mut(), 0),
        };
        Box::new(Self {
            event: key_event {
                client: ::core::ptr::null_mut(),
                key,
                m,
                buf,
                len,
            },
            bytes,
        })
    }
}

unsafe extern "C" fn server_client_handle_menu_key(
    mut c: *mut client,
    mut event: *mut key_event,
) -> ::core::ffi::c_int {
    let mut w: *mut window = (*(*(*c).session).curw).window;
    let mut new_event: key_event = key_event {
        client: ::core::ptr::null_mut::<client>(),
        key: 0,
        m: mouse_event {
            valid: 0,
            ignore: 0,
            key: 0,
            statusat: 0,
            statuslines: 0,
            x: 0,
            y: 0,
            b: 0,
            lx: 0,
            ly: 0,
            lb: 0,
            ox: 0,
            oy: 0,
            s: 0,
            w: 0,
            wp: 0,
            sgr_type: 0,
            sgr_b: 0,
        },
        buf: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        len: 0,
    };
    let mut m: *mut mouse_event = ::core::ptr::null_mut::<mouse_event>();
    let mut ox: u_int = 0;
    let mut oy: u_int = 0;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    if (*w).menu.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    memcpy(
        &raw mut new_event as *mut ::core::ffi::c_void,
        event as *const ::core::ffi::c_void,
        ::core::mem::size_of::<key_event>() as size_t,
    );
    if (*event).key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
        == KEYC_MOUSE as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
        || (*event).key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
            >= (KEYC_TYPE_MOUSEMOVE as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                << 32 as ::core::ffi::c_int
            && (*event).key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                <= (KEYC_TYPE_TRIPLECLICK as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                    << 32 as ::core::ffi::c_int
    {
        m = &raw mut new_event.m;
        (*m).statusat = status_at_line(c);
        (*m).statuslines = status_line_size(c);
        tty_window_offset(
            &raw mut (*c).tty,
            &raw mut ox,
            &raw mut oy,
            &raw mut sx,
            &raw mut sy,
        );
        (*m).x = (*m).x.wrapping_add(ox);
        if (*m).statusat == 0 as ::core::ffi::c_int {
            if (*m).y < (*m).statuslines {
                (*m).y = UINT_MAX as u_int;
                (*m).x = (*m).y;
            } else {
                (*m).y = (*m).y.wrapping_sub((*m).statuslines).wrapping_add(oy);
            }
        } else if (*m).statusat > 0 as ::core::ffi::c_int && (*m).y >= (*m).statusat as u_int {
            (*m).y = UINT_MAX as u_int;
            (*m).x = (*m).y;
        } else {
            (*m).y = (*m).y.wrapping_add(oy);
        }
    }
    if menu_key(c, (*w).menu, &raw mut new_event) == 1 as ::core::ffi::c_int {
        menu_close(w);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe fn server_client_handle_key0(
    mut c: *mut client,
    mut owned: Box<OwnedKeyEvent>,
    mut after: *mut cmdq_item,
    mut next: *mut *mut cmdq_item,
) -> ::core::ffi::c_int {
    let event: *mut key_event = &raw mut owned.event;
    let mut s: *mut session = (*c).session;
    let mut item: *mut cmdq_item = ::core::ptr::null_mut::<cmdq_item>();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    if s.is_null() || (*c).flags & CLIENT_UNATTACHEDFLAGS as uint64_t != 0 {
        return 0 as ::core::ffi::c_int;
    }
    if (*event).key == KEYC_REPORT_LIGHT_THEME as ::core::ffi::c_ulong as key_code {
        server_client_report_theme(c, THEME_LIGHT);
        return 0 as ::core::ffi::c_int;
    }
    if (*event).key == KEYC_REPORT_DARK_THEME as ::core::ffi::c_ulong as key_code {
        server_client_report_theme(c, THEME_DARK);
        return 0 as ::core::ffi::c_int;
    }
    if !(*c).flags & CLIENT_READONLY as uint64_t != 0 {
        if !(*c).message_string.is_null() {
            if (*c).message_ignore_keys != 0 {
                return 0 as ::core::ffi::c_int;
            }
            status_message_clear(c);
        }
        if (*c).overlay_key.is_some() {
            match (*c).overlay_key.expect("non-null function pointer")(c, (*c).overlay_data, event)
            {
                0 => return 0 as ::core::ffi::c_int,
                1 => {
                    server_client_clear_overlay(c);
                    return 0 as ::core::ffi::c_int;
                }
                _ => {}
            }
        }
        server_client_clear_overlay(c);
        wp = (*(*(*s).curw).window).active;
        if server_client_handle_dead_key(wp, (*event).key) != 0 {
            return 0 as ::core::ffi::c_int;
        }
        if !wp.is_null()
            && wp == (*(*wp).window).modal
            && (*wp).flags & PANE_CLOSEONCANCEL != 0
            && ((*event).key == '\u{1b}' as i32 as key_code
                || (*event).key == 'c' as i32 as ::core::ffi::c_ulonglong | KEYC_CTRL)
        {
            server_kill_pane(wp);
            return 0 as ::core::ffi::c_int;
        }
        if !wp.is_null()
            && (*wp).flags & PANE_CAPTUREALLKEYS != 0
            && (*wp).modes.tqh_first.is_null()
            && !((*event).key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
                == KEYC_MOUSE as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
                || (*event).key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                    >= (KEYC_TYPE_MOUSEMOVE as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                        << 32 as ::core::ffi::c_int
                    && (*event).key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                        <= (KEYC_TYPE_TRIPLECLICK as ::core::ffi::c_int
                            as ::core::ffi::c_ulonglong)
                            << 32 as ::core::ffi::c_int)
        {
            if !(*wp).flags & PANE_EXITED != 0 {
                window_pane_key(wp, c, s, (*s).curw, (*event).key, &raw mut (*event).m);
                return 0 as ::core::ffi::c_int;
            }
        }
        if server_client_handle_menu_key(c, event) != 0 {
            return 0 as ::core::ffi::c_int;
        }
        if !(*c).prompt.is_null() {
            match status_prompt_key(c, (*event).key, &raw mut (*event).m) as ::core::ffi::c_uint {
                1 | 2 => return 0 as ::core::ffi::c_int,
                0 | 3 | _ => {}
            }
        }
        wp = (*(*(*s).curw).window).active;
        if wp.is_null() || window_pane_has_prompt(wp) == 0 {
            wp = (*(*(*s).curw).window).panes.tqh_first;
            while !wp.is_null() {
                if window_pane_has_prompt(wp) != 0 && window_pane_is_visible(wp) != 0 {
                    break;
                }
                wp = (*wp).entry.tqe_next;
            }
        }
        if !wp.is_null() && window_pane_has_prompt(wp) != 0 && window_pane_is_visible(wp) != 0 {
            match window_pane_prompt_key(wp, c, (*event).key, &raw mut (*event).m)
                as ::core::ffi::c_uint
            {
                1..=3 => return 0 as ::core::ffi::c_int,
                0 => {
                    if (*event).key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
                        == KEYC_MOUSE as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
                        || (*event).key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                            >= (KEYC_TYPE_MOUSEMOVE as ::core::ffi::c_int
                                as ::core::ffi::c_ulonglong)
                                << 32 as ::core::ffi::c_int
                            && (*event).key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                                <= (KEYC_TYPE_TRIPLECLICK as ::core::ffi::c_int
                                    as ::core::ffi::c_ulonglong)
                                    << 32 as ::core::ffi::c_int
                    {
                        return 0 as ::core::ffi::c_int;
                    }
                }
                _ => {}
            }
        }
    }
    let queued_event = Box::into_raw(owned);
    item = cmdq_get_callback1(
        b"server_client_key_callback\0" as *const u8 as *const ::core::ffi::c_char,
        Some(
            server_client_key_callback
                as unsafe extern "C" fn(*mut cmdq_item, *mut ::core::ffi::c_void) -> cmd_retval,
        ),
        queued_event as *mut ::core::ffi::c_void,
    );
    if !after.is_null() {
        (*event).client = c;
        (*c).references += 1;
        item = cmdq_insert_after(after, item);
        if !next.is_null() {
            *next = item;
        }
        return 1 as ::core::ffi::c_int;
    }
    cmdq_append(c, item);
    return 1 as ::core::ffi::c_int;
}
pub unsafe fn server_client_handle_key(
    mut c: *mut client,
    event: Box<OwnedKeyEvent>,
) -> ::core::ffi::c_int {
    return server_client_handle_key0(
        c,
        event,
        ::core::ptr::null_mut::<cmdq_item>(),
        ::core::ptr::null_mut::<*mut cmdq_item>(),
    );
}
pub unsafe fn server_client_handle_key_after(
    mut c: *mut client,
    event: Box<OwnedKeyEvent>,
    mut after: *mut cmdq_item,
    mut next: *mut *mut cmdq_item,
) -> ::core::ffi::c_int {
    return server_client_handle_key0(c, event, after, next);
}
#[no_mangle]
pub unsafe extern "C" fn server_client_loop() {
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
    w = windows_minmax(&raw mut windows, RB_NEGINF);
    while !w.is_null() {
        server_client_check_window_resize(w);
        w = windows_next(w);
    }
    w = windows_minmax(&raw mut windows, RB_NEGINF);
    while !w.is_null() {
        wp = (*w).panes.tqh_first;
        while !wp.is_null() {
            if (*wp).flags & PANE_STYLECHANGED != 0 {
                wme = (*wp).modes.tqh_first;
                if !wme.is_null() && (*(*wme).mode).style_changed.is_some() {
                    (*(*wme).mode)
                        .style_changed
                        .expect("non-null function pointer")(wme);
                }
            }
            wp = (*wp).entry.tqe_next;
        }
        w = windows_next(w);
    }
    c = clients.tqh_first;
    while !c.is_null() {
        server_client_check_exit(c, 0 as ::core::ffi::c_int);
        if !(*c).session.is_null() && !(*(*c).session).curw.is_null() {
            server_client_check_modes(c);
            server_client_check_redraw(c);
            server_client_reset_state(c);
        }
        c = (*c).entry.tqe_next;
    }
    w = windows_minmax(&raw mut windows, RB_NEGINF);
    while !w.is_null() {
        wp = (*w).panes.tqh_first;
        while !wp.is_null() {
            if (*wp).fd != -(1 as ::core::ffi::c_int) {
                server_client_check_pane_resize(wp);
                server_client_check_pane_buffer(wp);
            }
            (*wp).flags &= !(PANE_REDRAW | PANE_REDRAWSCROLLBAR | PANE_ACTIVITY);
            wp = (*wp).entry.tqe_next;
        }
        check_window_name(w);
        w = windows_next(w);
    }
    w = windows_minmax(&raw mut windows, RB_NEGINF);
    while !w.is_null() {
        wp = (*w).panes.tqh_first;
        while !wp.is_null() {
            window_pane_send_theme_update(wp);
            wp = (*wp).entry.tqe_next;
        }
        w = windows_next(w);
    }
}
unsafe extern "C" fn server_client_check_window_resize(mut w: *mut window) {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    if !(*w).flags & WINDOW_RESIZE != 0 {
        return;
    }
    wl = (*w).winlinks.tqh_first;
    while !wl.is_null() {
        if (*(*wl).session).attached != 0 as u_int && (*(*wl).session).curw == wl {
            break;
        }
        wl = (*wl).wentry.tqe_next;
    }
    if wl.is_null() {
        return;
    }
    log_debug(
        b"%s: resizing window @%u\0" as *const u8 as *const ::core::ffi::c_char,
        b"server_client_check_window_resize\0" as *const u8 as *const ::core::ffi::c_char,
        (*w).id,
    );
    resize_window(
        w,
        (*w).new_sx,
        (*w).new_sy,
        (*w).new_xpixel as ::core::ffi::c_int,
        (*w).new_ypixel as ::core::ffi::c_int,
    );
}
unsafe extern "C" fn server_client_resize_timer(
    mut fd: ::core::ffi::c_int,
    mut events: ::core::ffi::c_short,
    mut data: *mut ::core::ffi::c_void,
) {
    let mut wp: *mut window_pane = data as *mut window_pane;
    log_debug(
        b"%s: %%%u resize timer expired\0" as *const u8 as *const ::core::ffi::c_char,
        b"server_client_resize_timer\0" as *const u8 as *const ::core::ffi::c_char,
        (*wp).id,
    );
    event_del(&raw mut (*wp).resize_timer);
}
unsafe extern "C" fn server_client_check_pane_resize(mut wp: *mut window_pane) {
    let mut tv: timeval = timeval {
        tv_sec: 0,
        tv_usec: 250000 as __suseconds_t,
    };
    if (*wp).resize_queue.is_empty() {
        return;
    }
    if event_initialized(&raw mut (*wp).resize_timer) == 0 {
        event_set(
            &raw mut (*wp).resize_timer,
            -(1 as ::core::ffi::c_int),
            0 as ::core::ffi::c_short,
            Some(
                server_client_resize_timer
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        ::core::ffi::c_short,
                        *mut ::core::ffi::c_void,
                    ) -> (),
            ),
            wp as *mut ::core::ffi::c_void,
        );
    }
    if event_pending(
        &raw mut (*wp).resize_timer,
        EV_TIMEOUT as ::core::ffi::c_short,
        ::core::ptr::null_mut::<timeval>(),
    ) != 0
    {
        return;
    }
    log_debug(
        b"%s: %%%u needs to be resized\0" as *const u8 as *const ::core::ffi::c_char,
        b"server_client_check_pane_resize\0" as *const u8 as *const ::core::ffi::c_char,
        (*wp).id,
    );
    let (queue_len, first_sx, first_sy, first_osx, first_osy, last_sx, last_sy, last_ptr, previous) = {
        let queue = (*wp)
            .resize_queue
            .as_ref()
            .expect("non-empty resize queue must have storage");
        for resize in queue {
            log_debug(
                b"queued resize: %ux%u -> %ux%u\0" as *const u8 as *const ::core::ffi::c_char,
                resize.osx,
                resize.osy,
                resize.sx,
                resize.sy,
            );
        }
        let first = queue.front().expect("non-empty resize queue");
        let last = queue.back().expect("non-empty resize queue");
        let previous = if queue.len() > 1 {
            Some(
                queue
                    .get(queue.len() - 2)
                    .expect("resize queue has a predecessor")
                    .as_ref(),
            )
        } else {
            None
        };
        (
            queue.len(),
            first.sx,
            first.sy,
            first.osx,
            first.osy,
            last.sx,
            last.sy,
            (&**last) as *const window_pane_resize as *mut window_pane_resize,
            previous.map(|resize| (resize.sx, resize.sy)),
        )
    };
    if queue_len == 1 {
        window_pane_send_resize(wp, first_sx, first_sy);
        window_pane_clear_resizes(wp, ::core::ptr::null_mut::<window_pane_resize>());
    } else if last_sx != first_osx || last_sy != first_osy {
        window_pane_send_resize(wp, last_sx, last_sy);
        window_pane_clear_resizes(wp, ::core::ptr::null_mut::<window_pane_resize>());
    } else {
        let (sx, sy) = previous.expect("multiple resize entries have a predecessor");
        window_pane_send_resize(wp, sx, sy);
        window_pane_clear_resizes(wp, last_ptr);
        tv.tv_usec = 10000 as __suseconds_t;
    }
    event_add(&raw mut (*wp).resize_timer, &raw mut tv);
}
unsafe extern "C" fn server_client_check_pane_buffer(mut wp: *mut window_pane) {
    let mut evb: *mut evbuffer = (*(*wp).event).input;
    let mut minimum: size_t = 0;
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut wpo: *mut window_pane_offset = ::core::ptr::null_mut::<window_pane_offset>();
    let mut off: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    let mut flag: ::core::ffi::c_int = 0;
    let mut attached_clients: u_int = 0 as u_int;
    let mut new_size: size_t = 0;
    minimum = (*wp).offset.used;
    if (*wp).pipe_fd != -(1 as ::core::ffi::c_int) && (*wp).pipe_offset.used < minimum {
        minimum = (*wp).pipe_offset.used;
    }
    c = clients.tqh_first;
    while !c.is_null() {
        if !(*c).session.is_null() {
            attached_clients = attached_clients.wrapping_add(1);
            if !(*c).flags & CLIENT_CONTROL as uint64_t != 0 {
                off = 0 as ::core::ffi::c_int;
            } else {
                wpo = control_pane_offset(c, wp, &raw mut flag);
                if wpo.is_null() {
                    if flag == 0 {
                        off = 0 as ::core::ffi::c_int;
                    }
                } else {
                    if flag == 0 {
                        off = 0 as ::core::ffi::c_int;
                    }
                    window_pane_get_new_data(wp, wpo, &raw mut new_size);
                    log_debug(
                        b"%s: %s has %zu bytes used and %zu left for %%%u\0" as *const u8
                            as *const ::core::ffi::c_char,
                        b"server_client_check_pane_buffer\0" as *const u8
                            as *const ::core::ffi::c_char,
                        (*c).name,
                        (*wpo).used.wrapping_sub((*wp).base_offset),
                        new_size,
                        (*wp).id,
                    );
                    if (*wpo).used < minimum {
                        minimum = (*wpo).used;
                    }
                }
            }
        }
        c = (*c).entry.tqe_next;
    }
    if attached_clients == 0 as u_int {
        off = 0 as ::core::ffi::c_int;
    }
    minimum = minimum.wrapping_sub((*wp).base_offset);
    if !(minimum == 0 as size_t) {
        log_debug(
            b"%s: %%%u has %zu minimum (of %zu) bytes used\0" as *const u8
                as *const ::core::ffi::c_char,
            b"server_client_check_pane_buffer\0" as *const u8 as *const ::core::ffi::c_char,
            (*wp).id,
            minimum,
            evbuffer_get_length(evb),
        );
        evbuffer_drain(evb, minimum);
        if (*wp).base_offset > (SIZE_MAX as size_t).wrapping_sub(minimum) {
            log_debug(
                b"%s: %%%u base offset has wrapped\0" as *const u8 as *const ::core::ffi::c_char,
                b"server_client_check_pane_buffer\0" as *const u8 as *const ::core::ffi::c_char,
                (*wp).id,
            );
            (*wp).offset.used = (*wp).offset.used.wrapping_sub((*wp).base_offset);
            if (*wp).pipe_fd != -(1 as ::core::ffi::c_int) {
                (*wp).pipe_offset.used = (*wp).pipe_offset.used.wrapping_sub((*wp).base_offset);
            }
            c = clients.tqh_first;
            while !c.is_null() {
                if !((*c).session.is_null() || !(*c).flags & CLIENT_CONTROL as uint64_t != 0) {
                    wpo = control_pane_offset(c, wp, &raw mut flag);
                    if !wpo.is_null() && flag == 0 {
                        (*wpo).used = (*wpo).used.wrapping_sub((*wp).base_offset);
                    }
                }
                c = (*c).entry.tqe_next;
            }
            (*wp).base_offset = minimum;
        } else {
            (*wp).base_offset = (*wp).base_offset.wrapping_add(minimum);
        }
    }
    log_debug(
        b"%s: pane %%%u is %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"server_client_check_pane_buffer\0" as *const u8 as *const ::core::ffi::c_char,
        (*wp).id,
        if off != 0 {
            b"off\0" as *const u8 as *const ::core::ffi::c_char
        } else {
            b"on\0" as *const u8 as *const ::core::ffi::c_char
        },
    );
    if off != 0 {
        bufferevent_disable((*wp).event, EV_READ as ::core::ffi::c_short);
    } else {
        bufferevent_enable((*wp).event, EV_READ as ::core::ffi::c_short);
    };
}
unsafe extern "C" fn server_client_prompt_cursor(
    mut c: *mut client,
    mut wp: *mut window_pane,
    mut mode: *mut ::core::ffi::c_int,
    mut cx: *mut u_int,
    mut cy: *mut u_int,
) -> ::core::ffi::c_int {
    let mut tty: *mut tty = &raw mut (*c).tty;
    let mut r: *mut visible_ranges = ::core::ptr::null_mut::<visible_ranges>();
    let mut ox: u_int = 0;
    let mut oy: u_int = 0;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    let mut px: ::core::ffi::c_int = 0;
    let mut py: ::core::ffi::c_int = 0;
    if window_pane_has_prompt(wp) == 0 {
        return 0 as ::core::ffi::c_int;
    }
    *mode &= !MODE_CURSOR;
    tty_window_offset(tty, &raw mut ox, &raw mut oy, &raw mut sx, &raw mut sy);
    if status_at_line(c) == 0 as ::core::ffi::c_int {
        py = (*wp).yoff;
    } else {
        py = ((*wp).yoff as u_int)
            .wrapping_add((*wp).sy)
            .wrapping_sub(1 as u_int) as ::core::ffi::c_int;
    }
    px = ((*wp).xoff as u_int).wrapping_add((*wp).prompt_cx) as ::core::ffi::c_int;
    if px < ox as ::core::ffi::c_int
        || px > ox.wrapping_add(sx) as ::core::ffi::c_int
        || py < oy as ::core::ffi::c_int
        || py > oy.wrapping_add(sy) as ::core::ffi::c_int
    {
        return 1 as ::core::ffi::c_int;
    }
    *cx = (px as u_int).wrapping_sub(ox);
    *cy = (py as u_int).wrapping_sub(oy);
    r = window_visible_ranges(
        wp,
        *cx as ::core::ffi::c_int,
        *cy as ::core::ffi::c_int,
        1 as u_int,
        ::core::ptr::null_mut::<visible_ranges>(),
    );
    if window_position_is_visible(r, *cx) != 0 {
        if status_at_line(c) == 0 as ::core::ffi::c_int {
            *cy = (*cy).wrapping_add(status_line_size(c));
        }
        *mode |= MODE_CURSOR;
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn server_client_reset_state(mut c: *mut client) {
    let mut tty: *mut tty = &raw mut (*c).tty;
    let mut w: *mut window = (*(*(*c).session).curw).window;
    let mut wp: *mut window_pane = (*w).active;
    let mut loop_0: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut s: *mut screen = ::core::ptr::null_mut::<screen>();
    let mut oo: *mut options = (*(*c).session).options;
    let mut mode: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut cursor: ::core::ffi::c_int = 0;
    let mut flags: ::core::ffi::c_int = 0;
    let mut pane_mode: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut cx: u_int = 0 as u_int;
    let mut cy: u_int = 0 as u_int;
    let mut ox: u_int = 0;
    let mut oy: u_int = 0;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    let mut prompt: u_int = 0 as u_int;
    let mut sb_w: u_int = 0;
    let mut r: *mut visible_ranges = ::core::ptr::null_mut::<visible_ranges>();
    if (*c).flags & (CLIENT_CONTROL | CLIENT_SUSPENDED) as uint64_t != 0 {
        return;
    }
    flags = (*tty).flags & TTY_BLOCK;
    (*tty).flags &= !TTY_BLOCK;
    if (*c).overlay_draw.is_some() {
        if (*c).overlay_mode.is_some() {
            s = (*c).overlay_mode.expect("non-null function pointer")(
                c,
                (*c).overlay_data,
                &raw mut cx,
                &raw mut cy,
            );
        }
    } else if !(*w).menu.is_null() {
        menu_get_cursor((*w).menu, &raw mut cx, &raw mut cy);
        s = menu_screen((*w).menu);
    } else if !wp.is_null() && (*c).prompt.is_null() {
        s = (*wp).screen;
    } else {
        s = (*c).status.active;
    }
    if !s.is_null() {
        mode = (*s).mode;
    }
    if log_get_level() != 0 as ::core::ffi::c_int {
        log_debug(
            b"%s: client %s mode %s\0" as *const u8 as *const ::core::ffi::c_char,
            b"server_client_reset_state\0" as *const u8 as *const ::core::ffi::c_char,
            (*c).name,
            screen_mode_to_string(mode),
        );
    }
    tty_region_off(tty);
    tty_margin_off(tty);
    if !(*c).prompt.is_null() {
        prompt = 1 as u_int;
        status_prompt_cursor(c, &raw mut cx, &raw mut cy);
    } else if !wp.is_null() && (*c).overlay_draw.is_none() {
        if !(*w).menu.is_null() {
            tty_window_offset(tty, &raw mut ox, &raw mut oy, &raw mut sx, &raw mut sy);
            if cx < ox || cx >= ox.wrapping_add(sx) || cy < oy || cy >= oy.wrapping_add(sy) {
                mode &= !MODE_CURSOR;
            } else {
                cx = cx.wrapping_sub(ox);
                cy = cy.wrapping_sub(oy);
                if status_at_line(c) == 0 as ::core::ffi::c_int {
                    cy = cy.wrapping_add(status_line_size(c));
                }
            }
            prompt = 1 as u_int;
        } else {
            prompt = server_client_prompt_cursor(c, wp, &raw mut mode, &raw mut cx, &raw mut cy)
                as u_int;
        }
        if prompt == 0 {
            cursor = 0 as ::core::ffi::c_int;
            pane_mode = (*wp).base.mode;
            tty_window_offset(tty, &raw mut ox, &raw mut oy, &raw mut sx, &raw mut sy);
            if (*wp).xoff + (*s).cx as ::core::ffi::c_int >= ox as ::core::ffi::c_int
                && (*wp).xoff + (*s).cx as ::core::ffi::c_int
                    <= ox as ::core::ffi::c_int + sx as ::core::ffi::c_int
                && (*wp).yoff + (*s).cy as ::core::ffi::c_int >= oy as ::core::ffi::c_int
                && (*wp).yoff + (*s).cy as ::core::ffi::c_int
                    <= oy as ::core::ffi::c_int + sy as ::core::ffi::c_int
            {
                cursor = 1 as ::core::ffi::c_int;
                cx = ((*wp).xoff + (*s).cx as ::core::ffi::c_int - ox as ::core::ffi::c_int)
                    as u_int;
                cy = ((*wp).yoff + (*s).cy as ::core::ffi::c_int - oy as ::core::ffi::c_int)
                    as u_int;
                r = window_visible_ranges(
                    wp,
                    cx as ::core::ffi::c_int,
                    cy as ::core::ffi::c_int,
                    1 as u_int,
                    ::core::ptr::null_mut::<visible_ranges>(),
                );
                if window_position_is_visible(r, cx) == 0 {
                    cursor = 0 as ::core::ffi::c_int;
                }
                if window_pane_scrollbar_overlay_visible(wp) != 0 {
                    sb_w = (*wp).scrollbar_style.width as u_int;
                    if sb_w > (*wp).sx {
                        sb_w = (*wp).sx;
                    }
                    if sb_w != 0 as u_int && (*w).sb_pos == PANE_SCROLLBARS_LEFT {
                        if (*s).cx < sb_w {
                            cursor = 0 as ::core::ffi::c_int;
                        }
                    } else if sb_w != 0 as u_int && (*s).cx >= (*wp).sx.wrapping_sub(sb_w) {
                        cursor = 0 as ::core::ffi::c_int;
                    }
                }
                if status_at_line(c) == 0 as ::core::ffi::c_int {
                    cy = cy.wrapping_add(status_line_size(c));
                }
            }
            if cursor == 0 {
                mode &= !MODE_CURSOR;
            }
        }
    } else if (*c).overlay_mode.is_none() || s.is_null() {
        mode &= !MODE_CURSOR;
    }
    if !pane_mode & MODE_SYNC != 0 {
        log_debug(
            b"%s: cursor to %u,%u\0" as *const u8 as *const ::core::ffi::c_char,
            b"server_client_reset_state\0" as *const u8 as *const ::core::ffi::c_char,
            cx,
            cy,
        );
        tty_cursor(tty, cx, cy);
    } else {
        mode &= !CURSOR_MODES;
        mode |= (*tty).mode & CURSOR_MODES;
        s = ::core::ptr::null_mut::<screen>();
    }
    if options_get_number(oo, b"mouse\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        if (*c).overlay_draw.is_none() && (*w).menu.is_null() {
            mode &= !ALL_MOUSE_MODES;
            loop_0 = (*w).panes.tqh_first;
            while !loop_0.is_null() {
                if (*(*loop_0).screen).mode & MODE_MOUSE_ALL != 0 {
                    mode |= MODE_MOUSE_ALL;
                }
                loop_0 = (*loop_0).entry.tqe_next;
            }
        }
        if options_get_number(
            oo,
            b"focus-follows-mouse\0" as *const u8 as *const ::core::ffi::c_char,
        ) != 0
            || (*w).sb == PANE_SCROLLBARS_MODAL
            || (*w).sb == PANE_SCROLLBARS_AUTOHIDE
        {
            mode |= MODE_MOUSE_ALL;
        } else if !mode & MODE_MOUSE_ALL != 0 {
            mode |= MODE_MOUSE_BUTTON;
        }
    }
    if (*c).overlay_draw.is_none() && prompt != 0 {
        mode &= !MODE_BRACKETPASTE;
    }
    tty_update_mode(tty, mode, s);
    tty_reset(tty);
    tty_sync_end(tty);
    (*tty).flags |= flags;
}
unsafe extern "C" fn server_client_repeat_timer(
    mut fd: ::core::ffi::c_int,
    mut events: ::core::ffi::c_short,
    mut data: *mut ::core::ffi::c_void,
) {
    let mut c: *mut client = data as *mut client;
    if (*c).flags & CLIENT_REPEAT as uint64_t != 0 {
        server_client_set_key_table(c, ::core::ptr::null::<::core::ffi::c_char>());
        (*c).flags &= !CLIENT_REPEAT as uint64_t;
        server_status_client(c);
    }
}
unsafe extern "C" fn server_client_click_timer(
    mut fd: ::core::ffi::c_int,
    mut events: ::core::ffi::c_short,
    mut data: *mut ::core::ffi::c_void,
) {
    let mut c: *mut client = data as *mut client;
    log_debug(b"click timer expired\0" as *const u8 as *const ::core::ffi::c_char);
    if (*c).flags & CLIENT_TRIPLECLICK as uint64_t != 0 {
        let event = OwnedKeyEvent::new(
            KEYC_DOUBLECLICK as ::core::ffi::c_ulong as key_code,
            (*c).click_event,
            None,
        );
        server_client_handle_key(c, event);
    }
    (*c).flags &= !(CLIENT_DOUBLECLICK | CLIENT_TRIPLECLICK) as uint64_t;
}
unsafe extern "C" fn server_client_start_exit_timer(mut c: *mut client) {
    let mut tv: timeval = timeval {
        tv_sec: 10 as __time_t,
        tv_usec: 0,
    };
    if event_pending(
        &raw mut (*c).exit_timer,
        EV_TIMEOUT as ::core::ffi::c_short,
        ::core::ptr::null_mut::<timeval>(),
    ) == 0
    {
        event_add(&raw mut (*c).exit_timer, &raw mut tv);
    }
}
unsafe extern "C" fn server_client_exit_timer(
    mut fd: ::core::ffi::c_int,
    mut events: ::core::ffi::c_short,
    mut data: *mut ::core::ffi::c_void,
) {
    let mut c: *mut client = data as *mut client;
    if (*c).flags & (CLIENT_DEAD | CLIENT_SUSPENDED) as uint64_t != 0 {
        return;
    }
    if (*c).flags & CLIENT_EXITED as uint64_t != 0 {
        log_debug(
            b"%s: %s took too long to exit\0" as *const u8 as *const ::core::ffi::c_char,
            b"server_client_exit_timer\0" as *const u8 as *const ::core::ffi::c_char,
            (*c).name,
        );
        server_client_lost(c);
    } else if (*c).flags & CLIENT_EXIT as uint64_t != 0 {
        log_debug(
            b"%s: %s took too long to flush\0" as *const u8 as *const ::core::ffi::c_char,
            b"server_client_exit_timer\0" as *const u8 as *const ::core::ffi::c_char,
            (*c).name,
        );
        server_client_check_exit(c, 1 as ::core::ffi::c_int);
    }
}
unsafe extern "C" fn server_client_check_exit(mut c: *mut client, mut force: ::core::ffi::c_int) {
    let mut cf: *mut client_file = ::core::ptr::null_mut::<client_file>();
    let mut name: *const ::core::ffi::c_char = (*c).exit_session;
    if (*c).flags & (CLIENT_DEAD | CLIENT_EXITED) as uint64_t != 0 {
        return;
    }
    if !(*c).flags & CLIENT_EXIT as uint64_t != 0 {
        return;
    }
    if (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
        if force != 0 {
            control_discard_all(c);
        } else {
            control_discard(c);
            if control_all_done(c) == 0 {
                server_client_start_exit_timer(c);
                return;
            }
        }
    }
    if force == 0 {
        cf = client_files_minmax(&raw mut (*c).files, RB_NEGINF);
        while !cf.is_null() {
            if evbuffer_get_length((*cf).buffer) != 0 as size_t {
                server_client_start_exit_timer(c);
                return;
            }
            cf = client_files_next(cf);
        }
    }
    (*c).flags |= CLIENT_EXITED as uint64_t;
    event_del(&raw mut (*c).exit_timer);
    server_client_start_exit_timer(c);
    match (*c).exit_type as ::core::ffi::c_uint {
        0 => {
            let mut data = Vec::from((*c).retval.to_ne_bytes());
            if !(*c).exit_message.is_null() {
                data.extend_from_slice(
                    ::std::ffi::CStr::from_ptr((*c).exit_message).to_bytes_with_nul(),
                );
            }
            proc_send(
                (*c).peer,
                MSG_EXIT,
                -(1 as ::core::ffi::c_int),
                data.as_ptr() as *const ::core::ffi::c_void,
                data.len() as size_t,
            );
        }
        1 => {
            proc_send(
                (*c).peer,
                MSG_SHUTDOWN,
                -(1 as ::core::ffi::c_int),
                ::core::ptr::null::<::core::ffi::c_void>(),
                0 as size_t,
            );
        }
        2 => {
            proc_send(
                (*c).peer,
                (*c).exit_msgtype,
                -(1 as ::core::ffi::c_int),
                name as *const ::core::ffi::c_void,
                strlen(name).wrapping_add(1 as size_t),
            );
        }
        _ => {}
    };
}
unsafe extern "C" fn server_client_redraw_timer(
    mut fd: ::core::ffi::c_int,
    mut events: ::core::ffi::c_short,
    mut data: *mut ::core::ffi::c_void,
) {
    log_debug(b"redraw timer fired\0" as *const u8 as *const ::core::ffi::c_char);
}
unsafe extern "C" fn server_client_check_modes(mut c: *mut client) {
    let mut w: *mut window = (*(*(*c).session).curw).window;
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
    if (*c).flags & (CLIENT_CONTROL | CLIENT_SUSPENDED) as uint64_t != 0 {
        return;
    }
    if !(*c).flags & CLIENT_REDRAWSTATUS as uint64_t != 0 {
        return;
    }
    wp = (*w).panes.tqh_first;
    while !wp.is_null() {
        wme = (*wp).modes.tqh_first;
        if !wme.is_null() && (*(*wme).mode).update.is_some() {
            (*(*wme).mode).update.expect("non-null function pointer")(wme);
        }
        wp = (*wp).entry.tqe_next;
    }
}
unsafe extern "C" fn server_client_any_pane_redraw(mut c: *mut client) -> ::core::ffi::c_int {
    let mut s: *mut session = (*c).session;
    let mut w: *mut window = (*(*s).curw).window;
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    if (*c).flags & CLIENT_REDRAWWINDOW as uint64_t != 0 {
        return 1 as ::core::ffi::c_int;
    }
    wp = (*w).panes.tqh_first;
    while !wp.is_null() {
        if (*wp).flags & (PANE_REDRAW | PANE_REDRAWSCROLLBAR) != 0 {
            return 1 as ::core::ffi::c_int;
        }
        wp = (*wp).entry.tqe_next;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn server_client_check_redraw(mut c: *mut client) {
    let mut s: *mut session = (*c).session;
    let mut tty: *mut tty = &raw mut (*c).tty;
    let mut w: *mut window = (*(*s).curw).window;
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut needed: ::core::ffi::c_int = 0;
    let mut tflags: ::core::ffi::c_int = 0;
    let mut mode: ::core::ffi::c_int = (*tty).mode;
    let mut tv: timeval = timeval {
        tv_sec: 0,
        tv_usec: 1000 as __suseconds_t,
    };
    static mut ev: event = event {
        ev_evcallback: event_callback {
            evcb_active_next: event_callback_entry {
                tqe_next: ::core::ptr::null::<event_callback>() as *mut event_callback,
                tqe_prev: ::core::ptr::null::<*mut event_callback>() as *mut *mut event_callback,
            },
            evcb_flags: 0,
            evcb_pri: 0,
            evcb_closure: 0,
            evcb_cb_union: event_callback_union {
                evcb_callback: None,
            },
            evcb_arg: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
        },
        ev_timeout_pos: event_timeout_pos {
            ev_next_with_common_timeout: event_timeout_entry {
                tqe_next: ::core::ptr::null::<event>() as *mut event,
                tqe_prev: ::core::ptr::null::<*mut event>() as *mut *mut event,
            },
        },
        ev_fd: 0,
        ev_base: ::core::ptr::null::<event_base>() as *mut event_base,
        ev_: event_io_or_signal {
            ev_io: event_io {
                ev_io_next: event_io_entry {
                    le_next: ::core::ptr::null::<event>() as *mut event,
                    le_prev: ::core::ptr::null::<*mut event>() as *mut *mut event,
                },
                ev_timeout: timeval {
                    tv_sec: 0,
                    tv_usec: 0,
                },
            },
        },
        ev_events: 0,
        ev_res: 0,
        ev_timeout: timeval {
            tv_sec: 0,
            tv_usec: 0,
        },
    };
    let mut n: size_t = 0;
    if (*c).flags & (CLIENT_CONTROL | CLIENT_SUSPENDED) as uint64_t != 0 {
        return;
    }
    if (*c).flags & CLIENT_ALLREDRAWFLAGS as uint64_t != 0 {
        log_debug(
            b"%s: redraw%s%s%s%s%s\0" as *const u8 as *const ::core::ffi::c_char,
            (*c).name,
            if (*c).flags & CLIENT_REDRAWWINDOW as uint64_t != 0 {
                b" window\0" as *const u8 as *const ::core::ffi::c_char
            } else {
                b"\0" as *const u8 as *const ::core::ffi::c_char
            },
            if (*c).flags & CLIENT_REDRAWSTATUS as uint64_t != 0 {
                b" status\0" as *const u8 as *const ::core::ffi::c_char
            } else {
                b"\0" as *const u8 as *const ::core::ffi::c_char
            },
            if (*c).flags & CLIENT_REDRAWBORDERS as uint64_t != 0 {
                b" borders\0" as *const u8 as *const ::core::ffi::c_char
            } else {
                b"\0" as *const u8 as *const ::core::ffi::c_char
            },
            if (*c).flags & CLIENT_REDRAWOVERLAY as uint64_t != 0 {
                b" overlay\0" as *const u8 as *const ::core::ffi::c_char
            } else {
                b"\0" as *const u8 as *const ::core::ffi::c_char
            },
            if (*c).flags & CLIENT_REDRAWMENU as uint64_t != 0 {
                b" menu\0" as *const u8 as *const ::core::ffi::c_char
            } else {
                b"\0" as *const u8 as *const ::core::ffi::c_char
            },
        );
    }
    needed = 0 as ::core::ffi::c_int;
    if (*c).flags as ::core::ffi::c_ulonglong
        & (CLIENT_ALLREDRAWFLAGS as ::core::ffi::c_ulonglong | CLIENT_REDRAWSCROLLBARS)
        != 0
    {
        needed = 1 as ::core::ffi::c_int;
    } else if server_client_any_pane_redraw(c) != 0 {
        needed = 1 as ::core::ffi::c_int;
    }
    if needed == 0 {
        (*c).flags &= !CLIENT_STATUSFORCE as uint64_t;
        return;
    }
    n = evbuffer_get_length((*tty).out);
    if n != 0 as size_t || (*tty).flags & TTY_BLOCK != 0 {
        if n != 0 as size_t {
            log_debug(
                b"%s: redraw deferred (%zu left)\0" as *const u8 as *const ::core::ffi::c_char,
                (*c).name,
                n,
            );
        } else {
            log_debug(
                b"%s: redraw deferred (blocked)\0" as *const u8 as *const ::core::ffi::c_char,
                (*c).name,
            );
        }
        if event_initialized(&raw mut ev) == 0 {
            event_set(
                &raw mut ev,
                -(1 as ::core::ffi::c_int),
                0 as ::core::ffi::c_short,
                Some(
                    server_client_redraw_timer
                        as unsafe extern "C" fn(
                            ::core::ffi::c_int,
                            ::core::ffi::c_short,
                            *mut ::core::ffi::c_void,
                        ) -> (),
                ),
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
            );
        }
        if event_pending(
            &raw mut ev,
            EV_TIMEOUT as ::core::ffi::c_short,
            ::core::ptr::null_mut::<timeval>(),
        ) == 0
        {
            log_debug(b"redraw timer started\0" as *const u8 as *const ::core::ffi::c_char);
            event_add(&raw mut ev, &raw mut tv);
        }
        wp = (*w).panes.tqh_first;
        while !wp.is_null() {
            if (*wp).flags & PANE_REDRAW != 0 {
                (*c).flags |= CLIENT_REDRAWWINDOW as uint64_t;
                break;
            } else {
                if (*wp).flags & PANE_REDRAWSCROLLBAR != 0 {
                    (*c).flags = ((*c).flags as ::core::ffi::c_ulonglong | CLIENT_REDRAWSCROLLBARS)
                        as uint64_t;
                }
                wp = (*wp).entry.tqe_next;
            }
        }
        return;
    }
    log_debug(
        b"%s: redraw needed\0" as *const u8 as *const ::core::ffi::c_char,
        (*c).name,
    );
    tflags = (*tty).flags & (TTY_BLOCK | TTY_FREEZE | TTY_NOCURSOR);
    (*tty).flags = (*tty).flags & !(TTY_BLOCK | TTY_FREEZE) | TTY_NOCURSOR;
    if !(*c).flags & CLIENT_REDRAWWINDOW as uint64_t != 0 {
        wp = (*w).panes.tqh_first;
        while !wp.is_null() {
            if (*wp).flags & PANE_REDRAW != 0 {
                log_debug(
                    b"%s: redraw pane %%%u\0" as *const u8 as *const ::core::ffi::c_char,
                    b"server_client_check_redraw\0" as *const u8 as *const ::core::ffi::c_char,
                    (*wp).id,
                );
                redraw_pane(c, wp);
            } else if (*wp).flags & PANE_REDRAWSCROLLBAR != 0
                || (*c).flags as ::core::ffi::c_ulonglong & CLIENT_REDRAWSCROLLBARS != 0
            {
                log_debug(
                    b"%s: redraw scrollbar %%%u\0" as *const u8 as *const ::core::ffi::c_char,
                    b"server_client_check_redraw\0" as *const u8 as *const ::core::ffi::c_char,
                    (*wp).id,
                );
                redraw_pane_scrollbar(c, wp);
            }
            wp = (*wp).entry.tqe_next;
        }
    }
    if (*c).flags & CLIENT_ALLREDRAWFLAGS as uint64_t != 0 {
        if options_get_number(
            (*s).options,
            b"set-titles\0" as *const u8 as *const ::core::ffi::c_char,
        ) != 0
        {
            server_client_set_title(c);
            server_client_set_path(c);
        }
        server_client_set_progress_bar(c);
        redraw_screen(c);
    }
    (*tty).flags = (*tty).flags & !TTY_NOCURSOR | tflags & TTY_NOCURSOR;
    tty_update_mode(tty, mode, ::core::ptr::null_mut::<screen>());
    (*tty).flags = (*tty).flags & !(TTY_BLOCK | TTY_FREEZE | TTY_NOCURSOR) | tflags;
    (*c).flags = ((*c).flags as ::core::ffi::c_ulonglong
        & !(CLIENT_ALLREDRAWFLAGS as ::core::ffi::c_ulonglong
            | CLIENT_REDRAWSCROLLBARS
            | CLIENT_STATUSFORCE as ::core::ffi::c_ulonglong)) as uint64_t;
    (*c).redraw = evbuffer_get_length((*tty).out);
    log_debug(
        b"%s: redraw added %zu bytes\0" as *const u8 as *const ::core::ffi::c_char,
        (*c).name,
        (*c).redraw,
    );
}
unsafe extern "C" fn server_client_set_title(mut c: *mut client) {
    let mut s: *mut session = (*c).session;
    let mut template: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut title: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    template = options_get_string(
        (*s).options,
        b"set-titles-string\0" as *const u8 as *const ::core::ffi::c_char,
    );
    ft = format_create(
        c,
        ::core::ptr::null_mut::<cmdq_item>(),
        FORMAT_NONE,
        0 as ::core::ffi::c_int,
    );
    format_defaults(
        ft,
        c,
        ::core::ptr::null_mut::<session>(),
        ::core::ptr::null_mut::<winlink>(),
        ::core::ptr::null_mut::<window_pane>(),
    );
    title = format_expand_time(ft, template);
    if (*c).title.is_null() || strcmp(title, (*c).title) != 0 as ::core::ffi::c_int {
        free((*c).title as *mut ::core::ffi::c_void);
        (*c).title = xstrdup(title);
        tty_set_title(&raw mut (*c).tty, (*c).title);
    }
    free(title as *mut ::core::ffi::c_void);
    format_free(ft);
}
unsafe extern "C" fn server_client_set_path(mut c: *mut client) {
    let mut s: *mut session = (*c).session;
    let mut path: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if (*s).curw.is_null() || (*(*(*s).curw).window).active.is_null() {
        return;
    }
    if (*(*(*(*s).curw).window).active).base.path.is_null() {
        path = b"\0" as *const u8 as *const ::core::ffi::c_char;
    } else {
        path = (*(*(*(*s).curw).window).active).base.path;
    }
    if (*c).path.is_null() || strcmp(path, (*c).path) != 0 as ::core::ffi::c_int {
        free((*c).path as *mut ::core::ffi::c_void);
        (*c).path = xstrdup(path);
        tty_set_path(&raw mut (*c).tty, (*c).path);
    }
}
unsafe extern "C" fn server_client_set_progress_bar(mut c: *mut client) {
    let mut s: *mut session = (*c).session;
    let mut pane_pb: *mut progress_bar = ::core::ptr::null_mut::<progress_bar>();
    if (*s).curw.is_null() || (*(*(*s).curw).window).active.is_null() {
        return;
    }
    pane_pb = &raw mut (*(*(*(*s).curw).window).active).base.progress_bar;
    if (*pane_pb).state as ::core::ffi::c_uint == (*c).progress_bar.state as ::core::ffi::c_uint
        && (*pane_pb).progress == (*c).progress_bar.progress
    {
        return;
    }
    memcpy(
        &raw mut (*c).progress_bar as *mut ::core::ffi::c_void,
        pane_pb as *const ::core::ffi::c_void,
        ::core::mem::size_of::<progress_bar>() as size_t,
    );
    tty_set_progress_bar(&raw mut (*c).tty, &raw mut (*c).progress_bar);
}
unsafe extern "C" fn server_client_dispatch(
    mut imsg: *mut imsg,
    mut arg: *mut ::core::ffi::c_void,
) {
    let mut current_block: u64;
    let mut c: *mut client = arg as *mut client;
    let mut datalen: ssize_t = 0;
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut old_sx: u_int = 0;
    let mut old_sy: u_int = 0;
    if (*c).flags & CLIENT_DEAD as uint64_t != 0 {
        return;
    }
    if imsg.is_null() {
        server_client_lost(c);
        return;
    }
    datalen = ((*imsg).hdr.len as usize).wrapping_sub(IMSG_HEADER_SIZE) as ssize_t;
    match (*imsg).hdr.type_0 {
        107 | 108 | 105 | 109 | 100 | 111 | 104 | 110 | 101 | 112 | 102 | 106 => {
            if server_client_dispatch_identify(c, imsg) != 0 as ::core::ffi::c_int {
                current_block = 13639960948656484833;
            } else {
                current_block = 14945149239039849694;
            }
        }
        200 => {
            if server_client_dispatch_command(c, imsg) != 0 as ::core::ffi::c_int {
                current_block = 13639960948656484833;
            } else {
                current_block = 14945149239039849694;
            }
        }
        208 => {
            if datalen != 0 as ssize_t {
                current_block = 13639960948656484833;
            } else if (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
                current_block = 14945149239039849694;
            } else {
                server_client_update_latest(c);
                old_sx = (*c).tty.sx;
                old_sy = (*c).tty.sy;
                tty_resize(&raw mut (*c).tty);
                tty_repeat_requests(&raw mut (*c).tty, 0 as ::core::ffi::c_int);
                recalculate_sizes();
                if (*c).overlay_resize.is_none() {
                    server_client_clear_overlay(c);
                } else {
                    (*c).overlay_resize.expect("non-null function pointer")(c, (*c).overlay_data);
                }
                server_redraw_client(c);
                if !(*c).session.is_null() {
                    server_client_fire_resized(c, old_sx, old_sy);
                }
                current_block = 14945149239039849694;
            }
        }
        205 => {
            if datalen != 0 as ssize_t {
                current_block = 13639960948656484833;
            } else {
                server_client_set_session(c, ::core::ptr::null_mut::<session>());
                recalculate_sizes();
                tty_close(&raw mut (*c).tty);
                proc_send(
                    (*c).peer,
                    MSG_EXITED,
                    -(1 as ::core::ffi::c_int),
                    ::core::ptr::null::<::core::ffi::c_void>(),
                    0 as size_t,
                );
                current_block = 14945149239039849694;
            }
        }
        216 | 215 => {
            if datalen != 0 as ssize_t {
                current_block = 13639960948656484833;
            } else if (*c).flags & CLIENT_SUSPENDED as uint64_t == 0 {
                current_block = 14945149239039849694;
            } else {
                (*c).flags &= !CLIENT_SUSPENDED as uint64_t;
                if (*c).fd == -(1 as ::core::ffi::c_int) || (*c).session.is_null() {
                    current_block = 14945149239039849694;
                } else {
                    s = (*c).session;
                    if gettimeofday(&raw mut (*c).activity_time, NULL) != 0 as ::core::ffi::c_int {
                        fatal(b"gettimeofday failed\0" as *const u8 as *const ::core::ffi::c_char);
                    }
                    tty_start_tty(&raw mut (*c).tty);
                    server_redraw_client(c);
                    recalculate_sizes();
                    if !s.is_null() {
                        session_update_activity(s, &raw mut (*c).activity_time);
                    }
                    current_block = 14945149239039849694;
                }
            }
        }
        209 => {
            if datalen != 0 as ssize_t {
                current_block = 13639960948656484833;
            } else if server_client_dispatch_shell(c) != 0 as ::core::ffi::c_int {
                current_block = 13639960948656484833;
            } else {
                current_block = 14945149239039849694;
            }
        }
        305 => {
            if file_write_ready(&raw mut (*c).files, imsg) != 0 as ::core::ffi::c_int {
                current_block = 13639960948656484833;
            } else {
                current_block = 14945149239039849694;
            }
        }
        308 => {
            if file_write_done(&raw mut (*c).files, imsg) != 0 as ::core::ffi::c_int {
                current_block = 13639960948656484833;
            } else {
                current_block = 14945149239039849694;
            }
        }
        301 => {
            if file_read_data(&raw mut (*c).files, imsg) != 0 as ::core::ffi::c_int {
                current_block = 13639960948656484833;
            } else {
                current_block = 14945149239039849694;
            }
        }
        302 => {
            if file_read_done(&raw mut (*c).files, imsg) != 0 as ::core::ffi::c_int {
                current_block = 13639960948656484833;
            } else {
                current_block = 14945149239039849694;
            }
        }
        _ => {
            current_block = 14945149239039849694;
        }
    }
    match current_block {
        14945149239039849694 => return,
        _ => {
            log_debug(
                b"client %p invalid message type %d\0" as *const u8 as *const ::core::ffi::c_char,
                c,
                (*imsg).hdr.type_0,
            );
            proc_kill_peer((*c).peer);
            return;
        }
    };
}
unsafe extern "C" fn server_client_read_only(
    mut item: *mut cmdq_item,
    mut data: *mut ::core::ffi::c_void,
) -> cmd_retval {
    cmdq_error(
        item,
        b"client is read-only\0" as *const u8 as *const ::core::ffi::c_char,
    );
    return CMD_RETURN_ERROR;
}
unsafe extern "C" fn server_client_default_command(
    mut item: *mut cmdq_item,
    mut data: *mut ::core::ffi::c_void,
) -> cmd_retval {
    let mut c: *mut client = cmdq_get_client(item);
    let mut cmdlist: *mut cmd_list = ::core::ptr::null_mut::<cmd_list>();
    let mut new_item: *mut cmdq_item = ::core::ptr::null_mut::<cmdq_item>();
    cmdlist = options_get_command(
        global_options,
        b"default-client-command\0" as *const u8 as *const ::core::ffi::c_char,
    );
    if (*c).flags & CLIENT_READONLY as uint64_t != 0
        && cmd_list_all_have(cmdlist, CMD_READONLY) == 0
    {
        new_item = cmdq_get_callback1(
            b"server_client_read_only\0" as *const u8 as *const ::core::ffi::c_char,
            Some(
                server_client_read_only
                    as unsafe extern "C" fn(*mut cmdq_item, *mut ::core::ffi::c_void) -> cmd_retval,
            ),
            ::core::ptr::null_mut::<::core::ffi::c_void>(),
        );
    } else {
        new_item = cmdq_get_command(cmdlist, ::core::ptr::null_mut::<cmdq_state>());
    }
    cmdq_insert_after(item, new_item);
    return CMD_RETURN_NORMAL;
}
unsafe extern "C" fn server_client_command_done(
    mut item: *mut cmdq_item,
    mut data: *mut ::core::ffi::c_void,
) -> cmd_retval {
    let mut c: *mut client = cmdq_get_client(item);
    if !(*c).flags & CLIENT_ATTACHED as uint64_t != 0 {
        (*c).flags |= CLIENT_EXIT as uint64_t;
    } else if !(*c).flags & CLIENT_EXIT as uint64_t != 0 {
        if (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
            control_ready(c);
        }
        tty_send_requests(&raw mut (*c).tty);
    }
    return CMD_RETURN_NORMAL;
}
unsafe extern "C" fn server_client_dispatch_command(
    mut c: *mut client,
    mut imsg: *mut imsg,
) -> ::core::ffi::c_int {
    let mut current_block: u64;
    let mut data: msg_command = msg_command { argc: 0 };
    let mut buf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut len: size_t = 0;
    let mut argc: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut argv: *mut *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    let mut cause: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut pr: *mut cmd_parse_result = ::core::ptr::null_mut::<cmd_parse_result>();
    let mut new_item: *mut cmdq_item = ::core::ptr::null_mut::<cmdq_item>();
    if (*c).flags & CLIENT_EXIT as uint64_t != 0 {
        return 0 as ::core::ffi::c_int;
    }
    if ((*imsg).hdr.len as usize).wrapping_sub(IMSG_HEADER_SIZE)
        < ::core::mem::size_of::<msg_command>() as usize
    {
        return -(1 as ::core::ffi::c_int);
    }
    memcpy(
        &raw mut data as *mut ::core::ffi::c_void,
        (*imsg).data,
        ::core::mem::size_of::<msg_command>() as size_t,
    );
    buf = ((*imsg).data as *mut ::core::ffi::c_char)
        .offset(::core::mem::size_of::<msg_command>() as usize as isize);
    len = ((*imsg).hdr.len as usize)
        .wrapping_sub(IMSG_HEADER_SIZE)
        .wrapping_sub(::core::mem::size_of::<msg_command>() as usize) as size_t;
    if len > 0 as size_t
        && *buf.offset(len.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int != '\0' as i32
    {
        return -(1 as ::core::ffi::c_int);
    }
    if cmd_unpack_argv(buf, len, data.argc, &raw mut argv) != 0 as ::core::ffi::c_int {
        cause = xstrdup(b"command too long\0" as *const u8 as *const ::core::ffi::c_char);
    } else {
        argc = data.argc;
        if argc == 0 as ::core::ffi::c_int {
            new_item = cmdq_get_callback1(
                b"server_client_default_command\0" as *const u8 as *const ::core::ffi::c_char,
                Some(
                    server_client_default_command
                        as unsafe extern "C" fn(
                            *mut cmdq_item,
                            *mut ::core::ffi::c_void,
                        ) -> cmd_retval,
                ),
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
            );
            current_block = 13472856163611868459;
        } else {
            let mut values = OwnedArgumentVector::from_argv(argc, argv);
            pr = cmd_parse_from_arguments(
                values.as_mut_ptr(),
                argc as u_int,
                ::core::ptr::null_mut::<cmd_parse_input>(),
            );
            drop(values);
            match (*pr).status as ::core::ffi::c_uint {
                0 => {
                    cause = (*pr).error;
                    current_block = 12680788052841528405;
                }
                1 | _ => {
                    cmd_free_argv(argc, argv);
                    if (*c).flags & CLIENT_READONLY as uint64_t != 0
                        && cmd_list_all_have((*pr).cmdlist, CMD_READONLY) == 0
                    {
                        new_item = cmdq_get_callback1(
                            b"server_client_read_only\0" as *const u8 as *const ::core::ffi::c_char,
                            Some(
                                server_client_read_only
                                    as unsafe extern "C" fn(
                                        *mut cmdq_item,
                                        *mut ::core::ffi::c_void,
                                    )
                                        -> cmd_retval,
                            ),
                            ::core::ptr::null_mut::<::core::ffi::c_void>(),
                        );
                    } else {
                        new_item =
                            cmdq_get_command((*pr).cmdlist, ::core::ptr::null_mut::<cmdq_state>());
                    }
                    cmd_list_free((*pr).cmdlist);
                    current_block = 13472856163611868459;
                }
            }
        }
        match current_block {
            12680788052841528405 => {}
            _ => {
                cmdq_append(c, new_item);
                cmdq_append(
                    c,
                    cmdq_get_callback1(
                        b"server_client_command_done\0" as *const u8 as *const ::core::ffi::c_char,
                        Some(
                            server_client_command_done
                                as unsafe extern "C" fn(
                                    *mut cmdq_item,
                                    *mut ::core::ffi::c_void,
                                )
                                    -> cmd_retval,
                        ),
                        ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ),
                );
                return 0 as ::core::ffi::c_int;
            }
        }
    }
    cmd_free_argv(argc, argv);
    cmdq_append(c, cmdq_get_error(cause));
    free(cause as *mut ::core::ffi::c_void);
    (*c).flags |= CLIENT_EXIT as uint64_t;
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn server_client_dispatch_identify(
    mut c: *mut client,
    mut imsg: *mut imsg,
) -> ::core::ffi::c_int {
    let mut data: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut home: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut datalen: size_t = 0;
    let mut flags: ::core::ffi::c_int = 0;
    let mut feat: ::core::ffi::c_int = 0;
    let mut longflags: uint64_t = 0;
    let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if (*c).flags & CLIENT_IDENTIFIED as uint64_t != 0 {
        return -(1 as ::core::ffi::c_int);
    }
    data = (*imsg).data as *const ::core::ffi::c_char;
    datalen = ((*imsg).hdr.len as usize).wrapping_sub(IMSG_HEADER_SIZE) as size_t;
    match (*imsg).hdr.type_0 {
        109 => {
            if datalen != ::core::mem::size_of::<::core::ffi::c_int>() as usize {
                return -(1 as ::core::ffi::c_int);
            }
            memcpy(
                &raw mut feat as *mut ::core::ffi::c_void,
                data as *const ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_int>() as size_t,
            );
            (*c).term_features |= feat;
            log_debug(
                b"client %p IDENTIFY_FEATURES %s\0" as *const u8 as *const ::core::ffi::c_char,
                c,
                tty_get_features(feat),
            );
        }
        100 => {
            if datalen != ::core::mem::size_of::<::core::ffi::c_int>() as usize {
                return -(1 as ::core::ffi::c_int);
            }
            memcpy(
                &raw mut flags as *mut ::core::ffi::c_void,
                data as *const ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_int>() as size_t,
            );
            (*c).flags |= flags as uint64_t;
            log_debug(
                b"client %p IDENTIFY_FLAGS %#x\0" as *const u8 as *const ::core::ffi::c_char,
                c,
                flags,
            );
        }
        111 => {
            if datalen != ::core::mem::size_of::<uint64_t>() as usize {
                return -(1 as ::core::ffi::c_int);
            }
            memcpy(
                &raw mut longflags as *mut ::core::ffi::c_void,
                data as *const ::core::ffi::c_void,
                ::core::mem::size_of::<uint64_t>() as size_t,
            );
            (*c).flags |= longflags;
            log_debug(
                b"client %p IDENTIFY_LONGFLAGS %#llx\0" as *const u8 as *const ::core::ffi::c_char,
                c,
                longflags as ::core::ffi::c_ulonglong,
            );
        }
        101 => {
            if datalen == 0 as size_t
                || *data.offset(datalen.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
                    != '\0' as i32
            {
                return -(1 as ::core::ffi::c_int);
            }
            (*c).term_name = xstrdup(data);
            log_debug(
                b"client %p IDENTIFY_TERM %s\0" as *const u8 as *const ::core::ffi::c_char,
                c,
                data,
            );
        }
        112 => {
            if datalen == 0 as size_t
                || *data.offset(datalen.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
                    != '\0' as i32
            {
                return -(1 as ::core::ffi::c_int);
            }
            server_client_add_term_cap(c, data);
            log_debug(
                b"client %p IDENTIFY_TERMINFO %s\0" as *const u8 as *const ::core::ffi::c_char,
                c,
                data,
            );
        }
        102 => {
            if datalen == 0 as size_t
                || *data.offset(datalen.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
                    != '\0' as i32
            {
                return -(1 as ::core::ffi::c_int);
            }
            (*c).ttyname = xstrdup(data);
            log_debug(
                b"client %p IDENTIFY_TTYNAME %s\0" as *const u8 as *const ::core::ffi::c_char,
                c,
                data,
            );
        }
        108 => {
            if datalen == 0 as size_t
                || *data.offset(datalen.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
                    != '\0' as i32
            {
                return -(1 as ::core::ffi::c_int);
            }
            if access(data, X_OK) == 0 as ::core::ffi::c_int {
                (*c).cwd = xstrdup(data);
            } else {
                home = find_home();
                if !home.is_null() {
                    (*c).cwd = xstrdup(home);
                } else {
                    (*c).cwd = xstrdup(b"/\0" as *const u8 as *const ::core::ffi::c_char);
                }
            }
            log_debug(
                b"client %p IDENTIFY_CWD %s\0" as *const u8 as *const ::core::ffi::c_char,
                c,
                data,
            );
        }
        104 => {
            if datalen != 0 as size_t {
                return -(1 as ::core::ffi::c_int);
            }
            (*c).fd = imsg_get_fd(imsg);
            log_debug(
                b"client %p IDENTIFY_STDIN %d\0" as *const u8 as *const ::core::ffi::c_char,
                c,
                (*c).fd,
            );
        }
        110 => {
            if datalen != 0 as size_t {
                return -(1 as ::core::ffi::c_int);
            }
            (*c).out_fd = imsg_get_fd(imsg);
            log_debug(
                b"client %p IDENTIFY_STDOUT %d\0" as *const u8 as *const ::core::ffi::c_char,
                c,
                (*c).out_fd,
            );
        }
        105 => {
            if datalen == 0 as size_t
                || *data.offset(datalen.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
                    != '\0' as i32
            {
                return -(1 as ::core::ffi::c_int);
            }
            if !strchr(data, '=' as i32).is_null() {
                environ_put((*c).environ, data, 0 as ::core::ffi::c_int);
            }
            log_debug(
                b"client %p IDENTIFY_ENVIRON %s\0" as *const u8 as *const ::core::ffi::c_char,
                c,
                data,
            );
        }
        107 => {
            if datalen != ::core::mem::size_of::<pid_t>() as usize {
                return -(1 as ::core::ffi::c_int);
            }
            memcpy(
                &raw mut (*c).pid as *mut ::core::ffi::c_void,
                data as *const ::core::ffi::c_void,
                ::core::mem::size_of::<pid_t>() as size_t,
            );
            log_debug(
                b"client %p IDENTIFY_CLIENTPID %ld\0" as *const u8 as *const ::core::ffi::c_char,
                c,
                (*c).pid as ::core::ffi::c_long,
            );
        }
        _ => {}
    }
    if (*imsg).hdr.type_0 != MSG_IDENTIFY_DONE as ::core::ffi::c_int as uint32_t {
        return 0 as ::core::ffi::c_int;
    }
    (*c).flags |= CLIENT_IDENTIFIED as uint64_t;
    if (*c).term_name.is_null() || *(*c).term_name as ::core::ffi::c_int == '\0' as i32 {
        free((*c).term_name as *mut ::core::ffi::c_void);
        (*c).term_name = xstrdup(b"unknown\0" as *const u8 as *const ::core::ffi::c_char);
    }
    if !(*c).ttyname.is_null() && *(*c).ttyname as ::core::ffi::c_int != '\0' as i32 {
        name = xstrdup((*c).ttyname);
    } else {
        xasprintf(
            &raw mut name,
            b"client-%ld\0" as *const u8 as *const ::core::ffi::c_char,
            (*c).pid as ::core::ffi::c_long,
        );
    }
    (*c).name = name;
    log_debug(
        b"client %p name is %s\0" as *const u8 as *const ::core::ffi::c_char,
        c,
        (*c).name,
    );
    if (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
        control_start(c);
    } else if (*c).fd != -(1 as ::core::ffi::c_int) {
        if tty_init(&raw mut (*c).tty, c) != 0 as ::core::ffi::c_int {
            close((*c).fd);
            (*c).fd = -(1 as ::core::ffi::c_int);
        } else {
            tty_resize(&raw mut (*c).tty);
            (*c).flags |= CLIENT_TERMINAL as uint64_t;
        }
        if (*c).out_fd != -(1 as ::core::ffi::c_int) {
            close((*c).out_fd);
        }
        (*c).out_fd = -(1 as ::core::ffi::c_int);
    }
    if (*c).flags & (CLIENT_CONTROL | CLIENT_TERMINAL) as uint64_t != 0 {
        events_fire_client(
            b"client-created\0" as *const u8 as *const ::core::ffi::c_char,
            c,
        );
    }
    if (*c).flags as ::core::ffi::c_ulonglong & (CLIENT_BRACKETPASTING | CLIENT_ASSUMEPASTING) != 0
        && current_time - (*c).paste_time > CLIENT_PASTE_TIME_LIMIT as time_t
    {
        log_debug(
            b"%s: paste time limit exceeded\0" as *const u8 as *const ::core::ffi::c_char,
            (*c).name,
        );
        (*c).flags = ((*c).flags as ::core::ffi::c_ulonglong
            & !(CLIENT_BRACKETPASTING | CLIENT_ASSUMEPASTING)) as uint64_t;
    }
    if !(*c).flags & CLIENT_EXIT as uint64_t != 0 && cfg_finished == 0 && c == clients.tqh_first {
        start_cfg();
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn server_client_dispatch_shell(mut c: *mut client) -> ::core::ffi::c_int {
    let mut shell: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    shell = options_get_string(
        global_s_options,
        b"default-shell\0" as *const u8 as *const ::core::ffi::c_char,
    );
    if checkshell(shell) == 0 {
        shell = _PATH_BSHELL.as_ptr();
    }
    proc_send(
        (*c).peer,
        MSG_SHELL,
        -(1 as ::core::ffi::c_int),
        shell as *const ::core::ffi::c_void,
        strlen(shell).wrapping_add(1 as size_t),
    );
    proc_kill_peer((*c).peer);
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn server_client_get_cwd(
    mut c: *mut client,
    mut s: *mut session,
) -> *const ::core::ffi::c_char {
    let mut home: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if cfg_finished == 0 && !cfg_client.is_null() {
        return (*cfg_client).cwd;
    }
    if !c.is_null() && (*c).session.is_null() && !(*c).cwd.is_null() {
        return (*c).cwd;
    }
    if !s.is_null() && !(*s).cwd.is_null() {
        return (*s).cwd;
    }
    if !c.is_null()
        && {
            s = (*c).session;
            !s.is_null()
        }
        && !(*s).cwd.is_null()
    {
        return (*s).cwd;
    }
    home = find_home();
    if !home.is_null() {
        return home;
    }
    return b"/\0" as *const u8 as *const ::core::ffi::c_char;
}
unsafe extern "C" fn server_client_control_flags(
    mut c: *mut client,
    mut next: *const ::core::ffi::c_char,
) -> uint64_t {
    if strcmp(
        next,
        b"pause-after\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        (*c).pause_age = 0 as u_int;
        return 0x100000000 as uint64_t;
    }
    if sscanf(
        next,
        b"pause-after=%u\0" as *const u8 as *const ::core::ffi::c_char,
        &raw mut (*c).pause_age,
    ) == 1 as ::core::ffi::c_int
    {
        (*c).pause_age = (*c).pause_age.wrapping_mul(1000 as u_int);
        return 0x100000000 as uint64_t;
    }
    if strcmp(
        next,
        b"no-output\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        return 0x4000000 as uint64_t;
    }
    if strcmp(
        next,
        b"wait-exit\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        return 0x200000000 as uint64_t;
    }
    if strcmp(
        next,
        b"new-layouts\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        return 0x800000000 as uint64_t;
    }
    return 0 as uint64_t;
}
#[no_mangle]
pub unsafe extern "C" fn server_client_set_flags(
    mut c: *mut client,
    mut flags: *const ::core::ffi::c_char,
) {
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut next: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut flag: uint64_t = 0;
    let mut not: ::core::ffi::c_int = 0;
    let mut copy = CStr::from_ptr(flags).to_bytes_with_nul().to_vec();
    s = copy.as_mut_ptr().cast();
    loop {
        next = strsep(
            &raw mut s,
            b",\0" as *const u8 as *const ::core::ffi::c_char,
        );
        if next.is_null() {
            break;
        }
        not = (*next as ::core::ffi::c_int == '!' as i32) as ::core::ffi::c_int;
        if not != 0 {
            next = next.offset(1);
        }
        if (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
            flag = server_client_control_flags(c, next);
        } else {
            flag = 0 as uint64_t;
        }
        if strcmp(
            next,
            b"read-only\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            flag = CLIENT_READONLY as uint64_t;
        } else if strcmp(
            next,
            b"ignore-size\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            flag = CLIENT_IGNORESIZE as uint64_t;
        } else if strcmp(
            next,
            b"no-detach-on-destroy\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            flag = CLIENT_NO_DETACH_ON_DESTROY as uint64_t;
        }
        if flag == 0 as uint64_t {
            continue;
        }
        log_debug(
            b"client %s set flag %s\0" as *const u8 as *const ::core::ffi::c_char,
            (*c).name,
            next,
        );
        if not != 0 {
            if (*c).flags & CLIENT_READONLY as uint64_t != 0 {
                flag &= !CLIENT_READONLY as uint64_t;
            }
            (*c).flags &= !flag;
        } else {
            (*c).flags |= flag;
        }
        if flag == CLIENT_CONTROL_NOOUTPUT as uint64_t {
            control_reset_offsets(c);
        }
    }
    drop(copy);
    proc_send(
        (*c).peer,
        MSG_FLAGS,
        -(1 as ::core::ffi::c_int),
        &raw mut (*c).flags as *const ::core::ffi::c_void,
        ::core::mem::size_of::<uint64_t>() as size_t,
    );
}
#[no_mangle]
pub unsafe extern "C" fn server_client_get_flags(mut c: *mut client) -> *const ::core::ffi::c_char {
    static mut s: [::core::ffi::c_char; 256] = [0; 256];
    let mut tmp: [::core::ffi::c_char; 32] = [0; 32];
    *(&raw mut s as *mut ::core::ffi::c_char) = '\0' as i32 as ::core::ffi::c_char;
    if (*c).flags & CLIENT_ATTACHED as uint64_t != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"attached,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if (*c).flags & CLIENT_FOCUSED as uint64_t != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"focused,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"control-mode,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if (*c).flags & CLIENT_IGNORESIZE as uint64_t != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"ignore-size,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if (*c).flags as ::core::ffi::c_ulonglong & CLIENT_NO_DETACH_ON_DESTROY != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"no-detach-on-destroy,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if (*c).flags & CLIENT_CONTROL_NOOUTPUT as uint64_t != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"no-output,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if (*c).flags as ::core::ffi::c_ulonglong & CLIENT_CONTROL_WAITEXIT != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"wait-exit,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if (*c).flags as ::core::ffi::c_ulonglong & CLIENT_CONTROL_NEWLAYOUTS != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"new-layouts,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if (*c).flags as ::core::ffi::c_ulonglong & CLIENT_CONTROL_PAUSEAFTER != 0 {
        xsnprintf(
            &raw mut tmp as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 32]>() as size_t,
            b"pause-after=%u,\0" as *const u8 as *const ::core::ffi::c_char,
            (*c).pause_age.wrapping_div(1000 as u_int),
        );
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            &raw mut tmp as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if (*c).flags & CLIENT_READONLY as uint64_t != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"read-only,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if (*c).flags & CLIENT_SUSPENDED as uint64_t != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"suspended,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if (*c).flags & CLIENT_UTF8 as uint64_t != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"UTF-8,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if *(&raw mut s as *mut ::core::ffi::c_char) as ::core::ffi::c_int != '\0' as i32 {
        s[strlen(&raw mut s as *mut ::core::ffi::c_char).wrapping_sub(1 as size_t) as usize] =
            '\0' as i32 as ::core::ffi::c_char;
    }
    return &raw mut s as *mut ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn server_client_remove_pane(mut wp: *mut window_pane) {
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    c = clients.tqh_first;
    while !c.is_null() {
        if (*c).tty.mouse_last_pane == (*wp).id as ::core::ffi::c_int {
            (*c).tty.mouse_last_pane = -(1 as ::core::ffi::c_int);
            (*c).tty.mouse_drag_update = None;
            (*c).tty.mouse_scrolling_flag = 0 as ::core::ffi::c_int;
        }
        c = (*c).entry.tqe_next;
    }
}
#[no_mangle]
pub unsafe extern "C" fn server_client_print(
    mut c: *mut client,
    mut parse: ::core::ffi::c_int,
    mut evb: *mut evbuffer,
) {
    let mut data: *mut ::core::ffi::c_void =
        evbuffer_pullup(evb, -(1 as ::core::ffi::c_int) as ssize_t) as *mut ::core::ffi::c_void;
    let mut size: size_t = evbuffer_get_length(evb);
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
    let mut sanitized: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut msg: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut line: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut empty: ::core::ffi::c_char = '\0' as i32 as ::core::ffi::c_char;
    if parse == 0 {
        utf8_stravisx(
            &raw mut msg,
            data as *const ::core::ffi::c_char,
            size,
            VIS_OCTAL | VIS_CSTYLE | VIS_NOSLASH,
        );
    } else if size == 0 as size_t {
        msg = &raw mut empty;
    } else {
        msg =
            evbuffer_pullup(evb, -(1 as ::core::ffi::c_int) as ssize_t) as *mut ::core::ffi::c_char;
        if *msg.offset(size.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int != '\0' as i32
        {
            evbuffer_add(
                evb,
                b"\0" as *const u8 as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
                1 as size_t,
            );
        }
    }
    log_debug(
        b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"server_client_print\0" as *const u8 as *const ::core::ffi::c_char,
        msg,
    );
    if !c.is_null() {
        if (*c).session.is_null() || (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
            if !(*c).flags & CLIENT_UTF8 as uint64_t != 0 {
                sanitized = utf8_sanitize(msg);
                if (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
                    control_write(
                        c,
                        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                        sanitized,
                    );
                } else {
                    file_print(
                        c,
                        b"%s\n\0" as *const u8 as *const ::core::ffi::c_char,
                        sanitized,
                    );
                }
                free(sanitized as *mut ::core::ffi::c_void);
            } else if (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
                control_write(c, b"%s\0" as *const u8 as *const ::core::ffi::c_char, msg);
            } else {
                file_print(c, b"%s\n\0" as *const u8 as *const ::core::ffi::c_char, msg);
            }
        } else {
            wp = (*(*(*(*c).session).curw).window).active;
            wme = (*wp).modes.tqh_first;
            if wme.is_null() || (*wme).mode != &raw const window_view_mode {
                window_pane_set_mode(
                    wp,
                    ::core::ptr::null_mut::<window_pane>(),
                    &raw const window_view_mode,
                    ::core::ptr::null_mut::<cmdq_item>(),
                    ::core::ptr::null_mut::<cmd_find_state>(),
                    ::core::ptr::null_mut::<args>(),
                );
            }
            if parse != 0 {
                loop {
                    line = evbuffer_readln(evb, ::core::ptr::null_mut::<size_t>(), EVBUFFER_EOL_LF);
                    if !line.is_null() {
                        window_copy_add(
                            wp,
                            1 as ::core::ffi::c_int,
                            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                            line,
                        );
                        free(line as *mut ::core::ffi::c_void);
                    }
                    if line.is_null() {
                        break;
                    }
                }
                size = evbuffer_get_length(evb);
                if size != 0 as size_t {
                    line = evbuffer_pullup(evb, -(1 as ::core::ffi::c_int) as ssize_t)
                        as *mut ::core::ffi::c_char;
                    window_copy_add(
                        wp,
                        1 as ::core::ffi::c_int,
                        b"%.*s\0" as *const u8 as *const ::core::ffi::c_char,
                        size as ::core::ffi::c_int,
                        line,
                    );
                }
            } else {
                window_copy_add(
                    wp,
                    0 as ::core::ffi::c_int,
                    b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                    msg,
                );
            }
        }
    }
    if parse == 0 {
        free(msg as *mut ::core::ffi::c_void);
    }
}
unsafe extern "C" fn server_client_report_theme(mut c: *mut client, mut theme: client_theme) {
    let mut old: client_theme = (*c).theme;
    if theme as ::core::ffi::c_uint == THEME_LIGHT as ::core::ffi::c_int as ::core::ffi::c_uint {
        (*c).theme = THEME_LIGHT;
        events_fire_client(
            b"client-light-theme\0" as *const u8 as *const ::core::ffi::c_char,
            c,
        );
    } else {
        (*c).theme = THEME_DARK;
        events_fire_client(
            b"client-dark-theme\0" as *const u8 as *const ::core::ffi::c_char,
            c,
        );
    }
    if (*c).theme as ::core::ffi::c_uint != old as ::core::ffi::c_uint {
        server_client_update_theme_colours(c);
        if (*c).tty.flags & TTY_OPENED != 0 {
            tty_invalidate(&raw mut (*c).tty);
        }
        server_redraw_client(c);
    }
    tty_repeat_requests(&raw mut (*c).tty, 1 as ::core::ffi::c_int);
}
