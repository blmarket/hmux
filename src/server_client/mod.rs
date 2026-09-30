use crate::src::session::Session as _;
use crate::src::shared::client::{ClientRef, ClientWeak};
use crate::src::shared::session::SessionRef;
use crate::src::shared::window::WindowRef;
use crate::src::window::Window as _;
use crate::src::window::Window as _;
use std::time::{Duration, SystemTime};
mod api;
mod format;
mod model;
mod overlay;
pub use api::{Client, PanDirection};
pub use model::client;
pub use overlay::Overlay;

use crate::src::alerts::alerts_check_session;
use crate::src::cfg::{cfg_client, cfg_finished, start_cfg};
use crate::src::cmd::find::{cmd_find_from_client, cmd_find_from_mouse};
use crate::src::cmd::parse::cmd_parse_from_argv;
use crate::src::cmd::queue::{
    cmdq_abort_file_wait, cmdq_append, cmdq_error, cmdq_get_callback_owned, cmdq_get_client,
    cmdq_get_command, cmdq_get_error, cmdq_insert_after, cmdq_new,
};
use crate::src::cmd::{cmd_list_all_have, cmd_log_argv};
use crate::src::compat::imsg::imsg_get_fd;
use crate::src::compat::imsg::msg_command;
use crate::src::control::{
    control_all_done, control_discard, control_discard_all, control_pane_offset, control_ready,
    control_reset_offsets, control_start, control_stop, control_write,
};
use crate::src::environ::{environ_create, environ_find, environ_put};
use crate::src::events::{events_fire, events_fire_client};
use crate::src::events_payload::{
    event_payload_create, event_payload_set_client, event_payload_set_int, event_payload_set_pane,
    event_payload_set_session, event_payload_set_target, event_payload_set_uint,
    event_payload_set_window,
};
use crate::src::ffi::libc::{
    access, close, free, isatty, memcpy, sscanf, strchr, strcmp, strlcat, strlen, strsep, ttyname,
};
use crate::src::file::{
    file_print, file_read_data, file_read_done, file_write_done, file_write_ready,
};
use crate::src::format::bytes::xformat;
use crate::src::format::bytes::{write_cstr, write_cstr_n};
use crate::src::format::{
    format_create, format_defaults, format_expand_cstring, format_expand_time_cstring, format_free,
    format_lost_client,
};
use crate::src::key_bindings::{key_bindings_dispatch, key_bindings_get, key_bindings_get_table};
use crate::src::key_string::key_string_format;
use crate::src::log::{fatal, log_cstr, log_debug, log_get_level, log_hex, log_pointer};
use crate::src::menu::{menu_close, menu_get_cursor, menu_key, menu_screen};
use crate::src::names::check_window_name;
use crate::src::options::options_owner_ptr;
use crate::src::options::{
    options_get_command, options_get_number, options_get_string, options_set_number,
};
use crate::src::proc::{proc_add_peer, proc_kill_peer, proc_remove_peer, proc_send};
use crate::src::prompt::prompt_free;
use crate::src::reactor::{
    bufferevent_disable, bufferevent_enable, bufferevent_get_input, evbuffer_add, evbuffer_drain,
    evbuffer_get_length, evbuffer_pullup, evbuffer_readln,
};
use crate::src::resize::{recalculate_size, recalculate_sizes, resize_window};
use crate::src::screen::screen_mode_display;
use crate::src::screen_redraw::{redraw_pane, redraw_pane_scrollbar, redraw_screen};
use crate::src::server::{current_time, server_add_accept, server_proc, server_update_socket};
use crate::src::server_fn::{
    server_check_unattached, server_destroy_pane, server_kill_pane, server_redraw_client,
    server_redraw_window_borders, server_status_client, server_status_window,
};
use crate::src::session::session_find_by_id;
use crate::src::shared::command::unpack_argv;
use crate::src::shared::events::event_payload;
use crate::src::status::{
    status_at_line, status_free, status_get_range, status_init, status_line_size,
    status_message_clear, status_prompt_clear, status_prompt_cursor, status_prompt_key,
    status_timer_start,
};
use crate::src::style::colour::{
    colour_parse_cstr, colour_theme_option, colour_theme_terminal_colour, colour_totheme,
};
use crate::src::text::utf8::{utf8_sanitize_cstring, utf8_stravisx_bytes};
use crate::src::tmux::{checkshell, find_home_cstr, global_options, global_s_options, setblocking};
use crate::src::tty::{
    tty_close, tty_cursor, tty_free, tty_init, tty_invalidate, tty_margin_off, tty_open,
    tty_region_off, tty_repeat_requests, tty_reset, tty_resize, tty_send_requests, tty_set_path,
    tty_set_progress_bar, tty_set_title, tty_start_tty, tty_stop_tty, tty_sync_end,
    tty_update_client_offset, tty_update_mode, tty_window_offset,
};
use crate::src::tty_features::tty_get_features;
use crate::src::tty_term::tty_term_has;
use crate::src::tty_term::tty_term_owner_ptr;
use crate::src::window::windows;
use crate::src::window::{
    all_window_panes, window_get_active_at, window_pane_clear_resizes, window_pane_contains,
    window_pane_find_by_id, window_pane_get_new_data, window_pane_get_pane_lines,
    window_pane_get_pane_status, window_pane_has_prompt, window_pane_is_floating,
    window_pane_is_visible, window_pane_key, window_pane_next, window_pane_paste,
    window_pane_prompt_key, window_pane_scrollbar_overlay, window_pane_scrollbar_overlay_visible,
    window_pane_scrollbar_reserve, window_pane_scrollbar_show, window_pane_scrollbar_start_timer,
    window_pane_scrollbar_visible, window_pane_send_resize, window_pane_send_theme_update,
    window_pane_set_mode, window_pane_status_get_range, window_pane_tree_minmax,
    window_pane_tree_next, window_redraw_active_switch, window_set_active_pane,
    window_update_focus, windows_minmax, winlink_find_by_index,
};
use crate::src::window_copy::{window_copy_add, window_view_mode};
use crate::src::window_visible::{window_position_is_visible, window_visible_ranges};
use hmux_buffer::SegmentedBuf;
use std::ffi::{CStr, CString};

use crate::src::shared::abi::*;
use crate::src::shared::abi::{ssize_t, uint32_t};
use crate::src::shared::arguments::args;
use crate::src::shared::client::*;
pub use crate::src::shared::client::{
    client_file, overlay_check_cb, overlay_draw_cb, overlay_free_cb, overlay_key_cb,
    overlay_mode_cb, overlay_resize_cb,
};

impl client {
    /// Allocate a client on the server thread.
    ///
    /// # Safety
    /// The caller must serialize access to the global server model and perform
    /// client-loss cleanup before releasing a fully initialized client.
    pub unsafe fn new() -> ClientRef {
        std::rc::Rc::new_cyclic(|observer| {
            let mut value = client::empty();
            value.observer = observer.clone();
            std::cell::UnsafeCell::new(value)
        })
    }
}

/// Own active clients in insertion order until explicit client-loss cleanup.
/// Each client retains its weak successor even after active removal.
pub struct ClientRegistry {
    ordered: Vec<ClientRef>,
}

impl ClientRegistry {
    pub(crate) const fn new() -> Self {
        Self {
            ordered: Vec::new(),
        }
    }

    pub(crate) fn first(&self) -> Option<ClientRef> {
        self.ordered.first().cloned()
    }

    pub(crate) fn next(&self, current: &ClientRef) -> Option<ClientRef> {
        unsafe { (*current.get()).registry_next.upgrade() }
    }

    pub(crate) fn push_back(&mut self, owner: ClientRef) {
        assert!(
            !self
                .ordered
                .iter()
                .any(|current| std::rc::Rc::ptr_eq(current, &owner)),
            "client registered twice"
        );
        unsafe {
            if let Some(previous) = self.ordered.last() {
                (*previous.get()).registry_next = std::rc::Rc::downgrade(&owner);
            }
            (*owner.get()).registry_next = std::rc::Weak::new();
        }
        self.ordered.push(owner);
    }

    /// Transfer the registry's strong reference to explicit cleanup, preserving
    /// the removed client's successor so an in-progress traversal can continue.
    pub(crate) fn remove(&mut self, observer: &ClientWeak) -> Option<ClientRef> {
        let index = self
            .ordered
            .iter()
            .position(|owner| std::rc::Weak::ptr_eq(&std::rc::Rc::downgrade(owner), observer))?;
        if index > 0 {
            unsafe {
                (*self.ordered[index - 1].get()).registry_next =
                    (*self.ordered[index].get()).registry_next.clone();
            }
        }
        Some(self.ordered.remove(index))
    }

    pub(crate) fn clear(&mut self) {
        for owner in std::mem::take(&mut self.ordered) {
            unsafe {
                (*owner.get()).registry_next = std::rc::Weak::new();
            }
            server_client_unref_owned(owner);
        }
    }
}

pub static mut clients: ClientRegistry = ClientRegistry::new();

fn server_client_set_message(c: &mut client, message: Option<CString>) {
    c.message_string = message;
}

fn server_client_set_ttyname(c: &mut client, ttyname: Option<CString>) {
    c.ttyname = ttyname;
}

fn server_client_set_term_name(c: &mut client, term_name: Option<CString>) {
    c.term_name = term_name;
}

fn server_client_set_cwd(c: &mut client, cwd: Option<CString>) {
    c.cwd = cwd;
}

fn server_client_replace_title(c: &mut client, title: Option<CString>) {
    c.title = title;
}

fn server_client_replace_path(c: &mut client, path: Option<CString>) {
    c.path = path;
}

fn server_client_set_exit_session(c: &mut client, exit_session: Option<CString>) {
    c.exit_session = exit_session;
}

fn server_client_set_user(c: &mut client, user: Option<CString>) {
    c.user = user;
}

fn server_client_set_name(c: &mut client, name: Option<CString>) {
    c.name = name;
}

fn server_client_set_exit_message(c: &mut client, exit_message: Option<CString>) {
    c.exit_message = exit_message;
}

fn server_client_set_term_type(c: &mut client, term_type: Option<CString>) {
    c.term_type = term_type;
}

unsafe fn server_client_ensure_term_name(c: &mut client) {
    if c.term_name.is_none() || *c.term_name.as_ref().unwrap().as_ptr() == 0 {
        server_client_set_term_name(c, Some(CString::new("unknown").unwrap()));
    }
}

fn server_client_add_term_cap(c: &mut client, data: &CStr) {
    assert!(c.term_caps.len() < u_int::MAX as usize);
    c.term_caps.push(data.to_owned());
}

fn server_client_clear_term_caps(c: &mut client) {
    // Release on client loss, even if other references delay client destruction.
    c.term_caps = Vec::new();
}

#[cfg(test)]
mod client_message_owner_tests {
    use super::{
        client, server_client_add_term_cap, server_client_clear_term_caps,
        server_client_ensure_term_name, server_client_replace_path, server_client_replace_title,
        server_client_set_cwd, server_client_set_exit_message, server_client_set_exit_session,
        server_client_set_message, server_client_set_name, server_client_set_term_name,
        server_client_set_term_type, server_client_set_ttyname, server_client_set_user,
    };
    use crate::src::status::status_message_clear;
    use std::ffi::{CStr, CString};

    #[test]
    fn message_replacement_and_clear_keep_the_client_pointer_stable() {
        unsafe {
            let owner = client::new();
            let c = owner.get();
            assert!((*c).message_string.is_none());

            server_client_set_message(&mut *c, Some(CString::new(vec![b'a', 0xff]).unwrap()));
            assert_eq!(
                ((*c).message_string)
                    .as_deref()
                    .expect("string is present")
                    .to_bytes(),
                b"a\xff"
            );
            assert_eq!(c, owner.get());

            server_client_set_message(&mut *c, Some(CString::new(Vec::<u8>::new()).unwrap()));
            assert_eq!(
                ((*c).message_string)
                    .as_deref()
                    .expect("string is present")
                    .to_bytes(),
                b""
            );
            assert!(!(*c).message_string.is_none());

            // A live message has pushed a status screen. Keep one extra
            // screen user so status_message_clear does not need a full screen.
            (*c).status.screen_users = 2;
            status_message_clear(&(*c).observer.upgrade().expect("live client"));
            assert!((*c).message_string.is_none());
            assert!((*owner.get()).message_string.is_none());
            assert_eq!((*c).status.screen_users, 1);
        }
    }

    #[test]
    fn ttyname_replacement_and_clear_keep_a_borrowed_client_view() {
        unsafe {
            let mut owner = Box::new(client::empty());
            let c = &raw mut *owner;
            assert!((*c).ttyname.is_none());

            server_client_set_ttyname(&mut *c, Some(CString::new(b"/dev/\xff".to_vec()).unwrap()));
            assert_eq!(
                ((*c).ttyname)
                    .as_deref()
                    .expect("string is present")
                    .to_bytes(),
                b"/dev/\xff"
            );
            assert_eq!(c, &raw mut *owner);

            server_client_set_ttyname(&mut *c, Some(CString::new("").unwrap()));
            assert!(!(*c).ttyname.is_none());
            assert_eq!(
                ((*c).ttyname)
                    .as_deref()
                    .expect("string is present")
                    .to_bytes(),
                b""
            );

            server_client_set_ttyname(&mut *c, None);
            assert!((*c).ttyname.is_none());
            assert!(owner.ttyname.is_none());
        }
    }

    #[test]
    fn term_name_replacement_fallback_and_clear_keep_a_borrowed_client_view() {
        unsafe {
            let mut owner = Box::new(client::empty());
            let c = &raw mut *owner;
            server_client_ensure_term_name(&mut *c);
            assert_eq!(
                ((*c).term_name)
                    .as_deref()
                    .expect("string is present")
                    .to_bytes(),
                b"unknown"
            );

            server_client_set_term_name(
                &mut *c,
                Some(CString::new(b"term-\xff".to_vec()).unwrap()),
            );
            assert_eq!(
                ((*c).term_name)
                    .as_deref()
                    .expect("string is present")
                    .to_bytes(),
                b"term-\xff"
            );
            assert_eq!(c, &raw mut *owner);

            server_client_set_term_name(&mut *c, Some(CString::new("").unwrap()));
            server_client_ensure_term_name(&mut *c);
            assert_eq!(
                ((*c).term_name)
                    .as_deref()
                    .expect("string is present")
                    .to_bytes(),
                b"unknown"
            );

            server_client_set_term_name(&mut *c, None);
            assert!((*c).term_name.is_none());
            assert!(owner.term_name.is_none());
        }
    }

    #[test]
    fn cwd_replacement_and_clear_keep_a_borrowed_client_view() {
        unsafe {
            let mut owner = Box::new(client::empty());
            let c = &raw mut *owner;
            assert!((*c).cwd.is_none());

            server_client_set_cwd(&mut *c, Some(CString::new(b"/work-\xff".to_vec()).unwrap()));
            assert_eq!(
                ((*c).cwd).as_deref().expect("string is present").to_bytes(),
                b"/work-\xff"
            );
            assert_eq!(c, &raw mut *owner);

            server_client_set_cwd(&mut *c, Some(CString::new("").unwrap()));
            assert!(!(*c).cwd.is_none());
            assert_eq!(
                ((*c).cwd).as_deref().expect("string is present").to_bytes(),
                b""
            );

            server_client_set_cwd(&mut *c, None);
            assert!((*c).cwd.is_none());
            assert!(owner.cwd.is_none());
        }
    }

    #[test]
    fn term_type_replacement_and_clear_keep_a_borrowed_client_view() {
        unsafe {
            let mut owner = Box::new(client::empty());
            let c = &raw mut *owner;
            assert!((*c).term_type.is_none());

            server_client_set_term_type(
                &mut *c,
                Some(CString::new(b"term-\xff".to_vec()).unwrap()),
            );
            assert_eq!(
                ((*c).term_type)
                    .as_deref()
                    .expect("string is present")
                    .to_bytes(),
                b"term-\xff"
            );
            assert_eq!(c, &raw mut *owner);

            server_client_set_term_type(&mut *c, Some(CString::new("").unwrap()));
            assert!(!(*c).term_type.is_none());
            assert_eq!(
                ((*c).term_type)
                    .as_deref()
                    .expect("string is present")
                    .to_bytes(),
                b""
            );

            server_client_set_term_type(&mut *c, None);
            assert!((*c).term_type.is_none());
            assert!(owner.term_type.is_none());
        }
    }

    #[test]
    fn title_replacement_and_clear_keep_a_borrowed_client_view() {
        unsafe {
            let mut owner = Box::new(client::empty());
            let c = &raw mut *owner;
            assert!((*c).title.is_none());

            server_client_replace_title(
                &mut *c,
                Some(CString::new(b"title-\xff".to_vec()).unwrap()),
            );
            assert_eq!(
                ((*c).title)
                    .as_deref()
                    .expect("string is present")
                    .to_bytes(),
                b"title-\xff"
            );
            assert_eq!(c, &raw mut *owner);

            server_client_replace_title(&mut *c, Some(CString::new("").unwrap()));
            assert!(!(*c).title.is_none());
            assert_eq!(
                ((*c).title)
                    .as_deref()
                    .expect("string is present")
                    .to_bytes(),
                b""
            );

            server_client_replace_title(&mut *c, None);
            assert!((*c).title.is_none());
            assert!(owner.title.is_none());
        }
    }

    #[test]
    fn path_replacement_and_clear_keep_a_borrowed_client_view() {
        unsafe {
            let mut owner = Box::new(client::empty());
            let c = &raw mut *owner;
            assert!((*c).path.is_none());

            server_client_replace_path(
                &mut *c,
                Some(CString::new(b"file:///work-\xff".to_vec()).unwrap()),
            );
            assert_eq!(
                ((*c).path)
                    .as_deref()
                    .expect("string is present")
                    .to_bytes(),
                b"file:///work-\xff"
            );
            assert_eq!(c, &raw mut *owner);

            server_client_replace_path(&mut *c, Some(CString::new("").unwrap()));
            assert!(!(*c).path.is_none());
            assert_eq!(
                ((*c).path)
                    .as_deref()
                    .expect("string is present")
                    .to_bytes(),
                b""
            );

            server_client_replace_path(&mut *c, None);
            assert!((*c).path.is_none());
            assert!(owner.path.is_none());
        }
    }

    #[test]
    fn exit_session_replacement_and_clear_keep_a_borrowed_client_view() {
        unsafe {
            let mut owner = Box::new(client::empty());
            let c = &raw mut *owner;
            assert!((*c).exit_session.is_none());

            server_client_set_exit_session(
                &mut *c,
                Some(CString::new(b"session-\xff".to_vec()).unwrap()),
            );
            assert_eq!(
                ((*c).exit_session)
                    .as_deref()
                    .expect("string is present")
                    .to_bytes(),
                b"session-\xff"
            );
            assert_eq!(c, &raw mut *owner);

            server_client_set_exit_session(&mut *c, Some(CString::new("").unwrap()));
            assert!(!(*c).exit_session.is_none());
            assert_eq!(
                ((*c).exit_session)
                    .as_deref()
                    .expect("string is present")
                    .to_bytes(),
                b""
            );

            server_client_set_exit_session(&mut *c, None);
            assert!((*c).exit_session.is_none());
            assert!(owner.exit_session.is_none());
        }
    }

    #[test]
    fn user_replacement_and_clear_keep_a_borrowed_client_view() {
        unsafe {
            let mut owner = Box::new(client::empty());
            let c = &raw mut *owner;
            assert!((*c).user.is_none());

            server_client_set_user(&mut *c, Some(CString::new(b"user-\xff".to_vec()).unwrap()));
            assert_eq!(
                ((*c).user)
                    .as_deref()
                    .expect("string is present")
                    .to_bytes(),
                b"user-\xff"
            );
            assert_eq!(c, &raw mut *owner);

            server_client_set_user(&mut *c, Some(CString::new("").unwrap()));
            assert!(!(*c).user.is_none());
            assert_eq!(
                ((*c).user)
                    .as_deref()
                    .expect("string is present")
                    .to_bytes(),
                b""
            );

            server_client_set_user(&mut *c, None);
            assert!((*c).user.is_none());
            assert!(owner.user.is_none());
        }
    }

    #[test]
    fn name_replacement_and_clear_keep_a_borrowed_client_view() {
        unsafe {
            let mut owner = Box::new(client::empty());
            let c = &raw mut *owner;
            assert!((*c).name.is_none());

            server_client_set_name(
                &mut *c,
                Some(CString::new(b"/dev/pts/\xff".to_vec()).unwrap()),
            );
            assert_eq!(
                ((*c).name)
                    .as_deref()
                    .expect("string is present")
                    .to_bytes(),
                b"/dev/pts/\xff"
            );
            assert_eq!(c, &raw mut *owner);

            server_client_set_name(&mut *c, Some(CString::new("").unwrap()));
            assert!(!(*c).name.is_none());
            assert_eq!(
                ((*c).name)
                    .as_deref()
                    .expect("string is present")
                    .to_bytes(),
                b""
            );

            server_client_set_name(&mut *c, None);
            assert!((*c).name.is_none());
            assert!(owner.name.is_none());
        }
    }

    #[test]
    fn exit_message_replacement_and_clear_keep_a_borrowed_client_view() {
        unsafe {
            let mut owner = Box::new(client::empty());
            let c = &raw mut *owner;
            assert!((*c).exit_message.is_none());

            server_client_set_exit_message(
                &mut *c,
                Some(CString::new(b"error-\xff".to_vec()).unwrap()),
            );
            assert_eq!(
                ((*c).exit_message)
                    .as_deref()
                    .expect("string is present")
                    .to_bytes(),
                b"error-\xff"
            );
            assert_eq!(c, &raw mut *owner);

            server_client_set_exit_message(&mut *c, Some(CString::new("").unwrap()));
            assert!(!(*c).exit_message.is_none());
            assert_eq!(
                ((*c).exit_message)
                    .as_deref()
                    .expect("string is present")
                    .to_bytes(),
                b""
            );

            server_client_set_exit_message(&mut *c, None);
            assert!((*c).exit_message.is_none());
            assert!(owner.exit_message.is_none());
        }
    }

