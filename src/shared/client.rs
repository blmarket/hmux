//! Authoritative client objects, file transfers, overlays, and scalar domains.

use super::abi::{pid_t, size_t, time_t, timeval, u_int, uint64_t};
use super::colour::client_theme;
use super::command::cmdq_list;
use super::control::control_state;
use super::display::{progress_bar, visible_ranges};
use super::environment::environ;
use super::event::{bufferevent, evbuffer, event};
use super::format::format_job_tree;
use super::key::{key_code, key_event, key_table};
use super::message::msgtype;
use super::mouse::mouse_event;
use super::process::tmuxpeer;
use super::prompt::prompt;
use super::redraw::redraw_scene;
use super::screen::screen;
use super::session::session;
use super::status::status_line;
use super::tty::tty;
pub type client_exit_type = ::core::ffi::c_uint;

pub const CLIENT_EXIT_DETACH: client_exit_type = 2;
pub const CLIENT_EXIT_SHUTDOWN: client_exit_type = 1;
pub const CLIENT_EXIT_RETURN: client_exit_type = 0;

pub type client_exit_reason = ::core::ffi::c_uint;

pub const CLIENT_EXIT_MESSAGE_PROVIDED: client_exit_reason = 8;
pub const CLIENT_EXIT_SERVER_EXITED: client_exit_reason = 7;
pub const CLIENT_EXIT_EXITED: client_exit_reason = 6;
pub const CLIENT_EXIT_LOST_SERVER: client_exit_reason = 5;
pub const CLIENT_EXIT_TERMINATED: client_exit_reason = 4;
pub const CLIENT_EXIT_LOST_TTY: client_exit_reason = 3;
pub const CLIENT_EXIT_DETACHED_HUP: client_exit_reason = 2;
pub const CLIENT_EXIT_DETACHED: client_exit_reason = 1;
pub const CLIENT_EXIT_NONE: client_exit_reason = 0;

pub const CLIENT_CONTROL: ::core::ffi::c_int = 0x2000 as ::core::ffi::c_int;
pub const CLIENT_LOGIN: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const CLIENT_NOSTARTSERVER: ::core::ffi::c_int = 0x1000 as ::core::ffi::c_int;
pub const CLIENT_CONTROLCONTROL: ::core::ffi::c_int = 0x4000 as ::core::ffi::c_int;
pub const CLIENT_STARTSERVER: ::core::ffi::c_int = 0x10000000 as ::core::ffi::c_int;
pub const CLIENT_CONTROL_WAITEXIT: ::core::ffi::c_ulonglong =
    0x200000000 as ::core::ffi::c_ulonglong;
pub const CLIENT_WRITE_ACK: ::core::ffi::c_ulonglong = 0x4000000000 as ::core::ffi::c_ulonglong;
pub const CLIENT_ATTACHED: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const CLIENT_READONLY: ::core::ffi::c_int = 0x800 as ::core::ffi::c_int;
pub const CLIENT_IGNORESIZE: ::core::ffi::c_int = 0x20000 as ::core::ffi::c_int;
pub const CLIENT_DEAD: ::core::ffi::c_int = 0x200 as ::core::ffi::c_int;
pub const CLIENT_UTF8: ::core::ffi::c_int = 0x10000 as ::core::ffi::c_int;
pub const CLIENT_STATUSFORCE: ::core::ffi::c_int = 0x80000 as ::core::ffi::c_int;
pub const CLIENT_SIZECHANGED: ::core::ffi::c_int = 0x400000 as ::core::ffi::c_int;
pub const CLIENT_WINDOWSIZECHANGED: ::core::ffi::c_ulonglong =
    0x400000000 as ::core::ffi::c_ulonglong;
pub const CLIENT_CONTROL_NEWLAYOUTS: ::core::ffi::c_ulonglong =
    0x800000000 as ::core::ffi::c_ulonglong;
