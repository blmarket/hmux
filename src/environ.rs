use crate::src::ffi::libc::{environ, fnmatch, free, getpid, setenv};
use crate::src::log::{log_cstr, log_debug};
use crate::src::options::{
    options_array_first, options_array_item_value, options_array_next, options_get,
    options_get_string,
};
use crate::src::tmux::{getversion, global_environ, global_options, socket_path};
use crate::src::xmalloc::{xcalloc, xvasprintf_cstring};
use std::ffi::{CStr, CString, NulError};
use std::marker::PhantomData;
use std::ptr::NonNull;

struct ProcessEnvironmentSeed(*mut *mut ::core::ffi::c_char);

impl Drop for ProcessEnvironmentSeed {
    fn drop(&mut self) {
        unsafe {
            // setenv may replace the initially allocated pointer array. If it
            // did, the seed is no longer the active process environment and
            // remains ours to free. Otherwise ownership has moved to environ.
            if environ != self.0 {
                free(self.0 as *mut ::core::ffi::c_void);
            }
        }
    }
}

/// Owns one boxed environment with Rust-owned ordered entries.
///
/// This wrapper owns a tree returned by
/// `environ_create` (or another function with the same allocation contract).
/// It is deliberately not `Copy` or `Clone`; dropping it releases the entry
/// storage before the outer record.
pub struct EnvironOwner {
    raw: NonNull<environ>,
}

impl EnvironOwner {
    /// Allocate an empty environment.
    pub fn new() -> Self {
        // Box allocation aborts on failure, so a successful return is a
        // non-null owned tree just like the legacy C call site expects.
        unsafe { Self::from_raw(environ_create()) }
    }

    /// Reclaim an already allocated tree whose ownership is being transferred.
    ///
    /// # Safety
    ///
    /// `raw` must be a non-null pointer returned by `environ_create` or an
    /// equivalent boxed allocation path, and no other owner may free it.
    pub unsafe fn from_raw(raw: *mut environ) -> Self {
        Self {
            raw: NonNull::new(raw).expect("owned environment pointer must not be null"),
        }
    }

    /// Fallible form of [`Self::from_raw`] for FFI boundaries that permit null.
    /// A null pointer is not adopted or freed.
    ///
    /// # Safety
    ///
    /// A non-null `raw` must be uniquely owned by the caller and allocated by
    /// the environment-tree allocator.
    pub unsafe fn from_raw_owned(raw: *mut environ) -> Option<Self> {
        NonNull::new(raw).map(|raw| Self { raw })
    }

    /// Borrow the raw pointer for a synchronous C call.
    pub fn as_ptr(&self) -> *mut environ {
        self.raw.as_ptr()
    }

    /// Create a borrowed view whose entries cannot outlive this owner borrow.
    pub fn borrow(&self) -> EnvironView<'_> {
        EnvironView {
            raw: self.raw,
            _owner: PhantomData,
        }
    }

    /// Find an entry by its NUL-terminated byte string name.
    pub fn find<'a>(&'a self, name: &CStr) -> Option<EnvironEntry<'a>> {
        self.borrow().find(name)
    }

    /// Find an entry by bytes, rejecting embedded NUL rather than truncating.
    pub fn find_bytes<'a>(&'a self, name: &[u8]) -> Result<Option<EnvironEntry<'a>>, NulError> {
        let name = CString::new(name)?;
        Ok(self.find(&name))
    }

    /// Set a value using the ordered index and byte-preserving C strings.
    pub fn set_cstr(&mut self, name: &CStr, flags: ::core::ffi::c_int, value: &CStr) {
        unsafe {
            environ_set(
                self.as_ptr(),
                name.as_ptr(),
                flags,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                value.as_ptr(),
            );
        }
    }

    /// Set a value from arbitrary bytes, rejecting embedded NUL bytes.
    pub fn set(
        &mut self,
        name: &[u8],
        flags: ::core::ffi::c_int,
        value: &[u8],
    ) -> Result<(), NulError> {
        let name = CString::new(name)?;
        let value = CString::new(value)?;
        self.set_cstr(&name, flags, &value);
        Ok(())
    }

    /// Record a present but valueless entry.
    pub fn clear_cstr(&mut self, name: &CStr) {
        unsafe { environ_clear(self.as_ptr(), name.as_ptr()) }
    }

    /// Record a present but valueless entry from arbitrary bytes.
    pub fn clear(&mut self, name: &[u8]) -> Result<(), NulError> {
        let name = CString::new(name)?;
        self.clear_cstr(&name);
        Ok(())
    }

    /// Remove an entry completely.
    pub fn unset_cstr(&mut self, name: &CStr) {
        unsafe { environ_unset(self.as_ptr(), name.as_ptr()) }
    }

    /// Copy entries from another borrowed environment view.
    pub fn copy_from(&mut self, source: EnvironView<'_>) {
        unsafe { environ_copy(source.as_ptr(), self.as_ptr()) }
    }

    /// Transfer the C allocation without running `Drop`.
    ///
    /// The returned pointer must be adopted by [`Self::from_raw`] or released
    /// with `environ_free`. This is the explicit hand-off used at C boundaries;
    /// an owner is never placed inside a `calloc`/`free`-managed record.
    pub fn into_raw(self) -> *mut environ {
        let raw = self.raw.as_ptr();
        std::mem::forget(self);
        raw
    }

    /// Alias for [`Self::into_raw`] that makes an ownership hand-off explicit.
    pub fn transfer(self) -> *mut environ {
        self.into_raw()
    }
}

