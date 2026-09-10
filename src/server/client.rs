use crate::ImsgMessage;
use crate::cmd::cmdq_item;
use crate::cmd::{DisplayPanesRef, cmd_retval};

use crate::WindowPane;
use crate::pane_identity::PaneIdentity;
use crate::window_scrollbar::WindowScrollbarState;
use crate::window_trait::Window as _;

use crate::window_dimensions::WindowDimensionsState;

use super::message::server_check_unattached;
use super::message::{server_destroy_pane, server_redraw_client, server_status_client};
use super::run::client_walk;
use super::run::{current_time, with_clients, with_clients_mut};
use super::run::{server_add_accept, server_update_socket};
use crate::alerts::alerts_check_session;
use crate::args::args_from_vector;
use crate::cfg::start_cfg;
use crate::cfg::{cfg_client, configuration_finished};

use crate::cmd::cmd_parse_from_arguments;
use crate::cmd::{CmdqItemRef, cmdq_append};
use crate::cmd::{cmd_find_from_client, cmd_find_from_mouse};
use crate::compat::imsg_get_fd;
use crate::control::{
    control_all_done, control_discard, control_pane_offset, control_pane_offset_mut, control_ready,
    control_reset_offsets, control_start, control_stop, control_write,
};
use crate::environ::EnvironmentStore;
use crate::environ::new_environment_box;
use crate::ffi::{access, close, isatty, sscanf, strcmp};
use crate::file::file_print;
use crate::fmt_args;
use crate::fmt_engine::format_alloc;
use crate::format::{format_create, format_defaults, format_expand_time, format_free_jobs};
use crate::input::input_cancel_requests;
use crate::key_bindings::{
    key_binding_flags, key_binding_key, key_bindings_dispatch, key_bindings_get_table_ref,
};
use crate::log::{log_debug, log_get_level};
use crate::modes::window_copy_add;
use crate::overlay::draw_pane_numbers;

use crate::notify::notify_client;

use crate::overlay::{menu_check_cb, menu_mode_cb, menu_resize_cb};
use crate::overlay::{popup_check_cb, popup_mode_cb};
use crate::pane_geometry::PaneGeometryState;
use crate::pane_output::PaneOutputOffset;
use crate::pane_resize::PaneResizeQueue;
use crate::pane_scrollbar::PaneScrollbar;
use crate::pane_scrollbar_style::PaneScrollbarStyleState;
use crate::proc::PeerDispatch;
use crate::reactor;
use crate::reactor::{Interest, IoWatch, Reactor, Timer};
use crate::resize::recalculate_sizes;

use crate::screen::screen_mode_to_string;
use crate::screen::screen_redraw_is_visible;
use crate::screen::{Screen, ScreenModeState};
use crate::screen::{screen_redraw_pane, screen_redraw_screen};
use crate::session::session_ref_of;

pub use crate::consts::{
    _PATH_BSHELL, ALL_MOUSE_MODES, CLIENT_ACTIVEPANE, CLIENT_ALLREDRAWFLAGS, CLIENT_ATTACHED,
    CLIENT_CONTROL, CLIENT_CONTROL_NOOUTPUT, CLIENT_CONTROL_PAUSEAFTER, CLIENT_CONTROL_WAITEXIT,
    CLIENT_DEAD, CLIENT_EXIT, CLIENT_EXIT_DETACH, CLIENT_EXIT_RETURN, CLIENT_EXIT_SHUTDOWN,
    CLIENT_EXITED, CLIENT_FOCUSED, CLIENT_IDENTIFIED, CLIENT_IGNORESIZE,
    CLIENT_NO_DETACH_ON_DESTROY, CLIENT_READONLY, CLIENT_REDRAWBORDERS, CLIENT_REDRAWOVERLAY,
    CLIENT_REDRAWPANES, CLIENT_REDRAWSCROLLBARS, CLIENT_REDRAWSTATUS, CLIENT_REDRAWWINDOW,
    CLIENT_STATUSFORCE, CLIENT_SUSPENDED, CLIENT_TERMINAL, CLIENT_UNATTACHEDFLAGS, CLIENT_UTF8,
    CMD_PARSE_ERROR, CMD_READONLY, CMD_RETURN_ERROR, CMD_RETURN_NORMAL, EINTR, FORMAT_NONE,
    IMSG_HEADER_SIZE, KEY_BINDING_REPEAT, KEYC_ANY, KEYC_CTRL, KEYC_DOUBLECLICK, KEYC_DRAGGING,
    KEYC_FOCUS_IN, KEYC_FOCUS_OUT, KEYC_MASK_KEY, KEYC_MASK_MODIFIERS, KEYC_MASK_TYPE, KEYC_META,
    KEYC_MOUSE, KEYC_MOUSEMOVE_BORDER, KEYC_MOUSEMOVE_PANE, KEYC_MOUSEMOVE_STATUS,
    KEYC_MOUSEMOVE_STATUS_DEFAULT, KEYC_MOUSEMOVE_STATUS_LEFT, KEYC_MOUSEMOVE_STATUS_RIGHT,
    KEYC_NONE, KEYC_PASTE_END, KEYC_PASTE_START, KEYC_REPORT_DARK_THEME, KEYC_REPORT_LIGHT_THEME,
    KEYC_SENT, KEYC_SHIFT, KEYC_TYPE_DOUBLECLICK, KEYC_TYPE_FUNCTION, KEYC_TYPE_MOUSEDOWN,
    KEYC_TYPE_MOUSEDRAG, KEYC_TYPE_MOUSEDRAGEND, KEYC_TYPE_MOUSEMOVE, KEYC_TYPE_MOUSEUP,
    KEYC_TYPE_NOTYPE, KEYC_TYPE_SECONDCLICK, KEYC_TYPE_TRIPLECLICK, KEYC_TYPE_WHEELDOWN,
    KEYC_TYPE_WHEELUP, KEYC_UNKNOWN, MODE_BRACKETPASTE, MODE_CURSOR, MODE_MOUSE_ALL,
    MODE_MOUSE_BUTTON, MOUSE_BUTTON_1, MOUSE_BUTTON_3, MOUSE_MASK_BUTTONS, MOUSE_MASK_CTRL,
    MOUSE_MASK_DRAG, MOUSE_MASK_META, MOUSE_MASK_SHIFT, MOUSE_WHEEL_DOWN, MOUSE_WHEEL_UP,
    MSG_COMMAND, MSG_EXEC, MSG_EXIT, MSG_EXITED, MSG_EXITING, MSG_FLAGS, MSG_IDENTIFY_CLIENTPID,
    MSG_IDENTIFY_CWD, MSG_IDENTIFY_DONE, MSG_IDENTIFY_ENVIRON, MSG_IDENTIFY_FEATURES,
    MSG_IDENTIFY_FLAGS, MSG_IDENTIFY_LONGFLAGS, MSG_IDENTIFY_STDIN, MSG_IDENTIFY_STDOUT,
    MSG_IDENTIFY_TERM, MSG_IDENTIFY_TERMINFO, MSG_IDENTIFY_TTYNAME, MSG_READ, MSG_READ_DONE,
    MSG_READY, MSG_RESIZE, MSG_SHELL, MSG_SHUTDOWN, MSG_SUSPEND, MSG_UNLOCK, MSG_WAKEUP,
    MSG_WRITE_READY, PANE_EXITED, PANE_REDRAW, PANE_REDRAWSCROLLBAR, PANE_SCROLLBARS_LEFT,
    PANE_SCROLLBARS_RIGHT, PANE_STATUS_BOTTOM, PANE_STATUS_OFF, PANE_STATUS_TOP, PANE_STYLECHANGED,
    PANE_ZOOMED, SIZE_MAX, STDERR_FILENO, STDIN_FILENO, STDOUT_FILENO, STYLE_RANGE_CONTROL,
    STYLE_RANGE_LEFT, STYLE_RANGE_NONE, STYLE_RANGE_PANE, STYLE_RANGE_RIGHT, STYLE_RANGE_SESSION,
    STYLE_RANGE_USER, STYLE_RANGE_WINDOW, THEME_DARK, THEME_LIGHT, THEME_UNKNOWN, TTY_BLOCK,
    TTY_FREEZE, TTY_NOCURSOR, TTYC_ENBP, VIS_CSTYLE, VIS_NOSLASH, VIS_OCTAL, WINDOW_RESIZE,
    WINDOW_SIZE_LATEST, WINDOW_ZOOMED, WINLINK_ALERTFLAGS, X_OK,
};
use crate::status::{
    status_at_line, status_free, status_get_range, status_init, status_line_size,
    status_message_clear, status_prompt_clear, status_prompt_key, status_prompt_line_at,
    status_timer_start,
};
use crate::terminfo::{RustTerminalFeatureSet, TerminalFeatureSet};
use crate::terminfo::{TerminalCapabilities, tty_term_of};
use crate::text::{KeyStringCodec, RustKeyStringCodec};
use crate::text::{RustUtf8VisModel, Utf8VisModel};
use crate::tmux::{checkshell, find_home, setblocking};
use crate::tmux::{global_options, global_s_options};
use crate::tty::{
    TTY_OPENED, tty_close, tty_cursor, tty_free, tty_init, tty_margin_off, tty_open,
    tty_region_off, tty_repeat_requests, tty_reset, tty_resize, tty_send_requests, tty_set_path,
    tty_set_progress_bar, tty_set_title, tty_start_tty, tty_stop_tty, tty_sync_end,
    tty_update_client_offset, tty_update_mode, tty_window_offset,
};
pub use crate::types::*;
use crate::window::pane_walk;
use crate::window::window_pane_current_mode_mut;
use crate::window::winlinks_into;
use crate::window::{
    WINDOWS, window_get_active_at, window_pane_border_status_get_range, window_pane_find_by_id,
    window_pane_is_floating, window_pane_key, window_pane_paste,
    window_pane_send_resize, window_pane_send_theme_update, window_pane_set_mode,
    window_pane_show_scrollbar, window_redraw_active_switch, window_update_focus,
};
use crate::window::{window_get_latest, window_set_latest};
use crate::xmalloc::xasprintf;
use crate::{CommandTextCodec, RustCommandTextCodec};
use ::core::ffi::CStr;
use ::std::ffi::CString;

pub type key_code_mouse_location = core::ffi::c_uint;
pub const KEYC_MOUSE_LOCATION_NOWHERE: key_code_mouse_location = 19;
pub const KEYC_MOUSE_LOCATION_CONTROL0: key_code_mouse_location = 9;
pub const KEYC_MOUSE_LOCATION_SCROLLBAR_DOWN: key_code_mouse_location = 8;
pub const KEYC_MOUSE_LOCATION_SCROLLBAR_SLIDER: key_code_mouse_location = 7;
pub const KEYC_MOUSE_LOCATION_SCROLLBAR_UP: key_code_mouse_location = 6;
pub const KEYC_MOUSE_LOCATION_BORDER: key_code_mouse_location = 5;
pub const KEYC_MOUSE_LOCATION_STATUS_DEFAULT: key_code_mouse_location = 4;
pub const KEYC_MOUSE_LOCATION_STATUS_RIGHT: key_code_mouse_location = 3;
pub const KEYC_MOUSE_LOCATION_STATUS_LEFT: key_code_mouse_location = 2;
pub const KEYC_MOUSE_LOCATION_STATUS: key_code_mouse_location = 1;
pub const KEYC_MOUSE_LOCATION_PANE: key_code_mouse_location = 0;

pub const _PATH_TTY: &CStr = c"/dev/tty";

pub const KEYC_CLICK_TIMEOUT: core::ffi::c_int = 300 as core::ffi::c_int;
pub const KEYC_MOUSE_LOCATION_SHIFT: core::ffi::c_int = 0 as core::ffi::c_int;
pub const KEYC_MOUSE_BUTTON_SHIFT: core::ffi::c_int = 8 as core::ffi::c_int;

pub const MOUSE_BUTTON_2: core::ffi::c_int = 1 as core::ffi::c_int;

pub const MOUSE_BUTTON_6: core::ffi::c_int = 66 as core::ffi::c_int;
pub const MOUSE_BUTTON_7: core::ffi::c_int = 67 as core::ffi::c_int;
pub const MOUSE_BUTTON_8: core::ffi::c_int = 128 as core::ffi::c_int;
pub const MOUSE_BUTTON_9: core::ffi::c_int = 129 as core::ffi::c_int;
pub const MOUSE_BUTTON_10: core::ffi::c_int = 130 as core::ffi::c_int;
pub const MOUSE_BUTTON_11: core::ffi::c_int = 131 as core::ffi::c_int;

pub const CLIENT_PASTE_TIME_LIMIT: core::ffi::c_int = 5 as core::ffi::c_int;

pub const CLIENT_REPEAT: core::ffi::c_int = 0x20 as core::ffi::c_int;

pub const CLIENT_DOUBLECLICK: core::ffi::c_int = 0x100000 as core::ffi::c_int;
pub const CLIENT_TRIPLECLICK: core::ffi::c_int = 0x200000 as core::ffi::c_int;

pub const CLIENT_BRACKETPASTING: core::ffi::c_ulonglong = 0x1000000000 as core::ffi::c_ulonglong;
pub const CLIENT_ASSUMEPASTING: core::ffi::c_ulonglong = 0x2000000000 as core::ffi::c_ulonglong;

pub const CLIENT_NODETACHFLAGS: core::ffi::c_int = CLIENT_DEAD | CLIENT_EXIT;

pub fn server_client_how_many() -> u_int {
    let mut n: u_int;
    n = 0 as u_int;
    for owner in client_walk() {
        if !{ owner.attached_session() }.is_none()
            && !unsafe { owner.flags() } & CLIENT_UNATTACHEDFLAGS as uint64_t != 0
        {
            n = n.wrapping_add(1);
        }
    }
    n
}
unsafe fn server_client_overlay_timer(c: &mut client) {
    unsafe {
        server_client_clear_overlay(&mut *c);
    }
}
impl OverlayData {
    pub fn is_none(&self) -> bool {
        matches!(self, OverlayData::None)
    }
    pub fn menu(self) -> MenuDataRef {
        match self {
            OverlayData::Menu(data) => data,
            _ => panic!("overlay data is not a menu"),
        }
    }
    pub fn popup(self) -> PopupDataRef {
        match self {
            OverlayData::Popup(data) => data,
            _ => panic!("overlay data is not a popup"),
        }
    }
    pub fn display_panes(self) -> DisplayPanesRef {
        match self {
            OverlayData::DisplayPanes(data) => data,
            _ => panic!("overlay data is not display-panes"),
        }
    }
}
impl Overlay {
    /// The check the overlay installs with itself. `display-panes` covers
    /// nothing: it draws over the panes and lets everything through.
    pub fn check(self) -> OverlayCheck {
        match self {
            Overlay::None => OverlayCheck::None,
            Overlay::Menu => OverlayCheck::Menu,
            Overlay::Popup => OverlayCheck::Popup,
            Overlay::DisplayPanes { .. } => OverlayCheck::None,
        }
    }
    pub fn has_mode(self) -> bool {
        matches!(self, Overlay::Menu | Overlay::Popup)
    }
    pub fn has_key(self) -> bool {
        match self {
            Overlay::None => false,
            Overlay::Menu | Overlay::Popup => true,
            Overlay::DisplayPanes { keys } => keys,
        }
    }
    pub fn has_resize(self) -> bool {
        matches!(self, Overlay::Menu | Overlay::Popup)
    }
    pub fn mode(self, data: OverlayData) -> (ScreenModeState, u_int, u_int) {
        {
            match self {
                Overlay::Menu => menu_mode_cb(&data.menu().borrow()),
                Overlay::Popup => {
                    let popup = data.popup();
                    popup_mode_cb(&popup.borrow())
                }
                _ => panic!("overlay has no mode"),
            }
        }
    }
    pub unsafe fn draw(self, c: &mut client, data: OverlayData, ctx: &mut screen_redraw_ctx) {
        unsafe {
            match self {
                Overlay::Menu => (data.menu()).draw(c, ctx),
                Overlay::Popup => {
                    let popup = data.popup();
                    popup.draw(c, ctx)
                }
                Overlay::DisplayPanes { .. } => draw_pane_numbers(
                    &mut crate::server::client_ref_of(c).expect("a callback client has an owner"),
                    ctx,
                ),
                Overlay::None => panic!("overlay is not set"),
            }
        }
    }
    pub unsafe fn key(
        self,
        c: &mut client,
        data: OverlayData,
        event: &mut key_event,
    ) -> core::ffi::c_int {
        unsafe {
            match self {
                Overlay::Menu => (data.menu()).key(c, event),
                Overlay::Popup => {
                    let popup = data.popup();
                    popup.key(c, event)
                }
                Overlay::DisplayPanes { keys: true } => (data.display_panes()).key(
                    &client_ref_of(c).expect("a callback client has an owner"),
                    event,
                ),
                _ => panic!("overlay has no key"),
            }
        }
    }
    pub unsafe fn free(self, c: &mut client, data: OverlayState) {
        unsafe {
            match (self, data) {
                (Overlay::Menu, OverlayState::Menu(data)) => data.close(c),
                (Overlay::Popup, OverlayState::Popup(data) | OverlayState::PopupMenu(data) | OverlayState::PopupHidden(data)) => data.close(c),
                (Overlay::DisplayPanes { .. }, OverlayState::DisplayPanes(data)) => data.close(),
                (Overlay::None, _) => panic!("overlay is not set"),
                _ => panic!("overlay data does not match overlay"),
            }
        }
    }
    pub unsafe fn resize(self, c: &mut client, data: OverlayData) {
        unsafe {
            match self {
                Overlay::Menu => menu_resize_cb(c, &mut data.menu().borrow_mut()),
                Overlay::Popup => {
                    let popup = data.popup();
                    popup.resize(c)
                }
                _ => panic!("overlay has no resize"),
            }
        }
    }
}

impl OverlayCheck {
    pub fn call(self, data: OverlayData, px: u_int, py: u_int, nx: u_int) -> VisibleRangesRef {
        {
            match self {
                OverlayCheck::Menu => {
                    let owner = data.menu();
                    let mut guard = owner.borrow_mut();
                    let menu = &mut *guard;
                    menu_check_cb(menu, px, py, nx);
                    menu.r.clone()
                }
                OverlayCheck::Popup => {
                    let owner = data.popup();
                    let mut guard = owner.borrow_mut();
                    let popup = &mut *guard;
                    popup_check_cb(popup, px, py, nx);
                    popup.r.clone()
                }
                OverlayCheck::None => panic!("overlay check is not set"),
            }
        }
    }
}

pub(crate) unsafe fn server_client_update_focus(c: &client) {
    unsafe {
        let Some(session) = c.attached_session() else {
            return;
        };
        let window = session.current_window();
        window_update_focus(window.as_ref());
    }
}

pub unsafe fn server_client_set_overlay(
    c: &mut client,
    delay: u_int,
    overlay: Overlay,
    data: OverlayState,
) {
    unsafe {
        let mut tv = timeval::default();
        if (*c).overlay().is_some() {
            server_client_clear_overlay(c);
        }
        tv.tv_sec = delay.wrapping_div(1000 as u_int) as __time_t;
        tv.tv_usec = (delay.wrapping_rem(1000 as u_int) as core::ffi::c_long
            * 1000 as core::ffi::c_long) as __suseconds_t;
        let watching = client_ref_of(c).map(|held| held.downgrade());
        c.overlay_timer.set_callback(move || {
            if let Some(mut c) = watching.as_ref().and_then(ClientWeak::upgrade) {
                server_client_overlay_timer(c.as_client_mut());
            }
        });
        if delay != 0 as u_int {
            c.overlay_timer.arm(tv);
        }
        (*c).set_overlay(overlay, data);
        if (*c).overlay_check().is_none() {
            c.tty.flags |= TTY_FREEZE;
        }
        if !overlay.has_mode() {
            c.tty.flags |= TTY_NOCURSOR;
        }
        server_client_update_focus(c);
        server_redraw_client(&mut *c);
    }
}
pub unsafe fn server_client_clear_overlay(c: &mut client) {
    unsafe {
        if (*c).overlay().is_none() {
            return;
        }
        c.overlay_timer.disarm();
        let (overlay, data) = (*c).take_overlay();
        if !data.is_none() {
            overlay.free(c, data);
        }
        if (*c).overlay().is_none() {
            c.tty.flags &= !(TTY_FREEZE | TTY_NOCURSOR);
            server_client_update_focus(c);
        }
        server_redraw_client(&mut *c);
    }
}

