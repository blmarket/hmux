//! Shared tmux command, state, and terminal engine.
//!
//! These modules hold the runtime-neutral parts of the tmux server — the
//! command table and handlers, session/window/pane state, option and format
//! machinery, and terminal capabilities — so both the blocking [`crate::native`]
//! runtime and the readiness-driven [`crate::event_loop`] runtime can share
//! them.

#[path = "cmd-send-keys.rs"]
pub(crate) mod cmd_send_keys;
pub mod command;
pub mod format;
pub(crate) mod key;
pub mod latmon;
pub(crate) mod mouse;
pub mod options;
pub mod registry;
pub mod state;
pub mod status;
pub(crate) mod style;
pub(crate) mod term;
