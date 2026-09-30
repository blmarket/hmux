use crate::src::server_client::Client as _;
use crate::src::session::Session;
use crate::src::shared::client::ClientRef;
use crate::src::shared::window::WindowRef;
use std::time::SystemTime;
mod alerts;
mod api;
pub use api::{LayoutView, PaneLayoutGeometry, PaneOrder, Window, WindowResize, WindowScrollbars};

mod model;
pub use crate::src::window_pane::*;
pub use crate::src::winlink::*;
pub use model::window;
use model::WindowLifecycle;

use crate::src::alerts::alerts_queue;
use crate::src::arguments::args_has;
use crate::src::cmd::cmd_mouse_at;
use crate::src::cmd::find::{cmd_find_from_pane, cmd_find_from_window};
use crate::src::cmd::queue::{cmdq_continue, cmdq_get_client};
use crate::src::compat::strtonum::strtonum;
use crate::src::events::{events_fire, events_fire_pane, events_fire_window};
use crate::src::events_payload::{
    event_payload_create, event_payload_set_int, event_payload_set_pane, event_payload_set_string,
    event_payload_set_target, event_payload_set_uint, event_payload_set_window,
};
use crate::src::ffi::libc::{
    __ctype_b_loc, close, fnmatch, gethostname, getpid, ioctl, kill, memcpy, memset, strcasecmp,
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
use crate::src::options::options_owner_ptr;
use crate::src::options::{
    options_create, options_free, options_get_number, options_get_number_ref,
};
use crate::src::prompt::{
    prompt_closed, prompt_create, prompt_free, prompt_incremental_start, prompt_key, prompt_mouse,
    prompt_set_options, prompt_type_string, prompt_update,
};
use crate::src::reactor::BufferEvent;
use crate::src::reactor::{
    bufferevent_disable, bufferevent_enable, bufferevent_new, bufferevent_write, evbuffer_drain,
    evbuffer_get_length, evbuffer_pullup, event_add, event_del, event_initialized, event_set,
};
use crate::src::screen::{
    screen_free, screen_init, screen_resize, screen_set_default_cursor, screen_set_title,
};

use crate::src::server::clients;
use crate::src::server::{marked_pane, server_check_marked, server_clear_marked};
use crate::src::server_client::{server_client_unref_owned, Client};
use crate::src::server_fn::{
    server_destroy_pane, server_kill_pane, server_redraw_window, server_redraw_window_borders,
    server_status_session, server_status_window,
};
use crate::src::shared::events::event_payload;
use crate::src::shared::pane::window_pane_tree;
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
use hmux_buffer::SegmentedBuf;
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
    window_pane, window_pane_history, window_pane_modes, window_pane_prompt, window_panes,
    PaneScreenSource,
};
use crate::src::shared::pane::{
    window_pane_offset, window_pane_resize, window_pane_resizes, PANE_CHANGED, PANE_DESTROYED,
    PANE_EMPTY, PANE_EXITED, PANE_FLOATOVERZOOM, PANE_FOCUSED, PANE_INPUTOFF, PANE_REDRAW,
    PANE_REDRAWSCROLLBAR, PANE_SCROLLBARS_ALWAYS, PANE_SCROLLBARS_AUTOHIDE, PANE_SCROLLBARS_LEFT,
    PANE_SCROLLBARS_MODAL, PANE_STATUSREADY, PANE_STATUS_BOTTOM, PANE_STATUS_BOTTOM_FLOATING,
    PANE_STATUS_OFF, PANE_STATUS_TOP, PANE_STATUS_TOP_FLOATING, PANE_STYLECHANGED,
    PANE_THEMECHANGED, PANE_UNSEENCHANGES, PANE_ZOOMED,
};
use crate::src::shared::posix_io::FNM_CASEFOLD;
use crate::src::shared::posix_terminal::{winsize, TIOCSWINSZ};
use crate::src::shared::prompt::prompt;
use crate::src::shared::prompt::*;
use crate::src::shared::prompt::{prompt_free_cb, prompt_input_cb, prompt_result, PROMPT_CLOSE};
use crate::src::shared::screen::{screen, MODE_BRACKETPASTE, MODE_FOCUSON, MODE_THEME_UPDATES};
use crate::src::shared::session::session;
use crate::src::shared::signal::SIGCHLD;
use crate::src::shared::spawn::spawn_editor_state;
use crate::src::shared::spawn::{SPAWN_BEFORE, SPAWN_FLOATING, SPAWN_FULLSIZE};
use crate::src::shared::status::status_prompt_input_cb;
use crate::src::shared::style::*;
use crate::src::shared::tree::{RB_INF, RB_NEGINF};
pub use crate::src::shared::window::{
    window_mode, window_mode_entry, window_winlinks, windows, winlink, winlink_stack, winlinks,
};
use crate::src::shared::window::{
    WINDOW_ACTIVITY, WINDOW_ALERTFLAGS, WINDOW_MODE_HIDE_PANE_STATUS, WINDOW_MODE_HIDE_SCROLLBARS,
    WINDOW_MODE_NO_STACK, WINDOW_PANE_NO_MODE, WINDOW_ZOOMED, WINLINK_ACTIVITY, WINLINK_ALERTFLAGS,
    WINLINK_BELL, WINLINK_SILENCE, WINLINK_VISITED,
};
use libc::{REG_EXTENDED, REG_ICASE};

pub const DEFAULT_XPIXEL: ::core::ffi::c_int = 16 as ::core::ffi::c_int;
pub const DEFAULT_YPIXEL: ::core::ffi::c_int = 32 as ::core::ffi::c_int;
pub const WINDOW_WASZOOMED: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub static mut windows: windows = windows { storage: None };

static mut next_window_id: u_int = 0;

fn windows_find(head: &windows, elm: &window) -> Option<WindowRef> {
    let owner = head.storage.as_ref()?;
    let map = owner
        .try_borrow_mut()
        .expect("window index already borrowed");
    map.get(&elm.id)?.upgrade()
}

/// Register an observer, without adding a strong reference to the window.
pub unsafe fn windows_insert(head: *mut windows, window: &WindowRef) -> Option<WindowRef> {
    let elm = window.get();
    let owner = (*head).storage.get_or_insert_with(refbox::RefBox::default);
    let observer = owner.downgrade();
    let mut map = owner
        .try_borrow_mut()
        .expect("window index already borrowed");
    if let Some(existing) = map.get(&(*elm).id).and_then(std::rc::Weak::upgrade) {
        return Some(existing);
    }
    map.insert((*elm).id, Rc::downgrade(window));
    (*elm).owner = observer;
    None
}

