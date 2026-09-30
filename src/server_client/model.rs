//! Authoritative client objects, file transfers, overlays, and scalar domains.
use crate::src::server_client::server_client_unref_owned;
use crate::src::shared::client::{ClientRef, ClientWeak};
use crate::src::shared::session::{SessionRef, SessionWeak};
use crate::src::shared::window::{WindowRef, WindowWeak};
use hmux_buffer::SegmentedBuf;
use std::cell::UnsafeCell;
use std::rc::Rc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::src::compat::imsg::msgtype;
use crate::src::shared::abi::{pid_t, size_t, time_t, u_int, uint64_t};
use crate::src::shared::colour::client_theme;
use crate::src::shared::command::cmdq_list;
use crate::src::shared::control::control_state;
use crate::src::shared::display::{progress_bar, visible_ranges};
use crate::src::shared::environment::environ;
use crate::src::shared::event::{bufferevent, Timer};
use crate::src::shared::format::format_job_tree;
use crate::src::shared::key::{key_code, key_event, key_table};
use crate::src::shared::mouse::mouse_event;
use crate::src::shared::process::tmuxpeer;
use crate::src::shared::redraw::redraw_scene;
use crate::src::shared::screen::screen;
use crate::src::shared::session::session;
use crate::src::shared::status::status_line;
use crate::src::shared::tty::tty;
use std::ffi::CStr;

use crate::src::shared::client::*;
pub struct client {
    /// Nonowning allocation observer for callbacks receiving borrowed pointers.
    pub(super) observer: ClientWeak,
    // Preserve after removal so a traversal holding this client can continue.
    pub(super) registry_next: ClientWeak,
    pub(super) name: Option<std::ffi::CString>,
    pub(super) peer: *mut tmuxpeer,
    pub(super) user: Option<std::ffi::CString>,
    pub(super) queue: Option<Box<cmdq_list>>,
    pub(super) control_state: Option<Box<control_state>>,
    pub(super) pause_age: u_int,
    pub(super) pid: pid_t,
    pub(super) fd: ::core::ffi::c_int,
    pub(super) out_fd: ::core::ffi::c_int,
    pub(super) retval: ::core::ffi::c_int,
    pub(super) creation_time: SystemTime,
    pub(super) activity_time: SystemTime,
    pub(super) last_activity_time: SystemTime,
    pub(super) environ: Option<Box<environ>>,
    /// Sole owner of this client's lazily allocated format-job cache.
    pub(super) jobs: Option<Box<format_job_tree>>,
    pub(super) title: Option<std::ffi::CString>,
    pub(super) path: Option<std::ffi::CString>,
    pub(super) cwd: Option<std::ffi::CString>,
    pub(super) progress_bar: progress_bar,
    pub(super) term_name: Option<std::ffi::CString>,
    pub(super) term_features: ::core::ffi::c_int,
    pub(super) term_nofeatures: ::core::ffi::c_int,
    pub(super) term_type: Option<std::ffi::CString>,
    pub(super) term_caps: Vec<std::ffi::CString>,
    pub(super) ttyname: Option<std::ffi::CString>,
    pub(super) tty: tty,
    pub(super) written: size_t,
    pub(super) discarded: size_t,
    pub(super) redraw: size_t,
    pub(super) redraw_scene: Option<Box<redraw_scene>>,
    pub(super) repeat_timer: Timer,
    pub(super) click_timer: Timer,
    pub(super) click_loc: ::core::ffi::c_int,
    pub(super) click_wp: ::core::ffi::c_int,
    pub(super) exit_timer: Timer,
    pub(super) click_button: u_int,
    pub(super) click_event: mouse_event,
    pub(super) status: status_line,
    pub(super) cycle_timer: Timer,
    pub(super) theme: client_theme,
    pub(super) input_requests: Vec<*mut crate::src::shared::input::input_request>,
    pub(super) flags: uint64_t,
    pub(super) exit_type: client_exit_type,
    pub(super) exit_msgtype: msgtype,
    pub(super) exit_session: Option<std::ffi::CString>,
    pub(super) exit_message: Option<std::ffi::CString>,
    pub(super) keytable: Option<std::rc::Rc<std::cell::RefCell<key_table>>>,
    pub(super) last_key: key_code,
    pub(super) paste_time: time_t,
    pub(super) message_ignore_keys: ::core::ffi::c_int,
    pub(super) message_ignore_styles: ::core::ffi::c_int,
    pub(super) message_string: Option<std::ffi::CString>,
    pub(super) message_timer: Timer,
    pub(super) prompt: Option<refbox::RefBox<crate::src::shared::prompt::prompt>>,
    /// Attached session identity; the session index owns the Rc allocation.
    pub(super) session: SessionWeak,
    /// Previous session observer; this link never keeps a session alive.
    pub(super) last_session: SessionWeak,
    pub(super) theme_colours: [::core::ffi::c_int; 10],
    /// Window whose manual pan offsets are active; this does not retain it.
    pub(super) pan_window: WindowWeak,
    pub(super) pan_ox: u_int,
    pub(super) pan_oy: u_int,
    pub(super) overlay_generation: u64,
    pub(super) overlay_check: overlay_check_cb,
    pub(super) overlay_mode: overlay_mode_cb,
    pub(super) overlay_draw: overlay_draw_cb,
    pub(super) overlay_key: overlay_key_cb,
    pub(super) overlay_free: overlay_free_cb,
    pub(super) overlay_resize: overlay_resize_cb,
    pub(super) overlay_data: Option<Box<dyn std::any::Any>>,
    pub(super) files: client_files,
    pub(super) source_file_depth: u_int,
}

