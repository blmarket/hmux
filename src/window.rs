use crate::src::options::options_owner_ptr;
use crate::src::alerts::alerts_queue;
use crate::src::arguments::args_has;
use crate::src::cmd::cmd_mouse_at;
use crate::src::cmd::find::{cmd_find_from_pane, cmd_find_from_window};
use crate::src::cmd::queue::{cmdq_continue, cmdq_get_client};
use crate::src::compat::strtonum::strtonum;
use crate::src::control::control_write_output;
use crate::src::events::{events_fire, events_fire_pane, events_fire_window};
use crate::src::events_payload::{
    event_payload_create, event_payload_set_int, event_payload_set_pane, event_payload_set_string,
    event_payload_set_target, event_payload_set_uint, event_payload_set_window,
};
use crate::src::ffi::libc::{
    __ctype_b_loc, close, fnmatch, gethostname, getpid, gettimeofday, ioctl, kill, memcpy, memset,
    strcasecmp,
};
use crate::src::ffi::regex::RegexStorage;
use crate::src::ffi::utempter::utempter_remove_record;
use crate::src::file::{file_cancel, file_read_with_cmdq_wait_init};
use crate::src::format::bytes::write_cstr;
use crate::src::grid::grid_cells_look_equal;
use crate::src::grid::view::grid_view_string_cells_bytes;
use crate::src::input::{input_free, input_init, input_parse_buffer, input_parse_pane};
use crate::src::input_keys::input_key_pane;
use crate::src::layout::{
    layout_assign_pane, layout_fix_panes, layout_floating_pane, layout_free, layout_init,
};
use crate::src::log::{fatal, fatalx, log_cstr, log_cstr_n, log_debug};
use crate::src::menu::{menu_destroy, menu_resize};
use crate::src::options::{options_create, options_free, options_get_number};
use crate::src::prompt::{
    prompt_closed, prompt_create, prompt_free, prompt_incremental_start, prompt_key, prompt_mouse,
    prompt_set_options, prompt_type_string, prompt_update,
};
use crate::src::reactor::{
    bufferevent_disable, bufferevent_enable, bufferevent_free, bufferevent_new, bufferevent_write,
    evbuffer_drain, evbuffer_get_length, evbuffer_pullup, event_add, event_del, event_initialized,
    event_set,
};
use crate::src::screen::{
    screen_free, screen_init, screen_resize, screen_set_default_cursor, screen_set_title,
};
use crate::src::screen_redraw::redraw_invalidate_scene;
use crate::src::screen_write::{screen_write_clear_dirty, screen_write_stop_sync};
use crate::src::server::clients;
use crate::src::server::{marked_pane, server_check_marked, server_clear_marked};
use crate::src::server_client::server_client_unref_owned;
use crate::src::server_fn::{
    server_destroy_pane, server_kill_pane, server_redraw_window, server_redraw_window_borders,
    server_status_session, server_status_window,
};
use crate::src::session::session_has;
use crate::src::shared::events::event_payload;
use crate::src::shared::pane::{window_pane_tree, WindowPanePromptOwner};
use crate::src::shared::prompt::prompt_create_data;
use crate::src::spawn::spawn_editor_finish;
use crate::src::status::status_at_line;
use crate::src::style::colour::{
    colour_palette_free, colour_palette_from_option, colour_palette_get, colour_palette_init,
    colour_totheme,
};
use crate::src::style::{
    style_ranges_free, style_ranges_get_range, style_ranges_init,
    style_set_scrollbar_style_from_option,
};
use crate::src::tmux::{clean_name_cstring, global_options, global_w_options, setblocking};
use crate::src::tty::{tty_default_colours, tty_update_window_offset};
use crate::src::window_copy::{window_copy_mode, window_view_mode};
use std::cell::{RefCell, UnsafeCell};
use std::ffi::{CStr, CString};
use std::ptr::NonNull;
use std::rc::{Rc, Weak};

use crate::src::shared::abi::ssize_t;
use crate::src::shared::abi::*;
use crate::src::shared::arguments::args;
use crate::src::shared::client::{client, client_file, client_file_cb};
use crate::src::shared::client::{
    CLIENT_CONTROL, CLIENT_DEAD, CLIENT_EXIT, CLIENT_EXITED, CLIENT_FOCUSED, CLIENT_UNATTACHEDFLAGS,
};
use crate::src::shared::colour::*;
use crate::src::shared::command::{cmd_find_state, cmdq_item};
use crate::src::shared::ctype::_ISspace;
use crate::src::shared::display::*;
use crate::src::shared::event::*;
use crate::src::shared::event::{EV_READ, EV_WRITE};
use crate::src::shared::grid::*;
use crate::src::shared::input::input_ctx;
use crate::src::shared::key::*;
use crate::src::shared::layout::layout_cell;
use crate::src::shared::layout::layout_geometry;
use crate::src::shared::layout::*;
use crate::src::shared::limits::{INT_MAX, UINT_MAX};
use crate::src::shared::mouse::{mouse_event, MOUSE_BUTTON_1, MOUSE_MASK_BUTTONS, MOUSE_MASK_DRAG};
use crate::src::shared::options::options;
use crate::src::shared::pane::{
    window_pane, window_pane_history, window_pane_modes, window_pane_prompt,
    window_pane_tree_entry, window_panes,
};
use crate::src::shared::pane::{
    window_pane_offset, window_pane_resize, window_pane_resizes, PANE_CHANGED, PANE_DESTROYED,
    PANE_EMPTY, PANE_EXITED, PANE_FLOATOVERZOOM, PANE_FOCUSED, PANE_INPUTOFF, PANE_REDRAW,
    PANE_REDRAWSCROLLBAR, PANE_SCROLLBARS_ALWAYS, PANE_SCROLLBARS_AUTOHIDE, PANE_SCROLLBARS_LEFT,
    PANE_SCROLLBARS_MODAL, PANE_STATUSREADY, PANE_STATUS_BOTTOM, PANE_STATUS_BOTTOM_FLOATING,
    PANE_STATUS_OFF, PANE_STATUS_TOP, PANE_STATUS_TOP_FLOATING, PANE_STYLECHANGED,
    PANE_THEMECHANGED, PANE_UNSEENCHANGES, PANE_VISITED, PANE_ZOOMED,
};
use crate::src::shared::posix_io::FNM_CASEFOLD;
use crate::src::shared::posix_terminal::{winsize, TIOCSWINSZ};
use crate::src::shared::prompt::prompt;
use crate::src::shared::prompt::*;
use crate::src::shared::prompt::{prompt_free_cb, prompt_input_cb, prompt_result, PROMPT_CLOSE};
use libc::{REG_EXTENDED, REG_ICASE};
use crate::src::shared::screen::{screen, MODE_BRACKETPASTE, MODE_FOCUSON, MODE_THEME_UPDATES};
use crate::src::shared::session::session;
use crate::src::shared::signal::SIGCHLD;
use crate::src::shared::spawn::spawn_editor_state;
use crate::src::shared::spawn::{SPAWN_BEFORE, SPAWN_FLOATING, SPAWN_FULLSIZE};
use crate::src::shared::status::status_prompt_input_cb;
use crate::src::shared::style::*;
use crate::src::shared::tree::{RB_INF, RB_NEGINF};
pub use crate::src::shared::window::{
    window, window_entry, window_mode, window_mode_entry, window_winlinks, windows, winlink,
    winlink_entry, winlink_stack, winlinks,
};
use crate::src::shared::window::{
    WINDOW_ACTIVITY, WINDOW_ALERTFLAGS, WINDOW_MODE_HIDE_PANE_STATUS, WINDOW_MODE_HIDE_SCROLLBARS,
    WINDOW_MODE_NO_STACK, WINDOW_PANE_NO_MODE, WINDOW_ZOOMED, WINLINK_ACTIVITY, WINLINK_ALERTFLAGS,
    WINLINK_BELL, WINLINK_SILENCE, WINLINK_VISITED,
};

struct window_pane_input_data {
    item: NonNull<cmdq_item>,
    client: Option<Rc<UnsafeCell<client>>>,
    wp: u_int,
    file: Weak<UnsafeCell<client_file>>,
}

impl window_pane_input_data {
    fn into_callback(self: Box<Self>) -> client_file_cb {
        Some(Box::new(move |event| unsafe {
            window_pane_input_callback(
                &self,
                event.error,
                event.closed,
                event.buffer.expect("read callback buffer"),
            );
        }))
    }
}

impl Drop for window_pane_input_data {
    fn drop(&mut self) {
        if let Some(client) = self.client.take() {
            server_client_unref_owned(client);
        }
    }
}

pub const FIONREAD: ::core::ffi::c_int = 0x541b as ::core::ffi::c_int;

pub const DEFAULT_XPIXEL: ::core::ffi::c_int = 16 as ::core::ffi::c_int;
pub const DEFAULT_YPIXEL: ::core::ffi::c_int = 32 as ::core::ffi::c_int;
pub const WINDOW_WASZOOMED: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub static mut windows: windows = windows { storage: None };
pub static mut all_window_panes: window_pane_tree = window_pane_tree { storage: None };
static mut next_window_pane_id: u_int = 0;
static mut next_window_id: u_int = 0;
static mut next_active_point: u_int = 0;
pub fn windows_find(head: &windows, elm: &window) -> *mut window {
    let Some(owner) = head.storage.as_ref() else {
        return std::ptr::null_mut();
    };
    let map = owner
        .try_borrow_mut()
        .expect("window index already borrowed");
    let key = elm.id;
    map.get(&key).copied().unwrap_or(std::ptr::null_mut())
}
pub unsafe fn windows_insert(head: *mut windows, elm: *mut window) -> *mut window {
    let key = (*elm).id;
    let owner = (*head).storage.get_or_insert_with(refbox::RefBox::default);
    let observer = owner.downgrade();
    let mut map = owner
        .try_borrow_mut()
        .expect("window index already borrowed");
    match map.entry(key) {
        std::collections::btree_map::Entry::Occupied(entry) => return *entry.get(),
        std::collections::btree_map::Entry::Vacant(entry) => {
            entry.insert(elm);
            (*elm).entry.owner = Some(observer);
        }
    }
    std::ptr::null_mut()
}
pub unsafe fn windows_remove(head: *mut windows, elm: *mut window) -> *mut window {
    if elm.is_null() {
        return std::ptr::null_mut();
    }
    let key = (*elm).id;
    let Some(owner) = (*head).storage.as_ref() else {
        return std::ptr::null_mut();
    };
    let empty = {
        let mut map = owner
            .try_borrow_mut()
            .expect("window index already borrowed");
        if map.get(&key).copied() != Some(elm) {
            return std::ptr::null_mut();
        }
        map.remove(&key);
        map.is_empty()
    };
    (*elm).entry.owner = None;
    if empty {
        (*head).storage = None;
    }
    elm
}
pub fn windows_minmax(head: &windows) -> *mut window {
    let Some(owner) = head.storage.as_ref() else {
        return std::ptr::null_mut();
    };
    let map = owner
        .try_borrow_mut()
        .expect("window index already borrowed");
    let pair = map.first_key_value();
    pair.map_or(std::ptr::null_mut(), |(_, node)| *node)
}
pub unsafe fn windows_next(elm: &window) -> *mut window {
    let Some(owner) = elm.entry.owner.as_ref() else {
        return std::ptr::null_mut();
    };
    let map = match owner.try_borrow_mut() {
        Ok(map) => map,
        Err(refbox::BorrowError::Dropped) => return std::ptr::null_mut(),
        Err(refbox::BorrowError::Borrowed) => panic!("window index already borrowed"),
    };
    let key = elm.id;
    map.range((std::ops::Bound::Excluded(&key), std::ops::Bound::Unbounded))
        .next()
        .map_or(std::ptr::null_mut(), |(_, node)| *node)
}

pub fn winlinks_find(head: &winlinks, elm: &winlink) -> *mut winlink {
    let Some(owner) = head.storage.as_ref() else {
        return std::ptr::null_mut();
    };
    let map = owner
        .try_borrow_mut()
        .expect("winlink index already borrowed");
    let key = elm.idx;
    map.get(&key)
        .map_or(std::ptr::null_mut(), |owner| owner.as_ptr() as *mut winlink)
}
pub fn winlinks_nfind(head: &winlinks, elm: &winlink) -> *mut winlink {
    let Some(owner) = head.storage.as_ref() else {
        return std::ptr::null_mut();
    };
    let map = owner
        .try_borrow_mut()
        .expect("winlink index already borrowed");
    let key = elm.idx;
    map.range((std::ops::Bound::Included(&key), std::ops::Bound::Unbounded))
        .next()
        .map_or(std::ptr::null_mut(), |(_, owner)| {
            owner.as_ptr() as *mut winlink
        })
}
pub fn winlinks_minmax(head: &winlinks, direction: ::core::ffi::c_int) -> *mut winlink {
    let Some(owner) = head.storage.as_ref() else {
        return std::ptr::null_mut();
    };
    let map = owner
        .try_borrow_mut()
        .expect("winlink index already borrowed");
    let pair = if direction < 0 {
        map.first_key_value()
    } else {
        map.last_key_value()
    };
    pair.map_or(std::ptr::null_mut(), |(_, owner)| {
        owner.as_ptr() as *mut winlink
    })
}
pub unsafe fn winlinks_next(elm: &winlink) -> *mut winlink {
    let Some(owner) = elm.entry.owner.as_ref() else {
        return std::ptr::null_mut();
    };
    let map = match owner.try_borrow_mut() {
        Ok(map) => map,
        Err(refbox::BorrowError::Dropped) => return std::ptr::null_mut(),
        Err(refbox::BorrowError::Borrowed) => panic!("winlink index already borrowed"),
    };
    let key = elm.idx;
    map.range((std::ops::Bound::Excluded(&key), std::ops::Bound::Unbounded))
        .next()
        .map_or(std::ptr::null_mut(), |(_, owner)| {
            owner.as_ptr() as *mut winlink
        })
}
pub unsafe fn winlinks_prev(elm: &winlink) -> *mut winlink {
    let Some(owner) = elm.entry.owner.as_ref() else {
        return std::ptr::null_mut();
    };
    let map = match owner.try_borrow_mut() {
        Ok(map) => map,
        Err(refbox::BorrowError::Dropped) => return std::ptr::null_mut(),
        Err(refbox::BorrowError::Borrowed) => panic!("winlink index already borrowed"),
    };
    let key = elm.idx;
    map.range((std::ops::Bound::Unbounded, std::ops::Bound::Excluded(&key)))
        .next_back()
        .map_or(std::ptr::null_mut(), |(_, owner)| {
            owner.as_ptr() as *mut winlink
        })
}

/// Return the first winlink in a window's association order.
pub unsafe fn window_winlinks_first(w: *mut window) -> *mut winlink {
    if w.is_null() {
        return std::ptr::null_mut();
    }
    (*w).winlinks
        .storage
        .as_deref()
        .and_then(|links| links.ordered.first().copied())
        .unwrap_or(std::ptr::null_mut())
}

/// Return the next winlink in `w` after `wl`, or null when it is no longer in
/// that window. Taking the owner explicitly keeps iteration correct if a
/// callback moves `wl` to another window while the original list is traversed.
pub unsafe fn window_winlinks_next(w: *mut window, wl: *mut winlink) -> *mut winlink {
    if w.is_null() || wl.is_null() {
        return std::ptr::null_mut();
    }
    let Some(links) = (*w).winlinks.storage.as_deref() else {
        return std::ptr::null_mut();
    };
    links
        .positions
        .get(&wl)
        .and_then(|&position| links.ordered.get(position + 1).copied())
        .unwrap_or(std::ptr::null_mut())
}

/// Append a non-owning winlink handle to the window's association order.
pub unsafe fn window_winlinks_append(w: *mut window, wl: *mut winlink) {
    assert!(!w.is_null() && !wl.is_null());
    let links = (*w).winlinks.storage.get_or_insert_with(|| Box::default());
    assert!(
        !links.positions.contains_key(&wl),
        "winlink is already present in this window"
    );
    let position = links.ordered.len();
    links.ordered.push(wl);
    links.positions.insert(wl, position);
}

/// Remove a non-owning winlink handle from its window's association order.
pub unsafe fn window_winlinks_remove(w: *mut window, wl: *mut winlink) {
    assert!(!w.is_null() && !wl.is_null());
    let links = (*w)
        .winlinks
        .storage
        .as_mut()
        .expect("window winlink collection must be alive");
    let position = links
        .positions
        .remove(&wl)
        .expect("winlink must belong to its window");
    links.ordered.remove(position);
    for (position, link) in links.ordered.iter().enumerate().skip(position) {
        links.positions.insert(*link, position);
    }
    if links.ordered.is_empty() {
        (*w).winlinks.storage = None;
    }
}