/// Remove the matching observer without retaining or releasing the window.
pub unsafe fn windows_remove(head: &mut windows, elm: &WindowRef) -> bool {
    let Some(owner) = head.storage.as_ref() else {
        return false;
    };
    let empty = {
        let mut map = owner
            .try_borrow_mut()
            .expect("window index already borrowed");
        let id = (*elm.get()).id;
        if !map
            .get(&id)
            .is_some_and(|weak| weak.ptr_eq(&Rc::downgrade(elm)))
        {
            return false;
        }
        map.remove(&id);
        map.is_empty()
    };
    (*elm.get()).owner = refbox::Weak::new();
    if empty {
        head.storage = None;
    }
    true
}

pub fn windows_minmax(head: &windows) -> Option<WindowRef> {
    let owner = head.storage.as_ref()?;
    let map = owner
        .try_borrow_mut()
        .expect("window index already borrowed");
    map.values().find_map(std::rc::Weak::upgrade)
}

fn windows_next(elm: &window) -> Option<WindowRef> {
    let owner = &elm.owner;
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
}

/// Return the first winlink in a window's association order.
unsafe fn window_winlinks_first(w_value: Option<&window>) -> refbox::Weak<winlink> {
    let w: *mut window = w_value.map_or(std::ptr::null_mut(), |value| value as *const _ as *mut _);
    if w.is_null() {
        return refbox::Weak::new();
    }
    (*w).winlinks
        .first()
        .filter(|link| link.is_alive())
        .map_or(refbox::Weak::new(), |link| link.clone())
}

/// Return the next winlink in `w` after `wl`, or an empty handle when it is no longer in
/// that window. Taking the owner explicitly keeps iteration correct if a
/// callback moves `wl` to another window while the original list is traversed.
unsafe fn window_winlinks_next(
    w_value: Option<&window>,
    wl: refbox::Weak<winlink>,
) -> refbox::Weak<winlink> {
    let w: *mut window = w_value.map_or(std::ptr::null_mut(), |value| value as *const _ as *mut _);
    if w.is_null() || !wl.is_alive() {
        return refbox::Weak::new();
    }
    let links = &(*w).winlinks;
    links
        .iter()
        .position(|link| *link == wl)
        .and_then(|position| links.get(position + 1))
        .filter(|link| link.is_alive())
        .map_or(refbox::Weak::new(), |link| link.clone())
}

/// Append a non-owning winlink handle to the window's association order.
unsafe fn window_winlinks_append(w_value: &mut window, wl: refbox::Weak<winlink>) {
    let w: *mut window = w_value as *mut _;
    assert!(!w.is_null() && wl.is_alive());
    let links = &mut (*w).winlinks;
    assert!(
        !links.contains(&wl),
        "winlink is already present in this window"
    );
    links.push(wl);
}

/// Remove a non-owning winlink handle from its window's association order.
unsafe fn window_winlinks_remove(w_value: &mut window, wl: refbox::Weak<winlink>) {
    let w: *mut window = w_value as *mut _;
    assert!(!w.is_null() && wl.is_alive());
    let links = &mut (*w).winlinks;
    let position = links
        .iter()
        .position(|link| *link == wl)
        .expect("winlink must belong to its window");
    links.remove(position);
}

unsafe fn window_fire_renamed(w_owner: &WindowRef, mut old_name: *const ::core::ffi::c_char) {
    let mut w = w_owner.get();
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        s: std::rc::Weak::new(),
        wl: refbox::Weak::new(),
        w: std::rc::Weak::new(),
        wp: std::rc::Weak::new(),
        idx: 0,
    };
    let mut ep = event_payload_create();
    cmd_find_from_window(
        &raw mut fs,
        &(*(w)).observer.upgrade().expect("live window"),
        0 as ::core::ffi::c_int,
    );
    event_payload_set_target(&mut *ep, &fs);
    event_payload_set_window(
        &mut *ep,
        b"window\0" as *const u8 as *const ::core::ffi::c_char,
        (*(w)).observer.upgrade().expect("live window"),
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
    window: &WindowRef,
    pane: &Rc<UnsafeCell<window_pane>>,
    previous: Option<&Rc<UnsafeCell<window_pane>>>,
) {
    let mut find = cmd_find_state::default();
    let mut payload = event_payload_create();
    cmd_find_from_pane(&mut find, pane, 0);
    event_payload_set_target(&mut payload, &find);
    event_payload_set_window(&mut payload, c"window".as_ptr(), window.clone());
    event_payload_set_pane(&mut payload, c"pane".as_ptr(), pane.clone());
    event_payload_set_pane(&mut payload, c"new_pane".as_ptr(), pane.clone());
    if let Some(previous) = previous {
        event_payload_set_pane(&mut payload, c"old_pane".as_ptr(), previous.clone());
    }
    events_fire(c"window-pane-changed".as_ptr(), payload);
}

