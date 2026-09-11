use crate::WindowPane as _;
use crate::fmt_args;
use crate::format::{format_create, format_defaults_pane, format_defaults_window, format_expand};
use crate::log::log_debug;

use crate::pane_command::PaneCommandState;
use crate::reactor::Timer;

use crate::tmux::clean_name;
pub use crate::types::*;

use crate::window_timestamps::WindowTimestampState;
use crate::window_trait::Window as _;
use crate::{CommandTextCodec, RustCommandTextCodec};
use ::std::ffi::CString;

/// Parses the program-shaped name tmux derives from a command string.
pub trait WindowNameParser {
    /// Returns an independently owned parsed name.
    fn parse_window_name(&self, input: &core::ffi::CStr) -> CString;
}

/// The parser used by the Rust tmux implementation.
#[derive(Clone, Copy, Debug, Default)]
pub struct RustWindowNameParser;
pub use crate::consts::{EV_TIMEOUT, FORMAT_WINDOW, PANE_CHANGED};
pub const NAME_INTERVAL: core::ffi::c_int = 500000 as core::ffi::c_int;

/// Microseconds still owed to the automatic-rename interval that started at
/// `since`, or zero once the interval has run out.
fn interval_left(since: timeval, now: timeval) -> core::ffi::c_int {
    let mut sec = now.tv_sec - since.tv_sec;
    let mut usec = now.tv_usec - since.tv_usec;
    if usec < 0 as __suseconds_t {
        sec -= 1;
        usec += 1000000 as __suseconds_t;
    }
    if sec != 0 as __time_t || usec > NAME_INTERVAL as __suseconds_t {
        return 0;
    }
    (NAME_INTERVAL as __suseconds_t - usec) as core::ffi::c_int
}

pub unsafe fn default_window_name(w: &WindowRef) -> CString {
    unsafe {
        let Some(pane) = w.active_pane() else {
            return CString::default();
        };
        let Some(pane) = pane.get() else {
            return CString::default();
        };
        let command = pane.pane_command();
        let cmd = RustCommandTextCodec.stringify(&command.argv);
        if !cmd.as_bytes().is_empty() {
            parse_window_name(&cmd)
        } else {
            parse_window_name(command.shell.as_deref().unwrap_or(c""))
        }
    }
}

/// Whether `b` is `isalnum` or `ispunct` under the process's current locale,
/// which is where `parse_window_name` stops trimming a name.
fn is_name_byte(b: u8) -> bool {
    unsafe { libc::isalnum(b.into()) != 0 || libc::ispunct(b.into()) != 0 }
}

/// The bare window name inside a command string: surrounding quotes removed,
/// an `exec` prefix and any leading spaces or dashes skipped, everything from
/// the first remaining space dropped, and trailing bytes that are neither
/// alphanumeric nor punctuation trimmed. Never trims away the first byte.
fn trim_name(input: &[u8]) -> &[u8] {
    let mut name = input;
    if name.first() == Some(&b'"') {
        name = &name[1..];
    }
    if let Some(end) = name.iter().position(|&b| b == b'"') {
        name = &name[..end];
    }
    if let Some(rest) = name.strip_prefix(b"exec ".as_slice()) {
        name = rest;
    }
    while matches!(name.first(), Some(b' ' | b'-')) {
        name = &name[1..];
    }
    if let Some(end) = name.iter().position(|&b| b == b' ') {
        name = &name[..end];
    }
    while name.len() > 1 && !is_name_byte(name[name.len() - 1]) {
        name = &name[..name.len() - 1];
    }
    name
}

/// The last component of `path`, the way `basename` reads one: trailing
/// slashes are no part of the name, and a path of nothing but slashes names
/// the root itself.
fn basename_of(path: &[u8]) -> &[u8] {
    let Some(last) = path.iter().rposition(|&b| b != b'/') else {
        return b"/";
    };
    let start = path[..last]
        .iter()
        .rposition(|&b| b == b'/')
        .map_or(0, |at| at + 1);
    &path[start..last + 1]
}