pub const CLIENT_REDRAWSTATUS: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const CLIENT_REDRAWBORDERS: ::core::ffi::c_int = 0x400 as ::core::ffi::c_int;
pub const CLIENT_EXIT: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const CLIENT_SUSPENDED: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const CLIENT_CONTROL_NOOUTPUT: ::core::ffi::c_int = 0x4000000 as ::core::ffi::c_int;
pub const CLIENT_CONTROL_PAUSEAFTER: ::core::ffi::c_ulonglong =
    0x100000000 as ::core::ffi::c_ulonglong;
pub const CLIENT_CONTROL_DISCARD: ::core::ffi::c_ulonglong =
    0x1000000000 as ::core::ffi::c_ulonglong;
pub const CLIENT_UNATTACHEDFLAGS: ::core::ffi::c_int = CLIENT_DEAD | CLIENT_SUSPENDED | CLIENT_EXIT;
pub const CLIENT_REDRAWOVERLAY: ::core::ffi::c_int = 0x2000000 as ::core::ffi::c_int;
pub const CLIENT_STATUSOFF: ::core::ffi::c_int = 0x800000 as ::core::ffi::c_int;
pub const CLIENT_NOSIZEFLAGS: ::core::ffi::c_int = CLIENT_DEAD | CLIENT_SUSPENDED | CLIENT_EXIT;
pub const CLIENT_REDRAWWINDOW: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const CLIENT_REDRAWMENU: ::core::ffi::c_int = 0x20000000 as ::core::ffi::c_int;
pub const CLIENT_IDENTIFIED: ::core::ffi::c_int = 0x40000 as ::core::ffi::c_int;
pub const CLIENT_DEFAULTSOCKET: ::core::ffi::c_int = 0x8000000 as ::core::ffi::c_int;
pub const CLIENT_NOFORK: ::core::ffi::c_int = 0x40000000 as ::core::ffi::c_int;
pub const CLIENT_PASTE_TIME_LIMIT: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
pub const CLIENT_TERMINAL: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const CLIENT_REPEAT: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const CLIENT_EXITED: ::core::ffi::c_int = 0x100 as ::core::ffi::c_int;
pub const CLIENT_FOCUSED: ::core::ffi::c_int = 0x8000 as ::core::ffi::c_int;
pub const CLIENT_DOUBLECLICK: ::core::ffi::c_int = 0x100000 as ::core::ffi::c_int;
pub const CLIENT_TRIPLECLICK: ::core::ffi::c_int = 0x200000 as ::core::ffi::c_int;
pub const CLIENT_REDRAWSTATUSALWAYS: ::core::ffi::c_int = 0x1000000 as ::core::ffi::c_int;
pub const CLIENT_REDRAWSCROLLBARS: ::core::ffi::c_ulonglong =
    0x80000000 as ::core::ffi::c_ulonglong;
pub const CLIENT_BRACKETPASTING: ::core::ffi::c_ulonglong =
    0x1000000000 as ::core::ffi::c_ulonglong;
pub const CLIENT_ASSUMEPASTING: ::core::ffi::c_ulonglong = 0x2000000000 as ::core::ffi::c_ulonglong;
pub const CLIENT_NO_DETACH_ON_DESTROY: ::core::ffi::c_ulonglong =
    0x8000000000 as ::core::ffi::c_ulonglong;
pub const CLIENT_ALLREDRAWFLAGS: ::core::ffi::c_int = CLIENT_REDRAWWINDOW
    | CLIENT_REDRAWSTATUS
    | CLIENT_REDRAWSTATUSALWAYS
    | CLIENT_REDRAWBORDERS
    | CLIENT_REDRAWOVERLAY
    | CLIENT_REDRAWMENU;
pub const CLIENT_NODETACHFLAGS: ::core::ffi::c_int = CLIENT_DEAD | CLIENT_EXIT;

#[cfg(test)]
mod tests {
    use super::{
        client_exit_reason, client_exit_type, CLIENT_EXIT_DETACH, CLIENT_EXIT_DETACHED,
        CLIENT_EXIT_MESSAGE_PROVIDED, CLIENT_EXIT_NONE,
    };
    use ::core::mem::{align_of, size_of};