pub unsafe fn window_find_by_id_str(mut s: *const ::core::ffi::c_char) -> Option<WindowRef> {
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
pub unsafe fn window_find_by_id(id: u_int) -> Option<WindowRef> {
    let mut w = window::default();
    w.id = id;
    return windows_find(&windows, &w);
}
pub unsafe fn window_update_activity(w_owner: &WindowRef) {
    let mut w = w_owner.get();
    (*w).activity_time = SystemTime::now();
    alerts_queue(w_owner, WINDOW_ACTIVITY);
}

pub(crate) unsafe fn window_replace_old_layout(
    w_owner: &WindowRef,
    layout: Option<CString>,
) -> Option<CString> {
    let mut w = w_owner.get();
    ::core::mem::replace(&mut (*w).old_layout, layout)
}

/// Replace the window's owned name, returning the previous value for callers
/// that need to keep it alive across synchronous callbacks.
unsafe fn window_replace_name(w_owner: &WindowRef, name: CString) -> CString {
    let mut w = w_owner.get();
    ::core::mem::replace(&mut (*w).name, name)
}
/// Return one owned reference; callers must release it after linking the window.
pub unsafe fn window_create(
    mut sx: u_int,
    mut sy: u_int,
    mut xpixel: u_int,
    mut ypixel: u_int,
) -> WindowRef {
    if xpixel == 0 as u_int {
        xpixel = DEFAULT_XPIXEL as u_int;
    }
    if ypixel == 0 as u_int {
        ypixel = DEFAULT_YPIXEL as u_int;
    }
    let owner = window::new();
    let w = owner.get();
    (*w).flags = 0 as ::core::ffi::c_int;
    (*w).panes = window_panes::default();
    (*w).z_index = window_panes::default();
    (*w).last_panes = window_pane_history::default();
    (*w).set_active(None);
    (*w).lastlayout = -(1 as ::core::ffi::c_int);
    (*w).layout_root = None;
    (*w).sx = sx;
    (*w).sy = sy;
    (*w).manual_sx = sx;
    (*w).manual_sy = sy;
    (*w).xpixel = xpixel;
    (*w).ypixel = ypixel;
    (*w).options = Some(crate::src::options::options_create_owned(Some(
        crate::src::options::OptionsScope::GlobalWindow,
    )));
    (*w).sb = options_get_number(
        options_owner_ptr(&mut (*w).options).map_or(std::ptr::null_mut(), |options| options),
        b"pane-scrollbars\0" as *const u8 as *const ::core::ffi::c_char,
    ) as ::core::ffi::c_int;
    (*w).sb_pos = options_get_number(
        options_owner_ptr(&mut (*w).options).map_or(std::ptr::null_mut(), |options| options),
        b"pane-scrollbars-position\0" as *const u8 as *const ::core::ffi::c_char,
    ) as ::core::ffi::c_int;
    (*w).winlinks = Default::default();
    (*w).owner = refbox::Weak::new();
    let fresh0 = next_window_id;
    next_window_id = next_window_id.wrapping_add(1);
    (*w).id = fresh0;
    windows_insert(&raw mut windows, &owner);
    (*w).creation_time = SystemTime::now();
    window_update_activity(&(*(w)).observer.upgrade().expect("live window"));
    log_debug(format_args!(
        "{}: @{} create {}x{} ({}x{})",
        "window_create",
        ((*w).id) as u32,
        (sx) as u32,
        (sy) as u32,
        ((*w).xpixel) as u32,
        ((*w).ypixel) as u32
    ));
    owner
}
unsafe fn window_destroy(w_owner: &WindowRef) {
    let mut w = w_owner.get();
    assert_eq!((*w).lifecycle, WindowLifecycle::Live);
    (*w).lifecycle = WindowLifecycle::Destroying;
    log_debug(format_args!("window @{} destroyed", ((*w).id) as u32));
    // The releasing owner keeps weak parent links upgradeable throughout cleanup.
    // Restore layout links without scheduling resize events for dying panes.
    window_unzoom_internal(w_owner, 0, false);
    if !(*w).owner.is_empty() {
        windows_remove(&mut windows, w_owner);
    }
    drop((*w).layout_root.take());
    drop((*w).saved_layout_root.take());
    drop(window_replace_old_layout(w_owner, None));
    menu_destroy((*w).menu.take());
    window_destroy_panes(w_owner);
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
    (*w).lifecycle = WindowLifecycle::Destroyed;
}

pub unsafe fn window_add_ref(w_owner: &WindowRef, from: *const ::core::ffi::c_char) -> WindowRef {
    let mut w = w_owner.get();
    let owner = Rc::clone(w_owner);
    log_debug(format_args!(
        "retain window @{} ({})",
        ((*w).id) as u32,
        log_cstr((from) as *const _)
    ));
    owner
}
/// Consume a window reference, performing final cleanup while it is still held.
/// Brief upgraded borrows may drop normally only while another owner is guaranteed
/// to remain. Any owner that may be the final live reference must use this path.
pub unsafe fn window_remove_ref(owner: WindowRef, from: *const ::core::ffi::c_char) {
    window_prepare_release(&owner, from);
    drop(owner);
}

// Both ordinary owners and winlinks keep their reference alive through this call.
// Winlinks keep the field published so close callbacks can still inspect it.
unsafe fn window_prepare_release(w_owner: &WindowRef, from: *const ::core::ffi::c_char) {
    let mut w = w_owner.get();
    if (*w).lifecycle == WindowLifecycle::Live && Rc::strong_count(w_owner) == 1 {
        events_fire_window(c"window-closed".as_ptr(), Rc::clone(w_owner));
        // Close callbacks can retain the window. Defer cleanup until their release.
        if (*w).lifecycle == WindowLifecycle::Live && Rc::strong_count(w_owner) == 1 {
            window_destroy(w_owner);
        }
    }
    log_debug(format_args!(
        "release window @{} ({})",
        ((*w).id) as u32,
        log_cstr((from) as *const _)
    ));
}

pub unsafe fn window_set_name(
    w_owner: &WindowRef,
    mut new_name: *const ::core::ffi::c_char,
    mut untrusted: ::core::ffi::c_int,
) {
    let _w = w_owner.get();
    if let Some(name) = clean_name_cstring(CStr::from_ptr(new_name), untrusted) {
        // Keep the previous owner alive across synchronous rename callbacks.
        let last = window_replace_name(w_owner, name);
        window_fire_renamed(w_owner, last.as_ptr());
    }
}
pub unsafe fn window_resize(
    w_owner: &WindowRef,
    mut sx: u_int,
    mut sy: u_int,
    mut xpixel: ::core::ffi::c_int,
    mut ypixel: ::core::ffi::c_int,
) {
    let mut w = w_owner.get();
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
        server_redraw_window(&(w_owner));
    }
    if xpixel != -(1 as ::core::ffi::c_int) {
        (*w).xpixel = xpixel as u_int;
    }
    if ypixel != -(1 as ::core::ffi::c_int) {
        (*w).ypixel = ypixel as u_int;
    }
    (*w).invalidate_scene();
}

fn window_has_pane(w: &window, pane: &std::rc::Weak<std::cell::UnsafeCell<window_pane>>) -> bool {
    pane.strong_count() != 0 && w.panes.position(pane).is_some()
}

pub unsafe fn window_update_focus(w_owner: Option<&WindowRef>) {
    if let Some(window) = w_owner {
        if let Some(pane) = window.active_pane() {
            pane.update_focus(window.pane_is_focused(&pane));
        }
    }
}

