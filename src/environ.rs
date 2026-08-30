use crate::ffi::{environ, fnmatch, free, getpid, setenv};
use crate::fmt_args;
use crate::fmt_engine::{FmtArg, format_alloc};
use crate::log::log_debug;
use crate::options::options_get_ptr;
use crate::options::{
    options_array_first, options_array_item_value, options_array_next, options_get_string,
};
use crate::session::{session_environ, session_id};
use crate::tmux::getversion;
use crate::tmux::{global_environ, global_options, socket_path};
pub use crate::types::*;
use crate::xmalloc::xcalloc;
use ::core::ffi::{CStr, c_char, c_int, c_void};
use ::std::ffi::CString;
/// One environment variable: the name it is filed under, the value it carries
/// if it has been given one, and whether it is hidden.
///
/// The fields are the entry's own. Outside this module an entry is read with
/// `environ_entry_name`, `environ_entry_value` and `environ_entry_flags`, and
/// changed only through `environ_set`, `environ_clear` and the rest, which is
/// what keeps an entry's name and the key it is filed under the same string.
#[repr(C)]
pub struct environ_entry {
    name: CString,
    value: Option<CString>,
    flags: c_int,
}

/// The entries of one environment, by name. Names order byte by byte, as
/// `strcmp` ordered them. An entry lives in the map, so a pointer to one
/// lasts only until the same environment is added to again.
pub type environ_t = ::std::collections::BTreeMap<CString, environ_entry>;
pub const RB_BLACK: c_int = 0 as c_int;
pub const RB_RED: c_int = 1 as c_int;
pub const RB_NEGINF: c_int = -(1 as c_int);
pub const ENVIRON_HIDDEN: c_int = 0x1 as c_int;

/// The name an entry is filed under.
unsafe fn name_of(envent: *mut environ_entry) -> &'static CStr {
    unsafe { (*envent).name.as_c_str() }
}

/// The name the entry is filed under.
pub unsafe fn environ_entry_name(envent: *const environ_entry) -> *const c_char {
    unsafe { (*envent).name.as_ptr() }
}

/// The value the entry carries, or null when it has been set to none, which
/// is how an entry says the variable is to be taken out of a child's
/// environment rather than given to it.
pub unsafe fn environ_entry_value(envent: *const environ_entry) -> *const c_char {
    unsafe { cstr_ptr(&(*envent).value) }
}

/// The entry's flags, of which `ENVIRON_HIDDEN` is the only one.
pub unsafe fn environ_entry_flags(envent: *const environ_entry) -> c_int {
    unsafe { (*envent).flags }
}

/// Every entry of `env` in name order. The pointers are read in one go, so
/// nothing may be put into the same environment while they are in use.
fn entries(env: *mut environ_t) -> impl Iterator<Item = *mut environ_entry> {
    let all: Vec<*mut environ_entry> =
        unsafe { (*env).values_mut().map(|envent| &raw mut *envent).collect() };
    all.into_iter()
}

/// A new entry for `name` with no value yet, put into the tree.
unsafe fn environ_add(
    env: *mut environ_t,
    name: *const c_char,
    flags: c_int,
) -> *mut environ_entry {
    unsafe {
        let name = CStr::from_ptr(name).to_owned();
        let envent = (*env).entry(name.clone()).or_insert(environ_entry {
            name,
            value: None,
            flags,
        });
        &raw mut *envent
    }
}

/// The environment a struct carries, or null if it carries none.
pub fn environ_ptr(env: &Option<Box<environ_t>>) -> *mut environ_t {
    match env {
        Some(env) => &raw const **env as *mut environ_t,
        None => ::core::ptr::null_mut::<environ_t>(),
    }
}

pub fn environ_create_box() -> Box<environ_t> {
    Box::new(environ_t::new())
}

pub unsafe fn environ_free(env: *mut environ_t) {
    unsafe {
        if env.is_null() {
            return;
        }
        drop(Box::from_raw(env));
    }
}

/// The entries a set holds, in name order. The walk borrows the set, since
/// adding to it while walking would move the entries about.
pub fn environ_entries(env: &environ_t) -> impl Iterator<Item = &environ_entry> {
    env.values()
}

pub unsafe fn environ_copy(srcenv: *mut environ_t, dstenv: *mut environ_t) {
    unsafe {
        for envent in entries(srcenv) {
            match &(*envent).value {
                None => environ_clear(dstenv, (*envent).name.as_ptr()),
                Some(value) => environ_set(
                    dstenv,
                    (*envent).name.as_ptr(),
                    (*envent).flags,
                    c"%s".as_ptr(),
                    fmt_args![value.as_ptr()],
                ),
            }
        }
    }
}

/// The entry a set holds for `name`, if it holds one.
pub unsafe fn environ_find(env: &environ_t, name: *const c_char) -> Option<&environ_entry> {
    unsafe { env.get(CStr::from_ptr(name)) }
}

/// The same entry, to write the value the set is being given.
pub unsafe fn environ_find_mut(
    env: &mut environ_t,
    name: *const c_char,
) -> Option<&mut environ_entry> {
    unsafe { env.get_mut(CStr::from_ptr(name)) }
}