    #[test]
    fn client_exit_domains_match_the_translated_c_baseline() {
        assert_eq!(size_of::<client_exit_type>(), 4);
        assert_eq!(align_of::<client_exit_type>(), 4);
        assert_eq!(size_of::<client_exit_reason>(), 4);
        assert_eq!(align_of::<client_exit_reason>(), 4);
        assert_eq!(CLIENT_EXIT_DETACH, 2);
        assert_eq!(CLIENT_EXIT_MESSAGE_PROVIDED, 8);
        assert_eq!(CLIENT_EXIT_DETACHED, 1);
        assert_eq!(CLIENT_EXIT_NONE, 0);
    }
}

#[repr(C)]
pub struct client {
    pub name: Option<std::ffi::CString>,
    pub peer: *mut tmuxpeer,
    pub user: Option<std::ffi::CString>,
    pub queue: *mut cmdq_list,
    pub control_state: *mut control_state,
    pub pause_age: u_int,
    pub pid: pid_t,
    pub fd: ::core::ffi::c_int,
    pub out_fd: ::core::ffi::c_int,
    pub event: event,
    pub retval: ::core::ffi::c_int,
    pub creation_time: timeval,
    pub activity_time: timeval,
    pub last_activity_time: timeval,
    pub environ: *mut environ,
    pub jobs: *mut format_job_tree,
    pub title: Option<std::ffi::CString>,
    pub path: Option<std::ffi::CString>,
    pub cwd: Option<std::ffi::CString>,
    pub progress_bar: progress_bar,
    pub term_name: Option<std::ffi::CString>,
    pub term_features: ::core::ffi::c_int,
    pub term_nofeatures: ::core::ffi::c_int,
    pub term_type: Option<std::ffi::CString>,
    pub term_caps: Vec<std::ffi::CString>,
    pub ttyname: Option<std::ffi::CString>,
    pub tty: tty,
    pub written: size_t,
    pub discarded: size_t,
    pub redraw: size_t,
    pub redraw_scene: *mut redraw_scene,
    pub repeat_timer: event,
    pub click_timer: event,
    pub click_loc: ::core::ffi::c_int,
    pub click_wp: ::core::ffi::c_int,
    pub exit_timer: event,
    pub click_button: u_int,
    pub click_event: mouse_event,
    pub status: status_line,
    pub cycle_timer: event,
    pub theme: client_theme,
    pub input_requests: Vec<*mut super::input::input_request>,
    pub flags: uint64_t,
    pub exit_type: client_exit_type,
    pub exit_msgtype: msgtype,
    pub exit_session: Option<std::ffi::CString>,
    pub exit_message: Option<std::ffi::CString>,
    pub keytable: *mut key_table,
    pub last_key: key_code,
    pub paste_time: time_t,
    pub message_ignore_keys: ::core::ffi::c_int,
    pub message_ignore_styles: ::core::ffi::c_int,
    pub message_string: Option<std::ffi::CString>,
    pub message_timer: event,
    pub prompt: *mut prompt,
    pub session: *mut session,
    pub last_session: *mut session,
    pub references: ::core::ffi::c_int,
    pub theme_colours: [::core::ffi::c_int; 10],
    pub pan_window: *mut ::core::ffi::c_void,
    pub pan_ox: u_int,
    pub pan_oy: u_int,
    pub overlay_check: overlay_check_cb,
    pub overlay_mode: overlay_mode_cb,
    pub overlay_draw: overlay_draw_cb,
    pub overlay_key: overlay_key_cb,
    pub overlay_free: overlay_free_cb,
    pub overlay_resize: overlay_resize_cb,
    pub overlay_data: *mut ::core::ffi::c_void,
    pub overlay_timer: event,
    pub files: client_files,
    pub source_file_depth: u_int,
    pub clipboard_panes: *mut u_int,
    pub clipboard_npanes: u_int,
    // status.active can borrow either the inline screen or this saved screen.
    // Keep ownership separate from that switching observer.
    pub(crate) saved_status_screen: Option<Box<screen>>,
}