    #[test]
    fn term_caps_view_survives_growth_and_preserves_order_and_bytes() {
        unsafe {
            let mut owner = Box::new(client::empty());
            let c = &raw mut *owner;
            let mut expected = Vec::new();
            for i in 0..64 {
                let value = if i % 3 == 0 {
                    CString::new(b"dup=\xff".to_vec()).unwrap()
                } else {
                    CString::new(format!("cap{i}=value")).unwrap()
                };
                server_client_add_term_cap(&mut *c, value.as_c_str());
                expected.push(value);
                assert_eq!((*c).term_caps.len(), expected.len());
                for (index, cap) in expected.iter().enumerate() {
                    assert_eq!((&(*c).term_caps)[index].as_bytes(), cap.as_bytes());
                }
            }
            server_client_clear_term_caps(&mut *c);
            assert!((*c).term_caps.is_empty());
            assert_eq!((*c).term_caps.len(), 0);
        }
    }
}
use crate::src::compat::imsg::imsg;
use crate::src::compat::imsg::IMSG_HEADER_SIZE;
use crate::src::compat::imsg::*;
use crate::src::shared::client::{
    CLIENT_ALLREDRAWFLAGS, CLIENT_ASSUMEPASTING, CLIENT_ATTACHED, CLIENT_BRACKETPASTING,
    CLIENT_CONTROL, CLIENT_CONTROL_NEWLAYOUTS, CLIENT_CONTROL_NOOUTPUT, CLIENT_CONTROL_PAUSEAFTER,
    CLIENT_CONTROL_WAITEXIT, CLIENT_DEAD, CLIENT_DOUBLECLICK, CLIENT_EXIT, CLIENT_EXITED,
    CLIENT_FOCUSED, CLIENT_IDENTIFIED, CLIENT_IGNORESIZE, CLIENT_NODETACHFLAGS,
    CLIENT_NO_DETACH_ON_DESTROY, CLIENT_PASTE_TIME_LIMIT, CLIENT_READONLY, CLIENT_REDRAWBORDERS,
    CLIENT_REDRAWMENU, CLIENT_REDRAWOVERLAY, CLIENT_REDRAWSCROLLBARS, CLIENT_REDRAWSTATUS,
    CLIENT_REDRAWWINDOW, CLIENT_REPEAT, CLIENT_STATUSFORCE, CLIENT_SUSPENDED, CLIENT_TERMINAL,
    CLIENT_TRIPLECLICK, CLIENT_UNATTACHEDFLAGS, CLIENT_UTF8,
};
use crate::src::shared::colour::*;
use crate::src::shared::colour::{COLOUR_FLAG_THEME, COLOUR_THEME_COUNT};
use crate::src::shared::command::cmd_parse_input;
use crate::src::shared::command::CMD_READONLY;
use crate::src::shared::command::*;
use crate::src::shared::command::{cmd_find_state, cmd_list, cmdq_item, cmdq_state};
use crate::src::shared::display::visible_ranges;
use crate::src::shared::display::*;
use crate::src::shared::environment::environ_entry;
use crate::src::shared::errno::EINTR;
use crate::src::shared::event::EV_READ;
use crate::src::shared::event::*;
use crate::src::shared::format::format_tree;
use crate::src::shared::format::{FORMAT_NOJOBS, FORMAT_NONE};
use crate::src::shared::key::KEY_BINDING_REPEAT;
use crate::src::shared::key::*;
use crate::src::shared::key::{key_event, key_table, KeyBindingCommand};
use crate::src::shared::layout::*;
use crate::src::shared::limits::{SIZE_MAX, UINT_MAX};
use crate::src::shared::mouse::{
    mouse_event, MOUSE_BUTTON_1, MOUSE_BUTTON_10, MOUSE_BUTTON_11, MOUSE_BUTTON_2, MOUSE_BUTTON_3,
    MOUSE_BUTTON_6, MOUSE_BUTTON_7, MOUSE_BUTTON_8, MOUSE_BUTTON_9, MOUSE_MASK_BUTTONS,
    MOUSE_MASK_CTRL, MOUSE_MASK_DRAG, MOUSE_MASK_META, MOUSE_MASK_SHIFT, MOUSE_WHEEL_DOWN,
    MOUSE_WHEEL_UP,
};
use crate::src::shared::options::options;
use crate::src::shared::pane::window_pane;
use crate::src::shared::pane::{
    window_pane_offset, window_pane_resize, PANE_ACTIVITY, PANE_CAPTUREALLKEYS, PANE_CLOSEONCANCEL,
    PANE_CLOSEONCLICK, PANE_EXITED, PANE_REDRAW, PANE_REDRAWSCROLLBAR, PANE_SCROLLBARS_AUTOHIDE,
    PANE_SCROLLBARS_LEFT, PANE_SCROLLBARS_MODAL, PANE_SCROLLBARS_RIGHT, PANE_STATUS_BOTTOM,
    PANE_STATUS_OFF, PANE_STATUS_TOP, PANE_STYLECHANGED,
};
use crate::src::shared::posix_io::{
    _PATH_BSHELL, STDERR_FILENO, STDIN_FILENO, STDOUT_FILENO, X_OK,
};
use crate::src::shared::process::tmuxpeer;
use crate::src::shared::screen::{
    screen, ScreenMode, ALL_MOUSE_MODES, CURSOR_MODES, MODE_BRACKETPASTE, MODE_CURSOR,
    MODE_MOUSE_ALL, MODE_MOUSE_BUTTON, MODE_SYNC,
};
use crate::src::shared::session::session;
use crate::src::shared::style::*;
use crate::src::shared::tree::RB_NEGINF;
use crate::src::shared::tty::tty;
use crate::src::shared::tty::*;
use crate::src::shared::tty::{TTY_BLOCK, TTY_FREEZE, TTY_NOCURSOR, TTY_OPENED};
use crate::src::shared::vis::{VIS_CSTYLE, VIS_NOSLASH, VIS_OCTAL};
use crate::src::shared::window::{window, window_mode_entry, winlink};
use crate::src::shared::window::{WINDOW_RESIZE, WINDOW_SIZE_LATEST, WINLINK_ALERTFLAGS};

pub const _PATH_TTY: [::core::ffi::c_char; 9] =
    unsafe { ::core::mem::transmute::<[u8; 9], [::core::ffi::c_char; 9]>(*b"/dev/tty\0") };
pub unsafe fn server_client_how_many() -> u_int {
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut n: u_int = 0;
    n = 0 as u_int;
    let mut registry_c_owner = clients.first();
    c = registry_c_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    while !c.is_null() {
        if !(*c).session_handle().is_none() && !(*c).flags & CLIENT_UNATTACHEDFLAGS as uint64_t != 0
        {
            n = n.wrapping_add(1);
        }
        registry_c_owner =
            clients.next(registry_c_owner.as_ref().expect("current registry client"));
        c = registry_c_owner
            .as_ref()
            .map_or(std::ptr::null_mut(), |owner| owner.get());
    }
    return n;
}
unsafe fn server_client_update_overlay_focus(owner: &ClientRef) {
    use crate::src::session::Session;
    use crate::src::window::Window;
    let Some(session) = owner.attached_session().upgrade() else {
        return;
    };
    let link = session.current_winlink();
    if let Some(window) = link.get_unchecked().window_handle().cloned() {
        window_update_focus(Some(&window));
        window.release(c"client overlay focus");
    }
}

unsafe fn server_client_set_overlay(c_owner: &ClientRef, overlay: Overlay) {
    let c = c_owner.get();
    // Cleanup can install another overlay. Retire it through the same path
    // before publishing this caller's replacement, never by overwriting it.
    while (*c).overlay.current.is_some() {
        server_client_clear_overlay(c_owner);
    }
    (*c).overlay.generation = (*c)
        .overlay
        .generation
        .checked_add(1)
        .expect("overlay generation exhausted");
    if overlay.check.is_none() {
        (*c).tty.flags |= TTY_FREEZE;
    }
    if overlay.mode.is_none() {
        (*c).tty.flags |= TTY_NOCURSOR;
    }
    (*c).overlay.current = Some(overlay);
    server_client_update_overlay_focus(c_owner);
    c_owner.request_redraw(CLIENT_ALLREDRAWFLAGS as u64);
}
unsafe fn server_client_clear_overlay(c_owner: &ClientRef) {
    let c = c_owner.get();
    let Some(overlay) = (*c).overlay.current.take() else {
        return;
    };
    (*c).overlay.generation = (*c)
        .overlay
        .generation
        .checked_add(1)
        .expect("overlay generation exhausted");
    let generation = (*c).overlay.generation;
    overlay.free(c_owner);
    if (*c).overlay.generation == generation {
        (*c).tty.flags &= !(TTY_FREEZE | TTY_NOCURSOR);
    }
    server_client_update_overlay_focus(c_owner);
    c_owner.request_redraw(CLIENT_ALLREDRAWFLAGS as u64);
}
// A callback may clear or replace its overlay. Restore only the generation
// that was borrowed, and discard results from a retired owner.
unsafe fn server_client_overlay_draw(c_owner: &ClientRef) {
    let c = c_owner.get();
    let generation = (*c).overlay.generation;
    let Some(mut callback) = (*c)
        .overlay
        .current
        .as_mut()
        .and_then(|overlay| overlay.draw.take())
    else {
        return;
    };
    callback(c_owner);
    if (*c).overlay.generation == generation {
        if let Some(overlay) = (*c).overlay.current.as_mut() {
            if overlay.draw.is_none() {
                overlay.draw = Some(callback);
            }
        }
    }
}

unsafe fn server_client_overlay_key(c_owner: &ClientRef, event: &mut key_event) -> Option<i32> {
    let c = c_owner.get();
    let generation = (*c).overlay.generation;
    let mut callback = (*c).overlay.current.as_mut()?.key.take()?;
    let result = callback(c_owner, event);
    if (*c).overlay.generation != generation {
        // This event was handled by the retired overlay. In particular, an
        // old close request must not close a newly installed overlay.
        return Some(0);
    }
    if let Some(overlay) = (*c).overlay.current.as_mut() {
        if overlay.key.is_none() {
            overlay.key = Some(callback);
        }
    }
    Some(result)
}

unsafe fn server_client_overlay_mode(c_owner: &ClientRef) -> Option<(ScreenMode, u_int, u_int)> {
    let c = c_owner.get();
    let generation = (*c).overlay.generation;
    let mut callback = (*c).overlay.current.as_mut()?.mode.take()?;
    let result = callback(c_owner);
    if (*c).overlay.generation != generation {
        return None;
    }
    if let Some(overlay) = (*c).overlay.current.as_mut() {
        if overlay.mode.is_none() {
            overlay.mode = Some(callback);
        }
    }
    result
}

unsafe fn server_client_overlay_resize(c_owner: &ClientRef) {
    let c = c_owner.get();
    let generation = (*c).overlay.generation;
    let Some(mut callback) = (*c)
        .overlay
        .current
        .as_mut()
        .and_then(|overlay| overlay.resize.take())
    else {
        return;
    };
    callback(c_owner);
    if (*c).overlay.generation == generation {
        if let Some(overlay) = (*c).overlay.current.as_mut() {
            if overlay.resize.is_none() {
                overlay.resize = Some(callback);
            }
        }
    }
}

unsafe fn server_client_overlay_check(
    c_owner: &ClientRef,
    px: u_int,
    py: u_int,
    nx: u_int,
) -> Option<visible_ranges> {
    let c = c_owner.get();
    let generation = (*c).overlay.generation;
    let mut callback = (*c).overlay.current.as_mut()?.check.take()?;
    let result = callback(c_owner, px, py, nx);
    if (*c).overlay.generation != generation {
        return None;
    }
    if let Some(overlay) = (*c).overlay.current.as_mut() {
        if overlay.check.is_none() {
            overlay.check = Some(callback);
        }
    }
    Some(result)
}

pub unsafe fn server_client_ranges_is_empty(mut r: *mut visible_ranges) -> ::core::ffi::c_int {
    let mut i: u_int = 0;
    i = 0 as u_int;
    while i < (*r).used {
        if (&(*r).storage)[i as usize].nx != 0 as u_int {
            return 0 as ::core::ffi::c_int;
        }
        i = i.wrapping_add(1);
    }
    return 1 as ::core::ffi::c_int;
}
pub unsafe fn server_client_ensure_ranges(mut r: *mut visible_ranges, mut n: u_int) {
    (*r).ensure(n);
}
pub unsafe fn server_client_overlay_range(
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
        (&mut (*r).storage)[0].px = px;
        (&mut (*r).storage)[0].nx = nx;
        (*r).used = 1 as u_int;
        return;
    }
    server_client_ensure_ranges(r, 2 as u_int);
    if px < x {
        (&mut (*r).storage)[0].px = px;
        (&mut (*r).storage)[0].nx = x.wrapping_sub(px);
        if (&mut (*r).storage)[0].nx > nx {
            (&mut (*r).storage)[0].nx = nx;
        }
    } else {
        (&mut (*r).storage)[0].px = 0 as u_int;
        (&mut (*r).storage)[0].nx = 0 as u_int;
    }
    ox = x.wrapping_add(sx);
    if px > ox {
        ox = px;
    }
    onx = px.wrapping_add(nx);
    if onx > ox {
        (&mut (*r).storage)[1].px = ox;
        (&mut (*r).storage)[1].nx = onx.wrapping_sub(ox);
    } else {
        (&mut (*r).storage)[1].px = 0 as u_int;
        (&mut (*r).storage)[1].nx = 0 as u_int;
    }
    (*r).used = 2 as u_int;
}
unsafe fn server_client_check_nested(c: &client) -> ::core::ffi::c_int {
    let mut envent: Option<&environ_entry> = None;
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    envent = environ_find(
        (*c).environ.as_deref().expect("environment"),
        b"TMUX\0" as *const u8 as *const ::core::ffi::c_char,
    );
    if envent.is_none()
        || *envent
            .unwrap()
            .value
            .as_ref()
            .expect("environment value is present")
            .as_ptr() as ::core::ffi::c_int
            == '\0' as i32
    {
        return 0 as ::core::ffi::c_int;
    }
    let mut indexed_pane_owner = window_pane_tree_minmax(&all_window_panes);
    wp = indexed_pane_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    while !wp.is_null() {
        if strcmp(
            &raw mut (*wp).tty as *mut ::core::ffi::c_char,
            ((*c).ttyname)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
        ) == 0 as ::core::ffi::c_int
        {
            return 1 as ::core::ffi::c_int;
        }
        indexed_pane_owner = window_pane_tree_next(&*wp);
        wp = indexed_pane_owner
            .as_ref()
            .map_or(std::ptr::null_mut(), |owner| owner.get());
    }
    return 0 as ::core::ffi::c_int;
}
pub unsafe fn server_client_set_key_table(
    c_owner: &ClientRef,
    mut name: *const ::core::ffi::c_char,
) {
    let mut c = c_owner.get();
    let default_name;
    if name.is_null() {
        default_name = server_client_get_key_table(&*c);
        name = default_name.as_ptr();
    }
    drop((*c).keytable.take());
    (*c).keytable = key_bindings_get_table(std::ffi::CStr::from_ptr(name), 1);
    (*c).keytable
        .as_ref()
        .expect("key table")
        .borrow_mut()
        .activity_time = SystemTime::now();
}
unsafe fn server_client_key_table_activity_diff(c: &client) -> uint64_t {
    c.activity_time
        .duration_since(
            c.keytable
                .as_ref()
                .expect("key table")
                .borrow()
                .activity_time,
        )
        .unwrap_or(Duration::MAX)
        .as_millis()
        .min(u64::MAX as u128) as u64
}
unsafe fn server_client_get_key_table(c: &client) -> std::ffi::CString {
    let Some(session) = c.session_handle() else {
        return c"root".to_owned();
    };
    let name =
        session.with_options_mut(|options| options_get_string(options, c"key-table".as_ptr()));
    if name.as_bytes().is_empty() {
        c"root".to_owned()
    } else {
        name
    }
}
unsafe fn server_client_is_default_key_table(c: &client, table: &key_table) -> ::core::ffi::c_int {
    return (strcmp(
        (table.name).as_ptr().cast_mut(),
        server_client_get_key_table(&*(c)).as_ptr(),
    ) == 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
}
unsafe fn server_client_init_timers(owner: &ClientRef) {
    let c = owner.get();
    let repeat_timer_observer = std::rc::Rc::downgrade(owner);
    (*c).repeat_timer.set(move || unsafe {
        if let Some(owner) = repeat_timer_observer.upgrade() {
            server_client_repeat_timer(&owner);
        }
    });
    let click_timer_observer = std::rc::Rc::downgrade(owner);
    (*c).click_timer.set(move || unsafe {
        if let Some(owner) = click_timer_observer.upgrade() {
            server_client_click_timer(&owner);
        }
    });
    let exit_timer_observer = std::rc::Rc::downgrade(owner);
    (*c).exit_timer.set(move || unsafe {
        if let Some(owner) = exit_timer_observer.upgrade() {
            server_client_exit_timer(&owner);
        }
    });
}
pub unsafe fn server_client_create(mut fd: ::core::ffi::c_int) -> ClientRef {
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut i: u_int = 0;
    setblocking(fd, 0 as ::core::ffi::c_int);
    let mut owner = client::new();
    c = owner.get();
    let peer_observer = std::rc::Rc::downgrade(&owner);
    (*c).peer = proc_add_peer(
        server_proc,
        fd,
        Box::new(move |message| unsafe {
            if let Some(owner) = peer_observer.upgrade() {
                server_client_dispatch(&owner, message);
            }
        }),
    );
    (*c).creation_time = SystemTime::now();
    (*c).activity_time = (*c).creation_time;
    (*c).environ = Some(environ_create());
    (*c).fd = -(1 as ::core::ffi::c_int);
    (*c).out_fd = -(1 as ::core::ffi::c_int);
    (*c).queue = Some(cmdq_new());
    (*c).tty.sx = 80 as u_int;
    (*c).tty.sy = 24 as u_int;
    i = 0 as u_int;
    while i < COLOUR_THEME_COUNT as u_int {
        (*c).theme_colours[i as usize] = 8 as ::core::ffi::c_int;
        i = i.wrapping_add(1);
    }
    (*c).theme = THEME_UNKNOWN;
    status_init(&mut (*c).status, (*c).tty.sx);
    (*c).flags |= CLIENT_FOCUSED as uint64_t;
    (*c).keytable = key_bindings_get_table(c"root", 1);
    server_client_init_timers(&owner);
    (*c).click_wp = -(1 as ::core::ffi::c_int);
    clients.push_back(owner.clone());
    log_debug(format_args!(
        "new client {}",
        log_pointer((c) as *const ::core::ffi::c_void)
    ));
    owner
}
pub unsafe fn server_client_open(owner: &ClientRef) -> Result<(), CString> {
    let c = owner.get();
    let mut ttynam: *const ::core::ffi::c_char = _PATH_TTY.as_ptr();
    if (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
        return Ok(());
    }
    if strcmp(
        ((*c).ttyname)
            .as_ref()
            .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
        ttynam,
    ) == 0 as ::core::ffi::c_int
        || (isatty(STDIN_FILENO) != 0
            && {
                ttynam = ttyname(STDIN_FILENO);
                !ttynam.is_null()
            }
            && strcmp(
                ((*c).ttyname)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
                ttynam,
            ) == 0 as ::core::ffi::c_int
            || isatty(STDOUT_FILENO) != 0
                && {
                    ttynam = ttyname(STDOUT_FILENO);
                    !ttynam.is_null()
                }
                && strcmp(
                    ((*c).ttyname)
                        .as_ref()
                        .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
                    ttynam,
                ) == 0 as ::core::ffi::c_int
            || isatty(STDERR_FILENO) != 0
                && {
                    ttynam = ttyname(STDERR_FILENO);
                    !ttynam.is_null()
                }
                && strcmp(
                    ((*c).ttyname)
                        .as_ref()
                        .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
                    ttynam,
                ) == 0 as ::core::ffi::c_int)
    {
        let mut message = b"can't use ".to_vec();
        message.extend_from_slice(
            (*c).ttyname
                .as_ref()
                .map_or(b"(null)".as_slice(), |name| name.to_bytes()),
        );
        return Err(CString::new(message).expect("terminal name contains no NUL"));
    }
    if (*c).flags & CLIENT_TERMINAL as uint64_t == 0 {
        return Err(c"not a terminal".to_owned());
    }
    tty_open(owner)?;
    server_client_update_theme_colours(
        (c).as_ref()
            .and_then(|model| model.observer.upgrade())
            .as_ref(),
    );
    Ok(())
}
unsafe fn server_client_attached_lost(c_owner: &ClientRef) {
    let mut c = c_owner.get();
    let mut s: Option<SessionRef> = None;
    let mut loop_0: *mut client = ::core::ptr::null_mut::<client>();
    let mut found: *mut client = ::core::ptr::null_mut::<client>();
    log_debug(format_args!(
        "lost attached client {}",
        log_pointer((c) as *const ::core::ffi::c_void)
    ));
    let mut window_cursor = windows_minmax(&windows);
    while let Some(window_owner) = window_cursor.take() {
        if window_owner.is_latest_client(c_owner) {
            found = ::core::ptr::null_mut::<client>();
            let mut registry_loop_0_owner = clients.first();
            loop_0 = registry_loop_0_owner
                .as_ref()
                .map_or(std::ptr::null_mut(), |owner| owner.get());
            while !loop_0.is_null() {
                s = (*loop_0).session_handle();
                if !(loop_0 == c
                    || s.is_none()
                    || (s.as_ref().expect("live session").current_winlink())
                        .get_unchecked()
                        .window_handle()
                        .is_none_or(|current| !std::rc::Rc::ptr_eq(current, &window_owner)))
                {
                    if found.is_null() || (*loop_0).activity_time > (*found).activity_time {
                        found = loop_0;
                    }
                }
                registry_loop_0_owner = clients.next(
                    registry_loop_0_owner
                        .as_ref()
                        .expect("current registry client"),
                );
                loop_0 = registry_loop_0_owner
                    .as_ref()
                    .map_or(std::ptr::null_mut(), |owner| owner.get());
            }
            if !found.is_null() {
                server_client_update_latest(&(*(found)).observer.upgrade().expect("live client"));
            }
        }
        window_cursor = window_owner.next_window();
        crate::src::window::window_remove_ref(window_owner, c"window traversal".as_ptr());
    }
}
unsafe fn server_client_fire_session_changed(c_owner: &ClientRef, old_owner: Option<&SessionRef>) {
    let mut c = c_owner.get();
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        s: Default::default(),
        wl: Default::default(),
        w: Default::default(),
        wp: Default::default(),
        idx: 0,
    };
    let mut ep = event_payload_create();
    cmd_find_from_client(&raw mut fs, Some(c_owner), 0 as ::core::ffi::c_int);
    event_payload_set_target(&mut *ep, &fs);
    event_payload_set_client(&mut *ep, (*(c)).observer.upgrade().expect("live client"));
    if !fs.session_handle().is_none() {
        event_payload_set_session(
            &mut *ep,
            b"session\0" as *const u8 as *const ::core::ffi::c_char,
            fs.session_handle().expect("live session"),
        );
        event_payload_set_session(
            &mut *ep,
            b"new_session\0" as *const u8 as *const ::core::ffi::c_char,
            fs.session_handle().expect("live session"),
        );
    }
    if let Some(old) = old_owner {
        event_payload_set_session(
            &mut *ep,
            b"old_session\0" as *const u8 as *const ::core::ffi::c_char,
            old.clone(),
        );
    }
    if !fs.window_handle().is_none() {
        event_payload_set_window(
            &mut *ep,
            b"window\0" as *const u8 as *const ::core::ffi::c_char,
            std::rc::Rc::clone(&((fs.window_handle().as_ref()).expect("live window"))),
        );
    }
    if fs.winlink_handle().is_alive() {
        event_payload_set_int(
            &mut *ep,
            b"window_index\0" as *const u8 as *const ::core::ffi::c_char,
            (fs.winlink_handle()).get_unchecked().idx,
        );
    } else if fs.idx != -(1 as ::core::ffi::c_int) {
        event_payload_set_int(
            &mut *ep,
            b"window_index\0" as *const u8 as *const ::core::ffi::c_char,
            fs.idx,
        );
    }
    if !fs.pane_handle().is_none() {
        event_payload_set_pane(
            &mut *ep,
            b"pane\0" as *const u8 as *const ::core::ffi::c_char,
            (*(fs
                .pane_handle()
                .as_ref()
                .map_or(std::ptr::null_mut(), |owner| owner.get())))
            .observer
            .upgrade()
            .expect("live window_pane"),
        );
    }
    events_fire(
        b"client-session-changed\0" as *const u8 as *const ::core::ffi::c_char,
        ep,
    );
}
unsafe fn server_client_fire_resized(c_owner: &ClientRef, mut old_sx: u_int, mut old_sy: u_int) {
    let mut c = c_owner.get();
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        s: Default::default(),
        wl: Default::default(),
        w: Default::default(),
        wp: Default::default(),
        idx: 0,
    };
    let mut ep = event_payload_create();
    cmd_find_from_client(&raw mut fs, Some(c_owner), 0 as ::core::ffi::c_int);
    event_payload_set_target(&mut *ep, &fs);
    event_payload_set_client(&mut *ep, (*(c)).observer.upgrade().expect("live client"));
    if !fs.session_handle().is_none() {
        event_payload_set_session(
            &mut *ep,
            b"session\0" as *const u8 as *const ::core::ffi::c_char,
            fs.session_handle().expect("live session"),
        );
    }
    if !fs.window_handle().is_none() {
        event_payload_set_window(
            &mut *ep,
            b"window\0" as *const u8 as *const ::core::ffi::c_char,
            std::rc::Rc::clone(&((fs.window_handle().as_ref()).expect("live window"))),
        );
    }
    if fs.winlink_handle().is_alive() {
        event_payload_set_int(
            &mut *ep,
            b"window_index\0" as *const u8 as *const ::core::ffi::c_char,
            (fs.winlink_handle()).get_unchecked().idx,
        );
    } else if fs.idx != -(1 as ::core::ffi::c_int) {
        event_payload_set_int(
            &mut *ep,
            b"window_index\0" as *const u8 as *const ::core::ffi::c_char,
            fs.idx,
        );
    }
    if !fs.pane_handle().is_none() {
        event_payload_set_pane(
            &mut *ep,
            b"pane\0" as *const u8 as *const ::core::ffi::c_char,
            (*(fs
                .pane_handle()
                .as_ref()
                .map_or(std::ptr::null_mut(), |owner| owner.get())))
            .observer
            .upgrade()
            .expect("live window_pane"),
        );
    }
    event_payload_set_uint(
        &mut *ep,
        b"width\0" as *const u8 as *const ::core::ffi::c_char,
        (*c).tty.sx,
    );
    event_payload_set_uint(
        &mut *ep,
        b"height\0" as *const u8 as *const ::core::ffi::c_char,
        (*c).tty.sy,
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
        b"client-resized\0" as *const u8 as *const ::core::ffi::c_char,
        ep,
    );
}
pub unsafe fn server_client_set_session(c_owner: &ClientRef, s_owner: Option<&SessionRef>) {
    use crate::src::session::Session;
    use crate::src::window::Window;

    let old = c_owner.attached_session().upgrade();
    let c = c_owner.get();
    if let Some(target) = s_owner {
        if old
            .as_ref()
            .is_some_and(|old| !std::rc::Rc::ptr_eq(old, target))
        {
            (*c).last_session = (*c).session.clone();
        }
    } else {
        (*c).last_session = std::rc::Weak::new();
    }
    (*c).session = s_owner.map_or_else(std::rc::Weak::new, std::rc::Rc::downgrade);
    (*c).flags |= CLIENT_FOCUSED as uint64_t;
    if let Some(old) = old.as_ref() {
        let link = old.current_winlink();
        if link.is_alive() {
            if let Some(window) = link.get_unchecked().window_handle().cloned() {
                window_update_focus(Some(&window));
                window.release(c"client old-session focus");
            }
        }
    }
    if let Some(session) = s_owner {
        {
            let link = session.current_winlink();
            let window = link
                .get_unchecked()
                .window_handle()
                .expect("attached session window");
            window.set_latest_client(Some(c_owner));
        }
        recalculate_sizes();
        {
            let link = session.current_winlink();
            let window = link
                .get_unchecked()
                .window_handle()
                .cloned()
                .expect("attached session window");
            window_update_focus(Some(&window));
            window.release(c"client new-session focus");
        }
        session.on_attached();
        tty_update_client_offset(c_owner);
        status_timer_start(c_owner);
        server_client_fire_session_changed(c_owner, old.as_ref());
        c_owner.request_redraw(CLIENT_ALLREDRAWFLAGS as u64);
    }
    server_check_unattached();
    server_update_socket();
}