pub unsafe fn server_client_check_nested(c: &mut client) -> core::ffi::c_int {
    unsafe {
        let envent = (*c).environ_mut().find(c"TMUX");
        if !envent
            .and_then(|entry| entry.value)
            .is_some_and(|value| !value.is_empty())
        {
            return 0 as core::ffi::c_int;
        }
        for pane in pane_walk() {
            let wp = pane.get().expect("registered pane");
            if c.ttyname.as_deref() == Some(wp.terminal_name()) {
                return 1 as core::ffi::c_int;
            }
        }
        0 as core::ffi::c_int
    }
}
pub fn server_client_set_key_table(c: &mut client, name: Option<&CStr>) {
    {
        let default_name;
        let name = match name {
            Some(name) => name,
            None => {
                default_name = server_client_get_key_table(c);
                default_name.as_ref()
            }
        };
        let table_ref = key_bindings_get_table_ref(name, 1 as core::ffi::c_int)
            .expect("key table creation requested");
        c.keytable = Some(table_ref);
        let now = timeval::now();
        c.keytable()
            .expect("client key table")
            .set_activity_time(now);
    }
}
fn server_client_key_table_activity_diff(c: &client) -> uint64_t {
    {
        let mut diff = timeval::default();
        let since = c.keytable().expect("client key table").activity_time();
        diff.tv_sec = c.activity_time.tv_sec - since.tv_sec;
        diff.tv_usec = c.activity_time.tv_usec - since.tv_usec;
        if diff.tv_usec < 0 as __suseconds_t {
            diff.tv_sec -= 1;
            diff.tv_usec += 1000000 as __suseconds_t;
        }
        (diff.tv_sec as core::ffi::c_ulonglong)
            .wrapping_mul(1000 as core::ffi::c_ulonglong)
            .wrapping_add(
                (diff.tv_usec as core::ffi::c_ulonglong)
                    .wrapping_div(1000 as core::ffi::c_ulonglong),
            ) as uint64_t
    }
}
pub fn server_client_get_key_table(c: &client) -> std::rc::Rc<CStr> {
    {
        let Some(session) = c.attached_session() else {
            return std::rc::Rc::from(c"root");
        };
        let name = session.options().string_ref(c"key-table");
        if name.is_empty() {
            return std::rc::Rc::from(c"root");
        }
        name
    }
}
fn server_client_is_default_key_table(c: &client, table: &key_table) -> core::ffi::c_int {
    ((table).name() == server_client_get_key_table(c).as_ref()) as core::ffi::c_int
}

/// The session the client was attached to before this one, while it lives.
pub fn client_get_last_session(c: &client) -> Option<SessionRef> {
    c.last_session.as_ref().and_then(SessionWeak::upgrade)
}

/// Records `s` as the session the client was attached to before this one.
pub fn client_set_last_session(c: &mut client, s: Option<&session>) {
    c.last_session = s.and_then(session_ref_of).map(|s| s.downgrade());
}

/// The window the client has anchored its pan offset to, while it lives.
pub fn client_get_pan_window(c: &client) -> Option<WindowRef> {
    c.pan_window.as_ref().and_then(WindowWeak::upgrade)
}

/// Anchors the client's pan offset to `w`.
pub fn client_set_pan_window(c: &mut client, w: Option<&WindowRef>) {
    c.pan_window = w.map(WindowRef::downgrade);
}

/// Upgrades the payload's owner without borrowing its storage.
pub(crate) fn client_ref_of(value: &client) -> Option<ClientRef> {
    value
        .owner
        .as_ref()?
        .upgrade()
        .filter(|owner| core::ptr::eq(owner.as_ptr(), value))
}

impl Drop for ClientStorage {
    fn drop(&mut self) {
        unsafe {
            let c = self.value.get_mut();
            c.event.disable();
            c.repeat_timer.disarm();
            c.click_timer.disarm();
            c.message_timer.disarm();
            c.overlay_timer.disarm();
            c.status.timer.disarm();
            c.tty.start_timer.disarm();
            c.tty.clipboard_timer.disarm();
            if c.overlay().is_some() {
                server_client_clear_overlay(c);
            }
            input_cancel_requests(&mut *c);
            status_free(c);
            if c.tty.flags & TTY_OPENED != 0 {
                tty_free(&mut c.tty);
            }
            format_free_jobs(c.jobs.take());
            c.queue = None;
            c.files.clear();
            c.windows.clear();
            c.environ = None;
            c.control_state = None;
            c.name = None;
            c.user = None;
            c.title = None;
            c.path = None;
            c.cwd = None;
            c.term_name = None;
            c.term_type = None;
            c.ttyname = None;
            c.term_caps.clear();
            c.prompt_buffer.clear();
            c.prompt_saved = None;
            c.exit_session = None;
            c.exit_message = None;
            c.message_string = None;
            c.prompt_string = None;
            c.prompt_last = None;
        }
    }
}

pub unsafe fn server_client_open(c: &mut client, cause: &mut Option<CString>) -> core::ffi::c_int {
    unsafe {
        if c.flags & CLIENT_CONTROL as uint64_t != 0 {
            return 0 as core::ffi::c_int;
        }
        let ttyname = c
            .ttyname
            .as_deref()
            .expect("a terminal client has a device name");
        if ttyname == _PATH_TTY
            || [STDIN_FILENO, STDOUT_FILENO, STDERR_FILENO]
                .into_iter()
                .any(|fd| {
                    isatty(fd) != 0
                        && crate::tty::terminal_device_name(fd)
                            .is_ok_and(|name| name.as_c_str() == ttyname)
                })
        {
            *cause = Some(xasprintf(c"can't use %s", fmt_args![c.ttyname.as_deref()]));
            return -(1 as core::ffi::c_int);
        }
        if c.flags & CLIENT_TERMINAL as uint64_t == 0 {
            *cause = Some(c"not a terminal".to_owned());
            return -(1 as core::ffi::c_int);
        }
        if tty_open(&mut c.tty, cause) != 0 as core::ffi::c_int {
            return -(1 as core::ffi::c_int);
        }
        0 as core::ffi::c_int
    }
}
unsafe fn server_client_attached_lost(c: &mut client) {
    WINDOWS.with(|windows| unsafe {
        log_debug(c"lost attached client %p", fmt_args![&raw const *c]);
        for w_ref in windows.iter().filter_map(|(_, window)| window.upgrade()) {
            if window_get_latest(&w_ref.as_window())
                .is_some_and(|latest| core::ptr::eq(latest.as_client(), c))
            {
                let mut found: Option<ClientRef> = None;
                for candidate in client_walk() {
                    if core::ptr::eq(candidate.as_client(), c) {
                        continue;
                    }
                    let Some(session) = candidate.attached_session() else {
                        continue;
                    };
                    let shows_window = session
                        .curw()
                        .and_then(|link| link.window())
                        .is_some_and(|window| window.ptr_eq(&w_ref));
                    if shows_window
                        && found.as_ref().is_none_or(|found| {
                            let activity = candidate.as_client().activity_time;
                            let previous = found.as_client().activity_time;
                            if activity.tv_sec == previous.tv_sec {
                                activity.tv_usec > previous.tv_usec
                            } else {
                                activity.tv_sec > previous.tv_sec
                            }
                        })
                    {
                        found = Some(candidate);
                    }
                }
                if let Some(mut found) = found {
                    server_client_update_latest(found.as_client_mut());
                }
            }
        }
    });
}
pub unsafe fn server_client_set_session(c: &mut client, s_ref: Option<&SessionRef>) {
    unsafe {
        let old = c.attached_session();
        if s_ref
            .zip(old.as_ref())
            .is_some_and(|(new, old)| !new.ptr_eq(old))
        {
            c.last_session = old.as_ref().map(SessionRef::downgrade);
        } else if s_ref.is_none() {
            c.last_session = None;
        }
        let session = s_ref.cloned();
        c.set_attached_session(session.as_ref());
        c.flags |= CLIENT_FOCUSED as uint64_t;
        if let Some(old) = old {
            let window = old.current_window();
            window_update_focus(window.as_ref());
        }
        if let Some(mut session) = session {
            let window = session.current_window();
            if let Some(window) = window {
                window_set_latest(&mut window.as_window_mut(), Some(c));
            }
            recalculate_sizes();
            let window = session.current_window();
            window_update_focus(window.as_ref());
            session.update_activity(None);
            session.theme_changed();
            session.as_session_mut().last_attached_time = timeval::now();
            let current = session.as_session().curw;
            if let Some(link) =
                current.and_then(|index| session.as_session_mut().windows.get_mut(&index))
            {
                link.flags &= !WINLINK_ALERTFLAGS;
            }
            alerts_check_session(session.as_session_mut());
            tty_update_client_offset(c);
            status_timer_start(c);
            notify_client(c"client-session-changed", Some(c));
            server_redraw_client(c);
        }
        server_check_unattached();
        server_update_socket();
    }
}