pub unsafe fn window_set_active_pane(
    window: &WindowRef,
    pane: &Rc<UnsafeCell<window_pane>>,
    notify: i32,
) -> i32 {
    let w = window.get();
    let observer = Rc::downgrade(pane);
    log_debug(format_args!("window_set_active_pane: pane %{}", pane.id()));
    if (*w).active.ptr_eq(&observer) {
        return 0;
    }
    if (*w).modal.upgrade().is_some() && !(*w).modal.ptr_eq(&observer) {
        return 0;
    }
    if window.is_zoomed() && window_pane_is_visible(pane) == 0 {
        window.unzoom(true);
    }
    let previous = window.active_pane();
    window_pane_stack_remove(&mut (*w).last_panes, Some(pane));
    window_pane_stack_push(&mut (*w).last_panes, previous.as_ref());
    (*w).active = observer;
    pane.on_selected(true);
    if options_get_number(global_options, c"focus-events".as_ptr()) != 0 {
        if let Some(previous) = previous.as_ref() {
            previous.update_focus(false);
        }
        window_update_focus(Some(window));
    }
    tty_update_window_offset(window);
    server_redraw_window(&(window));
    if notify != 0 {
        if let Some(active) = window.active_pane() {
            window_fire_pane_changed(window, &active, previous.as_ref());
        }
    }
    1
}
unsafe fn window_pane_get_palette(
    wp_owner: Option<&Rc<std::cell::UnsafeCell<window_pane>>>,
    mut c: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let Some(pane) = wp_owner else { return -1 };
    colour_palette_get(Some(&*pane.borrow_palette()), c)
}
pub unsafe fn window_redraw_active_switch(
    w_owner: &WindowRef,
    wp_owner: Option<&Rc<std::cell::UnsafeCell<window_pane>>>,
) {
    let mut w = w_owner.get();
    let mut wp = wp_owner.map_or(std::ptr::null_mut(), |owner| owner.get());
    let mut gc1: *mut grid_cell = ::core::ptr::null_mut::<grid_cell>();
    let mut gc2: *mut grid_cell = ::core::ptr::null_mut::<grid_cell>();
    let mut c1: ::core::ffi::c_int = 0;
    let mut c2: ::core::ffi::c_int = 0;
    if (*w).modal.upgrade().is_some()
        && !wp
            .as_ref()
            .is_some_and(|pane| (*w).modal.ptr_eq(&pane.observer))
    {
        return;
    }
    if wp
        == (*w)
            .active_pane()
            .as_ref()
            .map_or(std::ptr::null_mut(), |owner| owner.get())
    {
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
            c1 = window_pane_get_palette(
                (wp).as_ref()
                    .and_then(|model| model.observer.upgrade())
                    .as_ref(),
                (*gc1).fg,
            );
            c2 = window_pane_get_palette(
                (wp).as_ref()
                    .and_then(|model| model.observer.upgrade())
                    .as_ref(),
                (*gc2).fg,
            );
            if c1 != c2 {
                (*wp).flags |= PANE_REDRAW;
            } else {
                c1 = window_pane_get_palette(
                    (wp).as_ref()
                        .and_then(|model| model.observer.upgrade())
                        .as_ref(),
                    (*gc1).bg,
                );
                c2 = window_pane_get_palette(
                    (wp).as_ref()
                        .and_then(|model| model.observer.upgrade())
                        .as_ref(),
                    (*gc2).bg,
                );
                if c1 != c2 {
                    (*wp).flags |= PANE_REDRAW;
                }
            }
        }
        if wp
            == (*w)
                .active_pane()
                .as_ref()
                .map_or(std::ptr::null_mut(), |owner| owner.get())
        {
            break;
        }
        if window_pane_is_floating(&*wp) != 0 {
            window_pane_z_remove(&mut *w, &*wp);
            window_pane_z_insert_front(&mut *w, &*wp);
            (*wp).flags |= PANE_REDRAW;
            (*w).invalidate_scene();
        }
        wp = (*w)
            .active_pane()
            .as_ref()
            .map_or(std::ptr::null_mut(), |owner| owner.get());
        if wp.is_null() {
            break;
        }
    }
}
pub unsafe fn window_get_active_at(
    window_owner: &WindowRef,
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
            if !(window_pane_is_visible(&candidate) == 0 || window_pane_is_floating(&*wp) != 0) {
                (xoff, yoff, sx, sy) = candidate.outer_geometry();
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
        if !(window_pane_is_visible(&candidate) == 0) {
            (xoff, yoff, sx, sy) = candidate.outer_geometry();
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
    window_owner: &WindowRef,
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
pub unsafe fn window_zoom(
    pane_owner: &Rc<std::cell::UnsafeCell<window_pane>>,
) -> ::core::ffi::c_int {
    let wp = pane_owner.get();
    let mut w: *mut window = (*wp)
        .window_handle()
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
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
    if (*w)
        .active_pane()
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get())
        != wp
        && ((*w).active_pane().is_none()
            || !(*(*w)
                .active_pane()
                .as_ref()
                .map_or(std::ptr::null_mut(), |owner| owner.get()))
            .flags
                & PANE_FLOATOVERZOOM
                != 0
            || window_pane_is_floating(
                &*(*w)
                    .active_pane()
                    .as_ref()
                    .map_or(std::ptr::null_mut(), |owner| owner.get()),
            ) == 0)
    {
        window_set_active_pane(
            &(*(w)).observer.upgrade().expect("live window"),
            &(*(wp)).observer.upgrade().expect("live window_pane"),
            1 as ::core::ffi::c_int,
        );
    }
    (*wp).flags |= PANE_ZOOMED;
    let mut cursor = window_pane_first(w.as_ref());
    while let Some(owner) = cursor {
        let wp1 = &mut *owner.get();
        (*wp1).saved_layout_cell = (*wp1).layout_cell.take();
        cursor = window_pane_next(Some(&*wp1));
    }
    (*w).saved_layout_root = (*w).layout_root.take();
    layout_init(
        &(*(w)).observer.upgrade().expect("live window"),
        &(*(wp)).observer.upgrade().expect("live window_pane"),
    );
    let mut cursor = window_pane_first(w.as_ref());
    while let Some(owner) = cursor {
        let wp1 = owner.get();
        let saved = (*wp1).saved_layout_cell.and_then(|id| {
            (*w).saved_layout_root
                .as_deref()
                .and_then(|root| root.find(id))
                .map(|cell| (cell.flags, cell.g))
        });
        if wp1 != wp
            && (*wp1).flags & PANE_FLOATOVERZOOM != 0
            && saved.is_some_and(|(flags, _)| flags & LAYOUT_CELL_FLOATING != 0)
        {
            lg = saved.unwrap().1;
            let cell_id = layout_floating_pane(
                &(*(w)).observer.upgrade().expect("live window"),
                (wp).as_ref()
                    .and_then(|model| model.observer.upgrade())
                    .as_ref(),
                &raw mut lg,
            );
            layout_assign_pane(
                &(*w).observer.upgrade().expect("live zoom window"),
                cell_id,
                &(*(wp1)).observer.upgrade().expect("live window_pane"),
                0 as ::core::ffi::c_int,
            );
        }
        cursor = window_pane_next(Some(&*wp1));
    }
    if (*wp)
        .saved_layout_cell
        .and_then(|id| {
            (*w).saved_layout_root
                .as_deref()
                .and_then(|root| root.find(id))
        })
        .is_some_and(|cell| cell.flags & LAYOUT_CELL_FLOATING != 0)
    {
        window_pane_z_remove(&mut *w, &*wp);
        window_pane_z_insert_back(&mut *w, &*wp);
    }
    (*w).flags |= WINDOW_ZOOMED;
    events_fire_window(
        b"window-zoomed\0" as *const u8 as *const ::core::ffi::c_char,
        (*(w)).observer.upgrade().expect("live window"),
    );
    events_fire_window(
        b"window-layout-changed\0" as *const u8 as *const ::core::ffi::c_char,
        (*(w)).observer.upgrade().expect("live window"),
    );
    (*w).invalidate_scene();
    return 0 as ::core::ffi::c_int;
}
pub unsafe fn window_unzoom(w_owner: &WindowRef, notify: ::core::ffi::c_int) -> ::core::ffi::c_int {
    let _w = w_owner.get();
    window_unzoom_internal(w_owner, notify, true)
}

unsafe fn window_unzoom_internal(
    w_owner: &WindowRef,
    notify: ::core::ffi::c_int,
    resize_panes: bool,
) -> ::core::ffi::c_int {
    let mut w = w_owner.get();
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
                // Copy current geometry before mutably borrowing the saved tree.
                // A saved identity may equal the visible identity; copying it
                // onto itself was a no-op in the previous representation.
                let geometry = (*wp).layout_cell.and_then(|id| {
                    (*w).layout_root
                        .as_deref()
                        .and_then(|root| root.find(id))
                        .map(|cell| (cell.g, cell.fg))
                });
                if let (Some(saved), Some((geometry, floating_geometry))) =
                    ((*wp).saved_layout_cell, geometry)
                {
                    if let Some(cell) = (*w)
                        .saved_layout_root
                        .as_deref_mut()
                        .and_then(|root| root.find_mut(saved))
                    {
                        cell.g = geometry;
                        cell.fg = floating_geometry;
                    }
                }
            }
        }
        cursor = window_pane_next(Some(&*wp));
    }
    (*w).flags &= !WINDOW_ZOOMED;
    layout_free(w_owner);
    (*w).layout_root = (*w).saved_layout_root.take();
    let mut cursor = window_pane_first(w.as_ref());
    while let Some(owner) = cursor {
        let wp = &mut *owner.get();
        (*wp).layout_cell = (*wp).saved_layout_cell.take();
        (*wp).flags &= !PANE_ZOOMED;
        cursor = window_pane_next(Some(&*wp));
    }
    if let Some(zoomed_owner) =
        zoomed_owner.filter(|owner| window_pane_is_floating(&*owner.get()) != 0)
    {
        let zoomed = zoomed_owner.get();
        window_pane_z_remove(&mut *w, &*zoomed);
        if zoomed
            == (*w)
                .active_pane()
                .as_ref()
                .map_or(std::ptr::null_mut(), |owner| owner.get())
        {
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
        layout_fix_panes(w_owner, None);
    }
    if notify != 0 {
        events_fire_window(
            b"window-unzoomed\0" as *const u8 as *const ::core::ffi::c_char,
            (*(w)).observer.upgrade().expect("live window"),
        );
        events_fire_window(
            b"window-layout-changed\0" as *const u8 as *const ::core::ffi::c_char,
            (*(w)).observer.upgrade().expect("live window"),
        );
    }
    (*w).invalidate_scene();
    return 0 as ::core::ffi::c_int;
}
unsafe fn window_zoomed_pane(w: &window) -> Option<Rc<std::cell::UnsafeCell<window_pane>>> {
    if w.flags & WINDOW_ZOOMED == 0 {
        return None;
    }
    w.z_index.snapshot().into_iter().rev().find(|owner| {
        let pane = &*owner.get();
        pane.layout_cell
            .and_then(|id| w.layout_root.as_deref().and_then(|root| root.find(id)))
            .is_some_and(|cell| cell.flags & LAYOUT_CELL_FLOATING == 0)
    })
}
pub unsafe fn window_active_pane_is_over_zoom(w_owner: &WindowRef) -> ::core::ffi::c_int {
    let mut w = w_owner.get();
    if !(*w).flags & WINDOW_ZOOMED != 0 {
        return 0 as ::core::ffi::c_int;
    }
    if (*w).active_pane().is_none() {
        return 0 as ::core::ffi::c_int;
    }
    if !(*(*w)
        .active_pane()
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get()))
    .flags
        & PANE_FLOATOVERZOOM
        != 0
    {
        return 0 as ::core::ffi::c_int;
    }
    return window_pane_is_floating(
        &*(*w)
            .active_pane()
            .as_ref()
            .map_or(std::ptr::null_mut(), |owner| owner.get()),
    );
}
pub unsafe fn window_push_zoom(
    w_owner: &WindowRef,
    mut always: ::core::ffi::c_int,
    mut flag: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut w = w_owner.get();
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
        (*w).was_zoomed = pane_owner
            .as_ref()
            .map_or_else(std::rc::Weak::new, Rc::downgrade);
    } else {
        (*w).was_zoomed = std::rc::Weak::new();
    }
    return (window_unzoom(w_owner, 1 as ::core::ffi::c_int) == 0 as ::core::ffi::c_int)
        as ::core::ffi::c_int;
}
pub unsafe fn window_pop_zoom(w_owner: &WindowRef) -> ::core::ffi::c_int {
    let mut w = w_owner.get();
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
        if pane_owner
            .as_ref()
            .is_none_or(|owner| !window_has_pane(&*w, &Rc::downgrade(owner)))
        {
            pane_owner = active;
        }
        if let Some(owner) = pane_owner {
            return (window_zoom(&owner) == 0) as ::core::ffi::c_int;
        }
    }
    return 0 as ::core::ffi::c_int;
}
pub unsafe fn window_add_pane(
    w_owner: &WindowRef,
    other_owner: Option<&Rc<std::cell::UnsafeCell<window_pane>>>,
    mut hlimit: u_int,
    mut flags: ::core::ffi::c_int,
) -> Rc<std::cell::UnsafeCell<window_pane>> {
    let mut w = w_owner.get();
    let mut other = other_owner.map_or(std::ptr::null_mut(), |owner| owner.get());
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    if other.is_null() {
        other = (*w)
            .active_pane()
            .as_ref()
            .map_or(std::ptr::null_mut(), |owner| owner.get());
    }
    let owner = window_pane_create(w_owner, (*w).sx, (*w).sy, hlimit);
    wp = owner.get();
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
    (*w).invalidate_scene();
    owner
}
pub unsafe fn window_lost_pane(window: &WindowRef, pane: &Rc<UnsafeCell<window_pane>>) {
    let w = window.get();
    let observer = Rc::downgrade(pane);
    log_debug(format_args!(
        "window_lost_pane: @{} pane %{}",
        window.id(),
        pane.id()
    ));
    if marked_pane
        .pane_handle()
        .is_some_and(|marked| Rc::ptr_eq(&marked, pane))
    {
        server_clear_marked();
    }
    if (*w).modal_last.ptr_eq(&observer) {
        (*w).modal_last = Weak::new();
    }
    if (*w).was_zoomed.ptr_eq(&observer) {
        (*w).was_zoomed = Weak::new();
    }
    window_pane_stack_remove(&mut (*w).last_panes, Some(pane));
    if (*w).active.ptr_eq(&observer) {
        let mut replacement = if (*w).modal.ptr_eq(&observer) {
            (*w).modal = Weak::new();
            std::mem::take(&mut (*w).modal_last).upgrade()
        } else {
            None
        };
        if replacement
            .as_ref()
            .is_none_or(|owner| !window_has_pane(&*w, &Rc::downgrade(owner)))
        {
            replacement = crate::src::shared::pane::pane_history_first(&(*w).last_panes);
        }
        if replacement.is_none() {
            replacement = (*w)
                .panes
                .previous(&observer)
                .or_else(|| (*w).panes.next(&observer));
        }
        (*w).active = replacement.as_ref().map_or_else(Weak::new, Rc::downgrade);
        if let Some(replacement) = replacement {
            window_pane_stack_remove(&mut (*w).last_panes, Some(&replacement));
            replacement.on_selected(false);
            window_fire_pane_changed(window, &replacement, Some(pane));
            window_update_focus(Some(window));
        }
    } else if (*w).modal.ptr_eq(&observer) {
        (*w).modal_last = Weak::new();
        (*w).modal = Weak::new();
    }
    (*w).invalidate_scene();
}