pub unsafe fn environ_set(
    env: *mut environ_t,
    name: *const c_char,
    flags: c_int,
    fmt: *const c_char,
    args: &[FmtArg],
) {
    unsafe {
        let mut envent = match environ_find_mut(&mut *env, name) {
            Some(envent) => {
                envent.flags = flags;
                &raw mut *envent
            }
            None => environ_add(env, name, flags),
        };
        (*envent).value = Some(format_alloc(fmt, args));
    }
}

pub unsafe fn environ_clear(env: *mut environ_t, name: *const c_char) {
    unsafe {
        match environ_find_mut(&mut *env, name) {
            Some(envent) => envent.value = None,
            None => {
                environ_add(env, name, 0 as c_int);
            }
        }
    }
}

pub unsafe fn environ_put(env: *mut environ_t, var: *const c_char, flags: c_int) {
    unsafe {
        let var = CStr::from_ptr(var).to_bytes();
        let Some(split) = var.iter().position(|&b| b == b'=') else {
            return;
        };
        let name = CString::new(&var[..split]).expect("a C string holds no NUL");
        let value = CString::new(&var[split + 1..]).expect("a C string holds no NUL");
        environ_set(
            env,
            name.as_ptr(),
            flags,
            c"%s".as_ptr(),
            fmt_args![value.as_ptr()],
        );
    }
}

pub unsafe fn environ_unset(env: *mut environ_t, name: *const c_char) {
    unsafe {
        (*env).remove(CStr::from_ptr(name));
    }
}

pub unsafe fn environ_update(oo: *mut options, src: *mut environ_t, dst: *mut environ_t) {
    unsafe {
        let o = options_get_ptr(oo, c"update-environment".as_ptr());
        if o.is_null() {
            return;
        }
        let mut a = options_array_first(o);
        while !a.is_null() {
            let ov = options_array_item_value(a);
            let mut found = false;
            for envent in entries(src) {
                if fnmatch((*ov).string(), (*envent).name.as_ptr(), 0 as c_int) == 0 as c_int {
                    environ_set(
                        dst,
                        (*envent).name.as_ptr(),
                        0 as c_int,
                        c"%s".as_ptr(),
                        fmt_args![cstr_ptr(&(*envent).value)],
                    );
                    found = true;
                }
            }
            if !found {
                environ_clear(dst, (*ov).string());
            }
            a = options_array_next(o, a);
        }
    }
}

pub unsafe fn environ_push(env: *mut environ_t) {
    unsafe {
        let new_environ = xcalloc(1 as size_t, ::core::mem::size_of::<*mut c_char>() as size_t)
            as *mut *mut c_char;
        environ = new_environ;
        for envent in entries(env) {
            if let Some(value) = &(*envent).value
                && !name_of(envent).is_empty()
                && (*envent).flags & ENVIRON_HIDDEN == 0
            {
                setenv((*envent).name.as_ptr(), value.as_ptr(), 1 as c_int);
            }
        }
        if environ != new_environ {
            free(new_environ as *mut c_void);
        }
    }
}

pub unsafe fn environ_log(env: *mut environ_t, fmt: *const c_char, args: &[FmtArg]) {
    unsafe {
        let prefix = format_alloc(fmt, args);
        for envent in entries(env) {
            if let Some(value) = &(*envent).value
                && !name_of(envent).is_empty()
            {
                log_debug(
                    c"%s%s=%s".as_ptr(),
                    fmt_args![prefix.as_ptr(), (*envent).name.as_ptr(), value.as_ptr()],
                );
            }
        }
    }
}

pub unsafe fn environ_for_session(s: *mut session, no_TERM: c_int) -> Box<environ_t> {
    unsafe {
        let mut env = environ_create_box();
        environ_copy(global_environ, &mut *env);
        if !s.is_null() {
            environ_copy(session_environ(s), &mut *env);
        }
        if no_TERM == 0 {
            let value = options_get_string(global_options, c"default-terminal".as_ptr());
            environ_set(
                &mut *env,
                c"TERM".as_ptr(),
                0 as c_int,
                c"%s".as_ptr(),
                fmt_args![value],
            );
            environ_set(
                &mut *env,
                c"TERM_PROGRAM".as_ptr(),
                0 as c_int,
                c"%s".as_ptr(),
                fmt_args![c"tmux".as_ptr()],
            );
            environ_set(
                &mut *env,
                c"TERM_PROGRAM_VERSION".as_ptr(),
                0 as c_int,
                c"%s".as_ptr(),
                fmt_args![getversion()],
            );
            environ_set(
                &mut *env,
                c"COLORTERM".as_ptr(),
                0 as c_int,
                c"%s".as_ptr(),
                fmt_args![c"truecolor".as_ptr()],
            );
        }
        environ_clear(&mut *env, c"LISTEN_PID".as_ptr());
        environ_clear(&mut *env, c"LISTEN_FDS".as_ptr());
        environ_clear(&mut *env, c"LISTEN_FDNAMES".as_ptr());
        let idx = if s.is_null() {
            -(1 as c_int)
        } else {
            session_id(s) as c_int
        };
        environ_set(
            &mut *env,
            c"TMUX".as_ptr(),
            0 as c_int,
            c"%s,%ld,%d".as_ptr(),
            fmt_args![socket_path, getpid() as ::core::ffi::c_long, idx],
        );
        env
    }
}

#[cfg(test)]
#[path = "tests/test_environ.rs"]
mod tests;