impl Default for EnvironOwner {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for EnvironOwner {
    fn drop(&mut self) {
        unsafe { environ_free(self.raw.as_ptr()) }
    }
}

/// A read-only view of an environment tree tied to an [`EnvironOwner`] borrow.
#[derive(Clone, Copy)]
pub struct EnvironView<'a> {
    raw: NonNull<environ>,
    _owner: PhantomData<&'a EnvironOwner>,
}

impl<'a> EnvironView<'a> {
    /// Borrow the raw pointer for a synchronous C call.
    pub fn as_ptr(self) -> *mut environ {
        self.raw.as_ptr()
    }

    /// Find an entry without converting its bytes to UTF-8.
    pub fn find(self, name: &CStr) -> Option<EnvironEntry<'a>> {
        let entry = unsafe { environ_find(self.as_ptr(), name.as_ptr()) };
        NonNull::new(entry).map(|raw| EnvironEntry {
            raw,
            _owner: PhantomData,
        })
    }

    /// Iterate entries in bytewise name order.
    pub fn entries(self) -> impl Iterator<Item = EnvironEntry<'a>> + 'a {
        unsafe { environ_iter(&*self.as_ptr()) }
    }
}

/// A retained entry pointer whose lifetime is tied to an owner borrow.
#[derive(Clone, Copy)]
pub struct EnvironEntry<'a> {
    raw: NonNull<environ_entry>,
    _owner: PhantomData<&'a EnvironOwner>,
}

impl<'a> EnvironEntry<'a> {
    /// Borrow the original name bytes, including no trailing NUL.
    pub fn name(&self) -> &'a CStr {
        unsafe { CStr::from_ptr(((*self.raw.as_ptr()).name).as_ptr().cast_mut()) }
    }

    /// Borrow the original name bytes without UTF-8 decoding.
    pub fn name_bytes(&self) -> &'a [u8] {
        self.name().to_bytes()
    }

    /// Borrow the value, or return `None` for a present valueless entry.
    pub fn value(&self) -> Option<&'a CStr> {
        unsafe {
            let value = ((*self.raw.as_ptr()).value)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut());
            (!value.is_null()).then(|| CStr::from_ptr(value))
        }
    }

    /// Borrow value bytes without UTF-8 decoding.
    pub fn value_bytes(&self) -> Option<&'a [u8]> {
        self.value().map(CStr::to_bytes)
    }

    /// Return the C environment flags stored on this entry.
    pub fn flags(&self) -> ::core::ffi::c_int {
        unsafe { (*self.raw.as_ptr()).flags }
    }

    /// Borrow the retained C entry pointer for a synchronous C call.
    pub fn as_ptr(&self) -> *mut environ_entry {
        self.raw.as_ptr()
    }
}

/// Iterate entries in bytewise name order.
pub fn environ_iter(env: &environ) -> impl Iterator<Item = EnvironEntry<'_>> + '_ {
    env.entries.values().map(|entry| EnvironEntry {
        raw: NonNull::from(entry.as_ref()),
        _owner: PhantomData,
    })
}

use crate::src::shared::abi::*;
pub use crate::src::shared::environment::ENVIRON_HIDDEN;
pub use crate::src::shared::environment::{environ, environ_entry};
use crate::src::shared::options::{options, options_array_item, options_entry, options_value};
use crate::src::shared::session::session;