pub unsafe fn server_client_suspend(c: &mut client) {
    unsafe {
        if c.attached_session().is_none() || c.flags & CLIENT_UNATTACHEDFLAGS as uint64_t != 0 {
            return;
        }
        tty_stop_tty(&mut c.tty);
        c.flags |= CLIENT_SUSPENDED as uint64_t;
        (c.peer_handle()).send(MSG_SUSPEND, -(1 as core::ffi::c_int), &[]);
    }
}
pub fn server_client_detach(c: &mut client, msgtype: msgtype) {
    {
        let Some(session) = c.attached_session() else {
            return;
        };
        if c.flags & CLIENT_NODETACHFLAGS as uint64_t != 0 {
            return;
        }
        c.flags |= CLIENT_EXIT as uint64_t;
        c.exit_type = CLIENT_EXIT_DETACH;
        c.exit_msgtype = msgtype;
        c.exit_session = session.name();
    }
}
pub unsafe fn server_client_exec(c: &mut client, cmd: &CStr) {
    unsafe {
        if cmd.is_empty() {
            return;
        }
        let session = c.attached_session();
        let options = match session.as_ref() {
            Some(session) => session.options(),
            None => global_s_options
                .get()
                .as_ref()
                .expect("global options are initialized")
                .clone(),
        };
        let configured = options.string_ref(c"default-shell");
        let shell = if checkshell(Some(&configured)) == 0 {
            _PATH_BSHELL
        } else {
            &configured
        };
        let mut msg =
            Vec::with_capacity(cmd.to_bytes_with_nul().len() + shell.to_bytes_with_nul().len());
        msg.extend_from_slice(cmd.to_bytes_with_nul());
        msg.extend_from_slice(shell.to_bytes_with_nul());
        (c.peer_handle()).send(MSG_EXEC, -1, &msg);
    }
}
unsafe fn server_client_check_mouse_in_pane(
    pane: &RustWindowPaneWeak,
    px: core::ffi::c_int,
    py: core::ffi::c_int,
    sl_mpos: &mut u_int,
) -> key_code_mouse_location {
    unsafe {
        let Some(wp) = pane.get() else {
            return KEYC_MOUSE_LOCATION_NOWHERE;
        };
        let Some(window) = pane.window() else {
            return KEYC_MOUSE_LOCATION_NOWHERE;
        };
        let w = window.as_window();

        let mut sb_w: core::ffi::c_int;
        let mut sb_pad: core::ffi::c_int;

        let sl_top: core::ffi::c_int;
        let sl_bottom: core::ffi::c_int;
        let mut bdr_bottom: core::ffi::c_int;
        let mut bdr_top: core::ffi::c_int;
        let mut bdr_left: core::ffi::c_int;
        let mut bdr_right: core::ffi::c_int;
        let slider = wp.slider();
        let pane_status: core::ffi::c_int =
            (w.options_ref()).number(c"pane-border-status") as core::ffi::c_int;
        if window_pane_show_scrollbar(wp, w.scrollbar_settings().sb) != 0 {
            sb_w = wp.scrollbar_style().width;
            sb_pad = wp.scrollbar_style().padding;
        } else {
            sb_w = 0 as core::ffi::c_int;
            sb_pad = 0 as core::ffi::c_int;
        }
        let pane_status_line: core::ffi::c_int = if pane_status == PANE_STATUS_TOP {
            wp.geometry().yoff - 1 as core::ffi::c_int
        } else if pane_status == PANE_STATUS_BOTTOM {
            (wp.geometry().yoff as u_int).wrapping_add(wp.geometry().sy) as core::ffi::c_int
        } else {
            -(1 as core::ffi::c_int)
        };
        bdr_left = wp.geometry().xoff - 1 as core::ffi::c_int;
        if w.scrollbar_settings().sb_pos == PANE_SCROLLBARS_LEFT {
            bdr_left -= sb_pad + sb_w;
        }
        if (pane_status != PANE_STATUS_OFF
            && py != pane_status_line
            && py != wp.geometry().yoff + wp.geometry().sy as core::ffi::c_int
            || wp.geometry().yoff == 0 as core::ffi::c_int
                && py < wp.geometry().sy as core::ffi::c_int
            || py >= wp.geometry().yoff
                && py < wp.geometry().yoff + wp.geometry().sy as core::ffi::c_int)
            && (w.scrollbar_settings().sb_pos == PANE_SCROLLBARS_RIGHT
                && px < wp.geometry().xoff + wp.geometry().sx as core::ffi::c_int + sb_pad + sb_w
                || w.scrollbar_settings().sb_pos == PANE_SCROLLBARS_LEFT
                    && px
                        < wp.geometry().xoff + wp.geometry().sx as core::ffi::c_int - sb_pad - sb_w)
        {
            return if w.scrollbar_settings().sb_pos == PANE_SCROLLBARS_RIGHT
                && (px >= wp.geometry().xoff + wp.geometry().sx as core::ffi::c_int + sb_pad
                    && px
                        < wp.geometry().xoff + wp.geometry().sx as core::ffi::c_int + sb_pad + sb_w)
                || w.scrollbar_settings().sb_pos == PANE_SCROLLBARS_LEFT
                    && (px >= wp.geometry().xoff - sb_pad - sb_w && px < wp.geometry().xoff - sb_pad)
            {
                sl_top = (wp.geometry().yoff as u_int).wrapping_add(slider.sb_slider_y) as core::ffi::c_int;
                sl_bottom = (wp.geometry().yoff as u_int)
                    .wrapping_add(slider.sb_slider_y)
                    .wrapping_add(slider.sb_slider_h)
                    .wrapping_sub(1 as u_int) as core::ffi::c_int;
                if py < sl_top {
                    KEYC_MOUSE_LOCATION_SCROLLBAR_UP
                } else if py >= sl_top && py <= sl_bottom {
                    *sl_mpos = (py as u_int)
                        .wrapping_sub(slider.sb_slider_y)
                        .wrapping_sub(wp.geometry().yoff as u_int);
                    KEYC_MOUSE_LOCATION_SCROLLBAR_SLIDER
                } else {
                    KEYC_MOUSE_LOCATION_SCROLLBAR_DOWN
                }
            } else if window_pane_is_floating(
                &w,
                &{ (wp).observation() }.expect("the pane allocation exists"),
            ) != 0
                && (px == bdr_left
                    || py == wp.geometry().yoff - 1 as core::ffi::c_int
                    || py == wp.geometry().yoff + wp.geometry().sy as core::ffi::c_int)
            {
                KEYC_MOUSE_LOCATION_BORDER
            } else {
                KEYC_MOUSE_LOCATION_PANE
            };
        } else {
            for candidate in &w.panes {
                let Some(fwp) = candidate.get() else {
                    continue;
                };
                if !(w.flags & WINDOW_ZOOMED != 0 && !*fwp.flags() & PANE_ZOOMED != 0) {
                    if window_pane_show_scrollbar(fwp, w.scrollbar_settings().sb) != 0 {
                        sb_w = fwp.scrollbar_style().width;
                        sb_pad = fwp.scrollbar_style().padding;
                    } else {
                        sb_w = 0 as core::ffi::c_int;
                        sb_pad = 0 as core::ffi::c_int;
                    }
                    bdr_top = fwp.geometry().yoff - 1 as core::ffi::c_int;
                    bdr_left = fwp.geometry().xoff - 1 as core::ffi::c_int;
                    if w.scrollbar_settings().sb_pos == PANE_SCROLLBARS_LEFT {
                        bdr_left -= sb_pad + sb_w;
                        bdr_right = (fwp.geometry().xoff as u_int).wrapping_add(fwp.geometry().sx)
                            as core::ffi::c_int;
                    } else {
                        bdr_right = (fwp.geometry().xoff as u_int)
                            .wrapping_add(fwp.geometry().sx)
                            .wrapping_add(sb_pad as u_int)
                            .wrapping_add(sb_w as u_int)
                            as core::ffi::c_int;
                    }
                    if py >= fwp.geometry().yoff - 1 as core::ffi::c_int
                        && py <= fwp.geometry().yoff + fwp.geometry().sy as core::ffi::c_int
                    {
                        if px == bdr_right {
                            return KEYC_MOUSE_LOCATION_BORDER;
                        }
                        if window_pane_is_floating(
                            &w,
                            &{ (wp).observation() }
                                .expect("the pane allocation exists"),
                        ) != 0
                            && px == bdr_left
                        {
                            return KEYC_MOUSE_LOCATION_BORDER;
                        }
                    }
                    if px >= bdr_left
                        && px <= fwp.geometry().xoff + fwp.geometry().sx as core::ffi::c_int
                    {
                        bdr_bottom = (fwp.geometry().yoff as u_int).wrapping_add(fwp.geometry().sy)
                            as core::ffi::c_int;
                        if py == bdr_bottom {
                            return KEYC_MOUSE_LOCATION_BORDER;
                        }
                        if py == bdr_top {
                            return KEYC_MOUSE_LOCATION_BORDER;
                        }
                    }
                }
            }
        }
        KEYC_MOUSE_LOCATION_NOWHERE
    }
}
unsafe fn server_client_check_mouse(c: &mut client, event: &mut key_event) -> key_code {
    unsafe {
        let current_block: u64;
        let m: &mut mouse_event = &mut event.m;
        let Some(session) = c.attached_session() else {
            return KEYC_UNKNOWN;
        };
        let Some(window) = session.current_window() else {
            return KEYC_UNKNOWN;
        };
        let mut selected_pane = None;
        let mut last_pane = None;
        let mut x: u_int = 0;
        let mut y: u_int = 0;
        let sx: u_int;
        let sy: u_int;
        let mut px: u_int = 0;
        let mut py: u_int = 0;
        let mut n: u_int;
        let mut sl_mpos: u_int = 0 as u_int;
        let mut b: u_int = 0;
        let bn: u_int;
        let mut ignore: core::ffi::c_int = 0 as core::ffi::c_int;
        let mut key: key_code;
        let mut tv = timeval::default();
        let mut type_0: key_code_type = KEYC_TYPE_NOTYPE;
        let mut loc: key_code_mouse_location = KEYC_MOUSE_LOCATION_NOWHERE;
        log_debug(
            c"%s mouse %02x at %u,%u (last %u,%u) (%d)",
            fmt_args![
                c.name.as_deref(),
                m.b,
                m.x,
                m.y,
                m.lx,
                m.ly,
                c.tty.mouse_drag_flag
            ],
        );
        if c.tty.mouse_last_pane != -(1 as core::ffi::c_int) {
            last_pane = window_pane_find_by_id(c.tty.mouse_last_pane as u_int);
            if let Some(pane) = last_pane.as_ref() {
                log_debug(
                    c"%s mouse last pane %%%u",
                    fmt_args![c.name.as_deref(), pane.id()],
                );
            }
        }
        if event.key == KEYC_DOUBLECLICK as core::ffi::c_ulong as key_code {
            type_0 = KEYC_TYPE_DOUBLECLICK;
            x = m.x;
            y = m.y;
            b = m.b;
            ignore = 1 as core::ffi::c_int;
            log_debug(c"double-click at %u,%u", fmt_args![x, y]);
        } else if m.sgr_type != ' ' as i32 as u_int
            && m.sgr_b & MOUSE_MASK_DRAG as u_int != 0
            && m.sgr_b & MOUSE_MASK_BUTTONS as u_int == 3 as u_int
            || m.sgr_type == ' ' as i32 as u_int
                && m.b & MOUSE_MASK_DRAG as u_int != 0
                && m.b & MOUSE_MASK_BUTTONS as u_int == 3 as u_int
                && m.lb & MOUSE_MASK_BUTTONS as u_int == 3 as u_int
        {
            type_0 = KEYC_TYPE_MOUSEMOVE;
            x = m.x;
            y = m.y;
            b = 0 as u_int;
            log_debug(c"move at %u,%u", fmt_args![x, y]);
        } else if m.b & MOUSE_MASK_DRAG as u_int != 0 {
            type_0 = KEYC_TYPE_MOUSEDRAG;
            if c.tty.mouse_drag_flag != 0 {
                x = m.x;
                y = m.y;
                b = m.b;
                if x == m.lx && y == m.ly {
                    return KEYC_UNKNOWN as core::ffi::c_ulong as key_code;
                }
                log_debug(c"drag update at %u,%u", fmt_args![x, y]);
            } else {
                x = m.lx;
                y = m.ly;
                b = m.lb;
                log_debug(c"drag start at %u,%u", fmt_args![x, y]);
            }
        } else if m.b & MOUSE_MASK_BUTTONS as u_int == MOUSE_WHEEL_UP as u_int
            || m.b & MOUSE_MASK_BUTTONS as u_int == MOUSE_WHEEL_DOWN as u_int
        {
            if m.b & MOUSE_MASK_BUTTONS as u_int == MOUSE_WHEEL_UP as u_int {
                type_0 = KEYC_TYPE_WHEELUP;
            } else {
                type_0 = KEYC_TYPE_WHEELDOWN;
            }
            x = m.x;
            y = m.y;
            b = m.b;
            log_debug(c"wheel at %u,%u", fmt_args![x, y]);
        } else if m.b & MOUSE_MASK_BUTTONS as u_int == 3 as u_int {
            type_0 = KEYC_TYPE_MOUSEUP;
            x = m.x;
            y = m.y;
            b = m.lb;
            if m.sgr_type == 'm' as i32 as u_int {
                b = m.sgr_b;
            }
            log_debug(c"up at %u,%u", fmt_args![x, y]);
        } else {
            if c.flags & CLIENT_DOUBLECLICK as uint64_t != 0 {
                c.click_timer.disarm();
                c.flags &= !CLIENT_DOUBLECLICK as uint64_t;
                type_0 = KEYC_TYPE_SECONDCLICK;
                x = m.x;
                y = m.y;
                b = m.b;
                log_debug(c"second-click at %u,%u", fmt_args![x, y]);
                c.flags |= CLIENT_TRIPLECLICK as uint64_t;
                current_block = 9441801433784995173;
            } else if c.flags & CLIENT_TRIPLECLICK as uint64_t != 0 {
                c.click_timer.disarm();
                c.flags &= !CLIENT_TRIPLECLICK as uint64_t;
                type_0 = KEYC_TYPE_TRIPLECLICK;
                x = m.x;
                y = m.y;
                b = m.b;
                log_debug(c"triple-click at %u,%u", fmt_args![x, y]);
                current_block = 12711162995783632332;
            } else {
                current_block = 9441801433784995173;
            }
            match current_block {
                12711162995783632332 => {}
                _ => {
                    if type_0 as core::ffi::c_uint
                        == KEYC_TYPE_NOTYPE as core::ffi::c_int as core::ffi::c_uint
                    {
                        type_0 = KEYC_TYPE_MOUSEDOWN;
                        x = m.x;
                        y = m.y;
                        b = m.b;
                        log_debug(c"down at %u,%u", fmt_args![x, y]);
                        c.flags |= CLIENT_DOUBLECLICK as uint64_t;
                    }
                }
            }
        }
        if type_0 as core::ffi::c_uint == KEYC_TYPE_NOTYPE as core::ffi::c_int as core::ffi::c_uint
        {
            return KEYC_UNKNOWN as core::ffi::c_ulong as key_code;
        }
        m.s = session.id() as core::ffi::c_int;
        m.w = -(1 as core::ffi::c_int);
        m.wp = -(1 as core::ffi::c_int);
        m.ignore = ignore;
        m.statusat = status_at_line(&*c);
        m.statuslines = status_line_size(&*c);
        if m.statusat != -(1 as core::ffi::c_int)
            && y >= m.statusat as u_int
            && y < (m.statusat as u_int).wrapping_add(m.statuslines)
        {
            if let Some(sr) = status_get_range(c, x, y.wrapping_sub(m.statusat as u_int)) {
                match sr.type_0 {
                    STYLE_RANGE_NONE => return KEYC_UNKNOWN as core::ffi::c_ulong as key_code,
                    STYLE_RANGE_LEFT => {
                        log_debug(c"mouse range: left", fmt_args![]);
                        loc = KEYC_MOUSE_LOCATION_STATUS_LEFT;
                    }
                    STYLE_RANGE_RIGHT => {
                        log_debug(c"mouse range: right", fmt_args![]);
                        loc = KEYC_MOUSE_LOCATION_STATUS_RIGHT;
                    }
                    STYLE_RANGE_PANE => {
                        if window_pane_find_by_id(sr.argument).is_none() {
                            return KEYC_UNKNOWN as core::ffi::c_ulong as key_code;
                        }
                        m.wp = sr.argument as core::ffi::c_int;
                        log_debug(c"mouse range: pane %%%u", fmt_args![m.wp]);
                        loc = KEYC_MOUSE_LOCATION_STATUS;
                    }
                    STYLE_RANGE_WINDOW => {
                        let Some(linked_window) = session
                            .as_session()
                            .windows
                            .get(&(sr.argument as core::ffi::c_int))
                            .and_then(|link| link.window_handle())
                        else {
                            return KEYC_UNKNOWN;
                        };
                        m.w = linked_window.window_id() as core::ffi::c_int;
                        log_debug(c"mouse range: window @%u", fmt_args![m.w]);
                        loc = KEYC_MOUSE_LOCATION_STATUS;
                    }
                    STYLE_RANGE_SESSION => {
                        if SessionRef::find_by_id(sr.argument).is_none() {
                            return KEYC_UNKNOWN as core::ffi::c_ulong as key_code;
                        }
                        m.s = sr.argument as core::ffi::c_int;
                        log_debug(c"mouse range: session $%u", fmt_args![m.s]);
                        loc = KEYC_MOUSE_LOCATION_STATUS;
                    }
                    STYLE_RANGE_USER => {
                        log_debug(c"mouse range: user", fmt_args![]);
                        loc = KEYC_MOUSE_LOCATION_STATUS;
                    }
                    STYLE_RANGE_CONTROL => {
                        n = sr.argument;
                        log_debug(c"mouse range: control %u", fmt_args![n]);
                        loc = (KEYC_MOUSE_LOCATION_CONTROL0 as core::ffi::c_int as u_int)
                            .wrapping_add(n)
                            as key_code_mouse_location;
                    }
                    _ => {}
                }
            } else {
                loc = KEYC_MOUSE_LOCATION_STATUS_DEFAULT;
            }
        }
        if loc as core::ffi::c_uint
            == KEYC_MOUSE_LOCATION_NOWHERE as core::ffi::c_int as core::ffi::c_uint
        {
            if c.tty.mouse_scrolling_flag != 0 {
                if let Some(pane) = last_pane.as_ref()
                    && let Some(pane_window) = pane.window()
                {
                    loc = KEYC_MOUSE_LOCATION_SCROLLBAR_SLIDER;
                    m.wp = pane.id() as core::ffi::c_int;
                    m.w = pane_window.window_id() as core::ffi::c_int;
                }
            } else {
                px = x;
                if m.statusat == 0 as core::ffi::c_int && y >= m.statuslines {
                    py = y.wrapping_sub(m.statuslines);
                } else if m.statusat > 0 as core::ffi::c_int && y >= m.statusat as u_int {
                    py = (m.statusat - 1 as core::ffi::c_int) as u_int;
                } else {
                    py = y;
                }
                {
                    let (window_bigger, off_x, off_y, off_sx, off_sy) = tty_window_offset(&c.tty);
                    (m.ox, m.oy, sx, sy) = (off_x, off_y, off_sx, off_sy);
                    window_bigger
                };
                log_debug(
                    c"mouse window @%u at %u,%u (%ux%u)",
                    fmt_args![window.window_id(), m.ox, m.oy, sx, sy],
                );
                if px > sx || py > sy {
                    return KEYC_UNKNOWN as core::ffi::c_ulong as key_code;
                }
                px = px.wrapping_add(m.ox);
                py = py.wrapping_add(m.oy);
                if type_0 as core::ffi::c_uint
                    == KEYC_TYPE_MOUSEDRAG as core::ffi::c_int as core::ffi::c_uint
                    && last_pane.is_some()
                {
                    selected_pane = last_pane.clone();
                } else {
                    selected_pane = window_get_active_at(&window.as_window(), px, py);
                }
                let Some(pane) = selected_pane.as_ref() else {
                    return KEYC_UNKNOWN as core::ffi::c_ulong as key_code;
                };
                let Some(wp) = pane.get() else {
                    return KEYC_UNKNOWN;
                };
                let Some(pane_window) = pane.window() else {
                    return KEYC_UNKNOWN;
                };
                loc = server_client_check_mouse_in_pane(
                    pane,
                    px as core::ffi::c_int,
                    py as core::ffi::c_int,
                    &mut sl_mpos,
                );
                if loc as core::ffi::c_uint
                    == KEYC_MOUSE_LOCATION_PANE as core::ffi::c_int as core::ffi::c_uint
                {
                    log_debug(c"mouse %u,%u on pane %%%u", fmt_args![x, y, pane.id()]);
                } else if loc as core::ffi::c_uint
                    == KEYC_MOUSE_LOCATION_BORDER as core::ffi::c_int as core::ffi::c_uint
                {
                    if let Some(sr) = window_pane_border_status_get_range(Some(wp), px, py) {
                        n = sr.argument;
                        loc = (KEYC_MOUSE_LOCATION_CONTROL0 as core::ffi::c_int as u_int)
                            .wrapping_add(n)
                            as key_code_mouse_location;
                    }
                    log_debug(c"mouse on pane %%%u border", fmt_args![pane.id()]);
                } else if loc as core::ffi::c_uint
                    == KEYC_MOUSE_LOCATION_SCROLLBAR_UP as core::ffi::c_int as core::ffi::c_uint
                    || loc as core::ffi::c_uint
                        == KEYC_MOUSE_LOCATION_SCROLLBAR_SLIDER as core::ffi::c_int
                            as core::ffi::c_uint
                    || loc as core::ffi::c_uint
                        == KEYC_MOUSE_LOCATION_SCROLLBAR_DOWN as core::ffi::c_int
                            as core::ffi::c_uint
                {
                    log_debug(c"mouse on pane %%%u scrollbar", fmt_args![pane.id()]);
                }
                m.wp = pane.id() as core::ffi::c_int;
                m.w = pane_window.window_id() as core::ffi::c_int;
            }
        }
        if type_0 as core::ffi::c_uint
            == KEYC_TYPE_MOUSEDOWN as core::ffi::c_int as core::ffi::c_uint
            || type_0 as core::ffi::c_uint
                == KEYC_TYPE_SECONDCLICK as core::ffi::c_int as core::ffi::c_uint
            || type_0 as core::ffi::c_uint
                == KEYC_TYPE_TRIPLECLICK as core::ffi::c_int as core::ffi::c_uint
        {
            if type_0 as core::ffi::c_uint
                != KEYC_TYPE_MOUSEDOWN as core::ffi::c_int as core::ffi::c_uint
                && (m.b != c.click_button
                    || loc as core::ffi::c_uint
                        != c.click_loc as key_code_mouse_location as core::ffi::c_uint
                    || m.wp != c.click_wp)
            {
                type_0 = KEYC_TYPE_MOUSEDOWN;
                log_debug(c"click sequence reset at %u,%u", fmt_args![x, y]);
                c.flags &= !CLIENT_TRIPLECLICK as uint64_t;
                c.flags |= CLIENT_DOUBLECLICK as uint64_t;
            }
            if type_0 as core::ffi::c_uint
                != KEYC_TYPE_TRIPLECLICK as core::ffi::c_int as core::ffi::c_uint
                && KEYC_CLICK_TIMEOUT != 0 as core::ffi::c_int
            {
                c.click_event = *m;
                c.click_button = m.b;
                c.click_loc = loc as core::ffi::c_int;
                c.click_wp = m.wp;
                log_debug(c"click timer started", fmt_args![]);
                tv.tv_sec = (KEYC_CLICK_TIMEOUT / 1000 as core::ffi::c_int) as __time_t;
                tv.tv_usec = ((KEYC_CLICK_TIMEOUT % 1000 as core::ffi::c_int) as core::ffi::c_long
                    * 1000 as core::ffi::c_long) as __suseconds_t;
                c.click_timer.disarm();
                c.click_timer.arm(tv);
            }
        }
        key = KEYC_UNKNOWN as core::ffi::c_ulong as key_code;
        if type_0 as core::ffi::c_uint
            != KEYC_TYPE_MOUSEDRAG as core::ffi::c_int as core::ffi::c_uint
            && type_0 as core::ffi::c_uint
                != KEYC_TYPE_WHEELUP as core::ffi::c_int as core::ffi::c_uint
            && type_0 as core::ffi::c_uint
                != KEYC_TYPE_WHEELDOWN as core::ffi::c_int as core::ffi::c_uint
            && type_0 as core::ffi::c_uint
                != KEYC_TYPE_DOUBLECLICK as core::ffi::c_int as core::ffi::c_uint
            && type_0 as core::ffi::c_uint
                != KEYC_TYPE_TRIPLECLICK as core::ffi::c_int as core::ffi::c_uint
            && c.tty.mouse_drag_flag != 0 as core::ffi::c_int
        {
            if let Some(release) = c.tty.mouse_drag_release.clone() {
                release(&mut *c, &*m);
            }
            c.tty.mouse_drag_update = None;
            c.tty.mouse_drag_release = None;
            c.tty.mouse_scrolling_flag = 0 as core::ffi::c_int;
            type_0 = KEYC_TYPE_MOUSEDRAGEND;
            c.tty.mouse_drag_flag = 0 as core::ffi::c_int;
            c.tty.mouse_slider_mpos = -(1 as core::ffi::c_int);
            c.tty.mouse_last_pane = -(1 as core::ffi::c_int);
        }
        if type_0 as core::ffi::c_uint
            == KEYC_TYPE_MOUSEMOVE as core::ffi::c_int as core::ffi::c_uint
            && loc as core::ffi::c_uint
                == KEYC_MOUSE_LOCATION_PANE as core::ffi::c_int as core::ffi::c_uint
        {
            key = KEYC_MOUSEMOVE_PANE as core::ffi::c_ulong as key_code;
            if let Some(pane) = selected_pane.as_ref()
                && pane.get().is_some()
                && pane.window().is_some_and(|owner| owner.ptr_eq(&window))
                && !window
                    .active_pane()
                    .is_some_and(|active| active.ptr_eq(pane))
                && session.options().number(c"focus-follows-mouse") != 0
            {
                window_redraw_active_switch(
                    &mut window.as_window_mut(),
                    &crate::window::window_pane_find_by_id(pane.id())
                        .expect("the selected pane exists"),
                );
                window.set_active_pane(
                    &crate::window::window_pane_find_by_id(pane.id())
                        .expect("the selected pane exists"),
                    1 as core::ffi::c_int,
                );
                window.redraw_borders();
                window.redraw_status();
            }
        }
        if type_0 as core::ffi::c_uint
            == KEYC_TYPE_MOUSEDRAG as core::ffi::c_int as core::ffi::c_uint
        {
            if c.tty.mouse_drag_update.is_some() {
                key = KEYC_DRAGGING as core::ffi::c_ulong as key_code;
            }
            c.tty.mouse_drag_flag =
                (b & MOUSE_MASK_BUTTONS as u_int).wrapping_add(1 as u_int) as core::ffi::c_int;
            if last_pane.is_none()
                && let Some(pane) = window_get_active_at(&window.as_window(), px, py)
            {
                c.tty.mouse_last_pane = pane.id() as core::ffi::c_int;
            }
            if c.tty.mouse_scrolling_flag == 0 as core::ffi::c_int
                && loc as core::ffi::c_uint
                    == KEYC_MOUSE_LOCATION_SCROLLBAR_SLIDER as core::ffi::c_int as core::ffi::c_uint
            {
                c.tty.mouse_scrolling_flag = 1 as core::ffi::c_int;
                if m.statusat == 0 as core::ffi::c_int {
                    c.tty.mouse_slider_mpos =
                        sl_mpos.wrapping_add(m.statuslines) as core::ffi::c_int;
                } else {
                    c.tty.mouse_slider_mpos = sl_mpos as core::ffi::c_int;
                }
            }
        }
        if key == KEYC_UNKNOWN as core::ffi::c_ulong as key_code {
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
            key = ((type_0 as core::ffi::c_ulonglong) << 32 as core::ffi::c_int
                | (bn as core::ffi::c_ulonglong) << KEYC_MOUSE_BUTTON_SHIFT
                | (loc as core::ffi::c_ulonglong) << KEYC_MOUSE_LOCATION_SHIFT)
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
        if log_get_level() != 0 as core::ffi::c_int {
            let key_name = RustKeyStringCodec.format_key(key, true);
            log_debug(c"mouse key is %s", fmt_args![key_name.as_ref()]);
        }
        key
    }
}
unsafe fn server_client_is_bracket_paste(c: &mut client, key: key_code) -> core::ffi::c_int {
    {
        if key as core::ffi::c_ulonglong & KEYC_MASK_KEY
            == KEYC_PASTE_START as core::ffi::c_ulong as core::ffi::c_ulonglong
        {
            c.flags = (c.flags as core::ffi::c_ulonglong | CLIENT_BRACKETPASTING) as uint64_t;
            c.paste_time = current_time.get();
            log_debug(c"%s: bracket paste on", fmt_args![c.name.as_deref()]);
            return 0 as core::ffi::c_int;
        }
        if key as core::ffi::c_ulonglong & KEYC_MASK_KEY
            == KEYC_PASTE_END as core::ffi::c_ulong as core::ffi::c_ulonglong
        {
            c.flags = (c.flags as core::ffi::c_ulonglong & !CLIENT_BRACKETPASTING) as uint64_t;
            log_debug(c"%s: bracket paste off", fmt_args![c.name.as_deref()]);
            return 0 as core::ffi::c_int;
        }
        (c.flags as core::ffi::c_ulonglong & CLIENT_BRACKETPASTING != 0) as core::ffi::c_int
    }
}
unsafe fn server_client_is_assume_paste(c: &mut client) -> core::ffi::c_int {
    {
        let Some(session) = c.attached_session() else {
            return 0;
        };
        let mut tv = timeval::default();

        if c.flags as core::ffi::c_ulonglong & CLIENT_BRACKETPASTING != 0 {
            return 0 as core::ffi::c_int;
        }
        let t: core::ffi::c_int =
            (session.options()).number(c"assume-paste-time") as core::ffi::c_int;
        if t == 0 as core::ffi::c_int {
            return 0 as core::ffi::c_int;
        }
        if tty_term_of(&c.tty).has(TTYC_ENBP) {
            return 0 as core::ffi::c_int;
        }
        tv.tv_sec = c.activity_time.tv_sec - c.last_activity_time.tv_sec;
        tv.tv_usec = c.activity_time.tv_usec - c.last_activity_time.tv_usec;
        if tv.tv_usec < 0 as __suseconds_t {
            tv.tv_sec -= 1;
            tv.tv_usec += 1000000 as __suseconds_t;
        }
        if tv.tv_sec == 0 as __time_t
            && tv.tv_usec < (t * 1000 as core::ffi::c_int) as __suseconds_t
        {
            if c.flags as core::ffi::c_ulonglong & CLIENT_ASSUMEPASTING != 0 {
                return 1 as core::ffi::c_int;
            }
            c.flags = (c.flags as core::ffi::c_ulonglong | CLIENT_ASSUMEPASTING) as uint64_t;
            c.paste_time = current_time.get();
            log_debug(c"%s: assume paste on", fmt_args![c.name.as_deref()]);
            return 0 as core::ffi::c_int;
        }
        if c.flags as core::ffi::c_ulonglong & CLIENT_ASSUMEPASTING != 0 {
            c.flags = (c.flags as core::ffi::c_ulonglong & !CLIENT_ASSUMEPASTING) as uint64_t;
            log_debug(c"%s: assume paste off", fmt_args![c.name.as_deref()]);
        }
        0 as core::ffi::c_int
    }
}
unsafe fn server_client_update_latest(c: &mut client) {
    unsafe {
        let Some(session) = c.attached_session() else {
            return;
        };
        let Some(window) = session.current_window() else {
            return;
        };
        if let Some(client) = client_ref_of(c)
            && window_get_latest(&window.as_window()).is_some_and(|latest| latest.ptr_eq(&client))
        {
            return;
        }
        window_set_latest(&mut window.as_window_mut(), Some(c));
        if window.options().number(c"window-size") == WINDOW_SIZE_LATEST as i64 {
            window.recalculate_size(0);
        }
        notify_client(c"client-active", Some(c));
    }
}
fn server_client_repeat_time(c: &client, bd: &key_binding) -> u_int {
    {
        let Some(session) = c.attached_session() else {
            return 0;
        };
        let mut repeat: u_int;
        let initial: u_int;
        if !key_binding_flags(bd) & KEY_BINDING_REPEAT != 0 {
            return 0 as u_int;
        }
        repeat = (session.options()).number(c"repeat-time") as u_int;
        if repeat == 0 as u_int {
            return 0 as u_int;
        }
        if !c.flags & CLIENT_REPEAT as uint64_t != 0 || key_binding_key(bd) != c.last_key {
            initial = (session.options()).number(c"initial-repeat-time") as u_int;
            if initial != 0 as u_int {
                repeat = initial;
            }
        }
        repeat
    }
}
fn server_client_key_callback(item: &CmdqItemRef, mut event: Box<key_event>) -> cmd_retval {
    unsafe {
        let item_ref = item;
        let mut current_block: u64;
        let mut client = item_ref.client().expect("key callback has a client");
        let event = event.as_mut();
        let mut key = event.key;
        let mut session = client.attached_session();
        let mut tv = timeval::default();
        let mut table: KeyTableRef;

        let mut first: KeyTableRef;
        let mut bd: Option<key_binding>;
        let repeat: u_int;
        let mut flags: uint64_t;
        let mut prefix_delay: uint64_t;
        let mut fs = cmd_find_state::default();
        let mut key0: key_code;
        let mut prefix: key_code;
        let mut prefix2: key_code;
        if let Some(session) = session
            .as_mut()
            .filter(|_| client.flags() & CLIENT_UNATTACHEDFLAGS as uint64_t == 0)
        {
            client.as_client_mut().last_activity_time = client.as_client().activity_time;
            client.as_client_mut().activity_time = timeval::now();
            session.update_activity(Some(&client.as_client().activity_time));
            event.m.valid = 0 as core::ffi::c_int;
            if key == KEYC_MOUSE as core::ffi::c_ulong as key_code
                || key == KEYC_DOUBLECLICK as core::ffi::c_ulong as key_code
            {
                if client.flags() & CLIENT_READONLY as uint64_t != 0 {
                    current_block = 16906968430444679536;
                } else {
                    key = server_client_check_mouse(client.as_client_mut(), event);
                    if key == KEYC_UNKNOWN as core::ffi::c_ulong as key_code {
                        current_block = 16906968430444679536;
                    } else {
                        event.m.valid = 1 as core::ffi::c_int;
                        event.m.key = key;
                        if key as core::ffi::c_ulonglong & KEYC_MASK_KEY
                            == KEYC_DRAGGING as core::ffi::c_ulong as core::ffi::c_ulonglong
                        {
                            client
                                .as_tty()
                                .mouse_drag_update
                                .clone()
                                .expect("missing drag update")(
                                client.as_client_mut(), &event.m
                            );
                            current_block = 16906968430444679536;
                        } else {
                            event.key = key;
                            current_block = 5948590327928692120;
                        }
                    }
                }
            } else {
                current_block = 5948590327928692120;
            }
            match current_block {
                16906968430444679536 => {}
                _ => {
                    if !(key as core::ffi::c_ulonglong & KEYC_MASK_KEY
                        == KEYC_MOUSE as core::ffi::c_ulong as core::ffi::c_ulonglong
                        || key as core::ffi::c_ulonglong & KEYC_MASK_TYPE
                            >= (KEYC_TYPE_MOUSEMOVE as core::ffi::c_int as core::ffi::c_ulonglong)
                                << 32 as core::ffi::c_int
                            && key as core::ffi::c_ulonglong & KEYC_MASK_TYPE
                                <= (KEYC_TYPE_TRIPLECLICK as core::ffi::c_int
                                    as core::ffi::c_ulonglong)
                                    << 32 as core::ffi::c_int)
                        || cmd_find_from_mouse(&mut fs, &event.m, 0 as core::ffi::c_int)
                            != 0 as core::ffi::c_int
                    {
                        cmd_find_from_client(&mut fs, Some(&client), 0 as core::ffi::c_int);
                    }
                    let pane = fs.pane_list_ref();
                    if (key as core::ffi::c_ulonglong & KEYC_MASK_KEY
                        == KEYC_MOUSE as core::ffi::c_ulong as core::ffi::c_ulonglong
                        || key as core::ffi::c_ulonglong & KEYC_MASK_TYPE
                            >= (KEYC_TYPE_MOUSEMOVE as core::ffi::c_int as core::ffi::c_ulonglong)
                                << 32 as core::ffi::c_int
                            && key as core::ffi::c_ulonglong & KEYC_MASK_TYPE
                                <= (KEYC_TYPE_TRIPLECLICK as core::ffi::c_int
                                    as core::ffi::c_ulonglong)
                                    << 32 as core::ffi::c_int)
                        && (session.options()).number(c"mouse") == 0
                    {
                        current_block = 4899622246340545966;
                    } else {
                        if server_client_is_bracket_paste(client.as_client_mut(), key) != 0
                            || (!(key as core::ffi::c_ulonglong & KEYC_MASK_KEY
                                == KEYC_MOUSE as core::ffi::c_ulong as core::ffi::c_ulonglong
                                || key as core::ffi::c_ulonglong & KEYC_MASK_TYPE
                                    >= (KEYC_TYPE_MOUSEMOVE as core::ffi::c_int
                                        as core::ffi::c_ulonglong)
                                        << 32 as core::ffi::c_int
                                    && key as core::ffi::c_ulonglong & KEYC_MASK_TYPE
                                        <= (KEYC_TYPE_TRIPLECLICK as core::ffi::c_int
                                            as core::ffi::c_ulonglong)
                                            << 32 as core::ffi::c_int))
                                && key != KEYC_FOCUS_IN as core::ffi::c_ulong as key_code
                                && key != KEYC_FOCUS_OUT as core::ffi::c_ulong as key_code
                                && !(key as core::ffi::c_ulonglong) & KEYC_SENT != 0
                                && server_client_is_assume_paste(client.as_client_mut()) != 0
                        {
                            current_block = 875716535476481131;
                        } else {
                            let keytable = client.keytable().expect("client key table");
                            if server_client_is_default_key_table(
                                client.as_client(),
                                &keytable.borrow(),
                            ) != 0
                                && let Some(wp) = pane.as_ref().and_then(|pane| pane.get())
                                && let Some(wme) = crate::window::window_pane_current_mode(wp)
                                && let Some(name) = wme.mode().key_table(wme)
                            {
                                table =
                                    key_bindings_get_table_ref(name, 1).expect("mode key table");
                            } else {
                                table = client.keytable().expect("client key table");
                            }
                            first = table.clone();
                            '_table_changed: loop {
                                prefix = (session.options()).number(c"prefix") as key_code;
                                prefix2 = (session.options()).number(c"prefix2") as key_code;
                                key0 = (key as core::ffi::c_ulonglong
                                    & (KEYC_MASK_KEY | KEYC_MASK_MODIFIERS))
                                    as key_code;
                                if (key0
                                    == prefix as core::ffi::c_ulonglong
                                        & (KEYC_MASK_KEY | KEYC_MASK_MODIFIERS)
                                    || key0
                                        == prefix2 as core::ffi::c_ulonglong
                                            & (KEYC_MASK_KEY | KEYC_MASK_MODIFIERS))
                                    && table.name().as_c_str() != c"prefix"
                                {
                                    server_client_set_key_table(
                                        client.as_client_mut(),
                                        Some(c"prefix"),
                                    );
                                    server_status_client(client.as_client_mut());
                                    current_block = 16906968430444679536;
                                    break;
                                } else {
                                    flags = client.flags();
                                    loop {
                                        if let Some(pane) = &pane {
                                            log_debug(
                                                c"key table %s (pane %%%u)",
                                                fmt_args![table.name().as_c_str(), pane.id()],
                                            );
                                        } else {
                                            log_debug(
                                                c"key table %s (no pane)",
                                                fmt_args![table.name().as_c_str()],
                                            );
                                        }
                                        if client.flags() & CLIENT_REPEAT as uint64_t != 0 {
                                            log_debug(c"currently repeating", fmt_args![]);
                                        }
                                        bd = table.borrow().binding(key0).cloned();
                                        prefix_delay = (global_options
                                            .get()
                                            .as_ref()
                                            .expect("global options are initialized"))
                                        .number(c"prefix-timeout")
                                            as uint64_t;
                                        if prefix_delay > 0 as uint64_t
                                            && table.name().as_c_str() == c"prefix"
                                            && server_client_key_table_activity_diff(
                                                client.as_client(),
                                            ) > prefix_delay
                                        {
                                            if bd.is_some()
                                                && client.flags() & CLIENT_REPEAT as uint64_t != 0
                                                && key_binding_flags(
                                                    bd.as_ref().expect("binding was found"),
                                                ) & KEY_BINDING_REPEAT
                                                    != 0
                                            {
                                                log_debug(
                                                    c"prefix timeout ignored, repeat is active",
                                                    fmt_args![],
                                                );
                                            } else {
                                                log_debug(c"prefix timeout exceeded", fmt_args![]);
                                                server_client_set_key_table(
                                                    client.as_client_mut(),
                                                    None,
                                                );
                                                table =
                                                    client.keytable().expect("client key table");
                                                first = table.clone();
                                                server_status_client(client.as_client_mut());
                                                continue '_table_changed;
                                            }
                                        }
                                        if let Some(bd) = &bd {
                                            if client.flags() & CLIENT_REPEAT as uint64_t != 0
                                                && !key_binding_flags(bd) & KEY_BINDING_REPEAT != 0
                                            {
                                                current_block = 6717214610478484138;
                                                break;
                                            } else {
                                                current_block = 14220266465818359136;
                                                break;
                                            }
                                        } else if key0 != KEYC_ANY as core::ffi::c_ulong as key_code
                                        {
                                            key0 = KEYC_ANY as core::ffi::c_ulong as key_code;
                                        } else {
                                            if key
                                                == KEYC_MOUSEMOVE_PANE as core::ffi::c_ulong
                                                    as key_code
                                                || key
                                                    == KEYC_MOUSEMOVE_STATUS as core::ffi::c_ulong
                                                        as key_code
                                                || key
                                                    == KEYC_MOUSEMOVE_STATUS_LEFT
                                                        as core::ffi::c_ulong
                                                        as key_code
                                                || key
                                                    == KEYC_MOUSEMOVE_STATUS_RIGHT
                                                        as core::ffi::c_ulong
                                                        as key_code
                                                || key
                                                    == KEYC_MOUSEMOVE_STATUS_DEFAULT
                                                        as core::ffi::c_ulong
                                                        as key_code
                                                || key
                                                    == KEYC_MOUSEMOVE_BORDER as core::ffi::c_ulong
                                                        as key_code
                                            {
                                                current_block = 4899622246340545966;
                                                break '_table_changed;
                                            }
                                            log_debug(
                                                c"not found in key table %s",
                                                fmt_args![table.name().as_c_str()],
                                            );
                                            if server_client_is_default_key_table(
                                                client.as_client(),
                                                &table.borrow(),
                                            ) == 0
                                                || client.flags() & CLIENT_REPEAT as uint64_t != 0
                                            {
                                                current_block = 981995395831942902;
                                                break;
                                            } else {
                                                current_block = 13853033528615664019;
                                                break;
                                            }
                                        }
                                    }
                                    match current_block {
                                        13853033528615664019 => {
                                            if !first.ptr_eq(&table)
                                                && !flags & CLIENT_REPEAT as uint64_t != 0
                                            {
                                                current_block = 7178192492338286402;
                                                break;
                                            } else {
                                                current_block = 4899622246340545966;
                                                break;
                                            }
                                        }
                                        14220266465818359136 => {
                                            log_debug(
                                                c"found in key table %s",
                                                fmt_args![table.name().as_c_str()],
                                            );
                                            repeat = server_client_repeat_time(
                                                client.as_client(),
                                                bd.as_ref().expect("binding was found"),
                                            );
                                            if repeat != 0 as u_int {
                                                *client.flags_mut() |= CLIENT_REPEAT as uint64_t;
                                                client.as_client_mut().last_key = key_binding_key(
                                                    bd.as_ref().expect("binding was found"),
                                                );
                                                tv.tv_sec =
                                                    repeat.wrapping_div(1000 as u_int) as __time_t;
                                                tv.tv_usec = (repeat.wrapping_rem(1000 as u_int)
                                                    as core::ffi::c_long
                                                    * 1000 as core::ffi::c_long)
                                                    as __suseconds_t;
                                                client.as_client_mut().repeat_timer.disarm();
                                                client.as_client_mut().repeat_timer.arm(tv);
                                            } else {
                                                *client.flags_mut() &= !CLIENT_REPEAT as uint64_t;
                                                server_client_set_key_table(
                                                    client.as_client_mut(),
                                                    None,
                                                );
                                            }
                                            server_status_client(client.as_client_mut());
                                            key_bindings_dispatch(
                                                bd.as_ref().expect("binding was found"),
                                                Some(item_ref),
                                                Some(client.as_client_mut()),
                                                Some(&*event),
                                                Some(&fs),
                                            );
                                            current_block = 16906968430444679536;
                                            break;
                                        }
                                        981995395831942902 => {
                                            log_debug(c"trying in root table", fmt_args![]);
                                            server_client_set_key_table(
                                                client.as_client_mut(),
                                                None,
                                            );
                                            table = client.keytable().expect("client key table");
                                            if client.flags() & CLIENT_REPEAT as uint64_t != 0 {
                                                first = table.clone();
                                            }
                                            *client.flags_mut() &= !CLIENT_REPEAT as uint64_t;
                                            server_status_client(client.as_client_mut());
                                        }
                                        _ => {
                                            log_debug(
                                                c"found in key table %s (not repeating)",
                                                fmt_args![table.name().as_c_str()],
                                            );
                                            server_client_set_key_table(
                                                client.as_client_mut(),
                                                None,
                                            );
                                            table = client.keytable().expect("client key table");
                                            first = table.clone();
                                            *client.flags_mut() &= !CLIENT_REPEAT as uint64_t;
                                            server_status_client(client.as_client_mut());
                                        }
                                    }
                                }
                            }
                            match current_block {
                                16906968430444679536 => {}
                                4899622246340545966 => {}
                                _ => {
                                    server_client_set_key_table(client.as_client_mut(), None);
                                    server_status_client(client.as_client_mut());
                                    current_block = 16906968430444679536;
                                }
                            }
                        }
                        match current_block {
                            16906968430444679536 => {}
                            4899622246340545966 => {}
                            _ => {
                                if client.flags() & CLIENT_READONLY as uint64_t != 0 {
                                    current_block = 16906968430444679536;
                                } else {
                                    if !event.buf.is_empty()
                                        && let Some(wp) = pane.as_ref().and_then(|pane| pane.get())
                                    {
                                        window_pane_paste(
                                            wp,
                                            key,
                                            ByteBuffer::from(core::mem::take(&mut event.buf)),
                                        );
                                    }
                                    key = KEYC_NONE as core::ffi::c_ulong as key_code;
                                    current_block = 16906968430444679536;
                                }
                            }
                        }
                    }
                    match current_block {
                        16906968430444679536 => {}
                        _ => {
                            if let Some(wp) = pane.as_ref().and_then(|pane| pane.get())
                                && *wp.flags() & PANE_EXITED != 0
                                && !(key as core::ffi::c_ulonglong & KEYC_MASK_KEY
                                    == KEYC_MOUSE as core::ffi::c_ulong as core::ffi::c_ulonglong
                                    || key as core::ffi::c_ulonglong & KEYC_MASK_TYPE
                                        >= (KEYC_TYPE_MOUSEMOVE as core::ffi::c_int
                                            as core::ffi::c_ulonglong)
                                            << 32 as core::ffi::c_int
                                        && key as core::ffi::c_ulonglong & KEYC_MASK_TYPE
                                            <= (KEYC_TYPE_TRIPLECLICK as core::ffi::c_int
                                                as core::ffi::c_ulonglong)
                                                << 32 as core::ffi::c_int)
                                && !(key as core::ffi::c_ulonglong & KEYC_MASK_TYPE
                                    == (KEYC_TYPE_FUNCTION as core::ffi::c_int
                                        as core::ffi::c_ulonglong)
                                        << 32 as core::ffi::c_int
                                    && (key as core::ffi::c_ulonglong & KEYC_MASK_KEY
                                        == KEYC_PASTE_START as core::ffi::c_ulong
                                            as core::ffi::c_ulonglong
                                        || key as core::ffi::c_ulonglong & KEYC_MASK_KEY
                                            == KEYC_PASTE_END as core::ffi::c_ulong
                                                as core::ffi::c_ulonglong))
                                && wp.options_ref().number(c"remain-on-exit")
                                    == 3 as core::ffi::c_longlong
                            {
                                wp.options_ref()
                                    .set_number(c"remain-on-exit", 0 as core::ffi::c_longlong);
                                server_destroy_pane(
                                    &(wp).observation()
                                        .expect("the pane is owned"),
                                    0 as core::ffi::c_int,
                                );
                            } else if !(client.flags() & CLIENT_READONLY as uint64_t != 0)
                                && let Some(pane) = pane.filter(|pane| {
                                    pane.get().is_some()
                                        && window_pane_find_by_id(pane.id())
                                            .is_some_and(|registered| registered.ptr_eq(pane))
                                })
                            {
                                window_pane_key(
                                    pane,
                                    Some(client.as_client_mut()),
                                    key,
                                    Some(&event.m),
                                );
                            }
                        }
                    }
                }
            }
        }
        if session.is_some() && key != KEYC_FOCUS_OUT as core::ffi::c_ulong as key_code {
            server_client_update_latest(client.as_client_mut());
        }
        CMD_RETURN_NORMAL
    }
}
pub unsafe fn server_client_handle_key(
    c: &mut client,
    mut event: Box<key_event>,
) -> core::ffi::c_int {
    unsafe {
        if c.attached_session().is_none() || c.flags & CLIENT_UNATTACHEDFLAGS as uint64_t != 0 {
            return 0 as core::ffi::c_int;
        }
        if event.key == KEYC_REPORT_LIGHT_THEME as core::ffi::c_ulong as key_code {
            server_client_report_theme(&mut *c, THEME_LIGHT);
            return 0 as core::ffi::c_int;
        }
        if event.key == KEYC_REPORT_DARK_THEME as core::ffi::c_ulong as key_code {
            server_client_report_theme(&mut *c, THEME_DARK);
            return 0 as core::ffi::c_int;
        }
        if !c.flags & CLIENT_READONLY as uint64_t != 0 {
            if c.message_string.is_some() {
                if c.message_ignore_keys != 0 {
                    return 0 as core::ffi::c_int;
                }
                status_message_clear(&mut *c);
            }
            if (*c).overlay().has_key() {
                let overlay = (*c).overlay();
                let data = (*c).current_overlay_data();
                match overlay.key(c, data, event.as_mut()) {
                    0 => return 0 as core::ffi::c_int,
                    1 => {
                        server_client_clear_overlay(c);
                        return 0 as core::ffi::c_int;
                    }
                    _ => {}
                }
            }
            server_client_clear_overlay(c);
            if c.prompt_string.is_some()
                && status_prompt_key(&mut *c, event.key) == 0 as core::ffi::c_int
            {
                return 0 as core::ffi::c_int;
            }
        }
        cmdq_append(
            crate::server::client_ref_of(c).as_ref(),
            CmdqItemRef::callback_items(c"server_client_key_callback", move |item| {
                server_client_key_callback(item, event)
            }),
        );
        1 as core::ffi::c_int
    }
}
pub fn server_client_loop() {
    WINDOWS.with(|windows| unsafe {
        for window in windows.iter().filter_map(|(_, window)| window.upgrade()) {
            server_client_check_window_resize(&window);
        }
        for window in windows.iter().filter_map(|(_, window)| window.upgrade()) {
            let panes = window.panes();
            for mut pane in panes {
                let Some(wp) = pane.get_mut() else {
                    continue;
                };
                if *wp.flags() & PANE_STYLECHANGED != 0
                    && let Some(wme) = window_pane_current_mode_mut(wp)
                {
                    wme.mode().style_changed(wme);
                }
            }
        }
        for mut c in client_walk() {
            server_client_check_exit(c.as_client_mut());
            let session = c.attached_session();
            if session
                .as_ref()
                .is_some_and(|session| session.curw().is_some())
            {
                server_client_check_modes(c.as_client());
                server_client_check_redraw(c.as_client_mut());
                server_client_reset_state(c.as_client_mut());
            }
        }
        for window in windows.iter().filter_map(|(_, window)| window.upgrade()) {
            let panes = window.panes();
            for mut pane in panes {
                if pane.get().is_some_and(|wp| *wp.fd() != -1) {
                    pane.deliver_pending_resize();
                    if let Some(wp) = pane.get_mut() {
                        wp.maintain_output();
                    }
                }
                if let Some(wp) = pane.get_mut() {
                    *wp.flags_mut() &= !(PANE_REDRAW | PANE_REDRAWSCROLLBAR);
                }
            }
            window.check_name();
        }
        for window in windows.iter().filter_map(|(_, window)| window.upgrade()) {
            let panes = window.panes();
            for mut pane in panes {
                window_pane_send_theme_update(pane.get_mut());
            }
        }
    });
}
unsafe fn server_client_check_window_resize(w_ref: &WindowRef) {
    unsafe {
        let w = w_ref.as_window();
        if !w.flags & WINDOW_RESIZE != 0 {
            return;
        }
        let active = winlinks_into(&w).any(|held| {
            let Some(wl) = held.get() else {
                return false;
            };
            let session = held.session().as_session();
            crate::SessionAttachmentState::session_attached(session) != 0
                && session
                    .curw()
                    .is_some_and(|current| core::ptr::eq(current, wl))
        });
        if !active {
            return;
        }
        log_debug(
            c"%s: resizing window @%u",
            fmt_args![c"server_client_check_window_resize", w.window_id()],
        );
        let dimensions = w.dimensions();
        drop(w);
        let window = w_ref.clone();
        window.resize_with_layout(
            dimensions.pending_size.width,
            dimensions.pending_size.height,
            dimensions.pending_pixels.width as core::ffi::c_int,
            dimensions.pending_pixels.height as core::ffi::c_int,
        );
    }
}
unsafe fn server_client_reset_state(c: &mut client) {
    unsafe {
        let Some(session) = c.attached_session() else {
            return;
        };
        let Some(window) = session.current_window() else {
            return;
        };
        let selected_pane = server_client_get_pane(c);
        let mut s: Option<ScreenModeState> = None;
        let mut status_screen = false;
        let oo = session.options();
        let mut mode: core::ffi::c_int = 0 as core::ffi::c_int;
        let mut cursor: core::ffi::c_int;

        let mut cx: u_int = 0 as u_int;
        let mut cy: u_int = 0 as u_int;
        let ox: u_int;
        let oy: u_int;
        let sx: u_int;
        let sy: u_int;
        let mut n: u_int;
        let mut ranges = visible_ranges::default();
        if c.flags & (CLIENT_CONTROL | CLIENT_SUSPENDED) as uint64_t != 0 {
            return;
        }
        let flags: core::ffi::c_int = c.tty.flags & TTY_BLOCK;
        c.tty.flags &= !TTY_BLOCK;
        if (*c).overlay().is_some() {
            if (*c).overlay().has_mode() {
                let overlay = (*c).overlay();
                let data = (*c).current_overlay_data();
                let (state, x, y) = overlay.mode(data);
                s = Some(state);
                (cx, cy) = (x, y);
            }
        } else if c.prompt_string.is_none()
            && let Some(wp) = selected_pane.as_ref().and_then(|pane| pane.get())
        {
            s = Some(wp.screen_ref().mode_state());
        } else {
            mode = c.status.active_mode();
            status_screen = true;
        }
        if let Some(state) = s {
            mode = state.mode;
        }
        if log_get_level() != 0 as core::ffi::c_int {
            log_debug(
                c"%s: client %s mode %s",
                fmt_args![
                    c"server_client_reset_state".as_ptr(),
                    c.name.as_deref(),
                    screen_mode_to_string(mode).as_c_str()
                ],
            );
        }
        tty_region_off(&mut c.tty);
        tty_margin_off(&mut c.tty);
        if c.prompt_string.is_some() {
            n = (oo).number(c"status-position") as u_int;
            if n == 0 as u_int {
                cy = status_prompt_line_at(&*c);
            } else {
                n = status_line_size(&*c).wrapping_sub(status_prompt_line_at(&*c));
                if n <= c.tty.sy {
                    cy = c.tty.sy.wrapping_sub(n);
                } else {
                    cy = c.tty.sy.wrapping_sub(1 as u_int);
                }
            }
            cx = c.prompt_cursor as u_int;
        } else if c.overlay().is_none()
            && let Some(wp) = selected_pane.as_ref().and_then(|pane| pane.get())
        {
            cursor = 0 as core::ffi::c_int;
            {
                let (window_bigger, off_x, off_y, off_sx, off_sy) = tty_window_offset(&c.tty);
                (ox, oy, sx, sy) = (off_x, off_y, off_sx, off_sy);
                window_bigger
            };
            let (screen_cx, screen_cy) = s.expect("the pane has a screen mode").cursor;
            if wp.geometry().xoff + screen_cx as core::ffi::c_int >= ox as core::ffi::c_int
                && wp.geometry().xoff + screen_cx as core::ffi::c_int
                    <= ox as core::ffi::c_int + sx as core::ffi::c_int
                && wp.geometry().yoff + screen_cy as core::ffi::c_int >= oy as core::ffi::c_int
                && wp.geometry().yoff + screen_cy as core::ffi::c_int
                    <= oy as core::ffi::c_int + sy as core::ffi::c_int
            {
                cursor = 1 as core::ffi::c_int;
                cx = (wp.geometry().xoff + screen_cx as core::ffi::c_int - ox as core::ffi::c_int)
                    as u_int;
                cy = (wp.geometry().yoff + screen_cy as core::ffi::c_int - oy as core::ffi::c_int)
                    as u_int;
                ranges.set_visible_ranges(
                    Some(wp),
                    cx as core::ffi::c_int,
                    cy as core::ffi::c_int,
                    1 as u_int,
                );
                if !screen_redraw_is_visible(Some(&ranges), cx) {
                    cursor = 0 as core::ffi::c_int;
                }
                if status_at_line(&*c) == 0 as core::ffi::c_int {
                    cy = cy.wrapping_add(status_line_size(&*c));
                }
            }
            if cursor == 0 {
                mode &= !MODE_CURSOR;
            }
        } else if !(*c).overlay().has_mode() || s.is_none() {
            mode &= !MODE_CURSOR;
        }
        log_debug(
            c"%s: cursor to %u,%u",
            fmt_args![c"server_client_reset_state".as_ptr(), cx, cy],
        );
        tty_cursor(&mut c.tty, cx, cy);
        if (oo).number(c"mouse") != 0 {
            if (*c).overlay().is_none() {
                mode &= !ALL_MOUSE_MODES;
                if window.as_window().panes.iter().any(|pane| {
                    pane.get()
                        .is_some_and(|pane| pane.screen_ref().mode() & MODE_MOUSE_ALL != 0)
                }) {
                    mode |= MODE_MOUSE_ALL;
                }
            }
            if (oo).number(c"focus-follows-mouse") != 0 {
                mode |= MODE_MOUSE_ALL;
            } else if !mode & MODE_MOUSE_ALL != 0 {
                mode |= MODE_MOUSE_BUTTON;
            }
        }
        if (*c).overlay().is_none() && c.prompt_string.is_some() {
            mode &= !MODE_BRACKETPASTE;
        }
        if status_screen {
            let tty = &mut c.tty;
            c.status
                .with_active(|screen| tty_update_mode(tty, mode, Some(screen.mode_state())));
        } else {
            tty_update_mode(&mut c.tty, mode, s);
        }
        tty_reset(&mut c.tty);
        tty_sync_end(&mut c.tty);
        c.tty.flags |= flags;
    }
}
fn server_client_repeat_timer(c: &mut client) {
    {
        if c.flags & CLIENT_REPEAT as uint64_t != 0 {
            server_client_set_key_table(c, None);
            c.flags &= !CLIENT_REPEAT as uint64_t;
            server_status_client(c);
        }
    }
}
unsafe fn server_client_click_timer(c: &mut client) {
    unsafe {
        log_debug(c"click timer expired", fmt_args![]);
        if c.flags & CLIENT_TRIPLECLICK as uint64_t != 0 {
            let event = Box::new(key_event {
                key: KEYC_DOUBLECLICK as core::ffi::c_ulong as key_code,
                m: c.click_event,
                buf: Vec::new(),
            });
            server_client_handle_key(c, event);
        }
        c.flags &= !(CLIENT_DOUBLECLICK | CLIENT_TRIPLECLICK) as uint64_t;
    }
}
unsafe fn server_client_check_exit(c: &mut client) {
    unsafe {
        if c.flags & (CLIENT_DEAD | CLIENT_EXITED) as uint64_t != 0 {
            return;
        }
        if !c.flags & CLIENT_EXIT as uint64_t != 0 {
            return;
        }
        if c.flags & CLIENT_CONTROL as uint64_t != 0 {
            control_discard(c);
            if control_all_done(c) == 0 {
                return;
            }
        }
        for cf in c.files.values() {
            if cf.borrow().buffer.as_ref().len() != 0 as size_t {
                return;
            }
        }
        c.flags |= CLIENT_EXITED as uint64_t;
        match c.exit_type {
            CLIENT_EXIT_RETURN => {
                let mut data = c.retval.to_ne_bytes().to_vec();
                if let Some(message) = &c.exit_message {
                    data.extend_from_slice(message.to_bytes_with_nul());
                }
                (c.peer_handle()).send(MSG_EXIT, -(1 as core::ffi::c_int), &data);
            }
            CLIENT_EXIT_SHUTDOWN => {
                (c.peer_handle()).send(MSG_SHUTDOWN, -(1 as core::ffi::c_int), &[]);
            }
            CLIENT_EXIT_DETACH => {
                (c.peer_handle()).send(
                    c.exit_msgtype,
                    -(1 as core::ffi::c_int),
                    c.exit_session
                        .as_deref()
                        .expect("a detached client retains its session name")
                        .to_bytes_with_nul(),
                );
            }
            _ => {}
        }
        c.exit_session = None;
        c.exit_message = None;
    }
}
fn server_client_redraw_timer() {
    {
        log_debug(c"redraw timer fired", fmt_args![]);
    }
}
unsafe fn server_client_check_modes(c: &client) {
    unsafe {
        if c.flags & (CLIENT_CONTROL | CLIENT_SUSPENDED) as uint64_t != 0
            || c.flags & CLIENT_REDRAWSTATUS as uint64_t == 0
        {
            return;
        }
        let Some(session) = c.attached_session() else {
            return;
        };
        let Some(window) = session.current_window() else {
            return;
        };
        let panes = window.panes();
        for pane in panes {
            let target = pane
                .get()
                .and_then(crate::window::window_pane_current_mode)
                .and_then(window_mode_entry::update_target);
            if let Some(target) = target {
                target.dispatch();
            }
        }
    }
}
unsafe fn server_client_check_redraw(c: &mut client) {
    unsafe {
        let Some(session) = c.attached_session() else {
            return;
        };
        let Some(window) = session.current_window() else {
            return;
        };
        let mut needed: core::ffi::c_int;

        let mode: core::ffi::c_int = c.tty.mode;
        let mut client_flags: uint64_t = 0 as uint64_t;
        let mut redraw_pane: core::ffi::c_int;
        let mut redraw_scrollbar_only: core::ffi::c_int;
        let mut bit: u_int = 0 as u_int;
        let tv = timeval::from_usecs(1000 as __suseconds_t);
        const client_flags_timer: crate::server_state::Value<TimerHandle> =
            crate::server_state::Value::new(|state| &state.client_flags_timer);
        let left: size_t;
        if c.flags & (CLIENT_CONTROL | CLIENT_SUSPENDED) as uint64_t != 0 {
            return;
        }
        if c.flags as core::ffi::c_ulonglong & CLIENT_ALLREDRAWFLAGS != 0 {
            log_debug(
                c"%s: redraw%s%s%s%s%s%s",
                fmt_args![
                    c.name.as_deref(),
                    if c.flags & CLIENT_REDRAWWINDOW as uint64_t != 0 {
                        c" window".as_ptr()
                    } else {
                        c"".as_ptr()
                    },
                    if c.flags & CLIENT_REDRAWSTATUS as uint64_t != 0 {
                        c" status".as_ptr()
                    } else {
                        c"".as_ptr()
                    },
                    if c.flags & CLIENT_REDRAWBORDERS as uint64_t != 0 {
                        c" borders".as_ptr()
                    } else {
                        c"".as_ptr()
                    },
                    if c.flags & CLIENT_REDRAWOVERLAY as uint64_t != 0 {
                        c" overlay".as_ptr()
                    } else {
                        c"".as_ptr()
                    },
                    if c.flags & CLIENT_REDRAWPANES as uint64_t != 0 {
                        c" panes".as_ptr()
                    } else {
                        c"".as_ptr()
                    },
                    if c.flags as core::ffi::c_ulonglong & CLIENT_REDRAWSCROLLBARS != 0 {
                        c" scrollbars".as_ptr()
                    } else {
                        c"".as_ptr()
                    }
                ],
            );
        }
        needed = 0 as core::ffi::c_int;
        if c.flags as core::ffi::c_ulonglong & CLIENT_ALLREDRAWFLAGS != 0 {
            needed = 1 as core::ffi::c_int;
        } else {
            for pane in window.as_window().panes.iter() {
                let Some(wp) = pane.get() else { continue };
                if *wp.flags() & PANE_REDRAW != 0 {
                    needed = 1 as core::ffi::c_int;
                    client_flags |= CLIENT_REDRAWPANES as uint64_t;
                    break;
                } else {
                    if *wp.flags() & PANE_REDRAWSCROLLBAR != 0 {
                        needed = 1 as core::ffi::c_int;
                        client_flags = (client_flags as core::ffi::c_ulonglong
                            | CLIENT_REDRAWSCROLLBARS)
                            as uint64_t;
                    }
                }
            }
        }
        if needed != 0 && {
            left = c.tty.out.as_ref().unwrap().len();
            left != 0 as size_t
        } {
            log_debug(
                c"%s: redraw deferred (%zu left)",
                fmt_args![c.name.as_deref(), left],
            );
            if !client_flags_timer.get().is_set() {
                client_flags_timer.with_mut(|current| {
                    current.set_callback(move || {
                        server_client_redraw_timer();
                    })
                });
            }
            if !client_flags_timer.get().is_armed() {
                log_debug(c"redraw timer started", fmt_args![]);
                client_flags_timer.with_mut(|current| current.arm(tv));
            }
            if !c.flags & CLIENT_REDRAWWINDOW as uint64_t != 0 {
                for pane in window.as_window().panes.iter() {
                    let Some(wp) = pane.get() else { continue };
                    if *wp.flags() & 0x1 as core::ffi::c_int != 0 {
                        log_debug(
                            c"%s: pane %%%u needs redraw",
                            fmt_args![c.name.as_deref(), wp.pane_id()],
                        );
                        c.redraw_panes |= ((1 as core::ffi::c_int) << bit) as uint64_t;
                    } else if *wp.flags() & PANE_REDRAWSCROLLBAR != 0 {
                        log_debug(
                            c"%s: pane %%%u scrollbar needs redraw",
                            fmt_args![c.name.as_deref(), wp.pane_id()],
                        );
                        c.redraw_scrollbars |= ((1 as core::ffi::c_int) << bit) as uint64_t;
                    }
                    bit = bit.wrapping_add(1);
                    if bit == 64 as u_int {
                        client_flags = (client_flags as core::ffi::c_ulonglong
                            & !(CLIENT_REDRAWPANES as core::ffi::c_ulonglong
                                | CLIENT_REDRAWSCROLLBARS))
                            as uint64_t;
                        client_flags |= CLIENT_REDRAWWINDOW as uint64_t;
                        break;
                    }
                }
                if c.redraw_panes != 0 as uint64_t {
                    c.flags |= CLIENT_REDRAWPANES as uint64_t;
                }
                if c.redraw_scrollbars != 0 as uint64_t {
                    c.flags =
                        (c.flags as core::ffi::c_ulonglong | CLIENT_REDRAWSCROLLBARS) as uint64_t;
                }
            }
            c.flags |= client_flags;
            return;
        } else if needed != 0 {
            log_debug(c"%s: redraw needed", fmt_args![c.name.as_deref()]);
        }
        let tty_flags: core::ffi::c_int = c.tty.flags & (TTY_BLOCK | TTY_FREEZE | TTY_NOCURSOR);
        c.tty.flags = c.tty.flags & !(TTY_BLOCK | TTY_FREEZE) | TTY_NOCURSOR;
        if !c.flags & CLIENT_REDRAWWINDOW as uint64_t != 0 {
            let panes = window.panes();
            for mut pane in panes {
                let Some(wp) = pane.get() else { continue };
                redraw_pane = 0 as core::ffi::c_int;
                redraw_scrollbar_only = 0 as core::ffi::c_int;
                if *wp.flags() & PANE_REDRAW != 0 {
                    redraw_pane = 1 as core::ffi::c_int;
                } else if c.flags & CLIENT_REDRAWPANES as uint64_t != 0 {
                    if c.redraw_panes & ((1 as core::ffi::c_int) << bit) as uint64_t != 0 {
                        redraw_pane = 1 as core::ffi::c_int;
                    }
                } else if c.flags as core::ffi::c_ulonglong & CLIENT_REDRAWSCROLLBARS != 0
                    && c.redraw_scrollbars & ((1 as core::ffi::c_int) << bit) as uint64_t != 0
                {
                    redraw_scrollbar_only = 1 as core::ffi::c_int;
                }
                bit = bit.wrapping_add(1);
                if !(redraw_pane == 0 && redraw_scrollbar_only == 0) {
                    if redraw_scrollbar_only != 0 {
                        log_debug(
                            c"%s: redrawing (scrollbar only) pane %%%u",
                            fmt_args![c"server_client_check_redraw", wp.pane_id()],
                        );
                    } else {
                        log_debug(
                            c"%s: redrawing pane %%%u",
                            fmt_args![c"server_client_check_redraw", wp.pane_id()],
                        );
                    }
                    screen_redraw_pane(c, &mut pane, redraw_scrollbar_only);
                }
            }
            c.redraw_panes = 0 as uint64_t;
            c.redraw_scrollbars = 0 as uint64_t;
            c.flags = (c.flags as core::ffi::c_ulonglong
                & !(CLIENT_REDRAWPANES as core::ffi::c_ulonglong | CLIENT_REDRAWSCROLLBARS))
                as uint64_t;
        }
        if c.flags as core::ffi::c_ulonglong & CLIENT_ALLREDRAWFLAGS != 0 {
            if (session.options()).number(c"set-titles") != 0 {
                server_client_set_title(&mut *c);
                server_client_set_path(&mut *c);
            }
            server_client_set_progress_bar(&mut *c);
            screen_redraw_screen(c);
        }
        c.tty.flags = c.tty.flags & !TTY_NOCURSOR | tty_flags & TTY_NOCURSOR;
        tty_update_mode(&mut c.tty, mode, None);
        c.tty.flags = c.tty.flags & !(TTY_BLOCK | TTY_FREEZE | TTY_NOCURSOR) | tty_flags;
        c.flags = (c.flags as core::ffi::c_ulonglong
            & !(CLIENT_ALLREDRAWFLAGS | CLIENT_STATUSFORCE as core::ffi::c_ulonglong))
            as uint64_t;
        if needed != 0 {
            c.redraw = c.tty.out.as_ref().unwrap().len();
            log_debug(
                c"%s: redraw added %zu bytes",
                fmt_args![c.name.as_deref(), c.redraw],
            );
        }
    }
}
unsafe fn server_client_set_title(c: &mut client) {
    unsafe {
        let Some(session) = c.attached_session() else {
            return;
        };
        let template = session.options().string_ref(c"set-titles-string");
        let mut ft = format_create(Some(c), None, FORMAT_NONE, 0 as core::ffi::c_int);
        format_defaults(
            &mut ft,
            Some(c),
            None,
            None,
            None::<&dyn crate::WindowPane>,
        );
        let title = format_expand_time(&mut ft, &template);
        if c.title.as_deref() != Some(title.as_c_str()) {
            c.title = Some(title.clone());
            tty_set_title(&mut c.tty, &title);
        }
    }
}
fn server_client_session_pane(c: &client) -> Option<RustWindowPaneWeak> {
    {
        let session = c.attached_session()?;
        let window = session.current_window()?;
        window.active_pane()
    }
}
unsafe fn server_client_set_path(c: &mut client) {
    unsafe {
        let Some(pane) = server_client_session_pane(c) else {
            return;
        };
        let Some(pane) = pane.get() else {
            return;
        };
        let path = pane.base().path().unwrap_or(c"");
        if c.path.as_deref() != Some(path) {
            let path = path.to_owned();
            c.path = Some(path.clone());
            tty_set_path(&mut c.tty, &path);
        }
    }
}
unsafe fn server_client_set_progress_bar(c: &mut client) {
    unsafe {
        let Some(pane) = server_client_session_pane(c) else {
            return;
        };
        let Some(pane) = pane.get() else {
            return;
        };
        let progress = pane.base().progress_bar();
        if progress.state == c.progress_bar.state && progress.progress == c.progress_bar.progress {
            return;
        }
        c.progress_bar = progress;
        tty_set_progress_bar(&mut c.tty, &progress);
    }
}

