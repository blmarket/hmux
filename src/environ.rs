use crate::ffi::{environ, fnmatch, getenv, getpid, setenv};
use crate::fmt_args;
use crate::fmt_engine::{FmtArg, format_alloc};
use crate::log::log_debug;
use crate::options::{OptionsEngine, RustOptionsEngine};

use crate::tmux::getversion;
use crate::tmux::{global_options, socket_path};
pub use crate::types::*;
use ::core::ffi::{CStr, c_char, c_int};
use ::std::ffi::CString;

/// A borrowed observation of one environment entry.
#[derive(Clone, Copy)]
pub struct EnvironmentEntryRef<'a> {
    pub name: &'a CStr,
    pub value: Option<&'a CStr>,
    pub flags: c_int,
}

/// A tmux environment store.
pub trait EnvironmentStore: Sized {
    /// Creates an independent empty store whose later mutations are private.
    fn empty() -> Self;
    /// Returns entries in bytewise ascending name order, as `strcmp` does.
    fn entries(&self) -> impl Iterator<Item = EnvironmentEntryRef<'_>>;
    /// Finds the entry filed under `name`.
    fn find(&self, name: &CStr) -> Option<EnvironmentEntryRef<'_>>;
    /// Sets an exact value and replaces both value and flags on an existing entry.
    fn set(&mut self, name: &CStr, flags: c_int, value: &CStr);
    /// Leaves a named entry with no value, preserving existing flags or using zero for a new entry.
    fn clear(&mut self, name: &CStr);
    /// Parses the first `NAME=VALUE` split, retaining later `=` bytes, or does nothing without `=`.
    fn put(&mut self, assignment: &CStr, flags: c_int);
    /// Removes the named entry if it exists.
    fn unset(&mut self, name: &CStr);
    /// Overlays source entries without removing unrelated destination entries; valueless entries clear the destination.
    fn copy_from(&mut self, source: &Self);
}

struct RustEnvironmentEntry {
    name: CString,
    value: Option<CString>,
    flags: c_int,
}

/// The Rust-owned implementation of the tmux environment store.
pub struct RustEnvironment {
    entries: std::collections::BTreeMap<CString, RustEnvironmentEntry>,
}

std::thread_local! {
    static GLOBAL_ENVIRONMENT: std::cell::RefCell<RustEnvironment> =
        std::cell::RefCell::new(RustEnvironment::empty());
}

pub(crate) fn with_global_environment<R>(read: impl FnOnce(&RustEnvironment) -> R) -> R {
    GLOBAL_ENVIRONMENT.with(|env| read(&env.borrow()))
}

pub(crate) fn with_global_environment_mut<R>(write: impl FnOnce(&mut RustEnvironment) -> R) -> R {
    GLOBAL_ENVIRONMENT.with(|env| write(&mut env.borrow_mut()))
}

pub(crate) fn reset_global_environment() {
    with_global_environment_mut(|env| *env = RustEnvironment::empty());
}

pub use crate::consts::{ENVIRON_HIDDEN, RB_BLACK, RB_NEGINF, RB_RED};

impl EnvironmentStore for RustEnvironment {
    fn empty() -> Self {
        Self {
            entries: std::collections::BTreeMap::new(),
        }
    }

    fn entries(&self) -> impl Iterator<Item = EnvironmentEntryRef<'_>> {
        self.entries.values().map(|entry| EnvironmentEntryRef {
            name: entry.name.as_c_str(),
            value: entry.value.as_deref(),
            flags: entry.flags,
        })
    }

    fn find(&self, name: &CStr) -> Option<EnvironmentEntryRef<'_>> {
        self.entries.get(name).map(|entry| EnvironmentEntryRef {
            name: entry.name.as_c_str(),
            value: entry.value.as_deref(),
            flags: entry.flags,
        })
    }

    fn set(&mut self, name: &CStr, flags: c_int, value: &CStr) {
        let entry = self
            .entries
            .entry(name.to_owned())
            .or_insert_with_key(|name| RustEnvironmentEntry {
                name: name.clone(),
                value: None,
                flags,
            });
        entry.flags = flags;
        entry.value = Some(value.to_owned());
    }

    fn clear(&mut self, name: &CStr) {
        let entry = self
            .entries
            .entry(name.to_owned())
            .or_insert_with_key(|name| RustEnvironmentEntry {
                name: name.clone(),
                value: None,
                flags: 0,
            });
        entry.value = None;
    }

    fn put(&mut self, assignment: &CStr, flags: c_int) {
        let bytes = assignment.to_bytes();
        let Some(split) = bytes.iter().position(|&byte| byte == b'=') else {
            return;
        };
        let name = CString::new(&bytes[..split]).expect("a C string holds no NUL");
        let value = CString::new(&bytes[split + 1..]).expect("a C string holds no NUL");
        self.set(&name, flags, &value);
    }

    fn unset(&mut self, name: &CStr) {
        self.entries.remove(name);
    }

    fn copy_from(&mut self, source: &Self) {
        for entry in source.entries() {
            match entry.value {
                Some(value) => self.set(entry.name, entry.flags, value),
                None => self.clear(entry.name),
            }
        }
    }
}