impl client {
    #[cfg(test)]
    pub(crate) unsafe fn with_session_for_test(session: Option<&SessionRef>) -> ClientRef {
        let owner = Self::new();
        (*owner.get()).set_session(session);
        owner
    }

    /// Retain the attached session for the current operation.
    pub(super) fn session_handle(&self) -> Option<SessionRef> {
        self.session.upgrade()
    }

    /// The caller supplies a live Rc-backed session.
    pub(super) fn set_session(&mut self, session: Option<&SessionRef>) {
        self.session = session.map_or_else(std::rc::Weak::new, Rc::downgrade);
    }

    #[cfg(test)]
    pub(super) fn pan_window_is(&self, window: &WindowRef) -> bool {
        self.pan_window.strong_count() != 0
            && std::rc::Weak::ptr_eq(&self.pan_window, &Rc::downgrade(window))
    }

    #[cfg(test)]
    pub(super) fn set_pan_window(&mut self, window: &WindowRef) {
        self.pan_window = Rc::downgrade(window);
    }

    pub fn empty() -> Self {
        Self {
            observer: std::rc::Weak::new(),
            registry_next: std::rc::Weak::new(),
            name: Default::default(),
            peer: Default::default(),
            user: Default::default(),
            queue: Default::default(),
            control_state: Default::default(),
            pause_age: Default::default(),
            pid: Default::default(),
            fd: Default::default(),
            out_fd: Default::default(),
            retval: Default::default(),
            creation_time: UNIX_EPOCH,
            activity_time: UNIX_EPOCH,
            last_activity_time: UNIX_EPOCH,
            environ: Default::default(),
            jobs: Default::default(),
            title: Default::default(),
            path: Default::default(),
            cwd: Default::default(),
            progress_bar: Default::default(),
            term_name: Default::default(),
            term_features: Default::default(),
            term_nofeatures: Default::default(),
            term_type: Default::default(),
            term_caps: Vec::new(),
            ttyname: Default::default(),
            tty: tty::empty(),
            written: Default::default(),
            discarded: Default::default(),
            redraw: Default::default(),
            redraw_scene: Default::default(),
            repeat_timer: Default::default(),
            click_timer: Default::default(),
            click_loc: Default::default(),
            click_wp: Default::default(),
            exit_timer: Default::default(),
            click_button: Default::default(),
            click_event: Default::default(),
            status: status_line::empty(),
            cycle_timer: Default::default(),
            theme: Default::default(),
            input_requests: Vec::new(),
            flags: Default::default(),
            exit_type: Default::default(),
            exit_msgtype: Default::default(),
            exit_session: Default::default(),
            exit_message: Default::default(),
            keytable: Default::default(),
            last_key: Default::default(),
            paste_time: Default::default(),
            message_ignore_keys: Default::default(),
            message_ignore_styles: Default::default(),
            message_string: None,
            message_timer: Default::default(),
            prompt: Default::default(),
            session: Default::default(),
            last_session: Default::default(),
            theme_colours: Default::default(),
            pan_window: Default::default(),
            pan_ox: Default::default(),
            pan_oy: Default::default(),
            overlay_generation: 0,
            overlay_check: Default::default(),
            overlay_mode: Default::default(),
            overlay_draw: Default::default(),
            overlay_key: Default::default(),
            overlay_free: Default::default(),
            overlay_resize: Default::default(),
            overlay_data: None,
            files: Default::default(),
            source_file_depth: Default::default(),
        }
    }
}