fn server_client_read_only(item: &CmdqItemRef) -> cmd_retval {
    unsafe {
        let item = item.read();
        item.error(c"client is read-only", fmt_args![]);
        CMD_RETURN_ERROR
    }
}
fn server_client_default_command(item: &CmdqItemRef) -> cmd_retval {
    unsafe {
        let mut client = item.client().expect("default command without a client");
        let c = client.as_client_mut();
        let cmdlist = (global_options
            .get()
            .as_ref()
            .expect("global options are initialized"))
        .command(c"default-client-command")
        .unwrap();
        let queued =
            if c.flags & CLIENT_READONLY as uint64_t != 0 && cmdlist.all_have(CMD_READONLY) == 0 {
                CmdqItemRef::callback_items(c"server_client_read_only", server_client_read_only)
            } else {
                cmdlist.queue_items(None)
            };
        item.insert_after(queued);
        CMD_RETURN_NORMAL
    }
}
unsafe fn server_client_command_done(item: &cmdq_item) -> cmd_retval {
    unsafe {
        let mut client = item.client().expect("client command without a client");
        let c = client.as_client_mut();
        if !c.flags & CLIENT_ATTACHED as uint64_t != 0 {
            c.flags |= CLIENT_EXIT as uint64_t;
        } else if !c.flags & CLIENT_EXIT as uint64_t != 0 {
            if c.flags & CLIENT_CONTROL as uint64_t != 0 {
                control_ready(c);
            }
            tty_send_requests(&mut c.tty);
        }
        CMD_RETURN_NORMAL
    }
}
fn server_client_command_done_callback(item: &CmdqItemRef) -> cmd_retval {
    item.with_item(|item| unsafe { server_client_command_done(item) })
}
unsafe fn server_client_dispatch_command(c: &mut client, imsg: &mut imsg) -> core::ffi::c_int {
    unsafe {
        let current_block: u64;
        let mut cause = None;
        let mut queued = crate::cmd::cmdq_items::new();
        if c.flags & CLIENT_EXIT as uint64_t != 0 {
            return 0 as core::ffi::c_int;
        }
        let payload = imsg.imsg_message_data_mut();
        if payload.len() < size_of::<msg_command>() {
            return -1;
        }
        let (header, packed) = payload.split_at_mut(size_of::<msg_command>());
        let argc = core::ffi::c_int::from_ne_bytes(
            header.try_into().expect("command header size checked"),
        );
        if !packed.is_empty() && packed.last() != Some(&0) {
            return -1;
        }
        match RustCommandTextCodec.unpack(packed, argc) {
            None => {
                cause = Some(c"command too long".to_owned());
            }
            Some(argv) => {
                if argv.is_empty() {
                    queued = CmdqItemRef::callback_items(
                        c"server_client_default_command",
                        server_client_default_command,
                    );
                    current_block = 13472856163611868459;
                } else {
                    let values = args_from_vector(&argv);
                    let mut pr = cmd_parse_from_arguments(&values, None);
                    match pr.status {
                        CMD_PARSE_ERROR => {
                            cause = pr.error.take();
                            current_block = 3291689279699309301;
                        }
                        _ => {
                            let cmdlist = pr.cmdlist.take().unwrap();
                            if c.flags & CLIENT_READONLY as uint64_t != 0
                                && cmdlist.all_have(CMD_READONLY) == 0
                            {
                                queued = CmdqItemRef::callback_items(
                                    c"server_client_read_only",
                                    server_client_read_only,
                                );
                            } else {
                                queued = cmdlist.queue_items(None);
                            }
                            current_block = 13472856163611868459;
                        }
                    }
                }
                match current_block {
                    3291689279699309301 => {}
                    _ => {
                        cmdq_append(crate::server::client_ref_of(c).as_ref(), queued);
                        cmdq_append(
                            crate::server::client_ref_of(c).as_ref(),
                            CmdqItemRef::callback_items(
                                c"server_client_command_done",
                                server_client_command_done_callback,
                            ),
                        );
                        return 0 as core::ffi::c_int;
                    }
                }
            }
        }
        if let Some(cause) = cause.as_ref() {
            cmdq_append(
                crate::server::client_ref_of(c).as_ref(),
                CmdqItemRef::error_items(cause),
            );
        }
        c.flags |= CLIENT_EXIT as uint64_t;
        0 as core::ffi::c_int
    }
}
unsafe fn server_client_dispatch_identify(c: &mut client, imsg: &mut imsg) -> core::ffi::c_int {
    unsafe {
        let flags: core::ffi::c_int;
        let feat: core::ffi::c_int;
        let longflags: uint64_t;
        if c.flags & CLIENT_IDENTIFIED as uint64_t != 0 {
            return -(1 as core::ffi::c_int);
        }
        let data = imsg.imsg_message_data();
        let datalen = data.len();
        match imsg.hdr.type_0 {
            MSG_IDENTIFY_FEATURES => {
                if datalen != size_of::<core::ffi::c_int>() {
                    return -(1 as core::ffi::c_int);
                }
                feat =
                    core::ffi::c_int::from_ne_bytes(data.try_into().expect("integer size checked"));
                c.term_features |= feat;
                log_debug(
                    c"client %p IDENTIFY_FEATURES %s",
                    fmt_args![&raw const *c, RustTerminalFeatureSet.names(feat).as_c_str()],
                );
            }
            MSG_IDENTIFY_FLAGS => {
                if datalen != size_of::<core::ffi::c_int>() {
                    return -(1 as core::ffi::c_int);
                }
                flags =
                    core::ffi::c_int::from_ne_bytes(data.try_into().expect("integer size checked"));
                c.flags |= flags as uint64_t;
                log_debug(
                    c"client %p IDENTIFY_FLAGS %#x",
                    fmt_args![&raw const *c, flags],
                );
            }
            MSG_IDENTIFY_LONGFLAGS => {
                if datalen != size_of::<uint64_t>() {
                    return -(1 as core::ffi::c_int);
                }
                longflags = uint64_t::from_ne_bytes(data.try_into().expect("flags size checked"));
                c.flags |= longflags;
                log_debug(
                    c"client %p IDENTIFY_LONGFLAGS %#llx",
                    fmt_args![&raw const *c, longflags as core::ffi::c_ulonglong],
                );
            }
            MSG_IDENTIFY_TERM => {
                if data.last() != Some(&0) {
                    return -1;
                }
                let data = CStr::from_bytes_until_nul(data).expect("terminated identify field");
                c.term_name = Some(data.to_owned());
                log_debug(
                    c"client %p IDENTIFY_TERM %s",
                    fmt_args![&raw const *c, data],
                );
            }
            MSG_IDENTIFY_TERMINFO => {
                if data.last() != Some(&0) {
                    return -1;
                }
                let data = CStr::from_bytes_until_nul(data).expect("terminated identify field");
                c.term_caps.push(data.to_owned());
                log_debug(
                    c"client %p IDENTIFY_TERMINFO %s",
                    fmt_args![&raw const *c, data],
                );
            }
            MSG_IDENTIFY_TTYNAME => {
                if data.last() != Some(&0) {
                    return -1;
                }
                let data = CStr::from_bytes_until_nul(data).expect("terminated identify field");
                c.ttyname = Some(data.to_owned());
                log_debug(
                    c"client %p IDENTIFY_TTYNAME %s",
                    fmt_args![&raw const *c, data],
                );
            }
            MSG_IDENTIFY_CWD => {
                if data.last() != Some(&0) {
                    return -1;
                }
                let data = CStr::from_bytes_until_nul(data).expect("terminated identify field");
                if access(data.as_ptr(), X_OK) == 0 as core::ffi::c_int {
                    c.cwd = Some(data.to_owned());
                } else {
                    c.cwd = Some(find_home().unwrap_or_else(|| c"/".to_owned()).to_owned());
                }
                log_debug(c"client %p IDENTIFY_CWD %s", fmt_args![&raw const *c, data]);
            }
            MSG_IDENTIFY_STDIN => {
                if datalen != 0 as size_t {
                    return -(1 as core::ffi::c_int);
                }
                c.fd = imsg_get_fd(&mut *imsg);
                log_debug(
                    c"client %p IDENTIFY_STDIN %d",
                    fmt_args![&raw const *c, c.fd],
                );
            }
            MSG_IDENTIFY_STDOUT => {
                if datalen != 0 as size_t {
                    return -(1 as core::ffi::c_int);
                }
                c.out_fd = imsg_get_fd(&mut *imsg);
                log_debug(
                    c"client %p IDENTIFY_STDOUT %d",
                    fmt_args![&raw const *c, c.out_fd],
                );
            }
            MSG_IDENTIFY_ENVIRON => {
                if data.last() != Some(&0) {
                    return -1;
                }
                let data = CStr::from_bytes_until_nul(data).expect("terminated identify field");
                if data.to_bytes().contains(&b'=') {
                    (*c).environ_mut().put(data, 0);
                }
                log_debug(
                    c"client %p IDENTIFY_ENVIRON %s",
                    fmt_args![&raw const *c, data],
                );
            }
            MSG_IDENTIFY_CLIENTPID => {
                if datalen != size_of::<pid_t>() {
                    return -(1 as core::ffi::c_int);
                }
                c.pid = pid_t::from_ne_bytes(data.try_into().expect("pid size checked"));
                log_debug(
                    c"client %p IDENTIFY_CLIENTPID %ld",
                    fmt_args![&raw const *c, c.pid as core::ffi::c_long],
                );
            }
            _ => {}
        }
        if imsg.hdr.type_0 != MSG_IDENTIFY_DONE as core::ffi::c_int as uint32_t {
            return 0 as core::ffi::c_int;
        }
        c.flags |= CLIENT_IDENTIFIED as uint64_t;
        if c.term_name
            .as_ref()
            .is_none_or(|name| name.as_bytes().is_empty())
        {
            c.term_name = Some(c"unknown".to_owned());
        }
        if c.ttyname
            .as_ref()
            .is_some_and(|ttyname| !ttyname.as_bytes().is_empty())
        {
            c.name = c.ttyname.clone();
        } else {
            c.name = Some(xasprintf(
                c"client-%ld",
                fmt_args![c.pid as core::ffi::c_long],
            ));
        }
        log_debug(
            c"client %p name is %s",
            fmt_args![&raw const *c, c.name.as_deref()],
        );
        if c.flags & CLIENT_CONTROL as uint64_t != 0 {
            control_start(&mut *c);
        } else if c.fd != -(1 as core::ffi::c_int) {
            if tty_init(c) != 0 as core::ffi::c_int {
                close(c.fd);
                c.fd = -(1 as core::ffi::c_int);
            } else {
                tty_resize(&mut c.tty);
                c.flags |= CLIENT_TERMINAL as uint64_t;
            }
            if c.out_fd != -(1 as core::ffi::c_int) {
                close(c.out_fd);
            }
            c.out_fd = -(1 as core::ffi::c_int);
        }
        if c.flags as core::ffi::c_ulonglong & (CLIENT_BRACKETPASTING | CLIENT_ASSUMEPASTING) != 0
            && current_time.get() - c.paste_time > CLIENT_PASTE_TIME_LIMIT as time_t
        {
            log_debug(
                c"%s: paste time limit exceeded",
                fmt_args![c.name.as_deref()],
            );
            c.flags = (c.flags as core::ffi::c_ulonglong
                & !(CLIENT_BRACKETPASTING | CLIENT_ASSUMEPASTING))
                as uint64_t;
        }
        if !c.flags & CLIENT_EXIT as uint64_t != 0
            && !configuration_finished()
            && with_clients(|clients| {
                clients
                    .first()
                    .is_some_and(|first| core::ptr::eq(first.as_ptr(), c))
            })
        {
            start_cfg();
        }
        0 as core::ffi::c_int
    }
}
unsafe fn server_client_dispatch_shell(c: &mut client) -> core::ffi::c_int {
    unsafe {
        let configured = global_s_options
            .get()
            .expect("global options are initialized")
            .string_ref(c"default-shell");
        let shell = if checkshell(Some(&configured)) == 0 {
            _PATH_BSHELL
        } else {
            &configured
        };
        (c.peer_handle()).send(MSG_SHELL, -1, shell.to_bytes_with_nul());
        (c.peer_handle()).mark_bad();
        0
    }
}
pub unsafe fn server_client_get_cwd(c: Option<&client>, s: Option<&session>) -> CString {
    unsafe {
        let loading = cfg_client();
        if !configuration_finished()
            && let Some(loading) = loading
            && let Some(cwd) = loading.as_client().cwd.as_ref()
        {
            return cwd.clone();
        }
        if let Some(c) = c
            && c.attached_session().is_none()
            && let Some(cwd) = c.cwd.as_ref()
        {
            return cwd.clone();
        }
        // the session asked for, then the one the client is on
        let attached = c.and_then(client::attached_session);
        let from = s.or_else(|| attached.as_ref().map(|session| session.as_session()));
        if let Some(s) = from
            && let Some(cwd) = crate::SessionDirectoryState::session_directory(s)
        {
            return cwd.to_owned();
        }
        find_home().unwrap_or_else(|| c"/".to_owned()).to_owned()
    }
}
unsafe fn server_client_control_flags(c: &mut client, next: &CStr) -> uint64_t {
    unsafe {
        if strcmp(next.as_ptr(), c"pause-after".as_ptr()) == 0 as core::ffi::c_int {
            c.pause_age = 0 as u_int;
            return 0x100000000 as uint64_t;
        }
        if sscanf(
            next.as_ptr(),
            c"pause-after=%u".as_ptr(),
            &raw mut c.pause_age,
        ) == 1 as core::ffi::c_int
        {
            c.pause_age = c.pause_age.wrapping_mul(1000 as u_int);
            return 0x100000000 as uint64_t;
        }
        if strcmp(next.as_ptr(), c"no-output".as_ptr()) == 0 as core::ffi::c_int {
            return 0x4000000 as uint64_t;
        }
        if strcmp(next.as_ptr(), c"wait-exit".as_ptr()) == 0 as core::ffi::c_int {
            return 0x200000000 as uint64_t;
        }
        0 as uint64_t
    }
}
pub unsafe fn server_client_set_flags(c: &mut client, flags: &CStr) {
    unsafe {
        let mut flag: uint64_t;
        for next in flags.to_bytes().split(|&byte| byte == b',') {
            let not = (next.first() == Some(&b'!')) as core::ffi::c_int;
            let next = if not != 0 { &next[1..] } else { next };
            let next = CString::new(next).expect("a C string has no interior NUL");
            if c.flags & CLIENT_CONTROL as uint64_t != 0 {
                flag = server_client_control_flags(c, &next);
            } else {
                flag = 0 as uint64_t;
            }
            if strcmp(next.as_ptr(), c"read-only".as_ptr()) == 0 as core::ffi::c_int {
                flag = CLIENT_READONLY as uint64_t;
            } else if strcmp(next.as_ptr(), c"ignore-size".as_ptr()) == 0 as core::ffi::c_int {
                flag = CLIENT_IGNORESIZE as uint64_t;
            } else if strcmp(next.as_ptr(), c"active-pane".as_ptr()) == 0 as core::ffi::c_int {
                flag = CLIENT_ACTIVEPANE as uint64_t;
            } else if strcmp(next.as_ptr(), c"no-detach-on-destroy".as_ptr())
                == 0 as core::ffi::c_int
            {
                flag = CLIENT_NO_DETACH_ON_DESTROY as uint64_t;
            }
            if flag == 0 as uint64_t {
                continue;
            }
            log_debug(
                c"client %s set flag %s",
                fmt_args![c.name.as_deref(), next.as_ptr()],
            );
            if not != 0 {
                if c.flags & CLIENT_READONLY as uint64_t != 0 {
                    flag &= !CLIENT_READONLY as uint64_t;
                }
                c.flags &= !flag;
            } else {
                c.flags |= flag;
            }
            if flag == CLIENT_CONTROL_NOOUTPUT as uint64_t {
                control_reset_offsets(&mut *c);
            }
        }
        let flags = c.flags.to_ne_bytes();
        ((*c).peer_handle()).send(MSG_FLAGS, -(1 as core::ffi::c_int), &flags);
    }
}
/// The flags a client carries, comma-separated, as the caller's own string.
pub fn server_client_get_flags(c: &mut client) -> CString {
    let mut names: Vec<&CStr> = Vec::new();
    if c.flags & CLIENT_ATTACHED as uint64_t != 0 {
        names.push(c"attached");
    }
    if c.flags & CLIENT_FOCUSED as uint64_t != 0 {
        names.push(c"focused");
    }
    if c.flags & CLIENT_CONTROL as uint64_t != 0 {
        names.push(c"control-mode");
    }
    if c.flags & CLIENT_IGNORESIZE as uint64_t != 0 {
        names.push(c"ignore-size");
    }
    if c.flags as core::ffi::c_ulonglong & CLIENT_NO_DETACH_ON_DESTROY != 0 {
        names.push(c"no-detach-on-destroy");
    }
    if c.flags & CLIENT_CONTROL_NOOUTPUT as uint64_t != 0 {
        names.push(c"no-output");
    }
    if c.flags as core::ffi::c_ulonglong & CLIENT_CONTROL_WAITEXIT != 0 {
        names.push(c"wait-exit");
    }
    let paused;
    if c.flags as core::ffi::c_ulonglong & CLIENT_CONTROL_PAUSEAFTER != 0 {
        paused = format_alloc(
            c"pause-after=%u",
            fmt_args![c.pause_age.wrapping_div(1000 as u_int)],
        );
        names.push(&paused);
    }
    if c.flags & CLIENT_READONLY as uint64_t != 0 {
        names.push(c"read-only");
    }
    if c.flags as core::ffi::c_ulonglong & CLIENT_ACTIVEPANE != 0 {
        names.push(c"active-pane");
    }
    if c.flags & CLIENT_SUSPENDED as uint64_t != 0 {
        names.push(c"suspended");
    }
    if c.flags & CLIENT_UTF8 as uint64_t != 0 {
        names.push(c"UTF-8");
    }
    let joined = names
        .iter()
        .map(|name| name.to_bytes())
        .collect::<Vec<_>>()
        .join(b",".as_slice());
    CString::new(joined).expect("a flag name has no interior NUL")
}
pub fn server_client_get_client_window(c: &mut client, id: u_int) -> Option<&mut client_window> {
    c.windows.get_mut(&id)
}
pub fn server_client_add_client_window(c: &mut client, id: u_int) -> &mut client_window {
    c.windows.entry(id).or_insert(client_window {
        window: id,
        pane: None,
        sx: 0 as u_int,
        sy: 0 as u_int,
    })
}
pub fn server_client_get_pane(c: &client) -> Option<RustWindowPaneWeak> {
    {
        let session = c.attached_session()?;
        let window = session.current_window()?;
        server_client_get_pane_in_window(c, &window)
    }
}

