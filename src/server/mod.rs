//! The server: the loop it runs, the clients attached to it, the messages it
//! sends them, and who is allowed to connect.
//!
//! Everything below this module is private. What the rest of the crate may
//! use is exactly what is re-exported here.

mod acl;
mod client;
mod defaults;
mod message;
mod run;

pub use acl::{RustServerAclStore, ServerAclAccess, ServerAclEntry, ServerAclStore};
pub(crate) use client::ClientPanDirection;
pub(crate) use client::server_client_get_pane_in_window;
pub use client::{
    client_get_last_session, client_get_pan_window, client_set_last_session, client_set_pan_window,
    server_client_add_client_window, server_client_check_nested, server_client_clear_overlay,
    server_client_detach, server_client_exec, server_client_get_client_window,
    server_client_get_cwd, server_client_get_flags, server_client_get_key_table,
    server_client_get_pane, server_client_handle_key, server_client_how_many, server_client_open,
    server_client_print, server_client_remove_pane, server_client_set_flags,
    server_client_set_key_table, server_client_set_overlay, server_client_set_pane,
    server_client_set_session, server_client_suspend,
};
pub use message::{
    server_destroy_pane, server_destroy_session, server_kill_pane, server_lock, server_lock_client,
    server_lock_session, server_redraw_client, server_redraw_session, server_redraw_session_group,
    server_renumber_all, server_status_client, server_status_session, server_status_session_group,
    server_unlink_window,
};
pub use run::{
    current_time, first_client, marked_pane, server_check_marked, server_clear_marked,
    server_create_socket, server_is_marked, server_proc, server_set_marked, server_start,
};

#[cfg(test)]
pub(crate) use acl::server_acl_init;
pub(crate) use acl::{
    server_acl_display, server_acl_update_clients, with_server_acl, with_server_acl_mut,
};
#[cfg(test)]
pub(crate) use client::{CLIENT_ATTACHED, CLIENT_CONTROL, CLIENT_SUSPENDED};
pub(crate) use client::{client_ref_of, server_client_update_focus};
pub(crate) use defaults::server_apply_option_defaults;
pub(crate) use run::server_add_message;
#[cfg(test)]
pub(crate) use run::with_clients_mut;
#[cfg(test)]
pub(crate) use run::{
    MSG_COMMAND, MSG_FLAGS, MSG_IDENTIFY_CWD, MSG_IDENTIFY_DONE, MSG_IDENTIFY_FLAGS,
    MSG_IDENTIFY_TERM, MSG_READ, MSG_READ_DONE, MSG_READ_OPEN, MSG_VERSION, MSG_WRITE,
    MSG_WRITE_CLOSE, MSG_WRITE_OPEN, PANE_LINES_DOUBLE, PANE_LINES_SINGLE,
};
pub(crate) use run::{client_walk, with_clients};

pub(crate) use message::server_link_window;

pub(crate) use run::server_toggle_marked_pane;

pub(crate) use client::{
    client_clear_overlay, client_print_buffer, client_set_overlay, client_working_directory,
};
