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
use crate::src::options::{options_create, options_free, options_get_number, options_get_number_ref};
use crate::src::prompt::{
    prompt_closed, prompt_create, prompt_free, prompt_incremental_start, prompt_key, prompt_mouse,
    prompt_set_options, prompt_type_string, prompt_update,
};
use crate::src::reactor::{
    bufferevent_disable, bufferevent_enable, bufferevent_new, bufferevent_write,
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
use crate::src::shared::window::WinlinkIdentity;

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
    window_pane_tree_entry, window_panes, PaneScreenSource,
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
pub fn windows_find(
    head: &windows,
    elm: &window,
) -> Option<crate::src::shared::window::WindowOwner> {
    let owner = head.storage.as_ref()?;
    let map = owner
        .try_borrow_mut()
        .expect("window index already borrowed");
    map.get(&elm.id)?
        .upgrade()
        .map(crate::src::shared::window::WindowOwner::adopt)
}

/// Register an observer, without adding a strong reference to the window.
pub unsafe fn windows_insert(
    head: *mut windows,
    window: &Rc<std::cell::UnsafeCell<window>>,
) -> Option<crate::src::shared::window::WindowOwner> {
    let elm = window.get();
    let owner = (*head).storage.get_or_insert_with(refbox::RefBox::default);
    let observer = owner.downgrade();
    let mut map = owner
        .try_borrow_mut()
        .expect("window index already borrowed");
    if let Some(existing) = map.get(&(*elm).id).and_then(std::rc::Weak::upgrade) {
        return Some(crate::src::shared::window::WindowOwner::adopt(existing));
    }
    map.insert((*elm).id, Rc::downgrade(window));
    (*elm).entry.owner = Some(observer);
    None
}

pub unsafe fn windows_remove(head: *mut windows, elm: *mut window) -> *mut window {
    if elm.is_null() {
        return std::ptr::null_mut();
    }
    let Some(owner) = (*head).storage.as_ref() else {
        return std::ptr::null_mut();
    };
    let empty = {
        let mut map = owner
            .try_borrow_mut()
            .expect("window index already borrowed");
        // Removal also runs during the final Rc drop, when upgrade must fail.
        // Compare identity without dereferencing the Weak pointer.
        if !map
            .get(&(*elm).id)
            .is_some_and(|weak| weak.as_ptr().cast::<window>() == elm)
        {
            return std::ptr::null_mut();
        }
        map.remove(&(*elm).id);
        map.is_empty()
    };
    (*elm).entry.owner = None;
    if empty {
        (*head).storage = None;
    }
    elm
}

pub fn windows_minmax(head: &windows) -> Option<crate::src::shared::window::WindowOwner> {
    let owner = head.storage.as_ref()?;
    let map = owner
        .try_borrow_mut()
        .expect("window index already borrowed");
    map.values()
        .find_map(std::rc::Weak::upgrade)
        .map(crate::src::shared::window::WindowOwner::adopt)
}

pub fn windows_next(elm: &window) -> Option<crate::src::shared::window::WindowOwner> {
    let owner = elm.entry.owner.as_ref()?;
    let map = match owner.try_borrow_mut() {
        Ok(map) => map,
        Err(refbox::BorrowError::Dropped) => return None,
        Err(refbox::BorrowError::Borrowed) => panic!("window index already borrowed"),
    };
    map.range((
        std::ops::Bound::Excluded(&elm.id),
        std::ops::Bound::Unbounded,
    ))
    .find_map(|(_, weak)| weak.upgrade())
    .map(crate::src::shared::window::WindowOwner::adopt)
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
        .and_then(|links| links.ordered.first())
        .filter(|link| link.is_alive())
        .map_or(std::ptr::null_mut(), |link| link.as_ptr().cast_mut())
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
        .get(&WinlinkIdentity::of(wl))
        .and_then(|&position| links.ordered.get(position + 1))
        .filter(|link| link.is_alive())
        .map_or(std::ptr::null_mut(), |link| link.as_ptr().cast_mut())
}

/// Append a non-owning winlink handle to the window's association order.
pub unsafe fn window_winlinks_append(w: *mut window, wl: *mut winlink) {
    assert!(!w.is_null() && !wl.is_null());
    let links = (*w).winlinks.storage.get_or_insert_with(|| Box::default());
    assert!(
        !links.positions.contains_key(&WinlinkIdentity::of(wl)),
        "winlink is already present in this window"
    );
    let position = links.ordered.len();
    links.ordered.push((*wl).observer.clone());
    links.positions.insert(WinlinkIdentity::of(wl), position);
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
        .remove(&WinlinkIdentity::of(wl))
        .expect("winlink must belong to its window");
    links.ordered.remove(position);
    for (position, link) in links.ordered.iter().enumerate().skip(position) {
        links.positions.insert(WinlinkIdentity::of(link.as_ptr()), position);
    }
    if links.ordered.is_empty() {
        (*w).winlinks.storage = None;
    }
}

pub fn window_pane_tree_find(head: &window_pane_tree, id: u_int) -> Option<Rc<std::cell::UnsafeCell<window_pane>>> {
    let index = head.storage.as_ref()?;
    let map = index.try_borrow_mut().expect("pane index already borrowed");
    map.get(&id).cloned()
}
pub unsafe fn window_pane_tree_insert(
    head: &mut window_pane_tree,
    pane: Rc<std::cell::UnsafeCell<window_pane>>,
) -> Option<Rc<std::cell::UnsafeCell<window_pane>>> {
    let node = &mut *pane.get();
    let index = head.storage.get_or_insert_with(refbox::RefBox::default);
    let observer = index.downgrade();
    let mut map = index.try_borrow_mut().expect("pane index already borrowed");
    match map.entry(node.id) {
        std::collections::btree_map::Entry::Occupied(entry) => Some(entry.get().clone()),
        std::collections::btree_map::Entry::Vacant(entry) => {
            node.tree_entry.owner = Some(observer);
            entry.insert(pane);
            None
        }
    }
}
pub fn window_pane_tree_remove(
    head: &mut window_pane_tree,
    pane: &mut window_pane,
) -> Option<Rc<std::cell::UnsafeCell<window_pane>>> {
    let index = head.storage.as_ref()?;
    let (owner, empty) = {
        let mut map = index.try_borrow_mut().expect("pane index already borrowed");
        if !map.get(&pane.id).is_some_and(|owner| Rc::downgrade(owner).ptr_eq(&pane.observer)) {
            return None;
        }
        (map.remove(&pane.id).expect("matching pane"), map.is_empty())
    };
    pane.tree_entry.owner = None;
    if empty {
        head.storage = None;
    }
    Some(owner)
}
pub fn window_pane_tree_minmax(head: &window_pane_tree) -> Option<Rc<std::cell::UnsafeCell<window_pane>>> {
    let index = head.storage.as_ref()?;
    let map = index.try_borrow_mut().expect("pane index already borrowed");
    map.first_key_value().map(|(_, owner)| owner.clone())
}
pub fn window_pane_tree_next(pane: &window_pane) -> Option<Rc<std::cell::UnsafeCell<window_pane>>> {
    let index = pane.tree_entry.owner.as_ref()?;
    let map = match index.try_borrow_mut() {
        Ok(map) => map,
        Err(refbox::BorrowError::Dropped) => return None,
        Err(refbox::BorrowError::Borrowed) => panic!("pane index already borrowed"),
    };
    map.range((std::ops::Bound::Excluded(&pane.id), std::ops::Bound::Unbounded))
        .next().map(|(_, owner)| owner.clone())
}
unsafe fn window_fire_renamed(mut w: *mut window, mut old_name: *const ::core::ffi::c_char) {
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        s: std::rc::Weak::new(),
        wl: refbox::Weak::new(),
        w: std::rc::Weak::new(),
        wp: std::rc::Weak::new(),
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
        s: std::rc::Weak::new(),
        wl: refbox::Weak::new(),
        w: std::rc::Weak::new(),
        wp: std::rc::Weak::new(),
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
        s: std::rc::Weak::new(),
        wl: refbox::Weak::new(),
        w: std::rc::Weak::new(),
        wp: std::rc::Weak::new(),
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
        s: std::rc::Weak::new(),
        wl: refbox::Weak::new(),
        w: std::rc::Weak::new(),
        wp: std::rc::Weak::new(),
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
        s: std::rc::Weak::new(),
        wl: refbox::Weak::new(),
        w: std::rc::Weak::new(),
        wp: std::rc::Weak::new(),
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
        observer: Default::default(),
        idx: 0,
        session: std::rc::Weak::new(),
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
        // Decode -(base_index + 1) without negating INT_MIN first.
        idx = winlink_next_index(wwl, -(idx + 1));
        if idx == -(1 as ::core::ffi::c_int) {
            return ::core::ptr::null_mut::<winlink>();
        }
    } else if !winlink_find_by_index(wwl, idx).is_null() {
        return ::core::ptr::null_mut::<winlink>();
    }
    let owner = refbox::RefBox::new(winlink {
        observer: Default::default(),
        idx,
        session: std::rc::Weak::new(),
        window_owner: None,
        flags: 0,
        entry: winlink_entry { owner: None },
    });
    wl = owner.as_ptr() as *mut winlink;
    (*wl).observer = owner.downgrade();
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
    if let Some(session_owner) = (*wl).session.upgrade() {
        winlink_stack_remove(&raw mut (*session_owner.get()).lastw, wl);
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
pub unsafe fn window_find_by_id_str(mut s: *const ::core::ffi::c_char) -> Option<crate::src::shared::window::WindowOwner> {
    let mut errstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut id: u_int = 0;
    if *s as ::core::ffi::c_int != '@' as i32 {
        return None;
    }
    id = strtonum(
        s.offset(1 as ::core::ffi::c_int as isize),
        0 as ::core::ffi::c_longlong,
        UINT_MAX as ::core::ffi::c_longlong,
        &raw mut errstr,
    ) as u_int;
    if !errstr.is_null() {
        return None;
    }
    return window_find_by_id(id);
}
pub unsafe fn window_find_by_id(id: u_int) -> Option<crate::src::shared::window::WindowOwner> {
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
) -> crate::src::shared::window::WindowOwner {
    if xpixel == 0 as u_int {
        xpixel = DEFAULT_XPIXEL as u_int;
    }
    if ypixel == 0 as u_int {
        ypixel = DEFAULT_YPIXEL as u_int;
    }
    let owner = window::new();
    let w = crate::src::shared::rc::as_ptr(&owner);
    (*w).flags = 0 as ::core::ffi::c_int;
    (*w).panes = window_panes::default();
    (*w).z_index = window_panes::default();
    (*w).last_panes = window_pane_history::default();
    (*w).set_active(::core::ptr::null_mut::<window_pane>());
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
        options_owner_ptr(&mut (*w).options).map_or(std::ptr::null_mut(), |options| options),
        b"pane-scrollbars\0" as *const u8 as *const ::core::ffi::c_char,
    ) as ::core::ffi::c_int;
    (*w).sb_pos = options_get_number(
        options_owner_ptr(&mut (*w).options).map_or(std::ptr::null_mut(), |options| options),
        b"pane-scrollbars-position\0" as *const u8 as *const ::core::ffi::c_char,
    ) as ::core::ffi::c_int;
    (*w).winlinks.storage = None;
    (*w).entry.owner = None;
    let fresh0 = next_window_id;
    next_window_id = next_window_id.wrapping_add(1);
    (*w).id = fresh0;
    windows_insert(&raw mut windows, &owner);
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
    crate::src::shared::window::WindowOwner::adopt(owner)
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
        && (*wp).pipe_event.with_ptr(|event| unsafe {
            evbuffer_get_length(&*(*event).output) != 0 as size_t
        }).unwrap_or(false)
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
    if (*wp).wait_item.strong_count() != 0 && !(*wp).flags & PANE_STATUSREADY != 0 {
        return 0 as ::core::ffi::c_int;
    }
    if (*wp).editor.is_some() && !(*wp).flags & PANE_STATUSREADY != 0 {
        return 0 as ::core::ffi::c_int;
    }
    return 1 as ::core::ffi::c_int;
}
pub unsafe fn window_add_ref(w: *mut window, from: *const ::core::ffi::c_char) -> Rc<std::cell::UnsafeCell<window>> {
    let owner = (*w).observer.upgrade().expect("live Rc window");
    log_debug(format_args!(
        "retain window @{} ({})",
        ((*w).id) as u32,
        log_cstr((from) as *const _)
    ));
    owner
}
unsafe fn window_before_release(w: *mut window, from: *const ::core::ffi::c_char) {
    // Notify while a strong reference still exists: callbacks may retain w.
    if (*w).observer.strong_count() == 1 {
        events_fire_window(c"window-closed".as_ptr(), w);
    }
    log_debug(format_args!(
        "release window @{} ({})",
        ((*w).id) as u32,
        log_cstr((from) as *const _)
    ));
}
pub unsafe fn window_remove_ref(owner: Rc<std::cell::UnsafeCell<window>>, from: *const ::core::ffi::c_char) {
    window_before_release(crate::src::shared::rc::as_ptr(&owner), from);
    drop(owner);
}
pub unsafe fn window_pane_add_ref(wp: *mut window_pane, from: *const ::core::ffi::c_char) -> std::rc::Rc<std::cell::UnsafeCell<window_pane>> {
    let owner = (*wp).observer.upgrade().expect("live Rc pane");
    log_debug(format_args!(
        "retain pane %{} ({})",
        ((*wp).id) as u32,
        log_cstr((from) as *const _)
    ));
    owner
}
pub unsafe fn window_pane_remove_ref(owner: std::rc::Rc<std::cell::UnsafeCell<window_pane>>, from: *const ::core::ffi::c_char) {
    let wp = crate::src::shared::rc::as_ptr(&owner);
    log_debug(format_args!(
        "release pane %{} ({})",
        ((*wp).id) as u32,
        log_cstr((from) as *const _)
    ));
    drop(owner);
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
        server_redraw_window(&*(w));
    }
    if xpixel != -(1 as ::core::ffi::c_int) {
        (*w).xpixel = xpixel as u_int;
    }
    if ypixel != -(1 as ::core::ffi::c_int) {
        (*w).ypixel = ypixel as u_int;
    }
    redraw_invalidate_scene(w);
}
pub unsafe fn window_pane_send_resize(wp: &window_pane, sx: u_int, sy: u_int) {
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
pub fn window_has_pane(w: &window, pane: &std::rc::Weak<std::cell::UnsafeCell<window_pane>>) -> bool {
    pane.strong_count() != 0 && w.panes.position(pane).is_some()
}
pub unsafe fn window_pane_contains(
    pane_owner: &Rc<std::cell::UnsafeCell<window_pane>>,
    mut x: u_int,
    mut y: u_int,
) -> ::core::ffi::c_int {
    let wp = pane_owner.get();
    let mut xoff: ::core::ffi::c_int = 0;
    let mut yoff: ::core::ffi::c_int = 0;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    if window_pane_is_visible(&*wp) == 0 {
        return 0 as ::core::ffi::c_int;
    }
    (xoff, yoff, sx, sy) = window_pane_full_size_offset(pane_owner);
    if window_pane_is_floating(&*wp) == 0 {
        if (x as ::core::ffi::c_int) < xoff || x > (xoff as u_int).wrapping_add(sx) {
            return 0 as ::core::ffi::c_int;
        }
        if (y as ::core::ffi::c_int) < yoff || y > (yoff as u_int).wrapping_add(sy) {
            return 0 as ::core::ffi::c_int;
        }
    } else if window_pane_get_pane_lines(&*wp) as ::core::ffi::c_uint
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
        window_pane_update_focus((*w).active_ptr());
    }
}
pub unsafe fn window_pane_update_focus(mut wp: *mut window_pane) {
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut focused: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if !wp.is_null() && !(*wp).flags & PANE_EXITED != 0 {
        if wp != (*(*wp).window).active_ptr() {
            focused = 0 as ::core::ffi::c_int;
        } else {
            let mut registry_c_owner = clients.first();
            c = registry_c_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
            while !c.is_null() {
                if !(*c).session_ptr().is_null()
                    && (*(*c).session_ptr()).attached != 0 as u_int
                    && (*c).flags & CLIENT_FOCUSED as uint64_t != 0
                    && (*(*(*c).session_ptr()).curw_ptr()).window_ptr() == (*wp).window
                    && (*c).overlay_draw.is_none()
                    && (*(*wp).window).menu.is_none()
                {
                    focused = 1 as ::core::ffi::c_int;
                    break;
                } else {
                    registry_c_owner = clients.next(registry_c_owner.as_ref().expect("current registry client"));
                    c = registry_c_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
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
                let _ = (*wp).event.with_ptr(|event| unsafe { bufferevent_write(
                    event,
                    b"\x1B[O\0" as *const u8 as *const ::core::ffi::c_char
                        as *const ::core::ffi::c_void,
                    3 as size_t,
                ) });
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
                let _ = (*wp).event.with_ptr(|event| unsafe { bufferevent_write(
                    event,
                    b"\x1B[I\0" as *const u8 as *const ::core::ffi::c_char
                        as *const ::core::ffi::c_void,
                    3 as size_t,
                ) });
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
    if wp == (*w).active_ptr() {
        return 0 as ::core::ffi::c_int;
    }
    if (*w).modal.upgrade().is_some() && !wp.as_ref().is_some_and(|pane| (*w).modal.ptr_eq(&pane.observer)) {
        return 0 as ::core::ffi::c_int;
    }
    if (*w).flags & WINDOW_ZOOMED != 0 && window_pane_is_visible(&*wp) == 0 {
        window_unzoom(w, 1 as ::core::ffi::c_int);
    }
    lastwp = (*w).active_ptr();
    window_pane_stack_remove(&raw mut (*w).last_panes, wp);
    window_pane_stack_push(&raw mut (*w).last_panes, lastwp);
    (*w).set_active(wp);
    let fresh1 = next_active_point;
    next_active_point = next_active_point.wrapping_add(1);
    (*(*w).active_ptr()).active_point = fresh1;
    (*(*w).active_ptr()).flags |= PANE_CHANGED;
    if options_get_number(
        global_options,
        b"focus-events\0" as *const u8 as *const ::core::ffi::c_char,
    ) != 0
    {
        window_pane_update_focus(lastwp);
        window_pane_update_focus((*w).active_ptr());
    }
    tty_update_window_offset(w);
    server_redraw_window(&*(w));
    if notify != 0 {
        window_fire_pane_changed(w, (*w).active_ptr(), lastwp);
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
    if (*w).modal.upgrade().is_some() && !wp.as_ref().is_some_and(|pane| (*w).modal.ptr_eq(&pane.observer)) {
        return;
    }
    if wp == (*w).active_ptr() {
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
        if wp == (*w).active_ptr() {
            break;
        }
        if window_pane_is_floating(&*wp) != 0 {
            window_pane_z_remove(&mut *w, &*wp);
            window_pane_z_insert_front(&mut *w, &*wp);
            (*wp).flags |= PANE_REDRAW;
            redraw_invalidate_scene(w);
        }
        wp = (*w).active_ptr();
        if wp.is_null() {
            break;
        }
    }
}
pub unsafe fn window_get_active_at(
    window_owner: &Rc<std::cell::UnsafeCell<window>>,
    mut x: u_int,
    mut y: u_int,
) -> Option<Rc<std::cell::UnsafeCell<window_pane>>> {
    let w = window_owner.get();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut pane_status: ::core::ffi::c_int = 0;
    let mut xoff: ::core::ffi::c_int = 0;
    let mut yoff: ::core::ffi::c_int = 0;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    pane_status = window_get_pane_status(&*w);
    if let Some(modal) = (*w).modal.upgrade() {
        if window_pane_contains(&modal, x, y) != 0 {
            return Some(modal);
        }
        return None;
    }
    if pane_status == PANE_STATUS_TOP {
        for candidate in (*w).z_index.snapshot() {
            wp = candidate.get();
            if !(window_pane_is_visible(&*wp) == 0 || window_pane_is_floating(&*wp) != 0) {
                (xoff, yoff, sx, sy) = window_pane_full_size_offset(&candidate);
                if !((x as ::core::ffi::c_int) < xoff || x > (xoff as u_int).wrapping_add(sx)) {
                    if y as ::core::ffi::c_int == yoff - 1 as ::core::ffi::c_int {
                        return Some(candidate);
                    }
                }
            }

        }
    }
    let mut current_block_15: u64;
    for candidate in (*w).z_index.snapshot() {
        wp = candidate.get();
        if !(window_pane_is_visible(&*wp) == 0) {
            (xoff, yoff, sx, sy) = window_pane_full_size_offset(&candidate);
            if window_pane_is_floating(&*wp) == 0 {
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
            } else if window_pane_get_pane_lines(&*wp) as ::core::ffi::c_uint
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
                _ => return Some(candidate),
            }
        }

    }
    return None;
}
pub unsafe fn window_find_string(
    window_owner: &Rc<std::cell::UnsafeCell<window>>,
    name: &CStr,
) -> Option<Rc<std::cell::UnsafeCell<window_pane>>> {
    let w = window_owner.get();
    let s = name.as_ptr();
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    let mut top: u_int = 0 as u_int;
    let mut bottom: u_int = (*w).sy.wrapping_sub(1 as u_int);
    let mut status: ::core::ffi::c_int = 0;
    x = (*w).sx.wrapping_div(2 as u_int);
    y = (*w).sy.wrapping_div(2 as u_int);
    status = window_get_pane_status(&*w);
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
        return None;
    }
    return window_get_active_at(window_owner, x, y);
}
pub unsafe fn window_zoom(pane_owner: &Rc<std::cell::UnsafeCell<window_pane>>) -> ::core::ffi::c_int {
    let wp = pane_owner.get();
    let mut w: *mut window = (*wp).window as *mut window;
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
    if window_count_panes(&*w, 1 as ::core::ffi::c_int) == 1 as u_int {
        return -(1 as ::core::ffi::c_int);
    }
    if (*w).active_ptr() != wp
        && ((*w).active_ptr().is_null()
            || !(*(*w).active_ptr()).flags & PANE_FLOATOVERZOOM != 0
            || window_pane_is_floating(&*(*w).active_ptr()) == 0)
    {
        window_set_active_pane(w, wp, 1 as ::core::ffi::c_int);
    }
    (*wp).flags |= PANE_ZOOMED;
    let mut cursor = window_pane_first(w.as_ref());
    while let Some(owner) = cursor {
        let wp1 = &mut *owner.get();
        (*wp1).saved_layout_cell = (*wp1).layout_cell as *mut layout_cell;
        (*wp1).layout_cell = ::core::ptr::null_mut::<layout_cell>();
        cursor = window_pane_next(Some(&*wp1));
    }
    (*w).saved_layout_root = (*w).layout_root.take();
    layout_init(w, wp);
    let mut cursor = window_pane_first(w.as_ref());
    while let Some(owner) = cursor {
        let wp1 = owner.get();
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
        cursor = window_pane_next(Some(&*wp1));
    }
    if (*(*wp).saved_layout_cell).flags & LAYOUT_CELL_FLOATING != 0 {
        window_pane_z_remove(&mut *w, &*wp);
        window_pane_z_insert_back(&mut *w, &*wp);
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
    let mut zoomed_owner = None;
    let mut slc: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    if !(*w).flags & WINDOW_ZOOMED != 0 {
        return -(1 as ::core::ffi::c_int);
    }
    let mut cursor = window_pane_first(w.as_ref());
    while let Some(owner) = cursor {
        let wp = &mut *owner.get();
        if (*wp).flags & PANE_ZOOMED != 0 {
            zoomed_owner = Some(owner.clone());
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
        cursor = window_pane_next(Some(&*wp));
    }
    (*w).flags &= !WINDOW_ZOOMED;
    layout_free(w);
    (*w).layout_root = (*w).saved_layout_root.take();
    let mut cursor = window_pane_first(w.as_ref());
    while let Some(owner) = cursor {
        let wp = &mut *owner.get();
        (*wp).layout_cell = (*wp).saved_layout_cell as *mut layout_cell;
        (*wp).saved_layout_cell = ::core::ptr::null_mut::<layout_cell>();
        (*wp).flags &= !PANE_ZOOMED;
        cursor = window_pane_next(Some(&*wp));
    }
    if let Some(zoomed_owner) = zoomed_owner.filter(|owner| window_pane_is_floating(&*owner.get()) != 0) {
        let zoomed = zoomed_owner.get();
        window_pane_z_remove(&mut *w, &*zoomed);
        if zoomed == (*w).active_ptr() {
            window_pane_z_insert_front(&mut *w, &*zoomed);
        } else {
            let mut before = window_pane_z_first(w.as_ref());
            while let Some(owner) = before.as_ref() {
                if window_pane_is_floating(&*owner.get()) == 0 {
                    break;
                }
                before = window_pane_z_next(Some(&*owner.get()));
            }
            if let Some(before) = before {
                window_pane_z_insert_before(&mut *w, &*before.get(), &*zoomed);
            } else {
                window_pane_z_insert_back(&mut *w, &*zoomed);
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
pub unsafe fn window_zoomed_pane(w: &window) -> Option<Rc<std::cell::UnsafeCell<window_pane>>> {
    if w.flags & WINDOW_ZOOMED == 0 {
        return None;
    }
    w.z_index.snapshot().into_iter().rev().find(|owner| {
        let pane = &*owner.get();
        !pane.layout_cell.is_null() && window_pane_is_floating(&*owner.get()) == 0
    })
}
pub unsafe fn window_active_pane_is_over_zoom(mut w: *mut window) -> ::core::ffi::c_int {
    if !(*w).flags & WINDOW_ZOOMED != 0 {
        return 0 as ::core::ffi::c_int;
    }
    if (*w).active_ptr().is_null() {
        return 0 as ::core::ffi::c_int;
    }
    if !(*(*w).active_ptr()).flags & PANE_FLOATOVERZOOM != 0 {
        return 0 as ::core::ffi::c_int;
    }
    return window_pane_is_floating(&*(*w).active_ptr());
}
pub unsafe fn window_push_zoom(
    mut w: *mut window,
    mut always: ::core::ffi::c_int,
    mut flag: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let pane_owner = window_zoomed_pane(&*w);
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
        (*w).was_zoomed = pane_owner.as_ref().map_or_else(std::rc::Weak::new, Rc::downgrade);
    } else {
        (*w).was_zoomed = std::rc::Weak::new();
    }
    return (window_unzoom(w, 1 as ::core::ffi::c_int) == 0 as ::core::ffi::c_int)
        as ::core::ffi::c_int;
}
pub unsafe fn window_pop_zoom(mut w: *mut window) -> ::core::ffi::c_int {
    let mut pane_owner = (*w).was_zoomed.upgrade();
    log_debug(format_args!(
        "{}: @{} {}",
        "window_pop_zoom",
        ((*w).id) as u32,
        ((*w).flags & WINDOW_WASZOOMED != 0) as ::core::ffi::c_int
    ));
    if (*w).flags & WINDOW_WASZOOMED != 0 {
        (*w).flags &= !WINDOW_WASZOOMED;
        (*w).was_zoomed = std::rc::Weak::new();
        let active = (*w).active.upgrade();
        if active.as_ref().is_some_and(|owner| {
            (*owner.get()).flags & PANE_FLOATOVERZOOM == 0
                || window_pane_is_floating(&*owner.get()) == 0
        }) {
            pane_owner = active.clone();
        }
        if pane_owner.as_ref().is_none_or(|owner| !window_has_pane(&*w, &Rc::downgrade(owner))) {
            pane_owner = active;
        }
        if let Some(owner) = pane_owner {
            return (window_zoom(&owner) == 0) as ::core::ffi::c_int;
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
        other = (*w).active_ptr();
    }
    wp = window_pane_create(w, (*w).sx, (*w).sy, hlimit);
    if window_pane_first(w.as_ref()).is_none() {
        log_debug(format_args!(
            "{}: @{} at start",
            "window_add_pane",
            ((*w).id) as u32
        ));
        window_pane_list_insert_front(&mut *w, &*wp);
    } else if flags & SPAWN_BEFORE != 0 {
        log_debug(format_args!(
            "{}: @{} before %{}",
            "window_add_pane",
            ((*w).id) as u32,
            ((*wp).id) as u32
        ));
        if flags & SPAWN_FULLSIZE != 0 {
            window_pane_list_insert_front(&mut *w, &*wp);
        } else {
            window_pane_list_insert_before(&mut *w, &*other, &*wp);
        }
    } else {
        log_debug(format_args!(
            "{}: @{} after %{}",
            "window_add_pane",
            ((*w).id) as u32,
            ((*wp).id) as u32
        ));
        if flags & (SPAWN_FULLSIZE | SPAWN_FLOATING) != 0 {
            window_pane_list_insert_back(&mut *w, &*wp);
        } else {
            window_pane_list_insert_after(&mut *w, &*other, &*wp);
        }
    }
    if flags & SPAWN_FLOATING == 0 {
        window_pane_z_insert_back(&mut *w, &*wp);
    } else if let Some(modal) = (*w).modal.upgrade() {
        window_pane_z_insert_after(&mut *w, &*modal.get(), &*wp);
    } else {
        window_pane_z_insert_front(&mut *w, &*wp);
    }
    redraw_invalidate_scene(w);
    return wp;
}
pub unsafe fn window_lost_pane(mut w: *mut window, mut wp: *mut window_pane) {
    log_debug(format_args!(
        "{}: @{} pane %{}",
        "window_lost_pane",
        ((*w).id) as u32,
        ((*wp).id) as u32
    ));
    if wp == marked_pane.wp_ptr() {
        server_clear_marked();
    }
    if (*w).modal_last.ptr_eq(&(*wp).observer) {
        (*w).modal_last = std::rc::Weak::new();
    }
    if (*w).was_zoomed.ptr_eq(&(*wp).observer) {
        (*w).was_zoomed = std::rc::Weak::new();
    }
    window_pane_stack_remove(&raw mut (*w).last_panes, wp);
    if wp == (*w).active_ptr() {
        let mut replacement = if (*w).modal.ptr_eq(&(*wp).observer) {
            (*w).modal = std::rc::Weak::new();
            std::mem::take(&mut (*w).modal_last).upgrade()
        } else {
            None
        };
        if replacement.as_ref().is_none_or(|owner| !window_has_pane(&*w, &Rc::downgrade(owner))) {
            replacement = (*w).last_panes.first();
        }
        if replacement.is_none() {
            replacement = (*w).panes.previous(&(*wp).observer)
                .or_else(|| (*w).panes.next(&(*wp).observer));
        }
        (*w).set_active(replacement.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get()));
        if !(*w).active_ptr().is_null() {
            window_pane_stack_remove(&raw mut (*w).last_panes, (*w).active_ptr());
            (*(*w).active_ptr()).flags |= PANE_CHANGED;
            window_fire_pane_changed(w, (*w).active_ptr(), wp);
            window_update_focus(w);
        }
    } else if (*w).modal.ptr_eq(&(*wp).observer) {
        (*w).modal_last = std::rc::Weak::new();
        (*w).modal = std::rc::Weak::new();
    }
    redraw_invalidate_scene(w);
}
pub unsafe fn window_remove_pane(mut w: *mut window, pane_owner: &Rc<std::cell::UnsafeCell<window_pane>>) {
    let wp = pane_owner.get();
    window_lost_pane(w, wp);
    window_pane_list_remove(&mut *w, &*wp);
    window_pane_z_remove(&mut *w, &*wp);
    redraw_invalidate_scene(w);
    window_pane_destroy(pane_owner);
}
pub unsafe fn window_pane_at_index(w: &mut window, idx: u_int) -> Option<Rc<std::cell::UnsafeCell<window_pane>>> {
    let base = options_get_number(
        options_owner_ptr(&mut w.options).map_or(std::ptr::null_mut(), |options| options),
        c"pane-base-index".as_ptr(),
    ) as u_int;
    w.panes.storage.as_deref()?.get(idx.wrapping_sub(base) as usize)
        .map(|observer| observer.upgrade().expect("live pane in ordering"))
}
pub fn window_pane_next_by_number(
    w: &window,
    pane: Option<&Rc<std::cell::UnsafeCell<window_pane>>>,
    n: u_int,
) -> Option<Rc<std::cell::UnsafeCell<window_pane>>> {
    let mut current = pane.cloned();
    for _ in 0..n {
        current = current.as_ref().and_then(|owner| w.panes.next(&Rc::downgrade(owner)))
            .or_else(|| w.panes.first());
    }
    current
}
pub fn window_pane_previous_by_number(
    w: &window,
    pane: Option<&Rc<std::cell::UnsafeCell<window_pane>>>,
    n: u_int,
) -> Option<Rc<std::cell::UnsafeCell<window_pane>>> {
    let mut current = pane.cloned();
    for _ in 0..n {
        current = current.as_ref().and_then(|owner| w.panes.previous(&Rc::downgrade(owner)))
            .or_else(|| w.panes.last());
    }
    current
}
pub unsafe fn window_pane_index(wp: &window_pane) -> Option<u32> {
    let w = &mut *wp.window;
    let base = options_get_number(
        options_owner_ptr(&mut w.options).map_or(std::ptr::null_mut(), |options| options),
        c"pane-base-index".as_ptr(),
    ) as u_int;
    w.panes.position(&wp.observer).map(|position| base.wrapping_add(position as u32))
}
pub unsafe fn window_pane_zindex(wp: &window_pane) -> Option<u32> {
    let w = &*wp.window;
    let mut index = 0u32;
    for owner in w.z_index.snapshot() {
        let floating = window_pane_is_floating(&*owner.get()) != 0;
        if Rc::downgrade(&owner).ptr_eq(&wp.observer) {
            return Some(if floating { index } else { index.wrapping_add(1) });
        }
        if floating {
            index = index.wrapping_add(1);
        }
    }
    None
}
pub unsafe fn window_pane_last_index(wp: &window_pane) -> Option<u32> {
    let history = &(*wp.window).last_panes;
    history.storage.as_deref()?.iter()
        .position(|observer| observer.ptr_eq(&wp.observer))
        .map(|position| position as u32)
}
pub unsafe fn window_count_panes(
    w: &window,
    with_floating: ::core::ffi::c_int,
) -> u_int {
    w.panes.storage.as_deref().into_iter().flatten().fold(0, |count, observer| {
        let pane = observer.upgrade().expect("live pane in ordering");
        if with_floating != 0 || window_pane_is_floating(&*pane.get()) == 0 {
            count.wrapping_add(1)
        } else {
            count
        }
    })
}
pub unsafe fn window_destroy_panes(w: *mut window) {
    while let Some(owner) = window_pane_stack_first(w.as_ref()) {
        window_pane_stack_remove(&raw mut (*w).last_panes, owner.get());
    }
    while let Some(owner) = window_pane_first(w.as_ref()) {
        let wp = owner.get();
        window_pane_list_remove(&mut *w, &*wp);
        window_pane_z_remove(&mut *w, &*wp);
        window_pane_destroy(&owner);
    }
}
pub unsafe fn window_printable_flags(
    mut wl: *mut winlink,
    mut escape: ::core::ffi::c_int,
) -> std::ffi::CString {
    let session_owner = (*wl).session.upgrade();
    let mut flags: [::core::ffi::c_char; 32] = [0; 32];
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
    if session_owner.as_ref().is_some_and(|owner| wl == (*owner.get()).curw_ptr()) {
        let fresh7 = pos;
        pos = pos.wrapping_add(1);
        flags[fresh7 as usize] = '*' as i32 as ::core::ffi::c_char;
    }
    if session_owner.as_ref().is_some_and(|owner| wl == winlink_stack_first(&(*owner.get()).lastw)) {
        let fresh8 = pos;
        pos = pos.wrapping_add(1);
        flags[fresh8 as usize] = '-' as i32 as ::core::ffi::c_char;
    }
    if server_check_marked() != 0 && wl == marked_pane.wl_ptr() {
        let fresh9 = pos;
        pos = pos.wrapping_add(1);
        flags[fresh9 as usize] = 'M' as i32 as ::core::ffi::c_char;
    }
    if (*(*wl).window_ptr()).modal.upgrade().is_some() {
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
    std::ffi::CStr::from_ptr(flags.as_ptr()).to_owned()
}
pub unsafe fn window_pane_printable_flags(mut wp: *mut window_pane) -> std::ffi::CString {
    let mut w: *mut window = (*wp).window as *mut window;
    let mut flags: [::core::ffi::c_char; 32] = [0; 32];
    let mut pos: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if wp == (*w).active_ptr() {
        let fresh12 = pos;
        pos = pos + 1;
        flags[fresh12 as usize] = '*' as i32 as ::core::ffi::c_char;
    }
    if wp == window_pane_stack_first(w.as_ref()).as_ref().map_or(std::ptr::null_mut(), |owner| owner.get()) {
        let fresh13 = pos;
        pos = pos + 1;
        flags[fresh13 as usize] = '-' as i32 as ::core::ffi::c_char;
    }
    if (*wp).flags & PANE_ZOOMED != 0 {
        let fresh14 = pos;
        pos = pos + 1;
        flags[fresh14 as usize] = 'Z' as i32 as ::core::ffi::c_char;
    }
    if window_pane_is_floating(&*wp) != 0 {
        let fresh15 = pos;
        pos = pos + 1;
        flags[fresh15 as usize] = 'F' as i32 as ::core::ffi::c_char;
    }
    if (*wp).flags & PANE_FLOATOVERZOOM != 0 {
        let fresh16 = pos;
        pos = pos + 1;
        flags[fresh16 as usize] = 'A' as i32 as ::core::ffi::c_char;
    }
    if (*w).modal.ptr_eq(&(*wp).observer) {
        let fresh17 = pos;
        pos = pos + 1;
        flags[fresh17 as usize] = 'O' as i32 as ::core::ffi::c_char;
    }
    flags[pos as usize] = '\0' as i32 as ::core::ffi::c_char;
    std::ffi::CStr::from_ptr(flags.as_ptr()).to_owned()
}
pub unsafe fn window_pane_find_by_id_str(s: &CStr) -> Option<Rc<std::cell::UnsafeCell<window_pane>>> {
    if s.to_bytes().first() != Some(&b'%') {
        return None;
    }
    let mut error = std::ptr::null();
    let id = strtonum(s.as_ptr().add(1), 0, UINT_MAX as i64, &mut error) as u_int;
    if !error.is_null() {
        return None;
    }
    window_pane_find_by_id(id)
}
pub unsafe fn window_pane_find_by_id(id: u_int) -> Option<Rc<std::cell::UnsafeCell<window_pane>>> {
    window_pane_tree_find(&*std::ptr::addr_of!(all_window_panes), id)
}
pub(crate) unsafe fn window_pane_weak(
    wp: *mut window_pane,
) -> std::rc::Weak<std::cell::UnsafeCell<window_pane>> {
    (*wp).observer.clone()
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

pub fn window_pane_first(w: Option<&window>) -> Option<Rc<std::cell::UnsafeCell<window_pane>>> {
    w?.panes.first()
}

pub fn window_pane_last(w: Option<&window>) -> Option<Rc<std::cell::UnsafeCell<window_pane>>> {
    w?.panes.last()
}

pub unsafe fn window_pane_next(wp: Option<&window_pane>) -> Option<Rc<std::cell::UnsafeCell<window_pane>>> {
    let wp = wp?;
    (*wp.window).panes.next(&wp.observer)
}

pub unsafe fn window_pane_previous(wp: Option<&window_pane>) -> Option<Rc<std::cell::UnsafeCell<window_pane>>> {
    let wp = wp?;
    (*wp.window).panes.previous(&wp.observer)
}

pub fn window_pane_z_first(w: Option<&window>) -> Option<Rc<std::cell::UnsafeCell<window_pane>>> {
    w?.z_index.first()
}

pub fn window_pane_z_last(w: Option<&window>) -> Option<Rc<std::cell::UnsafeCell<window_pane>>> {
    w?.z_index.last()
}

pub unsafe fn window_pane_z_next(wp: Option<&window_pane>) -> Option<Rc<std::cell::UnsafeCell<window_pane>>> {
    let wp = wp?;
    (*wp.window).z_index.next(&wp.observer)
}

pub unsafe fn window_pane_z_previous(wp: Option<&window_pane>) -> Option<Rc<std::cell::UnsafeCell<window_pane>>> {
    let wp = wp?;
    (*wp.window).z_index.previous(&wp.observer)
}

pub fn window_pane_stack_first(w: Option<&window>) -> Option<Rc<std::cell::UnsafeCell<window_pane>>> {
    w?.last_panes.first()
}

pub fn window_pane_stack_next(
    w: Option<&window>,
    wp: &window_pane,
) -> Option<Rc<std::cell::UnsafeCell<window_pane>>> {
    w?.last_panes.next(&wp.observer)
}

pub fn window_pane_list_remove(w: &mut window, wp: &window_pane) {
    assert!(w.panes.remove(&wp.observer), "pane is not in its window order");
}

pub fn window_pane_list_insert_front(w: &mut window, wp: &window_pane) {
    w.panes.push_front(wp.observer.clone());
}

pub fn window_pane_list_insert_back(w: &mut window, wp: &window_pane) {
    w.panes.push_back(wp.observer.clone());
}

pub fn window_pane_list_insert_before(
    w: &mut window,
    before: &window_pane,
    wp: &window_pane,
) {
    w.panes.insert_before(&before.observer, wp.observer.clone());
}

pub fn window_pane_list_insert_after(
    w: &mut window,
    after: &window_pane,
    wp: &window_pane,
) {
    w.panes.insert_after(&after.observer, wp.observer.clone());
}

pub fn window_pane_z_remove(w: &mut window, wp: &window_pane) {
    assert!(w.z_index.remove(&wp.observer), "pane is not in its stacking order");
}

pub fn window_pane_z_insert_front(w: &mut window, wp: &window_pane) {
    w.z_index.push_front(wp.observer.clone());
}

pub fn window_pane_z_insert_back(w: &mut window, wp: &window_pane) {
    w.z_index.push_back(wp.observer.clone());
}

pub fn window_pane_z_insert_before(
    w: &mut window,
    before: &window_pane,
    wp: &window_pane,
) {
    w.z_index.insert_before(&before.observer, wp.observer.clone());
}

pub fn window_pane_z_insert_after(
    w: &mut window,
    after: &window_pane,
    wp: &window_pane,
) {
    w.z_index.insert_after(&after.observer, wp.observer.clone());
}

/// Swap within the first window when `second_window` is absent.
pub fn window_pane_swap_order(
    first_window: &mut window,
    first: &window_pane,
    second_window: Option<&mut window>,
    second: &window_pane,
) {
    let Some(second_window) = second_window else {
        first_window.panes.swap(&first.observer, &second.observer);
        return;
    };
    let first_position = first_window
        .panes
        .position(&first.observer)
        .expect("first pane is not in order");
    let second_position = second_window
        .panes
        .position(&second.observer)
        .expect("second pane is not in order");
    let first_weak = first_window.panes.remove_at(&first.observer);
    let second_weak = second_window.panes.remove_at(&second.observer);
    first_window.panes.insert_at(first_position, second_weak);
    second_window
        .panes
        .insert_at(second_position, first_weak);
}

/// Swap within the first window when `second_window` is absent.
pub fn window_pane_z_swap_order(
    first_window: &mut window,
    first: &window_pane,
    second_window: Option<&mut window>,
    second: &window_pane,
) {
    let Some(second_window) = second_window else {
        first_window.z_index.swap(&first.observer, &second.observer);
        return;
    };
    let first_position = first_window
        .z_index
        .position(&first.observer)
        .expect("first pane is not in stacking order");
    let second_position = second_window
        .z_index
        .position(&second.observer)
        .expect("second pane is not in stacking order");
    let first_weak = first_window.z_index.remove_at(&first.observer);
    let second_weak = second_window.z_index.remove_at(&second.observer);
    first_window
        .z_index
        .insert_at(first_position, second_weak);
    second_window
        .z_index
        .insert_at(second_position, first_weak);
}

/// Return the next mode in a pane's stack. Entries are boxed individually,
/// so this derives ordering from the owning collection without putting queue
/// links into each callback-visible mode entry.
pub(crate) unsafe fn window_pane_mode_next(wme: *mut window_mode_entry) -> *mut window_mode_entry {
    if wme.is_null() {
        return ::core::ptr::null_mut();
    }
    let Some(pane_owner) = (*wme).wp.upgrade() else {
        return ::core::ptr::null_mut();
    };
    let Some(storage) = (*pane_owner.get()).modes.storage.as_ref() else {
        return ::core::ptr::null_mut();
    };
    let Some(index) = storage.entries.iter().position(|entry| {
        entry.as_ptr() == (wme as *const window_mode_entry)
    }) else {
        return ::core::ptr::null_mut();
    };
    storage
        .entries
        .get(index + 1)
        .map_or(::core::ptr::null_mut(), |entry| {
            entry.as_ptr().cast_mut()
        })
}

/// Resolve a live mode entry against its pane-owned stack before observing it.
pub(crate) unsafe fn window_pane_mode_weak(
    wme: *mut window_mode_entry,
) -> refbox::Weak<window_mode_entry> {
    let pane_owner = (*wme).wp.upgrade().expect("mode belongs to a live pane");
    let storage = (*pane_owner.get())
        .modes
        .storage
        .as_ref()
        .expect("mode belongs to a pane mode stack");
    storage
        .entries
        .iter()
        .find(|entry| entry.as_ptr() == wme.cast_const())
        .expect("mode entry belongs to its pane")
        .downgrade()
}

unsafe fn window_pane_mode_insert_front(
    wp: &mut window_pane,
    entry: refbox::RefBox<window_mode_entry>,
) -> refbox::Weak<window_mode_entry> {
    let modes = &mut wp.modes;
    let storage = modes.storage.get_or_insert_with(Default::default);
    let weak = entry.downgrade();
    storage.entries.insert(0, entry);
    weak
}

unsafe fn window_pane_mode_remove(
    wp: *mut window_pane,
    wme: *mut window_mode_entry,
) -> Option<refbox::RefBox<window_mode_entry>> {
    let modes = &mut (*wp).modes;
    let (removed, empty) = {
        let storage = modes.storage.as_mut()?;
        let index = storage.entries.iter().position(|entry| {
            entry.as_ptr() == (wme as *const window_mode_entry)
        })?;
        let removed = storage.entries.remove(index);
        (removed, storage.entries.is_empty())
    };
    if empty {
        modes.storage = None;
    }
    Some(removed)
}

unsafe fn window_pane_mode_promote(wp: *mut window_pane, wme: *mut window_mode_entry) {
    if (*wp).modes.active_ptr() == wme {
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

    unsafe fn test_mode_display(wme: *mut window_mode_entry) -> *mut screen {
        let owner = (*wme).wp.upgrade().expect("mode belongs to pane");
        &raw mut (*owner.get()).status_screen
    }

    unsafe fn check_mode_cleanup(wme: *mut window_mode_entry) {
        let owner = (*wme).wp.upgrade().expect("cleanup retains parent pane");
        let pane = &mut *owner.get();
        assert_ne!(pane.modes.active_ptr(), wme);
        let expected = if pane.modes.active_ptr().is_null() {
            &raw mut pane.base
        } else {
            &raw mut pane.status_screen
        };
        assert_eq!(pane.screen_ptr(), expected);
        pane.sx += 1;
    }

    #[test]
    fn freeing_modes_keeps_parent_alive_until_all_callbacks_finish() {
        static MODE: std::sync::LazyLock<window_mode> = std::sync::LazyLock::new(|| window_mode {
            free: Some(check_mode_cleanup),
            display_screen: Some(test_mode_display),
            ..window_mode::default()
        });
        unsafe {
            let owner = window_pane::new();
            let observer = Rc::downgrade(&owner);
            let wp = owner.get();
            (*wp).sx = 0;
            for _ in 0..3 {
                let mut entry = boxed_mode(wp);
                entry.get_mut_unchecked().mode = &MODE;
                window_pane_mode_insert_front(&mut *wp, entry);
            }
            window_pane_free_modes(&owner);
            assert_eq!((*wp).sx, 3);
            assert!((*wp).modes.active_ptr().is_null());
            assert!((*wp).modes.storage.is_none());
            assert_eq!((*wp).screen_ptr(), &raw mut (*wp).base);
            drop(owner);
            assert!(observer.upgrade().is_none());
        }
    }

    unsafe fn boxed_mode(wp: *mut window_pane) -> refbox::RefBox<window_mode_entry> {
        refbox::RefBox::new(window_mode_entry {
            wp: (*wp).observer.clone(),
            swp: std::rc::Weak::new(),
            mode: &window_copy_mode,
            data: ::core::ptr::null_mut(),
            data_owner: None,
            prefix: 1,
            kill: 0,
        })
    }

    #[test]
    fn pane_mode_stack_reorders_and_removes_stable_entries() {
        unsafe {
            let pane_owner = window_pane::new();
            let wp = pane_owner.get();
            (*wp).modes = window_pane_modes::default();

            let a = window_pane_mode_insert_front(&mut *wp, boxed_mode(wp)).as_ptr().cast_mut();
            let b = window_pane_mode_insert_front(&mut *wp, boxed_mode(wp)).as_ptr().cast_mut();
            let c_observer = window_pane_mode_insert_front(&mut *wp, boxed_mode(wp));
            let c = c_observer.as_ptr().cast_mut();
            assert_eq!((*wp).modes.active_ptr(), c);
            assert_eq!(window_pane_mode_next(c), b);
            assert_eq!(window_pane_mode_next(b), a);
            assert!(window_pane_mode_next(a).is_null());

            window_pane_mode_promote(wp, a);
            assert!(c_observer.is_alive());
            assert_eq!((*wp).modes.active_ptr(), a);
            assert_eq!(window_pane_mode_next(a), c);
            assert_eq!(window_pane_mode_next(c), b);

            let removed = window_pane_mode_remove(wp, c).expect("mode was present");
            assert_eq!(removed.as_ptr().cast_mut(), c);
            assert_eq!((*wp).modes.active_ptr(), a);
            assert_eq!(window_pane_mode_next(a), b);
            drop(removed);
            assert!(!c_observer.is_alive());

            // Force Vec growth after callbacks already hold `a` and `b`.
            for _ in 0..64 {
                window_pane_mode_insert_front(&mut *wp, boxed_mode(wp));
            }
            assert_eq!(window_pane_mode_next(a), b);
            assert!(window_pane_mode_next(b).is_null());

            while !(*wp).modes.active_ptr().is_null() {
                let top = (*wp).modes.active_ptr();
                drop(window_pane_mode_remove(wp, top).expect("mode was present"));
            }
            assert!((*wp).modes.storage.is_none());

            let detached = boxed_mode(wp);
            drop(pane_owner);
            assert!(detached.get_unchecked().wp.upgrade().is_none());
            assert!(window_pane_mode_next(detached.as_ptr().cast_mut()).is_null());
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
    let owner = window_pane::new();
    wp = crate::src::shared::rc::as_ptr(&owner);
    (*wp).window = w as *mut window;
    (*wp).options = Some(crate::src::options::options_create_owned(options_owner_ptr(&mut (*w).options).map_or(std::ptr::null_mut(), |options| options)));
    (*wp).flags = PANE_STYLECHANGED;
    (*wp).cmd_status = -(1 as ::core::ffi::c_int);
    let fresh2 = next_window_pane_id;
    next_window_pane_id = next_window_pane_id.wrapping_add(1);
    (*wp).id = fresh2;
    window_pane_tree_insert(&mut *std::ptr::addr_of_mut!(all_window_panes), owner);
    (*wp).fd = -(1 as ::core::ffi::c_int);
    (*wp).modes = window_pane_modes::default();
    (*wp).resize_queue = window_pane_resizes::default();
    (*wp).sx = sx;
    (*wp).sy = sy;
    (*wp).pipe_fd = -(1 as ::core::ffi::c_int);
    (*wp).control_bg = -(1 as ::core::ffi::c_int);
    (*wp).control_fg = -(1 as ::core::ffi::c_int);
    style_set_scrollbar_style_from_option(&raw mut (*wp).scrollbar_style, options_owner_ptr(&mut (*wp).options).map_or(std::ptr::null_mut(), |options| options));
    colour_palette_init(&mut (*wp).palette);
    colour_palette_from_option(Some(&mut (*wp).palette), options_owner_ptr(&mut (*wp).options).map_or(std::ptr::null_mut(), |options| options));
    screen_init(&mut (*wp).base, sx, sy, hlimit);
    (*wp).screen_source = PaneScreenSource::Base;
    window_pane_default_cursor(wp);
    screen_init(&mut (*wp).status_screen, 1 as u_int, 1 as u_int, 0 as u_int);
    style_ranges_init(&raw mut (*wp).border_status_line.ranges);
    let scrollbar_observer = (*wp).observer.clone();
    event_set(
        &raw mut (*wp).sb_auto_timer,
        -(1 as ::core::ffi::c_int),
        0 as ::core::ffi::c_short,
        move |_, _| unsafe {
            if let Some(owner) = scrollbar_observer.upgrade() {
                window_pane_scrollbar_timer(&owner);
            }
        },
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
    let item_owner = std::mem::take(&mut (*wp).wait_item).upgrade();
    let Some(item_owner) = item_owner else { return };
    let item = item_owner.get();
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut retval: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
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
    let c_owner = cmdq_get_client(item);
    c = c_owner.as_ref().map_or(std::ptr::null_mut(), crate::src::shared::rc::as_ptr);
    if !c.is_null() && (*c).session_ptr().is_null() {
        (*c).retval = retval;
    }
    cmdq_continue(item);
}
unsafe fn window_pane_free_modes(pane_owner: &Rc<std::cell::UnsafeCell<window_pane>>) {
    let wp = pane_owner.get();
    let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
    while !(*wp).modes.active_ptr().is_null() {
        wme = (*wp).modes.active_ptr();
        let entry = window_pane_mode_remove(wp, wme).expect("mode entry is owned by pane");
        let next = (*wp).modes.active_ptr();
        (*wp).screen_source = if next.is_null() {
            PaneScreenSource::Base
        } else {
            PaneScreenSource::Mode(window_pane_mode_weak(next))
        };
        (*(*wme).mode).free.expect("non-null function pointer")(wme);
        drop(entry);
    }
    (*wp).screen_source = PaneScreenSource::Base;
}
unsafe fn window_pane_scrollbar_timer(owner: &Rc<std::cell::UnsafeCell<window_pane>>) {
    let wp = owner.get();
    (*wp).sb_auto_hover = 0 as ::core::ffi::c_int;
    window_pane_scrollbar_hide(owner);
}
unsafe fn window_pane_scrollbar_auto_hide(wp: &window_pane) -> ::core::ffi::c_int {
    return ((*wp.window).sb == PANE_SCROLLBARS_MODAL
        || (*wp.window).sb == PANE_SCROLLBARS_AUTOHIDE) as ::core::ffi::c_int;
}
pub unsafe fn window_pane_scrollbar_overlay_visible(
    wp: &window_pane,
) -> ::core::ffi::c_int {
    return (window_pane_scrollbar_overlay(wp) != 0 && window_pane_scrollbar_visible(wp) != 0)
        as ::core::ffi::c_int;
}
pub unsafe fn window_pane_scrollbar_redraw(pane_owner: &Rc<std::cell::UnsafeCell<window_pane>>) {
    let wp = pane_owner.get();
    if window_pane_scrollbar_visible(&*wp) == 0 {
        return;
    }
    if window_pane_scrollbar_overlay_visible(&*wp) != 0 {
        (*wp).flags |= PANE_REDRAW;
        return;
    }
    (*wp).flags |= PANE_REDRAWSCROLLBAR;
}
unsafe fn window_pane_scrollbar_redraw_visibility(pane_owner: &Rc<std::cell::UnsafeCell<window_pane>>) {
    let wp = pane_owner.get();
    redraw_invalidate_scene((*wp).window as *mut window);
    (*wp).flags |= PANE_REDRAW;
    server_redraw_window(&*((*wp).window));
}
unsafe fn window_pane_destroy(pane_owner: &Rc<std::cell::UnsafeCell<window_pane>>) {
    let wp = pane_owner.get();
    window_pane_wait_finish(wp);
    spawn_editor_finish(wp);
    let owner = window_pane_tree_remove(&mut *std::ptr::addr_of_mut!(all_window_panes), &mut *wp).expect("registered pane owner");
    (*wp).flags |= PANE_DESTROYED;
    window_pane_clear_prompt(&owner);
    window_pane_free_modes(&owner);
    screen_write_clear_dirty(wp);
    if (*wp).fd != -(1 as ::core::ffi::c_int) {
        utempter_remove_record((*wp).fd);
        kill(getpid(), SIGCHLD);
    }
    // Empty panes have stream buffers and an input parser without a PTY.
    (*wp).event.free();
    if (*wp).fd != -(1 as ::core::ffi::c_int) {
        close((*wp).fd);
        (*wp).fd = -(1 as ::core::ffi::c_int);
    }
    if let Some(ictx) = (*wp).ictx.take() {
        input_free(ictx);
    }
    if (*wp).pipe_fd != -(1 as ::core::ffi::c_int) {
        (*wp).pipe_event.free();
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
    window_pane_clear_resizes(&mut *wp, ::core::ptr::null_mut::<window_pane_resize>());
    window_pane_remove_ref(
        owner,
        b"window_pane_destroy\0" as *const u8 as *const ::core::ffi::c_char,
    );
}
unsafe fn window_pane_free(mut wp: *mut window_pane) {
    log_debug(format_args!("pane %{} freed", ((*wp).id) as u32));
    // Logical pane destruction normally takes this first. Also keep direct
    // owner destruction safe: parser timers and sync state precede screens.
    if let Some(input) = (*wp).ictx.take() {
        drop(input);
        // The parser cannot upgrade its weak pane during the final Rc drop.
        crate::src::screen_write::screen_write_stop_sync(wp);
    }
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
unsafe fn window_pane_read_callback(owner: &Rc<std::cell::UnsafeCell<window_pane>>) {
    let wp = owner.get();
    let mut wpo: *mut window_pane_offset = &raw mut (*wp).pipe_offset;
    let size: size_t = (*wp).event.with_ptr(|event| unsafe {
        evbuffer_get_length(&(*event).input)
    }).unwrap_or(0);
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    if (*wp).pipe_fd != -(1 as ::core::ffi::c_int) {
        let data = (*wp).event.with_ptr(|event| unsafe {
            window_pane_get_new_data(&mut *(*event).input, (*wp).base_offset, &*wpo).to_vec()
        }).unwrap_or_default();
        let new_size = data.len();
        if new_size > 0 as size_t {
            let _ = (*wp).pipe_event.with_ptr(|event| unsafe {
                bufferevent_write(event, data.as_ptr().cast(), new_size);
            });
            window_pane_update_used_data(wp, wpo, new_size);
        }
    }
    log_debug(format_args!(
        "%{} has {} bytes",
        ((*wp).id) as u32,
        (size) as usize
    ));
    let mut registry_c_owner = clients.first();
    c = registry_c_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    while !c.is_null() {
        if !(*c).session_ptr().is_null() && (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
            control_write_output(c, wp);
        }
        registry_c_owner = clients.next(registry_c_owner.as_ref().expect("current registry client"));
        c = registry_c_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    }
    input_parse_pane(wp);
    let _ = (*wp).event.with_ptr(|event| unsafe {
        bufferevent_disable(event, EV_READ as ::core::ffi::c_short);
    });
}
unsafe fn window_pane_error_callback(owner: &Rc<std::cell::UnsafeCell<window_pane>>) {
    let wp = owner.get();
    log_debug(format_args!("%{} error", ((*wp).id) as u32));
    (*wp).flags |= PANE_EXITED;
    if window_pane_destroy_ready(wp) != 0 {
        server_destroy_pane(owner, 1);
    }
}
pub unsafe fn window_pane_set_event(mut wp: *mut window_pane) {
    let read_observer = (*wp).observer.clone();
    let error_observer = read_observer.clone();
    setblocking((*wp).fd, 0 as ::core::ffi::c_int);
    let stream = bufferevent_new(
        (*wp).fd,
        bufferevent_data_callback(move |_| unsafe {
            if let Some(owner) = read_observer.upgrade() {
                window_pane_read_callback(&owner);
            }
        }),
        None,
        bufferevent_event_callback(move |_, _| unsafe {
            if let Some(owner) = error_observer.upgrade() {
                window_pane_error_callback(&owner);
            }
        }),
    );
    if stream.is_null() {
        fatalx(|out| out.write_all(b"out of memory"));
    }
    (*wp).event = crate::src::reactor::StreamHandle::from_ptr(stream);
    let pane_owner = (*wp).observer.upgrade().expect("live pane input owner");
    (*wp).ictx = Some(input_init(
        Some(&pane_owner),
        stream,
        crate::src::shared::input::InputPalette::Pane((*wp).observer.clone()),
        None,
    ));
    let _ = (*wp).event.with_ptr(|event| unsafe {
        bufferevent_enable(event, (EV_READ | EV_WRITE) as ::core::ffi::c_short);
    });
}
pub fn window_pane_clear_resizes(
    wp: &mut window_pane,
    mut except: *mut window_pane_resize,
) {
    wp.resize_queue.clear_except(except);
}
pub unsafe fn window_pane_resize(pane_owner: &Rc<std::cell::UnsafeCell<window_pane>>, sx: u_int, sy: u_int) {
    let wp = pane_owner.get();
    let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        s: std::rc::Weak::new(),
        wl: refbox::Weak::new(),
        w: std::rc::Weak::new(),
        wp: std::rc::Weak::new(),
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
    wme = (*wp).modes.active_ptr();
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
    pane_owner: &std::rc::Rc<std::cell::UnsafeCell<window_pane>>,
    source_owner: Option<&std::rc::Rc<std::cell::UnsafeCell<window_pane>>>,
    mode: &'static window_mode,
    mut item: *mut cmdq_item,
    mut fs: *mut cmd_find_state,
    mut args: *mut args,
) -> ::core::ffi::c_int {
    let wp = pane_owner.get();
    let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
    let mode_screen: *mut screen;
    let mut w: *mut window = (*wp).window as *mut window;
    let mut name: *const ::core::ffi::c_char = (*mode).name.as_ptr();
    let mut oname: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if !(*wp).modes.active_ptr().is_null() {
        if std::ptr::eq((*(*wp).modes.active_ptr()).mode, mode) {
            return 1 as ::core::ffi::c_int;
        }
        if (*(*(*wp).modes.active_ptr()).mode).flags & WINDOW_MODE_NO_STACK != 0 {
            window_pane_reset_mode(pane_owner);
        }
    }
    if !(*wp).modes.active_ptr().is_null() {
        oname = (*(*(*wp).modes.active_ptr()).mode).name.as_ptr();
    }
    wme = (*wp).modes.active_ptr();
    while !wme.is_null() {
        if std::ptr::eq((*wme).mode, mode) {
            break;
        }
        wme = window_pane_mode_next(wme);
    }
    if !wme.is_null() {
        window_pane_mode_promote(wp, wme);
        mode_screen = (*(*wme).mode).display_screen.expect("mode display screen getter")(wme);
    } else {
        // The pane owns a stable RefBox address for as long as callbacks retain it.
        let entry = refbox::RefBox::new(window_mode_entry {
            wp: (*wp).observer.clone(),
            swp: source_owner.map(std::rc::Rc::downgrade).unwrap_or_default(),
            mode,
            data: ::core::ptr::null_mut(),
            data_owner: None,
            prefix: 1,
            kill: 0,
        });
        wme = window_pane_mode_insert_front(&mut *wp, entry).as_ptr().cast_mut();
        mode_screen = (*(*wme).mode).init.expect("non-null function pointer")(wme, item, fs, args);
        if mode_screen.is_null() {
            drop(window_pane_mode_remove(wp, wme).expect("mode entry is owned by pane"));
            return 1 as ::core::ffi::c_int;
        }
    }
    (*wme).kill = if !args.is_null() {
        args_has(args, 'k' as i32 as u_char)
    } else {
        0 as ::core::ffi::c_int
    };
    assert!(!mode_screen.is_null(), "active mode has a screen");
    (*wp).screen_source = PaneScreenSource::Mode(window_pane_mode_weak(wme));
    (*wp).flags |= PANE_REDRAW | PANE_REDRAWSCROLLBAR | PANE_CHANGED;
    layout_fix_panes(w, ::core::ptr::null_mut::<window_pane>());
    server_redraw_window_borders(&*((*wp).window));
    server_status_window(&*((*wp).window));
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
pub unsafe fn window_pane_reset_mode(pane_owner: &Rc<std::cell::UnsafeCell<window_pane>>) {
    let wp = pane_owner.get();
    let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
    let mut next: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
    let mut w: *mut window = (*wp).window as *mut window;
    let mut kill_0: ::core::ffi::c_int = 0;
    let mut name: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut p: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if (*wp).modes.active_ptr().is_null() {
        return;
    }
    wme = (*wp).modes.active_ptr();
    p = (*(*wme).mode).name.as_ptr();
    kill_0 = (*wme).kill;
    let entry = window_pane_mode_remove(wp, wme).expect("mode entry is owned by pane");
    next = (*wp).modes.active_ptr();
    (*wp).screen_source = if next.is_null() {
        PaneScreenSource::Base
    } else {
        PaneScreenSource::Mode(window_pane_mode_weak(next))
    };
    (*(*wme).mode).free.expect("non-null function pointer")(wme);
    drop(entry);
    next = (*wp).modes.active_ptr();
    (*wp).screen_source = if next.is_null() {
        PaneScreenSource::Base
    } else {
        PaneScreenSource::Mode(window_pane_mode_weak(next))
    };
    if next.is_null() {
        (*wp).flags &= !PANE_UNSEENCHANGES;
        log_debug(format_args!("{}: no next mode", "window_pane_reset_mode"));
    } else {
        log_debug(format_args!(
            "{}: next mode is {}",
            "window_pane_reset_mode",
            log_cstr(((*(*next).mode).name.as_ptr()) as *const _)
        ));
        assert!(!(*wp).screen_ptr().is_null(), "restored mode has a screen");
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
    server_redraw_window_borders(&*((*wp).window));
    server_status_window(&*((*wp).window));
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
        server_kill_pane(pane_owner);
    }
}
pub unsafe fn window_pane_reset_mode_all(pane_owner: &Rc<std::cell::UnsafeCell<window_pane>>) {
    let wp = pane_owner.get();
    while !(*wp).modes.active_ptr().is_null() {
        window_pane_reset_mode(pane_owner);
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
    let lookup_wp_owner = window_pane_find_by_id(wp_id);
    let wp = lookup_wp_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
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
    pane_owner: &Rc<std::cell::UnsafeCell<window_pane>>,
    client_owner: Option<&Rc<std::cell::UnsafeCell<client>>>,
    mut fs: *mut cmd_find_state,
    mut msg: *const ::core::ffi::c_char,
    mut input: *const ::core::ffi::c_char,
    mut inputcb: status_prompt_input_cb,
    mut freecb: prompt_free_cb,
    mut flags: ::core::ffi::c_int,
    mut type_0: prompt_type,
) {
    let wp = pane_owner.get();
    let session_owner = client_owner.and_then(|owner| (*owner.get()).session.upgrade());
    let mut pd = prompt_create_data::default();
    window_pane_clear_prompt(pane_owner);
    let wpp = refbox::RefBox::new(window_pane_prompt {
        wp_id: (*wp).id,
        c: client_owner.map(Rc::downgrade).unwrap_or_default(),
        inputcb,
        freecb,
        type_0,
    });
    prompt_set_options(&mut pd, session_owner.as_ref().map(|owner| &mut *owner.get()));
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
pub unsafe fn window_pane_clear_prompt(owner: &Rc<std::cell::UnsafeCell<window_pane>>) {
    let wp = owner.get();
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
pub fn window_pane_has_prompt(wp: &window_pane) -> ::core::ffi::c_int {
    wp.prompt.is_some() as ::core::ffi::c_int
}
pub unsafe fn window_pane_update_prompt(
    pane_owner: &Rc<std::cell::UnsafeCell<window_pane>>,
    mut msg: *const ::core::ffi::c_char,
    mut input: *const ::core::ffi::c_char,
) {
    let wp = pane_owner.get();
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
    pane_owner: &Rc<std::cell::UnsafeCell<window_pane>>,
    client_owner: Option<&Rc<std::cell::UnsafeCell<client>>>,
    mut key: key_code,
    mut m: *mut mouse_event,
) -> prompt_key_result {
    let mut wp = pane_owner.get();
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
        wpp.try_borrow_mut().expect("live prompt callback record").c = client_owner.map(Rc::downgrade).unwrap_or_default();
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
            if client_owner.is_some_and(|owner| status_at_line(&*owner.get()) == 0) {
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
    let lookup_wp_owner = window_pane_find_by_id(wp_id);
    wp = lookup_wp_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
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
        window_pane_clear_prompt(lookup_wp_owner.as_ref().expect("live prompt pane"));
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
    owner: &std::rc::Rc<std::cell::UnsafeCell<window_pane>>,
    bytes: &[u8],
) {
    let wp = owner.get();
    let window_owner = (*(*wp).window).observer.upgrade().expect("pane window");
    let mut cursor = window_pane_first((window_owner.get()).as_ref());
    while let Some(pane_owner) = cursor {
        let loop_0 = pane_owner.get();
        if loop_0 != wp
            && (*loop_0).modes.active_ptr().is_null()
            && (*loop_0).fd != -(1 as ::core::ffi::c_int)
            && !(*loop_0).flags & PANE_INPUTOFF != 0
            && window_pane_is_visible(&*loop_0) != 0
            && options_get_number(
                options_owner_ptr(&mut (*loop_0).options).map_or(std::ptr::null_mut(), |options| options),
                b"synchronize-panes\0" as *const u8 as *const ::core::ffi::c_char,
            ) != 0
        {
            log_debug(format_args!(
                "{}: {}",
                "window_pane_copy_paste",
                log_cstr_n(bytes.as_ptr().cast(), bytes.len() as ::core::ffi::c_int)
            ));
            let _ = (*loop_0).event.with_ptr(|event| unsafe {
                bufferevent_write(event, bytes.as_ptr().cast(), bytes.len());
            });
        }
        cursor = window_pane_next(loop_0.as_ref());
    }
}
unsafe fn window_pane_copy_key(owner: &std::rc::Rc<std::cell::UnsafeCell<window_pane>>, key: key_code) {
    let wp = owner.get();
    let window_owner = (*(*wp).window).observer.upgrade().expect("pane window");
    let mut cursor = window_pane_first((window_owner.get()).as_ref());
    while let Some(pane_owner) = cursor {
        let loop_0 = pane_owner.get();
        if loop_0 != wp
            && (*loop_0).modes.active_ptr().is_null()
            && (*loop_0).fd != -(1 as ::core::ffi::c_int)
            && !(*loop_0).flags & PANE_INPUTOFF != 0
            && window_pane_is_visible(&*loop_0) != 0
            && options_get_number(
                options_owner_ptr(&mut (*loop_0).options).map_or(std::ptr::null_mut(), |options| options),
                b"synchronize-panes\0" as *const u8 as *const ::core::ffi::c_char,
            ) != 0
        {
            input_key_pane(&pane_owner, key, ::core::ptr::null_mut::<mouse_event>());
        }
        cursor = window_pane_next(loop_0.as_ref());
    }
}
pub unsafe fn window_pane_paste(
    pane_owner: &std::rc::Rc<std::cell::UnsafeCell<window_pane>>,
    mut key: key_code,
    bytes: &[u8],
) {
    let wp = pane_owner.get();
    if !(*wp).modes.active_ptr().is_null() {
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
        && !(*(*wp).screen_ptr()).mode & MODE_BRACKETPASTE != 0
    {
        return;
    }
    log_debug(format_args!(
        "{}: {}",
        "window_pane_paste",
        log_cstr_n(bytes.as_ptr().cast(), bytes.len() as ::core::ffi::c_int)
    ));
    let _ = (*wp).event.with_ptr(|event| unsafe {
        bufferevent_write(event, bytes.as_ptr().cast(), bytes.len());
    });
    if options_get_number(
        options_owner_ptr(&mut (*wp).options).map_or(std::ptr::null_mut(), |options| options),
        b"synchronize-panes\0" as *const u8 as *const ::core::ffi::c_char,
    ) != 0
    {
        window_pane_copy_paste(pane_owner, bytes);
    }
}
pub unsafe fn window_pane_key(
    pane_owner: &std::rc::Rc<std::cell::UnsafeCell<window_pane>>,
    client_owner: Option<&std::rc::Rc<std::cell::UnsafeCell<client>>>,
    mut wl: *mut winlink,
    mut key: key_code,
    mut m: *mut mouse_event,
) -> ::core::ffi::c_int {
    let wp = pane_owner.get();
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
    wme = (*wp).modes.active_ptr();
    if !wme.is_null() {
        if key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
            == (KEYC_TYPE_MOUSEMOVE as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                << 32 as ::core::ffi::c_int
        {
            return 0 as ::core::ffi::c_int;
        }
        if let (Some(callback), Some(client)) = ((*(*wme).mode).key, client_owner) {
            key &= !KEYC_MASK_FLAGS;
            callback(wme, client, wl, key, m);
        }
        return 0 as ::core::ffi::c_int;
    }
    if (*wp).fd == -(1 as ::core::ffi::c_int) || (*wp).flags & PANE_INPUTOFF != 0 {
        return 0 as ::core::ffi::c_int;
    }
    if input_key_pane(pane_owner, key, m) != 0 as ::core::ffi::c_int {
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
        options_owner_ptr(&mut (*wp).options).map_or(std::ptr::null_mut(), |options| options),
        b"synchronize-panes\0" as *const u8 as *const ::core::ffi::c_char,
    ) != 0
    {
        window_pane_copy_key(pane_owner, key);
    }
    return 0 as ::core::ffi::c_int;
}
pub unsafe fn window_pane_is_visible(wp: &window_pane) -> ::core::ffi::c_int {
    if !(*wp.window).flags & WINDOW_ZOOMED != 0 {
        return 1 as ::core::ffi::c_int;
    }
    return (!wp.layout_cell.is_null()) as ::core::ffi::c_int;
}
pub fn window_pane_exited(wp: &window_pane) -> ::core::ffi::c_int {
    (wp.fd == -1 || wp.flags & PANE_EXITED != 0) as ::core::ffi::c_int
}
pub unsafe fn window_pane_search(
    wp: &window_pane,
    term: &CStr,
    mut regex: ::core::ffi::c_int,
    mut ignore: ::core::ffi::c_int,
) -> u_int {
    let grid = wp.base.grid();
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
    while i < grid.sy {
        let mut line = grid_view_string_cells_bytes(grid, i, grid.sx);
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
    if i == grid.sy {
        return 0 as u_int;
    }
    return i.wrapping_add(1 as u_int);
}
unsafe fn window_pane_choose_best(list: &[Rc<std::cell::UnsafeCell<window_pane>>]) -> Option<Rc<std::cell::UnsafeCell<window_pane>>> {
    let (first, rest) = list.split_first()?;
    let mut best = first;
    for next in rest {
        if (*next.get()).active_point > (*best.get()).active_point {
            best = next;
        }
    }
    Some(best.clone())
}
unsafe fn window_pane_full_size_offset(
    pane_owner: &Rc<std::cell::UnsafeCell<window_pane>>,
) -> (::core::ffi::c_int, ::core::ffi::c_int, u_int, u_int) {
    let wp = pane_owner.get();
    let sb_w = if window_pane_scrollbar_reserve(&*wp) != 0 {
        ((*wp).scrollbar_style.width + (*wp).scrollbar_style.pad) as u_int
    } else {
        0
    };
    let xoff = if (*(*wp).window).sb_pos == PANE_SCROLLBARS_LEFT {
        ((*wp).xoff as u_int).wrapping_sub(sb_w) as ::core::ffi::c_int
    } else {
        (*wp).xoff
    };
    (xoff, (*wp).yoff, (*wp).sx.wrapping_add(sb_w), (*wp).sy)
}
pub unsafe fn window_pane_find_up(source: Option<&Rc<std::cell::UnsafeCell<window_pane>>>) -> Option<Rc<std::cell::UnsafeCell<window_pane>>> {
    let source = source?;
    let wp = source.get();
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut next: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut list = Vec::new();
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
    w = (*wp).window as *mut window;
    status = window_get_pane_status(&*w);
    (xoff, yoff, sx, sy) = window_pane_full_size_offset(source);
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
    for candidate in (*w).panes.snapshot() {
        next = candidate.get();
        (xoff, yoff, sx, sy) = window_pane_full_size_offset(&candidate);
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
                    list.push(candidate);
                }
            }
        }
    }
    window_pane_choose_best(&list)
}
pub unsafe fn window_pane_find_down(source: Option<&Rc<std::cell::UnsafeCell<window_pane>>>) -> Option<Rc<std::cell::UnsafeCell<window_pane>>> {
    let source = source?;
    let wp = source.get();
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut next: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut list = Vec::new();
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
    w = (*wp).window as *mut window;
    status = window_get_pane_status(&*w);
    (xoff, yoff, sx, sy) = window_pane_full_size_offset(source);
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
    for candidate in (*w).panes.snapshot() {
        next = candidate.get();
        (xoff, yoff, sx, sy) = window_pane_full_size_offset(&candidate);
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
                    list.push(candidate);
                }
            }
        }
    }
    window_pane_choose_best(&list)
}
pub unsafe fn window_pane_find_left(source: Option<&Rc<std::cell::UnsafeCell<window_pane>>>) -> Option<Rc<std::cell::UnsafeCell<window_pane>>> {
    let source = source?;
    let wp = source.get();
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut next: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut list = Vec::new();
    let mut edge: ::core::ffi::c_int = 0;
    let mut top: ::core::ffi::c_int = 0;
    let mut bottom: ::core::ffi::c_int = 0;
    let mut end: ::core::ffi::c_int = 0;
    let mut found: ::core::ffi::c_int = 0;
    let mut xoff: ::core::ffi::c_int = 0;
    let mut yoff: ::core::ffi::c_int = 0;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    w = (*wp).window as *mut window;
    (xoff, yoff, sx, sy) = window_pane_full_size_offset(source);
    edge = xoff;
    if edge == 0 as ::core::ffi::c_int {
        edge = (*w).sx as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
    }
    top = yoff;
    bottom = yoff + sy as ::core::ffi::c_int;
    for candidate in (*w).panes.snapshot() {
        next = candidate.get();
        (xoff, yoff, sx, sy) = window_pane_full_size_offset(&candidate);
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
                    list.push(candidate);
                }
            }
        }
    }
    window_pane_choose_best(&list)
}
pub unsafe fn window_pane_find_right(source: Option<&Rc<std::cell::UnsafeCell<window_pane>>>) -> Option<Rc<std::cell::UnsafeCell<window_pane>>> {
    let source = source?;
    let wp = source.get();
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut next: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut list = Vec::new();
    let mut edge: ::core::ffi::c_int = 0;
    let mut top: ::core::ffi::c_int = 0;
    let mut bottom: ::core::ffi::c_int = 0;
    let mut end: ::core::ffi::c_int = 0;
    let mut found: ::core::ffi::c_int = 0;
    let mut xoff: ::core::ffi::c_int = 0;
    let mut yoff: ::core::ffi::c_int = 0;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    w = (*wp).window as *mut window;
    (xoff, yoff, sx, sy) = window_pane_full_size_offset(source);
    edge = xoff + sx as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
    if edge >= (*w).sx as ::core::ffi::c_int {
        edge = 0 as ::core::ffi::c_int;
    }
    top = (*wp).yoff;
    bottom = (*wp).yoff + (*wp).sy as ::core::ffi::c_int;
    for candidate in (*w).panes.snapshot() {
        next = candidate.get();
        (xoff, yoff, sx, sy) = window_pane_full_size_offset(&candidate);
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
                    list.push(candidate);
                }
            }
        }
    }
    window_pane_choose_best(&list)
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
    if !wp.is_null() && (*stack).remove(&(*wp).observer) {
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
            if let Some(session_owner) = (*loop_0).session.upgrade() {
                server_status_session(&*(session_owner.get()));
            }
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
    let lookup_wp_owner = window_pane_find_by_id(cdata.wp);
    let wp = lookup_wp_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    if wp.is_null() {
        (*c).retval = 1;
        (*c).flags |= CLIENT_EXIT as uint64_t;
    }
    let file = cdata.file.upgrade();
    if file.is_some()
        && !closed
        && (wp.is_null() || (*c).flags & CLIENT_DEAD as uint64_t != 0 || error != 0)
    {
        file_cancel(&mut *file.as_ref().unwrap().get());
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
    let c_owner = cmdq_get_client(item);
    let c = c_owner.as_ref().map_or(std::ptr::null_mut(), crate::src::shared::rc::as_ptr);
    if (*wp).flags & PANE_EMPTY == 0 {
        return Err(c"pane is not empty".to_owned());
    }
    if (*c).flags & (CLIENT_DEAD | CLIENT_EXITED) as uint64_t != 0 {
        return Ok(1);
    }
    if !(*c).session_ptr().is_null() {
        return Ok(1);
    }
    // The file initializer publishes its weak identity before dispatch. The
    // callback then owns the box, including its retained client reference.
    let mut cdata = Box::new(window_pane_input_data {
        item: NonNull::new(item).expect("pane input command"),
        client: Some(
            (*c).observer
                .upgrade()
                .expect("live pane input client"),
        ),
        wp: (*wp).id,
        file: Weak::new(),
    });
    file_read_with_cmdq_wait_init(
        c_owner.as_ref(),
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
pub fn window_pane_get_new_data<'a>(
    input: &'a mut evbuffer,
    base_offset: size_t,
    offset: &window_pane_offset,
) -> &'a [u8] {
    let used = offset.used.wrapping_sub(base_offset);
    let data = evbuffer_pullup(input, -1).unwrap_or_default();
    data.get(used..).expect("pane offset is within input buffer")
}
pub unsafe fn window_pane_update_used_data(
    mut wp: *mut window_pane,
    mut wpo: *mut window_pane_offset,
    mut size: size_t,
) {
    let used: size_t = (*wpo).used.wrapping_sub((*wp).base_offset);
    let Some(available) = (*wp).event.with_ptr(|event| unsafe {
        evbuffer_get_length(&*(*event).input)
    }) else { return };
    if size > available.saturating_sub(used) {
        size = available.saturating_sub(used);
    }
    (*wpo).used = (*wpo).used.wrapping_add(size);
}
pub unsafe fn window_pane_default_cursor(mut wp: *mut window_pane) {
    screen_set_default_cursor(&mut *(*wp).screen_ptr(), options_owner_ptr(&mut (*wp).options).map_or(std::ptr::null_mut(), |options| options));
}
pub unsafe fn window_pane_mode(wp: &window_pane) -> ::core::ffi::c_int {
    if !wp.modes.active_ptr().is_null() {
        if std::ptr::eq((*wp.modes.active_ptr()).mode, &window_copy_mode) {
            return 1 as ::core::ffi::c_int;
        }
        if std::ptr::eq((*wp.modes.active_ptr()).mode, &window_view_mode) {
            return 2 as ::core::ffi::c_int;
        }
    }
    return 0 as ::core::ffi::c_int;
}
pub unsafe fn window_pane_show_scrollbar(wp: &window_pane) -> ::core::ffi::c_int {
    let mut w: *mut window = wp.window as *mut window;
    let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
    if wp.base.saved_grid.is_some() {
        return 0 as ::core::ffi::c_int;
    }
    if (*w).flags & WINDOW_ZOOMED != 0 && !(*w).active_ptr().is_null() {
        wme = (*(*w).active_ptr()).modes.active_ptr();
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
pub unsafe fn window_pane_scrollbar_reserve(wp: &window_pane) -> ::core::ffi::c_int {
    if window_pane_show_scrollbar(wp) == 0 {
        return 0 as ::core::ffi::c_int;
    }
    return ((*wp.window).sb == PANE_SCROLLBARS_ALWAYS) as ::core::ffi::c_int;
}
pub unsafe fn window_pane_scrollbar_overlay(wp: &window_pane) -> ::core::ffi::c_int {
    if window_pane_show_scrollbar(wp) == 0 {
        return 0 as ::core::ffi::c_int;
    }
    return window_pane_scrollbar_auto_hide(wp);
}
pub unsafe fn window_pane_scrollbar_visible(wp: &window_pane) -> ::core::ffi::c_int {
    if window_pane_show_scrollbar(wp) == 0 {
        return 0 as ::core::ffi::c_int;
    }
    if window_pane_scrollbar_auto_hide(wp) == 0 {
        return 1 as ::core::ffi::c_int;
    }
    return wp.sb_auto_visible;
}
pub unsafe fn window_pane_scrollbar_start_timer(pane_owner: &Rc<std::cell::UnsafeCell<window_pane>>) {
    let wp = pane_owner.get();
    let mut tv: timeval = timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    let mut delay: u_int = 0;
    if window_pane_scrollbar_auto_hide(&*wp) == 0 || (*wp).sb_auto_visible == 0 {
        return;
    }
    delay = options_get_number(
        options_owner_ptr(&mut (*(*wp).window).options).map_or(std::ptr::null_mut(), |options| options),
        b"pane-scrollbars-timeout\0" as *const u8 as *const ::core::ffi::c_char,
    ) as u_int;
    tv.tv_sec = delay.wrapping_div(1000 as u_int) as __time_t;
    tv.tv_usec = (delay.wrapping_rem(1000 as u_int) as ::core::ffi::c_long
        * 1000 as ::core::ffi::c_long) as __suseconds_t;
    event_del(&raw mut (*wp).sb_auto_timer);
    event_add(&raw mut (*wp).sb_auto_timer, &raw mut tv);
}
pub unsafe fn window_pane_scrollbar_show(pane_owner: &Rc<std::cell::UnsafeCell<window_pane>>) {
    let wp = pane_owner.get();
    let mut changed: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if window_pane_scrollbar_auto_hide(&*wp) == 0 {
        return;
    }
    if window_pane_show_scrollbar(&*wp) == 0 {
        return;
    }
    if (*wp).sb_auto_visible == 0 {
        (*wp).sb_auto_visible = 1 as ::core::ffi::c_int;
        changed = 1 as ::core::ffi::c_int;
    }
    event_del(&raw mut (*wp).sb_auto_timer);

    window_pane_scrollbar_start_timer(pane_owner);

    if changed != 0 {
        window_pane_scrollbar_redraw_visibility(pane_owner);
    }
}
pub unsafe fn window_pane_scrollbar_hide(pane_owner: &Rc<std::cell::UnsafeCell<window_pane>>) {
    let wp = pane_owner.get();
    if event_initialized(&(*wp).sb_auto_timer) != 0 {
        event_del(&raw mut (*wp).sb_auto_timer);
    }
    (*wp).sb_auto_hover = 0 as ::core::ffi::c_int;
    if (*wp).sb_auto_visible == 0 {
        return;
    }
    (*wp).sb_auto_visible = 0 as ::core::ffi::c_int;
    window_pane_scrollbar_redraw_visibility(pane_owner);
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
    let mut registry_loop_0_owner = clients.first();
    loop_0 = registry_loop_0_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    while !loop_0.is_null() {
        if !((*loop_0).flags & CLIENT_UNATTACHEDFLAGS as uint64_t != 0) {
            if !((*loop_0).session_ptr().is_null() || session_has(&*(*loop_0).session_ptr(), &*w) == 0) {
                if !((*loop_0).tty.bg == -(1 as ::core::ffi::c_int)) {
                    return (*loop_0).tty.bg;
                }
            }
        }
        registry_loop_0_owner = clients.next(registry_loop_0_owner.as_ref().expect("current registry client"));
        loop_0 = registry_loop_0_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    }
    return -(1 as ::core::ffi::c_int);
}
pub unsafe fn window_pane_get_bg_control_client(mut wp: *mut window_pane) -> ::core::ffi::c_int {
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    if (*wp).control_bg == -(1 as ::core::ffi::c_int) {
        return -(1 as ::core::ffi::c_int);
    }
    let mut registry_c_owner = clients.first();
    c = registry_c_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    while !c.is_null() {
        if (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
            return (*wp).control_bg;
        }
        registry_c_owner = clients.next(registry_c_owner.as_ref().expect("current registry client"));
        c = registry_c_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    }
    return -(1 as ::core::ffi::c_int);
}
pub unsafe fn window_pane_get_fg(mut wp: *mut window_pane) -> ::core::ffi::c_int {
    let mut w: *mut window = (*wp).window as *mut window;
    let mut loop_0: *mut client = ::core::ptr::null_mut::<client>();
    let mut registry_loop_0_owner = clients.first();
    loop_0 = registry_loop_0_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    while !loop_0.is_null() {
        if !((*loop_0).flags & CLIENT_UNATTACHEDFLAGS as uint64_t != 0) {
            if !((*loop_0).session_ptr().is_null() || session_has(&*(*loop_0).session_ptr(), &*w) == 0) {
                if !((*loop_0).tty.fg == -(1 as ::core::ffi::c_int)) {
                    return (*loop_0).tty.fg;
                }
            }
        }
        registry_loop_0_owner = clients.next(registry_loop_0_owner.as_ref().expect("current registry client"));
        loop_0 = registry_loop_0_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    }
    return -(1 as ::core::ffi::c_int);
}
pub unsafe fn window_pane_get_fg_control_client(mut wp: *mut window_pane) -> ::core::ffi::c_int {
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    if (*wp).control_fg == -(1 as ::core::ffi::c_int) {
        return -(1 as ::core::ffi::c_int);
    }
    let mut registry_c_owner = clients.first();
    c = registry_c_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    while !c.is_null() {
        if (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
            return (*wp).control_fg;
        }
        registry_c_owner = clients.next(registry_c_owner.as_ref().expect("current registry client"));
        c = registry_c_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
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
    let mut registry_loop_0_owner = clients.first();
    loop_0 = registry_loop_0_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    while !loop_0.is_null() {
        if !((*loop_0).flags & CLIENT_UNATTACHEDFLAGS as uint64_t != 0) {
            if !((*loop_0).session_ptr().is_null() || session_has(&*(*loop_0).session_ptr(), &*w) == 0) {
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
        registry_loop_0_owner = clients.next(registry_loop_0_owner.as_ref().expect("current registry client"));
        loop_0 = registry_loop_0_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    }
    if found_dark != 0 && found_light == 0 {
        return THEME_DARK;
    }
    if found_light != 0 && found_dark == 0 {
        return THEME_LIGHT;
    }
    return colour_totheme(window_pane_get_bg(wp));
}
pub unsafe fn window_pane_send_theme_update(pane_owner: &Rc<std::cell::UnsafeCell<window_pane>>) {
    let wp = pane_owner.get();
    let mut theme: client_theme = THEME_UNKNOWN;
    if window_pane_exited(&*wp) != 0 {
        return;
    }
    if !(*wp).flags & PANE_THEMECHANGED != 0 {
        return;
    }
    if !(*(*wp).screen_ptr()).mode & MODE_THEME_UPDATES != 0 {
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
            let _ = (*wp).event.with_ptr(|event| unsafe { bufferevent_write(
                event,
                b"\x1B[?997;2n\0" as *const u8 as *const ::core::ffi::c_char
                    as *const ::core::ffi::c_void,
                9 as size_t,
            ) });
        }
        2 => {
            log_debug(format_args!(
                "{}: %{} dark theme",
                "window_pane_send_theme_update",
                ((*wp).id) as u32
            ));
            let _ = (*wp).event.with_ptr(|event| unsafe { bufferevent_write(
                event,
                b"\x1B[?997;1n\0" as *const u8 as *const ::core::ffi::c_char
                    as *const ::core::ffi::c_void,
                9 as size_t,
            ) });
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
    pane_owner: &Rc<std::cell::UnsafeCell<window_pane>>,
    x: u_int,
    y: u_int,
) -> Option<style_range> {
    let wp = pane_owner.get();
    let pane_status = window_pane_get_pane_status(&*wp);
    let line = if pane_status == PANE_STATUS_TOP {
        ((*wp).yoff - 1) as u_int
    } else if pane_status == PANE_STATUS_BOTTOM {
        ((*wp).yoff as u_int).wrapping_add((*wp).sy)
    } else {
        0
    };
    if pane_status == PANE_STATUS_OFF || line != y {
        return None;
    }
    let x = x.wrapping_sub((*wp).xoff as u_int).wrapping_sub(2);
    (*wp).border_status_line.ranges.as_slice().iter()
        .find(|range| x >= range.start && x < range.end)
        .map(|range| **range)
}
pub unsafe fn window_get_pane_lines(w: &window) -> pane_lines {
    options_get_number_ref(w.options.as_deref().expect("window options"), c"pane-border-lines") as pane_lines
}
pub unsafe fn window_pane_get_pane_lines(wp: &window_pane) -> pane_lines {
    let options = if window_pane_is_floating(wp) == 0 {
        (*wp.window).options.as_deref().expect("window options")
    } else {
        wp.options.as_deref().expect("pane options")
    };
    options_get_number_ref(options, c"pane-border-lines") as pane_lines
}
pub unsafe fn window_get_pane_status(w: &window) -> ::core::ffi::c_int {
    let status = options_get_number_ref(
        w.options.as_deref().expect("window options"), c"pane-border-status",
    ) as ::core::ffi::c_int;
    if status == PANE_STATUS_TOP_FLOATING || status == PANE_STATUS_BOTTOM_FLOATING {
        return 0;
    }
    status
}
pub unsafe fn window_pane_get_pane_status(wp: &window_pane) -> ::core::ffi::c_int {
    let wme = wp.modes.active_ptr();
    if !wme.is_null()
        && (*(*wme).mode).flags & WINDOW_MODE_HIDE_PANE_STATUS != 0
        && wp.flags & PANE_ZOOMED != 0
    {
        return 0;
    }
    if window_pane_is_floating(wp) == 0 {
        return window_get_pane_status(&*wp.window);
    }
    if window_pane_get_pane_lines(wp) == PANE_LINES_NONE as pane_lines {
        return 0;
    }
    let status = options_get_number_ref(
        wp.options.as_deref().expect("pane options"), c"pane-border-status",
    ) as ::core::ffi::c_int;
    if status == PANE_STATUS_TOP_FLOATING {
        return 1;
    }
    if status == PANE_STATUS_BOTTOM_FLOATING {
        return 2;
    }
    status
}
pub unsafe fn window_pane_is_floating(wp: &window_pane) -> ::core::ffi::c_int {
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
            let wp_owner = window_pane::new();
            let wp = crate::src::shared::rc::as_ptr(&wp_owner);
            let weak = window_pane_weak(wp);
            let callback = window_pane_add_ref(wp, c"callback".as_ptr());
            window_pane_remove_ref(wp_owner, c"pane shutdown".as_ptr());
            let retained = weak.upgrade().expect("callback keeps the pane alive");
            assert_eq!(crate::src::shared::rc::as_ptr(&retained), wp);
            window_pane_remove_ref(callback, c"callback complete".as_ptr());
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
                window_pane::new();
            let first = crate::src::shared::rc::as_ptr(&first_owner);
            (*first).id = 1;
            let second_owner =
                window_pane::new();
            let second = crate::src::shared::rc::as_ptr(&second_owner);
            (*second).id = 2;
            let duplicate_owner =
                window_pane::new();
            let duplicate = crate::src::shared::rc::as_ptr(&duplicate_owner);
            (*duplicate).id = 1;

            assert!(window_pane_tree_insert(&mut head, first_owner.clone()).is_none());
            assert!(window_pane_tree_insert(&mut head, second_owner.clone()).is_none());
            let index_observer = (*first).tree_entry.owner.as_ref().unwrap().clone();
            assert!(Rc::ptr_eq(&window_pane_tree_insert(&mut head, duplicate_owner.clone()).unwrap(), &first_owner));
            assert!((*duplicate).tree_entry.owner.is_none());
            assert!(window_pane_tree_remove(&mut other, &mut *first).is_none());
            assert!((*first).tree_entry.owner.is_some());

            let mut moved = head;
            assert!(Rc::ptr_eq(&window_pane_tree_next(&*first).unwrap(), &second_owner));
            assert_eq!(crate::src::shared::rc::as_ptr(&window_pane_tree_remove(&mut moved, &mut *first).unwrap()), first);
            assert!((*first).tree_entry.owner.is_none());
            assert!(window_pane_tree_next(&*first).is_none());
            assert_eq!(crate::src::shared::rc::as_ptr(&window_pane_tree_remove(&mut moved, &mut *second).unwrap()), second);

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
            let pane = window_pane::new();
            let wp = rc::as_ptr(&pane);
            (*wp).id = u_int::MAX - 1;
            // This fixture exercises cleanup without firing pane hook events.
            (*wp).flags = PANE_DESTROYED;
            assert!(window_pane_tree_insert(&mut *std::ptr::addr_of_mut!(all_window_panes), pane.clone()).is_none());
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
            let callback_pane = Rc::downgrade(&pane);
            old_data.try_borrow_mut().unwrap().inputcb = Some(Box::new(move |_, text, kind| {
                let pane = callback_pane.upgrade().expect("prompt pane");
                let wp = pane.get();
                assert_eq!(text, Some(c"x"));
                assert_eq!(kind, PROMPT_KEY_CLOSE);
                window_pane_clear_prompt(&pane);
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
                    &pane,
                    None,
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
            window_pane_clear_prompt(&pane);
            assert!((*wp).prompt_data.is_none());
            assert!(!replacement_observer.is_alive());
            drop(replacement);
            assert!(!replacement_weak.is_alive());
            assert_eq!(rc::as_ptr(&window_pane_tree_remove(&mut *std::ptr::addr_of_mut!(all_window_panes), &mut *wp).unwrap()), wp);
        }
    }
}

#[cfg(test)]
mod pane_input_owner_tests {
    use super::*;
    use crate::src::cmd::queue::{cmdq_free_detached, cmdq_get_callback_owned};
    use crate::src::file::{file_fire_done, file_fire_read};
    use crate::src::reactor::{evbuffer_add, event_loop, shutdown_runtime};
    use crate::src::shared::command::CMDQ_WAITING;
    use crate::src::shared::rc;

    unsafe fn waiting_input(
        item: *mut cmdq_item,
        client: &Rc<UnsafeCell<client>>,
    ) -> Rc<UnsafeCell<client_file>> {
        let mut data = Box::new(window_pane_input_data {
            item: NonNull::new(item).expect("test queue item"),
            client: Some(Rc::clone(client)),
            wp: u_int::MAX,
            file: Weak::new(),
        });
        let owner = client_file::new();
        let file = rc::as_ptr(&owner);
        (*file).wait_item = (*item).observer.clone();
        (*file).wait_active = true;
        (*file).wait_client = Some(Rc::downgrade(client));
        data.file = (*file).observer.clone();
        (*file).cb = data.into_callback();
        (*item).wait_file = Some((*file).observer.clone());
        (*item).flags = CMDQ_WAITING;
        owner
    }

    #[test]
    fn progress_cancellation_keeps_record_and_client_until_terminal_dispatch() {
        unsafe {
            let client = client::new();
            let client_observer = Rc::downgrade(&client);
            let item = cmdq_get_callback_owned(c"pane input test".as_ptr(), None);
            let owner = waiting_input(item, &client);
            let file = rc::as_ptr(&owner);
            let file_observer = (*file).observer.clone();
            // Cancellation was already requested. Further progress must drain
            // the bytes and keep waiting until the peer sends its terminal event.
            (*file).closed = 1;
            evbuffer_add(&mut (*file).buffer, b"discard".as_ptr().cast(), 7);
            drop(client);

            file_fire_read(&owner);
            assert_eq!(evbuffer_get_length(&(*file).buffer), 0);
            assert_ne!((*item).flags & CMDQ_WAITING, 0);
            assert!((*file).cb.is_some());
            assert_eq!(
                (*file).observer.strong_count(),
                1,
                "the callback only weakly observes its file"
            );
            {
                let client = client_observer.upgrade().unwrap();
                assert_eq!((*client.get()).retval, 1);
                assert_ne!((*client.get()).flags & CLIENT_EXIT as u64, 0);
            }

            file_fire_done(&owner);
            file_fire_done(&owner);
            assert!((*file).cb.is_some());
            drop(owner);
            event_loop();
            assert_eq!((*item).flags & CMDQ_WAITING, 0);
            assert!((*item).wait_file.is_none());
            assert!(file_observer.upgrade().is_none());
            assert!(client_observer.upgrade().is_none());
            cmdq_free_detached(item);
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
                let item = cmdq_get_callback_owned(c"pane input test".as_ptr(), None);
                (*item).client = Rc::downgrade(&owner);
                (*item).flags = CMDQ_WAITING;
                let mut pane = window_pane::empty();
                pane.flags = PANE_EMPTY;
                pane.id = u_int::MAX;
                assert_eq!(window_pane_start_input(&mut pane, item), Ok(0));
                let file_owner = (*item).wait_file.as_ref().unwrap().upgrade().unwrap();
                let file = rc::as_ptr(&file_owner);
                assert!(!file.is_null());
                assert!((*file).cb.is_some());
                let file_observer = (*file).observer.clone();
                drop(owner);
                if cancel {
                    crate::src::file::file_cancel_cmdq_wait(&file_owner);
                    assert!((*file).cb.is_none());
                }
                drop(file_owner);
                event_loop();
                assert!((*item).wait_file.is_none());
                if !cancel {
                    assert_eq!((*item).flags & CMDQ_WAITING, 0);
                }
                assert!(file_observer.upgrade().is_none());
                assert!(observer.upgrade().is_none());
                cmdq_free_detached(item);
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
                let item = cmdq_get_callback_owned(c"pane input test".as_ptr(), None);
                let owner = waiting_input(item, &client);
            let file = rc::as_ptr(&owner);
                let file_observer = (*file).observer.clone();
                (*file).error = libc::EBADF;
                    drop(client);
                file_fire_done(&owner);
                file_fire_done(&owner);
                drop(owner);
                event_loop();
                assert_eq!((*item).flags & CMDQ_WAITING != 0, dead);
                assert!((*item).wait_file.is_none());
                assert!(file_observer.upgrade().is_none());
                assert!(client_observer.upgrade().is_none());
                cmdq_free_detached(item);
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
            let pane_owner = window_pane::new();
            let pane = crate::src::shared::rc::as_ptr(&pane_owner);
            (*pane).fd = -1;
            (*pane).pipe_fd = -1;
            let observer = window_pane_weak(pane);
            assert_eq!((*pane).observer.strong_count(), 1);
            let guard = window_pane_upgrade(&observer).unwrap();
            assert_eq!(rc::as_ptr(&guard), pane);

            window_pane_tree_insert(&mut *std::ptr::addr_of_mut!(all_window_panes), pane_owner);
            window_pane_destroy(&observer.upgrade().expect("registered pane"));

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
            let pane_owner = window_pane::new();
            let pane = crate::src::shared::rc::as_ptr(&pane_owner);
            let observer = window_pane_weak(pane);
            (*pane).fd = -1;
            (*pane).pipe_fd = -1;
            (*pane).flags = PANE_EMPTY;
            window_pane_set_event(pane);
            assert!((*pane).ictx.is_some());
            let stale_stream = (*pane).event.clone();
            assert!(stale_stream.is_alive());
            let callback = (*pane).event.with_ptr(|event| unsafe {
                Rc::downgrade((*event).readcb.as_ref().unwrap())
            }).unwrap();

            window_pane_tree_insert(&mut *std::ptr::addr_of_mut!(all_window_panes), pane_owner);
            window_pane_destroy(&observer.upgrade().expect("registered pane"));

            assert!(observer.upgrade().is_none());
            assert!(callback.upgrade().is_none());
            assert!(!stale_stream.is_alive());
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

    unsafe fn zoomed_window() -> Rc<std::cell::UnsafeCell<window>> {
        let w_owner = window::new();
        let w = rc::as_ptr(&w_owner);
        (*w).options = Some(crate::src::options::options_create_owned(std::ptr::null_mut()));
        let entry = options_table
            .iter()
            .find(|entry| {
                entry.name == Some(c"pane-border-status")
            })
            .unwrap();
        options_default(options_owner_ptr(&mut (*w).options).map_or(std::ptr::null_mut(), |options| options), entry);
        let pane_owner = window_pane::new();
            let pane = crate::src::shared::rc::as_ptr(&pane_owner);
        window_pane_tree_insert(&mut *std::ptr::addr_of_mut!(all_window_panes), pane_owner);
        (*pane).window = w;
        (*pane).fd = -1;
        (*pane).pipe_fd = -1;
        (*pane).sx = 80;
        (*pane).sy = 24;
        (*pane).base.grid = Some(grid_create(80, 24, 0));
        (*pane).screen_source = PaneScreenSource::Base;
        (*pane).flags = PANE_ZOOMED;
        (*w).set_active(pane);
        (*w).panes.push_back((*pane).observer.clone());
        (*w).z_index.push_back((*pane).observer.clone());
        let mut saved = layout_create_cell();
        layout_set_size(&mut *saved, 40, 24, 0, 0);
        layout_make_leaf(&mut *saved, &(*pane).observer.upgrade().unwrap());
        (*pane).saved_layout_cell = &mut *saved;
        let mut zoomed = layout_create_cell();
        layout_set_size(&mut *zoomed, 80, 24, 0, 0);
        layout_make_leaf(&mut *zoomed, &(*pane).observer.upgrade().unwrap());
        (*w).layout_root = Some(zoomed);
        (*w).saved_layout_root = Some(saved);
        (*w).flags = WINDOW_ZOOMED;
        w_owner
    }

    #[test]
    fn mode_tree_cleanup_unzooms_a_logically_destroyed_pane() {
        unsafe {
            let w_owner = zoomed_window();
            let w = rc::as_ptr(&w_owner);
            let pane = (*w).active_ptr();
            let tree_owner = std::rc::Rc::new_cyclic(|observer| std::cell::UnsafeCell::new(crate::src::shared::mode_tree::mode_tree_data {
                observer: observer.clone(),
                wp: window_pane_weak(pane),
                zoomed: 0,
                ..Default::default()
            }));
            let tree = rc::as_ptr(&tree_owner);
            let observed = (*tree).observer.clone();
            (*pane).flags |= PANE_DESTROYED;
            assert!(window_pane_upgrade(&(*tree).wp).is_none());

            crate::src::mode_tree::mode_tree_free(tree_owner);

            assert!(observed.upgrade().is_none());
            assert_eq!((*w).flags & WINDOW_ZOOMED, 0);
            drop(w_owner);
        }
    }

    #[test]
    fn plain_and_notifying_rc_drops_destroy_zoomed_windows_without_resize_events() {
        unsafe {
            for typed in [false, true] {
                let w_owner = zoomed_window();
            let w = rc::as_ptr(&w_owner);
                let observer = (*w).observer.clone();
                let pane_observer = (*(*w).active_ptr()).observer.clone();
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
                    drop(w_owner);
                } else {
                    window_remove_ref(w_owner, c"test final notifying owner".as_ptr());
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
            let w_owner = zoomed_window();
            let w = rc::as_ptr(&w_owner);
            let pane = (*w).active_ptr();
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
            window_remove_ref(w_owner, c"test live close".as_ptr());
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