pub(crate) fn server_client_get_pane_in_window(
    c: &client,
    window: &WindowRef,
) -> Option<RustWindowPaneWeak> {
    if c.flags & CLIENT_ACTIVEPANE == 0 {
        return window.active_pane();
    }
    let Some(cw) = c.windows.get(&window.window_id()) else {
        return window.active_pane();
    };
    cw.pane
        .clone()
        .filter(|pane| unsafe { pane.get().is_some() })
}

pub fn server_client_set_pane(c: &mut client, wp: &(impl crate::WindowPane + ?Sized)) {
    {
        let Some(session) = c.attached_session() else {
            return;
        };
        let Some(window) = session.current_window() else {
            return;
        };
        let cw = server_client_add_client_window(c, window.window_id());
        cw.pane = (wp).observation();
        log_debug(
            c"%s pane now %%%u",
            fmt_args![c.name.as_deref(), wp.pane_id()],
        );
    }
}
pub unsafe fn server_client_remove_pane(pane: &(impl crate::WindowPane + ?Sized)) {
    unsafe {
        let Some(window) = pane.window_context() else {
            return;
        };
        let window_id = window.window_id();
        let pane_id = pane.pane_id();
        for mut c in client_walk() {
            let remove = server_client_get_client_window(c.as_client_mut(), window_id)
                .filter(|cw| {
                    cw.pane
                        .as_ref()
                        .is_some_and(|reference| pane.observation().as_ref() == Some(reference))
                })
                .map(|cw| cw.window);
            if let Some(window) = remove {
                c.as_client_mut().windows.remove(&window);
            }
            if c.as_tty().mouse_last_pane == pane_id as core::ffi::c_int {
                c.as_tty_mut().mouse_last_pane = -(1 as core::ffi::c_int);
                c.as_tty_mut().mouse_drag_update = None;
                c.as_tty_mut().mouse_scrolling_flag = 0 as core::ffi::c_int;
            }
        }
    }
}
pub unsafe fn server_client_print(
    c: Option<&mut client>,
    parse: core::ffi::c_int,
    buffer: &mut ByteBuffer,
) {
    unsafe {
        let source = buffer.as_slice();
        let message = if parse == 0 {
            RustUtf8VisModel.encode_utf8(source, VIS_OCTAL | VIS_CSTYLE | VIS_NOSLASH)
        } else {
            let end = source
                .iter()
                .position(|byte| *byte == 0)
                .unwrap_or(source.len());
            CString::new(&source[..end]).expect("the message ends before its first NUL")
        };
        log_debug(
            c"%s: %s",
            fmt_args![c"server_client_print", message.as_c_str()],
        );
        let Some(c) = c else {
            return;
        };
        if c.attached_session().is_none() || c.flags & CLIENT_CONTROL as uint64_t != 0 {
            let sanitized = (c.flags & CLIENT_UTF8 as uint64_t == 0)
                .then(|| RustUtf8VisModel.sanitize(&message));
            let message = sanitized.as_deref().unwrap_or(&message);
            if c.flags & CLIENT_CONTROL as uint64_t != 0 {
                control_write(c, c"%s", fmt_args![message]);
            } else {
                file_print(Some(c), c"%s\n", fmt_args![message]);
            }
            return;
        }
        let Some(mut pane) = server_client_get_pane(c) else {
            return;
        };
        let Some(wp) = pane.get() else {
            return;
        };
        if wp
            .modes()
            .first()
            .is_none_or(|current| current.mode() != WindowMode::View)
            && let Some(wp) = pane.get_mut()
        {
            window_pane_set_mode(wp, None, WindowMode::View, None, None);
        }
        if parse != 0 {
            while let Some(line) = buffer.read_line() {
                if let Some(wp) = pane.get_mut() {
                    window_copy_add(wp, 1, c"%s", fmt_args![line.as_ref()]);
                }
            }
            let remainder = buffer.as_slice();
            if !remainder.is_empty()
                && let Some(wp) = pane.get_mut()
            {
                window_copy_add(
                    wp,
                    1,
                    c"%.*s",
                    fmt_args![remainder.len() as core::ffi::c_int, remainder],
                );
            }
        } else if let Some(wp) = pane.get_mut() {
            window_copy_add(wp, 0, c"%s", fmt_args![message.as_c_str()]);
        }
    }
}
unsafe fn server_client_report_theme(c: &mut client, theme: client_theme) {
    unsafe {
        if theme as core::ffi::c_uint == THEME_LIGHT as core::ffi::c_int as core::ffi::c_uint {
            c.theme = THEME_LIGHT;
            notify_client(c"client-light-theme", Some(c));
        } else {
            c.theme = THEME_DARK;
            notify_client(c"client-dark-theme", Some(c));
        }
        tty_repeat_requests(&mut c.tty, 1 as core::ffi::c_int);
    }
}