pub unsafe fn window_remove_pane(window: &WindowRef, pane: &Rc<UnsafeCell<window_pane>>) {
    window_lost_pane(window, pane);
    let observer = Rc::downgrade(pane);
    let w = window.get();
    assert!(
        (*w).panes.remove(&observer),
        "pane is not in its window order"
    );
    assert!(
        (*w).z_index.remove(&observer),
        "pane is not in its stacking order"
    );
    (*w).invalidate_scene();
    pane.destroy();
}

unsafe fn window_count_panes(w: &window, with_floating: ::core::ffi::c_int) -> u_int {
    w.panes.storage.iter().fold(0, |count, observer| {
        let pane = observer.upgrade().expect("live pane in ordering");
        if with_floating != 0 || window_pane_is_floating(&*pane.get()) == 0 {
            count.wrapping_add(1)
        } else {
            count
        }
    })
}
pub unsafe fn window_destroy_panes(w_owner: &WindowRef) {
    let mut w = w_owner.get();
    while let Some(owner) = window_pane_stack_first(w.as_ref()) {
        window_pane_stack_remove(
            &raw mut (*w).last_panes,
            (owner.get())
                .as_ref()
                .and_then(|model| model.observer.upgrade())
                .as_ref(),
        );
    }
    while let Some(owner) = window_pane_first(w.as_ref()) {
        let wp = owner.get();
        window_pane_list_remove(&mut *w, &*wp);
        window_pane_z_remove(&mut *w, &*wp);
        owner.destroy();
    }
}
pub unsafe fn window_printable_flags(
    mut wl: refbox::Weak<winlink>,
    mut escape: ::core::ffi::c_int,
) -> std::ffi::CString {
    let session_owner = wl.get_unchecked().session.upgrade();
    let mut flags: [::core::ffi::c_char; 32] = [0; 32];
    let mut pos: u_int = 0 as u_int;
    if wl.get_unchecked().flags & WINLINK_ACTIVITY != 0 {
        let fresh3 = pos;
        pos = pos.wrapping_add(1);
        flags[fresh3 as usize] = '#' as i32 as ::core::ffi::c_char;
        if escape != 0 {
            let fresh4 = pos;
            pos = pos.wrapping_add(1);
            flags[fresh4 as usize] = '#' as i32 as ::core::ffi::c_char;
        }
    }
    if wl.get_unchecked().flags & WINLINK_BELL != 0 {
        let fresh5 = pos;
        pos = pos.wrapping_add(1);
        flags[fresh5 as usize] = '!' as i32 as ::core::ffi::c_char;
    }
    if wl.get_unchecked().flags & WINLINK_SILENCE != 0 {
        let fresh6 = pos;
        pos = pos.wrapping_add(1);
        flags[fresh6 as usize] = '~' as i32 as ::core::ffi::c_char;
    }
    if session_owner
        .as_ref()
        .is_some_and(|owner| wl == owner.current_winlink())
    {
        let fresh7 = pos;
        pos = pos.wrapping_add(1);
        flags[fresh7 as usize] = '*' as i32 as ::core::ffi::c_char;
    }
    if session_owner
        .as_ref()
        .is_some_and(|owner| wl == owner.last_winlink())
    {
        let fresh8 = pos;
        pos = pos.wrapping_add(1);
        flags[fresh8 as usize] = '-' as i32 as ::core::ffi::c_char;
    }
    if server_check_marked() != 0 && wl == marked_pane.winlink_handle() {
        let fresh9 = pos;
        pos = pos.wrapping_add(1);
        flags[fresh9 as usize] = 'M' as i32 as ::core::ffi::c_char;
    }
    if (*wl
        .get_unchecked()
        .window_handle()
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get()))
    .modal
    .upgrade()
    .is_some()
    {
        let fresh10 = pos;
        pos = pos.wrapping_add(1);
        flags[fresh10 as usize] = 'O' as i32 as ::core::ffi::c_char;
    }
    if (*wl
        .get_unchecked()
        .window_handle()
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get()))
    .flags
        & WINDOW_ZOOMED
        != 0
    {
        let fresh11 = pos;
        pos = pos.wrapping_add(1);
        flags[fresh11 as usize] = 'Z' as i32 as ::core::ffi::c_char;
    }
    flags[pos as usize] = '\0' as i32 as ::core::ffi::c_char;
    std::ffi::CStr::from_ptr(flags.as_ptr()).to_owned()
}