pub(crate) fn new_environment_box() -> Box<RustEnvironment> {
    Box::new(RustEnvironment::empty())
}

/// Copies one process environment value, preserving empty values and raw bytes.
///
/// # Safety
/// The process environment must not change while the value is copied.
pub(crate) unsafe fn process_environment_value(name: &CStr) -> Option<CString> {
    unsafe {
        let value = getenv(name.as_ptr());
        (!value.is_null()).then(|| CStr::from_ptr(value).to_owned())
    }
}

/// Copies the C library's process environment before returning its iterator.
///
/// # Safety
/// The process environment must not change while the snapshot is copied.
pub(crate) unsafe fn process_environment() -> impl Iterator<Item = CString> {
    let mut vars = Vec::new();
    unsafe {
        let mut next = environ;
        while !next.is_null() {
            let var = *next;
            if var.is_null() {
                break;
            }
            vars.push(CStr::from_ptr(var).to_owned());
            next = next.add(1);
        }
    }
    vars.into_iter()
}

pub(crate) unsafe fn update_environment(
    oo: &RustOptionsRef,
    src: &RustEnvironment,
    dst: &mut RustEnvironment,
) {
    unsafe {
        oo.with_entry(c"update-environment", false, |entry| {
            let Some(entry) = entry else { return };
            for array_index in RustOptionsEngine.array_indices(entry) {
                let ov = RustOptionsEngine.array_get(entry, array_index).unwrap();
                let mut found = false;
                for envent in src.entries() {
                    if fnmatch(
                        RustOptionsEngine.value_string(ov).as_ptr(),
                        envent.name.as_ptr(),
                        0 as c_int,
                    ) == 0 as c_int
                    {
                        match envent.value {
                            Some(value) => dst.set(envent.name, 0, value),
                            None => dst.clear(envent.name),
                        }
                        found = true;
                    }
                }
                if !found {
                    dst.clear(RustOptionsEngine.value_string(ov));
                }
            }
        });
    }
}

pub(crate) unsafe fn push_environment_to_process(env: &RustEnvironment) {
    // The empty environment the first `setenv` grows from. The C library
    // allocates its own array the moment it has an entry to store, so this one
    // is never handed back and never has to be freed.
    static mut EMPTY_ENVIRON: [*mut c_char; 1] = [core::ptr::null_mut()];
    unsafe {
        environ = (&raw mut EMPTY_ENVIRON).cast::<*mut c_char>();
        for envent in env.entries() {
            if let Some(value) = envent.value
                && !envent.name.to_bytes().is_empty()
                && envent.flags & ENVIRON_HIDDEN == 0
            {
                setenv(envent.name.as_ptr(), value.as_ptr(), 1 as c_int);
            }
        }
    }
}

pub(crate) unsafe fn log_environment(env: &RustEnvironment, fmt: &CStr, args: &[FmtArg]) {
    {
        let prefix = format_alloc(fmt, args);
        for envent in env.entries() {
            if let Some(value) = envent.value
                && !envent.name.to_bytes().is_empty()
            {
                log_debug(c"%s%s=%s", fmt_args![prefix.as_c_str(), envent.name, value]);
            }
        }
    }
}

pub(crate) unsafe fn environment_for_session(
    s: Option<&session>,
    no_TERM: c_int,
) -> Box<RustEnvironment> {
    unsafe {
        let mut env = new_environment_box();
        with_global_environment(|global| env.copy_from(global));
        let idx = s.map_or(-(1 as c_int), |s| {
            crate::SessionIdentity::session_id(s) as c_int
        });
        if let Some(s) = s {
            env.copy_from(s.environ_ref());
        }
        if no_TERM == 0 {
            let value = (global_options
                .as_ref()
                .expect("global options are initialized"))
            .string_ref(c"default-terminal");
            env.set(c"TERM", 0, &value);
            env.set(c"TERM_PROGRAM", 0, c"tmux");
            env.set(c"TERM_PROGRAM_VERSION", 0, getversion());
            env.set(c"COLORTERM", 0, c"truecolor");
        }
        env.clear(c"LISTEN_PID");
        env.clear(c"LISTEN_FDS");
        env.clear(c"LISTEN_FDNAMES");
        let tmux = format_alloc(
            c"%s,%ld,%d",
            fmt_args![socket_path.as_deref(), getpid() as core::ffi::c_long, idx],
        );
        env.set(c"TMUX", 0, &tmux);
        env
    }
}

#[cfg(test)]
#[path = "tests/test_environ.rs"]
mod tests;