#[cfg(test)]
#[path = "../tests/test_server_client.rs"]
mod tests;
use crate::screen::RustScreen;

impl visible_ranges {
    pub fn is_empty(&self) -> bool {
        let r = self;

        !r.ranges
            .iter()
            .take(r.used as usize)
            .any(|range| range.nx != 0)
    }
    pub fn ensure_capacity(&mut self, n: u_int) {
        let r = self;

        if r.ranges.len() >= n as usize {
            return;
        }
        r.ranges.resize(n as usize, visible_range { px: 0, nx: 0 });
    }
    #[allow(clippy::too_many_arguments)]
    pub fn set_overlay_range(
        &mut self,
        x: u_int,
        y: u_int,
        sx: u_int,
        sy: u_int,
        px: u_int,
        py: u_int,
        nx: u_int,
    ) {
        let r = self;

        let mut ox: u_int;

        if py < y || py > y.wrapping_add(sy).wrapping_sub(1 as u_int) {
            r.ensure_capacity(1 as u_int);
            r.ranges[0].px = px;
            r.ranges[0].nx = nx;
            r.used = 1 as u_int;
            return;
        }
        r.ensure_capacity(2 as u_int);
        if px < x {
            r.ranges[0].px = px;
            r.ranges[0].nx = x.wrapping_sub(px);
            if r.ranges[0].nx > nx {
                r.ranges[0].nx = nx;
            }
        } else {
            r.ranges[0].px = 0 as u_int;
            r.ranges[0].nx = 0 as u_int;
        }
        ox = x.wrapping_add(sx);
        if px > ox {
            ox = px;
        }
        let onx: u_int = px.wrapping_add(nx);
        if onx > ox {
            r.ranges[1].px = ox;
            r.ranges[1].nx = onx.wrapping_sub(ox);
        } else {
            r.ranges[1].px = 0 as u_int;
            r.ranges[1].nx = 0 as u_int;
        }
        r.used = 2 as u_int;
    }
}