impl client {
    pub fn empty() -> Self {
        Self {
            name: Default::default(),
            peer: unsafe { ::core::mem::zeroed() },
            user: Default::default(),
            queue: unsafe { ::core::mem::zeroed() },
            control_state: unsafe { ::core::mem::zeroed() },
            pause_age: unsafe { ::core::mem::zeroed() },
            pid: unsafe { ::core::mem::zeroed() },
            fd: unsafe { ::core::mem::zeroed() },
            out_fd: unsafe { ::core::mem::zeroed() },
            event: unsafe { ::core::mem::zeroed() },
            retval: unsafe { ::core::mem::zeroed() },
            creation_time: unsafe { ::core::mem::zeroed() },
            activity_time: unsafe { ::core::mem::zeroed() },
            last_activity_time: unsafe { ::core::mem::zeroed() },
            environ: unsafe { ::core::mem::zeroed() },
            jobs: unsafe { ::core::mem::zeroed() },
            title: Default::default(),
            path: Default::default(),
            cwd: Default::default(),
            progress_bar: unsafe { ::core::mem::zeroed() },
            term_name: Default::default(),
            term_features: unsafe { ::core::mem::zeroed() },
            term_nofeatures: unsafe { ::core::mem::zeroed() },
            term_type: Default::default(),
            term_caps: Vec::new(),
            ttyname: Default::default(),
            tty: tty::empty(),
            written: unsafe { ::core::mem::zeroed() },
            discarded: unsafe { ::core::mem::zeroed() },
            redraw: unsafe { ::core::mem::zeroed() },
            redraw_scene: unsafe { ::core::mem::zeroed() },
            repeat_timer: unsafe { ::core::mem::zeroed() },
            click_timer: unsafe { ::core::mem::zeroed() },
            click_loc: unsafe { ::core::mem::zeroed() },
            click_wp: unsafe { ::core::mem::zeroed() },
            exit_timer: unsafe { ::core::mem::zeroed() },
            click_button: unsafe { ::core::mem::zeroed() },
            click_event: unsafe { ::core::mem::zeroed() },
            status: status_line::empty(),
            cycle_timer: unsafe { ::core::mem::zeroed() },
            theme: unsafe { ::core::mem::zeroed() },
            input_requests: Vec::new(),
            flags: unsafe { ::core::mem::zeroed() },
            exit_type: unsafe { ::core::mem::zeroed() },
            exit_msgtype: unsafe { ::core::mem::zeroed() },
            exit_session: Default::default(),
            exit_message: Default::default(),
            keytable: unsafe { ::core::mem::zeroed() },
            last_key: unsafe { ::core::mem::zeroed() },
            paste_time: unsafe { ::core::mem::zeroed() },
            message_ignore_keys: unsafe { ::core::mem::zeroed() },
            message_ignore_styles: unsafe { ::core::mem::zeroed() },
            message_string: None,
            message_timer: unsafe { ::core::mem::zeroed() },
            prompt: unsafe { ::core::mem::zeroed() },
            session: unsafe { ::core::mem::zeroed() },
            last_session: unsafe { ::core::mem::zeroed() },
            references: unsafe { ::core::mem::zeroed() },
            theme_colours: unsafe { ::core::mem::zeroed() },
            pan_window: unsafe { ::core::mem::zeroed() },
            pan_ox: unsafe { ::core::mem::zeroed() },
            pan_oy: unsafe { ::core::mem::zeroed() },
            overlay_check: unsafe { ::core::mem::zeroed() },
            overlay_mode: unsafe { ::core::mem::zeroed() },
            overlay_draw: unsafe { ::core::mem::zeroed() },
            overlay_key: unsafe { ::core::mem::zeroed() },
            overlay_free: unsafe { ::core::mem::zeroed() },
            overlay_resize: unsafe { ::core::mem::zeroed() },
            overlay_data: unsafe { ::core::mem::zeroed() },
            overlay_timer: unsafe { ::core::mem::zeroed() },
            files: unsafe { ::core::mem::zeroed() },
            source_file_depth: unsafe { ::core::mem::zeroed() },
            clipboard_panes: unsafe { ::core::mem::zeroed() },
            clipboard_npanes: unsafe { ::core::mem::zeroed() },
            saved_status_screen: Default::default(),
        }
    }
}