fn window_pane_first(w: Option<&window>) -> Option<Rc<std::cell::UnsafeCell<window_pane>>> {
    w?.panes.first()
}

fn window_pane_last(w: Option<&window>) -> Option<Rc<std::cell::UnsafeCell<window_pane>>> {
    w?.panes.last()
}

fn window_pane_z_first(w: Option<&window>) -> Option<Rc<std::cell::UnsafeCell<window_pane>>> {
    w?.z_index.first()
}

fn window_pane_z_last(w: Option<&window>) -> Option<Rc<std::cell::UnsafeCell<window_pane>>> {
    w?.z_index.last()
}

fn window_pane_stack_first(w: Option<&window>) -> Option<Rc<std::cell::UnsafeCell<window_pane>>> {
    crate::src::shared::pane::pane_history_first(&w?.last_panes)
}

fn window_pane_stack_next(
    w: Option<&window>,
    wp: &window_pane,
) -> Option<Rc<std::cell::UnsafeCell<window_pane>>> {
    crate::src::shared::pane::pane_history_next(&w?.last_panes, &wp.observer)
}

fn window_pane_list_remove(w: &mut window, wp: &window_pane) {
    assert!(
        w.panes.remove(&wp.observer),
        "pane is not in its window order"
    );
}

fn window_pane_list_insert_front(w: &mut window, wp: &window_pane) {
    w.panes.push_front(wp.observer.clone());
}