impl ClientRef {
    pub fn from_fd(fd: core::ffi::c_int) -> ClientRef {
        unsafe {
            setblocking(fd, 0 as core::ffi::c_int);
            let mut fresh = client::default();
            fresh.fd = -(1 as core::ffi::c_int);
            fresh.out_fd = -(1 as core::ffi::c_int);
            fresh.click_wp = -(1 as core::ffi::c_int);
            fresh.theme = THEME_UNKNOWN;
            fresh.environ = Some(new_environment_box());
            fresh.queue = Some(CmdqListRef::empty());
            fresh.tty = tty {
                sx: 80 as u_int,
                sy: 24 as u_int,
                ..tty::default()
            };
            let mut reference = ClientRef::new(fresh);
            let dispatch_client = reference.downgrade();
            let repeat_client = reference.downgrade();
            let click_client = reference.downgrade();
            let c = reference.as_client_mut();
            let dispatch: std::rc::Rc<PeerDispatch> = std::rc::Rc::new(move |imsg| {
                if let Some(client) = dispatch_client.upgrade() {
                    client.dispatch_message(imsg)
                }
            });
            c.peer = Some(PeerRef::from_fd(fd, Some(dispatch)));
            c.creation_time = timeval::now();
            c.activity_time = c.creation_time;
            status_init(&mut *c);
            c.flags |= CLIENT_FOCUSED as uint64_t;
            let table_ref = key_bindings_get_table_ref(c"root", 1 as core::ffi::c_int)
                .expect("root key table creation requested");
            c.keytable = Some(table_ref);
            c.repeat_timer.set_callback(move || {
                if let Some(mut c) = repeat_client.upgrade() {
                    server_client_repeat_timer(c.as_client_mut());
                }
            });
            c.click_timer.set_callback(move || {
                if let Some(mut c) = click_client.upgrade() {
                    server_client_click_timer(c.as_client_mut());
                }
            });
            log_debug(c"new client %p", fmt_args![&raw const *c]);
            with_clients_mut(|clients| clients.push(reference.clone()));
            reference
        }
    }
    pub fn on_lost(self) {
        let mut c = self;

        unsafe {
            if c.flags() & CLIENT_DEAD as uint64_t != 0 {
                return;
            }
            *c.flags_mut() |= CLIENT_DEAD as uint64_t;
            server_client_clear_overlay(c.as_client_mut());
            status_prompt_clear(c.as_client_mut());
            status_message_clear(c.as_client_mut());
            let files = c.as_client().files.values().cloned().collect::<Vec<_>>();
            for cf in files {
                cf.borrow_mut().error = EINTR;
                cf.fire_done();
            }
            drop(core::mem::take(&mut c.as_client_mut().windows));
            with_clients_mut(|clients| clients.retain(|listed| !listed.ptr_eq(&c)));
            log_debug(c"lost client %p", fmt_args![c.as_ptr()]);
            if c.flags() & CLIENT_ATTACHED as uint64_t != 0 {
                server_client_attached_lost(c.as_client_mut());
                notify_client(c"client-detached", Some(c.as_client_mut()));
            }
            if c.flags() & CLIENT_CONTROL as uint64_t != 0 {
                control_stop(c.as_client_mut());
            }
            if c.flags() & CLIENT_TERMINAL as uint64_t != 0 {
                tty_free(c.as_tty_mut());
            }
            c.as_client_mut().ttyname = None;
            c.as_client_mut().term_name = None;
            c.as_client_mut().term_type = None;
            drop(core::mem::take(&mut c.as_client_mut().term_caps));
            status_free(c.as_client_mut());
            c.as_client_mut().status.screen = RustScreen::default();
            input_cancel_requests(c.as_client_mut());
            c.as_client_mut().title = None;
            c.as_client_mut().path = None;
            c.as_client_mut().cwd = None;
            c.as_client_mut().exit_session = None;
            c.as_client_mut().exit_message = None;
            c.as_client_mut().repeat_timer.disarm();
            c.as_client_mut().click_timer.disarm();
            c.as_client_mut().keytable = None;
            c.as_client_mut().message_string = None;
            c.as_client_mut().message_timer.disarm();
            c.as_client_mut().prompt_saved = None;
            c.as_client_mut().prompt_string = None;
            c.as_client_mut().prompt_last = None;
            c.as_client_mut().prompt_buffer = Vec::new();
            format_free_jobs(c.as_client_mut().jobs.take());
            c.as_client_mut().environ = None;
            if let Some(peer) = c.as_client_mut().peer.take() {
                peer.close();
            }
            if c.as_client().out_fd != -(1 as core::ffi::c_int) {
                close(c.as_client().out_fd);
            }
            if c.fd() != -(1 as core::ffi::c_int) {
                close(c.fd());
                c.as_client_mut().fd = -(1 as core::ffi::c_int);
            }
            server_add_accept(0 as core::ffi::c_int);
            recalculate_sizes();
            server_check_unattached();
            server_update_socket();
            reactor::current().defer(move || drop(c));
        }
    }
    fn dispatch_message(self, imsg: Option<&mut imsg>) {
        let mut owner = self;

        unsafe {
            let current_block: u64;

            if owner.flags() & CLIENT_DEAD as uint64_t != 0 {
                return;
            }
            let Some(imsg) = imsg else {
                owner.on_lost();
                return;
            };
            let datalen: ssize_t =
                (imsg.hdr.len as usize).wrapping_sub(IMSG_HEADER_SIZE) as ssize_t;
            match imsg.hdr.type_0 {
                MSG_IDENTIFY_CLIENTPID
                | MSG_IDENTIFY_CWD
                | MSG_IDENTIFY_ENVIRON
                | MSG_IDENTIFY_FEATURES
                | MSG_IDENTIFY_FLAGS
                | MSG_IDENTIFY_LONGFLAGS
                | MSG_IDENTIFY_STDIN
                | MSG_IDENTIFY_STDOUT
                | MSG_IDENTIFY_TERM
                | MSG_IDENTIFY_TERMINFO
                | MSG_IDENTIFY_TTYNAME
                | MSG_IDENTIFY_DONE => {
                    if server_client_dispatch_identify(owner.as_client_mut(), imsg)
                        != 0 as core::ffi::c_int
                    {
                        current_block = 14480369916731894397;
                    } else {
                        current_block = 6174974146017752131;
                    }
                }
                MSG_COMMAND => {
                    if server_client_dispatch_command(owner.as_client_mut(), imsg)
                        != 0 as core::ffi::c_int
                    {
                        current_block = 14480369916731894397;
                    } else {
                        current_block = 6174974146017752131;
                    }
                }
                MSG_RESIZE => {
                    if datalen != 0 as ssize_t {
                        current_block = 14480369916731894397;
                    } else if owner.flags() & CLIENT_CONTROL as uint64_t != 0 {
                        current_block = 6174974146017752131;
                    } else {
                        server_client_update_latest(owner.as_client_mut());
                        tty_resize(owner.as_tty_mut());
                        tty_repeat_requests(owner.as_tty_mut(), 0 as core::ffi::c_int);
                        recalculate_sizes();
                        if !owner.overlay().has_resize() {
                            server_client_clear_overlay(owner.as_client_mut());
                        } else {
                            let overlay = owner.overlay();
                            let data = owner.current_overlay_data();
                            overlay.resize(owner.as_client_mut(), data);
                        }
                        server_redraw_client(owner.as_client_mut());
                        if !owner.attached_session().is_none() {
                            notify_client(c"client-resized", Some(owner.as_client_mut()));
                        }
                        current_block = 6174974146017752131;
                    }
                }
                MSG_EXITING => {
                    if datalen != 0 as ssize_t {
                        current_block = 14480369916731894397;
                    } else {
                        server_client_set_session(owner.as_client_mut(), None);
                        recalculate_sizes();
                        tty_close(owner.as_tty_mut());
                        (owner.peer_handle()).send(MSG_EXITED, -(1 as core::ffi::c_int), &[]);
                        current_block = 6174974146017752131;
                    }
                }
                MSG_WAKEUP | MSG_UNLOCK => {
                    if datalen != 0 as ssize_t {
                        current_block = 14480369916731894397;
                    } else if owner.flags() & CLIENT_SUSPENDED as uint64_t == 0 {
                        current_block = 6174974146017752131;
                    } else {
                        *owner.flags_mut() &= !CLIENT_SUSPENDED as uint64_t;
                        if owner.fd() == -(1 as core::ffi::c_int)
                            || owner.attached_session().is_none()
                        {
                            current_block = 6174974146017752131;
                        } else {
                            let session = owner.attached_session();
                            owner.as_client_mut().activity_time = timeval::now();
                            tty_start_tty(owner.as_tty_mut());
                            server_redraw_client(owner.as_client_mut());
                            recalculate_sizes();
                            if let Some(session) = session {
                                session.update_activity(Some(&owner.as_client().activity_time));
                            }
                            current_block = 6174974146017752131;
                        }
                    }
                }
                MSG_SHELL => {
                    if datalen != 0 as ssize_t
                        || server_client_dispatch_shell(owner.as_client_mut())
                            != 0 as core::ffi::c_int
                    {
                        current_block = 14480369916731894397;
                    } else {
                        current_block = 6174974146017752131;
                    }
                }
                MSG_WRITE_READY => {
                    if owner.handle_file_write_ready(imsg) != 0 as core::ffi::c_int {
                        current_block = 14480369916731894397;
                    } else {
                        current_block = 6174974146017752131;
                    }
                }
                MSG_READ => {
                    if owner.handle_file_read_data(imsg) != 0 as core::ffi::c_int {
                        current_block = 14480369916731894397;
                    } else {
                        current_block = 6174974146017752131;
                    }
                }
                MSG_READ_DONE if owner.handle_file_read_done(imsg) != 0 as core::ffi::c_int => {
                    current_block = 14480369916731894397;
                }
                _ => {
                    current_block = 6174974146017752131;
                }
            }
            match current_block {
                6174974146017752131 => (),
                _ => {
                    log_debug(
                        c"client %p invalid message type %d",
                        fmt_args![owner.as_ptr(), imsg.hdr.type_0],
                    );
                    (owner.peer_handle()).mark_bad();
                }
            }
        }
    }
}

impl ClientRef {
    /// # Safety
    /// Exclude conflicting client access, including reentrant callbacks, during this operation.
    pub(crate) unsafe fn handle_key(&mut self, event: Box<key_event>) -> core::ffi::c_int {
        unsafe { server_client_handle_key(self.as_client_mut(), event) }
    }
}

/// A direction for moving the client's manually selected window viewport.
#[derive(Clone, Copy)]
pub(crate) enum ClientPanDirection {
    Left,
    Right,
    Up,
    Down,
}