pub unsafe fn server_client_lost(client_owner: &ClientRef) {
    let c = client_owner.get();
    if (&cfg_client).ptr_eq(&(*c).observer) {
        cfg_client = std::rc::Weak::new();
    }
    (*c).flags |= CLIENT_DEAD as uint64_t;
    server_client_clear_overlay(&(*(c)).observer.upgrade().expect("live client"));
    status_prompt_clear(&(*(c)).observer.upgrade().expect("live client"));
    status_message_clear(&(*c).observer.upgrade().expect("live client"));
    cmdq_abort_file_wait(client_owner);
    let files = client_owner.file_handles();
    crate::src::file::client_files_interrupt(files, EINTR);
    let registry_owner = clients
        .remove(&(*c).observer)
        .expect("registered client owner");
    log_debug(format_args!(
        "lost client {}",
        log_pointer((c) as *const ::core::ffi::c_void)
    ));
    if (*c).flags & CLIENT_ATTACHED as uint64_t != 0 {
        server_client_attached_lost(&(*(c)).observer.upgrade().expect("live client"));
        events_fire_client(
            b"client-detached\0" as *const u8 as *const ::core::ffi::c_char,
            (*(c)).observer.upgrade().expect("live client"),
        );
    }
    if !(*c).name.is_none() && (*c).flags & (CLIENT_CONTROL | CLIENT_TERMINAL) as uint64_t != 0 {
        events_fire_client(
            b"client-closed\0" as *const u8 as *const ::core::ffi::c_char,
            (*(c)).observer.upgrade().expect("live client"),
        );
    }
    if (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
        control_stop(&(*(c)).observer.upgrade().expect("live client"));
    }
    if (*c).flags & CLIENT_TERMINAL as uint64_t != 0 {
        tty_free(&(*c).observer.upgrade().expect("live client"));
    }
    server_client_set_ttyname(&mut *c, None);
    server_client_set_term_name(&mut *c, None);
    server_client_set_term_type(&mut *c, None);
    server_client_clear_term_caps(&mut *c);
    status_free(&mut (*c).status);
    client_owner.cancel_input_requests();
    server_client_replace_title(&mut *c, None);
    server_client_replace_path(&mut *c, None);
    server_client_set_cwd(&mut *c, None);
    server_client_set_exit_session(&mut *c, None);
    server_client_set_exit_message(&mut *c, None);
    (*c).repeat_timer.cancel();
    (*c).click_timer.cancel();
    (*c).exit_timer.cancel();
    if (*c).cycle_timer.is_initialized() {
        (*c).cycle_timer.cancel();
    }
    drop((*c).keytable.take());
    // Callbacks during client loss can set another message after the earlier
    // clear. Preserve the final release point before cancelling its timer.
    server_client_set_message(&mut *c, None);
    if (*c).message_timer.is_initialized() {
        (*c).message_timer.cancel();
    }
    if let Some(prompt) = (*c).prompt.take() {
        prompt_free(&prompt.downgrade());
    }
    format_lost_client(&(*(c)).observer.upgrade().expect("live client"));
    drop((*c).environ.take());
    proc_remove_peer((*c).peer);
    (*c).peer = ::core::ptr::null_mut::<tmuxpeer>();
    if (*c).out_fd != -(1 as ::core::ffi::c_int) {
        close((*c).out_fd);
    }
    if (*c).fd != -(1 as ::core::ffi::c_int) {
        close((*c).fd);
        (*c).fd = -(1 as ::core::ffi::c_int);
    }
    server_client_unref_owned(registry_owner);
    server_add_accept(0 as ::core::ffi::c_int);
    recalculate_sizes();
    server_check_unattached();
    server_update_socket();
}
/// Transfer a client reference to deferred cleanup, preserving the event-loop
/// lifetime required by client teardown callbacks.
pub fn server_client_unref_owned(c: ClientRef) {
    unsafe {
        log_debug(format_args!("unref client {}", log_pointer(c.get().cast())));
    }
    crate::src::shared::rc::release_later(c);
}
unsafe fn server_client_free(c_value: &mut client) {
    let c: *mut client = c_value as *mut _;
    log_debug(format_args!(
        "free client {}",
        log_pointer((c) as *const ::core::ffi::c_void)
    ));
    drop((*c).redraw_scene.take());
    drop((*c).queue.take());
    assert!(
        crate::src::file::client_files_is_empty(&(*c).files),
        "client file index still contains live records at client teardown"
    );
}
pub unsafe fn server_client_suspend(c_owner: &ClientRef) {
    let mut c = c_owner.get();
    let mut s: Option<SessionRef> = (*c).session_handle();
    if s.is_none() || (*c).flags & CLIENT_UNATTACHEDFLAGS as uint64_t != 0 {
        return;
    }
    tty_stop_tty(&(*c).observer.upgrade().expect("live client"));
    (*c).flags |= CLIENT_SUSPENDED as uint64_t;
    proc_send(
        (*c).peer,
        MSG_SUSPEND,
        -(1 as ::core::ffi::c_int),
        ::core::ptr::null::<::core::ffi::c_void>(),
        0 as size_t,
    );
}
pub unsafe fn server_client_detach(c_owner: &ClientRef, mut msgtype: msgtype) {
    let mut c = c_owner.get();
    let mut s: Option<SessionRef> = (*c).session_handle();
    if s.is_none() || (*c).flags & CLIENT_NODETACHFLAGS as uint64_t != 0 {
        return;
    }
    (*c).flags |= CLIENT_EXIT as uint64_t;
    (*c).exit_type = CLIENT_EXIT_DETACH;
    (*c).exit_msgtype = msgtype;
    server_client_set_exit_session(
        &mut *c,
        Some(s.as_ref().expect("live session").name().clone()),
    );
}
pub unsafe fn server_client_exec(c_owner: &ClientRef, mut cmd: *const ::core::ffi::c_char) {
    let mut shell_session_value: Option<std::ffi::CString> = None;

    let mut c = c_owner.get();
    let mut s: Option<SessionRef> = (*c).session_handle();
    let mut shell: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if *cmd as ::core::ffi::c_int == '\0' as i32 {
        return;
    }
    if !s.is_none() {
        shell_session_value = Some(
            s.as_ref()
                .expect("live session")
                .with_options_mut(|options| {
                    options_get_string(
                        options,
                        b"default-shell\0" as *const u8 as *const ::core::ffi::c_char,
                    )
                }),
        );
        shell = shell_session_value
            .as_ref()
            .expect("option snapshot")
            .as_ptr();
    } else {
        shell_session_value = Some(options_get_string(
            global_s_options,
            b"default-shell\0" as *const u8 as *const ::core::ffi::c_char,
        ));
        shell = shell_session_value
            .as_ref()
            .expect("option snapshot")
            .as_ptr();
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
unsafe fn server_client_in_scrollbar_area(
    wp: &window_pane,
    mut px: ::core::ffi::c_int,
    mut py: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut width: u_int = 0;
    let mut pad: u_int = 0;
    let mut total: u_int = 0;
    let mut start: ::core::ffi::c_int = 0;
    let mut end: ::core::ffi::c_int = 0;
    if window_pane_scrollbar_overlay(wp) == 0 {
        return 0 as ::core::ffi::c_int;
    }
    if py < wp.yoff || py >= wp.yoff + wp.sy as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    width = wp.scrollbar_style.width as u_int;
    pad = wp.scrollbar_style.pad as u_int;
    total = width.wrapping_add(pad);
    if total == 0 as u_int || total > wp.sx {
        total = wp.sx;
    }
    if ((wp.window_handle().as_ref()).expect("live window"))
        .scrollbars()
        .position
        == PANE_SCROLLBARS_LEFT
    {
        start = wp.xoff;
        end = wp.xoff + total as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
    } else {
        end = wp.xoff + wp.sx as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
        start = end - total as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
    }
    return (px >= start && px <= end) as ::core::ffi::c_int;
}
unsafe fn server_client_update_scrollbar_hover(
    client_owner: &ClientRef,
    mut type_0: ::core::ffi::c_int,
    mut px: ::core::ffi::c_int,
    mut py: ::core::ffi::c_int,
) {
    let c = client_owner.get();
    let Some(session_owner) = (*c).session.upgrade() else {
        return;
    };
    let window_owner = (session_owner.current_winlink())
        .get_unchecked()
        .window_owner
        .as_ref()
        .expect("hover window")
        .clone();
    let result = (|| {
        let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
        if type_0 != KEYC_TYPE_MOUSEMOVE as ::core::ffi::c_int {
            return;
        }
        let mut cursor = window_owner.next_pane(None);
        while let Some(pane_owner) = cursor {
            wp = pane_owner.get();
            if !(window_pane_is_visible(&pane_owner) == 0) {
                if server_client_in_scrollbar_area(&*wp, px, py) != 0 {
                    window_pane_scrollbar_show(&pane_owner);
                } else {
                    window_pane_scrollbar_start_timer(&pane_owner);
                }
            }
            cursor = window_pane_next(wp.as_ref());
        }
    })();
    crate::src::window::window_remove_ref(
        window_owner,
        c"server_client_update_scrollbar_hover".as_ptr(),
    );
    result
}
unsafe fn server_client_check_mouse_in_pane(
    pane_owner: &std::rc::Rc<std::cell::UnsafeCell<window_pane>>,
    mut px: ::core::ffi::c_int,
    mut py: ::core::ffi::c_int,
    sl_mpos: &mut u_int,
) -> key_code_mouse_location {
    let wp = pane_owner.get();
    let window_owner = (*wp).window_handle().expect("mouse pane window");
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
    pane_status = window_pane_get_pane_status(&*wp);
    sb_overlay = window_pane_scrollbar_overlay(&*wp);
    if window_pane_scrollbar_visible(&*wp) != 0 {
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
    if sb_overlay == 0
        && (((*wp).window_handle().as_ref()).expect("live window"))
            .scrollbars()
            .position
            == PANE_SCROLLBARS_LEFT
    {
        bdr_left -= sb_pad + sb_w;
    }
    if sb_overlay != 0
        && sb_w != 0 as ::core::ffi::c_int
        && py >= (*wp).yoff
        && py < (*wp).yoff + (*wp).sy as ::core::ffi::c_int
        && px >= (*wp).xoff
        && px < (*wp).xoff + (*wp).sx as ::core::ffi::c_int
    {
        if (((*wp).window_handle().as_ref()).expect("live window"))
            .scrollbars()
            .position
            == PANE_SCROLLBARS_LEFT
        {
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
        && ((((*wp).window_handle().as_ref()).expect("live window"))
            .scrollbars()
            .position
            == PANE_SCROLLBARS_RIGHT
            && px < (*wp).xoff + (*wp).sx as ::core::ffi::c_int + sb_pad + sb_w
            || (((*wp).window_handle().as_ref()).expect("live window"))
                .scrollbars()
                .position
                == PANE_SCROLLBARS_LEFT
                && px < (*wp).xoff + (*wp).sx as ::core::ffi::c_int - sb_pad - sb_w)
    {
        if (((*wp).window_handle().as_ref()).expect("live window"))
            .scrollbars()
            .position
            == PANE_SCROLLBARS_RIGHT
            && (px >= (*wp).xoff + (*wp).sx as ::core::ffi::c_int + sb_pad
                && px < (*wp).xoff + (*wp).sx as ::core::ffi::c_int + sb_pad + sb_w)
            || (((*wp).window_handle().as_ref()).expect("live window"))
                .scrollbars()
                .position
                == PANE_SCROLLBARS_LEFT
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
        } else if window_pane_is_floating(&*wp) != 0
            && window_pane_get_pane_lines(&*wp) as ::core::ffi::c_uint
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
        let mut cursor = window_owner.next_pane(None);
        while let Some(border_pane) = cursor {
            fwp = border_pane.get();
            if !(window_pane_is_visible(&border_pane) == 0) {
                if !(window_pane_is_floating(&*fwp) != 0
                    && window_pane_get_pane_lines(&*fwp) as ::core::ffi::c_uint
                        == PANE_LINES_NONE as ::core::ffi::c_int as ::core::ffi::c_uint)
                {
                    if window_pane_scrollbar_reserve(&*fwp) != 0 {
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
                    if (((*wp).window_handle().as_ref()).expect("live window"))
                        .scrollbars()
                        .position
                        == PANE_SCROLLBARS_LEFT
                    {
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
                        if window_pane_is_floating(&*wp) != 0 {
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
            cursor = window_pane_next(fwp.as_ref());
        }
        if !fwp.is_null() {
            return KEYC_MOUSE_LOCATION_BORDER;
        }
    }
    return KEYC_MOUSE_LOCATION_NOWHERE;
}
unsafe fn server_client_check_mouse(
    client_owner: &ClientRef,
    mut event: *mut key_event,
) -> key_code {
    let c = client_owner.get();
    let mut selected_pane = None;
    let mut last_pane = None;
    let mut m: *mut mouse_event = &raw mut (*event).m;
    let Some(session_owner) = (*c).session.upgrade() else {
        return KEYC_UNKNOWN;
    };
    let s = Some(session_owner.clone());
    let window_owner = (s.as_ref().expect("live session").current_winlink())
        .get_unchecked()
        .window_owner
        .as_ref()
        .expect("mouse window")
        .clone();
    let result = (|| {
        let mut current_block: u64;
        let mut fwl: refbox::Weak<winlink> = refbox::Weak::new();
        let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
        let mut x: u_int = 0;
        let mut y: u_int = 0;
        let mut px: u_int = 0;
        let mut py: u_int = 0;
        let mut n: u_int = 0;
        let mut sl_mpos: u_int = 0 as u_int;
        let mut b: u_int = 0;
        let mut bn: u_int = 0;
        let mut ignore: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut modal_drag: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut key: key_code = 0;
        let mut type_0: key_code_type = KEYC_TYPE_NOTYPE;
        let mut loc: key_code_mouse_location = KEYC_MOUSE_LOCATION_NOWHERE;
        log_debug(format_args!(
            "{} mouse {:02x} at {},{} (last {},{}) ({})",
            log_cstr(
                (((*c).name)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                    as *const _
            ),
            ((*m).b) as u32,
            ((*m).x) as u32,
            ((*m).y) as u32,
            ((*m).lx) as u32,
            ((*m).ly) as u32,
            ((*c).tty.mouse_drag_flag) as i32
        ));
        if (*c).tty.mouse_last_pane != -(1 as ::core::ffi::c_int) {
            last_pane = window_pane_find_by_id((*c).tty.mouse_last_pane as u_int);
            if let Some(pane) = last_pane.as_ref() {
                log_debug(format_args!(
                    "{} mouse last pane %{}",
                    log_cstr(
                        (((*c).name)
                            .as_ref()
                            .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                            as *const _
                    ),
                    ((*pane.get()).id) as u32
                ));
            }
        }
        if (*event).key == KEYC_DOUBLECLICK as ::core::ffi::c_ulong as key_code {
            type_0 = KEYC_TYPE_DOUBLECLICK;
            x = (*m).x;
            y = (*m).y;
            b = (*m).b;
            ignore = 1 as ::core::ffi::c_int;
            log_debug(format_args!(
                "double-click at {},{}",
                (x) as u32,
                (y) as u32
            ));
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
            log_debug(format_args!("move at {},{}", (x) as u32, (y) as u32));
        } else if (*m).b & MOUSE_MASK_DRAG as u_int != 0 {
            type_0 = KEYC_TYPE_MOUSEDRAG;
            if (*c).tty.mouse_drag_flag != 0 {
                x = (*m).x;
                y = (*m).y;
                b = (*m).b;
                if x == (*m).lx && y == (*m).ly {
                    return KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code;
                }
                log_debug(format_args!("drag update at {},{}", (x) as u32, (y) as u32));
            } else {
                x = (*m).lx;
                y = (*m).ly;
                b = (*m).lb;
                log_debug(format_args!("drag start at {},{}", (x) as u32, (y) as u32));
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
            log_debug(format_args!("wheel at {},{}", (x) as u32, (y) as u32));
        } else if (*m).b & MOUSE_MASK_BUTTONS as u_int == 3 as u_int {
            type_0 = KEYC_TYPE_MOUSEUP;
            x = (*m).x;
            y = (*m).y;
            b = (*m).lb;
            if (*m).sgr_type == 'm' as i32 as u_int {
                b = (*m).sgr_b;
            }
            log_debug(format_args!("up at {},{}", (x) as u32, (y) as u32));
        } else {
            if (*c).flags & CLIENT_DOUBLECLICK as uint64_t != 0 {
                (*c).click_timer.cancel();
                (*c).flags &= !CLIENT_DOUBLECLICK as uint64_t;
                type_0 = KEYC_TYPE_SECONDCLICK;
                x = (*m).x;
                y = (*m).y;
                b = (*m).b;
                log_debug(format_args!(
                    "second-click at {},{}",
                    (x) as u32,
                    (y) as u32
                ));
                (*c).flags |= CLIENT_TRIPLECLICK as uint64_t;
                current_block = 16799951812150840583;
            } else if (*c).flags & CLIENT_TRIPLECLICK as uint64_t != 0 {
                (*c).click_timer.cancel();
                (*c).flags &= !CLIENT_TRIPLECLICK as uint64_t;
                type_0 = KEYC_TYPE_TRIPLECLICK;
                x = (*m).x;
                y = (*m).y;
                b = (*m).b;
                log_debug(format_args!(
                    "triple-click at {},{}",
                    (x) as u32,
                    (y) as u32
                ));
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
                        log_debug(format_args!("down at {},{}", (x) as u32, (y) as u32));
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
        (*m).s = s.as_ref().expect("live session").id() as ::core::ffi::c_int;
        (*m).w = -(1 as ::core::ffi::c_int);
        (*m).wp = -(1 as ::core::ffi::c_int);
        (*m).ignore = ignore;
        (*m).statusat = status_at_line(&(*c).observer.upgrade().expect("live client"));
        (*m).statuslines = status_line_size(&(*c).observer.upgrade().expect("live client"));
        if (*m).statusat != -(1 as ::core::ffi::c_int)
            && y >= (*m).statusat as u_int
            && y < ((*m).statusat as u_int).wrapping_add((*m).statuslines)
        {
            if let Some(sr) = status_get_range(
                &(*c).observer.upgrade().expect("live client"),
                x,
                y.wrapping_sub((*m).statusat as u_int),
            ) {
                match sr.type_0 as ::core::ffi::c_uint {
                    0 => return KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code,
                    1 => {
                        log_debug(format_args!("mouse range: left"));
                        loc = KEYC_MOUSE_LOCATION_STATUS_LEFT;
                    }
                    2 => {
                        log_debug(format_args!("mouse range: right"));
                        loc = KEYC_MOUSE_LOCATION_STATUS_RIGHT;
                    }
                    3 => {
                        if window_pane_find_by_id(sr.argument).is_none() {
                            return KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code;
                        }
                        (*m).wp = sr.argument as ::core::ffi::c_int;
                        log_debug(format_args!("mouse range: pane %{}", ((*m).wp) as u32));
                        loc = KEYC_MOUSE_LOCATION_STATUS;
                    }
                    4 => {
                        fwl = s.as_ref().expect("live session").with_winlinks(|links| {
                            winlink_find_by_index(links, sr.argument as ::core::ffi::c_int)
                        });
                        if !fwl.is_alive() {
                            return KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code;
                        }
                        (*m).w = ((fwl.get_unchecked().window_handle().as_ref())
                            .expect("live window"))
                        .id() as ::core::ffi::c_int;
                        log_debug(format_args!("mouse range: window @{}", ((*m).w) as u32));
                        loc = KEYC_MOUSE_LOCATION_STATUS;
                    }
                    5 => {
                        if session_find_by_id(sr.argument).is_none() {
                            return KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code;
                        }
                        (*m).s = sr.argument as ::core::ffi::c_int;
                        log_debug(format_args!("mouse range: session ${}", ((*m).s) as u32));
                        loc = KEYC_MOUSE_LOCATION_STATUS;
                    }
                    6 => {
                        log_debug(format_args!("mouse range: user"));
                        loc = KEYC_MOUSE_LOCATION_STATUS;
                    }
                    7 => {
                        n = sr.argument;
                        log_debug(format_args!("mouse range: control {}", (n) as u32));
                        loc = (KEYC_MOUSE_LOCATION_CONTROL0 as ::core::ffi::c_int as u_int)
                            .wrapping_add(n)
                            as key_code_mouse_location;
                    }
                    _ => {}
                }
            } else {
                loc = KEYC_MOUSE_LOCATION_STATUS_DEFAULT;
            }
        }
        if loc as ::core::ffi::c_uint
            == KEYC_MOUSE_LOCATION_NOWHERE as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*c).tty.mouse_scrolling_flag != 0
        {
            if let Some(pane) = last_pane.as_ref() {
                loc = KEYC_MOUSE_LOCATION_SCROLLBAR_SLIDER;
                (*m).wp = (*pane.get()).id as ::core::ffi::c_int;
                (*m).w = (((*pane.get()).window_handle().as_ref()).expect("live window")).id()
                    as ::core::ffi::c_int;
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
            let view = tty_window_offset(&(*c).tty);
            (*m).ox = view.ox;
            (*m).oy = view.oy;
            let (sx, sy) = (view.sx, view.sy);
            log_debug(format_args!(
                "mouse window @{} at {},{} ({}x{})",
                ((window_owner).id()) as u32,
                ((*m).ox) as u32,
                ((*m).oy) as u32,
                (sx) as u32,
                (sy) as u32
            ));
            if px > sx || py > sy {
                server_client_update_scrollbar_hover(
                    client_owner,
                    type_0 as ::core::ffi::c_int,
                    -(1 as ::core::ffi::c_int),
                    -(1 as ::core::ffi::c_int),
                );
                return KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code;
            }
            px = px.wrapping_add((*m).ox);
            py = py.wrapping_add((*m).oy);
            let modal_owner = window_owner.modal_pane();
            if let Some(modal) =
                modal_owner.filter(|owner| window_pane_contains(owner, px, py) == 0)
            {
                if last_pane
                    .as_ref()
                    .is_some_and(|last| std::rc::Rc::ptr_eq(&modal, last))
                    && (*c).tty.mouse_drag_flag != 0 as ::core::ffi::c_int
                    && (type_0 as ::core::ffi::c_uint
                        == KEYC_TYPE_MOUSEDRAG as ::core::ffi::c_int as ::core::ffi::c_uint
                        || type_0 as ::core::ffi::c_uint
                            == KEYC_TYPE_MOUSEUP as ::core::ffi::c_int as ::core::ffi::c_uint)
                {
                    modal_drag = 1 as ::core::ffi::c_int;
                    selected_pane = last_pane.clone();
                    wp = selected_pane.as_ref().expect("drag pane").get();
                    loc = KEYC_MOUSE_LOCATION_PANE;
                    (*m).wp = (*wp).id as ::core::ffi::c_int;
                    (*m).w = (((*wp).window_handle().as_ref()).expect("live window")).id()
                        as ::core::ffi::c_int;
                } else {
                    server_client_update_scrollbar_hover(
                        client_owner,
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
                    if (*modal.get()).flags & PANE_CLOSEONCLICK != 0
                        && (type_0 as ::core::ffi::c_uint
                            == KEYC_TYPE_MOUSEDOWN as ::core::ffi::c_int as ::core::ffi::c_uint
                            || type_0 as ::core::ffi::c_uint
                                == KEYC_TYPE_SECONDCLICK as ::core::ffi::c_int
                                    as ::core::ffi::c_uint
                            || type_0 as ::core::ffi::c_uint
                                == KEYC_TYPE_TRIPLECLICK as ::core::ffi::c_int
                                    as ::core::ffi::c_uint)
                    {
                        server_kill_pane(&modal);
                    }
                    return KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code;
                }
            }
            server_client_update_scrollbar_hover(
                client_owner,
                type_0 as ::core::ffi::c_int,
                px as ::core::ffi::c_int,
                py as ::core::ffi::c_int,
            );
            if !(modal_drag != 0) {
                if type_0 as ::core::ffi::c_uint
                    == KEYC_TYPE_MOUSEDRAG as ::core::ffi::c_int as ::core::ffi::c_uint
                    && last_pane.is_some()
                {
                    selected_pane = last_pane.clone();
                    wp = selected_pane.as_ref().expect("drag pane").get();
                } else {
                    let hit_window_owner = &window_owner;
                    selected_pane = window_get_active_at(hit_window_owner, px, py);
                    wp = selected_pane
                        .as_ref()
                        .map_or(std::ptr::null_mut(), |owner| owner.get());
                }
            }
            if wp.is_null() {
                loc = KEYC_MOUSE_LOCATION_EMPTY;
                (*m).w = (window_owner).id() as ::core::ffi::c_int;
                log_debug(format_args!(
                    "mouse {},{} on empty area",
                    (x) as u32,
                    (y) as u32
                ));
            } else {
                if modal_drag == 0 {
                    let pane_owner = selected_pane.as_ref().expect("mouse target pane");
                    loc = server_client_check_mouse_in_pane(
                        pane_owner,
                        px as ::core::ffi::c_int,
                        py as ::core::ffi::c_int,
                        &mut sl_mpos,
                    );
                }
                if loc as ::core::ffi::c_uint
                    == KEYC_MOUSE_LOCATION_PANE as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    log_debug(format_args!(
                        "mouse {},{} on pane %{}",
                        (x) as u32,
                        (y) as u32,
                        ((*wp).id) as u32
                    ));
                } else if loc as ::core::ffi::c_uint
                    == KEYC_MOUSE_LOCATION_BORDER as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    if let Some(range) = window_pane_status_get_range(
                        selected_pane.as_ref().expect("mouse border pane"),
                        px,
                        py,
                    ) {
                        n = range.argument;
                        loc = (KEYC_MOUSE_LOCATION_CONTROL0 as ::core::ffi::c_int as u_int)
                            .wrapping_add(n)
                            as key_code_mouse_location;
                    }
                    log_debug(format_args!("mouse on pane %{} border", ((*wp).id) as u32));
                } else if loc as ::core::ffi::c_uint
                    == KEYC_MOUSE_LOCATION_SCROLLBAR_UP as ::core::ffi::c_int as ::core::ffi::c_uint
                    || loc as ::core::ffi::c_uint
                        == KEYC_MOUSE_LOCATION_SCROLLBAR_SLIDER as ::core::ffi::c_int
                            as ::core::ffi::c_uint
                    || loc as ::core::ffi::c_uint
                        == KEYC_MOUSE_LOCATION_SCROLLBAR_DOWN as ::core::ffi::c_int
                            as ::core::ffi::c_uint
                {
                    log_debug(format_args!(
                        "mouse on pane %{} scrollbar",
                        ((*wp).id) as u32
                    ));
                }
                (*m).wp = (*wp).id as ::core::ffi::c_int;
                (*m).w = (((*wp).window_handle().as_ref()).expect("live window")).id()
                    as ::core::ffi::c_int;
            }
        } else {
            server_client_update_scrollbar_hover(
                client_owner,
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
                log_debug(format_args!(
                    "click sequence reset at {},{}",
                    (x) as u32,
                    (y) as u32
                ));
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
                log_debug(format_args!("click timer started"));
                let timeout = Duration::from_millis(KEYC_CLICK_TIMEOUT as u64);
                (*c).click_timer.cancel();
                (*c).click_timer.arm(timeout).expect("arm timer");
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
            if let Some(release) = (*c).tty.mouse_drag_release.take() {
                release(&mut *m);
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
                && wp
                    != (window_owner)
                        .active_pane()
                        .as_ref()
                        .map_or(std::ptr::null_mut(), |owner| owner.get())
                && s.as_ref()
                    .expect("live session")
                    .with_options_mut(|options| {
                        options_get_number(
                            options,
                            b"focus-follows-mouse\0" as *const u8 as *const ::core::ffi::c_char,
                        )
                    })
                    != 0
            {
                window_redraw_active_switch(
                    &std::rc::Rc::clone(&(window_owner)),
                    (wp).as_ref()
                        .and_then(|model| model.observer.upgrade())
                        .as_ref(),
                );
                window_set_active_pane(
                    &std::rc::Rc::clone(&(window_owner)),
                    &(*(wp)).observer.upgrade().expect("live window_pane"),
                    1 as ::core::ffi::c_int,
                );
                server_redraw_window_borders(&(window_owner));
                server_status_window(&(window_owner));
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
            if last_pane.is_none() {
                let hit_window_owner = &window_owner;
                selected_pane = window_get_active_at(hit_window_owner, px, py);
                wp = selected_pane
                    .as_ref()
                    .map_or(std::ptr::null_mut(), |owner| owner.get());
                last_pane = selected_pane.clone();
                if !wp.is_null() {
                    (*c).tty.mouse_last_pane = (*wp).id as ::core::ffi::c_int;
                }
            }
            if (*c).tty.mouse_scrolling_flag == 0 as ::core::ffi::c_int
                && loc as ::core::ffi::c_uint
                    == KEYC_MOUSE_LOCATION_SCROLLBAR_SLIDER as ::core::ffi::c_int
                        as ::core::ffi::c_uint
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
            log_debug(format_args!(
                "mouse key is {}",
                log_cstr((key_string.as_ptr()) as *const _)
            ));
        }
        return key;
    })();
    crate::src::window::window_remove_ref(window_owner, c"server_client_check_mouse".as_ptr());
    result
}
pub unsafe fn server_client_update_theme_colours(c_owner: Option<&ClientRef>) {
    let mut c = c_owner.map_or(std::ptr::null_mut(), |owner| owner.get());
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut name: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut value: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut theme: client_theme = THEME_UNKNOWN;
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
    let mut ft_owner = format_create(c_owner, None, FORMAT_NONE, FORMAT_NOJOBS);
    ft = &raw mut *ft_owner;
    format_defaults(ft, c_owner, None, (refbox::Weak::new()).clone(), None);
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
            let value = options_get_string(global_options, name);
            let expanded = format_expand_cstring(ft, value.as_ptr());
            colour = colour_parse_cstr(expanded.as_c_str()).unwrap_or(-1);
            if !(colour == -(1 as ::core::ffi::c_int) || colour & COLOUR_FLAG_THEME != 0) {
                (*c).theme_colours[i as usize] = colour;
            }
        }
        i = i.wrapping_add(1);
    }
    format_free(ft_owner);
}
unsafe fn server_client_is_bracket_paste(c: &mut client, mut key: key_code) -> ::core::ffi::c_int {
    if key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
        == KEYC_PASTE_START as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
    {
        (*c).flags = ((*c).flags as ::core::ffi::c_ulonglong | CLIENT_BRACKETPASTING) as uint64_t;
        (*c).paste_time = current_time;
        log_debug(format_args!(
            "{}: bracket paste on",
            log_cstr(
                (((*c).name)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                    as *const _
            )
        ));
        return 0 as ::core::ffi::c_int;
    }
    if key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
        == KEYC_PASTE_END as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
    {
        (*c).flags = ((*c).flags as ::core::ffi::c_ulonglong & !CLIENT_BRACKETPASTING) as uint64_t;
        log_debug(format_args!(
            "{}: bracket paste off",
            log_cstr(
                (((*c).name)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                    as *const _
            )
        ));
        return 0 as ::core::ffi::c_int;
    }
    return ((*c).flags as ::core::ffi::c_ulonglong & CLIENT_BRACKETPASTING != 0)
        as ::core::ffi::c_int;
}
unsafe fn server_client_is_assume_paste(c: &mut client) -> ::core::ffi::c_int {
    let mut s: Option<SessionRef> = (*c).session_handle();
    let mut t: ::core::ffi::c_int = 0;
    if (*c).flags as ::core::ffi::c_ulonglong & CLIENT_BRACKETPASTING != 0 {
        return 0 as ::core::ffi::c_int;
    }
    t = s
        .as_ref()
        .expect("live session")
        .with_options_mut(|options| {
            options_get_number(
                options,
                b"assume-paste-time\0" as *const u8 as *const ::core::ffi::c_char,
            )
        }) as ::core::ffi::c_int;
    if t == 0 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    if tty_term_has(
        tty_term_owner_ptr(&(*c).tty.term).map_or(std::ptr::null(), |term| term),
        TTYC_ENBP,
    ) != 0
    {
        return 0 as ::core::ffi::c_int;
    }
    let elapsed = c.activity_time.duration_since(c.last_activity_time);
    if elapsed
        .is_ok_and(|elapsed| elapsed.as_secs() == 0 && elapsed < Duration::from_millis(t as u64))
    {
        if (*c).flags as ::core::ffi::c_ulonglong & CLIENT_ASSUMEPASTING != 0 {
            return 1 as ::core::ffi::c_int;
        }
        (*c).flags = ((*c).flags as ::core::ffi::c_ulonglong | CLIENT_ASSUMEPASTING) as uint64_t;
        (*c).paste_time = current_time;
        log_debug(format_args!(
            "{}: assume paste on",
            log_cstr(
                (((*c).name)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                    as *const _
            )
        ));
        return 0 as ::core::ffi::c_int;
    }
    if (*c).flags as ::core::ffi::c_ulonglong & CLIENT_ASSUMEPASTING != 0 {
        (*c).flags = ((*c).flags as ::core::ffi::c_ulonglong & !CLIENT_ASSUMEPASTING) as uint64_t;
        log_debug(format_args!(
            "{}: assume paste off",
            log_cstr(
                (((*c).name)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                    as *const _
            )
        ));
    }
    return 0 as ::core::ffi::c_int;
}
unsafe fn server_client_update_latest(c_owner: &ClientRef) {
    use crate::src::session::Session;
    use crate::src::window::Window;

    let Some(session) = c_owner.attached_session().upgrade() else {
        return;
    };
    let link = session.current_winlink();
    let window = link
        .get_unchecked()
        .window_handle()
        .cloned()
        .expect("attached session window");
    if !window.set_latest_client(Some(c_owner)) {
        window.release(c"client latest window");
        return;
    }
    let use_latest = window.with_options_mut(|options| {
        options_get_number(
            (options as *const crate::src::shared::options::options).cast_mut(),
            c"window-size".as_ptr(),
        ) == WINDOW_SIZE_LATEST as i64
    });
    if use_latest {
        recalculate_size(&window, 0);
    }
    window.release(c"client latest window");
    events_fire_client(c"client-active".as_ptr(), c_owner.clone());
}

unsafe fn server_client_repeat_time(c: &client, bd: &KeyBindingCommand) -> u_int {
    let mut s: Option<SessionRef> = (*c).session_handle();
    let mut repeat: u_int = 0;
    let mut initial: u_int = 0;
    if !bd.flags & KEY_BINDING_REPEAT != 0 {
        return 0 as u_int;
    }
    repeat = s
        .as_ref()
        .expect("live session")
        .with_options_mut(|options| {
            options_get_number(
                options,
                b"repeat-time\0" as *const u8 as *const ::core::ffi::c_char,
            )
        }) as u_int;
    if repeat == 0 as u_int {
        return 0 as u_int;
    }
    if !(*c).flags & CLIENT_REPEAT as uint64_t != 0 || bd.key != (*c).last_key {
        initial = s
            .as_ref()
            .expect("live session")
            .with_options_mut(|options| {
                options_get_number(
                    options,
                    b"initial-repeat-time\0" as *const u8 as *const ::core::ffi::c_char,
                )
            }) as u_int;
        if initial != 0 as u_int {
            repeat = initial;
        }
    }
    return repeat;
}
unsafe fn server_client_handle_dead_key(
    pane_owner: Option<&std::rc::Rc<std::cell::UnsafeCell<window_pane>>>,
    mut key: key_code,
) -> ::core::ffi::c_int {
    let Some(pane_owner) = pane_owner else {
        return 0;
    };
    let wp = pane_owner.get();
    let mut remain_on_exit: ::core::ffi::c_int = 0;
    if !(*wp).flags & PANE_EXITED != 0
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
        options_owner_ptr(&mut (*wp).options).map_or(std::ptr::null_mut(), |options| options),
        b"remain-on-exit\0" as *const u8 as *const ::core::ffi::c_char,
    ) as ::core::ffi::c_int;
    if remain_on_exit != 3 as ::core::ffi::c_int && remain_on_exit != 4 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    options_set_number(
        options_owner_ptr(&mut (*wp).options).map_or(std::ptr::null_mut(), |options| options),
        b"remain-on-exit\0" as *const u8 as *const ::core::ffi::c_char,
        0 as ::core::ffi::c_longlong,
    );
    server_destroy_pane(pane_owner, 0);
    return 1 as ::core::ffi::c_int;
}
unsafe fn server_client_key_callback(
    item_handle: &std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>,
    mut owned: QueuedKeyEvent,
) -> cmd_retval {
    let item = item_handle.get();
    let mut current_block: u64;
    // The queued callback owns the event and its bytes until this call returns.
    let mut event: *mut key_event = &raw mut *owned.0;
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut key: key_code = (*event).key;
    let mut m: *mut mouse_event = &raw mut (*event).m;
    let mut s: Option<SessionRef> = None;
    let mut wl: refbox::Weak<winlink> = refbox::Weak::new();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut wme: refbox::Weak<window_mode_entry> = refbox::Weak::new();
    let mut table: std::rc::Rc<std::cell::RefCell<key_table>>;
    let mut table_owner;
    let mut first: std::rc::Rc<std::cell::RefCell<key_table>>;
    let mut bd: Option<KeyBindingCommand> = None;
    let mut repeat: u_int = 0;
    let mut flags: uint64_t = 0;
    let mut prefix_delay: uint64_t = 0;
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        s: Default::default(),
        wl: Default::default(),
        w: Default::default(),
        wp: Default::default(),
        idx: 0,
    };
    let mut key0: key_code = 0;
    let mut prefix: key_code = 0;
    let mut prefix2: key_code = 0;
    let c_owner = (*event).resolve_client(|| cmdq_get_client((item).as_ref()));
    let Some(c_owner) = c_owner else {
        return CMD_RETURN_NORMAL;
    };
    c = c_owner.get();
    s = (*c).session_handle();
    if !(s.is_none() || (*c).flags & CLIENT_UNATTACHEDFLAGS as uint64_t != 0) {
        wl = s.as_ref().expect("live session").current_winlink();
        (*c).last_activity_time = (*c).activity_time;
        (*c).activity_time = SystemTime::now();
        s.as_ref()
            .expect("live session")
            .update_activity(Some((*c).activity_time));
        (*m).valid = 0 as ::core::ffi::c_int;
        if key == KEYC_MOUSE as ::core::ffi::c_ulong as key_code
            || key == KEYC_DOUBLECLICK as ::core::ffi::c_ulong as key_code
        {
            if (*c).flags & CLIENT_READONLY as uint64_t != 0 {
                current_block = 1578459965781631232;
            } else {
                key = server_client_check_mouse(&c_owner, event);
                if key == KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code {
                    current_block = 1578459965781631232;
                } else {
                    (*m).valid = 1 as ::core::ffi::c_int;
                    (*m).key = key;
                    if key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
                        == KEYC_DRAGGING as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
                    {
                        if let Some(mut update) = (*c).tty.mouse_drag_update.take() {
                            update(&mut *m);
                            if (*c).tty.mouse_drag_update.is_none() {
                                (*c).tty.mouse_drag_update = Some(update);
                            }
                        }
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
                    || cmd_find_from_mouse(&raw mut fs, m) != 0 as ::core::ffi::c_int
                {
                    cmd_find_from_client(
                        &raw mut fs,
                        (c).as_ref()
                            .and_then(|model| model.observer.upgrade())
                            .as_ref(),
                        0 as ::core::ffi::c_int,
                    );
                }
                wp = fs
                    .pane_handle()
                    .as_ref()
                    .map_or(std::ptr::null_mut(), |owner| owner.get());
                if (key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
                    == KEYC_MOUSE as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
                    || key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                        >= (KEYC_TYPE_MOUSEMOVE as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                            << 32 as ::core::ffi::c_int
                        && key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                            <= (KEYC_TYPE_TRIPLECLICK as ::core::ffi::c_int
                                as ::core::ffi::c_ulonglong)
                                << 32 as ::core::ffi::c_int)
                    && s.as_ref()
                        .expect("live session")
                        .with_options_mut(|options| {
                            options_get_number(
                                options,
                                b"mouse\0" as *const u8 as *const ::core::ffi::c_char,
                            )
                        })
                        == 0
                {
                    current_block = 15469183920764600035;
                } else {
                    if server_client_is_bracket_paste(&mut *(c), key) != 0 {
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
                        && server_client_is_assume_paste(&mut *(c)) != 0
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
                        && (*wp).modes.is_empty()
                    {
                        current_block = 15469183920764600035;
                    } else if key == KEYC_FOCUS_IN as ::core::ffi::c_ulong as key_code
                        || key == KEYC_FOCUS_OUT as ::core::ffi::c_ulong as key_code
                    {
                        current_block = 15469183920764600035;
                    } else {
                        if server_client_is_default_key_table(
                            &*(c),
                            &(*c).keytable.as_ref().expect("key table").borrow(),
                        ) != 0
                            && !wp.is_null()
                            && {
                                wme = (*wp).active_mode_entry();
                                wme.is_alive()
                            }
                            && (*wme.get_unchecked().mode).key_table.is_some()
                        {
                            table_owner = key_bindings_get_table(
                                std::ffi::CStr::from_ptr((*wme.get_unchecked().mode)
                                    .key_table
                                    .expect("non-null function pointer")(
                                    wme
                                )),
                                1 as ::core::ffi::c_int,
                            );
                        } else {
                            table_owner = (*c).keytable.clone();
                        }
                        table = table_owner.as_ref().expect("key table").clone();
                        first = table.clone();
                        '_table_changed: loop {
                            prefix = s
                                .as_ref()
                                .expect("live session")
                                .with_options_mut(|options| {
                                    options_get_number(
                                        options,
                                        b"prefix\0" as *const u8 as *const ::core::ffi::c_char,
                                    )
                                }) as key_code;
                            prefix2 =
                                s.as_ref()
                                    .expect("live session")
                                    .with_options_mut(|options| {
                                        options_get_number(
                                            options,
                                            b"prefix2\0" as *const u8 as *const ::core::ffi::c_char,
                                        )
                                    }) as key_code;
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
                                    (table.borrow().name).as_ptr().cast_mut(),
                                    b"prefix\0" as *const u8 as *const ::core::ffi::c_char,
                                ) != 0 as ::core::ffi::c_int
                            {
                                server_client_set_key_table(
                                    &(*(c)).observer.upgrade().expect("live client"),
                                    b"prefix\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                (*c).flags |= CLIENT_REDRAWSTATUS as u64;
                                current_block = 1578459965781631232;
                                break;
                            } else {
                                flags = (*c).flags;
                                loop {
                                    if wp.is_null() {
                                        log_debug(format_args!(
                                            "key table {} (no pane)",
                                            log_cstr(
                                                ((table.borrow().name).as_ptr().cast_mut())
                                                    as *const _
                                            )
                                        ));
                                    } else {
                                        log_debug(format_args!(
                                            "key table {} (pane %{})",
                                            log_cstr(
                                                ((table.borrow().name).as_ptr().cast_mut())
                                                    as *const _
                                            ),
                                            ((*wp).id) as u32
                                        ));
                                    }
                                    if (*c).flags & CLIENT_REPEAT as uint64_t != 0 {
                                        log_debug(format_args!("currently repeating"));
                                    }
                                    bd = key_bindings_get(&table.borrow(), key0)
                                        .map(|bd| bd.command());
                                    prefix_delay = options_get_number(
                                        global_options,
                                        b"prefix-timeout\0" as *const u8
                                            as *const ::core::ffi::c_char,
                                    )
                                        as uint64_t;
                                    if prefix_delay > 0 as uint64_t
                                        && strcmp(
                                            (table.borrow().name).as_ptr().cast_mut(),
                                            b"prefix\0" as *const u8 as *const ::core::ffi::c_char,
                                        ) == 0 as ::core::ffi::c_int
                                        && server_client_key_table_activity_diff(&*(c))
                                            > prefix_delay
                                    {
                                        if bd.is_some()
                                            && (*c).flags & CLIENT_REPEAT as uint64_t != 0
                                            && bd.as_ref().unwrap().flags & KEY_BINDING_REPEAT != 0
                                        {
                                            log_debug(format_args!(
                                                "prefix timeout ignored, repeat is active"
                                            ));
                                        } else {
                                            log_debug(format_args!("prefix timeout exceeded"));
                                            server_client_set_key_table(
                                                &(*(c)).observer.upgrade().expect("live client"),
                                                ::core::ptr::null::<::core::ffi::c_char>(),
                                            );
                                            table_owner = (*c).keytable.clone();
                                            table =
                                                table_owner.as_ref().expect("key table").clone();
                                            first = table.clone();
                                            (*c).flags |= CLIENT_REDRAWSTATUS as u64;
                                            continue '_table_changed;
                                        }
                                    }
                                    if bd.is_some() {
                                        if (*c).flags & CLIENT_REPEAT as uint64_t != 0
                                            && !bd.as_ref().unwrap().flags & KEY_BINDING_REPEAT != 0
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
                                        log_debug(format_args!(
                                            "not found in key table {}",
                                            log_cstr(
                                                ((table.borrow().name).as_ptr().cast_mut())
                                                    as *const _
                                            )
                                        ));
                                        if server_client_is_default_key_table(
                                            &*(c),
                                            &table.borrow(),
                                        ) == 0
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
                                        if !std::rc::Rc::ptr_eq(&first, &table)
                                            && !flags & CLIENT_REPEAT as uint64_t != 0
                                        {
                                            current_block = 10435735846551762309;
                                            break;
                                        } else {
                                            current_block = 15469183920764600035;
                                            break;
                                        }
                                    }
                                    13763002826403452995 => {
                                        log_debug(format_args!(
                                            "found in key table {}",
                                            log_cstr(
                                                ((table.borrow().name).as_ptr().cast_mut())
                                                    as *const _
                                            )
                                        ));
                                        let bd = bd.take().expect("matched binding");
                                        repeat = server_client_repeat_time(&*(c), &bd);
                                        if repeat != 0 as u_int {
                                            (*c).flags |= CLIENT_REPEAT as uint64_t;
                                            (*c).last_key = bd.key;
                                            let timeout = Duration::from_millis(repeat as u64);
                                            (*c).repeat_timer.cancel();
                                            (*c).repeat_timer.arm(timeout).expect("arm timer");
                                        } else {
                                            (*c).flags &= !CLIENT_REPEAT as uint64_t;
                                            server_client_set_key_table(
                                                &(*(c)).observer.upgrade().expect("live client"),
                                                ::core::ptr::null::<::core::ffi::c_char>(),
                                            );
                                        }
                                        (*c).flags |= CLIENT_REDRAWSTATUS as u64;
                                        key_bindings_dispatch(
                                            bd,
                                            Some(item_handle),
                                            (c).as_ref()
                                                .and_then(|model| model.observer.upgrade())
                                                .as_ref(),
                                            event,
                                            &raw mut fs,
                                        );
                                        current_block = 1578459965781631232;
                                        break;
                                    }
                                    13484060386966298149 => {
                                        log_debug(format_args!("trying in root table"));
                                        server_client_set_key_table(
                                            &(*(c)).observer.upgrade().expect("live client"),
                                            ::core::ptr::null::<::core::ffi::c_char>(),
                                        );
                                        table_owner = (*c).keytable.clone();
                                        table = table_owner.as_ref().expect("key table").clone();
                                        if (*c).flags & CLIENT_REPEAT as uint64_t != 0 {
                                            first = table.clone();
                                        }
                                        (*c).flags &= !CLIENT_REPEAT as uint64_t;
                                        (*c).flags |= CLIENT_REDRAWSTATUS as u64;
                                    }
                                    _ => {
                                        log_debug(format_args!(
                                            "found in key table {} (not repeating)",
                                            log_cstr(
                                                ((table.borrow().name).as_ptr().cast_mut())
                                                    as *const _
                                            )
                                        ));
                                        server_client_set_key_table(
                                            &(*(c)).observer.upgrade().expect("live client"),
                                            ::core::ptr::null::<::core::ffi::c_char>(),
                                        );
                                        table_owner = (*c).keytable.clone();
                                        table = table_owner.as_ref().expect("key table").clone();
                                        first = table.clone();
                                        (*c).flags &= !CLIENT_REPEAT as uint64_t;
                                        (*c).flags |= CLIENT_REDRAWSTATUS as u64;
                                    }
                                }
                            }
                        }
                        match current_block {
                            1578459965781631232 => {}
                            15469183920764600035 => {}
                            _ => {
                                server_client_set_key_table(
                                    &(*(c)).observer.upgrade().expect("live client"),
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                );
                                (*c).flags |= CLIENT_REDRAWSTATUS as u64;
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
                                if let Some(bytes) = (*event).bytes_ptr_len() {
                                    window_pane_paste(
                                        &(*wp).observer.upgrade().expect("paste target pane"),
                                        key,
                                        bytes,
                                    );
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
                        let pane_owner = wp.as_ref().and_then(|pane| pane.observer.upgrade());
                        if !(server_client_handle_dead_key(pane_owner.as_ref(), key) != 0) {
                            if !((*c).flags & CLIENT_READONLY as uint64_t != 0) {
                                if let Some(pane_owner) = pane_owner.as_ref() {
                                    window_pane_key(pane_owner, Some(&c_owner), wl.clone(), key, m);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    if !s.is_none() && key != KEYC_FOCUS_OUT as ::core::ffi::c_ulong as key_code {
        server_client_update_latest(&(*(c)).observer.upgrade().expect("live client"));
    }
    return CMD_RETURN_NORMAL;
}

struct QueuedKeyEvent(Box<key_event>, Option<ClientRef>);

impl Drop for QueuedKeyEvent {
    fn drop(&mut self) {
        if let Some(client) = self.1.take() {
            server_client_unref_owned(client);
        }
    }
}

unsafe fn server_client_handle_menu_key(
    owner: &ClientRef,
    mut event: *mut key_event,
) -> ::core::ffi::c_int {
    let c = owner.get();
    let (window, menu) = {
        let retained = (*c)
            .session_handle()
            .expect("live session")
            .current_winlink()
            .get_unchecked()
            .window_handle()
            .cloned()
            .expect("current window");
        (std::rc::Rc::downgrade(&retained), retained.menu_observer())
    };
    let Some(menu) = menu else { return 0 };
    let mut new_event = (*event).metadata_snapshot();
    let mut m: *mut mouse_event = std::ptr::null_mut();
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
        (*m).statusat = status_at_line(&(*c).observer.upgrade().expect("live client"));
        (*m).statuslines = status_line_size(&(*c).observer.upgrade().expect("live client"));
        let tty_window_view { ox, oy, .. } = tty_window_offset(&(*c).tty);
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
    if menu_key(Some(owner), &menu, &new_event) == 1 {
        menu_close(&window, Some(&menu));
    }
    return 1 as ::core::ffi::c_int;
}
unsafe fn server_client_handle_key0(
    owner: &ClientRef,
    mut owned: Box<key_event>,
    after_handle: Option<&std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>>,
    next: Option<&mut std::rc::Weak<std::cell::UnsafeCell<cmdq_item>>>,
) -> ::core::ffi::c_int {
    let after = after_handle.map_or(std::ptr::null_mut(), |item| item.get());
    let c = owner.get();
    let event: *mut key_event = &raw mut *owned;
    let mut s: Option<SessionRef> = (*c).session_handle();
    let item_allocation;
    let mut item: *mut cmdq_item = ::core::ptr::null_mut::<cmdq_item>();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    if s.is_none() || (*c).flags & CLIENT_UNATTACHEDFLAGS as uint64_t != 0 {
        return 0 as ::core::ffi::c_int;
    }
    if (*event).key == KEYC_REPORT_LIGHT_THEME as ::core::ffi::c_ulong as key_code {
        server_client_report_theme(
            &(*(c)).observer.upgrade().expect("live client"),
            THEME_LIGHT,
        );
        return 0 as ::core::ffi::c_int;
    }
    if (*event).key == KEYC_REPORT_DARK_THEME as ::core::ffi::c_ulong as key_code {
        server_client_report_theme(&(*(c)).observer.upgrade().expect("live client"), THEME_DARK);
        return 0 as ::core::ffi::c_int;
    }
    if !(*c).flags & CLIENT_READONLY as uint64_t != 0 {
        if !(*c).message_string.is_none() {
            if (*c).message_ignore_keys != 0 {
                return 0 as ::core::ffi::c_int;
            }
            status_message_clear(&(*c).observer.upgrade().expect("live client"));
        }
        if let Some(result) = server_client_overlay_key(
            &(*(c)).observer.upgrade().expect("live client"),
            &mut *event,
        ) {
            match result {
                0 => return 0 as ::core::ffi::c_int,
                1 => {
                    server_client_clear_overlay(&(*(c)).observer.upgrade().expect("live client"));
                    return 0 as ::core::ffi::c_int;
                }
                _ => {}
            }
        }
        server_client_clear_overlay(&(*(c)).observer.upgrade().expect("live client"));
        wp = (((s.as_ref().expect("live session").current_winlink())
            .get_unchecked()
            .window_handle()
            .as_ref())
        .expect("live window"))
        .active_pane()
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
        let active_pane_owner = wp.as_ref().and_then(|pane| pane.observer.upgrade());
        if server_client_handle_dead_key(active_pane_owner.as_ref(), (*event).key) != 0 {
            return 0 as ::core::ffi::c_int;
        }
        if !wp.is_null()
            && (*wp)
                .window_handle()
                .expect("pane window")
                .modal_pane()
                .is_some_and(|modal| {
                    std::rc::Rc::ptr_eq(&modal, active_pane_owner.as_ref().expect("active pane"))
                })
            && (*wp).flags & PANE_CLOSEONCANCEL != 0
            && ((*event).key == '\u{1b}' as i32 as key_code
                || (*event).key == 'c' as i32 as ::core::ffi::c_ulonglong | KEYC_CTRL)
        {
            server_kill_pane(active_pane_owner.as_ref().expect("live modal pane"));
            return 0 as ::core::ffi::c_int;
        }
        if !wp.is_null()
            && (*wp).flags & PANE_CAPTUREALLKEYS != 0
            && (*wp).modes.is_empty()
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
                window_pane_key(
                    active_pane_owner.as_ref().expect("key target pane"),
                    Some(owner),
                    (s.as_ref().expect("live session").current_winlink()).clone(),
                    (*event).key,
                    &raw mut (*event).m,
                );
                return 0 as ::core::ffi::c_int;
            }
        }
        if server_client_handle_menu_key(owner, event) != 0 {
            return 0 as ::core::ffi::c_int;
        }
        if (*c).prompt.is_some() {
            match status_prompt_key(
                &(*(c)).observer.upgrade().expect("live client"),
                (*event).key,
                &raw mut (*event).m,
            ) as ::core::ffi::c_uint
            {
                1 | 2 => return 0 as ::core::ffi::c_int,
                0 | 3 | _ => {}
            }
        }
        let prompt_window = s
            .as_ref()
            .expect("live session")
            .current_winlink()
            .get_unchecked()
            .window_handle()
            .cloned()
            .expect("prompt window");
        let mut prompt_pane = prompt_window.active_pane();
        if !prompt_pane
            .as_ref()
            .is_some_and(|pane| window_pane_has_prompt(&*pane.get()) != 0)
        {
            prompt_pane = prompt_window.next_pane(None);
            while let Some(pane_owner) = prompt_pane.as_ref() {
                if window_pane_has_prompt(&*pane_owner.get()) != 0
                    && window_pane_is_visible(pane_owner) != 0
                {
                    break;
                }
                prompt_pane = window_pane_next((pane_owner.get()).as_ref());
            }
        }
        prompt_window.release(c"prompt pane lookup");
        if let Some(pane_owner) = prompt_pane.filter(|pane| {
            window_pane_has_prompt(&*pane.get()) != 0 && window_pane_is_visible(pane) != 0
        }) {
            match window_pane_prompt_key(
                &pane_owner,
                Some(owner),
                (*event).key,
                &raw mut (*event).m,
            ) as ::core::ffi::c_uint
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
    let client_owner = if after.is_null() {
        None
    } else {
        (*event).client = std::rc::Rc::downgrade(owner);
        Some(owner.clone())
    };
    let queued_event = QueuedKeyEvent(owned, client_owner);
    item_allocation = cmdq_get_callback_owned(
        c"server_client_key_callback",
        Some(Box::new(move |item| unsafe {
            server_client_key_callback(item, queued_event)
        })),
    );
    item = item_allocation.get();
    if let Some(after) = after_handle {
        let inserted = cmdq_insert_after(after, item_allocation);
        if let Some(next) = next {
            *next = inserted;
        }
        return 1;
    }
    cmdq_append(Some(owner), item_allocation);
    return 1 as ::core::ffi::c_int;
}
pub unsafe fn server_client_handle_key(
    owner: &ClientRef,
    event: Box<key_event>,
) -> ::core::ffi::c_int {
    return server_client_handle_key0(owner, event, None, None);
}
pub unsafe fn server_client_handle_key_after(
    owner: &ClientRef,
    event: Box<key_event>,
    after_handle: Option<&std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>>,
    next: Option<&mut std::rc::Weak<std::cell::UnsafeCell<cmdq_item>>>,
) -> ::core::ffi::c_int {
    return server_client_handle_key0(owner, event, after_handle, next);
}
pub unsafe fn server_client_loop() {
    let mut window_cursor = windows_minmax(&windows);
    while let Some(window_owner) = window_cursor.take() {
        server_client_check_window_resize(&window_owner);
        window_cursor = window_owner.next_window();
        crate::src::window::window_remove_ref(window_owner, c"window traversal".as_ptr());
    }
    let mut window_cursor = windows_minmax(&windows);
    while let Some(window_owner) = window_cursor.take() {
        let mut pane_cursor = window_owner.next_pane(None);
        while let Some(pane_owner) = pane_cursor {
            let wp = pane_owner.get();
            if (*wp).flags & PANE_STYLECHANGED != 0 {
                let wme = (*wp).active_mode_entry();
                if wme.is_alive() && (*(wme).get_unchecked().mode).style_changed.is_some() {
                    (*(wme).get_unchecked().mode)
                        .style_changed
                        .expect("non-null function pointer")(wme);
                }
            }
            pane_cursor = window_pane_next(Some(&*wp));
        }
        window_cursor = window_owner.next_window();
        crate::src::window::window_remove_ref(window_owner, c"window traversal".as_ptr());
    }
    let mut registry_c_owner = clients.first();
    while let Some(client_owner) = registry_c_owner {
        let c = client_owner.get();
        server_client_check_exit(&client_owner, 0 as ::core::ffi::c_int);
        if !(*c).session_handle().is_none()
            && (*c)
                .session_handle()
                .expect("live session")
                .current_winlink()
                .is_alive()
        {
            server_client_check_modes(&client_owner);
            server_client_check_redraw(&client_owner);
            server_client_reset_state(&client_owner);
        }
        registry_c_owner = clients.next(&client_owner);
    }
    let mut window_cursor = windows_minmax(&windows);
    while let Some(window_owner) = window_cursor.take() {
        let mut pane_cursor = window_owner.next_pane(None);
        while let Some(pane_owner) = pane_cursor {
            let wp = pane_owner.get();
            crate::src::window::WindowPane::finish_cycle(&pane_owner);
            pane_cursor = window_pane_next(Some(&*wp));
        }
        check_window_name(&window_owner);
        window_cursor = window_owner.next_window();
        crate::src::window::window_remove_ref(window_owner, c"window traversal".as_ptr());
    }
    let mut window_cursor = windows_minmax(&windows);
    while let Some(window_owner) = window_cursor.take() {
        let mut pane_cursor = window_owner.next_pane(None);
        while let Some(pane_owner) = pane_cursor {
            let wp = pane_owner.get();
            window_pane_send_theme_update(&pane_owner);
            pane_cursor = window_pane_next(Some(&*wp));
        }
        window_cursor = window_owner.next_window();
        crate::src::window::window_remove_ref(window_owner, c"window traversal".as_ptr());
    }
}
unsafe fn server_client_check_window_resize(owner: &WindowRef) {
    let mut wl: refbox::Weak<winlink> = refbox::Weak::new();
    let Some(request) = owner.pending_resize() else {
        return;
    };
    wl = owner.next_winlink(None);
    while wl.is_alive() {
        if wl
            .get_unchecked()
            .session
            .upgrade()
            .is_some_and(|owner| owner.is_attached() && owner.current_winlink() == wl)
        {
            break;
        }
        wl = owner.next_winlink(Some(wl.clone()));
    }
    if !wl.is_alive() {
        return;
    }
    log_debug(format_args!(
        "{}: resizing window @{}",
        "server_client_check_window_resize",
        ((owner).id()) as u32
    ));
    resize_window(
        owner,
        request.sx,
        request.sy,
        request.xpixel as ::core::ffi::c_int,
        request.ypixel as ::core::ffi::c_int,
    );
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PromptCursor {
    mode: ::core::ffi::c_int,
    cx: u_int,
    cy: u_int,
}

// No prompt leaves the caller's cursor unchanged. A clipped prompt still owns
// the cursor but hides it, retaining the incoming coordinates outside the view.
unsafe fn server_client_prompt_cursor(
    owner: &ClientRef,
    wp: &window_pane,
    mut cursor: PromptCursor,
) -> Option<PromptCursor> {
    let mut r = Vec::new();
    let mut px: ::core::ffi::c_int = 0;
    let mut py: ::core::ffi::c_int = 0;
    if window_pane_has_prompt(wp) == 0 {
        return None;
    }
    cursor.mode &= !MODE_CURSOR;
    let tty_window_view { ox, oy, sx, sy, .. } = owner.terminal_view();
    if status_at_line(owner) == 0 as ::core::ffi::c_int {
        py = wp.yoff;
    } else {
        py = (wp.yoff as u_int)
            .wrapping_add(wp.sy)
            .wrapping_sub(1 as u_int) as ::core::ffi::c_int;
    }
    px = (wp.xoff as u_int).wrapping_add(wp.prompt_cx) as ::core::ffi::c_int;
    if px < ox as ::core::ffi::c_int
        || px > ox.wrapping_add(sx) as ::core::ffi::c_int
        || py < oy as ::core::ffi::c_int
        || py > oy.wrapping_add(sy) as ::core::ffi::c_int
    {
        return Some(cursor);
    }
    cursor.cx = (px as u_int).wrapping_sub(ox);
    cursor.cy = (py as u_int).wrapping_sub(oy);
    window_visible_ranges(
        Some(wp),
        cursor.cx as ::core::ffi::c_int,
        cursor.cy as ::core::ffi::c_int,
        1 as u_int,
        &mut r,
    );
    if window_position_is_visible(&r, cursor.cx) {
        if status_at_line(owner) == 0 as ::core::ffi::c_int {
            cursor.cy = cursor.cy.wrapping_add(status_line_size(owner));
        }
        cursor.mode |= MODE_CURSOR;
    }
    return Some(cursor);
}

#[cfg(test)]
mod prompt_cursor_tests {
    use super::*;
    use crate::src::shared::prompt::prompt;
    use crate::src::shared::rc;
    use crate::src::shared::screen::MODE_WRAP;

    #[test]
    fn pane_prompt_cursor_preserves_clipped_and_occluded_coordinates() {
        unsafe {
            use crate::src::session::Session;
            let mut options = crate::src::options::options_create(None);
            for key in [c"status", c"status-position"] {
                let definition = crate::src::options_table::options_table
                    .iter()
                    .find(|definition| definition.name == Some(key))
                    .unwrap();
                crate::src::options::options_default(&mut *options, definition);
            }
            let session_owner = session::with_options_for_test(options);
            let c = client::with_session_for_test(Some(&session_owner));
            c.borrow_terminal_mut().sy = 24;
            let previous_sessions = std::mem::replace(
                &mut crate::src::session::sessions,
                crate::src::shared::session::sessions { storage: None },
            );
            crate::src::session::sessions_insert(
                &mut crate::src::session::sessions,
                session_owner.clone(),
            );
            {
                let mut terminal = c.borrow_terminal_mut();
                (terminal.oox, terminal.ooy, terminal.osx, terminal.osy) = (10, 5, 80, 20);
            }
            let incoming = PromptCursor {
                mode: MODE_CURSOR | MODE_WRAP,
                cx: 67,
                cy: 68,
            };

            // Golden mode/coordinates from tmux e880cf63e0a9's cursor helper.
            // Its viewport comparisons intentionally include the right/bottom edge.
            for (at, has_prompt, x, y, covered, expected) in [
                (22, false, 15, 8, false, None),
                (22, true, 15, 8, false, Some((17, 7, 6))),
                (0, true, 15, 8, false, Some((17, 7, 5))),
                (-1, true, 15, 8, false, Some((17, 7, 6))),
                (22, true, 5, 8, false, Some((16, 67, 68))),
                (22, true, 89, 8, false, Some((16, 67, 68))),
                (22, true, 88, 8, false, Some((17, 80, 6))),
                (22, true, 15, 1, false, Some((16, 67, 68))),
                (22, true, 15, 23, false, Some((16, 67, 68))),
                (22, true, 15, 22, false, Some((17, 7, 20))),
                (22, true, 15, 8, true, Some((16, 7, 6))),
                (0, true, 15, 8, true, Some((16, 7, 3))),
            ] {
                session_owner.with_options_mut(|options| {
                    options_set_number(options, c"status".as_ptr(), 2);
                    options_set_number(
                        options,
                        c"status-position".as_ptr(),
                        if at == 0 { 0 } else { 1 },
                    );
                });
                crate::src::session::recalculate_size_state();
                let flags = if at == -1 { CLIENT_STATUSOFF as u64 } else { 0 };
                c.update_flags(flags, !flags);
                let window_owner = window::with_size_for_test(100, 40);
                let prompt = refbox::RefBox::new(prompt::default());
                let base = window_pane::new();
                let wp = rc::as_ptr(&base);
                (*wp).window = std::rc::Rc::downgrade(&window_owner);
                (*wp).xoff = x;
                (*wp).yoff = y;
                (*wp).sy = 4;
                (*wp).prompt_cx = 2;
                if has_prompt {
                    (*wp).prompt = Some(prompt);
                }
                window_owner
                    .borrow_pane_order_mut(crate::src::window::PaneOrder::Stacking)
                    .push_front(std::rc::Rc::downgrade(&base));
                let blocker = window_pane::new();
                if covered {
                    let cover = rc::as_ptr(&blocker);
                    (*cover).window = std::rc::Rc::downgrade(&window_owner);
                    (*cover).xoff = 6;
                    (*cover).yoff = if at == 0 { 3 } else { 6 };
                    (*cover).sx = 3;
                    (*cover).sy = 1;
                    window_owner
                        .borrow_pane_order_mut(crate::src::window::PaneOrder::Stacking)
                        .push_front(std::rc::Rc::downgrade(&blocker));
                }
                let result = server_client_prompt_cursor(&c, &*wp, incoming);
                assert_eq!(
                    result.map(|cursor| (cursor.mode, cursor.cx, cursor.cy)),
                    expected,
                    "status {at}, prompt {has_prompt}, pane {x},{y}, covered {covered}"
                );
                crate::src::window::window_remove_ref(window_owner, c"test owner".as_ptr());
            }
            crate::src::session::sessions_remove(
                &mut crate::src::session::sessions,
                &session_owner,
            );
            crate::src::session::sessions = previous_sessions;
        }
    }
}

unsafe fn server_client_reset_state(client_owner: &ClientRef) {
    let c = client_owner.get();
    let Some(session_owner) = (*c).session.upgrade() else {
        return;
    };
    let current_link = session_owner.current_winlink();
    let Ok(link) = current_link.try_borrow_mut() else {
        return;
    };
    let window_owner = link
        .window_owner
        .as_ref()
        .expect("current link window")
        .clone();
    drop(link);
    let result = (|| {
        let mut r = Vec::new();
        let mut tty: *mut tty = &raw mut (*c).tty;
        let active_owner = window_owner.active_pane();
        let wp = active_owner
            .as_ref()
            .map_or(std::ptr::null_mut(), |owner| owner.get());
        let mut s: Option<ScreenMode> = None;

        let mut mode: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut cursor: ::core::ffi::c_int = 0;
        let mut flags: ::core::ffi::c_int = 0;
        let mut pane_mode: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut cx: u_int = 0 as u_int;
        let mut cy: u_int = 0 as u_int;
        let mut prompt: u_int = 0 as u_int;
        let mut sb_w: u_int = 0;
        if (*c).flags & (CLIENT_CONTROL | CLIENT_SUSPENDED) as uint64_t != 0 {
            return;
        }
        flags = (*tty).flags & TTY_BLOCK;
        (*tty).flags &= !TTY_BLOCK;
        let menu_owner = window_owner.menu_observer();
        if (*c).overlay.has_draw() {
            if let Some((overlay_screen, overlay_cx, overlay_cy)) =
                server_client_overlay_mode(&(*(c)).observer.upgrade().expect("live client"))
            {
                s = Some(overlay_screen);
                cx = overlay_cx;
                cy = overlay_cy;
            }
        } else if let Some(menu) = menu_owner.as_ref() {
            let menu_borrow = menu.try_borrow_mut().expect("live unborrowed menu");
            let md = &*menu_borrow;
            (cx, cy) = menu_get_cursor(md);
            s = Some(ScreenMode::from(menu_screen(md)));
        } else if !wp.is_null() && (*c).prompt.is_none() {
            s = Some(ScreenMode::from(&*(*wp).screen_ptr()));
        } else {
            s = Some(ScreenMode::from(&*(*c).status.active_screen()));
        }
        if let Some(s) = s {
            mode = s.mode;
        }
        if log_get_level() != 0 as ::core::ffi::c_int {
            log_debug(format_args!(
                "{}: client {} mode {}",
                "server_client_reset_state",
                log_cstr(
                    (((*c).name)
                        .as_ref()
                        .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                        as *const _
                ),
                screen_mode_display(mode)
            ));
        }
        client_owner.with_terminal_output(|terminal| {
            tty_region_off(terminal);
            tty_margin_off(terminal);
        });
        if (*c).prompt.is_some() {
            prompt = 1 as u_int;
            (cx, cy) = status_prompt_cursor(&(*c).observer.upgrade().expect("live client"));
        } else if !wp.is_null() && !(*c).overlay.has_draw() {
            if window_owner.menu_observer().is_some() {
                let tty_window_view { ox, oy, sx, sy, .. } = tty_window_offset(&*tty);
                if cx < ox || cx >= ox.wrapping_add(sx) || cy < oy || cy >= oy.wrapping_add(sy) {
                    mode &= !MODE_CURSOR;
                } else {
                    cx = cx.wrapping_sub(ox);
                    cy = cy.wrapping_sub(oy);
                    if status_at_line(&(*c).observer.upgrade().expect("live client"))
                        == 0 as ::core::ffi::c_int
                    {
                        cy = cy.wrapping_add(status_line_size(
                            &(*c).observer.upgrade().expect("live client"),
                        ));
                    }
                }
                prompt = 1 as u_int;
            } else {
                if let Some(cursor) =
                    server_client_prompt_cursor(client_owner, &*wp, PromptCursor { mode, cx, cy })
                {
                    prompt = 1;
                    mode = cursor.mode;
                    cx = cursor.cx;
                    cy = cursor.cy;
                }
            }
            if prompt == 0 {
                let s = s.expect("active pane screen mode");
                cursor = 0 as ::core::ffi::c_int;
                pane_mode = (*wp).base.mode;
                let tty_window_view { ox, oy, sx, sy, .. } = tty_window_offset(&*tty);
                if (*wp).xoff + s.cx as ::core::ffi::c_int >= ox as ::core::ffi::c_int
                    && (*wp).xoff + s.cx as ::core::ffi::c_int
                        <= ox as ::core::ffi::c_int + sx as ::core::ffi::c_int
                    && (*wp).yoff + s.cy as ::core::ffi::c_int >= oy as ::core::ffi::c_int
                    && (*wp).yoff + s.cy as ::core::ffi::c_int
                        <= oy as ::core::ffi::c_int + sy as ::core::ffi::c_int
                {
                    cursor = 1 as ::core::ffi::c_int;
                    cx = ((*wp).xoff + s.cx as ::core::ffi::c_int - ox as ::core::ffi::c_int)
                        as u_int;
                    cy = ((*wp).yoff + s.cy as ::core::ffi::c_int - oy as ::core::ffi::c_int)
                        as u_int;
                    window_visible_ranges(
                        wp.as_ref(),
                        cx as ::core::ffi::c_int,
                        cy as ::core::ffi::c_int,
                        1 as u_int,
                        &mut r,
                    );
                    if !window_position_is_visible(&r, cx) {
                        cursor = 0 as ::core::ffi::c_int;
                    }
                    if window_pane_scrollbar_overlay_visible(&*wp) != 0 {
                        sb_w = (*wp).scrollbar_style.width as u_int;
                        if sb_w > (*wp).sx {
                            sb_w = (*wp).sx;
                        }
                        if sb_w != 0 as u_int
                            && (window_owner).scrollbars().position == PANE_SCROLLBARS_LEFT
                        {
                            if s.cx < sb_w {
                                cursor = 0 as ::core::ffi::c_int;
                            }
                        } else if sb_w != 0 as u_int && s.cx >= (*wp).sx.wrapping_sub(sb_w) {
                            cursor = 0 as ::core::ffi::c_int;
                        }
                    }
                    if status_at_line(&(*c).observer.upgrade().expect("live client"))
                        == 0 as ::core::ffi::c_int
                    {
                        cy = cy.wrapping_add(status_line_size(
                            &(*c).observer.upgrade().expect("live client"),
                        ));
                    }
                }
                if cursor == 0 {
                    mode &= !MODE_CURSOR;
                }
            }
        } else if !(*c).overlay.has_mode() || s.is_none() {
            mode &= !MODE_CURSOR;
        }
        if !pane_mode & MODE_SYNC != 0 {
            log_debug(format_args!(
                "{}: cursor to {},{}",
                "server_client_reset_state",
                (cx) as u32,
                (cy) as u32
            ));
            client_owner.with_terminal_output(|terminal| tty_cursor(terminal, cx, cy));
        } else {
            mode &= !CURSOR_MODES;
            mode |= (*tty).mode & CURSOR_MODES;
            s = None;
        }
        if session_owner.with_options_mut(|options| {
            options_get_number(
                options,
                b"mouse\0" as *const u8 as *const ::core::ffi::c_char,
            )
        }) != 0
        {
            if !(*c).overlay.has_draw() && window_owner.menu_observer().is_none() {
                mode &= !ALL_MOUSE_MODES;
                let mut cursor = window_owner.next_pane(None);
                while let Some(pane_owner) = cursor {
                    let pane = &*pane_owner.get();
                    if (*pane.screen_ptr()).mode & MODE_MOUSE_ALL != 0 {
                        mode |= MODE_MOUSE_ALL;
                    }
                    cursor = window_pane_next(Some(pane));
                }
            }
            if session_owner.with_options_mut(|options| {
                options_get_number(
                    options,
                    b"focus-follows-mouse\0" as *const u8 as *const ::core::ffi::c_char,
                )
            }) != 0
                || (window_owner).scrollbars().mode == PANE_SCROLLBARS_MODAL
                || (window_owner).scrollbars().mode == PANE_SCROLLBARS_AUTOHIDE
            {
                mode |= MODE_MOUSE_ALL;
            } else if !mode & MODE_MOUSE_ALL != 0 {
                mode |= MODE_MOUSE_BUTTON;
            }
        }
        if !(*c).overlay.has_draw() && prompt != 0 {
            mode &= !MODE_BRACKETPASTE;
        }
        client_owner.with_terminal_output(|terminal| {
            tty_update_mode(terminal, mode, s);
            tty_reset(terminal);
            tty_sync_end(terminal);
        });
        (*tty).flags |= flags;
    })();
    crate::src::window::window_remove_ref(window_owner, c"server_client_reset_state".as_ptr());
    result
}
unsafe fn server_client_repeat_timer(owner: &ClientRef) {
    let c = owner.get();
    if (*c).flags & CLIENT_REPEAT as uint64_t != 0 {
        server_client_set_key_table(
            &(*(c)).observer.upgrade().expect("live client"),
            ::core::ptr::null::<::core::ffi::c_char>(),
        );
        (*c).flags &= !CLIENT_REPEAT as uint64_t;
        (*c).flags |= CLIENT_REDRAWSTATUS as u64;
    }
}
unsafe fn server_client_click_timer(owner: &ClientRef) {
    let c = owner.get();
    log_debug(format_args!("click timer expired"));
    if (*c).flags & CLIENT_TRIPLECLICK as uint64_t != 0 {
        let event = key_event::new(
            KEYC_DOUBLECLICK as ::core::ffi::c_ulong as key_code,
            (*c).click_event,
            None,
        );
        server_client_handle_key(owner, event);
    }
    (*c).flags &= !(CLIENT_DOUBLECLICK | CLIENT_TRIPLECLICK) as uint64_t;
}
unsafe fn server_client_start_exit_timer(c: &mut client) {
    let timeout = Duration::from_secs(10);
    if !(*c).exit_timer.is_pending() {
        (*c).exit_timer.arm(timeout).expect("arm timer");
    }
}
unsafe fn server_client_exit_timer(owner: &ClientRef) {
    let c = owner.get();
    if (*c).flags & (CLIENT_DEAD | CLIENT_SUSPENDED) as uint64_t != 0 {
        return;
    }
    if (*c).flags & CLIENT_EXITED as uint64_t != 0 {
        log_debug(format_args!(
            "{}: {} took too long to exit",
            "server_client_exit_timer",
            log_cstr(
                (((*c).name)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                    as *const _
            )
        ));
        server_client_lost(owner);
    } else if (*c).flags & CLIENT_EXIT as uint64_t != 0 {
        log_debug(format_args!(
            "{}: {} took too long to flush",
            "server_client_exit_timer",
            log_cstr(
                (((*c).name)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                    as *const _
            )
        ));
        server_client_check_exit(owner, 1 as ::core::ffi::c_int);
    }
}
unsafe fn server_client_check_exit(client_owner: &ClientRef, force: ::core::ffi::c_int) {
    let c = client_owner.get();
    let mut name: *const ::core::ffi::c_char = ((*c).exit_session)
        .as_ref()
        .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut());
    if (*c).flags & (CLIENT_DEAD | CLIENT_EXITED) as uint64_t != 0 {
        return;
    }
    if !(*c).flags & CLIENT_EXIT as uint64_t != 0 {
        return;
    }
    if (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
        if force != 0 {
            control_discard_all(&(*c).observer.upgrade().expect("live client"));
        } else {
            control_discard(&(*c).observer.upgrade().expect("live client"));
            if control_all_done(&(*c).observer.upgrade().expect("live client")) == 0 {
                server_client_start_exit_timer(&mut *c);
                return;
            }
        }
    }
    if force == 0 && crate::src::file::client_files_has_pending_data(&(*c).files) {
        server_client_start_exit_timer(&mut *c);
        return;
    }
    (*c).flags |= CLIENT_EXITED as uint64_t;
    (*c).exit_timer.cancel();
    server_client_start_exit_timer(&mut *c);
    match (*c).exit_type as ::core::ffi::c_uint {
        0 => {
            let mut data = Vec::from((*c).retval.to_ne_bytes());
            if !(*c).exit_message.is_none() {
                data.extend_from_slice(
                    ((*c).exit_message)
                        .as_deref()
                        .expect("string is present")
                        .to_bytes_with_nul(),
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
unsafe fn server_client_redraw_timer() {
    log_debug(format_args!("redraw timer fired"));
}
unsafe fn server_client_check_modes(client_owner: &ClientRef) {
    let c = client_owner.get();
    let Some(session_owner) = (*c).session.upgrade() else {
        return;
    };
    let current_link = session_owner.current_winlink();
    let Ok(link) = current_link.try_borrow_mut() else {
        return;
    };
    let window_owner = link
        .window_owner
        .as_ref()
        .expect("current link window")
        .clone();
    drop(link);
    let result = (|| {
        if (*c).flags & (CLIENT_CONTROL | CLIENT_SUSPENDED) as uint64_t != 0 {
            return;
        }
        if !(*c).flags & CLIENT_REDRAWSTATUS as uint64_t != 0 {
            return;
        }
        let mut cursor = window_owner.next_pane(None);
        while let Some(pane_owner) = cursor {
            let wp = pane_owner.get();
            let wme = (*wp).active_mode_entry();
            if wme.is_alive() && (*(wme).get_unchecked().mode).update.is_some() {
                (*(wme).get_unchecked().mode)
                    .update
                    .expect("non-null function pointer")(wme);
            }
            cursor = window_pane_next(Some(&*wp));
        }
    })();
    crate::src::window::window_remove_ref(window_owner, c"server_client_check_modes".as_ptr());
    result
}
unsafe fn server_client_any_pane_redraw(c: &client, window: &WindowRef) -> bool {
    if c.flags & CLIENT_REDRAWWINDOW as uint64_t != 0 {
        return true;
    }
    window
        .pane_snapshot()
        .into_iter()
        .any(|pane| (*pane.get()).flags & (PANE_REDRAW | PANE_REDRAWSCROLLBAR) != 0)
}
unsafe fn server_client_check_redraw(client_owner: &ClientRef) {
    let c = client_owner.get();
    let Some(session_owner) = (*c).session.upgrade() else {
        return;
    };
    let current_link = session_owner.current_winlink();
    let Ok(link) = current_link.try_borrow_mut() else {
        return;
    };
    let window_owner = link
        .window_owner
        .as_ref()
        .expect("current link window")
        .clone();
    drop(link);
    let result = (|| {
        let s = Some(session_owner.clone());
        let mut tty: *mut tty = &raw mut (*c).tty;
        let mut needed: ::core::ffi::c_int = 0;
        let mut tflags: ::core::ffi::c_int = 0;
        let mut mode: ::core::ffi::c_int = (*tty).mode;
        let timeout = Duration::from_micros(1000);
        static mut ev: Timer = Timer::new();
        let mut n: size_t = 0;
        if (*c).flags & (CLIENT_CONTROL | CLIENT_SUSPENDED) as uint64_t != 0 {
            return;
        }
        if (*c).flags & CLIENT_ALLREDRAWFLAGS as uint64_t != 0 {
            log_debug(format_args!(
                "{}: redraw{}{}{}{}{}",
                log_cstr(
                    (((*c).name)
                        .as_ref()
                        .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                        as *const _
                ),
                log_cstr(
                    (if (*c).flags & CLIENT_REDRAWWINDOW as uint64_t != 0 {
                        b" window\0" as *const u8 as *const ::core::ffi::c_char
                    } else {
                        b"\0" as *const u8 as *const ::core::ffi::c_char
                    }) as *const _
                ),
                log_cstr(
                    (if (*c).flags & CLIENT_REDRAWSTATUS as uint64_t != 0 {
                        b" status\0" as *const u8 as *const ::core::ffi::c_char
                    } else {
                        b"\0" as *const u8 as *const ::core::ffi::c_char
                    }) as *const _
                ),
                log_cstr(
                    (if (*c).flags & CLIENT_REDRAWBORDERS as uint64_t != 0 {
                        b" borders\0" as *const u8 as *const ::core::ffi::c_char
                    } else {
                        b"\0" as *const u8 as *const ::core::ffi::c_char
                    }) as *const _
                ),
                log_cstr(
                    (if (*c).flags & CLIENT_REDRAWOVERLAY as uint64_t != 0 {
                        b" overlay\0" as *const u8 as *const ::core::ffi::c_char
                    } else {
                        b"\0" as *const u8 as *const ::core::ffi::c_char
                    }) as *const _
                ),
                log_cstr(
                    (if (*c).flags & CLIENT_REDRAWMENU as uint64_t != 0 {
                        b" menu\0" as *const u8 as *const ::core::ffi::c_char
                    } else {
                        b"\0" as *const u8 as *const ::core::ffi::c_char
                    }) as *const _
                )
            ));
        }
        needed = 0 as ::core::ffi::c_int;
        if (*c).flags as ::core::ffi::c_ulonglong
            & (CLIENT_ALLREDRAWFLAGS as ::core::ffi::c_ulonglong | CLIENT_REDRAWSCROLLBARS)
            != 0
        {
            needed = 1 as ::core::ffi::c_int;
        } else if server_client_any_pane_redraw(&*c, &window_owner) {
            needed = 1 as ::core::ffi::c_int;
        }
        if needed == 0 {
            (*c).flags &= !CLIENT_STATUSFORCE as uint64_t;
            return;
        }
        n = evbuffer_get_length((*tty).out.as_deref().expect("open TTY buffer"));
        if n != 0 as size_t || (*tty).flags & TTY_BLOCK != 0 {
            if n != 0 as size_t {
                log_debug(format_args!(
                    "{}: redraw deferred ({} left)",
                    log_cstr(
                        (((*c).name)
                            .as_ref()
                            .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                            as *const _
                    ),
                    (n) as usize
                ));
            } else {
                log_debug(format_args!(
                    "{}: redraw deferred (blocked)",
                    log_cstr(
                        (((*c).name)
                            .as_ref()
                            .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                            as *const _
                    )
                ));
            }
            if !ev.is_initialized() {
                ev.set(move || unsafe { server_client_redraw_timer() });
            }
            if !ev.is_pending() {
                log_debug(format_args!("redraw timer started"));
                ev.arm(timeout).expect("arm timer");
            }
            let mut cursor = window_owner.next_pane(None);
            while let Some(pane_owner) = cursor {
                let wp = &*pane_owner.get();
                if (*wp).flags & PANE_REDRAW != 0 {
                    (*c).flags |= CLIENT_REDRAWWINDOW as uint64_t;
                    break;
                } else {
                    if (*wp).flags & PANE_REDRAWSCROLLBAR != 0 {
                        (*c).flags = ((*c).flags as ::core::ffi::c_ulonglong
                            | CLIENT_REDRAWSCROLLBARS)
                            as uint64_t;
                    }
                    cursor = window_pane_next(Some(wp));
                }
            }
            return;
        }
        log_debug(format_args!(
            "{}: redraw needed",
            log_cstr(
                (((*c).name)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                    as *const _
            )
        ));
        tflags = (*tty).flags & (TTY_BLOCK | TTY_FREEZE | TTY_NOCURSOR);
        (*tty).flags = (*tty).flags & !(TTY_BLOCK | TTY_FREEZE) | TTY_NOCURSOR;
        if !(*c).flags & CLIENT_REDRAWWINDOW as uint64_t != 0 {
            for pane_owner in window_owner.pane_snapshot() {
                let wp = &*pane_owner.get();
                if (*wp).flags & PANE_REDRAW != 0 {
                    log_debug(format_args!(
                        "{}: redraw pane %{}",
                        "server_client_check_redraw",
                        ((*wp).id) as u32
                    ));
                    redraw_pane(client_owner, &pane_owner);
                } else if (*wp).flags & PANE_REDRAWSCROLLBAR != 0
                    || (*c).flags as ::core::ffi::c_ulonglong & CLIENT_REDRAWSCROLLBARS != 0
                {
                    log_debug(format_args!(
                        "{}: redraw scrollbar %{}",
                        "server_client_check_redraw",
                        ((*wp).id) as u32
                    ));
                    redraw_pane_scrollbar(client_owner, &pane_owner);
                }
            }
        }
        if (*c).flags & CLIENT_ALLREDRAWFLAGS as uint64_t != 0 {
            if s.as_ref()
                .expect("live session")
                .with_options_mut(|options| {
                    options_get_number(
                        options,
                        b"set-titles\0" as *const u8 as *const ::core::ffi::c_char,
                    )
                })
                != 0
            {
                server_client_set_title(client_owner);
                server_client_set_path(client_owner);
            }
            server_client_set_progress_bar(client_owner);
            redraw_screen(client_owner);
        }
        (*tty).flags = (*tty).flags & !TTY_NOCURSOR | tflags & TTY_NOCURSOR;
        client_owner.with_terminal_output(|terminal| tty_update_mode(terminal, mode, None));
        (*tty).flags = (*tty).flags & !(TTY_BLOCK | TTY_FREEZE | TTY_NOCURSOR) | tflags;
        (*c).flags = ((*c).flags as ::core::ffi::c_ulonglong
            & !(CLIENT_ALLREDRAWFLAGS as ::core::ffi::c_ulonglong
                | CLIENT_REDRAWSCROLLBARS
                | CLIENT_STATUSFORCE as ::core::ffi::c_ulonglong)) as uint64_t;
        (*c).redraw = evbuffer_get_length((*tty).out.as_deref().expect("open TTY buffer"));
        log_debug(format_args!(
            "{}: redraw added {} bytes",
            log_cstr(
                (((*c).name)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                    as *const _
            ),
            ((*c).redraw) as usize
        ));
    })();
    crate::src::window::window_remove_ref(window_owner, c"server_client_check_redraw".as_ptr());
    result
}
unsafe fn server_client_set_title(client_owner: &ClientRef) {
    let mut template_session_value: Option<std::ffi::CString> = None;

    let c = client_owner.get();
    let Some(session_owner) = (*c).session.upgrade() else {
        return;
    };
    let s = Some(session_owner.clone());
    let mut template: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    template_session_value = Some(
        s.as_ref()
            .expect("live session")
            .with_options_mut(|options| {
                options_get_string(
                    options,
                    b"set-titles-string\0" as *const u8 as *const ::core::ffi::c_char,
                )
            }),
    );
    template = template_session_value
        .as_ref()
        .expect("option snapshot")
        .as_ptr();
    let mut ft_owner = crate::src::format::format_create_with_client(
        Some(client_owner),
        None,
        FORMAT_NONE,
        0 as ::core::ffi::c_int,
    );
    ft = &raw mut *ft_owner;
    format_defaults(
        ft,
        (c).as_ref()
            .and_then(|model| model.observer.upgrade())
            .as_ref(),
        None,
        (refbox::Weak::new()).clone(),
        None,
    );
    let title = format_expand_time_cstring(ft, template);
    if (*c).title.as_ref().is_none_or(|old| old != &title) {
        server_client_replace_title(&mut *c, Some(title.clone()));
        client_owner.with_terminal_output(|terminal| tty_set_title(terminal, &title));
    }
    format_free(ft_owner);
}
/// Acquire the current active pane for an immediate client operation.
/// Raw relationship fields are accessed only while resolving the handle.
unsafe fn server_client_active_pane(
    c: &client,
) -> Option<std::rc::Rc<std::cell::UnsafeCell<window_pane>>> {
    let session_owner = c.session.upgrade()?;
    let current = session_owner.current_winlink();
    let link = current.try_borrow_mut().ok()?;
    link.window_owner.as_ref()?.active_pane()
}

unsafe fn server_client_set_path(owner: &ClientRef) {
    let Some(active_owner) = server_client_active_pane(&*owner.get()) else {
        return;
    };
    let path = (*active_owner.get())
        .base
        .path
        .as_deref()
        .unwrap_or(c"")
        .to_owned();
    if (*owner.get()).path.as_deref() != Some(path.as_c_str()) {
        server_client_replace_path(&mut *owner.get(), Some(path.clone()));
        owner.with_terminal_output(|terminal| tty_set_path(terminal, &path));
    }
}
unsafe fn server_client_set_progress_bar(owner: &ClientRef) {
    let Some(active_owner) = server_client_active_pane(&*owner.get()) else {
        return;
    };
    let mut pane_progress = (*active_owner.get()).base.progress_bar;
    {
        let state = &mut *owner.get();
        if pane_progress.state == state.progress_bar.state
            && pane_progress.progress == state.progress_bar.progress
        {
            return;
        }
        state.progress_bar = pane_progress;
    }
    owner.with_terminal_output(|terminal| tty_set_progress_bar(terminal, &mut pane_progress));
}

unsafe fn server_client_dispatch(
    owner: &ClientRef,
    message: crate::src::shared::process::PeerMessage<'_>,
) {
    let c = owner.get();
    let mut current_block: u64;
    let mut datalen: ssize_t = 0;
    let mut s: Option<SessionRef> = None;
    let mut old_sx: u_int = 0;
    let mut old_sy: u_int = 0;
    if (*c).flags & CLIENT_DEAD as uint64_t != 0 {
        return;
    }
    let imsg = match message {
        crate::src::shared::process::PeerMessage::Disconnected => {
            server_client_lost(owner);
            return;
        }
        crate::src::shared::process::PeerMessage::Message(imsg) => imsg,
    };
    datalen = imsg.data.len() as ssize_t;
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
            if server_client_dispatch_identify(owner, imsg) != 0 as ::core::ffi::c_int {
                current_block = 13639960948656484833;
            } else {
                current_block = 14945149239039849694;
            }
        }
        MSG_COMMAND => {
            if server_client_dispatch_command(owner, imsg) != 0 as ::core::ffi::c_int {
                current_block = 13639960948656484833;
            } else {
                current_block = 14945149239039849694;
            }
        }
        MSG_RESIZE => {
            if datalen != 0 as ssize_t {
                current_block = 13639960948656484833;
            } else if (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
                current_block = 14945149239039849694;
            } else {
                server_client_update_latest(&(*(c)).observer.upgrade().expect("live client"));
                old_sx = (*c).tty.sx;
                old_sy = (*c).tty.sy;
                tty_resize(&(*c).observer.upgrade().expect("live client"));
                (*c).observer
                    .upgrade()
                    .expect("live client")
                    .with_terminal_output(|terminal| {
                        tty_repeat_requests(terminal, 0 as ::core::ffi::c_int)
                    });
                recalculate_sizes();
                if !(*c).overlay.has_resize() {
                    server_client_clear_overlay(&(*(c)).observer.upgrade().expect("live client"));
                } else {
                    server_client_overlay_resize(&(*(c)).observer.upgrade().expect("live client"));
                }
                (*c).flags |= CLIENT_ALLREDRAWFLAGS as u64;
                if !(*c).session_handle().is_none() {
                    server_client_fire_resized(
                        &(*(c)).observer.upgrade().expect("live client"),
                        old_sx,
                        old_sy,
                    );
                }
                current_block = 14945149239039849694;
            }
        }
        MSG_EXITING => {
            if datalen != 0 as ssize_t {
                current_block = 13639960948656484833;
            } else {
                server_client_set_session(&(*(c)).observer.upgrade().expect("live client"), None);
                recalculate_sizes();
                tty_close(&(*c).observer.upgrade().expect("live client"));
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
        MSG_WAKEUP | MSG_UNLOCK => {
            if datalen != 0 as ssize_t {
                current_block = 13639960948656484833;
            } else if (*c).flags & CLIENT_SUSPENDED as uint64_t == 0 {
                current_block = 14945149239039849694;
            } else {
                (*c).flags &= !CLIENT_SUSPENDED as uint64_t;
                if (*c).fd == -(1 as ::core::ffi::c_int) || (*c).session_handle().is_none() {
                    current_block = 14945149239039849694;
                } else {
                    s = (*c).session_handle();
                    (*c).activity_time = SystemTime::now();
                    tty_start_tty(&(*c).observer.upgrade().expect("live client"));
                    (*c).flags |= CLIENT_ALLREDRAWFLAGS as u64;
                    recalculate_sizes();
                    if !s.is_none() {
                        s.as_ref()
                            .expect("live session")
                            .update_activity(Some((*c).activity_time));
                    }
                    current_block = 14945149239039849694;
                }
            }
        }
        MSG_SHELL => {
            if datalen != 0 as ssize_t {
                current_block = 13639960948656484833;
            } else if server_client_dispatch_shell(&*owner.get()) != 0 as ::core::ffi::c_int {
                current_block = 13639960948656484833;
            } else {
                current_block = 14945149239039849694;
            }
        }
        MSG_WRITE_READY => {
            if file_write_ready(owner, imsg) != 0 as ::core::ffi::c_int {
                current_block = 13639960948656484833;
            } else {
                current_block = 14945149239039849694;
            }
        }
        MSG_WRITE_DONE => {
            if file_write_done(owner, imsg) != 0 as ::core::ffi::c_int {
                current_block = 13639960948656484833;
            } else {
                current_block = 14945149239039849694;
            }
        }
        MSG_READ => {
            if file_read_data(owner, imsg) != 0 as ::core::ffi::c_int {
                current_block = 13639960948656484833;
            } else {
                current_block = 14945149239039849694;
            }
        }
        MSG_READ_DONE => {
            if file_read_done(owner, imsg) != 0 as ::core::ffi::c_int {
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
            log_debug(format_args!(
                "client {} invalid message type {}",
                log_pointer((c) as *const ::core::ffi::c_void),
                (imsg.hdr.type_0) as i32
            ));
            proc_kill_peer((*c).peer);
            return;
        }
    };
}
unsafe fn server_client_read_only(
    item_handle: &std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>,
) -> cmd_retval {
    cmdq_error(item_handle, |out| out.write_all(b"client is read-only"));
    return CMD_RETURN_ERROR;
}
unsafe fn server_client_default_command(
    item_handle: &std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>,
) -> cmd_retval {
    let item = item_handle.get();
    let c_owner = cmdq_get_client((item).as_ref());
    let mut c: *mut client = c_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    let new_item_allocation;
    let cmdlist = options_get_command(global_options);
    if (*c).flags & CLIENT_READONLY as uint64_t != 0 && cmd_list_all_have(&cmdlist.borrow()) == 0 {
        new_item_allocation = cmdq_get_callback_owned(
            c"server_client_read_only",
            Some(Box::new(|item| unsafe { server_client_read_only(item) })),
        );
    } else {
        new_item_allocation = cmdq_get_command(&cmdlist, None);
    }
    cmdq_insert_after(item_handle, new_item_allocation);
    return CMD_RETURN_NORMAL;
}
unsafe fn server_client_command_done(
    item_handle: &std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>,
) -> cmd_retval {
    let item = item_handle.get();
    let c_owner = cmdq_get_client((item).as_ref());
    let mut c: *mut client = c_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    if !(*c).flags & CLIENT_ATTACHED as uint64_t != 0 {
        (*c).flags |= CLIENT_EXIT as uint64_t;
    } else if !(*c).flags & CLIENT_EXIT as uint64_t != 0 {
        if (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
            control_ready(&(*(c)).observer.upgrade().expect("live client"));
        }
        (*c).observer
            .upgrade()
            .expect("live client")
            .with_terminal_output(|terminal| tty_send_requests(terminal));
    }
    return CMD_RETURN_NORMAL;
}
unsafe fn server_client_dispatch_command(owner: &ClientRef, imsg: &mut imsg) -> ::core::ffi::c_int {
    let c = owner.get();
    let mut current_block: u64;
    let mut data: msg_command = msg_command { argc: 0 };
    let mut len: size_t = 0;
    let mut argv = Vec::new();
    let mut argc: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut cause: Option<CString> = None;
    let mut new_item_allocation = None;
    if (*c).flags & CLIENT_EXIT as uint64_t != 0 {
        return 0 as ::core::ffi::c_int;
    }
    let command_size = ::core::mem::size_of::<msg_command>();
    if imsg.data.len() < command_size {
        return -(1 as ::core::ffi::c_int);
    }
    let argc_bytes: [u8; ::core::mem::size_of::<::core::ffi::c_int>()] =
        imsg.data[..command_size].try_into().unwrap();
    data.argc = ::core::ffi::c_int::from_ne_bytes(argc_bytes);
    let trailing = &mut imsg.data[command_size..];
    len = trailing.len();
    if len > 0 && trailing[len - 1] != 0 {
        return -(1 as ::core::ffi::c_int);
    }
    let unpacked = if len == 0 {
        unpack_argv(&mut [], data.argc)
    } else {
        unpack_argv(trailing, data.argc)
    };
    if let Ok(decoded) = unpacked {
        argv = decoded;
        argc = ::core::ffi::c_int::try_from(argv.len()).expect("argv length exceeds c_int");
        if argc == 0 as ::core::ffi::c_int {
            new_item_allocation = Some(cmdq_get_callback_owned(
                c"server_client_default_command",
                Some(Box::new(|item| unsafe {
                    server_client_default_command(item)
                })),
            ));
            current_block = 13472856163611868459;
        } else {
            cmd_log_argv(&argv, c"cmd_unpack_argv");
            let mut pr = cmd_parse_from_argv(&argv);
            match pr.status as ::core::ffi::c_uint {
                0 => {
                    cause = pr.error;
                    current_block = 12680788052841528405;
                }
                1 | _ => {
                    if (*c).flags & CLIENT_READONLY as uint64_t != 0
                        && cmd_list_all_have(
                            &pr.cmdlist.as_ref().expect("parsed command list").borrow(),
                        ) == 0
                    {
                        new_item_allocation = Some(cmdq_get_callback_owned(
                            c"server_client_read_only",
                            Some(Box::new(|item| unsafe { server_client_read_only(item) })),
                        ));
                    } else {
                        new_item_allocation = Some(cmdq_get_command(
                            pr.cmdlist.as_ref().expect("successful command parse"),
                            None,
                        ));
                    }
                    drop(pr.cmdlist.take());
                    current_block = 13472856163611868459;
                }
            }
        }
        match current_block {
            12680788052841528405 => {}
            _ => {
                cmdq_append(
                    Some(owner),
                    new_item_allocation.take().expect("parsed command item"),
                );
                cmdq_append(
                    Some(owner),
                    cmdq_get_callback_owned(
                        c"server_client_command_done",
                        Some(Box::new(|item| unsafe { server_client_command_done(item) })),
                    ),
                );
                return 0 as ::core::ffi::c_int;
            }
        }
    } else {
        cause = Some(CString::new("command too long").unwrap());
    }
    cmdq_append(
        Some(owner),
        cmdq_get_error(
            cause
                .as_ref()
                .map_or(::core::ptr::null(), |error| error.as_ptr()),
        ),
    );
    (*c).flags |= CLIENT_EXIT as uint64_t;
    return 0 as ::core::ffi::c_int;
}
unsafe fn server_client_dispatch_identify(
    owner: &ClientRef,
    imsg: &mut imsg,
) -> ::core::ffi::c_int {
    let c = owner.get();
    let mut data: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut datalen: size_t = 0;
    let mut flags: ::core::ffi::c_int = 0;
    let mut feat: ::core::ffi::c_int = 0;
    let mut longflags: uint64_t = 0;
    if (*c).flags & CLIENT_IDENTIFIED as uint64_t != 0 {
        return -(1 as ::core::ffi::c_int);
    }
    data = imsg.data.as_ptr().cast::<::core::ffi::c_char>();
    datalen = imsg.data.len();
    match imsg.hdr.type_0 {
        MSG_IDENTIFY_FEATURES => {
            if datalen != ::core::mem::size_of::<::core::ffi::c_int>() as usize {
                return -(1 as ::core::ffi::c_int);
            }
            memcpy(
                &raw mut feat as *mut ::core::ffi::c_void,
                data as *const ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_int>() as size_t,
            );
            (*c).term_features |= feat;
            log_debug(format_args!(
                "client {} IDENTIFY_FEATURES {}",
                log_pointer((c) as *const ::core::ffi::c_void),
                log_cstr(tty_get_features(feat).as_ptr())
            ));
        }
        MSG_IDENTIFY_FLAGS => {
            if datalen != ::core::mem::size_of::<::core::ffi::c_int>() as usize {
                return -(1 as ::core::ffi::c_int);
            }
            memcpy(
                &raw mut flags as *mut ::core::ffi::c_void,
                data as *const ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_int>() as size_t,
            );
            (*c).flags |= flags as uint64_t;
            log_debug(format_args!(
                "client {} IDENTIFY_FLAGS {}",
                log_pointer((c) as *const ::core::ffi::c_void),
                log_hex(((flags) as u32) as u64)
            ));
        }
        MSG_IDENTIFY_LONGFLAGS => {
            if datalen != ::core::mem::size_of::<uint64_t>() as usize {
                return -(1 as ::core::ffi::c_int);
            }
            memcpy(
                &raw mut longflags as *mut ::core::ffi::c_void,
                data as *const ::core::ffi::c_void,
                ::core::mem::size_of::<uint64_t>() as size_t,
            );
            (*c).flags |= longflags;
            log_debug(format_args!(
                "client {} IDENTIFY_LONGFLAGS {}",
                log_pointer((c) as *const ::core::ffi::c_void),
                log_hex(longflags as ::core::ffi::c_ulonglong)
            ));
        }
        MSG_IDENTIFY_TERM => {
            if datalen == 0 as size_t
                || *data.offset(datalen.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
                    != '\0' as i32
            {
                return -(1 as ::core::ffi::c_int);
            }
            server_client_set_term_name(&mut *c, Some(CStr::from_ptr(data).to_owned()));
            log_debug(format_args!(
                "client {} IDENTIFY_TERM {}",
                log_pointer((c) as *const ::core::ffi::c_void),
                log_cstr((data) as *const _)
            ));
        }
        MSG_IDENTIFY_TERMINFO => {
            if datalen == 0 as size_t
                || *data.offset(datalen.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
                    != '\0' as i32
            {
                return -(1 as ::core::ffi::c_int);
            }
            server_client_add_term_cap(&mut *c, CStr::from_ptr(data));
            log_debug(format_args!(
                "client {} IDENTIFY_TERMINFO {}",
                log_pointer((c) as *const ::core::ffi::c_void),
                log_cstr((data) as *const _)
            ));
        }
        MSG_IDENTIFY_TTYNAME => {
            if datalen == 0 as size_t
                || *data.offset(datalen.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
                    != '\0' as i32
            {
                return -(1 as ::core::ffi::c_int);
            }
            server_client_set_ttyname(&mut *c, Some(CStr::from_ptr(data).to_owned()));
            log_debug(format_args!(
                "client {} IDENTIFY_TTYNAME {}",
                log_pointer((c) as *const ::core::ffi::c_void),
                log_cstr((data) as *const _)
            ));
        }
        MSG_IDENTIFY_CWD => {
            if datalen == 0 as size_t
                || *data.offset(datalen.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
                    != '\0' as i32
            {
                return -(1 as ::core::ffi::c_int);
            }
            if access(data, X_OK) == 0 as ::core::ffi::c_int {
                server_client_set_cwd(&mut *c, Some(CStr::from_ptr(data).to_owned()));
            } else {
                if let Some(home) = find_home_cstr() {
                    server_client_set_cwd(&mut *c, Some(home.to_owned()));
                } else {
                    server_client_set_cwd(&mut *c, Some(CString::new("/").unwrap()));
                }
            }
            log_debug(format_args!(
                "client {} IDENTIFY_CWD {}",
                log_pointer((c) as *const ::core::ffi::c_void),
                log_cstr((data) as *const _)
            ));
        }
        MSG_IDENTIFY_STDIN => {
            if datalen != 0 as size_t {
                return -(1 as ::core::ffi::c_int);
            }
            (*c).fd = imsg_get_fd(imsg)
                .map(std::os::fd::IntoRawFd::into_raw_fd)
                .unwrap_or(-1);
            log_debug(format_args!(
                "client {} IDENTIFY_STDIN {}",
                log_pointer((c) as *const ::core::ffi::c_void),
                ((*c).fd) as i32
            ));
        }
        MSG_IDENTIFY_STDOUT => {
            if datalen != 0 as size_t {
                return -(1 as ::core::ffi::c_int);
            }
            (*c).out_fd = imsg_get_fd(imsg)
                .map(std::os::fd::IntoRawFd::into_raw_fd)
                .unwrap_or(-1);
            log_debug(format_args!(
                "client {} IDENTIFY_STDOUT {}",
                log_pointer((c) as *const ::core::ffi::c_void),
                ((*c).out_fd) as i32
            ));
        }
        MSG_IDENTIFY_ENVIRON => {
            if datalen == 0 as size_t
                || *data.offset(datalen.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
                    != '\0' as i32
            {
                return -(1 as ::core::ffi::c_int);
            }
            if !strchr(data, '=' as i32).is_null() {
                environ_put(
                    (*c).environ.as_deref_mut().expect("environment"),
                    data,
                    0 as ::core::ffi::c_int,
                );
            }
            log_debug(format_args!(
                "client {} IDENTIFY_ENVIRON {}",
                log_pointer((c) as *const ::core::ffi::c_void),
                log_cstr((data) as *const _)
            ));
        }
        MSG_IDENTIFY_CLIENTPID => {
            if datalen != ::core::mem::size_of::<pid_t>() as usize {
                return -(1 as ::core::ffi::c_int);
            }
            memcpy(
                &raw mut (*c).pid as *mut ::core::ffi::c_void,
                data as *const ::core::ffi::c_void,
                ::core::mem::size_of::<pid_t>() as size_t,
            );
            log_debug(format_args!(
                "client {} IDENTIFY_CLIENTPID {}",
                log_pointer((c) as *const ::core::ffi::c_void),
                (*c).pid as ::core::ffi::c_long
            ));
        }
        _ => {}
    }
    if imsg.hdr.type_0 != MSG_IDENTIFY_DONE {
        return 0 as ::core::ffi::c_int;
    }
    (*c).flags |= CLIENT_IDENTIFIED as uint64_t;
    server_client_ensure_term_name(&mut *c);
    if !(*c).ttyname.is_none()
        && *(*c).ttyname.as_ref().unwrap().as_ptr() as ::core::ffi::c_int != '\0' as i32
    {
        let name = ((*c).ttyname)
            .as_deref()
            .expect("string is present")
            .to_owned();
        server_client_set_name(&mut *c, Some(name));
    } else {
        let name = CString::new(format!("client-{}", (*c).pid as ::core::ffi::c_long)).unwrap();
        server_client_set_name(&mut *c, Some(name));
    }
    log_debug(format_args!(
        "client {} name is {}",
        log_pointer((c) as *const ::core::ffi::c_void),
        log_cstr(
            (((*c).name)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                as *const _
        )
    ));
    if (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
        control_start(owner);
    } else if (*c).fd != -(1 as ::core::ffi::c_int) {
        if tty_init(owner) != 0 as ::core::ffi::c_int {
            close((*c).fd);
            (*c).fd = -(1 as ::core::ffi::c_int);
        } else {
            (*c).tty.r.ensure(1);
            tty_resize(&(*c).observer.upgrade().expect("live client"));
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
            (*(c)).observer.upgrade().expect("live client"),
        );
    }
    if (*c).flags as ::core::ffi::c_ulonglong & (CLIENT_BRACKETPASTING | CLIENT_ASSUMEPASTING) != 0
        && current_time - (*c).paste_time > CLIENT_PASTE_TIME_LIMIT as time_t
    {
        log_debug(format_args!(
            "{}: paste time limit exceeded",
            log_cstr(
                (((*c).name)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                    as *const _
            )
        ));
        (*c).flags = ((*c).flags as ::core::ffi::c_ulonglong
            & !(CLIENT_BRACKETPASTING | CLIENT_ASSUMEPASTING)) as uint64_t;
    }
    if !(*c).flags & CLIENT_EXIT as uint64_t != 0
        && cfg_finished == 0
        && clients.first().is_some_and(|owner| owner.get() == c)
    {
        start_cfg();
    }
    return 0 as ::core::ffi::c_int;
}
unsafe fn server_client_dispatch_shell(c: &client) -> ::core::ffi::c_int {
    let mut shell: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let shell_value = options_get_string(
        global_s_options,
        b"default-shell\0" as *const u8 as *const ::core::ffi::c_char,
    );
    shell = shell_value.as_ptr();
    if checkshell(shell) == 0 {
        shell = _PATH_BSHELL.as_ptr();
    }
    proc_send(
        c.peer,
        MSG_SHELL,
        -(1 as ::core::ffi::c_int),
        shell as *const ::core::ffi::c_void,
        strlen(shell).wrapping_add(1 as size_t),
    );
    proc_kill_peer(c.peer);
    return 0 as ::core::ffi::c_int;
}
/// Copy the selected directory while any observed startup client is retained.
pub unsafe fn server_client_get_cwd(
    c: Option<&ClientRef>,
    s: Option<&SessionRef>,
) -> Option<CString> {
    use crate::src::session::Session;
    if let Some(client) = c {
        return client.cwd(s);
    }
    if cfg_finished == 0 {
        if let Some(owner) = (&cfg_client).upgrade() {
            return (*owner.get()).cwd.clone();
        }
    }
    if let Some(session) = s {
        if let Some(cwd) = session.cwd() {
            return Some(cwd);
        }
    }
    Some(find_home_cstr().unwrap_or(c"/").to_owned())
}
unsafe fn server_client_control_flags(
    c_owner: &ClientRef,
    mut next: *const ::core::ffi::c_char,
) -> uint64_t {
    let mut c = c_owner.get();
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
pub unsafe fn server_client_set_flags(c_owner: &ClientRef, mut flags: *const ::core::ffi::c_char) {
    let mut c = c_owner.get();
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
            flag = server_client_control_flags(c_owner, next);
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
        log_debug(format_args!(
            "client {} set flag {}",
            log_cstr(
                (((*c).name)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                    as *const _
            ),
            log_cstr((next) as *const _)
        ));
        if not != 0 {
            if (*c).flags & CLIENT_READONLY as uint64_t != 0 {
                flag &= !CLIENT_READONLY as uint64_t;
            }
            (*c).flags &= !flag;
        } else {
            (*c).flags |= flag;
        }
        if flag == CLIENT_CONTROL_NOOUTPUT as uint64_t {
            control_reset_offsets(&(*c).observer.upgrade().expect("live client"));
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
unsafe fn server_client_get_flags(c: &client) -> *const ::core::ffi::c_char {
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
        xformat(
            &mut tmp,
            format_args!(
                "pause-after={},",
                ((*c).pause_age.wrapping_div(1000 as u_int)) as u32
            ),
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
pub unsafe fn server_client_remove_pane(
    wp_owner: &std::rc::Rc<std::cell::UnsafeCell<window_pane>>,
) {
    let mut wp = wp_owner.get();
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut registry_c_owner = clients.first();
    c = registry_c_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    while !c.is_null() {
        if (*c).tty.mouse_last_pane == (*wp).id as ::core::ffi::c_int {
            (*c).tty.mouse_last_pane = -(1 as ::core::ffi::c_int);
            (*c).tty.mouse_drag_update = None;
            (*c).tty.mouse_scrolling_flag = 0 as ::core::ffi::c_int;
        }
        registry_c_owner =
            clients.next(registry_c_owner.as_ref().expect("current registry client"));
        c = registry_c_owner
            .as_ref()
            .map_or(std::ptr::null_mut(), |owner| owner.get());
    }
}
pub unsafe fn server_client_print(
    client_owner: Option<&ClientRef>,
    mut parse: ::core::ffi::c_int,
    mut evb: &mut SegmentedBuf,
) {
    let c = client_owner.map_or(std::ptr::null_mut(), |owner| owner.get());
    let mut data: *mut ::core::ffi::c_void = evbuffer_pullup(evb, -1)
        .map_or(std::ptr::null_mut(), |bytes| bytes.as_mut_ptr())
        as *mut ::core::ffi::c_void;
    let mut size: size_t = evbuffer_get_length(&*(evb));
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut wme: refbox::Weak<window_mode_entry> = refbox::Weak::new();
    let mut msg: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut line: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut empty: ::core::ffi::c_char = '\0' as i32 as ::core::ffi::c_char;
    let mut escaped = Vec::new();
    if parse == 0 {
        let input = if size == 0 {
            &[][..]
        } else {
            std::slice::from_raw_parts(data.cast::<u8>(), size)
        };
        escaped = utf8_stravisx_bytes(input, VIS_OCTAL | VIS_CSTYLE | VIS_NOSLASH);
        escaped.push(0);
        msg = escaped.as_mut_ptr().cast();
    } else if size == 0 as size_t {
        msg = &raw mut empty;
    } else {
        msg = evbuffer_pullup(evb, -1).map_or(std::ptr::null_mut(), |bytes| bytes.as_mut_ptr())
            as *mut ::core::ffi::c_char;
        if *msg.offset(size.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int != '\0' as i32
        {
            evbuffer_add(
                evb,
                b"\0" as *const u8 as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
                1 as size_t,
            );
            msg = evbuffer_pullup(evb, -1)
                .expect("nonempty print buffer")
                .as_mut_ptr()
                .cast();
        }
    }
    log_debug(format_args!(
        "{}: {}",
        "server_client_print",
        log_cstr((msg) as *const _)
    ));
    if !c.is_null() {
        if (*c).session_handle().is_none() || (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
            if !(*c).flags & CLIENT_UTF8 as uint64_t != 0 {
                let sanitized = utf8_sanitize_cstring(CStr::from_ptr(msg));
                if (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
                    control_write(&(*(c)).observer.upgrade().expect("live client"), |out| {
                        write_cstr(out, sanitized.as_ptr())
                    });
                } else {
                    file_print(client_owner, |out| {
                        write_cstr(out, sanitized.as_ptr())?;
                        out.write_all(b"\n")
                    });
                }
            } else if (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
                control_write(&(*(c)).observer.upgrade().expect("live client"), |out| {
                    write_cstr(out, msg)
                });
            } else {
                file_print(client_owner, |out| {
                    write_cstr(out, msg)?;
                    out.write_all(b"\n")
                });
            }
        } else {
            wp = ((((*c)
                .session_handle()
                .expect("live session")
                .current_winlink())
            .get_unchecked()
            .window_handle()
            .as_ref())
            .expect("live window"))
            .active_pane()
            .as_ref()
            .map_or(std::ptr::null_mut(), |owner| owner.get());
            let pane_owner = (*wp).observer.upgrade().expect("view-mode pane");
            wme = (*wp).active_mode_entry();
            if !wme.is_alive() || !std::ptr::eq(wme.get_unchecked().mode, &window_view_mode) {
                window_pane_set_mode(
                    &pane_owner,
                    None,
                    &window_view_mode,
                    None,
                    ::core::ptr::null_mut::<cmd_find_state>(),
                    ::core::ptr::null_mut::<args>(),
                );
            }
            if parse != 0 {
                loop {
                    let Some(line) = evbuffer_readln(evb) else {
                        break;
                    };
                    window_copy_add(&pane_owner, 1 as ::core::ffi::c_int, |out| {
                        write_cstr(out, line.as_ptr().cast::<::core::ffi::c_char>())
                    });
                }
                size = evbuffer_get_length(&*(evb));
                if size != 0 as size_t {
                    line = evbuffer_pullup(evb, -1)
                        .map_or(std::ptr::null_mut(), |bytes| bytes.as_mut_ptr())
                        as *mut ::core::ffi::c_char;
                    window_copy_add(&pane_owner, 1 as ::core::ffi::c_int, |out| {
                        write_cstr_n(out, line, (size as ::core::ffi::c_int) as i32)
                    });
                }
            } else {
                window_copy_add(&pane_owner, 0 as ::core::ffi::c_int, |out| {
                    write_cstr(out, msg)
                });
            }
        }
    }
}
unsafe fn server_client_report_theme(c_owner: &ClientRef, mut theme: client_theme) {
    let mut c = c_owner.get();
    let mut old: client_theme = (*c).theme;
    if theme as ::core::ffi::c_uint == THEME_LIGHT as ::core::ffi::c_int as ::core::ffi::c_uint {
        (*c).theme = THEME_LIGHT;
        events_fire_client(
            b"client-light-theme\0" as *const u8 as *const ::core::ffi::c_char,
            (*(c)).observer.upgrade().expect("live client"),
        );
    } else {
        (*c).theme = THEME_DARK;
        events_fire_client(
            b"client-dark-theme\0" as *const u8 as *const ::core::ffi::c_char,
            (*(c)).observer.upgrade().expect("live client"),
        );
    }
    if (*c).theme as ::core::ffi::c_uint != old as ::core::ffi::c_uint {
        server_client_update_theme_colours(Some(c_owner));
        c_owner.with_terminal_output(|terminal| {
            if terminal.flags & TTY_OPENED != 0 {
                tty_invalidate(terminal);
            }
        });
        (*c).flags |= CLIENT_ALLREDRAWFLAGS as u64;
    }
    (*c).observer
        .upgrade()
        .expect("live client")
        .with_terminal_output(|terminal| tty_repeat_requests(terminal, 1 as ::core::ffi::c_int));
}

#[cfg(test)]
mod client_registry_tests {
    use super::{client, ClientRegistry};

    #[test]
    fn client_links_handle_head_tail_removal_and_append() {
        use std::rc::Rc;
        unsafe {
            let mut registry = ClientRegistry::new();
            let first = client::new();
            let middle = client::new();
            let last = client::new();
            let appended = client::new();
            for owner in [&first, &middle, &last] {
                registry.push_back(owner.clone());
            }

            let removed = registry.remove(&Rc::downgrade(&first)).unwrap();
            assert!(Rc::ptr_eq(&registry.first().unwrap(), &middle));
            assert!(Rc::ptr_eq(&registry.next(&removed).unwrap(), &middle));
            super::server_client_unref_owned(removed);

            let removed = registry.remove(&Rc::downgrade(&last)).unwrap();
            assert!(registry.next(&middle).is_none());
            registry.push_back(appended.clone());
            assert!(Rc::ptr_eq(&registry.next(&middle).unwrap(), &appended));
            assert!(registry.next(&removed).is_none());
            assert!(registry.next(&appended).is_none());
            super::server_client_unref_owned(removed);

            registry.clear();
            assert!(registry.first().is_none());
            assert!(registry.next(&middle).is_none());
            crate::src::reactor::poll_runtime();
        }
    }

    #[test]
    fn client_index_preserves_order_after_removal() {
        use std::rc::Rc;
        unsafe {
            let mut registry = ClientRegistry::new();
            let first = client::new();
            let middle = client::new();
            let last = client::new();
            for owner in [&first, &middle, &last] {
                registry.push_back(owner.clone());
            }
            assert!(Rc::ptr_eq(&registry.first().unwrap(), &first));
            assert!(Rc::ptr_eq(&registry.next(&first).unwrap(), &middle));
            let held_successor = registry.next(&middle).unwrap();
            assert!(Rc::ptr_eq(&held_successor, &last));
            let middle_observer = Rc::downgrade(&middle);
            let middle_owner = registry.remove(&middle_observer).unwrap();
            assert!(Rc::ptr_eq(&middle_owner, &middle));
            assert!(registry.remove(&middle_observer).is_none());
            super::server_client_unref_owned(middle_owner);
            assert!(Rc::ptr_eq(&registry.next(&first).unwrap(), &last));
            assert!(Rc::ptr_eq(&registry.next(&middle).unwrap(), &last));
            let last_observer = Rc::downgrade(&last);
            registry.clear();
            drop(first);
            drop(middle);
            drop(last);
            crate::src::reactor::poll_runtime();
            assert!(middle_observer.upgrade().is_none());
            assert!(
                last_observer.upgrade().is_some(),
                "saved successor retains its client"
            );
            drop(held_successor);
            assert!(last_observer.upgrade().is_none());
            assert!(registry.first().is_none());
        }
    }
}

#[cfg(test)]
mod key_event_owner_tests {
    use super::{client, key_event, server_client_handle_key, QueuedKeyEvent};

    #[test]
    fn absent_and_empty_bytes_remain_distinct_in_snapshots() {
        let mouse = Default::default();
        let absent = key_event::new(1, mouse, None);
        let empty = key_event::new(1, mouse, Some(Vec::new()));
        assert!(absent.bytes_ptr_len().is_none());
        assert_eq!(empty.bytes_ptr_len(), Some([].as_slice()));
        assert!(empty.metadata_snapshot().bytes.is_none());
    }

    #[test]
    fn early_return_and_cancellation_release_owned_events() {
        unsafe {
            let owner = client::new();
            let mouse = Default::default();
            assert_eq!(
                server_client_handle_key(&owner, key_event::new(1, mouse, Some(vec![1]))),
                0
            );

            let mut queued = key_event::new(2, mouse, Some(vec![2]));
            queued.client = std::rc::Rc::downgrade(&owner);
            drop(QueuedKeyEvent(queued, Some(owner.clone())));
            assert_eq!(std::rc::Rc::strong_count(&owner), 2);
            crate::src::reactor::poll_runtime();
            assert_eq!(std::rc::Rc::strong_count(&owner), 1);
        }
    }
}

impl Drop for client {
    fn drop(&mut self) {
        unsafe { server_client_free(self) }
    }
}

#[cfg(test)]
mod overlay_dispatch_tests {
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;

    unsafe fn with_client(test: impl FnOnce(&ClientRef)) {
        let session_owner = session::new();
        let session = Some(session_owner.clone());
        let link = crate::src::session::test_support::add_link(&session_owner, 0);
        crate::src::session::test_support::current(&session_owner, link.clone());
        let client_owner = client::new();
        let client = client_owner.get();
        (*client).session = Rc::downgrade(&session_owner);
        (*client).tty.client = Rc::downgrade(&client_owner);
        test(&client_owner);
        client_owner.clear_overlay();
        (*client).session = std::rc::Weak::new();
        crate::src::session::test_support::current(&session_owner, refbox::Weak::new());
        crate::src::session::test_support::remove_link(&session_owner, link);
    }

    unsafe fn install(owner: &ClientRef) {
        let c = owner.get();
        server_client_set_overlay(
            &(*(c)).observer.upgrade().expect("live client"),
            Overlay::callbacks(None, None, Some(Box::new(|_| {})), None, None, None),
        );
    }

    #[test]
    fn callbacks_clearing_the_overlay_are_not_restored_and_stale_results_are_discarded() {
        unsafe {
            with_client(|owner| {
                let c = owner.get();
                install(&(*(c)).observer.upgrade().expect("live client"));
                let freed = Rc::new(Cell::new(0));
                let observed = freed.clone();
                (*c).overlay.current.as_mut().unwrap().free =
                    Some(Box::new(move |_| observed.set(observed.get() + 1)));
                (*c).overlay.current.as_mut().unwrap().draw = Some(Box::new(|c| {
                    // Draw has been taken out of the client during dispatch.
                    assert!((*c.get()).overlay.current.as_mut().unwrap().draw.is_none());
                    c.clear_overlay();
                    assert!((*c.get()).overlay.current.is_none());
                }));
                server_client_overlay_draw(&(*(c)).observer.upgrade().expect("live client"));
                assert!(!(*c).overlay.has_draw());
                assert_eq!(freed.get(), 1);

                install(&(*(c)).observer.upgrade().expect("live client"));
                (*c).overlay.current.as_mut().unwrap().resize =
                    Some(Box::new(|c| c.clear_overlay()));
                server_client_overlay_resize(&(*(c)).observer.upgrade().expect("live client"));
                assert!(!(*c).overlay.has_resize());

                install(&(*(c)).observer.upgrade().expect("live client"));
                let owned_screen = Box::new(screen::empty());
                let screen = ScreenMode::from(&*owned_screen);
                (*c).overlay.current.as_mut().unwrap().mode = Some(Box::new(move |c| {
                    let _keep_screen_alive = &owned_screen;
                    c.clear_overlay();
                    Some((screen, 1, 2))
                }));
                assert!(server_client_overlay_mode(
                    &(*(c)).observer.upgrade().expect("live client")
                )
                .is_none());
                assert!(!(*c).overlay.has_mode());

                install(&(*(c)).observer.upgrade().expect("live client"));
                (*c).overlay.current.as_mut().unwrap().check = Some(Box::new(|c, _, _, _| {
                    c.clear_overlay();
                    visible_ranges::default()
                }));
                let ranges = crate::src::tty::tty_check_overlay_range(
                    &(*c).observer.upgrade().unwrap(),
                    3,
                    4,
                    5,
                );
                assert_eq!(ranges.used, 1);
                assert_eq!(ranges.storage[0].px, 3);
                assert_eq!(ranges.storage[0].nx, 5);
                assert!(!(*c).overlay.clips_output());
            });
        }
    }

    #[test]
    fn a_retired_key_callback_cannot_close_or_replace_the_new_overlay() {
        unsafe {
            with_client(|owner| {
                let c = owner.get();
                install(&(*(c)).observer.upgrade().expect("live client"));
                let generation = (*c).overlay.generation;
                (*c).overlay.current.as_mut().unwrap().key = Some(Box::new(|c, _| {
                    install(c);
                    (*c.get()).overlay.current.as_mut().unwrap().key = Some(Box::new(|_, _| 42));
                    1
                }));
                let mut event = key_event::new(0, mouse_event::default(), None);
                assert_eq!(
                    server_client_overlay_key(
                        &(*(c)).observer.upgrade().expect("live client"),
                        &mut event
                    ),
                    Some(0)
                );
                assert!((*c).overlay.generation > generation);
                assert!((*c).overlay.current.is_some());
                assert_eq!(
                    server_client_overlay_key(
                        &(*(c)).observer.upgrade().expect("live client"),
                        &mut event
                    ),
                    Some(42)
                );
            });
        }
    }

    #[test]
    fn cleanup_preserves_replacement_flags_and_set_retires_nested_owners() {
        unsafe {
            with_client(|owner| {
                let c = owner.get();
                install(&(*(c)).observer.upgrade().expect("live client"));
                (*c).overlay.current.as_mut().unwrap().free = Some(Box::new(|c| install(c)));
                server_client_clear_overlay(&(*(c)).observer.upgrade().expect("live client"));
                assert!((*c).overlay.current.is_some());
                assert_eq!(
                    (*c).tty.flags & (TTY_FREEZE | TTY_NOCURSOR),
                    TTY_FREEZE | TTY_NOCURSOR
                );

                let nested_frees = Rc::new(Cell::new(0));
                let observed = nested_frees.clone();
                (*c).overlay.current.as_mut().unwrap().free = Some(Box::new(move |c| {
                    install(c);
                    (*c.get()).overlay.current.as_mut().unwrap().free =
                        Some(Box::new(move |_| observed.set(observed.get() + 1)));
                }));
                install(&(*(c)).observer.upgrade().expect("live client"));
                assert_eq!(nested_frees.get(), 1);
                assert!((*c).overlay.current.is_some());
            });
        }
    }

    #[test]
    fn explicit_free_runs_after_detach_and_before_callback_capture_destruction() {
        struct DropProbe(Rc<Cell<bool>>);
        impl Drop for DropProbe {
            fn drop(&mut self) {
                self.0.set(true);
            }
        }

        unsafe {
            with_client(|owner| {
                let dropped = Rc::new(Cell::new(false));
                let probe = DropProbe(dropped.clone());
                let observed = dropped.clone();
                owner.set_overlay(Overlay::callbacks(
                    None,
                    None,
                    Some(Box::new(move |_| {
                        let _keep_probe_alive = &probe;
                    })),
                    None,
                    Some(Box::new(move |client| {
                        assert!((*client.get()).overlay.current.is_none());
                        assert!(!observed.get());
                        install(client);
                    })),
                    None,
                ));
                owner.clear_overlay();
                assert!(dropped.get());
                assert!((*owner.get()).overlay.current.is_some());
                assert_eq!(
                    (*owner.get()).tty.flags & (TTY_FREEZE | TTY_NOCURSOR),
                    TTY_FREEZE | TTY_NOCURSOR
                );
                owner.clear_overlay();
                assert!((*owner.get()).overlay.current.is_none());
            });
        }
    }
}

#[cfg(test)]
mod client_timer_observer_tests {
    use super::*;

    #[test]
    fn click_timer_updates_live_client_and_can_outlive_it_without_retaining_it() {
        unsafe {
            let owner = client::new();
            let observer = std::rc::Rc::downgrade(&owner);
            server_client_init_timers(&owner);
            (*owner.get()).flags |= CLIENT_DOUBLECLICK as uint64_t;
            let immediate = Duration::ZERO;
            (*owner.get())
                .click_timer
                .arm(immediate)
                .expect("arm timer");
            crate::src::reactor::poll_runtime();
            assert_eq!((*owner.get()).flags & CLIENT_DOUBLECLICK as uint64_t, 0);
            let mut detached_timer = std::mem::take(&mut (*owner.get()).click_timer);
            drop(owner);
            assert!(
                observer.upgrade().is_none(),
                "timer callbacks must not retain clients"
            );
            detached_timer.arm(immediate).expect("arm timer");
            crate::src::reactor::poll_runtime();
            assert!(observer.upgrade().is_none());
            detached_timer.cancel();
        }
    }
}

#[cfg(test)]
mod cwd_observer_ownership_tests {
    use crate::src::cfg::{cfg_client, cfg_finished};
    use crate::src::server_client::server_client_get_cwd;
    use crate::src::session::Session;
    use crate::src::shared::{client::client, session::session};
    use std::rc::{Rc, Weak};

    #[test]
    fn startup_observer_and_owned_directory_preserve_lifetime_and_precedence() {
        unsafe {
            let startup = client::new();
            (*startup.get()).cwd = Some(c"/startup".to_owned());
            cfg_client = Rc::downgrade(&startup);
            cfg_finished = 0;
            let other = client::new();
            (*other.get()).cwd = Some(c"/other".to_owned());
            let session = session::new();
            session.set_cwd(Some(c"/session".to_owned()));
            let saved = server_client_get_cwd(Some(&other), Some(&session)).unwrap();
            assert_eq!(saved.as_c_str(), c"/startup");
            (*startup.get()).cwd = None;
            assert!(server_client_get_cwd(Some(&other), None).is_none());
            drop(startup);
            assert!((&cfg_client).upgrade().is_none());
            assert_eq!(saved.as_c_str(), c"/startup");
            assert_eq!(
                server_client_get_cwd(Some(&other), Some(&session))
                    .unwrap()
                    .as_c_str(),
                c"/other",
            );
            assert_eq!(
                server_client_get_cwd(None, Some(&session))
                    .unwrap()
                    .as_c_str(),
                c"/session",
            );
            cfg_client = Weak::new();
            cfg_finished = 1;
        }
    }
}