fn window_pane_list_insert_back(w: &mut window, wp: &window_pane) {
    w.panes.push_back(wp.observer.clone());
}

fn window_pane_list_insert_before(w: &mut window, before: &window_pane, wp: &window_pane) {
    w.panes.insert_before(&before.observer, wp.observer.clone());
}

fn window_pane_list_insert_after(w: &mut window, after: &window_pane, wp: &window_pane) {
    w.panes.insert_after(&after.observer, wp.observer.clone());
}

fn window_pane_z_remove(w: &mut window, wp: &window_pane) {
    assert!(
        w.z_index.remove(&wp.observer),
        "pane is not in its stacking order"
    );
}

fn window_pane_z_insert_front(w: &mut window, wp: &window_pane) {
    w.z_index.push_front(wp.observer.clone());
}

fn window_pane_z_insert_back(w: &mut window, wp: &window_pane) {
    w.z_index.push_back(wp.observer.clone());
}

fn window_pane_z_insert_before(w: &mut window, before: &window_pane, wp: &window_pane) {
    w.z_index
        .insert_before(&before.observer, wp.observer.clone());
}

fn window_pane_z_insert_after(w: &mut window, after: &window_pane, wp: &window_pane) {
    w.z_index.insert_after(&after.observer, wp.observer.clone());
}

unsafe fn window_get_pane_lines(w: &window) -> pane_lines {
    options_get_number_ref(
        w.options.as_deref().expect("window options"),
        c"pane-border-lines",
    ) as pane_lines
}