impl ClientRef {
    /// Clears the weak pan anchor, recalculates terminal offsets and requests a
    /// full redraw. An unattached client does not require a current window.
    ///
    /// # Safety
    /// Run on the server thread without outstanding client, TTY, session or
    /// pane payload borrows. Offset calculation reads the current attachment.
    pub(crate) unsafe fn reset_window_pan(&mut self) {
        unsafe {
            let client = self.as_client_mut();
            client_set_pan_window(client, None);
            tty_update_client_offset(client);
            server_redraw_client(client);
        }
    }

    /// Moves a retained window's viewport, keeping its anchor weak. A changed
    /// anchor starts at the cached terminal offsets. Left/up clamp at zero;
    /// right/down preserve unsigned wrapping before clamping to the cached
    /// visible extent. Terminal offset recalculation precedes the full redraw.
    /// No hooks or command callbacks run, and no payload borrow escapes.
    ///
    /// # Safety
    /// Run on the server thread without outstanding client, TTY, session or
    /// pane payload borrows. `window` must be the client's current window;
    /// resolve it immediately before calling without an intervening callback.
    pub(crate) unsafe fn pan_window(
        &mut self,
        window: &WindowRef,
        direction: ClientPanDirection,
        adjustment: u_int,
    ) {
        unsafe {
            let client = self.as_client_mut();
            if !client_get_pan_window(client).is_some_and(|pan| pan.ptr_eq(window)) {
                client_set_pan_window(client, Some(window));
                client.pan_ox = client.tty.oox;
                client.pan_oy = client.tty.ooy;
            }
            match direction {
                ClientPanDirection::Left => {
                    client.pan_ox = client.pan_ox.saturating_sub(adjustment);
                }
                ClientPanDirection::Right => {
                    let limit = window.dimensions().size.width.wrapping_sub(client.tty.osx);
                    client.pan_ox = client.pan_ox.wrapping_add(adjustment).min(limit);
                }
                ClientPanDirection::Up => {
                    client.pan_oy = client.pan_oy.saturating_sub(adjustment);
                }
                ClientPanDirection::Down => {
                    let limit = window.dimensions().size.height.wrapping_sub(client.tty.osy);
                    client.pan_oy = client.pan_oy.wrapping_add(adjustment).min(limit);
                }
            }
            tty_update_client_offset(client);
            server_redraw_client(client);
        }
    }

    /// Forces status regeneration and schedules a status-only or full redraw.
    /// No callbacks run and no client or terminal borrow escapes.
    ///
    /// # Safety
    /// Exclude other client payload access during this call.
    pub(crate) unsafe fn refresh_display(&mut self, status_only: bool) {
        unsafe {
            let client = self.as_client_mut();
            client.flags |= CLIENT_STATUSFORCE as uint64_t;
            if status_only {
                server_status_client(client);
            } else {
                server_redraw_client(client);
            }
        }
    }
}

impl ClientRef {
    /// Whether the client's environment and terminal identify a nested session.
    ///
    /// # Safety
    /// Exclude conflicting client and registered-pane access during this call.
    pub(crate) unsafe fn is_nested(&mut self) -> bool {
        unsafe { server_client_check_nested(self.as_client_mut()) != 0 }
    }

    /// Applies the named flag changes and sends the resulting flags to the peer.
    /// Control flags retain their existing offset-reset effects.
    ///
    /// # Safety
    /// Exclude conflicting client, control and pane access during this call.
    pub(crate) unsafe fn apply_flags(&mut self, flags: &CStr) {
        unsafe { server_client_set_flags(self.as_client_mut(), flags) }
    }

    /// Makes the client read-only and excludes it from size calculations.
    ///
    /// # Safety
    /// Exclude other access to the client's flags during this call.
    pub(crate) unsafe fn make_read_only(&mut self) {
        unsafe { *self.flags_mut() |= (CLIENT_READONLY | CLIENT_IGNORESIZE) as uint64_t }
    }

    /// Toggles read-only mode and its paired exclusion from size calculations.
    ///
    /// # Safety
    /// Exclude other access to the client's flags during this call.
    pub(crate) unsafe fn toggle_read_only(&mut self) {
        unsafe {
            let flags = self.flags_mut();
            let mask = (CLIENT_READONLY | CLIENT_IGNORESIZE) as uint64_t;
            if *flags & CLIENT_READONLY as uint64_t != 0 {
                *flags &= !mask;
            } else {
                *flags |= mask;
            }
        }
    }

    /// Records the current session as the previous session, or clears it when
    /// unattached. Returns a retained handle for the caller's transition while
    /// the stored previous-session reference remains weak.
    ///
    /// # Safety
    /// Exclude conflicting client and session access during this call.
    pub(crate) unsafe fn remember_session(&mut self) -> Option<SessionRef> {
        unsafe {
            let session = self.attached_session();
            client_set_last_session(
                self.as_client_mut(),
                session.as_ref().map(|session| session.as_session()),
            );
            session
        }
    }

    /// Opens the client's terminal, returning the existing failure cause.
    /// Control clients succeed without opening a terminal.
    ///
    /// # Safety
    /// Run on the initialized server thread with no outstanding client or TTY
    /// payload borrows. Terminal setup may revisit the client through its owner.
    pub(crate) unsafe fn open_terminal(&mut self, cause: &mut Option<CString>) -> core::ffi::c_int {
        unsafe { server_client_open(self.as_client_mut(), cause) }
    }

    /// Records a pending detach with the requested peer message. Unattached,
    /// dead and already-exiting clients retain the existing no-op behavior.
    ///
    /// # Safety
    /// Exclude conflicting client and attached-session access during this call.
    pub(crate) unsafe fn detach(&mut self, message: msgtype) {
        unsafe { server_client_detach(self.as_client_mut(), message) }
    }

    /// Requests that the peer execute a shell command, using the session's shell
    /// option or global options while unattached. Empty commands do nothing.
    ///
    /// # Safety
    /// Exclude conflicting client, session, options and peer access during this call.
    pub(crate) unsafe fn exec_shell(&mut self, command: &CStr) {
        unsafe { server_client_exec(self.as_client_mut(), command) }
    }

    /// Stops the attached terminal and requests peer suspension when eligible.
    ///
    /// # Safety
    /// Run on the server thread with no outstanding client or TTY payload borrows.
    pub(crate) unsafe fn suspend(&mut self) {
        unsafe { server_client_suspend(self.as_client_mut()) }
    }

    /// Records an exit message and requests exit through the server event loop.
    ///
    /// # Safety
    /// Exclude other access to the client's exit message and flags during this call.
    pub(crate) unsafe fn request_exit(&mut self, message: &CStr) {
        unsafe {
            *self.exit_message_mut() = Some(message.to_owned());
            *self.flags_mut() |= CLIENT_EXIT as uint64_t;
        }
    }

    /// Updates the supplied environment from this client using the configured
    /// `update-environment` patterns. The existing store handles matching and
    /// clearing; unrelated entries are not copied into an intermediate store.
    ///
    /// # Safety
    /// Exclude mutation of the client environment or options and all other
    /// destination access during this synchronous call. No callbacks run.
    pub(crate) unsafe fn update_environment(
        &self,
        options: &RustOptionsRef,
        destination: &mut crate::environ::RustEnvironment,
    ) {
        unsafe {
            crate::environ::update_environment(options, self.environ_ref(), destination);
        }
    }

    /// Assigns a retained session through the server's lifecycle transition.
    /// Preserves previous-session tracking, focus, sizing, alerts, activity,
    /// redraw and queued `client-session-changed` delivery.
    ///
    /// # Safety
    /// Run on the initialized server thread with no outstanding client, session,
    /// window or pane payload borrows. Lifecycle work may revisit their handles;
    /// notification hooks run later through the command queue.
    pub(crate) unsafe fn set_session(&mut self, session: Option<&SessionRef>) {
        unsafe { server_client_set_session(self.as_client_mut(), session) }
    }

    /// Selects a key table and updates its activity time. `None` selects the
    /// current session's default table, falling back to `root`.
    ///
    /// # Safety
    /// Exclude conflicting client, session and key-table access during this call.
    pub(crate) unsafe fn set_key_table(&mut self, name: Option<&CStr>) {
        unsafe { server_client_set_key_table(self.as_client_mut(), name) }
    }

    /// Sends readiness to a terminal peer. Control clients require no message.
    ///
    /// # Safety
    /// Exclude conflicting client and peer access during this call.
    pub(crate) unsafe fn send_ready(&self) {
        unsafe {
            if self.flags() & CLIENT_CONTROL as uint64_t == 0 {
                self.peer_handle().send(MSG_READY, -1, &[]);
            }
        }
    }

    /// Records attachment after the caller's session and output stages finish.
    /// This does not send readiness or queue a client-attached notification.
    ///
    /// # Safety
    /// Exclude other access to the client's flags during this call.
    pub(crate) unsafe fn mark_attached(&mut self) {
        unsafe { *self.flags_mut() |= CLIENT_ATTACHED as uint64_t }
    }

    /// Completes a fresh attachment after session and key-table selection.
    /// Sends terminal readiness, queues `client-attached`, then marks the client
    /// attached. Control clients omit the readiness message.
    ///
    /// # Safety
    /// Run on the initialized server thread after a successful fresh attachment,
    /// with no outstanding client or target payload borrows. Notification hooks
    /// run later through the command queue.
    pub(crate) unsafe fn finish_attachment(&mut self) {
        unsafe {
            self.send_ready();
            notify_client(c"client-attached", Some(self.as_client_mut()));
            self.mark_attached();
        }
    }

    /// Returns the names of the client's enabled flags.
    ///
    /// # Safety
    /// Exclude conflicting access to the client during this call.
    pub(crate) unsafe fn flag_names(&mut self) -> CString {
        unsafe { server_client_get_flags(self.as_client_mut()) }
    }

    /// Returns the default key table selected by the client's session.
    ///
    /// # Safety
    /// Exclude conflicting client and session access during this call.
    pub(crate) unsafe fn default_key_table(&self) -> std::rc::Rc<CStr> {
        unsafe { server_client_get_key_table(self.as_client()) }
    }

    /// Retains the client's previous session while it is alive.
    ///
    /// # Safety
    /// Exclude conflicting client access during this call.
    pub(crate) unsafe fn last_session(&self) -> Option<SessionRef> {
        unsafe { client_get_last_session(self.as_client()) }
    }

    /// Resolves the working directory for commands started by this client.
    ///
    /// # Safety
    /// Exclude conflicting client and session access during this call.
    pub(crate) unsafe fn working_directory(&self) -> CString {
        unsafe { server_client_get_cwd(Some(self.as_client()), None) }
    }
}

impl ClientRef {
    /// Whether this client uses the control protocol.
    ///
    /// # Safety
    /// Exclude conflicting client payload access during this call.
    pub(crate) unsafe fn is_control(&self) -> bool {
        unsafe { self.flags() & CLIENT_CONTROL as uint64_t != 0 }
    }

    /// Sets the control client's terminal dimensions, clears pixel dimensions,
    /// marks the size changed and immediately recalculates server sizes.
    /// The client borrow ends before recalculation, which may resize windows and
    /// enqueue layout notifications and hooks.
    ///
    /// # Safety
    /// Run on the server thread without outstanding client, TTY, session,
    /// window or pane payload borrows. The client must use the control protocol.
    /// Both cell dimensions must be within the supported window size limits.
    pub(crate) unsafe fn set_control_size(&mut self, width: u_int, height: u_int) {
        unsafe {
            {
                let client = self.as_client_mut();
                crate::tty::tty_set_size(&mut client.tty, width, height, 0, 0);
                client.flags |= crate::resize::CLIENT_SIZECHANGED as uint64_t;
            }
            crate::resize::recalculate_sizes_now(1);
        }
    }

    /// Stores a control client's per-window dimensions and marks its window
    /// sizes changed before immediately recalculating server sizes. The entry
    /// is created even if the window ID is not registered. No window is retained.
    /// The client borrow ends before recalculation and deferred layout effects.
    ///
    /// # Safety
    /// Run on the server thread without outstanding client, TTY, session,
    /// window or pane payload borrows. The client must use the control protocol.
    /// Both cell dimensions must be within the supported window size limits.
    pub(crate) unsafe fn set_control_window_size(
        &mut self,
        window: u_int,
        width: u_int,
        height: u_int,
    ) {
        unsafe {
            {
                let client = self.as_client_mut();
                let entry = server_client_add_client_window(client, window);
                entry.sx = width;
                entry.sy = height;
                client.flags |= crate::resize::CLIENT_WINDOWSIZECHANGED;
            }
            crate::resize::recalculate_sizes_now(1);
        }
    }

    /// Clears the dimensions of an existing control-client window entry and
    /// immediately recalculates server sizes.
    /// Does not create or remove an entry or mark the window-size-changed flag.
    /// Missing entries do nothing. The client borrow ends before recalculation
    /// and deferred layout effects.
    ///
    /// # Safety
    /// Run on the server thread without outstanding client, TTY, session,
    /// window or pane payload borrows. The client must use the control protocol.
    pub(crate) unsafe fn clear_control_window_size(&mut self, window: u_int) {
        unsafe {
            {
                let client = self.as_client_mut();
                let name = client.name.clone();
                let Some(entry) = server_client_get_client_window(client, window) else {
                    return;
                };
                log_debug(
                    c"%s: client %s window @%u: no size",
                    fmt_args![c"clear_control_window_size", name.as_deref(), window],
                );
                entry.sx = 0;
                entry.sy = 0;
            }
            crate::resize::recalculate_sizes_now(1);
        }
    }
}

impl ClientRef {
    /// Schedules this attached client's display updates after selection in a
    /// retained window. Control and unattached clients do nothing. A current
    /// window larger than the terminal requests a full redraw; otherwise its
    /// borders redraw when current, and status redraws when linked to the session.
    /// Window identity and session membership are checked independently, so a
    /// session with no current window can still request its linked-window status.
    /// No hooks, callbacks or offset recalculation run and no borrow escapes.
    ///
    /// # Safety
    /// Run on the server thread without conflicting client, terminal, session
    /// or window payload access. The window need not be current or registered;
    /// the client terminal's owner relationship must be initialized when queried.
    pub(crate) unsafe fn redraw_pane_selection(&mut self, window: &WindowRef) {
        unsafe {
            if self.flags() & CLIENT_CONTROL as uint64_t != 0 {
                return;
            }
            let Some(session) = self.attached_session() else {
                return;
            };
            let current = session
                .curw()
                .and_then(|link| link.window())
                .is_some_and(|current| current.ptr_eq(window));
            if current && crate::tty::tty_window_bigger(self.as_tty()) != 0 {
                server_redraw_client(self.as_client_mut());
            } else {
                let linked = session.has(window);
                if current {
                    *self.flags_mut() |= CLIENT_REDRAWBORDERS as uint64_t;
                }
                if linked {
                    *self.flags_mut() |= CLIENT_REDRAWSTATUS as uint64_t;
                }
            }
        }
    }
}

impl ClientRef {
    /// Returns this client's active pane using the existing per-client policy.
    ///
    /// # Safety
    /// Exclude conflicting client, window and pane access on the server thread.
    pub(crate) unsafe fn selected_pane(&self) -> Option<RustWindowPaneWeak> {
        unsafe { server_client_get_pane(self.as_client()) }
    }

    /// Updates the client's pane selection without changing global window selection.
    ///
    /// # Safety
    /// Resolve a live pane immediately before this call and exclude conflicting
    /// client, window and pane access on the server thread.
    pub(crate) unsafe fn select_client_pane(&mut self, pane: &RustWindowPaneWeak) {
        unsafe { server_client_set_pane(self.as_client_mut(), pane.as_pane()) };
    }

    /// Scrolls copy mode using the client's current scrollbar drag position.
    ///
    /// # Safety
    /// The client TTY must be initialized and the pane must remain live in copy
    /// mode. Exclude conflicting client, TTY, window, pane and mode access during
    /// offset calculation and scrolling. Existing mode behavior is reused.
    pub(crate) unsafe fn scroll_copy_pane(
        &self,
        pane: &RustWindowPaneWeak,
        y: u_int,
        exit: core::ffi::c_int,
    ) {
        unsafe {
            let (_, _, oy, _, _) = crate::tty::tty_window_offset(self.as_tty());
            let position = self.as_tty().mouse_slider_mpos;
            pane.copy_scroll(position, y, oy, exit);
        }
    }
}

/// Resolves cwd using the existing client/session fallback order into owned text.
///
/// # Safety
/// Run on the server thread without conflicting client, TTY, session or queue
/// payload access. Existing replacement/completion callbacks may run inline;
/// exclude conflicting callback state access. No payload reference escapes.
pub(crate) unsafe fn client_working_directory(
    c: Option<&ClientRef>,
    s: Option<&SessionRef>,
) -> CString {
    unsafe { server_client_get_cwd(c.map(|c| c.as_client()), s.map(|s| s.as_session())) }
}

/// Routes output with the existing control/attached-client print behavior.
///
/// # Safety
/// Run on the server thread without conflicting client, TTY, session or queue
/// payload access. Existing replacement/completion callbacks may run inline;
/// exclude conflicting callback state access. No payload reference escapes.
pub(crate) unsafe fn client_print_buffer(
    c: Option<&mut ClientRef>,
    parse: core::ffi::c_int,
    buffer: &mut ByteBuffer,
) {
    unsafe { server_client_print(c.map(|c| c.as_client_mut()), parse, buffer) }
}

/// Installs the existing overlay and timeout, including replacement cleanup.
///
/// # Safety
/// Run on the server thread without conflicting client, TTY, session or queue
/// payload access. Existing replacement/completion callbacks may run inline;
/// exclude conflicting callback state access. No payload reference escapes.
pub(crate) unsafe fn client_set_overlay(
    c: &mut ClientRef,
    delay: u_int,
    overlay: Overlay,
    data: OverlayState,
) {
    unsafe { server_client_set_overlay(c.as_client_mut(), delay, overlay, data) }
}

/// Clears the current overlay through its existing free/completion callbacks.
///
/// # Safety
/// Run on the server thread without conflicting client, TTY, session or queue
/// payload access. Existing replacement/completion callbacks may run inline;
/// exclude conflicting callback state access. No payload reference escapes.
pub(crate) unsafe fn client_clear_overlay(c: &mut ClientRef) {
    unsafe { server_client_clear_overlay(c.as_client_mut()) }
}

impl ClientRef {
    /// Sends bytes to the terminal clipboard using the existing selection encoder.
    ///
    /// # Safety
    /// The TTY must be initialized. Exclude conflicting client/TTY access. The
    /// caller retains attachment/death policy; no completion callback is added.
    pub(crate) unsafe fn set_clipboard(&mut self, bytes: &[u8]) {
        unsafe { crate::tty::tty_set_selection(self.as_tty_mut(), c"", bytes) };
    }

    /// Prints captured pane bytes, retaining control formatting and file routing.
    /// Returns false when ordinary client output is unavailable. Control output
    /// retains printf precision, including its handling of embedded NUL bytes.
    ///
    /// # Safety
    /// Exclude conflicting client, file and control state access on the server
    /// thread. The existing output paths may queue asynchronous writes.
    pub(crate) unsafe fn print_capture(&mut self, bytes: &[u8]) -> bool {
        unsafe {
            if self.is_control() {
                crate::control::control_write(
                    self.as_client_mut(),
                    c"%.*s",
                    fmt_args![bytes.len() as core::ffi::c_int, bytes],
                );
            } else {
                if crate::file::file_can_print(Some(self.as_client())) == 0 {
                    return false;
                }
                crate::file::file_print_buffer(Some(self.as_client_mut()), bytes);
                crate::file::file_print(Some(self.as_client_mut()), c"\n", fmt_args![]);
            }
            true
        }
    }
}

#[cfg(test)]
pub use crate::consts::{
    KEYC_MOUSEDRAG1_PANE, KEYC_MOUSEDRAG1_SCROLLBAR_SLIDER, PROGRESS_BAR_HIDDEN,
    PROGRESS_BAR_NORMAL, PROGRESS_BAR_PAUSED,
};