/// Retain an optional live, Rc-owned client.
/// Panics if a supplied client is not backed by a live Rc allocation.
pub(super) fn client_retain(value: Option<&client>) -> Option<ClientRef> {
    value.map(|client| client.observer.upgrade().expect("live Rc client"))
}

#[cfg(test)]
impl client {
    /// No descriptors or monitors are installed; tests explicitly stop control
    /// after populating the component through its borrow API.
    pub(crate) unsafe fn with_control_for_test(
        name: Option<&std::ffi::CStr>,
        session: Option<&SessionRef>,
    ) -> ClientRef {
        let owner = Self::with_session_for_test(session);
        (*owner.get()).name = name.map(ToOwned::to_owned);
        (*owner.get()).control_state =
            Some(Box::new(crate::src::shared::control::control_state::empty()));
        owner
    }

    pub(crate) unsafe fn activity_for_test(owner: &ClientRef, seconds: u64, micros: u64) {
        (*owner.get()).activity_time =
            UNIX_EPOCH + Duration::from_secs(seconds) + Duration::from_micros(micros);
    }

    pub(crate) unsafe fn with_names_for_test(
        name: Option<&std::ffi::CStr>,
        tty_name: Option<&std::ffi::CStr>,
    ) -> ClientRef {
        let owner = Self::new();
        (*owner.get()).name = name.map(ToOwned::to_owned);
        (*owner.get()).ttyname = tty_name.map(ToOwned::to_owned);
        owner
    }

    /// A queue fixture uses the same explicit item cleanup as server clients.
    pub(crate) unsafe fn with_queue_for_test() -> ClientRef {
        let owner = Self::new();
        (*owner.get()).queue = Some(crate::src::cmd::queue::cmdq_new());
        owner
    }
}

#[cfg(test)]
mod retained_client_tests {
    use super::*;
    use crate::src::reactor;

    #[test]
    fn attached_session_observer_does_not_retain_a_removed_session() {
        let mut client = client::empty();
        let session = crate::src::shared::session::session::new();
        let observer = Rc::downgrade(&session);
        unsafe {
            client.set_session(Some(&session));
        }
        assert!(Rc::ptr_eq(
            &client.session_handle().expect("attached session"),
            &session,
        ));
        assert_eq!(Rc::strong_count(&session), 1);
        drop(session);
        assert!(observer.upgrade().is_none());
        assert!(client.session_handle().is_none());
    }

    #[test]
    fn pan_window_observes_identity_without_retaining_the_window() {
        let mut client = client::empty();
        let first = crate::src::shared::window::window::new();
        let first_observer = Rc::downgrade(&first);
        unsafe {
            let first_window = &first;
            assert!(!client.pan_window_is(first_window));
            client.set_pan_window(first_window);
            assert!(client.pan_window_is(first_window));
        }
        assert_eq!(Rc::strong_count(&first), 1);
        unsafe {
            crate::src::window::window_remove_ref(first, c"test owner".as_ptr());
        }
        assert!(first_observer.upgrade().is_none());

        let second = crate::src::shared::window::window::new();
        unsafe {
            assert!(!client.pan_window_is(&second));
            client.set_pan_window(&second);
            assert!(client.pan_window_is(&second));
        }
        assert_eq!(Rc::strong_count(&second), 1);
        unsafe {
            crate::src::window::window_remove_ref(second, c"test owner".as_ptr());
        }
    }

    #[test]
    fn owner_release_defers_cleanup_until_dispatch_or_cancellation() {
        unsafe {
            for cancel in [false, true] {
                let initial = client::new();
                let ptr = initial.get();
                let observer = (*ptr).observer.clone();
                let owner = client_retain((ptr).as_ref()).unwrap();
                drop(initial);
                assert_eq!(owner.get(), ptr);
                server_client_unref_owned(owner);
                assert_eq!(observer.strong_count(), 1);
                if cancel {
                    reactor::shutdown_runtime();
                } else {
                    reactor::poll_runtime();
                }
                assert_eq!(observer.strong_count(), 0);
                reactor::shutdown_runtime();
            }
            assert!(client_retain(None).is_none());
        }
    }
}