unsafe fn window_get_pane_status(w: &window) -> ::core::ffi::c_int {
    let status = options_get_number_ref(
        w.options.as_deref().expect("window options"),
        c"pane-border-status",
    ) as ::core::ffi::c_int;
    if status == PANE_STATUS_TOP_FLOATING || status == PANE_STATUS_BOTTOM_FLOATING {
        return 0;
    }
    status
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
            let weak = window_pane_weak(&*(wp));
            let callback = window_pane_add_ref(
                &(*(wp)).observer.upgrade().expect("live window_pane"),
                c"callback".as_ptr(),
            );
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
    fn pane_index_observers_clear_on_removal_and_expire_with_the_owner() {
        unsafe {
            let mut head = window_pane_tree { storage: None };
            let mut other = window_pane_tree { storage: None };
            let first_owner = window_pane::new();
            let first = crate::src::shared::rc::as_ptr(&first_owner);
            (*first).id = 1;
            let second_owner = window_pane::new();
            let second = crate::src::shared::rc::as_ptr(&second_owner);
            (*second).id = 2;
            let duplicate_owner = window_pane::new();
            let duplicate = crate::src::shared::rc::as_ptr(&duplicate_owner);
            (*duplicate).id = 1;

            assert!(window_pane_tree_insert(&mut head, first_owner.clone()).is_none());
            assert!(window_pane_tree_insert(&mut head, second_owner.clone()).is_none());
            let index_observer = (*first).owner.clone();
            assert!(Rc::ptr_eq(
                &window_pane_tree_insert(&mut head, duplicate_owner.clone()).unwrap(),
                &first_owner
            ));
            assert!((*duplicate).owner.is_empty());
            assert!(window_pane_tree_remove(&mut other, &mut *first).is_none());
            assert!(!(*first).owner.is_empty());

            let mut moved = head;
            assert!(Rc::ptr_eq(
                &window_pane_tree_next(&*first).unwrap(),
                &second_owner
            ));
            assert_eq!(
                crate::src::shared::rc::as_ptr(
                    &window_pane_tree_remove(&mut moved, &mut *first).unwrap()
                ),
                first
            );
            assert!((*first).owner.is_empty());
            assert!(window_pane_tree_next(&*first).is_none());
            assert_eq!(
                crate::src::shared::rc::as_ptr(
                    &window_pane_tree_remove(&mut moved, &mut *second).unwrap()
                ),
                second
            );

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
        // Stack lookup keys have no shared lifecycle. Rc windows must be explicitly
        // cleaned up before the last owner is dropped; Drop does no model cleanup.
        assert!(
            self.observer.ptr_eq(&std::rc::Weak::new())
                || self.lifecycle == WindowLifecycle::Destroyed,
            "window must be released through window_remove_ref"
        );
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

    unsafe fn check_parent_during_final_mode_cleanup(wme: refbox::Weak<window_mode_entry>) {
        let pane_owner = wme
            .get_unchecked()
            .wp
            .upgrade()
            .expect("pane is retained during cleanup");
        let pane = pane_owner.get();
        let parent_owner = pane_owner
            .window_observer()
            .upgrade()
            .expect("cleanup retains the parent");
        assert_eq!((*parent_owner.get()).lifecycle, WindowLifecycle::Destroying);
        let parent = parent_owner.get();
        // Releasing a temporary owner during cleanup must not restart cleanup.
        window_remove_ref(parent_owner.clone(), c"reentrant mode cleanup".as_ptr());
        if let Some(slot) = wme.get_unchecked().boxed_data.as_ref() {
            let slot = slot
                .downcast_ref::<Rc<RefCell<Option<WindowRef>>>>()
                .unwrap();
            *slot.borrow_mut() = Some(parent_owner);
        }
        assert!((*parent).observer.ptr_eq(&pane_owner.window_observer()));
        (*pane).sx += 1;
    }

    #[test]
    fn explicit_window_release_retains_parent_during_mode_cleanup() {
        static MODE: std::sync::LazyLock<window_mode> = std::sync::LazyLock::new(|| window_mode {
            free: Some(check_parent_during_final_mode_cleanup),
            ..window_mode::default()
        });
        unsafe {
            let owner = zoomed_window();
            let pane = (*owner.get())
                .active_pane()
                .as_ref()
                .map_or(std::ptr::null_mut(), |owner| owner.get());
            let retained = (*pane).observer.upgrade().expect("registered pane");
            let before = (*pane).sx;
            let late_owner = Rc::new(RefCell::new(None::<WindowRef>));
            let entry = refbox::RefBox::new(window_mode_entry {
                wp: (*pane).observer.clone(),
                swp: std::rc::Weak::new(),
                mode: &MODE,
                boxed_data: Some(Box::new(late_owner.clone())),
                data_owner: None,
                prefix: 0,
                kill: 0,
            });
            window_pane::install_mode_for_test(&retained, entry);
            owner.release(c"test mode cleanup");
            assert_eq!((*retained.get()).sx, before + 1);
            assert_ne!((*retained.get()).flags & PANE_DESTROYED, 0);
            let late_owner = late_owner.borrow_mut().take().unwrap();
            assert_eq!((*late_owner.get()).lifecycle, WindowLifecycle::Destroyed);
            let observer = Rc::downgrade(&late_owner);
            let closed = Rc::new(Cell::new(0));
            let calls = closed.clone();
            let sink = events_add_sink(
                c"window-closed",
                events_callback(move |_, _| {
                    calls.set(calls.get() + 1);
                }),
            );
            late_owner.release(c"retained during cleanup");
            assert!(observer.upgrade().is_none());
            assert_eq!(closed.get(), 0, "a cleaned window must not close again");
            assert_eq!((*retained.get()).sx, before + 1, "mode cleanup runs once");
            events_remove_sink(sink);
        }
    }

    #[test]
    #[should_panic(expected = "window must be released through window_remove_ref")]
    fn dropping_the_last_owner_without_explicit_release_is_rejected() {
        drop(window::new());
    }

    #[test]
    fn close_callback_retention_defers_pane_cleanup() {
        unsafe {
            let owner = zoomed_window();
            let observer = Rc::downgrade(&owner);
            let pane = (*owner.get()).active.upgrade().unwrap();
            let retained = Rc::new(RefCell::new(None));
            let slot = retained.clone();
            let sink = events_add_sink(
                c"window-closed",
                events_callback(move |_, payload| {
                    *slot.borrow_mut() = Some(window_add_ref(
                        &(*(event_payload_get_window(payload)
                            .map_or(std::ptr::null_mut(), |owner| owner.get())))
                        .observer
                        .upgrade()
                        .expect("live window"),
                        c"close callback".as_ptr(),
                    ));
                }),
            );
            owner.release(c"initial owner");
            let owner = retained
                .borrow_mut()
                .take()
                .expect("callback retained window");
            assert_eq!((*owner.get()).lifecycle, WindowLifecycle::Live);
            assert_ne!((*owner.get()).flags & WINDOW_ZOOMED, 0);
            assert_eq!((*pane.get()).flags & PANE_DESTROYED, 0);
            events_remove_sink(sink);
            owner.release(c"close callback owner");
            assert!(observer.upgrade().is_none());
            assert_ne!((*pane.get()).flags & PANE_DESTROYED, 0);
        }
    }

    pub(super) unsafe fn zoomed_window() -> WindowRef {
        let w_owner = window::new();
        let w = w_owner.get();
        (*w).options = Some(crate::src::options::options_create_owned(None));
        let entry = options_table
            .iter()
            .find(|entry| entry.name == Some(c"pane-border-status"))
            .unwrap();
        options_default(
            options_owner_ptr(&mut (*w).options).map_or(std::ptr::null_mut(), |options| options),
            entry,
        );
        let pane_owner = window_pane::new();
        let pane = crate::src::shared::rc::as_ptr(&pane_owner);
        window_pane_tree_insert(&mut all_window_panes, pane_owner);
        (*pane).window = (*w).observer.clone();
        (*pane).fd = -1;
        (*pane).pipe_fd = -1;
        (*pane).sx = 80;
        (*pane).sy = 24;
        (*pane).base.grid = Some(grid_create(80, 24, 0));
        (*pane).screen_source = PaneScreenSource::Base;
        (*pane).flags = PANE_ZOOMED;
        (*w).set_active((pane).as_ref());
        (*w).panes.push_back((*pane).observer.clone());
        (*w).z_index.push_back((*pane).observer.clone());
        let mut saved = layout_create_cell();
        layout_set_size(&mut *saved, 40, 24, 0, 0);
        layout_make_leaf(&mut *saved, &(*pane).observer.upgrade().unwrap());
        (*pane).saved_layout_cell = Some(saved.id());
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
            let w = w_owner.get();
            let pane = (*w)
                .active_pane()
                .as_ref()
                .map_or(std::ptr::null_mut(), |owner| owner.get());
            let tree_owner = std::rc::Rc::new(std::cell::UnsafeCell::new(
                crate::src::shared::mode_tree::mode_tree_data {
                    wp: window_pane_weak(&*(pane)),
                    zoomed: 0,
                    ..Default::default()
                },
            ));
            let tree = rc::as_ptr(&tree_owner);
            let observed = Rc::downgrade(&tree_owner);
            (*pane).flags |= PANE_DESTROYED;
            assert!(window_pane_upgrade(&(*tree).wp).is_none());

            crate::src::mode_tree::mode_tree_free(tree_owner);

            assert!(observed.upgrade().is_none());
            assert_eq!((*w).flags & WINDOW_ZOOMED, 0);
            window_remove_ref(w_owner, c"test mode tree cleanup".as_ptr());
        }
    }

    #[test]
    fn explicit_release_destroys_zoomed_windows_without_resize_events() {
        unsafe {
            let w_owner = zoomed_window();
            let w = w_owner.get();
            let observer = (*w).observer.clone();
            let pane_observer = (*(*w)
                .active_pane()
                .as_ref()
                .map_or(std::ptr::null_mut(), |owner| owner.get()))
            .observer
            .clone();
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
                        (*event_payload_get_window(payload)
                            .map_or(std::ptr::null_mut(), |owner| owner.get()))
                        .flags
                            & WINDOW_ZOOMED,
                        0
                    );
                    close_count.set(close_count.get() + 1);
                }),
            );
            window_remove_ref(w_owner, c"test final notifying owner".as_ptr());
            assert_eq!(resized.get(), 0);
            assert_eq!(closed.get(), 1);
            assert!(observer.upgrade().is_none());
            assert!(pane_observer.upgrade().is_none());
            events_remove_sink(resize_sink);
            events_remove_sink(close_sink);
        }
    }

    #[test]
    fn live_unzoom_keeps_pane_resize_and_window_notifications() {
        unsafe {
            let w_owner = zoomed_window();
            let w = w_owner.get();
            let pane = (*w)
                .active_pane()
                .as_ref()
                .map_or(std::ptr::null_mut(), |owner| owner.get());
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
            assert_eq!(
                window_unzoom(&(*(w)).observer.upgrade().expect("live window"), 1),
                0
            );
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

#[cfg(test)]
pub(crate) mod test_support {
    use super::*;
    pub(crate) unsafe fn zoomed_window() -> WindowRef {
        super::zoom_teardown_tests::zoomed_window()
    }
    /// Visibility tests deliberately change this flag without repairing layout.
    pub(crate) unsafe fn set_zoomed(owner: &WindowRef, enabled: bool) {
        if enabled {
            (*owner.get()).flags |= WINDOW_ZOOMED;
        } else {
            (*owner.get()).flags &= !WINDOW_ZOOMED;
        }
    }
}
