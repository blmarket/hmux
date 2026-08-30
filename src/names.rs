use crate::cmd::cmd_stringify_argv;
use crate::ffi::{__ctype_b_loc, __xpg_basename, gettimeofday, strcmp};
use crate::fmt_args;
use crate::format::{format_create, format_defaults_pane, format_defaults_window, format_expand};
use crate::log::log_debug;
use crate::options::{options_get_number, options_get_string, options_ptr};
use crate::reactor::Timer;
use crate::server::{server_redraw_window_borders, server_status_window};
use crate::tmux::clean_name;
pub use crate::types::*;
use crate::window::window_get_active;
use crate::window::{window_ref_from_ptr, window_set_name};
use ::std::ffi::CString;
pub type ctype_mask = ::core::ffi::c_uint;
pub const _ISalnum: ctype_mask = 8;
pub const _ISpunct: ctype_mask = 4;
pub const EV_TIMEOUT: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const NAME_INTERVAL: ::core::ffi::c_int = 500000 as ::core::ffi::c_int;
pub const PANE_CHANGED: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const FORMAT_WINDOW: ::core::ffi::c_uint = 0x40000000 as ::core::ffi::c_uint;

unsafe fn name_time_callback(w: *mut window) {
    unsafe {
        let Some(w_ref) = window_ref_from_ptr(w) else {
            return;
        };
        let w = w_ref.as_ptr();
        log_debug(c"@%u name timer expired".as_ptr(), fmt_args![(*w).id]);
    }
}

/// Microseconds still owed to the automatic-rename interval that started at
/// `since`, or zero once the interval has run out.
fn interval_left(since: timeval, now: timeval) -> ::core::ffi::c_int {
    let mut sec = now.tv_sec - since.tv_sec;
    let mut usec = now.tv_usec - since.tv_usec;
    if usec < 0 as __suseconds_t {
        sec -= 1;
        usec += 1000000 as __suseconds_t;
    }
    if sec != 0 as __time_t || usec > NAME_INTERVAL as __suseconds_t {
        return 0;
    }
    (NAME_INTERVAL as __suseconds_t - usec) as ::core::ffi::c_int
}

unsafe fn name_time_expired(w: *mut window, tv: *mut timeval) -> ::core::ffi::c_int {
    unsafe { interval_left((*w).name_time, *tv) }
}

/// Drives the automatic-rename timer and the rename itself. Its whole body is
/// live-server work — ensure_reactor timers, the format engine and the server redraw
/// paths — so it stays a raw-pointer function; the conformance suite is what
/// exercises it.
pub unsafe fn check_window_name(w: *mut window) {
    unsafe {
        let mut tv = timeval::default();
        if window_get_active(w).is_null() {
            return;
        }
        if options_get_number(options_ptr(&(*w).options), c"automatic-rename".as_ptr()) == 0 {
            return;
        }
        if !(*window_get_active(w)).flags & PANE_CHANGED != 0 {
            log_debug(c"@%u active pane not changed".as_ptr(), fmt_args![(*w).id]);
            return;
        }
        log_debug(c"@%u active pane changed".as_ptr(), fmt_args![(*w).id]);
        gettimeofday(&raw mut tv, ::core::ptr::null_mut());
        let left = name_time_expired(w, &raw mut tv);
        if left != 0 {
            if !(*w).name_event.is_set() {
                let w_weak = window_ref_from_ptr(w).map(|w_ref| w_ref.downgrade());
                (*w).name_event.set_callback(move || {
                    let Some(w_ref) = w_weak.as_ref().and_then(WindowWeak::upgrade) else {
                        return;
                    };
                    name_time_callback(w_ref.as_ptr());
                });
            }
            if !(*w).name_event.is_armed() {
                log_debug(
                    c"@%u name timer queued (%d left)".as_ptr(),
                    fmt_args![(*w).id, left],
                );
                let mut next = timeval::from_usecs(left as __suseconds_t);
                (*w).name_event.arm(next);
            } else {
                log_debug(
                    c"@%u name timer already queued (%d left)".as_ptr(),
                    fmt_args![(*w).id, left],
                );
            }
            return;
        }
        (*w).name_time = tv;
        (*w).name_event.disarm();
        (*window_get_active(w)).flags &= !PANE_CHANGED;
        let name = format_window_name(w);
        if strcmp(name.as_ptr(), cstr_ptr(&(*w).name)) != 0 {
            log_debug(
                c"@%u new name %s (was %s)".as_ptr(),
                fmt_args![(*w).id, name.as_ptr(), cstr_ptr(&(*w).name)],
            );
            window_set_name(w, name.as_ptr(), 1);
            server_redraw_window_borders(w);
            server_status_window(w);
        } else {
            log_debug(
                c"@%u name not changed (still %s)".as_ptr(),
                fmt_args![(*w).id, cstr_ptr(&(*w).name)],
            );
        }
    }
}

pub unsafe fn default_window_name(w: *mut window) -> CString {
    unsafe {
        if window_get_active(w).is_null() {
            return CString::default();
        }
        let cmd = cmd_stringify_argv(&(*window_get_active(w)).argv);
        if !cmd.as_bytes().is_empty() {
            parse_window_name(cmd.as_ptr())
        } else {
            parse_window_name(cstr_ptr(&(*window_get_active(w)).shell))
        }
    }
}

/// Renders `automatic-rename-format` for the window. The format engine reads
/// the live server state through the tree it builds.
unsafe fn format_window_name(w: *mut window) -> CString {
    unsafe {
        let mut ft = format_create(
            ::core::ptr::null_mut::<client>(),
            ::core::ptr::null_mut::<cmdq_item>(),
            (FORMAT_WINDOW | (*w).id) as ::core::ffi::c_int,
            0,
        );
        format_defaults_window(&mut ft, w);
        format_defaults_pane(&mut ft, window_get_active(w));
        let fmt = options_get_string(
            options_ptr(&(*w).options),
            c"automatic-rename-format".as_ptr(),
        );
        format_expand(&mut ft, ::core::ffi::CStr::from_ptr(fmt))
    }
}

/// Whether `b` is `isalnum` or `ispunct` under the process's current locale,
/// which is where `parse_window_name` stops trimming a name.
fn is_name_byte(b: u8) -> bool {
    let class = unsafe { *(*__ctype_b_loc()).add(b as usize) } as ctype_mask;
    class & (_ISalnum | _ISpunct) != 0
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

pub unsafe fn parse_window_name(in_0: *const ::core::ffi::c_char) -> CString {
    unsafe {
        let trimmed = trim_name(::core::ffi::CStr::from_ptr(in_0).to_bytes());
        let mut copy: Vec<::core::ffi::c_char> = trimmed
            .iter()
            .map(|&b| b as ::core::ffi::c_char)
            .chain(::core::iter::once(0))
            .collect();
        let mut name = copy.as_mut_ptr();
        if *name == b'/' as ::core::ffi::c_char {
            name = __xpg_basename(name);
        }
        clean_name(name, 0).unwrap_or_default()
    }
}

#[cfg(test)]
#[path = "tests/test_names.rs"]
mod tests;
