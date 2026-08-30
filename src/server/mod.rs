//! The server: the loop it runs, the clients attached to it, the messages it
//! sends them, and who is allowed to connect.
//!
//! Everything below this module is private. What the rest of the crate may
//! use is exactly what is re-exported here.

mod acl;
mod client;
mod message;
mod run;

pub use acl::{
    server_acl_display, server_acl_get_uid, server_acl_user, server_acl_user_allow,
    server_acl_user_allow_write, server_acl_user_deny, server_acl_user_deny_write,
    server_acl_user_find,
};
pub use client::{
    client_get_last_session, client_get_pan_window, client_set_last_session, client_set_pan_window,
    server_client_add_client_window, server_client_check_nested, server_client_clear_overlay,
    server_client_detach, server_client_ensure_ranges, server_client_exec,
    server_client_get_client_window, server_client_get_cwd, server_client_get_flags,
    server_client_get_key_table, server_client_get_pane, server_client_handle_key,
    server_client_how_many, server_client_lost, server_client_open, server_client_overlay_range,
    server_client_print, server_client_ranges_is_empty, server_client_remove_pane,
    server_client_set_flags, server_client_set_key_table, server_client_set_overlay,
    server_client_set_pane, server_client_set_session, server_client_suspend,
};
pub use message::{
    server_destroy_pane, server_destroy_session, server_kill_pane, server_kill_window,
    server_link_window, server_lock, server_lock_client, server_lock_session, server_redraw_client,
    server_redraw_session, server_redraw_session_group, server_redraw_window,
    server_redraw_window_borders, server_renumber_all, server_status_client, server_status_session,
    server_status_session_group, server_status_window, server_unlink_window, server_unzoom_window,
};
pub use run::{
    current_time, first_client, marked_pane, message_log, server_add_message, server_check_marked,
    server_clear_marked, server_create_socket, server_is_marked, server_proc, server_set_marked,
    server_start,
};

#[cfg(test)]
pub(crate) use acl::{SERVER_ACL_READONLY, server_acl_init, server_acl_join};
#[cfg(test)]
pub(crate) use client::{
    CLIENT_ATTACHED, CLIENT_CONTROL, CLIENT_DEAD, CLIENT_EXIT, CLIENT_EXIT_DETACH,
    CLIENT_EXIT_RETURN, CLIENT_IGNORESIZE, CLIENT_READONLY, CLIENT_REDRAWSTATUS, CLIENT_SUSPENDED,
    CLIENT_UTF8, TTY_FREEZE, TTY_NOCURSOR,
};
pub(crate) use client::{client_ref_from_ptr, client_weak_from_ptr, register_client_handle};
#[cfg(test)]
pub(crate) use message::{CLIENT_ALLREDRAWFLAGS, CLIENT_REDRAWWINDOW};
pub(crate) use run::client_walk;
#[cfg(test)]
pub(crate) use run::{
    MSG_COMMAND, MSG_FLAGS, MSG_IDENTIFY_CWD, MSG_IDENTIFY_DONE, MSG_IDENTIFY_FLAGS,
    MSG_IDENTIFY_TERM, MSG_READ, MSG_READ_DONE, MSG_READ_OPEN, MSG_VERSION, MSG_WRITE,
    MSG_WRITE_CLOSE, MSG_WRITE_OPEN, PANE_LINES_DOUBLE, PANE_LINES_SINGLE, clients,
};