#[repr(C)]
pub struct client_files {
    /// The client owns its stream index; file records remain externally owned.
    pub storage: Option<refbox::RefBox<std::collections::BTreeMap<i32, *mut client_file>>>,
}

#[repr(C)]
pub struct client_file {
    pub c: *mut client,
    pub peer: *mut tmuxpeer,
    pub tree: *mut client_files,
    pub references: ::core::ffi::c_int,
    pub stream: ::core::ffi::c_int,
    pub path: Option<std::ffi::CString>,
    pub buffer: *mut evbuffer,
    pub event: *mut bufferevent,
    pub fd: ::core::ffi::c_int,
    pub error: ::core::ffi::c_int,
    pub closed: ::core::ffi::c_int,
    pub cb: client_file_cb,
    pub data: *mut ::core::ffi::c_void,
    pub entry: client_file_entry,
    pub(crate) callback_data: Option<Box<dyn std::any::Any>>,
    pub(crate) wait_item: *mut super::command::cmdq_item,
    pub(crate) wait_client: *mut client,
    pub(crate) cancel_data: Option<unsafe fn(*mut ::core::ffi::c_void)>,
    pub(crate) terminal_scheduled: bool,
}

impl client_file {
    pub fn empty() -> Self {
        Self {
            c: unsafe { ::core::mem::zeroed() },
            peer: unsafe { ::core::mem::zeroed() },
            tree: unsafe { ::core::mem::zeroed() },
            references: unsafe { ::core::mem::zeroed() },
            stream: unsafe { ::core::mem::zeroed() },
            path: Default::default(),
            buffer: unsafe { ::core::mem::zeroed() },
            event: unsafe { ::core::mem::zeroed() },
            fd: unsafe { ::core::mem::zeroed() },
            error: unsafe { ::core::mem::zeroed() },
            closed: unsafe { ::core::mem::zeroed() },
            cb: unsafe { ::core::mem::zeroed() },
            data: unsafe { ::core::mem::zeroed() },
            entry: client_file_entry { owner: None },
            callback_data: Default::default(),
            wait_item: Default::default(),
            wait_client: Default::default(),
            cancel_data: Default::default(),
            terminal_scheduled: Default::default(),
        }
    }
}

#[repr(C)]
pub struct client_file_entry {
    /// Weak traversal handle into the client file index.
    pub owner: Option<refbox::Weak<std::collections::BTreeMap<i32, *mut client_file>>>,
}

pub type client_file_cb = Option<
    unsafe extern "C" fn(
        *mut client,
        *const ::core::ffi::c_char,
        ::core::ffi::c_int,
        ::core::ffi::c_int,
        *mut evbuffer,
        *mut ::core::ffi::c_void,
    ) -> (),
>;

pub type overlay_resize_cb =
    Option<unsafe extern "C" fn(*mut client, *mut ::core::ffi::c_void) -> ()>;

pub type overlay_free_cb =
    Option<unsafe extern "C" fn(*mut client, *mut ::core::ffi::c_void) -> ()>;

pub type overlay_key_cb = Option<
    unsafe extern "C" fn(
        *mut client,
        *mut ::core::ffi::c_void,
        *mut key_event,
    ) -> ::core::ffi::c_int,
>;

pub type overlay_draw_cb =
    Option<unsafe extern "C" fn(*mut client, *mut ::core::ffi::c_void) -> ()>;

pub type overlay_mode_cb = Option<
    unsafe extern "C" fn(
        *mut client,
        *mut ::core::ffi::c_void,
        *mut u_int,
        *mut u_int,
    ) -> *mut screen,
>;

pub type overlay_check_cb = Option<
    unsafe extern "C" fn(
        *mut client,
        *mut ::core::ffi::c_void,
        u_int,
        u_int,
        u_int,
    ) -> *mut visible_ranges,
>;