unsafe fn environ_insert(
    env: &mut environ,
    name: CString,
    flags: ::core::ffi::c_int,
    value: Option<CString>,
) {
    let key = name.as_bytes().to_vec();
    // Updating an existing entry must preserve its address.
    env.entries
        .entry(key)
        .or_insert_with(|| Box::new(environ_entry { name, value, flags }));
}

pub unsafe fn environ_create() -> *mut environ {
    Box::into_raw(Box::new(environ::default()))
}
pub unsafe fn environ_free(mut env: *mut environ) {
    if env.is_null() {
        return;
    }
    drop(Box::from_raw(env));
}
pub unsafe fn environ_copy(mut srcenv: *mut environ, mut dstenv: *mut environ) {
    if srcenv == dstenv {
        return;
    }
    for entry in environ_iter(&*srcenv) {
        let envent = entry.as_ptr();
        if (*envent).value.is_none() {
            environ_clear(dstenv, ((*envent).name).as_ptr().cast_mut());
        } else {
            environ_set(
                dstenv,
                ((*envent).name).as_ptr().cast_mut(),
                (*envent).flags,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                ((*envent).value)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
            );
        }
    }
}
pub unsafe fn environ_find(
    mut env: *mut environ,
    mut name: *const ::core::ffi::c_char,
) -> *mut environ_entry {
    (*env)
        .entries
        .get(CStr::from_ptr(name).to_bytes())
        .map(|owned| &**owned as *const environ_entry as *mut environ_entry)
        .unwrap_or(std::ptr::null_mut())
}

pub unsafe extern "C" fn environ_set(
    mut env: *mut environ,
    mut name: *const ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
    mut fmt: *const ::core::ffi::c_char,
    mut args: ...
) {
    let mut ap: ::core::ffi::VaList;
    ap = args.clone();
    // Format before replacing the old value: a caller may pass that value as
    // a `%s` argument while updating the same entry.
    let mut value = Some(xvasprintf_cstring(fmt, ap));
    let inserted = {
        if let Some(owned) = (*env).entries.get_mut(CStr::from_ptr(name).to_bytes()) {
            owned.flags = flags;
            owned.value = value.take();
            true
        } else {
            false
        }
    };
    if !inserted {
        environ_insert(&mut *env, CStr::from_ptr(name).to_owned(), flags, value);
    };
}
pub unsafe fn environ_clear(mut env: *mut environ, mut name: *const ::core::ffi::c_char) {
    let inserted = {
        if let Some(owned) = (*env).entries.get_mut(CStr::from_ptr(name).to_bytes()) {
            owned.value = None;
            true
        } else {
            false
        }
    };
    if !inserted {
        environ_insert(&mut *env, CStr::from_ptr(name).to_owned(), 0, None);
    };
}
pub unsafe fn environ_put(
    mut env: *mut environ,
    mut var: *const ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
) {
    let var = CStr::from_ptr(var).to_bytes_with_nul();
    let Some(equals) = var[..var.len() - 1].iter().position(|&byte| byte == b'=') else {
        return;
    };
    let name = CString::new(&var[..equals]).expect("environment name contains no NUL");
    let value = CStr::from_bytes_with_nul(&var[equals + 1..])
        .expect("environment value ends at the input NUL");
    environ_set(
        env,
        name.as_ptr(),
        flags,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        value.as_ptr(),
    );
}
pub unsafe fn environ_unset(mut env: *mut environ, mut name: *const ::core::ffi::c_char) {
    (*env).entries.remove(CStr::from_ptr(name).to_bytes());
}

