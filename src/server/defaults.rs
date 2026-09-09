//! The option defaults this server ships in place of tmux's.
//!
//! A default is presentation, and presentation is the server's: the plugins
//! publish variables, and what the status line does with them is decided here,
//! in one place, rather than in whichever plugin happens to name them. That
//! also keeps the format from depending on registration order — an option
//! default only replaces a value still holding tmux's, so two plugins
//! declaring one format would leave the first to register deciding it.
//!
//! Only options still holding their built-in default are replaced, and this
//! runs before any configuration file is read, so `.tmux.conf` still wins.

use crate::options::{OptionsEngine, OptionsRef, RustOptionsEngine};
use std::ffi::{CStr, CString};

use crate::fmt_args;

use crate::tmux::{global_options, global_s_options, global_w_options};

/// The window status format this server draws.
///
/// The pane's state glyph, the model on the glyph's background, and then where
/// the pane is: the worktree it sits in — with a trailing `/` when it sits
/// below the root — falling back to the directory's own name outside a
/// repository, and the operation the repository is in the middle of.
///
/// Only a matching model branch emits a directive — `bg=default` would punch a
/// terminal-background hole in the status bar rather than leaving it whatever
/// `status-style` painted.
const WINDOW_STATUS_FORMAT: &str = "#I:#{?#{m:*fable*,#{pane_agent_model}},#[bg=red],#{?#{m:*luna*,#{pane_agent_model}},#[bg=brightblue],}}#{pane_state_emoji}#[default] #{?git_worktree,#{git_worktree}#{?git_subdir,/,},#{?pane_current_path,#{b:pane_current_path},#{b:session_path}}}#{?git_action, [#{git_action}#{?git_action_total, #{git_action_step}/#{git_action_total},}],}#{?window_flags,#{window_flags}, }";

/// What this server puts in place of tmux's own defaults, as `(name, value)`.
const SERVER_DEFAULTS: &[(&str, &str)] = &[
    ("window-status-format", WINDOW_STATUS_FORMAT),
    ("window-status-current-format", WINDOW_STATUS_FORMAT),
];

/// Put this server's own option defaults in place.
///
/// Nothing is replaced when no plugin is running. Every variable the format
/// draws on comes from one, and a server with none of them is meant to be
/// tmux — which is what `TMUX_C2RS_PLUGINS=none` asks for and what the
/// conformance suite compares.
pub(crate) fn server_default_options() {
    if !crate::plugin::any_registered() {
        return;
    }
    server_apply_option_defaults(SERVER_DEFAULTS);
}

/// Set each named option, unless the server already holds a value for it other
/// than the built-in default. This is what a plugin's own defaults go in
/// through as well.
pub(crate) fn server_apply_option_defaults(defaults: &[(&str, &str)]) {
    for (name, value) in defaults {
        let (Ok(name), Ok(value)) = (CString::new(*name), CString::new(*value)) else {
            continue;
        };
        unsafe { set_if_default(&name, &value) };
    }
}

/// Set one option, unless the server already holds a value for it other than
/// the built-in default. Every global option carries an explicit value from
/// startup, so "untouched" is a comparison against the table default rather
/// than an absent entry.
unsafe fn set_if_default(name: &CStr, value: &CStr) {
    unsafe {
        for oo in [&global_options, &global_s_options, &global_w_options] {
            if oo.is_none() {
                continue;
            }
            let is_default =
                oo.as_ref()
                    .expect("global options exist")
                    .with_entry(name, true, |entry| {
                        let Some(entry) = entry else { return false };
                        let Some(definition) = RustOptionsEngine.definition(Some(entry)) else {
                            return false;
                        };
                        RustOptionsEngine.display(entry, -1, 0)
                            == RustOptionsEngine.default_text(definition)
                    });
            if !is_default {
                continue;
            }
            (oo.as_ref().expect("global options exist")).set_string(
                name,
                0,
                c"%s",
                fmt_args![value.as_ptr()],
            );
        }
    }
}