fn parse_window_name(in_0: &core::ffi::CStr) -> CString {
    let mut name = trim_name(in_0.to_bytes());
    if name.first() == Some(&b'/') {
        name = basename_of(name);
    }
    let name = unsafe { CString::from_vec_unchecked(name.to_vec()) };
    unsafe { clean_name(&name, 0).unwrap_or_default() }
}

impl WindowNameParser for RustWindowNameParser {
    fn parse_window_name(&self, input: &core::ffi::CStr) -> CString {
        parse_window_name(input)
    }
}

#[cfg(test)]
#[path = "tests/test_names.rs"]
mod tests;

impl WindowRef {
    fn on_name_timer(&self) {
        let w_ref = self;

        log_debug(c"@%u name timer expired", fmt_args![w_ref.window_id()]);
    }
    fn name_time_expired(&self, now: timeval) -> core::ffi::c_int {
        let w = self;

        interval_left(w.timestamps().name_time, now)
    }
    /// Drives automatic renaming while retaining the window across format expansion.
    pub unsafe fn check_name(&self) {
        let window = self;

        unsafe {
            let window = window.clone();
            let Some(mut active) = window.active_pane() else {
                return;
            };
            let options = window.options();
            if options.number(c"automatic-rename") == 0 {
                return;
            }
            let Some(pane) = active.get() else {
                return;
            };
            if *pane.flags() & PANE_CHANGED == 0 {
                log_debug(
                    c"@%u active pane not changed",
                    fmt_args![window.window_id()],
                );
                return;
            }
            log_debug(c"@%u active pane changed", fmt_args![window.window_id()]);
            let tv = timeval::now();
            let left = window.name_time_expired(tv);
            if left != 0 {
                let weak = window.downgrade();
                let mut w = window.as_window_mut();
                if !w.name_event.is_set() {
                    w.name_event.set_callback(move || {
                        if let Some(window) = weak.upgrade() {
                            window.on_name_timer();
                        }
                    });
                }
                if !w.name_event.is_armed() {
                    log_debug(
                        c"@%u name timer queued (%d left)",
                        fmt_args![w.window_id(), left],
                    );
                    w.name_event.arm(timeval::from_usecs(left as __suseconds_t));
                } else {
                    log_debug(
                        c"@%u name timer already queued (%d left)",
                        fmt_args![w.window_id(), left],
                    );
                }
                return;
            }
            {
                let mut w = window.as_window_mut();
                w.set_name_update_time(tv);
                w.name_event.disarm();
            }
            if let Some(pane) = active.get_mut() {
                pane.finish_name_update();
            }
            let name = window.format_name();
            if window.window_name().as_deref() != Some(name.as_c_str()) {
                log_debug(
                    c"@%u new name %s (was %s)",
                    fmt_args![
                        window.window_id(),
                        name.as_c_str(),
                        window.window_name().as_deref()
                    ],
                );
                window.set_name(&name, 1);
                window.redraw_borders();
                window.redraw_status();
            } else {
                log_debug(
                    c"@%u name not changed (still %s)",
                    fmt_args![window.window_id(), window.window_name().as_deref()],
                );
            }
        }
    }
    /// Renders `automatic-rename-format` for the window. The format engine reads
    /// the live server state through the tree it builds.
    unsafe fn format_name(&self) -> CString {
        let window = self;

        unsafe {
            let mut ft = format_create(
                None,
                None,
                (FORMAT_WINDOW | window.window_id()) as core::ffi::c_int,
                0,
            );
            format_defaults_window(&mut ft, window);
            let active = window.active_pane();
            if let Some(pane) = active.as_ref().and_then(|pane| pane.get()) {
                format_defaults_pane(&mut ft, pane);
            }
            let fmt = window.options().string_ref(c"automatic-rename-format");
            format_expand(&mut ft, &fmt)
        }
    }
}