pub unsafe fn environ_update(mut oo: *mut options, mut src: *mut environ, mut dst: *mut environ) {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut a: *mut options_array_item = ::core::ptr::null_mut::<options_array_item>();
    let mut ov: *mut options_value = ::core::ptr::null_mut::<options_value>();
    let mut found: ::core::ffi::c_int = 0;
    o = options_get(
        oo,
        b"update-environment\0" as *const u8 as *const ::core::ffi::c_char,
    );
    if o.is_null() {
        return;
    }
    a = options_array_first(o);
    while !a.is_null() {
        ov = options_array_item_value(a);
        found = 0 as ::core::ffi::c_int;
        // An aliased destination may change values or add a missing pattern.
        // Refresh each pass so later patterns see those additions.
        let snapshot = (src == dst).then(|| (*src).clone());
        let source = snapshot.as_ref().unwrap_or_else(|| &*src);
        for entry in environ_iter(source) {
            let envent = entry.as_ptr();
            if fnmatch(
                (*ov).string_ptr(),
                ((*envent).name).as_ptr().cast_mut(),
                0 as ::core::ffi::c_int,
            ) == 0 as ::core::ffi::c_int
            {
                environ_set(
                    dst,
                    ((*envent).name).as_ptr().cast_mut(),
                    0 as ::core::ffi::c_int,
                    b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                    ((*envent).value)
                        .as_ref()
                        .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
                );
                found = 1 as ::core::ffi::c_int;
            }
        }
        if found == 0 {
            environ_clear(dst, (*ov).string_ptr());
        }
        a = options_array_next(a);
    }
}
pub unsafe fn environ_push(mut env: *mut environ) {
    let seed = xcalloc(::core::mem::size_of::<*mut ::core::ffi::c_char>() as size_t)
        as *mut *mut ::core::ffi::c_char;
    environ = seed;
    let seed_owner = ProcessEnvironmentSeed(seed);
    for entry in environ_iter(&*env) {
        let envent = entry.as_ptr();
        if !(*envent).value.is_none()
            && *(*envent).name.as_ptr() as ::core::ffi::c_int != '\0' as i32
            && !(*envent).flags & ENVIRON_HIDDEN != 0
        {
            setenv(
                ((*envent).name).as_ptr().cast_mut(),
                ((*envent).value)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
                1 as ::core::ffi::c_int,
            );
        }
    }
    drop(seed_owner);
}
pub unsafe extern "C" fn environ_log(
    mut env: *mut environ,
    mut fmt: *const ::core::ffi::c_char,
    mut args: ...
) {
    let mut ap: ::core::ffi::VaList;
    ap = args.clone();
    let prefix = xvasprintf_cstring(fmt, ap);
    for entry in environ_iter(&*env) {
        let envent = entry.as_ptr();
        if !(*envent).value.is_none()
            && *(*envent).name.as_ptr() as ::core::ffi::c_int != '\0' as i32
        {
            log_debug(format_args!(
                "{}{}={}",
                log_cstr((prefix.as_ptr()) as *const _),
                log_cstr((((*envent).name).as_ptr().cast_mut()) as *const _),
                log_cstr(
                    (((*envent).value)
                        .as_ref()
                        .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                        as *const _
                )
            ));
        }
    }
}
pub unsafe fn environ_for_session(
    mut s: *mut session,
    mut no_TERM: ::core::ffi::c_int,
) -> *mut environ {
    let mut env: *mut environ = ::core::ptr::null_mut::<environ>();
    let mut value: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut idx: ::core::ffi::c_int = 0;
    env = environ_create();
    environ_copy(global_environ, env);
    if !s.is_null() {
        environ_copy((*s).environ, env);
    }
    if no_TERM == 0 {
        value = options_get_string(
            global_options,
            b"default-terminal\0" as *const u8 as *const ::core::ffi::c_char,
        );
        environ_set(
            env,
            b"TERM\0" as *const u8 as *const ::core::ffi::c_char,
            0 as ::core::ffi::c_int,
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            value,
        );
        environ_set(
            env,
            b"TERM_PROGRAM\0" as *const u8 as *const ::core::ffi::c_char,
            0 as ::core::ffi::c_int,
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            b"tmux\0" as *const u8 as *const ::core::ffi::c_char,
        );
        environ_set(
            env,
            b"TERM_PROGRAM_VERSION\0" as *const u8 as *const ::core::ffi::c_char,
            0 as ::core::ffi::c_int,
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            getversion(),
        );
        environ_set(
            env,
            b"COLORTERM\0" as *const u8 as *const ::core::ffi::c_char,
            0 as ::core::ffi::c_int,
            b"truecolor\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    environ_clear(
        env,
        b"LISTEN_PID\0" as *const u8 as *const ::core::ffi::c_char,
    );
    environ_clear(
        env,
        b"LISTEN_FDS\0" as *const u8 as *const ::core::ffi::c_char,
    );
    environ_clear(
        env,
        b"LISTEN_FDNAMES\0" as *const u8 as *const ::core::ffi::c_char,
    );
    if !s.is_null() {
        idx = (*s).id as ::core::ffi::c_int;
    } else {
        idx = -(1 as ::core::ffi::c_int);
    }
    environ_set(
        env,
        b"TMUX\0" as *const u8 as *const ::core::ffi::c_char,
        0 as ::core::ffi::c_int,
        b"%s,%ld,%d\0" as *const u8 as *const ::core::ffi::c_char,
        socket_path,
        getpid() as ::core::ffi::c_long,
        idx,
    );
    return env;
}