pub fn window_pane_tree_find(head: &window_pane_tree, elm: &window_pane) -> *mut window_pane {
    let Some(owner) = head.storage.as_ref() else {
        return std::ptr::null_mut();
    };
    let map = owner.try_borrow_mut().expect("pane index already borrowed");
    let key = elm.id;
    map.get(&key).copied().unwrap_or(std::ptr::null_mut())
}
pub unsafe fn window_pane_tree_insert(
    head: *mut window_pane_tree,
    elm: *mut window_pane,
) -> *mut window_pane {
    let key = (*elm).id;
    let owner = (*head).storage.get_or_insert_with(refbox::RefBox::default);
    let observer = owner.downgrade();
    let mut map = owner.try_borrow_mut().expect("pane index already borrowed");
    match map.entry(key) {
        std::collections::btree_map::Entry::Occupied(entry) => return *entry.get(),
        std::collections::btree_map::Entry::Vacant(entry) => {
            entry.insert(elm);
            (*elm).tree_entry.owner = Some(observer);
        }
    }
    std::ptr::null_mut()
}
pub unsafe fn window_pane_tree_remove(
    head: *mut window_pane_tree,
    elm: *mut window_pane,
) -> *mut window_pane {
    if elm.is_null() {
        return std::ptr::null_mut();
    }
    let key = (*elm).id;
    let Some(owner) = (*head).storage.as_ref() else {
        return std::ptr::null_mut();
    };
    let empty = {
        let mut map = owner.try_borrow_mut().expect("pane index already borrowed");
        if map.get(&key).copied() != Some(elm) {
            return std::ptr::null_mut();
        }
        map.remove(&key);
        map.is_empty()
    };
    (*elm).tree_entry.owner = None;
    if empty {
        (*head).storage = None;
    }
    elm
}
pub fn window_pane_tree_minmax(head: &window_pane_tree) -> *mut window_pane {
    let Some(owner) = head.storage.as_ref() else {
        return std::ptr::null_mut();
    };
    let map = owner.try_borrow_mut().expect("pane index already borrowed");
    let pair = map.first_key_value();
    pair.map_or(std::ptr::null_mut(), |(_, node)| *node)
}
pub unsafe fn window_pane_tree_next(elm: &window_pane) -> *mut window_pane {
    let Some(owner) = elm.tree_entry.owner.as_ref() else {
        return std::ptr::null_mut();
    };
    let map = match owner.try_borrow_mut() {
        Ok(map) => map,
        Err(refbox::BorrowError::Dropped) => return std::ptr::null_mut(),
        Err(refbox::BorrowError::Borrowed) => panic!("pane index already borrowed"),
    };
    let key = elm.id;
    map.range((std::ops::Bound::Excluded(&key), std::ops::Bound::Unbounded))
        .next()
        .map_or(std::ptr::null_mut(), |(_, node)| *node)
}
unsafe fn window_fire_renamed(mut w: *mut window, mut old_name: *const ::core::ffi::c_char) {
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null::<cmd_find_state>() as *mut cmd_find_state,
        s: ::core::ptr::null::<session>() as *mut session,
        wl: ::core::ptr::null::<winlink>() as *mut winlink,
        w: ::core::ptr::null::<window>() as *mut window,
        wp: ::core::ptr::null::<window_pane>() as *mut window_pane,
        idx: 0,
    };
    let mut ep = event_payload_create();
    cmd_find_from_window(&raw mut fs, w, 0 as ::core::ffi::c_int);
    event_payload_set_target(&mut *ep, &fs);
    event_payload_set_window(
        &mut *ep,
        b"window\0" as *const u8 as *const ::core::ffi::c_char,
        w,
    );
    event_payload_set_string(
        &mut *ep,
        b"old_name\0" as *const u8 as *const ::core::ffi::c_char,
        |out| write_cstr(out, old_name),
    );
    event_payload_set_string(
        &mut *ep,
        b"new_name\0" as *const u8 as *const ::core::ffi::c_char,
        |out| write_cstr(out, (*w).name.as_ptr().cast_mut()),
    );
    events_fire(
        b"window-renamed\0" as *const u8 as *const ::core::ffi::c_char,
        ep,
    );
}
unsafe fn window_fire_pane_changed(
    mut w: *mut window,
    mut wp: *mut window_pane,
    mut lastwp: *mut window_pane,
) {
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null::<cmd_find_state>() as *mut cmd_find_state,
        s: ::core::ptr::null::<session>() as *mut session,
        wl: ::core::ptr::null::<winlink>() as *mut winlink,
        w: ::core::ptr::null::<window>() as *mut window,
        wp: ::core::ptr::null::<window_pane>() as *mut window_pane,
        idx: 0,
    };
    let mut ep = event_payload_create();
    cmd_find_from_pane(&raw mut fs, wp, 0 as ::core::ffi::c_int);
    event_payload_set_target(&mut *ep, &fs);
    event_payload_set_window(
        &mut *ep,
        b"window\0" as *const u8 as *const ::core::ffi::c_char,
        w,
    );
    event_payload_set_pane(
        &mut *ep,
        b"pane\0" as *const u8 as *const ::core::ffi::c_char,
        wp,
    );
    event_payload_set_pane(
        &mut *ep,
        b"new_pane\0" as *const u8 as *const ::core::ffi::c_char,
        wp,
    );
    if !lastwp.is_null() {
        event_payload_set_pane(
            &mut *ep,
            b"old_pane\0" as *const u8 as *const ::core::ffi::c_char,
            lastwp,
        );
    }
    events_fire(
        b"window-pane-changed\0" as *const u8 as *const ::core::ffi::c_char,
        ep,
    );
}
pub unsafe fn window_fire_pane_moved(
    mut wp: *mut window_pane,
    mut old_w: *mut window,
    mut old_idx: ::core::ffi::c_int,
    mut new_w: *mut window,
    mut new_idx: ::core::ffi::c_int,
) {
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null::<cmd_find_state>() as *mut cmd_find_state,
        s: ::core::ptr::null::<session>() as *mut session,
        wl: ::core::ptr::null::<winlink>() as *mut winlink,
        w: ::core::ptr::null::<window>() as *mut window,
        wp: ::core::ptr::null::<window_pane>() as *mut window_pane,
        idx: 0,
    };
    let mut ep = event_payload_create();
    cmd_find_from_pane(&raw mut fs, wp, 0 as ::core::ffi::c_int);
    event_payload_set_target(&mut *ep, &fs);
    event_payload_set_pane(
        &mut *ep,
        b"pane\0" as *const u8 as *const ::core::ffi::c_char,
        wp,
    );
    event_payload_set_window(
        &mut *ep,
        b"window\0" as *const u8 as *const ::core::ffi::c_char,
        new_w,
    );
    event_payload_set_window(
        &mut *ep,
        b"old_window\0" as *const u8 as *const ::core::ffi::c_char,
        old_w,
    );
    event_payload_set_window(
        &mut *ep,
        b"new_window\0" as *const u8 as *const ::core::ffi::c_char,
        new_w,
    );
    if old_idx != -(1 as ::core::ffi::c_int) {
        event_payload_set_int(
            &mut *ep,
            b"old_window_index\0" as *const u8 as *const ::core::ffi::c_char,
            old_idx,
        );
    }
    if new_idx != -(1 as ::core::ffi::c_int) {
        event_payload_set_int(
            &mut *ep,
            b"window_index\0" as *const u8 as *const ::core::ffi::c_char,
            new_idx,
        );
        event_payload_set_int(
            &mut *ep,
            b"new_window_index\0" as *const u8 as *const ::core::ffi::c_char,
            new_idx,
        );
    }
    events_fire(
        b"pane-moved\0" as *const u8 as *const ::core::ffi::c_char,
        ep,
    );
}
unsafe fn window_fire_pane_mode_changed(
    mut name: *const ::core::ffi::c_char,
    mut wp: *mut window_pane,
    mut previous: *const ::core::ffi::c_char,
    mut current: *const ::core::ffi::c_char,
    mut entered: ::core::ffi::c_int,
) {
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null::<cmd_find_state>() as *mut cmd_find_state,
        s: ::core::ptr::null::<session>() as *mut session,
        wl: ::core::ptr::null::<winlink>() as *mut winlink,
        w: ::core::ptr::null::<window>() as *mut window,
        wp: ::core::ptr::null::<window_pane>() as *mut window_pane,
        idx: 0,
    };
    let mut ep = event_payload_create();
    cmd_find_from_pane(&raw mut fs, wp, 0 as ::core::ffi::c_int);
    event_payload_set_target(&mut *ep, &fs);
    event_payload_set_pane(
        &mut *ep,
        b"pane\0" as *const u8 as *const ::core::ffi::c_char,
        wp,
    );
    event_payload_set_window(
        &mut *ep,
        b"window\0" as *const u8 as *const ::core::ffi::c_char,
        (*wp).window as *mut window,
    );
    if !current.is_null() {
        event_payload_set_string(
            &mut *ep,
            b"current_mode\0" as *const u8 as *const ::core::ffi::c_char,
            |out| write_cstr(out, current),
        );
    }
    if !previous.is_null() {
        event_payload_set_string(
            &mut *ep,
            b"previous_mode\0" as *const u8 as *const ::core::ffi::c_char,
            |out| write_cstr(out, previous),
        );
    }
    event_payload_set_int(
        &mut *ep,
        b"mode_entered\0" as *const u8 as *const ::core::ffi::c_char,
        entered,
    );
    events_fire(name, ep);
}
unsafe fn window_fire_pane_prompt(
    mut name: *const ::core::ffi::c_char,
    mut wp: *mut window_pane,
    mut type_0: prompt_type,
) {
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null::<cmd_find_state>() as *mut cmd_find_state,
        s: ::core::ptr::null::<session>() as *mut session,
        wl: ::core::ptr::null::<winlink>() as *mut winlink,
        w: ::core::ptr::null::<window>() as *mut window,
        wp: ::core::ptr::null::<window_pane>() as *mut window_pane,
        idx: 0,
    };
    let type_string = prompt_type_string(type_0);
    let mut ep = event_payload_create();
    cmd_find_from_pane(&raw mut fs, wp, 0 as ::core::ffi::c_int);
    event_payload_set_target(&mut *ep, &fs);
    event_payload_set_pane(
        &mut *ep,
        b"pane\0" as *const u8 as *const ::core::ffi::c_char,
        wp,
    );
    event_payload_set_window(
        &mut *ep,
        b"window\0" as *const u8 as *const ::core::ffi::c_char,
        (*wp).window as *mut window,
    );
    event_payload_set_string(
        &mut *ep,
        b"prompt_type\0" as *const u8 as *const ::core::ffi::c_char,
        |out| out.write_all(type_string.to_bytes()),
    );
    events_fire(name, ep);
}
pub unsafe fn winlink_find_by_window(mut wwl: *mut winlinks, mut w: *mut window) -> *mut winlink {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    wl = winlinks_minmax(&*wwl, RB_NEGINF);
    while !wl.is_null() {
        if (*wl).window_ptr() == w {
            return wl;
        }
        wl = winlinks_next(&*wl);
    }
    return ::core::ptr::null_mut::<winlink>();
}
pub unsafe fn winlink_find_by_index(
    mut wwl: *mut winlinks,
    mut idx: ::core::ffi::c_int,
) -> *mut winlink {
    let mut wl: winlink = winlink {
        idx: 0,
        session: ::core::ptr::null_mut::<session>(),
        window_owner: None,
        flags: 0,
        entry: winlink_entry { owner: None },
    };
    if idx < 0 as ::core::ffi::c_int {
        fatalx(|out| out.write_all(b"bad index"));
    }
    wl.idx = idx;
    return winlinks_find(&*wwl, &wl);
}
pub unsafe fn winlink_find_by_window_id(mut wwl: *mut winlinks, mut id: u_int) -> *mut winlink {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    wl = winlinks_minmax(&*wwl, RB_NEGINF);
    while !wl.is_null() {
        if (*(*wl).window_ptr()).id == id {
            return wl;
        }
        wl = winlinks_next(&*wl);
    }
    return ::core::ptr::null_mut::<winlink>();
}
unsafe fn winlink_next_index(
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
pub unsafe fn winlink_count(mut wwl: *mut winlinks) -> u_int {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut n: u_int = 0;
    n = 0 as u_int;
    wl = winlinks_minmax(&*wwl, RB_NEGINF);
    while !wl.is_null() {
        n = n.wrapping_add(1);
        wl = winlinks_next(&*wl);
    }
    return n;
}
pub unsafe fn winlink_add(mut wwl: *mut winlinks, mut idx: ::core::ffi::c_int) -> *mut winlink {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    if idx < 0 as ::core::ffi::c_int {
        idx = winlink_next_index(wwl, -idx - 1 as ::core::ffi::c_int);
        if idx == -(1 as ::core::ffi::c_int) {
            return ::core::ptr::null_mut::<winlink>();
        }
    } else if !winlink_find_by_index(wwl, idx).is_null() {
        return ::core::ptr::null_mut::<winlink>();
    }
    let owner = refbox::RefBox::new(winlink {
        idx,
        session: std::ptr::null_mut(),
        window_owner: None,
        flags: 0,
        entry: winlink_entry { owner: None },
    });
    wl = owner.as_ptr() as *mut winlink;
    let storage = (*wwl).storage.get_or_insert_with(refbox::RefBox::default);
    let observer = storage.downgrade();
    let mut map = storage
        .try_borrow_mut()
        .expect("winlink index already borrowed");
    match map.entry(idx) {
        std::collections::btree_map::Entry::Vacant(entry) => {
            entry.insert(owner);
        }
        std::collections::btree_map::Entry::Occupied(_) => {
            unreachable!("winlink index was checked above")
        }
    }
    (*wl).entry.owner = Some(observer);
    return wl;
}
// Keep the field published through the close notification. Its callback can
// inspect the winlink or retain the window. Detach only after that notification,
// then drop the consumed Rc without firing the notification twice.
unsafe fn winlink_release_window(wl: *mut winlink, from: &CStr) {
    let w = (*wl).window_ptr();
    window_before_release(w, from.as_ptr());
    let mut owner = (*wl).window_owner.take().expect("winlink window owner");
    drop(owner.0.take());
}

pub unsafe fn winlink_set_window(wl: *mut winlink, w: *mut window) {
    if !(*wl).window_ptr().is_null() {
        window_winlinks_remove((*wl).window_ptr(), wl);
        winlink_release_window(wl, c"winlink_set_window");
    }
    (*wl).window_owner = Some(crate::src::shared::window::WindowOwner::retain(
        w, c"winlink_set_window",
    ));
    window_winlinks_append(w, wl);
}
pub unsafe fn winlink_remove(mut wwl: *mut winlinks, mut wl: *mut winlink) {
    if !(*wl).session.is_null() {
        winlink_stack_remove(&raw mut (*(*wl).session).lastw, wl);
    }
    let mut w: *mut window = (*wl).window_ptr();
    if !w.is_null() {
        window_winlinks_remove(w, wl);
        winlink_release_window(wl, c"winlink_remove");
    }
    // Window teardown above may reenter; borrow the owning map only afterward.
    let idx = (*wl).idx;
    let (owner, empty) = {
        let storage = (*wwl)
            .storage
            .as_ref()
            .expect("winlink index must be alive");
        let mut map = storage
            .try_borrow_mut()
            .expect("winlink index already borrowed");
        assert_eq!(
            map.get(&idx).map(|owner| owner.as_ptr()),
            Some(wl as *const winlink),
            "removed winlink must belong to this index"
        );
        let owner = map.remove(&idx).expect("winlink must have an owner");
        (*wl).entry.owner = None;
        (owner, map.is_empty())
    };
    if empty {
        (*wwl).storage = None;
    }
    drop(owner);
}
pub unsafe fn winlink_next(mut wl: *mut winlink) -> *mut winlink {
    return winlinks_next(&*wl);
}
pub unsafe fn winlink_previous(mut wl: *mut winlink) -> *mut winlink {
    return winlinks_prev(&*wl);
}
pub unsafe fn winlink_next_by_number(
    mut wl: *mut winlink,
    mut s: *mut session,
    mut n: ::core::ffi::c_int,
) -> *mut winlink {
    while n > 0 as ::core::ffi::c_int {
        wl = winlinks_next(&*wl);
        if wl.is_null() {
            wl = winlinks_minmax(&(*s).windows, RB_NEGINF);
        }
        n -= 1;
    }
    return wl;
}
pub unsafe fn winlink_previous_by_number(
    mut wl: *mut winlink,
    mut s: *mut session,
    mut n: ::core::ffi::c_int,
) -> *mut winlink {
    while n > 0 as ::core::ffi::c_int {
        wl = winlinks_prev(&*wl);
        if wl.is_null() {
            wl = winlinks_minmax(&(*s).windows, RB_INF);
        }
        n -= 1;
    }
    return wl;
}
/// Borrow the owner through the existing window index. The raw pointer remains
/// a compatibility view; neither it nor its address is a separate ownership key.
unsafe fn winlink_weak(wl: *mut winlink) -> refbox::Weak<winlink> {
    let owner = (*wl)
        .entry
        .owner
        .as_ref()
        .expect("visited winlink index must be alive");
    let map = owner
        .try_borrow_mut()
        .expect("winlink index already borrowed");
    let owner = map
        .get(&(*wl).idx)
        .expect("visited winlink must have an owner");
    assert_eq!(
        owner.as_ptr(),
        wl as *const winlink,
        "visited winlink must belong to its index"
    );
    owner.downgrade()
}

/// Move the owner between keys without invalidating the winlink or its observers.
/// The caller must supply a live member of `head` and an unused destination index.
pub unsafe fn winlinks_reindex(head: *mut winlinks, wl: *mut winlink, idx: i32) {
    let owner = (*head)
        .storage
        .as_ref()
        .expect("winlink index must be alive");
    let mut map = owner
        .try_borrow_mut()
        .expect("winlink index already borrowed");
    let old_idx = (*wl).idx;
    assert_eq!(
        map.get(&old_idx).map(|owner| owner.as_ptr()),
        Some(wl as *const winlink),
        "reindexed winlink must belong to this index"
    );
    if old_idx == idx {
        return;
    }
    assert!(
        !map.contains_key(&idx),
        "destination winlink index must be vacant"
    );
    let owner = map.remove(&old_idx).expect("winlink must have an owner");
    owner
        .try_access_mut(|link| link.idx = idx)
        .expect("reindexed winlink is already borrowed");
    map.insert(idx, owner);
}
pub unsafe fn winlink_stack_push(stack: *mut winlink_stack, wl: *mut winlink) {
    if wl.is_null() {
        return;
    }
    winlink_stack_remove(stack, wl);
    if (*stack).storage.is_none() {
        (*stack).storage = Some(Box::default());
    }
    let weak = winlink_weak(wl);
    (*stack)
        .storage
        .as_mut()
        .expect("visit history was just initialized")
        .push_front(weak);
    (*wl).flags |= WINLINK_VISITED;
}
pub unsafe fn winlink_stack_remove(stack: *mut winlink_stack, wl: *mut winlink) {
    if wl.is_null() {
        return;
    }
    if let Some(storage) = (*stack).storage.as_mut() {
        storage.retain(|link| checked_winlink_ptr(link) != wl);
    }
    (*wl).flags &= !WINLINK_VISITED;
}

/// Append while rebuilding a session's saved visit order.
pub unsafe fn winlink_stack_append(stack: &mut winlink_stack, wl: *mut winlink) {
    if stack.storage.is_none() {
        stack.storage = Some(Box::default());
    }
    let weak = winlink_weak(wl);
    stack
        .storage
        .as_mut()
        .expect("visit history was just initialized")
        .push_back(weak);
    (*wl).flags |= WINLINK_VISITED;
}

pub fn winlink_stack_clear(stack: &mut winlink_stack) {
    stack.storage = None;
}

pub fn winlink_stack_indices(stack: &winlink_stack) -> Vec<::core::ffi::c_int> {
    let Some(storage) = stack.storage.as_ref() else {
        return Vec::new();
    };
    storage
        .iter()
        .filter_map(|link| match link.try_access_mut(|node| node.idx) {
            Ok(idx) => Some(idx),
            Err(refbox::BorrowError::Dropped) => {
                panic!("visited winlink owner was dropped before observer teardown")
            }
            Err(refbox::BorrowError::Borrowed) => panic!("visited winlink is already borrowed"),
        })
        .collect()
}

fn checked_winlink_ptr(link: &refbox::Weak<winlink>) -> *mut winlink {
    match link.try_access_mut(|node| node as *mut winlink) {
        Ok(ptr) => ptr,
        Err(refbox::BorrowError::Dropped) => {
            panic!("visited winlink owner was dropped before observer teardown")
        }
        Err(refbox::BorrowError::Borrowed) => panic!("visited winlink is already borrowed"),
    }
}

pub fn winlink_stack_first(stack: &winlink_stack) -> *mut winlink {
    let Some(storage) = stack.storage.as_ref() else {
        return std::ptr::null_mut();
    };
    storage
        .iter()
        .map(checked_winlink_ptr)
        .next()
        .unwrap_or(std::ptr::null_mut())
}

pub fn winlink_stack_next(stack: &winlink_stack, wl: *mut winlink) -> *mut winlink {
    if wl.is_null() || stack.storage.is_none() {
        return std::ptr::null_mut();
    }
    let queue = stack.storage.as_ref().expect("checked above");
    let Some(position) = queue
        .iter()
        .position(|link| checked_winlink_ptr(link) == wl)
    else {
        return std::ptr::null_mut();
    };
    queue
        .iter()
        .skip(position + 1)
        .map(checked_winlink_ptr)
        .next()
        .unwrap_or(std::ptr::null_mut())
}
pub unsafe fn window_find_by_id_str(mut s: *const ::core::ffi::c_char) -> *mut window {
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
pub unsafe fn window_find_by_id(id: u_int) -> *mut window {
    let mut w = window::default();
    w.id = id;
    return windows_find(&*std::ptr::addr_of!(windows), &w);
}
pub unsafe fn window_update_activity(mut w: *mut window) {
    gettimeofday(&raw mut (*w).activity_time, NULL);
    alerts_queue(w, WINDOW_ACTIVITY);
}

pub(crate) unsafe fn window_replace_old_layout(
    w: *mut window,
    layout: Option<CString>,
) -> Option<CString> {
    ::core::mem::replace(&mut (*w).old_layout, layout)
}

/// Replace the window's owned name, returning the previous value for callers
/// that need to keep it alive across synchronous callbacks.
pub(crate) unsafe fn window_replace_name(w: *mut window, name: CString) -> CString {
    ::core::mem::replace(&mut (*w).name, name)
}
/// Return one owned reference; callers must release it after linking the window.
pub unsafe fn window_create(
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
    w = crate::src::shared::rc::new(window::default());
    (*w).flags = 0 as ::core::ffi::c_int;
    (*w).panes = window_panes::default();
    (*w).z_index = window_panes::default();
    (*w).last_panes = window_pane_history::default();
    (*w).active = ::core::ptr::null_mut::<window_pane>();
    (*w).lastlayout = -(1 as ::core::ffi::c_int);
    (*w).layout_root = None;
    (*w).sx = sx;
    (*w).sy = sy;
    (*w).manual_sx = sx;
    (*w).manual_sy = sy;
    (*w).xpixel = xpixel;
    (*w).ypixel = ypixel;
    (*w).options = Some(crate::src::options::options_create_owned(global_w_options));
    (*w).sb = options_get_number(
        options_owner_ptr(&mut (*w).options),
        b"pane-scrollbars\0" as *const u8 as *const ::core::ffi::c_char,
    ) as ::core::ffi::c_int;
    (*w).sb_pos = options_get_number(
        options_owner_ptr(&mut (*w).options),
        b"pane-scrollbars-position\0" as *const u8 as *const ::core::ffi::c_char,
    ) as ::core::ffi::c_int;
    (*w).winlinks.storage = None;
    (*w).entry.owner = None;
    let fresh0 = next_window_id;
    next_window_id = next_window_id.wrapping_add(1);
    (*w).id = fresh0;
    windows_insert(&raw mut windows, w);
    if gettimeofday(&raw mut (*w).creation_time, NULL) != 0 as ::core::ffi::c_int {
        fatal(|out| out.write_all(b"gettimeofday failed"));
    }
    window_update_activity(w);
    log_debug(format_args!(
        "{}: @{} create {}x{} ({}x{})",
        "window_create",
        ((*w).id) as u32,
        (sx) as u32,
        (sy) as u32,
        ((*w).xpixel) as u32,
        ((*w).ypixel) as u32
    ));
    return w;
}
unsafe fn window_destroy(mut w: *mut window) {
    log_debug(format_args!("window @{} destroyed", ((*w).id) as u32));
    // The final Rc owner is already being dropped. Restore the layout links,
    // but do not resize dying panes: their events would retain this window.
    window_unzoom_internal(w, 0, false);
    if (*w).entry.owner.is_some() {
        windows_remove(&raw mut windows, w);
    }
    drop((*w).layout_root.take());
    drop((*w).saved_layout_root.take());
    drop(window_replace_old_layout(w, None));
    menu_destroy((*w).menu.take());
    window_destroy_panes(w);
    if event_initialized(&(*w).name_event) != 0 {
        event_del(&raw mut (*w).name_event);
    }
    if event_initialized(&(*w).alerts_timer) != 0 {
        event_del(&raw mut (*w).alerts_timer);
    }
    if event_initialized(&(*w).offset_timer) != 0 {
        event_del(&raw mut (*w).offset_timer);
    }
    drop((*w).options.take());
}
pub unsafe fn window_pane_destroy_ready(mut wp: *mut window_pane) -> ::core::ffi::c_int {
    let mut n: ::core::ffi::c_int = 0;
    if (*wp).pipe_fd != -(1 as ::core::ffi::c_int)
        && evbuffer_get_length(&*((*(*wp).pipe_event).output)) != 0 as size_t
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
    if (*wp).editor.is_some() && !(*wp).flags & PANE_STATUSREADY != 0 {
        return 0 as ::core::ffi::c_int;
    }
    return 1 as ::core::ffi::c_int;
}
pub unsafe fn window_add_ref(w: *mut window, from: *const ::core::ffi::c_char) {
    crate::src::shared::rc::retain(w);
    log_debug(format_args!(
        "retain window @{} ({})",
        ((*w).id) as u32,
        log_cstr((from) as *const _)
    ));
}
unsafe fn window_before_release(w: *mut window, from: *const ::core::ffi::c_char) {
    // Notify while a strong reference still exists: callbacks may retain w.
    if crate::src::shared::rc::strong_count(w) == 1 {
        events_fire_window(c"window-closed".as_ptr(), w);
    }
    log_debug(format_args!(
        "release window @{} ({})",
        ((*w).id) as u32,
        log_cstr((from) as *const _)
    ));
}
pub unsafe fn window_remove_ref(w: *mut window, from: *const ::core::ffi::c_char) {
    window_before_release(w, from);
    crate::src::shared::rc::release(w);
}
pub unsafe fn window_pane_add_ref(wp: *mut window_pane, from: *const ::core::ffi::c_char) {
    crate::src::shared::rc::retain(wp);
    log_debug(format_args!(
        "retain pane %{} ({})",
        ((*wp).id) as u32,
        log_cstr((from) as *const _)
    ));
}
pub unsafe fn window_pane_remove_ref(wp: *mut window_pane, from: *const ::core::ffi::c_char) {
    log_debug(format_args!(
        "release pane %{} ({})",
        ((*wp).id) as u32,
        log_cstr((from) as *const _)
    ));
    crate::src::shared::rc::release(wp);
}
pub unsafe fn window_set_name(
    mut w: *mut window,
    mut new_name: *const ::core::ffi::c_char,
    mut untrusted: ::core::ffi::c_int,
) {
    if let Some(name) = clean_name_cstring(CStr::from_ptr(new_name), untrusted) {
        // Keep the previous owner alive across synchronous rename callbacks.
        let last = window_replace_name(w, name);
        window_fire_renamed(w, last.as_ptr());
    }
}
pub unsafe fn window_resize(
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
    log_debug(format_args!(
        "{}: @{} resize {}x{} ({}x{})",
        "window_resize",
        ((*w).id) as u32,
        (sx) as u32,
        (sy) as u32,
        (if xpixel == -(1 as ::core::ffi::c_int) {
            (*w).xpixel
        } else {
            xpixel as u_int
        }) as u32,
        (if ypixel == -(1 as ::core::ffi::c_int) {
            (*w).ypixel
        } else {
            ypixel as u_int
        }) as u32
    ));
    (*w).sx = sx;
    (*w).sy = sy;
    if let Some(menu) = (*w).menu.as_ref().map(|menu| menu.downgrade()) {
        menu_resize(
            &mut menu.try_borrow_mut().expect("live unborrowed menu"),
            sx,
            sy,
        );
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
pub unsafe fn window_pane_send_resize(mut wp: *mut window_pane, mut sx: u_int, mut sy: u_int) {
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
    log_debug(format_args!(
        "{}: %{} resize to {},{}",
        "window_pane_send_resize",
        ((*wp).id) as u32,
        (sx) as u32,
        (sy) as u32
    ));
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
        fatal(|out| out.write_all(b"ioctl failed"));
    }
}
pub unsafe fn window_has_pane(mut w: *mut window, mut wp: *mut window_pane) -> ::core::ffi::c_int {
    let mut wp1: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    wp1 = window_pane_first(w);
    while !wp1.is_null() {
        if wp1 == wp {
            return 1 as ::core::ffi::c_int;
        }
        wp1 = window_pane_next(wp1);
    }
    return 0 as ::core::ffi::c_int;
}
pub unsafe fn window_pane_contains(
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
pub unsafe fn window_update_focus(mut w: *mut window) {
    if !w.is_null() {
        log_debug(format_args!(
            "{}: @{}",
            "window_update_focus",
            ((*w).id) as u32
        ));
        window_pane_update_focus((*w).active);
    }
}
pub unsafe fn window_pane_update_focus(mut wp: *mut window_pane) {
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut focused: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if !wp.is_null() && !(*wp).flags & PANE_EXITED != 0 {
        if wp != (*(*wp).window).active {
            focused = 0 as ::core::ffi::c_int;
        } else {
            c = clients.first();
            while !c.is_null() {
                if !(*c).session.is_null()
                    && (*(*c).session).attached != 0 as u_int
                    && (*c).flags & CLIENT_FOCUSED as uint64_t != 0
                    && (*(*(*c).session).curw).window_ptr() == (*wp).window
                    && (*c).overlay_draw.is_none()
                    && (*(*wp).window).menu.is_none()
                {
                    focused = 1 as ::core::ffi::c_int;
                    break;
                } else {
                    c = clients.next(c);
                }
            }
        }
        if focused == 0 && (*wp).flags & PANE_FOCUSED != 0 {
            log_debug(format_args!(
                "{}: %{} focus out",
                "window_pane_update_focus",
                ((*wp).id) as u32
            ));
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
            log_debug(format_args!(
                "{}: %{} focus in",
                "window_pane_update_focus",
                ((*wp).id) as u32
            ));
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
            log_debug(format_args!(
                "{}: %{} focus unchanged",
                "window_pane_update_focus",
                ((*wp).id) as u32
            ));
        }
    }
}
pub unsafe fn window_set_active_pane(
    mut w: *mut window,
    mut wp: *mut window_pane,
    mut notify: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut lastwp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    log_debug(format_args!(
        "{}: pane %{}",
        "window_set_active_pane",
        ((*wp).id) as u32
    ));
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
unsafe fn window_pane_get_palette(
    mut wp: *mut window_pane,
    mut c: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if wp.is_null() {
        return -(1 as ::core::ffi::c_int);
    }
    return colour_palette_get(Some(&(*wp).palette), c);
}
pub unsafe fn window_redraw_active_switch(mut w: *mut window, mut wp: *mut window_pane) {
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
        if !grid_cells_look_equal(&*gc1, &*gc2) {
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
            window_pane_z_remove(w, wp);
            window_pane_z_insert_front(w, wp);
            (*wp).flags |= PANE_REDRAW;
            redraw_invalidate_scene(w);
        }
        wp = (*w).active;
        if wp.is_null() {
            break;
        }
    }
}
pub unsafe fn window_get_active_at(
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
        wp = window_pane_z_first(w);
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
            wp = window_pane_z_next(wp);
        }
    }
    let mut current_block_15: u64;
    wp = window_pane_z_first(w);
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
        wp = window_pane_z_next(wp);
    }
    return ::core::ptr::null_mut::<window_pane>();
}
pub unsafe fn window_find_string(
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
pub unsafe fn window_zoom(mut wp: *mut window_pane) -> ::core::ffi::c_int {
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
    wp1 = window_pane_first(w);
    while !wp1.is_null() {
        (*wp1).saved_layout_cell = (*wp1).layout_cell as *mut layout_cell;
        (*wp1).layout_cell = ::core::ptr::null_mut::<layout_cell>();
        wp1 = window_pane_next(wp1);
    }
    (*w).saved_layout_root = (*w).layout_root.take();
    layout_init(w, wp);
    wp1 = window_pane_first(w);
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
        wp1 = window_pane_next(wp1);
    }
    if (*(*wp).saved_layout_cell).flags & LAYOUT_CELL_FLOATING != 0 {
        window_pane_z_remove(w, wp);
        window_pane_z_insert_back(w, wp);
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
pub unsafe fn window_unzoom(w: *mut window, notify: ::core::ffi::c_int) -> ::core::ffi::c_int {
    window_unzoom_internal(w, notify, true)
}

unsafe fn window_unzoom_internal(
    w: *mut window,
    notify: ::core::ffi::c_int,
    resize_panes: bool,
) -> ::core::ffi::c_int {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut zoomed: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut slc: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    if !(*w).flags & WINDOW_ZOOMED != 0 {
        return -(1 as ::core::ffi::c_int);
    }
    wp = window_pane_first(w);
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
        wp = window_pane_next(wp);
    }
    (*w).flags &= !WINDOW_ZOOMED;
    layout_free(w);
    (*w).layout_root = (*w).saved_layout_root.take();
    wp = window_pane_first(w);
    while !wp.is_null() {
        (*wp).layout_cell = (*wp).saved_layout_cell as *mut layout_cell;
        (*wp).saved_layout_cell = ::core::ptr::null_mut::<layout_cell>();
        (*wp).flags &= !PANE_ZOOMED;
        wp = window_pane_next(wp);
    }
    if !zoomed.is_null() && window_pane_is_floating(zoomed) != 0 {
        window_pane_z_remove(w, zoomed);
        if zoomed == (*w).active {
            window_pane_z_insert_front(w, zoomed);
        } else {
            wp = window_pane_z_first(w);
            while !wp.is_null() {
                if window_pane_is_floating(wp) == 0 {
                    break;
                }
                wp = window_pane_z_next(wp);
            }
            if wp.is_null() {
                window_pane_z_insert_back(w, zoomed);
            } else {
                window_pane_z_insert_before(w, wp, zoomed);
            }
        }
    }
    if resize_panes {
        layout_fix_panes(w, ::core::ptr::null_mut::<window_pane>());
    }
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
pub unsafe fn window_zoomed_pane(mut w: *mut window) -> *mut window_pane {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    if !(*w).flags & WINDOW_ZOOMED != 0 {
        return ::core::ptr::null_mut::<window_pane>();
    }
    wp = window_pane_z_last(w);
    while !wp.is_null() {
        if !(*wp).layout_cell.is_null() && window_pane_is_floating(wp) == 0 {
            return wp;
        }
        wp = window_pane_z_previous(wp);
    }
    return ::core::ptr::null_mut::<window_pane>();
}
pub unsafe fn window_active_pane_is_over_zoom(mut w: *mut window) -> ::core::ffi::c_int {
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
pub unsafe fn window_push_zoom(
    mut w: *mut window,
    mut always: ::core::ffi::c_int,
    mut flag: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut wp: *mut window_pane = window_zoomed_pane(w);
    log_debug(format_args!(
        "{}: @{} {}",
        "window_push_zoom",
        ((*w).id) as u32,
        (flag != 0 && (*w).flags & WINDOW_ZOOMED != 0) as ::core::ffi::c_int
    ));
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
pub unsafe fn window_pop_zoom(mut w: *mut window) -> ::core::ffi::c_int {
    let mut wp: *mut window_pane = (*w).was_zoomed;
    log_debug(format_args!(
        "{}: @{} {}",
        "window_pop_zoom",
        ((*w).id) as u32,
        ((*w).flags & WINDOW_WASZOOMED != 0) as ::core::ffi::c_int
    ));
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
pub unsafe fn window_add_pane(
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
    if window_pane_first(w).is_null() {
        log_debug(format_args!(
            "{}: @{} at start",
            "window_add_pane",
            ((*w).id) as u32
        ));
        window_pane_list_insert_front(w, wp);
    } else if flags & SPAWN_BEFORE != 0 {
        log_debug(format_args!(
            "{}: @{} before %{}",
            "window_add_pane",
            ((*w).id) as u32,
            ((*wp).id) as u32
        ));
        if flags & SPAWN_FULLSIZE != 0 {
            window_pane_list_insert_front(w, wp);
        } else {
            window_pane_list_insert_before(w, other, wp);
        }
    } else {
        log_debug(format_args!(
            "{}: @{} after %{}",
            "window_add_pane",
            ((*w).id) as u32,
            ((*wp).id) as u32
        ));
        if flags & (SPAWN_FULLSIZE | SPAWN_FLOATING) != 0 {
            window_pane_list_insert_back(w, wp);
        } else {
            window_pane_list_insert_after(w, other, wp);
        }
    }
    if flags & SPAWN_FLOATING == 0 {
        window_pane_z_insert_back(w, wp);
    } else if !(*w).modal.is_null() {
        window_pane_z_insert_after(w, (*w).modal, wp);
    } else {
        window_pane_z_insert_front(w, wp);
    }
    redraw_invalidate_scene(w);
    return wp;
}
pub unsafe fn window_lost_pane(mut w: *mut window, mut wp: *mut window_pane) {
    let mut lastwp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    log_debug(format_args!(
        "{}: @{} pane %{}",
        "window_lost_pane",
        ((*w).id) as u32,
        ((*wp).id) as u32
    ));
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
            (*w).active = window_pane_stack_first(w);
        }
        if (*w).active.is_null() {
            (*w).active = window_pane_previous(wp);
            if (*w).active.is_null() {
                (*w).active = window_pane_next(wp);
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
pub unsafe fn window_remove_pane(mut w: *mut window, mut wp: *mut window_pane) {
    window_lost_pane(w, wp);
    window_pane_list_remove(w, wp);
    window_pane_z_remove(w, wp);
    redraw_invalidate_scene(w);
    window_pane_destroy(wp);
}
pub unsafe fn window_pane_at_index(mut w: *mut window, mut idx: u_int) -> *mut window_pane {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut n: u_int = 0;
    n = options_get_number(
        options_owner_ptr(&mut (*w).options),
        b"pane-base-index\0" as *const u8 as *const ::core::ffi::c_char,
    ) as u_int;
    wp = window_pane_first(w);
    while !wp.is_null() {
        if n == idx {
            return wp;
        }
        n = n.wrapping_add(1);
        wp = window_pane_next(wp);
    }
    return ::core::ptr::null_mut::<window_pane>();
}
pub unsafe fn window_pane_next_by_number(
    mut w: *mut window,
    mut wp: *mut window_pane,
    mut n: u_int,
) -> *mut window_pane {
    while n > 0 as u_int {
        wp = window_pane_next(wp);
        if wp.is_null() {
            wp = window_pane_first(w);
        }
        n = n.wrapping_sub(1);
    }
    return wp;
}
pub unsafe fn window_pane_previous_by_number(
    mut w: *mut window,
    mut wp: *mut window_pane,
    mut n: u_int,
) -> *mut window_pane {
    while n > 0 as u_int {
        wp = window_pane_previous(wp);
        if wp.is_null() {
            wp = window_pane_last(w);
        }
        n = n.wrapping_sub(1);
    }
    return wp;
}
pub unsafe fn window_pane_index(mut wp: *mut window_pane, mut i: *mut u_int) -> ::core::ffi::c_int {
    let mut w: *mut window = (*wp).window as *mut window;
    let mut wq: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    *i = options_get_number(
        options_owner_ptr(&mut (*w).options),
        b"pane-base-index\0" as *const u8 as *const ::core::ffi::c_char,
    ) as u_int;
    wq = window_pane_first(w);
    while !wq.is_null() {
        if wp == wq {
            return 0 as ::core::ffi::c_int;
        }
        *i = (*i).wrapping_add(1);
        wq = window_pane_next(wq);
    }
    return -(1 as ::core::ffi::c_int);
}
pub unsafe fn window_pane_zindex(
    mut wp: *mut window_pane,
    mut i: *mut u_int,
) -> ::core::ffi::c_int {
    let mut w: *mut window = (*wp).window as *mut window;
    let mut wq: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    *i = 0 as u_int;
    wq = window_pane_z_first(w);
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
        wq = window_pane_z_next(wq);
    }
    return -(1 as ::core::ffi::c_int);
}
pub unsafe fn window_pane_last_index(
    mut wp: *mut window_pane,
    mut i: *mut u_int,
) -> ::core::ffi::c_int {
    let mut w: *mut window = (*wp).window as *mut window;
    let mut wq: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    *i = 0 as u_int;
    wq = window_pane_stack_first(w);
    while !wq.is_null() {
        if wq == wp {
            return 0 as ::core::ffi::c_int;
        }
        *i = (*i).wrapping_add(1);
        wq = window_pane_stack_next(w, wq);
    }
    return -(1 as ::core::ffi::c_int);
}
pub unsafe fn window_count_panes(
    mut w: *mut window,
    mut with_floating: ::core::ffi::c_int,
) -> u_int {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut n: u_int = 0 as u_int;
    wp = window_pane_first(w);
    while !wp.is_null() {
        if with_floating != 0 || window_pane_is_floating(wp) == 0 {
            n = n.wrapping_add(1);
        }
        wp = window_pane_next(wp);
    }
    return n;
}
pub unsafe fn window_destroy_panes(mut w: *mut window) {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    while !window_pane_stack_first(w).is_null() {
        wp = window_pane_stack_first(w);
        window_pane_stack_remove(&raw mut (*w).last_panes, wp);
    }
    while !window_pane_first(w).is_null() {
        wp = window_pane_first(w);
        window_pane_list_remove(w, wp);
        window_pane_z_remove(w, wp);
        window_pane_destroy(wp);
    }
}
pub unsafe fn window_printable_flags(
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
    if wl == winlink_stack_first(&(*s).lastw) {
        let fresh8 = pos;
        pos = pos.wrapping_add(1);
        flags[fresh8 as usize] = '-' as i32 as ::core::ffi::c_char;
    }
    if server_check_marked() != 0 && wl == marked_pane.wl {
        let fresh9 = pos;
        pos = pos.wrapping_add(1);
        flags[fresh9 as usize] = 'M' as i32 as ::core::ffi::c_char;
    }
    if !(*(*wl).window_ptr()).modal.is_null() {
        let fresh10 = pos;
        pos = pos.wrapping_add(1);
        flags[fresh10 as usize] = 'O' as i32 as ::core::ffi::c_char;
    }
    if (*(*wl).window_ptr()).flags & WINDOW_ZOOMED != 0 {
        let fresh11 = pos;
        pos = pos.wrapping_add(1);
        flags[fresh11 as usize] = 'Z' as i32 as ::core::ffi::c_char;
    }
    flags[pos as usize] = '\0' as i32 as ::core::ffi::c_char;
    return &raw mut flags as *mut ::core::ffi::c_char;
}
pub unsafe fn window_pane_printable_flags(mut wp: *mut window_pane) -> *const ::core::ffi::c_char {
    let mut w: *mut window = (*wp).window as *mut window;
    static mut flags: [::core::ffi::c_char; 32] = [0; 32];
    let mut pos: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if wp == (*w).active {
        let fresh12 = pos;
        pos = pos + 1;
        flags[fresh12 as usize] = '*' as i32 as ::core::ffi::c_char;
    }
    if wp == window_pane_stack_first(w) {
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
pub unsafe fn window_pane_find_by_id_str(mut s: *const ::core::ffi::c_char) -> *mut window_pane {
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
pub unsafe fn window_pane_find_by_id(mut id: u_int) -> *mut window_pane {
    let mut wp: window_pane = window_pane {
        id: 0,
        active_point: 0,
        window: ::core::ptr::null_mut::<window>(),
        options: None,
        layout_cell: ::core::ptr::null_mut::<layout_cell>(),
        saved_layout_cell: ::core::ptr::null_mut::<layout_cell>(),
        sx: 0,
        sy: 0,
        xoff: 0,
        yoff: 0,
        flags: 0,
        sync_dirty: None,
        sync_dirty_size: 0,
        sb_slider_y: 0,
        sb_slider_h: 0,
        sb_auto_visible: 0,
        sb_auto_hover: 0,
        sb_auto_timer: event::default(),
        argv: Vec::new(),
        shell: None,
        cwd: None,
        pid: 0,
        tty: [0; 32],
        status: 0,
        dead_time: timeval {
            tv_sec: 0,
            tv_usec: 0,
        },
        wait_item: ::core::ptr::null_mut::<cmdq_item>(),
        editor: None,
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
        resize_queue: window_pane_resizes::default(),
        resize_timer: event::default(),
        sync_timer: event::default(),
        ictx: None,
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
            palette: None,
            default_palette: None,
        },
        last_theme: THEME_UNKNOWN,
        border_status_line: style_line_entry {
            expanded: None,
            ranges: style_ranges::default(),
        },
        pipe_fd: 0,
        pipe_pid: 0,
        pipe_event: ::core::ptr::null_mut::<bufferevent>(),
        pipe_offset: window_pane_offset { used: 0 },
        screen: ::core::ptr::null_mut::<screen>(),
        base: screen::empty(),
        status_screen: screen::empty(),
        modes: window_pane_modes::default(),
        searchstr: None,
        searchregex: 0,
        prompt: None,
        prompt_data: None,
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
        tree_entry: window_pane_tree_entry { owner: None },
    };
    wp.id = id;
    return window_pane_tree_find(&*std::ptr::addr_of!(all_window_panes), &wp);
}
pub(crate) unsafe fn window_pane_weak(
    wp: *mut window_pane,
) -> std::rc::Weak<std::cell::UnsafeCell<window_pane>> {
    crate::src::shared::rc::downgrade(wp)
}

/// Retain a live pane for an operation through a nonowning model link.
/// Logical destruction invalidates observers even if another owner keeps the
/// allocation alive. This is not an accessor for final-drop cleanup.
pub(crate) unsafe fn window_pane_upgrade(
    pane: &std::rc::Weak<std::cell::UnsafeCell<window_pane>>,
) -> Option<std::rc::Rc<std::cell::UnsafeCell<window_pane>>> {
    let owner = pane.upgrade()?;
    if (*crate::src::shared::rc::as_ptr(&owner)).flags & PANE_DESTROYED != 0 {
        return None;
    }
    Some(owner)
}

pub unsafe fn window_pane_first(w: *mut window) -> *mut window_pane {
    if w.is_null() {
        std::ptr::null_mut()
    } else {
        (*w).panes.first()
    }
}

pub unsafe fn window_pane_last(w: *mut window) -> *mut window_pane {
    if w.is_null() {
        std::ptr::null_mut()
    } else {
        (*w).panes.last()
    }
}

pub unsafe fn window_pane_next(wp: *mut window_pane) -> *mut window_pane {
    if wp.is_null() {
        std::ptr::null_mut()
    } else {
        (*(*wp).window).panes.next(wp)
    }
}

pub unsafe fn window_pane_previous(wp: *mut window_pane) -> *mut window_pane {
    if wp.is_null() {
        std::ptr::null_mut()
    } else {
        (*(*wp).window).panes.previous(wp)
    }
}

pub unsafe fn window_pane_z_first(w: *mut window) -> *mut window_pane {
    if w.is_null() {
        std::ptr::null_mut()
    } else {
        (*w).z_index.first()
    }
}

pub unsafe fn window_pane_z_last(w: *mut window) -> *mut window_pane {
    if w.is_null() {
        std::ptr::null_mut()
    } else {
        (*w).z_index.last()
    }
}

pub unsafe fn window_pane_z_next(wp: *mut window_pane) -> *mut window_pane {
    if wp.is_null() {
        std::ptr::null_mut()
    } else {
        (*(*wp).window).z_index.next(wp)
    }
}

pub unsafe fn window_pane_z_previous(wp: *mut window_pane) -> *mut window_pane {
    if wp.is_null() {
        std::ptr::null_mut()
    } else {
        (*(*wp).window).z_index.previous(wp)
    }
}

pub unsafe fn window_pane_stack_first(w: *mut window) -> *mut window_pane {
    if w.is_null() {
        std::ptr::null_mut()
    } else {
        (*w).last_panes.first()
    }
}

pub unsafe fn window_pane_stack_next(w: *mut window, wp: *mut window_pane) -> *mut window_pane {
    if w.is_null() {
        std::ptr::null_mut()
    } else {
        (*w).last_panes.next(wp)
    }
}

pub unsafe fn window_pane_list_remove(w: *mut window, wp: *mut window_pane) {
    assert!(!w.is_null() && !wp.is_null());
    assert!((*w).panes.remove_ptr(wp), "pane is not in its window order");
}

pub unsafe fn window_pane_z_remove(w: *mut window, wp: *mut window_pane) {
    assert!(!w.is_null() && !wp.is_null());
    assert!(
        (*w).z_index.remove_ptr(wp),
        "pane is not in its stacking order"
    );
}

pub unsafe fn window_pane_list_insert_front(w: *mut window, wp: *mut window_pane) {
    (*w).panes.push_front(window_pane_weak(wp));
}

pub unsafe fn window_pane_list_insert_back(w: *mut window, wp: *mut window_pane) {
    (*w).panes.push_back(window_pane_weak(wp));
}

pub unsafe fn window_pane_list_insert_before(
    w: *mut window,
    before: *mut window_pane,
    wp: *mut window_pane,
) {
    (*w).panes.insert_before(before, window_pane_weak(wp));
}

pub unsafe fn window_pane_list_insert_after(
    w: *mut window,
    after: *mut window_pane,
    wp: *mut window_pane,
) {
    (*w).panes.insert_after(after, window_pane_weak(wp));
}

pub unsafe fn window_pane_z_insert_front(w: *mut window, wp: *mut window_pane) {
    (*w).z_index.push_front(window_pane_weak(wp));
}

pub unsafe fn window_pane_z_insert_back(w: *mut window, wp: *mut window_pane) {
    (*w).z_index.push_back(window_pane_weak(wp));
}

pub unsafe fn window_pane_z_insert_before(
    w: *mut window,
    before: *mut window_pane,
    wp: *mut window_pane,
) {
    (*w).z_index.insert_before(before, window_pane_weak(wp));
}

pub unsafe fn window_pane_z_insert_after(
    w: *mut window,
    after: *mut window_pane,
    wp: *mut window_pane,
) {
    (*w).z_index.insert_after(after, window_pane_weak(wp));
}

pub unsafe fn window_pane_swap_order(
    first_window: *mut window,
    first: *mut window_pane,
    second_window: *mut window,
    second: *mut window_pane,
) {
    if first_window == second_window {
        (*first_window).panes.swap_ptrs(first, second);
        return;
    }
    let first_position = (*first_window)
        .panes
        .position(first)
        .expect("first pane is not in order");
    let second_position = (*second_window)
        .panes
        .position(second)
        .expect("second pane is not in order");
    let first_weak = (*first_window).panes.remove_at(first);
    let second_weak = (*second_window).panes.remove_at(second);
    (*first_window).panes.insert_at(first_position, second_weak);
    (*second_window)
        .panes
        .insert_at(second_position, first_weak);
}

pub unsafe fn window_pane_z_swap_order(
    first_window: *mut window,
    first: *mut window_pane,
    second_window: *mut window,
    second: *mut window_pane,
) {
    if first_window == second_window {
        (*first_window).z_index.swap_ptrs(first, second);
        return;
    }
    let first_position = (*first_window)
        .z_index
        .position(first)
        .expect("first pane is not in stacking order");
    let second_position = (*second_window)
        .z_index
        .position(second)
        .expect("second pane is not in stacking order");
    let first_weak = (*first_window).z_index.remove_at(first);
    let second_weak = (*second_window).z_index.remove_at(second);
    (*first_window)
        .z_index
        .insert_at(first_position, second_weak);
    (*second_window)
        .z_index
        .insert_at(second_position, first_weak);
}

/// Return the next mode in a pane's stack. Entries are boxed individually,
/// so this derives ordering from the owning collection without putting queue
/// links into each callback-visible mode entry.
pub(crate) unsafe fn window_pane_mode_next(wme: *mut window_mode_entry) -> *mut window_mode_entry {
    if wme.is_null() || (*wme).wp.is_null() {
        return ::core::ptr::null_mut();
    }
    let Some(storage) = (*(*wme).wp).modes.storage.as_ref() else {
        return ::core::ptr::null_mut();
    };
    let Some(index) = storage.entries.iter().position(|entry| {
        (&**entry as *const window_mode_entry) == (wme as *const window_mode_entry)
    }) else {
        return ::core::ptr::null_mut();
    };
    storage
        .entries
        .get(index + 1)
        .map_or(::core::ptr::null_mut(), |entry| {
            (&**entry) as *const window_mode_entry as *mut window_mode_entry
        })
}

unsafe fn window_pane_mode_insert_front(
    wp: &mut window_pane,
    entry: Box<window_mode_entry>,
) -> *mut window_mode_entry {
    let modes = &mut wp.modes;
    let storage = modes.storage.get_or_insert_with(Default::default);
    let wme = (&*entry) as *const window_mode_entry as *mut window_mode_entry;
    storage.entries.insert(0, entry);
    modes.active = wme;
    wme
}

unsafe fn window_pane_mode_remove(
    wp: *mut window_pane,
    wme: *mut window_mode_entry,
) -> Option<Box<window_mode_entry>> {
    let modes = &mut (*wp).modes;
    let (removed, empty) = {
        let storage = modes.storage.as_mut()?;
        let index = storage.entries.iter().position(|entry| {
            (&**entry as *const window_mode_entry) == (wme as *const window_mode_entry)
        })?;
        let removed = storage.entries.remove(index);
        (removed, storage.entries.is_empty())
    };
    if empty {
        modes.storage = None;
        modes.active = ::core::ptr::null_mut();
    } else {
        modes.active = modes
            .storage
            .as_ref()
            .and_then(|storage| storage.entries.first())
            .map_or(::core::ptr::null_mut(), |entry| {
                (&**entry) as *const window_mode_entry as *mut window_mode_entry
            });
    }
    Some(removed)
}

unsafe fn window_pane_mode_promote(wp: *mut window_pane, wme: *mut window_mode_entry) {
    if (*wp).modes.active == wme {
        return;
    }
    let Some(entry) = window_pane_mode_remove(wp, wme) else {
        return;
    };
    window_pane_mode_insert_front(&mut *wp, entry);
}

#[cfg(test)]
mod window_mode_collection_tests {
    use super::*;

    unsafe fn boxed_mode(wp: *mut window_pane) -> Box<window_mode_entry> {
        Box::new(window_mode_entry {
            wp,
            swp: ::core::ptr::null_mut(),
            mode: &window_copy_mode,
            data: ::core::ptr::null_mut(),
            data_owner: None,
            screen: ::core::ptr::null_mut(),
            prefix: 1,
            kill: 0,
        })
    }

    #[test]
    fn pane_mode_stack_reorders_and_removes_stable_entries() {
        unsafe {
            // The production pane owner is zero-initialized before its fields
            // are populated; modes itself is initialized explicitly here.
            let wp = Box::into_raw(Box::new(window_pane::empty()));
            (*wp).modes = window_pane_modes::default();

            let a = window_pane_mode_insert_front(&mut *wp, boxed_mode(wp));
            let b = window_pane_mode_insert_front(&mut *wp, boxed_mode(wp));
            let c = window_pane_mode_insert_front(&mut *wp, boxed_mode(wp));
            assert_eq!((*wp).modes.active, c);
            assert_eq!(window_pane_mode_next(c), b);
            assert_eq!(window_pane_mode_next(b), a);
            assert!(window_pane_mode_next(a).is_null());

            window_pane_mode_promote(wp, a);
            assert_eq!((*wp).modes.active, a);
            assert_eq!(window_pane_mode_next(a), c);
            assert_eq!(window_pane_mode_next(c), b);

            let removed = window_pane_mode_remove(wp, c).expect("mode was present");
            assert_eq!((&*removed) as *const window_mode_entry as *mut _, c);
            assert_eq!((*wp).modes.active, a);
            assert_eq!(window_pane_mode_next(a), b);
            drop(removed);

            // Force Vec growth after callbacks already hold `a` and `b`.
            for _ in 0..64 {
                window_pane_mode_insert_front(&mut *wp, boxed_mode(wp));
            }
            assert_eq!(window_pane_mode_next(a), b);
            assert!(window_pane_mode_next(b).is_null());

            while !(*wp).modes.active.is_null() {
                let top = (*wp).modes.active;
                drop(window_pane_mode_remove(wp, top).expect("mode was present"));
            }
            assert!((*wp).modes.storage.is_none());

            drop(Box::from_raw(wp));
        }
    }
}

/// Replace the pane-owned searchstr string.
pub(crate) fn window_pane_set_searchstr(wp: &mut window_pane, searchstr: Option<CString>) {
    wp.searchstr = searchstr;
}

/// Replace the pane-owned shell string.
pub(crate) fn window_pane_set_shell(wp: &mut window_pane, shell: Option<CString>) {
    wp.shell = shell;
}

/// Replace the pane-owned cwd string.
pub(crate) fn window_pane_set_cwd(wp: &mut window_pane, cwd: Option<CString>) {
    wp.cwd = cwd;
}

unsafe fn window_pane_create(
    mut w: *mut window,
    mut sx: u_int,
    mut sy: u_int,
    mut hlimit: u_int,
) -> *mut window_pane {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut host: [::core::ffi::c_char; 65] = [0; 65];
    wp = crate::src::shared::rc::new(window_pane::empty());
    (*wp).window = w as *mut window;
    (*wp).options = Some(crate::src::options::options_create_owned(options_owner_ptr(&mut (*w).options)));
    (*wp).flags = PANE_STYLECHANGED;
    (*wp).cmd_status = -(1 as ::core::ffi::c_int);
    let fresh2 = next_window_pane_id;
    next_window_pane_id = next_window_pane_id.wrapping_add(1);
    (*wp).id = fresh2;
    window_pane_tree_insert(&raw mut all_window_panes, wp);
    (*wp).fd = -(1 as ::core::ffi::c_int);
    (*wp).modes = window_pane_modes::default();
    (*wp).resize_queue = window_pane_resizes::default();
    (*wp).sx = sx;
    (*wp).sy = sy;
    (*wp).pipe_fd = -(1 as ::core::ffi::c_int);
    (*wp).control_bg = -(1 as ::core::ffi::c_int);
    (*wp).control_fg = -(1 as ::core::ffi::c_int);
    style_set_scrollbar_style_from_option(&raw mut (*wp).scrollbar_style, options_owner_ptr(&mut (*wp).options));
    colour_palette_init(&mut (*wp).palette);
    colour_palette_from_option(Some(&mut (*wp).palette), options_owner_ptr(&mut (*wp).options));
    screen_init(&mut (*wp).base, sx, sy, hlimit);
    (*wp).screen = &raw mut (*wp).base;
    window_pane_default_cursor(wp);
    screen_init(&mut (*wp).status_screen, 1 as u_int, 1 as u_int, 0 as u_int);
    style_ranges_init(&raw mut (*wp).border_status_line.ranges);
    event_set(
        &raw mut (*wp).sb_auto_timer,
        -(1 as ::core::ffi::c_int),
        0 as ::core::ffi::c_short,
        move |_, _| unsafe { window_pane_scrollbar_timer(wp as *mut ::core::ffi::c_void) },
    );
    if gethostname(
        &raw mut host as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 65]>() as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        screen_set_title(
            &mut (*wp).base,
            CStr::from_ptr(host.as_ptr()),
            0 as ::core::ffi::c_int,
        );
    }
    return wp;
}
pub unsafe fn window_pane_wait_finish(mut wp: *mut window_pane) {
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
unsafe fn window_pane_free_modes(mut wp: *mut window_pane) {
    let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
    while !(*wp).modes.active.is_null() {
        wme = (*wp).modes.active;
        let entry = window_pane_mode_remove(wp, wme).expect("mode entry is owned by pane");
        (*(*wme).mode).free.expect("non-null function pointer")(wme);
        drop(entry);
    }
    (*wp).screen = &raw mut (*wp).base;
}
unsafe fn window_pane_scrollbar_timer(mut arg: *mut ::core::ffi::c_void) {
    let mut wp: *mut window_pane = arg as *mut window_pane;
    (*wp).sb_auto_hover = 0 as ::core::ffi::c_int;
    window_pane_scrollbar_hide(wp);
}
unsafe fn window_pane_scrollbar_auto_hide(mut wp: *mut window_pane) -> ::core::ffi::c_int {
    return ((*(*wp).window).sb == PANE_SCROLLBARS_MODAL
        || (*(*wp).window).sb == PANE_SCROLLBARS_AUTOHIDE) as ::core::ffi::c_int;
}
pub unsafe fn window_pane_scrollbar_overlay_visible(
    mut wp: *mut window_pane,
) -> ::core::ffi::c_int {
    return (window_pane_scrollbar_overlay(wp) != 0 && window_pane_scrollbar_visible(wp) != 0)
        as ::core::ffi::c_int;
}
pub unsafe fn window_pane_scrollbar_redraw(mut wp: *mut window_pane) {
    if window_pane_scrollbar_visible(wp) == 0 {
        return;
    }
    if window_pane_scrollbar_overlay_visible(wp) != 0 {
        (*wp).flags |= PANE_REDRAW;
        return;
    }
    (*wp).flags |= PANE_REDRAWSCROLLBAR;
}
unsafe fn window_pane_scrollbar_redraw_visibility(mut wp: *mut window_pane) {
    redraw_invalidate_scene((*wp).window as *mut window);
    (*wp).flags |= PANE_REDRAW;
    server_redraw_window((*wp).window as *mut window);
}
unsafe fn window_pane_destroy(mut wp: *mut window_pane) {
    window_pane_wait_finish(wp);
    spawn_editor_finish(wp);
    window_pane_tree_remove(&raw mut all_window_panes, wp);
    (*wp).flags |= PANE_DESTROYED;
    window_pane_clear_prompt(wp);
    window_pane_free_modes(wp);
    screen_write_clear_dirty(wp);
    if (*wp).fd != -(1 as ::core::ffi::c_int) {
        utempter_remove_record((*wp).fd);
        kill(getpid(), SIGCHLD);
    }
    // Empty panes have stream buffers and an input parser without a PTY.
    bufferevent_free((*wp).event);
    (*wp).event = ::core::ptr::null_mut::<bufferevent>();
    if (*wp).fd != -(1 as ::core::ffi::c_int) {
        close((*wp).fd);
        (*wp).fd = -(1 as ::core::ffi::c_int);
    }
    if let Some(ictx) = (*wp).ictx.take() {
        input_free(ictx);
    }
    if (*wp).pipe_fd != -(1 as ::core::ffi::c_int) {
        bufferevent_free((*wp).pipe_event);
        (*wp).pipe_event = ::core::ptr::null_mut::<bufferevent>();
        close((*wp).pipe_fd);
        (*wp).pipe_fd = -(1 as ::core::ffi::c_int);
    }
    if event_initialized(&(*wp).resize_timer) != 0 {
        event_del(&raw mut (*wp).resize_timer);
    }
    if event_initialized(&(*wp).sync_timer) != 0 {
        event_del(&raw mut (*wp).sync_timer);
    }
    if event_initialized(&(*wp).sb_auto_timer) != 0 {
        event_del(&raw mut (*wp).sb_auto_timer);
    }
    window_pane_clear_resizes(wp, ::core::ptr::null_mut::<window_pane_resize>());
    window_pane_remove_ref(
        wp,
        b"window_pane_destroy\0" as *const u8 as *const ::core::ffi::c_char,
    );
}
unsafe fn window_pane_free(mut wp: *mut window_pane) {
    log_debug(format_args!("pane %{} freed", ((*wp).id) as u32));
    // Logical pane destruction normally takes this first. Also keep direct
    // owner destruction safe: parser timers and sync state precede screens.
    drop((*wp).ictx.take());
    window_pane_set_searchstr(&mut *wp, None);
    if (*wp).status_screen.grid.is_some() {
        screen_free(&mut (*wp).status_screen);
    }
    if (*wp).base.grid.is_some() {
        screen_free(&mut (*wp).base);
    }
    drop((*wp).options.take());
    window_pane_set_cwd(&mut *wp, None);
    window_pane_set_shell(&mut *wp, None);
    colour_palette_free(Some(&mut (*wp).palette));
    style_ranges_free(&raw mut (*wp).border_status_line.ranges);
}
unsafe fn window_pane_read_callback(mut data: *mut ::core::ffi::c_void) {
    let mut wp: *mut window_pane = data as *mut window_pane;
    let mut wpo: *mut window_pane_offset = &raw mut (*wp).pipe_offset;
    let mut size: size_t = evbuffer_get_length(&(*(*wp).event).input);
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
    log_debug(format_args!(
        "%{} has {} bytes",
        ((*wp).id) as u32,
        (size) as usize
    ));
    c = clients.first();
    while !c.is_null() {
        if !(*c).session.is_null() && (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
            control_write_output(c, wp);
        }
        c = clients.next(c);
    }
    input_parse_pane(wp);
    bufferevent_disable((*wp).event, EV_READ as ::core::ffi::c_short);
}
unsafe fn window_pane_error_callback(mut data: *mut ::core::ffi::c_void) {
    let mut wp: *mut window_pane = data as *mut window_pane;
    log_debug(format_args!("%{} error", ((*wp).id) as u32));
    (*wp).flags |= PANE_EXITED;
    if window_pane_destroy_ready(wp) != 0 {
        server_destroy_pane(wp, 1 as ::core::ffi::c_int);
    }
}
pub unsafe fn window_pane_set_event(mut wp: *mut window_pane) {
    setblocking((*wp).fd, 0 as ::core::ffi::c_int);
    (*wp).event = bufferevent_new(
        (*wp).fd,
        bufferevent_data_callback(move |_| unsafe {
            window_pane_read_callback(wp as *mut ::core::ffi::c_void)
        }),
        None,
        bufferevent_event_callback(move |_, _| unsafe {
            window_pane_error_callback(wp as *mut ::core::ffi::c_void)
        }),
    );
    if (*wp).event.is_null() {
        fatalx(|out| out.write_all(b"out of memory"));
    }
    (*wp).ictx = Some(input_init(
        wp,
        (*wp).event,
        &raw mut (*wp).palette,
        ::core::ptr::null_mut::<client>(),
    ));
    bufferevent_enable((*wp).event, (EV_READ | EV_WRITE) as ::core::ffi::c_short);
}
pub unsafe fn window_pane_clear_resizes(
    mut wp: *mut window_pane,
    mut except: *mut window_pane_resize,
) {
    (*wp).resize_queue.clear_except(except);
}
pub unsafe fn window_pane_resize(mut wp: *mut window_pane, mut sx: u_int, mut sy: u_int) {
    let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
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
    let old_sx = (*wp).sx;
    let old_sy = (*wp).sy;
    (*wp).resize_queue.push_back(window_pane_resize {
        sx,
        sy,
        osx: old_sx,
        osy: old_sy,
    });
    (*wp).sx = sx;
    (*wp).sy = sy;
    log_debug(format_args!(
        "{}: %{} resize {}x{}",
        "window_pane_resize",
        ((*wp).id) as u32,
        (sx) as u32,
        (sy) as u32
    ));
    let reflow = (*wp).base.saved_grid.is_none() as ::core::ffi::c_int;
    screen_resize(&mut (*wp).base, sx, sy, reflow);
    wme = (*wp).modes.active;
    if !wme.is_null() && (*(*wme).mode).resize.is_some() {
        (*(*wme).mode).resize.expect("non-null function pointer")(wme, sx, sy);
    }
    let mut ep = event_payload_create();
    cmd_find_from_pane(&raw mut fs, wp, 0 as ::core::ffi::c_int);
    event_payload_set_target(&mut *ep, &fs);
    event_payload_set_pane(
        &mut *ep,
        b"pane\0" as *const u8 as *const ::core::ffi::c_char,
        wp,
    );
    event_payload_set_window(
        &mut *ep,
        b"window\0" as *const u8 as *const ::core::ffi::c_char,
        (*wp).window as *mut window,
    );
    event_payload_set_uint(
        &mut *ep,
        b"width\0" as *const u8 as *const ::core::ffi::c_char,
        sx,
    );
    event_payload_set_uint(
        &mut *ep,
        b"height\0" as *const u8 as *const ::core::ffi::c_char,
        sy,
    );
    event_payload_set_uint(
        &mut *ep,
        b"old_width\0" as *const u8 as *const ::core::ffi::c_char,
        old_sx,
    );
    event_payload_set_uint(
        &mut *ep,
        b"old_height\0" as *const u8 as *const ::core::ffi::c_char,
        old_sy,
    );
    events_fire(
        b"pane-resized\0" as *const u8 as *const ::core::ffi::c_char,
        ep,
    );
}
pub unsafe fn window_pane_set_mode(
    mut wp: *mut window_pane,
    mut swp: *mut window_pane,
    mode: &'static window_mode,
    mut item: *mut cmdq_item,
    mut fs: *mut cmd_find_state,
    mut args: *mut args,
) -> ::core::ffi::c_int {
    let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
    let mut w: *mut window = (*wp).window as *mut window;
    let mut name: *const ::core::ffi::c_char = (*mode).name.as_ptr();
    let mut oname: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if !(*wp).modes.active.is_null() {
        if std::ptr::eq((*(*wp).modes.active).mode, mode) {
            return 1 as ::core::ffi::c_int;
        }
        if (*(*(*wp).modes.active).mode).flags & WINDOW_MODE_NO_STACK != 0 {
            window_pane_reset_mode(wp);
        }
    }
    if !(*wp).modes.active.is_null() {
        oname = (*(*(*wp).modes.active).mode).name.as_ptr();
    }
    wme = (*wp).modes.active;
    while !wme.is_null() {
        if std::ptr::eq((*wme).mode, mode) {
            break;
        }
        wme = window_pane_mode_next(wme);
    }
    if !wme.is_null() {
        window_pane_mode_promote(wp, wme);
    } else {
        // The pane owns a stable Box address for as long as callbacks retain it.
        let entry = Box::new(window_mode_entry {
            wp,
            swp,
            mode,
            data: ::core::ptr::null_mut(),
            data_owner: None,
            screen: ::core::ptr::null_mut(),
            prefix: 1,
            kill: 0,
        });
        wme = window_pane_mode_insert_front(&mut *wp, entry);
        (*wme).screen =
            (*(*wme).mode).init.expect("non-null function pointer")(wme, item, fs, args);
        if (*wme).screen.is_null() {
            drop(window_pane_mode_remove(wp, wme).expect("mode entry is owned by pane"));
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
pub unsafe fn window_pane_reset_mode(mut wp: *mut window_pane) {
    let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
    let mut next: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
    let mut w: *mut window = (*wp).window as *mut window;
    let mut kill_0: ::core::ffi::c_int = 0;
    let mut name: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut p: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if (*wp).modes.active.is_null() {
        return;
    }
    wme = (*wp).modes.active;
    p = (*(*wme).mode).name.as_ptr();
    kill_0 = (*wme).kill;
    let entry = window_pane_mode_remove(wp, wme).expect("mode entry is owned by pane");
    (*(*wme).mode).free.expect("non-null function pointer")(wme);
    drop(entry);
    next = (*wp).modes.active;
    if next.is_null() {
        (*wp).flags &= !PANE_UNSEENCHANGES;
        log_debug(format_args!("{}: no next mode", "window_pane_reset_mode"));
        (*wp).screen = &raw mut (*wp).base;
    } else {
        log_debug(format_args!(
            "{}: next mode is {}",
            "window_pane_reset_mode",
            log_cstr(((*(*next).mode).name.as_ptr()) as *const _)
        ));
        (*wp).screen = (*next).screen;
        if (*(*next).mode).resize.is_some() {
            (*(*next).mode).resize.expect("non-null function pointer")(next, (*wp).sx, (*wp).sy);
        }
    }
    name = if next.is_null() {
        ::core::ptr::null::<::core::ffi::c_char>()
    } else {
        (*(*next).mode).name.as_ptr()
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
pub unsafe fn window_pane_reset_mode_all(mut wp: *mut window_pane) {
    while !(*wp).modes.active.is_null() {
        window_pane_reset_mode(wp);
    }
}
fn window_pane_prompt_input_callback(
    data: &refbox::Weak<window_pane_prompt>,
    input: Option<&CStr>,
    key: prompt_key_result,
) -> prompt_result {
    let (client, callback) = {
        let Ok(mut state) = data.try_borrow_mut() else {
            return PROMPT_CLOSE;
        };
        (state.c.upgrade(), state.inputcb.take())
    };
    let Some(mut callback) = callback else {
        return PROMPT_CLOSE;
    };
    let result = callback(
        client
            .as_ref()
            .and_then(|client| std::ptr::NonNull::new(crate::src::shared::rc::as_ptr(client))),
        input,
        key,
    );
    if let Ok(mut state) = data.try_borrow_mut() {
        state.inputcb = Some(callback);
    }
    result
}
unsafe fn window_pane_prompt_free_callback(data: &WindowPanePromptOwner) {
    let (wp_id, callback, inputcb) = {
        let mut state = data.try_borrow_mut().expect("prompt cleanup record");
        (state.wp_id, state.freecb.take(), state.inputcb.take())
    };
    let wp = window_pane_find_by_id(wp_id);
    if !wp.is_null()
        && (*wp)
            .prompt_data
            .as_ref()
            .is_some_and(|current| current.is(data))
    {
        (*wp).prompt_data = None;
    }
    if let Some(callback) = callback {
        callback();
    }
    drop(inputcb);
}
pub unsafe fn window_pane_set_prompt(
    mut wp: *mut window_pane,
    mut c: *mut client,
    mut fs: *mut cmd_find_state,
    mut msg: *const ::core::ffi::c_char,
    mut input: *const ::core::ffi::c_char,
    mut inputcb: status_prompt_input_cb,
    mut freecb: prompt_free_cb,
    mut flags: ::core::ffi::c_int,
    mut type_0: prompt_type,
) {
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut pd = prompt_create_data::default();
    if !c.is_null() {
        s = (*c).session;
    }
    window_pane_clear_prompt(wp);
    let wpp = refbox::RefBox::new(window_pane_prompt {
        wp_id: (*wp).id,
        c: if c.is_null() {
            Weak::new()
        } else {
            crate::src::shared::rc::downgrade(c)
        },
        inputcb,
        freecb,
        type_0,
    });
    prompt_set_options(&mut pd, s.as_mut());
    pd.fs = fs.as_ref();
    pd.prompt = CStr::from_ptr(msg);
    pd.input = if input.is_null() {
        None
    } else {
        Some(CStr::from_ptr(input))
    };
    pd.type_0 = type_0;
    pd.flags = flags;
    let input_data = wpp.downgrade();
    let identity = input_data.clone();
    pd.inputcb = Some(Box::new(move |s, key| {
        window_pane_prompt_input_callback(&input_data, s, key)
    }));
    let free_data = wpp;
    pd.freecb = Some(Box::new(move || unsafe {
        window_pane_prompt_free_callback(&free_data)
    }));
    let prompt = prompt_create(pd);
    let prompt_observer = prompt.downgrade();
    (*wp).prompt = Some(prompt);
    let prompt = prompt_observer;
    (*wp).prompt_data = Some(identity);
    (*wp).flags |= PANE_REDRAW;
    prompt_incremental_start(&prompt);
    window_fire_pane_prompt(
        b"pane-prompt-opened\0" as *const u8 as *const ::core::ffi::c_char,
        wp,
        type_0,
    );
}
pub unsafe fn window_pane_clear_prompt(mut wp: *mut window_pane) {
    let prompt = (*wp).prompt.take();
    let wpp = (*wp).prompt_data.clone();
    let mut type_0: prompt_type = PROMPT_TYPE_INVALID;
    if let Some(prompt) = prompt {
        if let Some(wpp) = wpp {
            type_0 = wpp.try_borrow_mut().expect("live pane prompt record").type_0;
        }
        prompt_free(&prompt.downgrade());
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
pub unsafe fn window_pane_has_prompt(mut wp: *mut window_pane) -> ::core::ffi::c_int {
    return (*wp).prompt.is_some() as ::core::ffi::c_int;
}
pub unsafe fn window_pane_update_prompt(
    mut wp: *mut window_pane,
    mut msg: *const ::core::ffi::c_char,
    mut input: *const ::core::ffi::c_char,
) {
    if (*wp).prompt.is_some() {
        prompt_update(
            &mut (*wp).prompt.as_ref().expect("active prompt").try_borrow_mut().expect("unborrowed prompt"),
            CStr::from_ptr(msg),
            (!input.is_null()).then(|| CStr::from_ptr(input)),
        );
        (*wp).flags |= PANE_REDRAW;
    }
}
pub unsafe fn window_pane_prompt_key(
    mut wp: *mut window_pane,
    mut c: *mut client,
    mut key: key_code,
    mut m: *mut mouse_event,
) -> prompt_key_result {
    let prompt = (*wp).prompt.as_ref().map(|prompt| prompt.downgrade());
    let wpp = (*wp).prompt_data.clone();
    let mut result: prompt_key_result = PROMPT_KEY_NOT_HANDLED;
    let mut wp_id: u_int = (*wp).id;
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    let mut py: u_int = 0;
    let mut redraw: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let Some(prompt) = prompt else {
        return PROMPT_KEY_NOT_HANDLED;
    };
    if let Some(wpp) = &wpp {
        wpp.try_borrow_mut().expect("live prompt callback record").c = if c.is_null() {
            Weak::new()
        } else {
            crate::src::shared::rc::downgrade(c)
        };
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
            if !c.is_null() && status_at_line(&*c) == 0 as ::core::ffi::c_int {
                py = 0 as u_int;
            } else {
                py = (*wp).sy.wrapping_sub(1 as u_int);
            }
            if y == py {
                result = prompt_mouse(
                    &mut prompt.try_borrow_mut().expect("live unborrowed prompt"),
                    x,
                    0 as u_int,
                    (*wp).sx,
                    Some(&mut redraw),
                );
            } else {
                result = PROMPT_KEY_NOT_HANDLED;
            }
        }
    } else {
        result = prompt_key(&prompt, key, &mut redraw);
    }
    wp = window_pane_find_by_id(wp_id);
    if wp.is_null() {
        return result;
    }
    if let Some(wpp) = &wpp {
        if (*wp).prompt_data.as_ref() == Some(wpp) {
            wpp.try_borrow_mut().expect("live prompt callback record").c = Weak::new();
        }
    }
    if (*wp)
        .prompt
        .as_ref()
        .is_some_and(|current| prompt.is(current))
        && (result as ::core::ffi::c_uint
            == PROMPT_KEY_CLOSE as ::core::ffi::c_int as ::core::ffi::c_uint
            || prompt_closed(&prompt.try_borrow_mut().expect("live unborrowed prompt")) != 0)
    {
        window_pane_clear_prompt(wp);
    }
    if redraw != 0
        || !(*wp)
            .prompt
            .as_ref()
            .is_some_and(|current| prompt.is(current))
    {
        (*wp).flags |= PANE_REDRAW;
    }
    return result;
}
unsafe fn window_pane_copy_paste(
    mut wp: *mut window_pane,
    mut buf: *mut ::core::ffi::c_char,
    mut len: size_t,
) {
    let mut loop_0: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    loop_0 = window_pane_first((*wp).window);
    while !loop_0.is_null() {
        if loop_0 != wp
            && (*loop_0).modes.active.is_null()
            && (*loop_0).fd != -(1 as ::core::ffi::c_int)
            && !(*loop_0).flags & PANE_INPUTOFF != 0
            && window_pane_is_visible(loop_0) != 0
            && options_get_number(
                options_owner_ptr(&mut (*loop_0).options),
                b"synchronize-panes\0" as *const u8 as *const ::core::ffi::c_char,
            ) != 0
        {
            log_debug(format_args!(
                "{}: {}",
                "window_pane_copy_paste",
                log_cstr_n((buf) as *const _, len as ::core::ffi::c_int)
            ));
            bufferevent_write((*loop_0).event, buf as *const ::core::ffi::c_void, len);
        }
        loop_0 = window_pane_next(loop_0);
    }
}
unsafe fn window_pane_copy_key(mut wp: *mut window_pane, mut key: key_code) {
    let mut loop_0: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    loop_0 = window_pane_first((*wp).window);
    while !loop_0.is_null() {
        if loop_0 != wp
            && (*loop_0).modes.active.is_null()
            && (*loop_0).fd != -(1 as ::core::ffi::c_int)
            && !(*loop_0).flags & PANE_INPUTOFF != 0
            && window_pane_is_visible(loop_0) != 0
            && options_get_number(
                options_owner_ptr(&mut (*loop_0).options),
                b"synchronize-panes\0" as *const u8 as *const ::core::ffi::c_char,
            ) != 0
        {
            input_key_pane(loop_0, key, ::core::ptr::null_mut::<mouse_event>());
        }
        loop_0 = window_pane_next(loop_0);
    }
}
pub unsafe fn window_pane_paste(
    mut wp: *mut window_pane,
    mut key: key_code,
    mut buf: *mut ::core::ffi::c_char,
    mut len: size_t,
) {
    if !(*wp).modes.active.is_null() {
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
    log_debug(format_args!(
        "{}: {}",
        "window_pane_paste",
        log_cstr_n((buf) as *const _, len as ::core::ffi::c_int)
    ));
    bufferevent_write((*wp).event, buf as *const ::core::ffi::c_void, len);
    if options_get_number(
        options_owner_ptr(&mut (*wp).options),
        b"synchronize-panes\0" as *const u8 as *const ::core::ffi::c_char,
    ) != 0
    {
        window_pane_copy_paste(wp, buf, len);
    }
}
pub unsafe fn window_pane_key(
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
    wme = (*wp).modes.active;
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
        options_owner_ptr(&mut (*wp).options),
        b"synchronize-panes\0" as *const u8 as *const ::core::ffi::c_char,
    ) != 0
    {
        window_pane_copy_key(wp, key);
    }
    return 0 as ::core::ffi::c_int;
}
pub unsafe fn window_pane_is_visible(mut wp: *mut window_pane) -> ::core::ffi::c_int {
    if !(*(*wp).window).flags & WINDOW_ZOOMED != 0 {
        return 1 as ::core::ffi::c_int;
    }
    return ((*wp).layout_cell != NULL as *mut layout_cell) as ::core::ffi::c_int;
}
pub unsafe fn window_pane_exited(mut wp: *mut window_pane) -> ::core::ffi::c_int {
    return ((*wp).fd == -(1 as ::core::ffi::c_int) || (*wp).flags & PANE_EXITED != 0)
        as ::core::ffi::c_int;
}
pub unsafe fn window_pane_search(
    mut wp: *mut window_pane,
    term: &CStr,
    mut regex: ::core::ffi::c_int,
    mut ignore: ::core::ffi::c_int,
) -> u_int {
    let mut s: *mut screen = &raw mut (*wp).base;
    let mut regex_storage = RegexStorage::default();
    let mut i: u_int = 0;
    let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut found: ::core::ffi::c_int = 0;
    let glob = if regex == 0 {
        if ignore != 0 {
            flags |= FNM_CASEFOLD;
        }
        let term = term.to_bytes();
        let mut pattern = Vec::with_capacity(term.len() + 2);
        pattern.push(b'*');
        pattern.extend_from_slice(term);
        pattern.push(b'*');
        Some(CString::new(pattern).expect("search term contains no NUL"))
    } else {
        if ignore != 0 {
            flags |= REG_ICASE;
        }
        None
    };
    let regex_owner = if regex != 0 {
        let Ok(regex) = regex_storage.compile(term, flags | REG_EXTENDED) else {
            return 0;
        };
        Some(regex)
    } else {
        None
    };
    i = 0 as u_int;
    while i < (*s).grid().sy {
        let mut line = grid_view_string_cells_bytes((*s).grid(), i, (*s).grid().sx);
        if let Some(nul) = line.iter().position(|&byte| byte == 0) {
            line.truncate(nul);
        }
        while let Some(&last) = line.last() {
            if *(*__ctype_b_loc()).offset(last as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                & _ISspace as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
                == 0
            {
                break;
            }
            line.pop();
        }
        line.push(0);
        log_debug(format_args!(
            "{}: {}",
            "window_pane_search",
            log_cstr((line.as_ptr().cast::<::core::ffi::c_char>()) as *const _)
        ));
        if regex == 0 {
            found = (fnmatch(
                glob.as_ref()
                    .expect("nonregex search has a pattern")
                    .as_ptr(),
                line.as_ptr().cast::<::core::ffi::c_char>(),
                flags,
            ) == 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
        } else {
            found = regex_owner
                .as_ref()
                .unwrap()
                .is_match(CStr::from_bytes_with_nul(&line).unwrap()) as i32;
        }
        if found != 0 {
            break;
        }
        i = i.wrapping_add(1);
    }
    drop(regex_owner);
    if i == (*s).grid().sy {
        return 0 as u_int;
    }
    return i.wrapping_add(1 as u_int);
}
unsafe fn window_pane_choose_best(list: &[*mut window_pane]) -> *mut window_pane {
    let Some((&first, rest)) = list.split_first() else {
        return ::core::ptr::null_mut();
    };
    let mut best = first;
    for &next in rest {
        if (*next).active_point > (*best).active_point {
            best = next;
        }
    }
    best
}
unsafe fn window_pane_full_size_offset(
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
pub unsafe fn window_pane_find_up(mut wp: *mut window_pane) -> *mut window_pane {
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut next: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut best: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut list: Vec<*mut window_pane> = Vec::new();
    let mut edge: ::core::ffi::c_int = 0;
    let mut left: ::core::ffi::c_int = 0;
    let mut right: ::core::ffi::c_int = 0;
    let mut end: ::core::ffi::c_int = 0;
    let mut status: ::core::ffi::c_int = 0;
    let mut found: ::core::ffi::c_int = 0;
    let mut xoff: ::core::ffi::c_int = 0;
    let mut yoff: ::core::ffi::c_int = 0;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    if wp.is_null() {
        return ::core::ptr::null_mut::<window_pane>();
    }
    w = (*wp).window as *mut window;
    status = window_get_pane_status(w);
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
    next = window_pane_first(w);
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
                    list.push(next);
                }
            }
        }
        next = window_pane_next(next);
    }
    best = window_pane_choose_best(&list);
    return best;
}
pub unsafe fn window_pane_find_down(mut wp: *mut window_pane) -> *mut window_pane {
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut next: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut best: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut list: Vec<*mut window_pane> = Vec::new();
    let mut edge: ::core::ffi::c_int = 0;
    let mut left: ::core::ffi::c_int = 0;
    let mut right: ::core::ffi::c_int = 0;
    let mut end: ::core::ffi::c_int = 0;
    let mut status: ::core::ffi::c_int = 0;
    let mut found: ::core::ffi::c_int = 0;
    let mut xoff: ::core::ffi::c_int = 0;
    let mut yoff: ::core::ffi::c_int = 0;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    if wp.is_null() {
        return ::core::ptr::null_mut::<window_pane>();
    }
    w = (*wp).window as *mut window;
    status = window_get_pane_status(w);
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
    next = window_pane_first(w);
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
                    list.push(next);
                }
            }
        }
        next = window_pane_next(next);
    }
    best = window_pane_choose_best(&list);
    return best;
}
pub unsafe fn window_pane_find_left(mut wp: *mut window_pane) -> *mut window_pane {
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut next: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut best: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut list: Vec<*mut window_pane> = Vec::new();
    let mut edge: ::core::ffi::c_int = 0;
    let mut top: ::core::ffi::c_int = 0;
    let mut bottom: ::core::ffi::c_int = 0;
    let mut end: ::core::ffi::c_int = 0;
    let mut found: ::core::ffi::c_int = 0;
    let mut xoff: ::core::ffi::c_int = 0;
    let mut yoff: ::core::ffi::c_int = 0;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    if wp.is_null() {
        return ::core::ptr::null_mut::<window_pane>();
    }
    w = (*wp).window as *mut window;
    window_pane_full_size_offset(wp, &raw mut xoff, &raw mut yoff, &raw mut sx, &raw mut sy);
    edge = xoff;
    if edge == 0 as ::core::ffi::c_int {
        edge = (*w).sx as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
    }
    top = yoff;
    bottom = yoff + sy as ::core::ffi::c_int;
    next = window_pane_first(w);
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
                    list.push(next);
                }
            }
        }
        next = window_pane_next(next);
    }
    best = window_pane_choose_best(&list);
    return best;
}
pub unsafe fn window_pane_find_right(mut wp: *mut window_pane) -> *mut window_pane {
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut next: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut best: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut list: Vec<*mut window_pane> = Vec::new();
    let mut edge: ::core::ffi::c_int = 0;
    let mut top: ::core::ffi::c_int = 0;
    let mut bottom: ::core::ffi::c_int = 0;
    let mut end: ::core::ffi::c_int = 0;
    let mut found: ::core::ffi::c_int = 0;
    let mut xoff: ::core::ffi::c_int = 0;
    let mut yoff: ::core::ffi::c_int = 0;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    if wp.is_null() {
        return ::core::ptr::null_mut::<window_pane>();
    }
    w = (*wp).window as *mut window;
    window_pane_full_size_offset(wp, &raw mut xoff, &raw mut yoff, &raw mut sx, &raw mut sy);
    edge = xoff + sx as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
    if edge >= (*w).sx as ::core::ffi::c_int {
        edge = 0 as ::core::ffi::c_int;
    }
    top = (*wp).yoff;
    bottom = (*wp).yoff + (*wp).sy as ::core::ffi::c_int;
    next = window_pane_first(w);
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
                    list.push(next);
                }
            }
        }
        next = window_pane_next(next);
    }
    best = window_pane_choose_best(&list);
    return best;
}
pub unsafe fn window_pane_stack_push(
    mut stack: *mut window_pane_history,
    mut wp: *mut window_pane,
) {
    if !wp.is_null() {
        window_pane_stack_remove(stack, wp);
        (*stack).push_front(window_pane_weak(wp));
        (*wp).flags |= PANE_VISITED;
    }
}
pub unsafe fn window_pane_stack_remove(
    mut stack: *mut window_pane_history,
    mut wp: *mut window_pane,
) {
    // Cross-window swaps may look for a pane in the destination history while
    // it is still recorded in the source history. Only clear the flag when
    // this history actually contained the pane; membership is authoritative.
    if !wp.is_null() && (*stack).remove_ptr(wp) {
        (*wp).flags &= !PANE_VISITED;
    }
}
pub unsafe fn winlink_clear_flags(mut wl: *mut winlink) {
    let mut loop_0: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let w = (*wl).window_ptr();
    (*w).flags &= !WINDOW_ALERTFLAGS;
    loop_0 = window_winlinks_first(w);
    while !loop_0.is_null() {
        if (*loop_0).flags & WINLINK_ALERTFLAGS != 0 as ::core::ffi::c_int {
            (*loop_0).flags &= !WINLINK_ALERTFLAGS;
            server_status_session((*loop_0).session);
        }
        loop_0 = window_winlinks_next(w, loop_0);
    }
}
pub unsafe fn winlink_shuffle_up(
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
        winlinks_reindex(&raw mut (*s).windows, wl, last);
        last -= 1;
    }
    return idx;
}
unsafe fn window_pane_input_callback(
    cdata: &window_pane_input_data,
    error: ::core::ffi::c_int,
    closed: bool,
    buffer: &mut evbuffer,
) {
    let c = cdata.client.as_ref().expect("pane input client").get();
    let buf = evbuffer_pullup(buffer, -1).map_or(std::ptr::null_mut(), |bytes| bytes.as_mut_ptr());
    let len = evbuffer_get_length(buffer);
    let wp = window_pane_find_by_id(cdata.wp);
    if wp.is_null() {
        (*c).retval = 1;
        (*c).flags |= CLIENT_EXIT as uint64_t;
    }
    let file = cdata.file.upgrade();
    if file.is_some()
        && !closed
        && (wp.is_null() || (*c).flags & CLIENT_DEAD as uint64_t != 0 || error != 0)
    {
        file_cancel(crate::src::shared::rc::as_ptr(file.as_ref().unwrap()));
    } else if file.is_none() || closed || error != 0 {
        cmdq_continue(cdata.item.as_ptr());
    } else {
        input_parse_buffer(wp, buf, len);
    }
    evbuffer_drain(buffer, len);
}

pub unsafe fn window_pane_start_input(
    wp: *mut window_pane,
    item: *mut cmdq_item,
) -> Result<::core::ffi::c_int, std::ffi::CString> {
    let c = cmdq_get_client(item);
    if (*wp).flags & PANE_EMPTY == 0 {
        return Err(c"pane is not empty".to_owned());
    }
    if (*c).flags & (CLIENT_DEAD | CLIENT_EXITED) as uint64_t != 0 {
        return Ok(1);
    }
    if !(*c).session.is_null() {
        return Ok(1);
    }
    // The file initializer publishes its weak identity before dispatch. The
    // callback then owns the box, including its retained client reference.
    let mut cdata = Box::new(window_pane_input_data {
        item: NonNull::new(item).expect("pane input command"),
        client: Some(
            crate::src::shared::rc::downgrade(c)
                .upgrade()
                .expect("live pane input client"),
        ),
        wp: (*wp).id,
        file: Weak::new(),
    });
    file_read_with_cmdq_wait_init(
        c,
        c"-".as_ptr(),
        move |file| {
            cdata.file = file;
            cdata.into_callback()
        },
        item,
        None,
    );
    Ok(0)
}
pub unsafe fn window_pane_get_new_data(
    mut wp: *mut window_pane,
    mut wpo: *mut window_pane_offset,
    mut size: *mut size_t,
) -> *mut ::core::ffi::c_void {
    let mut used: size_t = (*wpo).used.wrapping_sub((*wp).base_offset);
    *size = evbuffer_get_length(&*((*(*wp).event).input)).wrapping_sub(used);
    return evbuffer_pullup(&mut *(*(*wp).event).input, -1)
        .map_or(std::ptr::null_mut(), |bytes| bytes.as_mut_ptr())
        .offset(used as isize) as *mut ::core::ffi::c_void;
}
pub unsafe fn window_pane_update_used_data(
    mut wp: *mut window_pane,
    mut wpo: *mut window_pane_offset,
    mut size: size_t,
) {
    let mut used: size_t = (*wpo).used.wrapping_sub((*wp).base_offset);
    if size > evbuffer_get_length(&*((*(*wp).event).input)).wrapping_sub(used) {
        size = evbuffer_get_length(&*((*(*wp).event).input)).wrapping_sub(used);
    }
    (*wpo).used = (*wpo).used.wrapping_add(size);
}
pub unsafe fn window_pane_default_cursor(mut wp: *mut window_pane) {
    screen_set_default_cursor(&mut *(*wp).screen, options_owner_ptr(&mut (*wp).options));
}
pub unsafe fn window_pane_mode(mut wp: *mut window_pane) -> ::core::ffi::c_int {
    if !(*wp).modes.active.is_null() {
        if std::ptr::eq((*(*wp).modes.active).mode, &window_copy_mode) {
            return 1 as ::core::ffi::c_int;
        }
        if std::ptr::eq((*(*wp).modes.active).mode, &window_view_mode) {
            return 2 as ::core::ffi::c_int;
        }
    }
    return 0 as ::core::ffi::c_int;
}
pub unsafe fn window_pane_show_scrollbar(mut wp: *mut window_pane) -> ::core::ffi::c_int {
    let mut w: *mut window = (*wp).window as *mut window;
    let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
    if (*wp).base.saved_grid.is_some() {
        return 0 as ::core::ffi::c_int;
    }
    if (*w).flags & WINDOW_ZOOMED != 0 && !(*w).active.is_null() {
        wme = (*(*w).active).modes.active;
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
pub unsafe fn window_pane_scrollbar_reserve(mut wp: *mut window_pane) -> ::core::ffi::c_int {
    if window_pane_show_scrollbar(wp) == 0 {
        return 0 as ::core::ffi::c_int;
    }
    return ((*(*wp).window).sb == PANE_SCROLLBARS_ALWAYS) as ::core::ffi::c_int;
}
pub unsafe fn window_pane_scrollbar_overlay(mut wp: *mut window_pane) -> ::core::ffi::c_int {
    if window_pane_show_scrollbar(wp) == 0 {
        return 0 as ::core::ffi::c_int;
    }
    return window_pane_scrollbar_auto_hide(wp);
}
pub unsafe fn window_pane_scrollbar_visible(mut wp: *mut window_pane) -> ::core::ffi::c_int {
    if window_pane_show_scrollbar(wp) == 0 {
        return 0 as ::core::ffi::c_int;
    }
    if window_pane_scrollbar_auto_hide(wp) == 0 {
        return 1 as ::core::ffi::c_int;
    }
    return (*wp).sb_auto_visible;
}
pub unsafe fn window_pane_scrollbar_start_timer(mut wp: *mut window_pane) {
    let mut tv: timeval = timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    let mut delay: u_int = 0;
    if window_pane_scrollbar_auto_hide(wp) == 0 || (*wp).sb_auto_visible == 0 {
        return;
    }
    delay = options_get_number(
        options_owner_ptr(&mut (*(*wp).window).options),
        b"pane-scrollbars-timeout\0" as *const u8 as *const ::core::ffi::c_char,
    ) as u_int;
    tv.tv_sec = delay.wrapping_div(1000 as u_int) as __time_t;
    tv.tv_usec = (delay.wrapping_rem(1000 as u_int) as ::core::ffi::c_long
        * 1000 as ::core::ffi::c_long) as __suseconds_t;
    event_del(&raw mut (*wp).sb_auto_timer);
    event_add(&raw mut (*wp).sb_auto_timer, &raw mut tv);
}
pub unsafe fn window_pane_scrollbar_show(mut wp: *mut window_pane) {
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

    window_pane_scrollbar_start_timer(wp);

    if changed != 0 {
        window_pane_scrollbar_redraw_visibility(wp);
    }
}
pub unsafe fn window_pane_scrollbar_hide(mut wp: *mut window_pane) {
    if event_initialized(&(*wp).sb_auto_timer) != 0 {
        event_del(&raw mut (*wp).sb_auto_timer);
    }
    (*wp).sb_auto_hover = 0 as ::core::ffi::c_int;
    if (*wp).sb_auto_visible == 0 {
        return;
    }
    (*wp).sb_auto_visible = 0 as ::core::ffi::c_int;
    window_pane_scrollbar_redraw_visibility(wp);
}
pub unsafe fn window_pane_get_bg(mut wp: *mut window_pane) -> ::core::ffi::c_int {
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
        defaults = tty_default_colours(wp).0;
        if defaults.bg == 8 as ::core::ffi::c_int || defaults.bg == 9 as ::core::ffi::c_int {
            c = window_get_bg_client(wp);
        } else {
            c = defaults.bg;
        }
    }
    return c;
}
pub unsafe fn window_get_bg_client(mut wp: *mut window_pane) -> ::core::ffi::c_int {
    let mut w: *mut window = (*wp).window as *mut window;
    let mut loop_0: *mut client = ::core::ptr::null_mut::<client>();
    loop_0 = clients.first();
    while !loop_0.is_null() {
        if !((*loop_0).flags & CLIENT_UNATTACHEDFLAGS as uint64_t != 0) {
            if !((*loop_0).session.is_null() || session_has((*loop_0).session, w) == 0) {
                if !((*loop_0).tty.bg == -(1 as ::core::ffi::c_int)) {
                    return (*loop_0).tty.bg;
                }
            }
        }
        loop_0 = clients.next(loop_0);
    }
    return -(1 as ::core::ffi::c_int);
}
pub unsafe fn window_pane_get_bg_control_client(mut wp: *mut window_pane) -> ::core::ffi::c_int {
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    if (*wp).control_bg == -(1 as ::core::ffi::c_int) {
        return -(1 as ::core::ffi::c_int);
    }
    c = clients.first();
    while !c.is_null() {
        if (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
            return (*wp).control_bg;
        }
        c = clients.next(c);
    }
    return -(1 as ::core::ffi::c_int);
}
pub unsafe fn window_pane_get_fg(mut wp: *mut window_pane) -> ::core::ffi::c_int {
    let mut w: *mut window = (*wp).window as *mut window;
    let mut loop_0: *mut client = ::core::ptr::null_mut::<client>();
    loop_0 = clients.first();
    while !loop_0.is_null() {
        if !((*loop_0).flags & CLIENT_UNATTACHEDFLAGS as uint64_t != 0) {
            if !((*loop_0).session.is_null() || session_has((*loop_0).session, w) == 0) {
                if !((*loop_0).tty.fg == -(1 as ::core::ffi::c_int)) {
                    return (*loop_0).tty.fg;
                }
            }
        }
        loop_0 = clients.next(loop_0);
    }
    return -(1 as ::core::ffi::c_int);
}
pub unsafe fn window_pane_get_fg_control_client(mut wp: *mut window_pane) -> ::core::ffi::c_int {
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    if (*wp).control_fg == -(1 as ::core::ffi::c_int) {
        return -(1 as ::core::ffi::c_int);
    }
    c = clients.first();
    while !c.is_null() {
        if (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
            return (*wp).control_fg;
        }
        c = clients.next(c);
    }
    return -(1 as ::core::ffi::c_int);
}
pub unsafe fn window_pane_get_theme(mut wp: *mut window_pane) -> client_theme {
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut loop_0: *mut client = ::core::ptr::null_mut::<client>();
    let mut found_light: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut found_dark: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if wp.is_null() {
        return THEME_UNKNOWN;
    }
    w = (*wp).window as *mut window;
    loop_0 = clients.first();
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
        loop_0 = clients.next(loop_0);
    }
    if found_dark != 0 && found_light == 0 {
        return THEME_DARK;
    }
    if found_light != 0 && found_dark == 0 {
        return THEME_LIGHT;
    }
    return colour_totheme(window_pane_get_bg(wp));
}
pub unsafe fn window_pane_send_theme_update(mut wp: *mut window_pane) {
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
            log_debug(format_args!(
                "{}: %{} light theme",
                "window_pane_send_theme_update",
                ((*wp).id) as u32
            ));
            bufferevent_write(
                (*wp).event,
                b"\x1B[?997;2n\0" as *const u8 as *const ::core::ffi::c_char
                    as *const ::core::ffi::c_void,
                9 as size_t,
            );
        }
        2 => {
            log_debug(format_args!(
                "{}: %{} dark theme",
                "window_pane_send_theme_update",
                ((*wp).id) as u32
            ));
            bufferevent_write(
                (*wp).event,
                b"\x1B[?997;1n\0" as *const u8 as *const ::core::ffi::c_char
                    as *const ::core::ffi::c_void,
                9 as size_t,
            );
        }
        0 => {
            log_debug(format_args!(
                "{}: %{} unknown theme",
                "window_pane_send_theme_update",
                ((*wp).id) as u32
            ));
        }
        _ => {}
    };
}
pub unsafe fn window_pane_status_get_range(
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
pub unsafe fn window_get_pane_lines(mut w: *mut window) -> pane_lines {
    let mut oo: *mut options = ::core::ptr::null_mut::<options>();
    oo = options_owner_ptr(&mut (*w).options);
    return options_get_number(
        oo,
        b"pane-border-lines\0" as *const u8 as *const ::core::ffi::c_char,
    ) as pane_lines;
}
pub unsafe fn window_pane_get_pane_lines(mut wp: *mut window_pane) -> pane_lines {
    let mut oo: *mut options = ::core::ptr::null_mut::<options>();
    if window_pane_is_floating(wp) == 0 {
        oo = options_owner_ptr(&mut (*(*wp).window).options);
    } else {
        oo = options_owner_ptr(&mut (*wp).options);
    }
    return options_get_number(
        oo,
        b"pane-border-lines\0" as *const u8 as *const ::core::ffi::c_char,
    ) as pane_lines;
}
pub unsafe fn window_get_pane_status(mut w: *mut window) -> ::core::ffi::c_int {
    let mut status: ::core::ffi::c_int = 0;
    status = options_get_number(
        options_owner_ptr(&mut (*w).options),
        b"pane-border-status\0" as *const u8 as *const ::core::ffi::c_char,
    ) as ::core::ffi::c_int;
    if status == PANE_STATUS_TOP_FLOATING || status == PANE_STATUS_BOTTOM_FLOATING {
        return 0 as ::core::ffi::c_int;
    }
    return status;
}
pub unsafe fn window_pane_get_pane_status(mut wp: *mut window_pane) -> ::core::ffi::c_int {
    let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
    let mut status: ::core::ffi::c_int = 0;
    wme = (*wp).modes.active;
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
        options_owner_ptr(&mut (*wp).options),
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
pub unsafe fn window_pane_is_floating(mut wp: *mut window_pane) -> ::core::ffi::c_int {
    let mut lc: *mut layout_cell = (*wp).layout_cell as *mut layout_cell;
    if lc.is_null() || (*lc).flags & LAYOUT_CELL_FLOATING == 0 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    return 1 as ::core::ffi::c_int;
}

#[cfg(test)]
mod collection_index_tests {
    use super::*;

    #[test]
    fn pane_rc_keeps_weak_observers_alive_until_the_last_release() {
        unsafe {
            // No display resources in this fixture; exercise the actual pane
            // retain/release functions with ordinary field drop.
            let wp = crate::src::shared::rc::new(window_pane::empty());
            let weak = window_pane_weak(wp);
            window_pane_add_ref(wp, c"callback".as_ptr());
            window_pane_remove_ref(wp, c"pane shutdown".as_ptr());
            let retained = weak.upgrade().expect("callback keeps the pane alive");
            assert_eq!(crate::src::shared::rc::as_ptr(&retained), wp);
            window_pane_remove_ref(wp, c"callback complete".as_ptr());
            assert!(
                weak.upgrade().is_some(),
                "upgraded Rc independently owns the pane"
            );
            drop(retained);
            assert!(weak.upgrade().is_none());
        }
    }

    #[test]
    fn winlink_index_keeps_order_and_weak_observers_across_moves() {
        unsafe {
            let mut head = winlinks { storage: None };
            let first = winlink_add(&mut head, 1);
            let second = winlink_add(&mut head, 3);
            assert!(!first.is_null() && !second.is_null());
            assert!(winlink_add(&mut head, 1).is_null());
            winlinks_reindex(&mut head, second, 2);

            let index_observer = (*first).entry.owner.as_ref().unwrap().clone();
            let node_observer = winlink_weak(first);
            let mut moved = head;
            assert_eq!(winlinks_minmax(&moved, RB_NEGINF), first);
            assert_eq!(winlinks_next(&*first), second);
            assert_eq!((*second).idx, 2);

            winlink_remove(&mut moved, first);
            assert!(matches!(
                node_observer.try_access_mut(|link| link.idx),
                Err(refbox::BorrowError::Dropped)
            ));
            assert!(index_observer.try_borrow_mut().is_ok());
            let second_observer = winlink_weak(second);
            winlink_remove(&mut moved, second);
            drop(moved);
            assert!(matches!(
                second_observer.try_access_mut(|link| link.idx),
                Err(refbox::BorrowError::Dropped)
            ));
            assert!(matches!(
                index_observer.try_borrow_mut(),
                Err(refbox::BorrowError::Dropped)
            ));
        }
    }

    #[test]
    fn pane_index_observers_clear_on_removal_and_expire_with_the_owner() {
        unsafe {
            let mut head = window_pane_tree { storage: None };
            let mut other = window_pane_tree { storage: None };
            let first_owner =
                crate::src::shared::rc::take(crate::src::shared::rc::new(window_pane::empty()));
            let first = crate::src::shared::rc::as_ptr(&first_owner);
            (*first).id = 1;
            let second_owner =
                crate::src::shared::rc::take(crate::src::shared::rc::new(window_pane::empty()));
            let second = crate::src::shared::rc::as_ptr(&second_owner);
            (*second).id = 2;
            let duplicate_owner =
                crate::src::shared::rc::take(crate::src::shared::rc::new(window_pane::empty()));
            let duplicate = crate::src::shared::rc::as_ptr(&duplicate_owner);
            (*duplicate).id = 1;

            assert!(window_pane_tree_insert(&mut head, first).is_null());
            assert!(window_pane_tree_insert(&mut head, second).is_null());
            let index_observer = (*first).tree_entry.owner.as_ref().unwrap().clone();
            assert_eq!(window_pane_tree_insert(&mut head, duplicate), first);
            assert!((*duplicate).tree_entry.owner.is_none());
            assert!(window_pane_tree_remove(&mut other, first).is_null());
            assert!((*first).tree_entry.owner.is_some());

            let mut moved = head;
            assert_eq!(window_pane_tree_next(&*first), second);
            assert_eq!(window_pane_tree_remove(&mut moved, first), first);
            assert!((*first).tree_entry.owner.is_none());
            assert!(window_pane_tree_next(&*first).is_null());
            assert_eq!(window_pane_tree_remove(&mut moved, second), second);

            drop(first_owner);
            drop(second_owner);
            drop(duplicate_owner);
            drop(moved);
            assert!(matches!(
                index_observer.try_borrow_mut(),
                Err(refbox::BorrowError::Dropped)
            ));
        }
    }
}

impl Drop for window {
    fn drop(&mut self) {
        unsafe { window_destroy(self) }
    }
}

impl Drop for window_pane {
    fn drop(&mut self) {
        unsafe { window_pane_free(self) }
    }
}

#[cfg(test)]
mod pane_prompt_data_tests {
    use super::*;
    use crate::src::shared::rc;
    use crate::src::text::utf8::utf8_fromcstr_vec;
    use std::cell::Cell;

    fn data(wp_id: u_int) -> WindowPanePromptOwner {
        refbox::RefBox::new(window_pane_prompt {
            wp_id,
            c: Weak::new(),
            inputcb: None,
            freecb: None,
            type_0: PROMPT_TYPE_COMMAND,
        })
    }

    fn attach(data: WindowPanePromptOwner) -> crate::src::shared::prompt::PromptOwner {
        let pr = refbox::RefBox::new(prompt {
            flags: PROMPT_SINGLE,
            buffer: utf8_fromcstr_vec(c""),
            ..Default::default()
        });
        let input = data.downgrade();
        pr.try_borrow_mut().unwrap().inputcb = Some(Box::new(move |text, key| {
            window_pane_prompt_input_callback(&input, text, key)
        }));
        let freed = data;
        pr.try_borrow_mut().unwrap().freecb = Some(Box::new(move || unsafe {
            window_pane_prompt_free_callback(&freed)
        }));
        pr
    }

    #[test]
    fn callback_client_is_weak_between_calls_and_retained_during_dispatch() {
        unsafe {
            let client = client::new();
            let weak_client = Rc::downgrade(&client);
            let pointer = rc::as_ptr(&client);
            let client_slot = Rc::new(RefCell::new(Some(client)));
            let data = data(u_int::MAX);
            data.try_borrow_mut().unwrap().c = weak_client.clone();
            let weak_data = data.downgrade();
            let calls = Rc::new(Cell::new(0));
            let count = calls.clone();
            let slot = client_slot.clone();
            let current = weak_data.clone();
            let observed = weak_client.clone();
            data.try_borrow_mut().unwrap().inputcb = Some(Box::new(move |client, text, key| {
                assert_eq!(text, Some(c"input"));
                assert_eq!(key, PROMPT_KEY_HANDLED);
                if count.get() == 0 {
                    assert_eq!(client.unwrap().as_ptr(), pointer);
                    drop(slot.borrow_mut().take());
                    assert!(observed.upgrade().is_some());
                    // Dispatch must release the data borrow before callbacks.
                    current.try_borrow_mut().unwrap().c = Weak::new();
                } else {
                    assert!(client.is_none());
                }
                count.set(count.get() + 1);
                PROMPT_CONTINUE
            }));
            assert_eq!(
                window_pane_prompt_input_callback(&weak_data, Some(c"input"), PROMPT_KEY_HANDLED),
                PROMPT_CONTINUE
            );
            assert!(weak_client.upgrade().is_none());
            assert!(client_slot.borrow().is_none());
            assert_eq!(
                window_pane_prompt_input_callback(&weak_data, Some(c"input"), PROMPT_KEY_HANDLED),
                PROMPT_CONTINUE
            );
            assert_eq!(calls.get(), 2);
            let frees = Rc::new(Cell::new(0));
            let freed = frees.clone();
            data.try_borrow_mut().unwrap().freecb =
                Some(Box::new(move || freed.set(freed.get() + 1)));
            window_pane_prompt_free_callback(&data);
            window_pane_prompt_free_callback(&data);
            assert_eq!(frees.get(), 1);
            assert!(data.try_borrow_mut().unwrap().inputcb.is_none());
            drop(data);
            assert!(!weak_data.is_alive());
            assert_eq!(
                window_pane_prompt_input_callback(&weak_data, None, PROMPT_KEY_CLOSE),
                PROMPT_CLOSE,
            );
            assert_eq!(Rc::strong_count(&calls), 1);
        }
    }

    #[test]
    fn callback_replacement_keeps_new_pane_data_and_releases_the_old_record() {
        unsafe {
            let pane = rc::take(rc::new(window_pane::empty()));
            let wp = rc::as_ptr(&pane);
            (*wp).id = u_int::MAX - 1;
            // This fixture exercises cleanup without firing pane hook events.
            (*wp).flags = PANE_DESTROYED;
            assert!(window_pane_tree_insert(&raw mut all_window_panes, wp).is_null());
            let old_data = data((*wp).id);
            let old_weak = old_data.downgrade();
            let old_prompt = attach(old_data);
            let old_data = old_weak.clone();
            let replacement = data((*wp).id);
            let replacement_weak = replacement.downgrade();
            let replacement_prompt = attach(replacement);
            let replacement = replacement_weak.clone();
            let next_data = replacement.clone();
            let replacement_observer = replacement_prompt.downgrade();
            let mut next_prompt = Some(replacement_prompt);
            old_data.try_borrow_mut().unwrap().inputcb = Some(Box::new(move |_, text, kind| {
                assert_eq!(text, Some(c"x"));
                assert_eq!(kind, PROMPT_KEY_CLOSE);
                window_pane_clear_prompt(wp);
                (*wp).prompt = next_prompt.take();
                (*wp).prompt_data = Some(next_data.clone());
                PROMPT_CLOSE
            }));
            let frees = Rc::new(Cell::new(0));
            let count = frees.clone();
            old_data.try_borrow_mut().unwrap().freecb =
                Some(Box::new(move || count.set(count.get() + 1)));
            let old_observer = old_prompt.downgrade();
            (*wp).prompt = Some(old_prompt);
            (*wp).prompt_data = Some(old_weak.clone());
            assert_eq!(
                window_pane_prompt_key(
                    wp,
                    std::ptr::null_mut(),
                    b'x' as key_code,
                    std::ptr::null_mut()
                ),
                PROMPT_KEY_CLOSE
            );
            assert_eq!(frees.get(), 1);
            assert!((*wp).prompt_data.as_ref() == Some(&replacement_weak));
            assert!(replacement_observer.is((*wp).prompt.as_ref().unwrap()));
            assert!(!old_observer.is_alive());
            drop(old_data);
            assert!(!old_weak.is_alive());
            window_pane_clear_prompt(wp);
            assert!((*wp).prompt_data.is_none());
            assert!(!replacement_observer.is_alive());
            drop(replacement);
            assert!(!replacement_weak.is_alive());
            assert_eq!(window_pane_tree_remove(&raw mut all_window_panes, wp), wp);
        }
    }
}

#[cfg(test)]
mod pane_input_owner_tests {
    use super::*;
    use crate::src::file::{file_fire_done, file_fire_read};
    use crate::src::reactor::{evbuffer_add, event_loop, shutdown_runtime};
    use crate::src::shared::command::CMDQ_WAITING;
    use crate::src::shared::rc;

    unsafe fn waiting_input(
        item: &mut cmdq_item,
        client: &Rc<UnsafeCell<client>>,
    ) -> *mut client_file {
        let mut data = Box::new(window_pane_input_data {
            item: NonNull::from(&mut *item),
            client: Some(Rc::clone(client)),
            wp: u_int::MAX,
            file: Weak::new(),
        });
        let file = rc::new(client_file::empty());
        (*file).wait_item = item;
        (*file).wait_client = Some(Rc::downgrade(client));
        data.file = rc::downgrade(file);
        (*file).cb = data.into_callback();
        item.wait_file = Some(rc::downgrade(file));
        item.flags = CMDQ_WAITING;
        file
    }

    #[test]
    fn progress_cancellation_keeps_record_and_client_until_terminal_dispatch() {
        unsafe {
            let client = client::new();
            let client_observer = Rc::downgrade(&client);
            let mut item = cmdq_item::empty();
            let file = waiting_input(&mut item, &client);
            let file_observer = rc::downgrade(file);
            // Cancellation was already requested. Further progress must drain
            // the bytes and keep waiting until the peer sends its terminal event.
            (*file).closed = 1;
            evbuffer_add(&mut (*file).buffer, b"discard".as_ptr().cast(), 7);
            drop(client);

            file_fire_read(file);
            assert_eq!(evbuffer_get_length(&(*file).buffer), 0);
            assert_ne!(item.flags & CMDQ_WAITING, 0);
            assert!((*file).cb.is_some());
            assert_eq!(
                rc::strong_count(file),
                1,
                "the callback only weakly observes its file"
            );
            {
                let client = client_observer.upgrade().unwrap();
                assert_eq!((*client.get()).retval, 1);
                assert_ne!((*client.get()).flags & CLIENT_EXIT as u64, 0);
            }

            file_fire_done(file);
            file_fire_done(file);
            assert!((*file).cb.is_some());
            event_loop();
            assert_eq!(item.flags & CMDQ_WAITING, 0);
            assert!(item.wait_file.is_none());
            assert!(file_observer.upgrade().is_none());
            assert!(client_observer.upgrade().is_none());
            shutdown_runtime();
        }
    }

    #[test]
    fn failed_input_startup_transfers_box_before_completion_or_cancellation() {
        unsafe {
            for cancel in [false, true] {
                let owner = client::new();
                let client = rc::as_ptr(&owner);
                let observer = Rc::downgrade(&owner);
                // Control clients cannot read stdin. Opening schedules a terminal
                // error callback after the initializer has installed its box.
                (*client).flags = CLIENT_CONTROL as u64;
                let mut item = cmdq_item::empty();
                item.client = client;
                item.flags = CMDQ_WAITING;
                let mut pane = window_pane::empty();
                pane.flags = PANE_EMPTY;
                pane.id = u_int::MAX;
                assert_eq!(window_pane_start_input(&mut pane, &mut item), Ok(0));
                let file_owner = item.wait_file.as_ref().unwrap().upgrade().unwrap();
                let file = rc::as_ptr(&file_owner);
                assert!(!file.is_null());
                assert!((*file).cb.is_some());
                let file_observer = rc::downgrade(file);
                drop(owner);
                if cancel {
                    crate::src::file::file_cancel_cmdq_wait(file);
                    assert!((*file).cb.is_none());
                }
                drop(file_owner);
                event_loop();
                assert!(item.wait_file.is_none());
                if !cancel {
                    assert_eq!(item.flags & CMDQ_WAITING, 0);
                }
                assert!(file_observer.upgrade().is_none());
                assert!(observer.upgrade().is_none());
                shutdown_runtime();
            }
        }
    }

    #[test]
    fn failed_open_and_dead_client_release_callback_records_once() {
        unsafe {
            for dead in [false, true] {
                let client = client::new();
                let client_observer = Rc::downgrade(&client);
                if dead {
                    (*client.get()).flags |= CLIENT_DEAD as u64;
                }
                let mut item = cmdq_item::empty();
                let file = waiting_input(&mut item, &client);
                let file_observer = rc::downgrade(file);
                (*file).error = libc::EBADF;
                    drop(client);
                file_fire_done(file);
                file_fire_done(file);
                event_loop();
                assert_eq!(item.flags & CMDQ_WAITING != 0, dead);
                assert!(item.wait_file.is_none());
                assert!(file_observer.upgrade().is_none());
                assert!(client_observer.upgrade().is_none());
                shutdown_runtime();
            }
        }
    }
}

#[cfg(test)]
mod pane_stream_lifecycle_tests {
    use super::*;
    use crate::src::reactor::shutdown_runtime;
    use crate::src::shared::rc;

    #[test]
    fn pane_observers_expire_on_logical_destruction_before_the_last_guard() {
        unsafe {
            let pane = rc::new(window_pane::empty());
            (*pane).fd = -1;
            (*pane).pipe_fd = -1;
            let observer = window_pane_weak(pane);
            assert_eq!(rc::strong_count(pane), 1);
            let guard = window_pane_upgrade(&observer).unwrap();
            assert_eq!(rc::as_ptr(&guard), pane);

            window_pane_destroy(pane);

            // An in-flight operation can still hold the allocation, but new
            // operations must not follow a mode's link into a destroyed pane.
            assert!(observer.upgrade().is_some());
            assert!(window_pane_upgrade(&observer).is_none());
            assert_ne!((*rc::as_ptr(&guard)).flags & PANE_DESTROYED, 0);
            drop(guard);
            assert!(observer.upgrade().is_none());
            assert!(window_pane_upgrade(&observer).is_none());
        }
    }

    #[test]
    fn destroying_an_empty_pane_releases_its_stream_callbacks() {
        unsafe {
            let pane = rc::new(window_pane::empty());
            let observer = window_pane_weak(pane);
            (*pane).fd = -1;
            (*pane).pipe_fd = -1;
            (*pane).flags = PANE_EMPTY;
            window_pane_set_event(pane);
            assert!((*pane).ictx.is_some());
            let callback = Rc::downgrade((*(*pane).event).readcb.as_ref().unwrap());

            window_pane_destroy(pane);

            assert!(observer.upgrade().is_none());
            assert!(callback.upgrade().is_none());
            shutdown_runtime();
        }
    }
}

#[cfg(test)]
mod zoom_teardown_tests {
    use super::*;
    use crate::src::events::{events_add_sink, events_remove_sink};
    use crate::src::events_payload::event_payload_get_window;
    use crate::src::grid::grid_create;
    use crate::src::layout::{layout_create_cell, layout_make_leaf, layout_set_size};
    use crate::src::options::{options_create, options_default};
    use crate::src::options_table::options_table;
    use crate::src::shared::events::events_callback;
    use crate::src::shared::rc;
    use std::cell::Cell;

    unsafe fn zoomed_window() -> *mut window {
        let w = rc::new(window::default());
        (*w).options = Some(crate::src::options::options_create_owned(std::ptr::null_mut()));
        let entry = options_table
            .iter()
            .find(|entry| {
                entry.name == Some(c"pane-border-status")
            })
            .unwrap();
        options_default(options_owner_ptr(&mut (*w).options), entry);
        let pane = rc::new(window_pane::empty());
        (*pane).window = w;
        (*pane).fd = -1;
        (*pane).pipe_fd = -1;
        (*pane).sx = 80;
        (*pane).sy = 24;
        (*pane).base.grid = Some(grid_create(80, 24, 0));
        (*pane).screen = &raw mut (*pane).base;
        (*pane).flags = PANE_ZOOMED;
        (*w).active = pane;
        (*w).panes.push_back(rc::downgrade(pane));
        (*w).z_index.push_back(rc::downgrade(pane));
        let mut saved = layout_create_cell();
        layout_set_size(&mut *saved, 40, 24, 0, 0);
        layout_make_leaf(&mut *saved, pane);
        (*pane).saved_layout_cell = &mut *saved;
        let mut zoomed = layout_create_cell();
        layout_set_size(&mut *zoomed, 80, 24, 0, 0);
        layout_make_leaf(&mut *zoomed, pane);
        (*w).layout_root = Some(zoomed);
        (*w).saved_layout_root = Some(saved);
        (*w).flags = WINDOW_ZOOMED;
        w
    }

    #[test]
    fn mode_tree_cleanup_unzooms_a_logically_destroyed_pane() {
        unsafe {
            let w = zoomed_window();
            let pane = (*w).active;
            let tree_owner = std::rc::Rc::new(std::cell::UnsafeCell::new(crate::src::shared::mode_tree::mode_tree_data {
                wp: window_pane_weak(pane),
                zoomed: 0,
                ..Default::default()
            }));
            let tree = rc::as_ptr(&tree_owner);
            let observed = rc::downgrade(tree);
            (*pane).flags |= PANE_DESTROYED;
            assert!(window_pane_upgrade(&(*tree).wp).is_none());

            crate::src::mode_tree::mode_tree_free(tree_owner);

            assert!(observed.upgrade().is_none());
            assert_eq!((*w).flags & WINDOW_ZOOMED, 0);
            drop(rc::take(w));
        }
    }

    #[test]
    fn raw_and_typed_final_owners_destroy_zoomed_windows_without_resize_events() {
        unsafe {
            for typed in [false, true] {
                let w = zoomed_window();
                let observer = rc::downgrade(w);
                let pane_observer = rc::downgrade((*w).active);
                let resized = Rc::new(Cell::new(0));
                let resize_count = resized.clone();
                let resize_sink = events_add_sink(
                    c"pane-resized",
                    events_callback(move |_, _| {
                        resize_count.set(resize_count.get() + 1);
                    }),
                );
                let closed = Rc::new(Cell::new(0));
                let close_count = closed.clone();
                let live = observer.clone();
                let close_sink = events_add_sink(
                    c"window-closed",
                    events_callback(move |_, payload| {
                        assert!(live.upgrade().is_some());
                        assert_ne!(
                            (*event_payload_get_window(payload)).flags & WINDOW_ZOOMED,
                            0
                        );
                        close_count.set(close_count.get() + 1);
                    }),
                );
                if typed {
                    drop(rc::take(w));
                } else {
                    window_remove_ref(w, c"test final raw owner".as_ptr());
                }
                assert_eq!(resized.get(), 0);
                assert_eq!(closed.get(), usize::from(!typed));
                assert!(observer.upgrade().is_none());
                assert!(pane_observer.upgrade().is_none());
                events_remove_sink(resize_sink);
                events_remove_sink(close_sink);
            }
        }
    }

    #[test]
    fn live_unzoom_keeps_pane_resize_and_window_notifications() {
        unsafe {
            let w = zoomed_window();
            let pane = (*w).active;
            let notifications = Rc::new(RefCell::new(Vec::new()));
            let mut sinks = Vec::new();
            for name in [c"pane-resized", c"window-unzoomed", c"window-closed"] {
                let notifications = notifications.clone();
                sinks.push(events_add_sink(
                    name,
                    events_callback(move |_, _| {
                        notifications.borrow_mut().push(name);
                    }),
                ));
            }
            assert_eq!(window_unzoom(w, 1), 0);
            assert_eq!(((*pane).sx, (*pane).sy), (40, 24));
            assert!((*w).saved_layout_root.is_none());
            window_remove_ref(w, c"test live close".as_ptr());
            assert_eq!(
                *notifications.borrow(),
                [c"pane-resized", c"window-unzoomed", c"window-closed"]
            );
            for sink in sinks {
                events_remove_sink(sink);
            }
        }
    }
}
